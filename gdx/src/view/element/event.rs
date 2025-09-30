use std::sync::Arc;

use godot::{
    builtin::{Callable, Variant},
    classes::Node,
    meta::ToGodot,
};

use crate::{
    AnchorType, Attr, ElementView, Message, MessageResult, View, ViewID, ctx::FullMessage,
    view::element::impl_element_view,
};

pub struct Event<Name, Cb, Inner> {
    pub(crate) inner: Inner,
    pub(crate) name: Name,
    pub(crate) cb: Cb,
}

pub struct EventViewState<InnerViewState> {
    callable: Callable,
    inner_view_state: InnerViewState,
}

impl<State, Name, Cb, Inner> View<State> for Event<Name, Cb, Inner>
where
    Inner: ElementView<State>,
    Name: AsRef<str> + Clone,
    Cb: Fn(&mut State, &[Variant]),
{
    type ViewState = EventViewState<Inner::ViewState>;

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) -> Self::ViewState {
        let inner_view_state = self.inner.build(ctx, anchor, anchor_type);
        let mut node = self.inner.get_node(&inner_view_state);

        let msgs = ctx.msg_queue.clone();
        let path: Arc<[ViewID]> = ctx.path.clone().into();
        let name: Arc<str> = self.name.as_ref().into();
        let callable = Callable::from_fn("boing", move |args| {
            msgs.lock().push_back(FullMessage {
                msg: Message::Event {
                    name: name.clone(),
                    args: args.iter().map(|v| (**v).clone()).collect(),
                },
                path: path.clone(),
            });
        });

        node.connect(self.name.as_ref(), &callable);
        EventViewState {
            callable,
            inner_view_state,
        }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        self.inner.rebuild(
            &prev.inner,
            &mut state.inner_view_state,
            ctx,
            anchor,
            anchor_type,
        );
        let mut node = self.get_node(state);

        node.disconnect(prev.name.as_ref(), &state.callable);

        let msgs = ctx.msg_queue.clone();
        let path: Arc<[ViewID]> = ctx.path.clone().into();
        let name: Arc<str> = self.name.as_ref().into();
        let callable = Callable::from_fn("boing", move |args| {
            msgs.lock().push_back(FullMessage {
                msg: Message::Event {
                    name: name.clone(),
                    args: args.iter().map(|v| (**v).clone()).collect(),
                },
                path: path.clone(),
            });
        });

        node.connect(self.name.as_ref(), &callable);
        state.callable = callable;
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
    ) {
        self.inner
            .teardown(&mut state.inner_view_state, ctx, anchor, anchor_type);
    }

    fn message(
        &self,
        msg: crate::Message,
        path: &[crate::ViewID],
        view_state: &mut Self::ViewState,
        app_state: &mut State,
    ) -> MessageResult {
        if path.is_empty() {
            match msg {
                Message::Event { ref name, ref args } => {
                    if **name == *self.name.as_ref() {
                        (self.cb)(app_state, args);
                        MessageResult::Action
                    } else {
                        self.inner
                            .message(msg, path, &mut view_state.inner_view_state, app_state)
                    }
                }
            }
        } else {
            self.inner
                .message(msg, path, &mut view_state.inner_view_state, app_state)
        }
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<godot::prelude::Gd<Node>>) {
        self.inner.collect_nodes(&state.inner_view_state, nodes);
    }
}

impl<State, Name, Cb, Inner> ElementView<State> for Event<Name, Cb, Inner>
where
    Inner: ElementView<State>,
    Name: AsRef<str> + Clone,
    Cb: Fn(&mut State, &[Variant]),
{
    fn get_node(&self, state: &Self::ViewState) -> godot::prelude::Gd<Node> {
        self.inner.get_node(&state.inner_view_state)
    }
}

impl<Name0, Cb0, Inner> Event<Name0, Cb0, Inner> {
    impl_element_view! {}
}
