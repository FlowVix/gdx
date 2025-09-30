use std::{collections::VecDeque, marker::PhantomData, sync::Arc};

use godot::{
    classes::Node,
    obj::{Gd, Inherits},
};
use parking_lot::Mutex;

use crate::{Context, View, view::AnchorType};

pub struct GDXApp<State, AppView: View<State>, AppFn: FnMut(&mut State) -> AppView> {
    state: State,
    view: Option<(AppView, AppView::ViewState)>,
    app_fn: AppFn,

    root: Gd<Node>,
    ctx: Context,

    _p: PhantomData<AppView>,
}

impl<State, AppView, AppFn> GDXApp<State, AppView, AppFn>
where
    AppView: View<State>,
    AppFn: FnMut(&mut State) -> AppView,
{
    pub fn new<N>(root: Gd<N>, state: State, app_fn: AppFn) -> Self
    where
        N: Inherits<Node>,
    {
        Self {
            state,
            view: None,
            app_fn,
            root: root.upcast::<Node>(),
            ctx: Context {
                id_counter: 0,
                path: vec![],
                msg_queue: Arc::new(Mutex::new(VecDeque::new())),
            },
            _p: PhantomData,
        }
    }
}

pub trait App {
    fn run(&mut self);
}

impl<State, AppView, AppFn> App for GDXApp<State, AppView, AppFn>
where
    AppView: View<State>,
    AppFn: FnMut(&mut State) -> AppView,
{
    fn run(&mut self) {
        if let Some((prev, mut state)) = self.view.take() {
            if !{ self.ctx.msg_queue.lock().is_empty() } {
                while let Some(v) = self.ctx.msg_queue.lock().pop_front() {
                    prev.message(v.msg, &v.path, &mut state, &mut self.state);
                }
            }
            let new = (self.app_fn)(&mut self.state);
            new.rebuild(
                &prev,
                &mut state,
                &mut self.ctx,
                &mut self.root,
                AnchorType::ChildOf,
            );
            self.view = Some((new, state));
        } else {
            let view = (self.app_fn)(&mut self.state);
            let state = view.build(&mut self.ctx, &mut self.root, AnchorType::ChildOf);
            self.view = Some((view, state));
        }
    }
}
