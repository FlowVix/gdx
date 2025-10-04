use godot::{
    builtin::{Color, StringName, Variant},
    classes::{Control, Font, Node, StyleBox, Texture2D},
    meta::{AsArg, ToGodot},
    obj::Inherits,
    prelude::Gd,
};
use std::marker::PhantomData;

use crate::{
    ElementView, ViewID,
    ctx::{Message, MessageResult},
    view::{AnchorType, ArgTuple, View, element::impl_element_view},
};

macro_rules! theme_override_types {
    (
        $(
            $(($ref:tt))? $name:ident: $typ:ty,
        )*
    ) => {
        paste::paste! {
            $(
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                pub struct [< ThemeOverride $name:camel >];

                impl ThemeOverrideType for [< ThemeOverride $name:camel >] {
                    type ValueType = $typ;

                    fn set(node: &mut Control, name: impl AsArg<StringName>, value: Self::ValueType) {
                        node.[< add_theme_ $name _override >](name, $($ref)? value);
                    }
                    fn remove(node: &mut Control, name: impl AsArg<StringName>) {
                        node.[< remove_theme_ $name _override >](name);
                    }
                }
            )*
        }
    };
}
pub trait ThemeOverrideType: Copy {
    type ValueType: Clone + PartialEq;
    fn set(node: &mut Control, name: impl AsArg<StringName>, value: Self::ValueType);
    fn remove(node: &mut Control, name: impl AsArg<StringName>);
}
theme_override_types! {
    color: Color,
    constant: i32,
    (&)font: Gd<Font>,
    font_size: i32,
    (&)icon: Gd<Texture2D>,
    (&)stylebox: Gd<StyleBox>,
}

pub struct ThemeOverride<N, Typ: ThemeOverrideType, Name, Inner> {
    pub(crate) inner: Inner,
    pub(crate) name: Name,
    pub(crate) value: Typ::ValueType,
    pub(crate) _p: PhantomData<(N, Typ)>,
}

pub struct ThemeOverrideViewState<InnerViewState> {
    inner_view_state: InnerViewState,
}

impl<State: ArgTuple, N, Typ, Name, Inner> View<State> for ThemeOverride<N, Typ, Name, Inner>
where
    Inner: ElementView<N, State>,
    Typ: ThemeOverrideType,
    Name: AsRef<str> + Clone,
    N: Inherits<Control> + Inherits<Node>,
{
    type ViewState = ThemeOverrideViewState<Inner::ViewState>;

    fn build(
        &self,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    ) -> Self::ViewState {
        let inner_view_state = self.inner.build(ctx, anchor, anchor_type, app_state);
        let mut node = self.inner.get_node(&inner_view_state);
        Typ::set(node.upcast_mut(), self.name.as_ref(), self.value.clone());
        ThemeOverrideViewState { inner_view_state }
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    ) {
        self.inner.rebuild(
            &prev.inner,
            &mut state.inner_view_state,
            ctx,
            anchor,
            anchor_type,
            app_state,
        );

        let mut node = self.get_node(state);
        if self.name.as_ref() != prev.name.as_ref() {
            Typ::remove(node.upcast_mut(), prev.name.as_ref());
        }
        Typ::set(node.upcast_mut(), self.name.as_ref(), self.value.clone());
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
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
        msg: Message,
        path: &[ViewID],
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

impl<State: ArgTuple, N, Typ, Name, Inner> ElementView<N, State>
    for ThemeOverride<N, Typ, Name, Inner>
where
    Inner: ElementView<N, State>,
    Typ: ThemeOverrideType,
    Name: AsRef<str> + Clone,
    N: Inherits<Control> + Inherits<Node>,
{
    fn get_node(&self, state: &Self::ViewState) -> Gd<N> {
        self.inner.get_node(&state.inner_view_state)
    }
}

impl<N, Typ0: ThemeOverrideType, Name0, Inner> ThemeOverride<N, Typ0, Name0, Inner> {
    impl_element_view! { N }
}
