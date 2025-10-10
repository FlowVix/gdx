use std::{marker::PhantomData, sync::Arc};

use crate::{
    ArgTuple, Message, MessageResult, View, ViewID,
    ctx::{FullMessage, MsgQueue},
};

#[derive(Clone)]
pub struct MessageProxy<T> {
    queue: MsgQueue,
    path: Arc<[ViewID]>,
    _p: PhantomData<T>,
}
impl<T: 'static> MessageProxy<T> {
    pub fn send(&self, value: T) {
        self.queue.lock().push_back(FullMessage {
            msg: Message::Proxy {
                value: Box::new(value),
            },
            path: self.path.clone(),
        });
    }
}

pub struct Proxy<State, T, InnerFn, Cb> {
    inner_fn: InnerFn,
    cb: Cb,
    _p: PhantomData<(State, T)>,
}

impl<State: ArgTuple, T: 'static, InnerFn, Cb, Inner> View<State> for Proxy<State, T, InnerFn, Cb>
where
    Inner: View<State>,
    InnerFn: Fn(MessageProxy<T>) -> Inner,
    Cb: Fn(&mut State, Box<T>),
{
    type ViewState = (Inner, Inner::ViewState);

    fn build(
        &self,
        ctx: &mut crate::Context,
        anchor: &mut godot::prelude::Node,
        anchor_type: super::AnchorType,
        app_state: &mut State,
    ) -> Self::ViewState {
        let proxy = MessageProxy {
            queue: ctx.msg_queue.clone(),
            path: ctx.path.clone().into(),
            _p: PhantomData,
        };
        let inner = (self.inner_fn)(proxy);
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
        let proxy = MessageProxy {
            queue: ctx.msg_queue.clone(),
            path: ctx.path.clone().into(),
            _p: PhantomData,
        };
        let inner = (self.inner_fn)(proxy);
        inner.rebuild(&state.0, &mut state.1, ctx, anchor, anchor_type, app_state);
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
        if path.is_empty() {
            match msg {
                Message::Proxy { value } => {
                    (self.cb)(app_state, value.downcast().unwrap());
                    return MessageResult::Success;
                }
                _ => {}
            }
        }
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

pub fn proxy<State: ArgTuple, T, InnerFn, Cb, Inner>(
    cb: Cb,
    inner_fn: InnerFn,
) -> Proxy<State, T, InnerFn, Cb>
where
    Inner: View<State>,
    InnerFn: Fn(MessageProxy<T>) -> Inner,
    Cb: Fn(&mut State, Box<T>),
{
    Proxy {
        inner_fn,
        cb,
        _p: PhantomData,
    }
}
