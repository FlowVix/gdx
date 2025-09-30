pub mod attributes;
pub mod event;

use std::marker::PhantomData;

use godot::{
    builtin::Variant,
    classes::Node,
    meta::ToGodot,
    obj::{Gd, Inherits, NewAlloc},
};

use crate::{
    Attr, Event,
    ctx::{Message, MessageResult},
    view::{AnchorType, View, ViewID},
};

pub struct Element<N, Children> {
    children: Children,
    _p: PhantomData<N>,
}

pub fn el<N: Inherits<Node> + NewAlloc>() -> Element<N, ()> {
    Element {
        children: (),
        _p: PhantomData,
    }
}
impl<N, Children> Element<N, Children> {
    pub fn children<NewChildren>(self, children: NewChildren) -> Element<N, NewChildren> {
        Element {
            children,
            _p: PhantomData,
        }
    }
}

pub struct ElementViewState<N: Inherits<Node>, ChildViewState> {
    node: Gd<N>,
    child_id: ViewID,
    child_view_state: ChildViewState,
}

impl<State, N, Children> View<State> for Element<N, Children>
where
    N: Inherits<Node> + NewAlloc,
    Children: View<State>,
{
    type ViewState = ElementViewState<N, Children::ViewState>;

    fn build(
        &self,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState {
        let mut node = N::new_alloc();
        anchor_type.add(anchor, &node.clone().upcast::<Node>());

        let child_id = ctx.new_structural_id();
        let child_view_state = ctx.with_id(child_id, |ctx| {
            self.children
                .build(ctx, node.upcast_mut::<Node>(), AnchorType::ChildOf)
        });

        ElementViewState {
            node,
            child_id,
            child_view_state,
        }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        ctx.with_id(state.child_id, |ctx| {
            self.children.rebuild(
                &prev.children,
                &mut state.child_view_state,
                ctx,
                state.node.upcast_mut::<Node>(),
                AnchorType::ChildOf,
            );
        })
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        ctx.with_id(state.child_id, |ctx| {
            self.children.teardown(
                &mut state.child_view_state,
                ctx,
                state.node.upcast_mut(),
                AnchorType::ChildOf,
            );
        });

        anchor_type.remove(anchor, &state.node.clone().upcast());
        state.node.upcast_mut::<Node>().queue_free();
    }

    fn message(
        &self,
        msg: Message,
        path: &[ViewID],
        view_state: &mut Self::ViewState,
        app_state: &mut State,
    ) -> MessageResult {
        if let Some((start, rest)) = path.split_first() {
            if *start == view_state.child_id {
                self.children
                    .message(msg, rest, &mut view_state.child_view_state, app_state)
            } else {
                MessageResult::Stale(msg)
            }
        } else {
            MessageResult::Stale(msg)
        }
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
        nodes.push(state.node.clone().upcast::<Node>());
    }
}

pub trait ElementView<State>: View<State> + Sized {
    fn get_node(&self, state: &Self::ViewState) -> Gd<Node>;
}

impl<State, N, Children> ElementView<State> for Element<N, Children>
where
    N: Inherits<Node> + NewAlloc,
    Children: View<State>,
{
    fn get_node(&self, state: &Self::ViewState) -> Gd<Node> {
        state.node.clone().upcast()
    }
}

// doing this instead of the trait because rust was smelly
macro_rules! impl_element_view {
    () => {
        pub fn attr<Name, Value>(self, name: Name, value: Value) -> Attr<Name, Self>
        where
            Name: AsRef<str>,
            Value: ToGodot,
        {
            Attr {
                inner: self,
                name,
                value: value.to_variant(),
            }
        }
        pub fn on<State, Name, Cb>(self, name: Name, cb: Cb) -> Event<Name, Cb, Self>
        where
            Name: AsRef<str>,
            Cb: Fn(&mut State, &[Variant]),
        {
            Event {
                inner: self,
                name,
                cb,
            }
        }
    };
}
pub(crate) use impl_element_view;

impl<N, Children> Element<N, Children> {
    impl_element_view! {}
}
