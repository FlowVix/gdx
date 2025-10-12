use std::marker::PhantomData;

use crate::{ArgTuple, View};

pub struct Using<State, InnerFn> {
    inner_fn: InnerFn,
    _p: PhantomData<State>,
}

impl<State: ArgTuple, InnerFn, Inner> View<State> for Using<State, InnerFn>
where
    InnerFn: Fn(&mut State) -> Inner,
    Inner: View<State>,
{
    type ViewState = (Inner, Inner::ViewState);

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
        app_state: &mut State,
    ) -> Self::ViewState {
        let inner = (self.inner_fn)(app_state);
        let vstate = inner.build(ctx, anchor, anchor_type, app_state);
        (inner, vstate)
    }

    fn rebuild(
        &self,
        _prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
        app_state: &mut State,
    ) {
        let inner = (self.inner_fn)(app_state);
        inner.rebuild(&state.0, &mut state.1, ctx, anchor, anchor_type, app_state);
        state.0 = inner;
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
        app_state: &mut State,
    ) {
        state
            .0
            .teardown(&mut state.1, ctx, anchor, anchor_type, app_state);
    }

    fn message(
        &self,
        msg: crate::Message,
        path: &[super::ViewID],
        view_state: &mut Self::ViewState,
        app_state: &mut State,
    ) -> crate::MessageResult {
        view_state
            .0
            .message(msg, path, &mut view_state.1, app_state)
    }

    fn collect_nodes(
        &self,
        state: &Self::ViewState,
        nodes: &mut Vec<godot::prelude::Gd<godot::prelude::Node>>,
    ) {
        state.0.collect_nodes(&state.1, nodes);
    }
}

pub fn using<State: ArgTuple, InnerFn, Inner>(inner_fn: InnerFn) -> Using<State, InnerFn>
where
    Inner: View<State>,
    InnerFn: Fn(&mut State) -> Inner,
{
    Using {
        inner_fn,
        _p: PhantomData,
    }
}
