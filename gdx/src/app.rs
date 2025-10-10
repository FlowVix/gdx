use std::{collections::VecDeque, marker::PhantomData, sync::Arc};

use gdx_macro::view;
use godot::{
    classes::Node,
    global::godot_print,
    obj::{Gd, Inherits},
};
use parking_lot::Mutex;

use crate::{
    Context, View,
    view::{AnchorType, ArgTuple},
};

pub struct GDXApp<State: ArgTuple, AppView: View<State>, AppFn: FnMut(&mut State) -> AppView> {
    state: State,
    view: Option<(AppView, AppView::ViewState)>,
    app_fn: AppFn,

    root: Gd<Node>,
    ctx: Context,

    _p: PhantomData<AppView>,
}

impl<State: ArgTuple, AppView, AppFn> GDXApp<State, AppView, AppFn>
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
                needs_rebuild: false,
            },
            _p: PhantomData,
        }
    }
}

pub trait App {
    fn run(&mut self);
}

impl<State: ArgTuple, AppView, AppFn> App for GDXApp<State, AppView, AppFn>
where
    AppView: View<State>,
    AppFn: FnMut(&mut State) -> AppView,
{
    fn run(&mut self) {
        if let Some((prev, state)) = &mut self.view {
            if !{ self.ctx.msg_queue.lock().is_empty() } {
                self.ctx.needs_rebuild = true;
                while let Some(v) = self.ctx.msg_queue.lock().pop_front() {
                    prev.message(v.msg, &v.path, state, &mut self.state);
                }
            }
            while self.ctx.needs_rebuild {
                self.ctx.needs_rebuild = false;

                let new = (self.app_fn)(&mut self.state);
                godot_print!("Rebuilding");
                new.rebuild(
                    prev,
                    state,
                    &mut self.ctx,
                    &mut self.root,
                    AnchorType::ChildOf,
                    &mut self.state,
                );
                *prev = new;
            }
        } else {
            let view = (self.app_fn)(&mut self.state);
            godot_print!("Initial build");
            let state = view.build(
                &mut self.ctx,
                &mut self.root,
                AnchorType::ChildOf,
                &mut self.state,
            );
            self.view = Some((view, state));
        }
    }
}
