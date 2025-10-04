pub mod attr;
pub mod on_build;
pub mod on_mounted;
pub mod on_rebuild;
pub mod on_signal;
pub mod on_teardown;
pub mod theme_override;

use std::marker::PhantomData;

use godot::{
    builtin::Variant,
    classes::Node,
    global::godot_print,
    meta::ToGodot,
    obj::{Gd, Inherits, NewAlloc},
};

use crate::{
    ctx::{Message, MessageResult},
    view::{AnchorType, ArgTuple, View, ViewID},
};

pub struct Element<N, Children> {
    children: Children,
    _p: PhantomData<N>,
}

pub fn el<N: Inherits<Node> + NewAlloc>() -> Element<N, ()> {
    Element {
        children: (),
        _p: PhantomData,
    }
}
impl<N, Children> Element<N, Children> {
    pub fn children<NewChildren>(self, children: NewChildren) -> Element<N, NewChildren> {
        Element {
            children,
            _p: PhantomData,
        }
    }
}

pub struct ElementViewState<N: Inherits<Node>, ChildViewState> {
    node: Gd<N>,
    child_id: ViewID,
    child_view_state: ChildViewState,
}

impl<State: ArgTuple, N, Children> View<State> for Element<N, Children>
where
    N: Inherits<Node> + NewAlloc,
    Children: View<State>,
{
    type ViewState = ElementViewState<N, Children::ViewState>;

    fn build(
        &self,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    ) -> Self::ViewState {
        let mut node = N::new_alloc();
        anchor_type.add(anchor, &node.clone().upcast::<Node>());

        let child_id = ctx.new_structural_id();
        let child_view_state = ctx.with_id(child_id, |ctx| {
            self.children.build(
                ctx,
                node.upcast_mut::<Node>(),
                AnchorType::ChildOf,
                app_state,
            )
        });

        ElementViewState {
            node,
            child_id,
            child_view_state,
        }
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
        ctx.with_id(state.child_id, |ctx| {
            self.children.rebuild(
                &prev.children,
                &mut state.child_view_state,
                ctx,
                state.node.upcast_mut::<Node>(),
                AnchorType::ChildOf,
                app_state,
            );
        })
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut crate::ctx::Context,
        anchor: &mut Node,
        anchor_type: AnchorType,
        app_state: &mut State,
    ) {
        ctx.with_id(state.child_id, |ctx| {
            self.children.teardown(
                &mut state.child_view_state,
                ctx,
                state.node.upcast_mut(),
                AnchorType::ChildOf,
                app_state,
            );
        });

        anchor_type.remove(anchor, &state.node.clone().upcast());
        state.node.upcast_mut::<Node>().queue_free();
    }

    fn message(
        &self,
        msg: Message,
        path: &[ViewID],
        view_state: &mut Self::ViewState,
        app_state: &mut State,
    ) -> MessageResult {
        if let Some((start, rest)) = path.split_first() {
            if *start == view_state.child_id {
                self.children
                    .message(msg, rest, &mut view_state.child_view_state, app_state)
            } else {
                MessageResult::Stale(msg)
            }
        } else {
            MessageResult::Stale(msg)
        }
    }

    fn collect_nodes(&self, state: &Self::ViewState, nodes: &mut Vec<Gd<Node>>) {
        nodes.push(state.node.clone().upcast::<Node>());
    }
}

pub trait ElementView<N: Inherits<Node>, State: ArgTuple>: View<State> + Sized {
    fn get_node(&self, state: &Self::ViewState) -> Gd<N>;
}

impl<State: ArgTuple, N, Children> ElementView<N, State> for Element<N, Children>
where
    N: Inherits<Node> + NewAlloc,
    Children: View<State>,
{
    fn get_node(&self, state: &Self::ViewState) -> Gd<N> {
        state.node.clone()
    }
}

// doing this instead of the trait because rust was smelly
macro_rules! impl_element_view {
    ($node:ident) => {
        pub fn attr<Name, Value>(self, name: Name, value: Value) -> $crate::Attr<$node, Name, Self>
        where
            Name: AsRef<str>,
            Value: ToGodot,
            $node: godot::prelude::Inherits<godot::prelude::Node>,
        {
            use std::marker::PhantomData;
            $crate::Attr {
                inner: self,
                name,
                value: value.to_variant(),
                _p: PhantomData,
            }
        }
        pub fn on_signal<State, Name, Cb>(
            self,
            name: Name,
            cb: Cb,
        ) -> $crate::OnSignal<$node, Name, Cb, Self>
        where
            Name: AsRef<str>,
            Cb: Fn(&mut State, &[Variant], Gd<$node>),
            $node: godot::prelude::Inherits<godot::prelude::Node>,
        {
            use std::marker::PhantomData;
            $crate::OnSignal {
                inner: self,
                name,
                cb,
                _p: PhantomData,
            }
        }
        pub fn on_mounted<State, Cb>(self, cb: Cb) -> $crate::OnMounted<$node, Cb, Self>
        where
            Cb: Fn(&mut State, Gd<$node>),
            $node: godot::prelude::Inherits<godot::prelude::Node>,
        {
            use std::marker::PhantomData;
            $crate::OnMounted {
                inner: self,
                cb,
                _p: PhantomData,
            }
        }
        pub fn on_build<Cb>(self, cb: Cb) -> $crate::OnBuild<$node, Cb, Self>
        where
            Cb: Fn(Gd<$node>),
            $node: godot::prelude::Inherits<godot::prelude::Node>,
        {
            use std::marker::PhantomData;
            $crate::OnBuild {
                inner: self,
                cb,
                _p: PhantomData,
            }
        }
        pub fn on_rebuild<Cb>(self, cb: Cb) -> $crate::OnRebuild<$node, Cb, Self>
        where
            Cb: Fn(Gd<$node>),
            $node: godot::prelude::Inherits<godot::prelude::Node>,
        {
            use std::marker::PhantomData;
            $crate::OnRebuild {
                inner: self,
                cb,
                _p: PhantomData,
            }
        }
        pub fn on_teardown<Cb>(self, cb: Cb) -> $crate::OnTeardown<$node, Cb, Self>
        where
            Cb: Fn(Gd<$node>),
            $node: godot::prelude::Inherits<godot::prelude::Node>,
        {
            use std::marker::PhantomData;
            $crate::OnTeardown {
                inner: self,
                cb,
                _p: PhantomData,
            }
        }
        pub fn theme_override<Typ: crate::ThemeOverrideType, Name>(
            self,
            name: Name,
            value: Typ::ValueType,
        ) -> $crate::ThemeOverride<$node, Typ, Name, Self>
        where
            Name: AsRef<str>,
            $node: godot::prelude::Inherits<godot::prelude::Node>,
        {
            use std::marker::PhantomData;
            $crate::ThemeOverride {
                inner: self,
                name,
                value,
                _p: PhantomData,
            }
        }
    };
}
pub(crate) use impl_element_view;

impl<N, Children> Element<N, Children> {
    impl_element_view! { N }
}
