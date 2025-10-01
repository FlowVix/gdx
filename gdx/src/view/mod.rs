pub mod either;
pub mod element;
pub mod iter;
pub mod lens;
pub mod option;

use std::ops::Deref;
use std::{collections::HashMap, hash::Hash};

use godot::{
    classes::Node,
    meta::AsArg,
    obj::{Gd, NewAlloc},
};

use crate::ctx::{Context, Message, MessageResult};
use crate::util::hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewID {
    Structural(u64),
    Key(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnchorType {
    ChildOf,
    Before,
}
impl AnchorType {
    pub fn add(self, anchor: &mut Node, node: &Gd<Node>) {
        match self {
            AnchorType::ChildOf => anchor.add_child(node),
            AnchorType::Before => {
                let idx = anchor.get_index();
                let mut parent = anchor.get_parent().unwrap();
                parent.add_child(node);
                parent.move_child(node, idx);
            }
        }
    }
    pub fn remove(self, anchor: &mut Node, node: &Gd<Node>) {
        match self {
            AnchorType::ChildOf => anchor.remove_child(node),
            AnchorType::Before => anchor.get_parent().unwrap().remove_child(node),
        }
    }
}

pub trait View<State> {
    type ViewState;

    fn build(
        &self,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState;
    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    );
    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    );
    fn message(
        &self,
        msg: Message,
        path: &[ViewID],
        view_state: &mut Self::ViewState,
        app_state: &mut State,
    ) -> MessageResult;

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>);
}

impl<State, Inner> View<State> for Box<Inner>
where
    Inner: View<State>,
{
    type ViewState = Inner::ViewState;

    fn build(
        &self,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState {
        self.deref().build(ctx, anchor, anchor_type)
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        self.deref().rebuild(prev, state, ctx, anchor, anchor_type);
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        self.deref().teardown(state, ctx, anchor, anchor_type);
    }

    fn message(
        &self,
        msg: Message,
        path: &[ViewID],
        view_state: &mut Self::ViewState,
        app_state: &mut State,
    ) -> MessageResult {
        self.deref().message(msg, path, view_state, app_state)
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
        self.deref().collect_nodes(state, nodes);
    }
}

macro_rules! tuple_impl {
    ($($v:literal)*) => {
        paste::paste! {
            impl<State, $( [< V $v >] ,)*> View<State> for ($( [< V $v >] ,)*) where $( [< V $v >] : View<State>,)* {
                type ViewState = ($( ([<V $v>]::ViewState, ViewID), )*);

                #[allow(clippy::unused_unit)]
                #[allow(unused_variables)]
                fn build(&self, ctx: &mut Context, anchor: &mut Node, anchor_type: AnchorType) -> Self::ViewState {
                    (
                        $(
                            {
                                let child_id = ctx.new_structural_id();
                                (ctx.with_id(child_id, |ctx| {
                                    self.$v.build(ctx, anchor, anchor_type)
                                }), child_id)
                            },
                        )*
                    )
                }
                #[allow(unused_variables)]
                fn rebuild(
                    &self,
                    prev: &Self,
                    state: &mut Self::ViewState,
                    ctx: &mut Context,
                    anchor: &mut Node, anchor_type: AnchorType,
                ) {
                    $(
                        ctx.with_id(state.$v.1, |ctx| {
                            self.$v.rebuild(&prev.$v, &mut state.$v.0, ctx, anchor, anchor_type);
                        });
                    )*
                }
                #[allow(unused_variables)]
                fn teardown(&self, state: &mut Self::ViewState, ctx: &mut Context, anchor: &mut Node, anchor_type: AnchorType) {
                    $(
                        ctx.with_id(state.$v.1, |ctx| {
                            self.$v.teardown(&mut state.$v.0, ctx, anchor, anchor_type);
                        });
                    )*
                }

                #[allow(unused_variables)]
                fn message(
                    &self,
                    msg: Message,
                    path: &[ViewID],
                    view_state: &mut Self::ViewState,
                    app_state: &mut State,
                ) -> MessageResult {
                    if let Some((start, rest)) = path.split_first() {
                        $(
                            if *start == view_state.$v.1 {
                                return self.$v.message(msg, rest, &mut view_state.$v.0, app_state);
                            }
                        )*
                        MessageResult::Stale(msg)
                    } else {
                        MessageResult::Stale(msg)
                    }
                }

                #[allow(unused_variables)]
                fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
                    $(
                        self.$v.collect_nodes(&state.$v.0, nodes);
                    )*
                }
            }
        }
    };
}

tuple_impl! {}
tuple_impl! { 0 }
tuple_impl! { 0 1 }
tuple_impl! { 0 1 2 }
tuple_impl! { 0 1 2 3 }
tuple_impl! { 0 1 2 3 4 }
tuple_impl! { 0 1 2 3 4 5 }
tuple_impl! { 0 1 2 3 4 5 6 }
tuple_impl! { 0 1 2 3 4 5 6 7 }
tuple_impl! { 0 1 2 3 4 5 6 7 8 }
tuple_impl! { 0 1 2 3 4 5 6 7 8 9 }
tuple_impl! { 0 1 2 3 4 5 6 7 8 9 10 }
