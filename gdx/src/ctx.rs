use std::{collections::VecDeque, rc::Rc, sync::Arc};

use godot::builtin::Variant;
use parking_lot::Mutex;

use crate::view::ViewID;

pub struct Context {
    pub(crate) id_counter: u64,
    pub(crate) path: Vec<ViewID>,

    pub(crate) msg_queue: Arc<Mutex<VecDeque<FullMessage>>>,
}

impl Context {
    pub(crate) fn new_structural_id(&mut self) -> ViewID {
        let out = ViewID::Structural(self.id_counter);
        self.id_counter += 1;
        out
    }
    pub(crate) fn with_id<R>(&mut self, id: ViewID, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.push(id);
        let out = f(self);
        self.path.pop();
        out
    }
}

pub struct FullMessage {
    pub(crate) msg: Message,
    pub(crate) path: Arc<[ViewID]>,
}
pub enum Message {
    Signal {
        name: Arc<str>,
        args: Box<[Variant]>,
    },
    Mounted,
}
pub enum MessageResult {
    Success,
    // Nop,
    Stale(Message),
}
