use std::marker::PhantomData;

use gdx_macro::impl_arg_tuple;
use replace_with::replace_with_or_abort;

use crate::{View, view::ArgTuple};

pub struct MapState<Inner, MapFn, ChildState> {
    inner: Inner,
    map_fn: MapFn,
    _p: PhantomData<ChildState>,
}

impl<ParentState: ArgTuple, ChildState: ArgTuple, Inner, MapFn> View<ParentState>
    for MapState<Inner, MapFn, ChildState>
where
    Inner: View<ChildState>,
    MapFn: Fn(&mut ParentState) -> ChildState::Ref<'_>,
{
    type ViewState = Inner::ViewState;

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
        app_state: &mut ParentState,
    ) -> Self::ViewState {
        ArgTuple::extract_call((self.map_fn)(app_state), |child| {
            self.inner.build(ctx, anchor, anchor_type, child)
        })
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
        app_state: &mut ParentState,
    ) {
        ArgTuple::extract_call((self.map_fn)(app_state), |child| {
            self.inner
                .rebuild(&prev.inner, state, ctx, anchor, anchor_type, child);
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
            self.inner.teardown(state, ctx, anchor, anchor_type, child);
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
            self.inner.message(msg, path, view_state, child)
        })
    }

    fn collect_nodes(
        &self,
        state: &Self::ViewState,
        nodes: &mut Vec<godot::prelude::Gd<godot::prelude::Node>>,
    ) {
        self.inner.collect_nodes(state, nodes);
    }
}

pub fn map<ParentState: ArgTuple, ChildState: ArgTuple, MapFn, Inner>(
    map_fn: MapFn,
    view: Inner,
) -> MapState<Inner, MapFn, ChildState>
where
    MapFn: Fn(&mut ParentState) -> ChildState::Ref<'_>,
    Inner: View<ChildState>,
{
    MapState {
        inner: view,
        map_fn,
        _p: PhantomData,
    }
}
