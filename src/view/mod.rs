pub mod element;
// pub mod iter;

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

pub struct OptionViewState<InnerViewState> {
    anchor: Gd<Node>,
    state: Option<(InnerViewState, ViewID)>,
}

impl<State, Inner> View<State> for Option<Inner>
where
    Inner: View<State>,
{
    type ViewState = OptionViewState<Inner::ViewState>;

    fn build(
        &self,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState {
        let mut opt_anchor = Node::new_alloc();
        anchor_type.add(anchor, &opt_anchor);
        OptionViewState {
            anchor: opt_anchor.clone(),
            state: self.as_ref().map(|inner| {
                let inner_id = ctx.new_structural_id();
                (
                    ctx.with_id(inner_id, |ctx| {
                        inner.build(ctx, &mut opt_anchor, AnchorType::Before)
                    }),
                    inner_id,
                )
            }),
        }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        assert_eq!(
            prev.is_some(),
            state.state.is_some(),
            "Bruh why are they not the same"
        );
        let mut opt_anchor = state.anchor.clone();
        match (self, prev.as_ref().zip(state.state.as_mut())) {
            (None, None) => {}
            (None, Some((prev, (inner_state, id)))) => {
                ctx.with_id(*id, |ctx| {
                    prev.teardown(inner_state, ctx, &mut opt_anchor, AnchorType::Before);
                });
                state.state = None;
            }
            (Some(new), None) => {
                let inner_id = ctx.new_structural_id();
                state.state = Some((
                    ctx.with_id(inner_id, |ctx| {
                        new.build(ctx, &mut opt_anchor, AnchorType::Before)
                    }),
                    inner_id,
                ));
            }
            (Some(new), Some((prev, (inner_state, id)))) => {
                ctx.with_id(*id, |ctx| {
                    new.rebuild(prev, inner_state, ctx, &mut opt_anchor, AnchorType::Before);
                });
            }
        }
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        assert_eq!(
            self.is_some(),
            state.state.is_some(),
            "Bruh why are they not the same"
        );
        let mut opt_anchor = state.anchor.clone();

        if let Some((inner, (state, id))) = self.as_ref().zip(state.state.as_mut()) {
            ctx.with_id(*id, |ctx| {
                inner.teardown(state, ctx, &mut opt_anchor, AnchorType::Before);
            });
        }
        anchor_type.remove(anchor, &opt_anchor);
        opt_anchor.queue_free();
    }

    fn message(
        &self,
        msg: Message,
        path: &[ViewID],
        view_state: &mut Self::ViewState,
        app_state: &mut State,
    ) -> MessageResult {
        assert_eq!(
            self.is_some(),
            view_state.state.is_some(),
            "Bruh why are they not the same"
        );
        if let Some((start, rest)) = path.split_first() {
            match self.as_ref().zip(view_state.state.as_mut()) {
                Some((inner, (inner_state, child_id))) => {
                    if start == child_id {
                        inner.message(msg, rest, inner_state, app_state)
                    } else {
                        MessageResult::Stale(msg)
                    }
                }
                None => MessageResult::Stale(msg),
            }
        } else {
            MessageResult::Stale(msg)
        }
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
        assert_eq!(
            self.is_some(),
            state.state.is_some(),
            "Bruh why are they not the same"
        );
        if let Some((inner, (state, _))) = self.as_ref().zip(state.state.as_ref()) {
            inner.collect_nodes(state, nodes);
        }
        nodes.push(state.anchor.clone());
    }
}

pub struct VecViewState<InnerViewState> {
    anchor: Gd<Node>,
    state: Vec<InnerViewState>,
}

impl<State, K, Inner> View<State> for Vec<(K, Inner)>
where
    Inner: View<State>,
    K: Hash + Eq,
{
    type ViewState = VecViewState<Inner::ViewState>;

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: super::AnchorType,
    ) -> Self::ViewState {
        let mut opt_anchor = Node::new_alloc();
        anchor_type.add(anchor, &opt_anchor);
        VecViewState {
            anchor: opt_anchor.clone(),
            state: self
                .iter()
                .map(|(k, inner)| {
                    ctx.with_id(ViewID::Key(hash(k)), |ctx| {
                        inner.build(ctx, &mut opt_anchor, AnchorType::Before)
                    })
                })
                .collect(),
        }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: super::AnchorType,
    ) {
        assert_eq!(
            prev.len(),
            state.state.len(),
            "Bruh why are they not the same"
        );
        let mut opt_anchor = state.anchor.clone();
        let mut prev_map = state
            .state
            .drain(..)
            .enumerate()
            .map(|(idx, inner)| {
                let mut nodes = vec![];
                prev[idx].1.collect_nodes(&inner, &mut nodes);
                for i in &nodes {
                    opt_anchor.get_parent().unwrap().remove_child(i);
                }
                (&prev[idx].0, (inner, nodes, &prev[idx].1))
            })
            .collect::<HashMap<_, _>>();

        for (k, v) in self {
            if let Some((mut inner, nodes, prev)) = prev_map.remove(k) {
                for node in &nodes {
                    AnchorType::Before.add(&mut opt_anchor, node);
                }
                ctx.with_id(ViewID::Key(hash(k)), |ctx| {
                    v.rebuild(prev, &mut inner, ctx, &mut opt_anchor, AnchorType::Before);
                });
                state.state.push(inner);
            } else {
                let inner = ctx.with_id(ViewID::Key(hash(k)), |ctx| {
                    v.build(ctx, &mut opt_anchor, AnchorType::Before)
                });
                state.state.push(inner);
            }
        }
        for (k, (mut inner, nodes, prev)) in prev_map.drain() {
            for node in &nodes {
                AnchorType::Before.add(&mut opt_anchor, node);
            }
            ctx.with_id(ViewID::Key(hash(k)), |ctx| {
                prev.teardown(&mut inner, ctx, &mut opt_anchor, AnchorType::Before);
            });
        }
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: super::AnchorType,
    ) {
        assert_eq!(
            self.len(),
            state.state.len(),
            "Bruh why are they not the same"
        );
        let mut opt_anchor = state.anchor.clone();

        for ((k, inner), state) in self.iter().zip(&mut state.state) {
            ctx.with_id(ViewID::Key(hash(k)), |ctx| {
                inner.teardown(state, ctx, &mut opt_anchor, AnchorType::Before);
            });
        }
        anchor_type.remove(anchor, &opt_anchor);
        opt_anchor.queue_free();
    }

    fn message(
        &self,
        msg: crate::Message,
        path: &[ViewID],
        view_state: &mut Self::ViewState,
        app_state: &mut State,
    ) -> crate::MessageResult {
        assert_eq!(
            self.len(),
            view_state.state.len(),
            "Bruh why are they not the same"
        );
        if let Some((start, rest)) = path.split_first() {
            for ((k, inner), state) in self.iter().zip(&mut view_state.state) {
                if *start == ViewID::Key(hash(k)) {
                    return inner.message(msg, rest, state, app_state);
                }
            }
            // if *start == view_state.$v.1 {
            //     return self.$v.message(msg, rest, &mut view_state.$v.0, app_state);
            // }
            MessageResult::Stale(msg)
        } else {
            MessageResult::Stale(msg)
        }
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
        assert_eq!(
            self.len(),
            state.state.len(),
            "Bruh why are they not the same"
        );
        for ((_, inner), state) in self.iter().zip(&state.state) {
            inner.collect_nodes(state, nodes);
        }
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
