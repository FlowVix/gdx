use std::marker::PhantomData;

use gdx_macro::impl_arg_tuple;
use replace_with::replace_with_or_abort;

use crate::{View, view::ArgTuple};

pub struct Lens<InnerFn, MapFn, ChildState> {
    inner_fn: InnerFn,
    map_fn: MapFn,
    _p: PhantomData<ChildState>,
}

impl<ParentState: ArgTuple, ChildState: ArgTuple, Inner, InnerFn, MapFn> View<ParentState>
    for Lens<InnerFn, MapFn, ChildState>
where
    Inner: View<ChildState>,
    InnerFn: Fn(&mut ChildState) -> Inner,
    MapFn: Fn(&mut ParentState) -> ChildState::Ref<'_>,
{
    type ViewState = (Inner, Inner::ViewState);

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
        app_state: &mut ParentState,
    ) -> Self::ViewState {
        ArgTuple::extract_call((self.map_fn)(app_state), |child| {
            let child_comp = (self.inner_fn)(child);
            let state = child_comp.build(ctx, anchor, anchor_type, child);
            (child_comp, state)
        })
    }

    fn rebuild(
        &self,
        _prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
        app_state: &mut ParentState,
    ) {
        ArgTuple::extract_call((self.map_fn)(app_state), |child| {
            let child_comp = (self.inner_fn)(child);
            child_comp.rebuild(&state.0, &mut state.1, ctx, anchor, anchor_type, child);
        })
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
        app_state: &mut ParentState,
    ) {
        ArgTuple::extract_call((self.map_fn)(app_state), |child| {
            state
                .0
                .teardown(&mut state.1, ctx, anchor, anchor_type, child);
        })
    }

    fn message(
        &self,
        msg: crate::Message,
        path: &[super::ViewID],
        view_state: &mut Self::ViewState,
        app_state: &mut ParentState,
    ) -> crate::MessageResult {
        ArgTuple::extract_call((self.map_fn)(app_state), |child| {
            view_state.0.message(msg, path, &mut view_state.1, child)
        })
    }

    fn collect_nodes(
        &self,
        state: &Self::ViewState,
        nodes: &mut Vec<godot::prelude::Gd<godot::prelude::Node>>,
    ) {
        state.0.collect_nodes(&state.1, nodes);
    }
}

pub fn lens<ParentState: ArgTuple, ChildState: ArgTuple, MapFn, InnerFn, Inner>(
    map_fn: MapFn,
    inner_fn: InnerFn,
) -> Lens<InnerFn, MapFn, ChildState>
where
    MapFn: Fn(&mut ParentState) -> ChildState::Ref<'_>,
    InnerFn: Fn(&mut ChildState) -> Inner,
    Inner: View<ChildState>,
{
    Lens {
        inner_fn,
        map_fn,
        _p: PhantomData,
    }
}
