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
    AnchorType, ArgTuple, View, ViewID,
    any::{AnyView, AnyViewState},
    either::EitherViewState,
    element::{
        Element, ElementView, ElementViewState,
        attr::{Attr, AttrViewState},
        el,
        on_build::{OnBuild, OnBuildViewState},
        on_mounted::{OnMounted, OnMountedViewState},
        on_rebuild::{OnRebuild, OnRebuildViewState},
        on_signal::{OnSignal, OnSignalViewState},
        on_teardown::{OnTeardown, OnTeardownViewState},
        theme_override::{
            ThemeOverride, ThemeOverrideColor, ThemeOverrideConstant, ThemeOverrideFont,
            ThemeOverrideFontSize, ThemeOverrideIcon, ThemeOverrideStylebox, ThemeOverrideType,
            ThemeOverrideViewState,
        },
    },
    iter::VecViewState,
    map::{MapState, map},
    option::OptionViewState,
    proxy::{MessageProxy, Proxy, proxy},
    using::{Using, using},
};
