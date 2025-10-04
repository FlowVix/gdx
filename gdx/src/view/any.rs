use std::any::Any;

use godot::{
    classes::Node,
    obj::{Gd, NewAlloc},
};

use crate::{AnchorType, Context, Message, MessageResult, View, ViewID, view::ArgTuple};

pub trait AnyView<State: ArgTuple> {
    fn as_any(&self) -> &dyn Any;
    fn dyn_build(
        &self,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    ) -> AnyViewState;
    fn dyn_rebuild(
        &self,
        prev: &dyn AnyView<State>,
        state: &mut AnyViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    );
    fn dyn_teardown(
        &self,
        state: &mut AnyViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    );
    fn dyn_message(
        &self,
        msg: Message,
        path: &[ViewID],
        view_state: &mut AnyViewState,
        app_state: &mut State,
    ) -> MessageResult;
    fn collect_nodes(&self, state: &AnyViewState, nodes: &mut Vec<Gd<Node>>);
}

pub struct AnyViewState {
    anchor: Gd<Node>,
    inner: Box<dyn Any>,
    id: ViewID,
}

// MARK: AnyView for View

impl<State: ArgTuple, V> AnyView<State> for V
where
    V: View<State> + 'static,
    V::ViewState: 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_build(
        &self,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    ) -> AnyViewState {
        let mut any_anchor = Node::new_alloc();
        anchor_type.add(anchor, &any_anchor);
        let inner_id = ctx.new_structural_id();

        let inner = ctx.with_id(inner_id, |ctx| {
            self.build(ctx, &mut any_anchor, AnchorType::Before, app_state)
        });
        AnyViewState {
            anchor: any_anchor,
            inner: Box::new(inner),
            id: inner_id,
        }
    }

    fn dyn_rebuild(
        &self,
        prev: &dyn AnyView<State>,
        state: &mut AnyViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    ) {
        let mut any_anchor = state.anchor.clone();
        if let Some(prev) = prev.as_any().downcast_ref::<V>() {
            let inner = state
                .inner
                .downcast_mut::<V::ViewState>()
                .expect("What the hell bro");

            ctx.with_id(state.id, |ctx| {
                self.rebuild(
                    prev,
                    inner,
                    ctx,
                    &mut any_anchor,
                    AnchorType::Before,
                    app_state,
                );
            })
        } else {
            ctx.with_id(state.id, |ctx| {
                prev.dyn_teardown(state, ctx, &mut any_anchor, AnchorType::Before, app_state);
            });
            state.id = ctx.new_structural_id();
            let inner = ctx.with_id(state.id, |ctx| {
                self.build(ctx, &mut any_anchor, AnchorType::Before, app_state)
            });
            state.inner = Box::new(inner);
        }
    }

    fn dyn_teardown(
        &self,
        state: &mut AnyViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    ) {
        let inner = state
            .inner
            .downcast_mut::<V::ViewState>()
            .expect("What the hell bro");
        let mut any_anchor = state.anchor.clone();
        ctx.with_id(state.id, |ctx| {
            self.teardown(inner, ctx, &mut any_anchor, AnchorType::Before, app_state);
        });
    }

    fn dyn_message(
        &self,
        msg: Message,
        path: &[ViewID],
        view_state: &mut AnyViewState,
        app_state: &mut State,
    ) -> MessageResult {
        let inner = view_state
            .inner
            .downcast_mut::<V::ViewState>()
            .expect("What the hell bro");
        if let Some((start, rest)) = path.split_first() {
            if *start == view_state.id {
                self.message(msg, rest, inner, app_state)
            } else {
                MessageResult::Stale(msg)
            }
        } else {
            MessageResult::Stale(msg)
        }
    }

    fn collect_nodes(&self, state: &AnyViewState, nodes: &mut Vec<Gd<Node>>) {
        let inner = state
            .inner
            .downcast_ref::<V::ViewState>()
            .expect("What the hell bro");
        self.collect_nodes(inner, nodes);
    }
}

// MARK: View for dyn AnyView

macro_rules! dyn_anyview_impl {
    ($generic:ident, $($who:tt)*) => {
        impl<$generic: ArgTuple> View<$generic> for $($who)* {
            type ViewState = AnyViewState;

            fn build(
                &self,
                ctx: &mut Context,
                anchor: &mut Node,
                anchor_type: AnchorType,
                app_state: &mut State,
            ) -> Self::ViewState {
                self.dyn_build(ctx, anchor, anchor_type, app_state)
            }

            fn rebuild(
                &self,
                prev: &Self,
                state: &mut Self::ViewState,
                ctx: &mut Context,
                anchor: &mut Node,
                anchor_type: AnchorType,
                app_state: &mut State,
            ) {
                self.dyn_rebuild(prev, state, ctx, anchor, anchor_type, app_state);
            }

            fn teardown(
                &self,
                state: &mut Self::ViewState,
                ctx: &mut Context,
                anchor: &mut Node,
                anchor_type: AnchorType,
                app_state: &mut State,
            ) {
                self.dyn_teardown(state, ctx, anchor, anchor_type, app_state);
            }

            fn message(
                &self,
                msg: Message,
                path: &[ViewID],
                view_state: &mut Self::ViewState,
                app_state: &mut $generic,
            ) -> MessageResult {
                self.dyn_message(msg, path, view_state, app_state)
            }

            fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
                self.collect_nodes(state, nodes);
            }
        }
    };
}

dyn_anyview_impl! { State, dyn AnyView<State> }
dyn_anyview_impl! { State, dyn AnyView<State> + Send }
dyn_anyview_impl! { State, dyn AnyView<State> + Send + Sync }
