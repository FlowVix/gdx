use godot::{
    classes::Node,
    obj::{Gd, NewAlloc},
};

use crate::{AnchorType, Context, Message, MessageResult, View, ViewID};

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

        if let Some((val, (inner, id))) = self.as_ref().zip(state.state.as_mut()) {
            ctx.with_id(*id, |ctx| {
                val.teardown(inner, ctx, &mut opt_anchor, AnchorType::Before);
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
                Some((val, (inner, child_id))) => {
                    if start == child_id {
                        val.message(msg, rest, inner, app_state)
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
        if let Some((val, (inner, _))) = self.as_ref().zip(state.state.as_ref()) {
            val.collect_nodes(inner, nodes);
        }
        nodes.push(state.anchor.clone());
    }
}
