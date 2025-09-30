use godot::{
    classes::Node,
    obj::{Gd, NewAlloc},
};
use std::{collections::HashMap, hash::Hash};

use crate::{AnchorType, Context, Message, MessageResult, View, ViewID, util::hash};

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
