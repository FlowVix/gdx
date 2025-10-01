use crate::View;

pub struct Lens<Inner, Map> {
    inner: Inner,
    map_fn: Map,
}

impl<ParentState, ChildState, Inner, Map> View<ParentState> for Lens<Inner, Map>
where
    Inner: View<ChildState>,
    Map: Fn(&mut ParentState) -> &mut ChildState,
{
    type ViewState = Inner::ViewState;

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) -> Self::ViewState {
        self.inner.build(ctx, anchor, anchor_type)
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        self.inner
            .rebuild(&prev.inner, state, ctx, anchor, anchor_type);
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
    ) {
        self.inner.teardown(state, ctx, anchor, anchor_type);
    }

    fn message(
        &self,
        msg: crate::Message,
        path: &[super::ViewID],
        view_state: &mut Self::ViewState,
        app_state: &mut ParentState,
    ) -> crate::MessageResult {
        self.inner
            .message(msg, path, view_state, (self.map_fn)(app_state))
    }

    fn collect_nodes(
        &self,
        state: &Self::ViewState,
        nodes: &mut Vec<godot::prelude::Gd<godot::prelude::Node>>,
    ) {
        self.inner.collect_nodes(state, nodes);
    }
}

pub fn lens<ParentState, ChildState, MapFn, ViewFn, Inner>(
    state: &mut ParentState,
    map_fn: MapFn,
    view_fn: ViewFn,
) -> Lens<Inner, MapFn>
where
    MapFn: Fn(&mut ParentState) -> &mut ChildState,
    ViewFn: FnOnce(&mut ChildState) -> Inner,
    Inner: View<ChildState>,
{
    let child_state = map_fn(state);
    let inner = view_fn(child_state);
    Lens { inner, map_fn }
}
