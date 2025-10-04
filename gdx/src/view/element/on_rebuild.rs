use std::marker::PhantomData;

use godot::{builtin::Variant, classes::Node, meta::ToGodot, obj::Inherits, prelude::Gd};

use crate::{
    AnchorType, ElementView, Message, MessageResult, View,
    ctx::FullMessage,
    view::{ArgTuple, element::impl_element_view},
};

pub struct OnRebuild<N, Cb, Inner> {
    pub(crate) inner: Inner,
    pub(crate) cb: Cb,
    pub(crate) _p: PhantomData<N>,
}

pub struct OnRebuildViewState<InnerViewState> {
    inner_view_state: InnerViewState,
}

impl<N, State: ArgTuple, Cb, Inner> View<State> for OnRebuild<N, Cb, Inner>
where
    Inner: ElementView<N, State>,
    Cb: Fn(Gd<N>),
    N: Inherits<Node>,
{
    type ViewState = OnRebuildViewState<Inner::ViewState>;

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    ) -> Self::ViewState {
        let inner_view_state = self.inner.build(ctx, anchor, anchor_type, app_state);

        OnRebuildViewState { inner_view_state }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    ) {
        let node = prev.inner.get_node(&state.inner_view_state);
        (self.cb)(node);

        self.inner.rebuild(
            &prev.inner,
            &mut state.inner_view_state,
            ctx,
            anchor,
            anchor_type,
            app_state,
        );
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    ) {
        self.inner.teardown(
            &mut state.inner_view_state,
            ctx,
            anchor,
            anchor_type,
            app_state,
        );
    }

    fn message(
        &self,
        msg: crate::Message,
        path: &[crate::ViewID],
        view_state: &mut Self::ViewState,
        app_state: &mut State,
    ) -> MessageResult {
        self.inner
            .message(msg, path, &mut view_state.inner_view_state, app_state)
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
        self.inner.collect_nodes(&state.inner_view_state, nodes);
    }
}

impl<N, State: ArgTuple, Cb, Inner> ElementView<N, State> for OnRebuild<N, Cb, Inner>
where
    Inner: ElementView<N, State>,
    Cb: Fn(Gd<N>),
    N: Inherits<Node>,
{
    fn get_node(&self, state: &Self::ViewState) -> Gd<N> {
        self.inner.get_node(&state.inner_view_state)
    }
}

impl<N, Cb0, Inner> OnRebuild<N, Cb0, Inner> {
    impl_element_view! { N }
}
