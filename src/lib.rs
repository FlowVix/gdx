#![deny(unused_must_use, unnameable_types)]
#![allow(clippy::too_many_arguments)]

mod app;
mod ctx;
mod util;
mod view;

pub use app::{App, GDXApp};
pub use ctx::{Context, Message, MessageResult};

pub use view::{
    AnchorType, OptionViewState, VecViewState, View, ViewID,
    element::{
        Element, ElementView, ElementViewState,
        attributes::{Attr, AttrViewState},
        el,
        event::{Event, EventViewState},
    },
};
