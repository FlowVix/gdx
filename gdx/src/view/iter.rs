use godot::{
    classes::Node,
    obj::{Gd, NewAlloc},
};
use std::{collections::HashMap, hash::Hash};

use crate::{
    AnchorType, Context, Message, MessageResult, View, ViewID, util::hash, view::ArgTuple,
};

pub struct VecViewState<InnerViewState> {
    anchor: Gd<Node>,
    inner: Vec<InnerViewState>,
}

impl<State: ArgTuple, K, Inner> View<State> for Vec<(K, Inner)>
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
        let mut vec_anchor = Node::new_alloc();
        anchor_type.add(anchor, &vec_anchor);
        VecViewState {
            anchor: vec_anchor.clone(),
            inner: self
                .iter()
                .map(|(k, inner)| {
                    ctx.with_id(ViewID::Key(hash(k)), |ctx| {
                        inner.build(ctx, &mut vec_anchor, AnchorType::Before)
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
            state.inner.len(),
            "Bruh why are they not the same"
        );
        let mut vec_anchor = state.anchor.clone();

        let mut total_nodes = 0;

        let mut prev_map = state
            .inner
            .drain(..)
            .enumerate()
            .map(|(idx, inner)| {
                let mut nodes = vec![];
                prev[idx].1.collect_nodes(&inner, &mut nodes);
                total_nodes += nodes.len();
                (&prev[idx].0, (inner, nodes, &prev[idx].1))
            })
            .collect::<HashMap<_, _>>();

        let mut move_idx = vec_anchor.get_index() as usize - total_nodes;
        for (k, v) in self {
            if let Some((mut inner, nodes, prev)) = prev_map.remove(k) {
                for node in &nodes {
                    node.get_parent().unwrap().move_child(node, move_idx as i32);
                    move_idx += 1;
                }
                ctx.with_id(ViewID::Key(hash(k)), |ctx| {
                    v.rebuild(prev, &mut inner, ctx, &mut vec_anchor, AnchorType::Before);
                });
                state.inner.push(inner);
            } else {
                let inner = ctx.with_id(ViewID::Key(hash(k)), |ctx| {
                    v.build(ctx, &mut vec_anchor, AnchorType::Before)
                });
                let mut nodes = vec![];
                v.collect_nodes(&inner, &mut nodes);
                for node in &nodes {
                    node.get_parent().unwrap().move_child(node, move_idx as i32);
                    move_idx += 1;
                }
                state.inner.push(inner);
            }
        }
        for (k, (mut inner, _, prev)) in prev_map.drain() {
            ctx.with_id(ViewID::Key(hash(k)), |ctx| {
                prev.teardown(&mut inner, ctx, &mut vec_anchor, AnchorType::Before);
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
            state.inner.len(),
            "Bruh why are they not the same"
        );
        let mut vec_anchor = state.anchor.clone();

        for ((k, inner), state) in self.iter().zip(&mut state.inner) {
            ctx.with_id(ViewID::Key(hash(k)), |ctx| {
                inner.teardown(state, ctx, &mut vec_anchor, AnchorType::Before);
            });
        }
        anchor_type.remove(anchor, &vec_anchor);
        vec_anchor.queue_free();
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
            view_state.inner.len(),
            "Bruh why are they not the same"
        );
        if let Some((start, rest)) = path.split_first() {
            for ((k, inner), state) in self.iter().zip(&mut view_state.inner) {
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
            state.inner.len(),
            "Bruh why are they not the same"
        );
        for ((_, inner), state) in self.iter().zip(&state.inner) {
            inner.collect_nodes(state, nodes);
        }
        nodes.push(state.anchor.clone());
    }
}
