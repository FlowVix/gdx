#![deny(unused_must_use, unnameable_types)]
#![allow(clippy::too_many_arguments, clippy::single_match)]

mod app;
mod ctx;
mod util;
mod view;

pub use app::{App, GDXApp};
pub use ctx::{Context, Message, MessageResult};

pub use either;
pub use gdx_macro::view;
pub use view::{
    AnchorType, View, ViewID,
    any::{AnyView, AnyViewState},
    either::EitherViewState,
    element::{
        Element, ElementView, ElementViewState,
        attr::{Attr, AttrViewState},
        el,
        on_mounted::{OnMounted, OnMountedViewState},
        on_signal::{OnSignal, OnSignalViewState},
        theme_override::{
            ThemeOverride, ThemeOverrideColor, ThemeOverrideConstant, ThemeOverrideFont,
            ThemeOverrideFontSize, ThemeOverrideIcon, ThemeOverrideStylebox, ThemeOverrideType,
            ThemeOverrideViewState,
        },
    },
    iter::VecViewState,
    lens::{Lens, lens},
    option::OptionViewState,
};
