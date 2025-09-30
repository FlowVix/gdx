// use std::hash::Hash;

// use godot::{classes::Node, obj::Gd};

// use crate::{View, ViewID};

// pub struct ForEach<I, KeyCb, ViewCb> {
//     iter: I,
//     key_cb: KeyCb,
//     view_cb: ViewCb,
// }

// pub struct ForEachViewState<InnerViewState> {
//     anchor: Gd<Node>,
//     state: Vec<(InnerViewState, ViewID)>,
// }

// impl<State, I, KeyCb, ViewCb, Item, ItemView, Key> View<State> for ForEach<I, KeyCb, ViewCb>
// where
//     I: IntoIterator<Item = Item>,
//     KeyCb: FnMut(&Item) -> Key,
//     Key: Hash,
//     ViewCb: FnMut(Item) -> ItemView,
//     ItemView: View<State>,
// {
//     type ViewState = ();

//     fn build(
//         &self,
//         ctx: &mut crate::Context,
//         anchor: &mut godot::prelude::Node,
//         anchor_type: super::AnchorType,
//     ) -> Self::ViewState {
//         todo!()
//     }

//     fn rebuild(
//         &self,
//         prev: &Self,
//         state: &mut Self::ViewState,
//         ctx: &mut crate::Context,
//         anchor: &mut godot::prelude::Node,
//         anchor_type: super::AnchorType,
//     ) {
//         todo!()
//     }

//     fn teardown(
//         &self,
//         state: &mut Self::ViewState,
//         ctx: &mut crate::Context,
//         anchor: &mut godot::prelude::Node,
//         anchor_type: super::AnchorType,
//     ) {
//         todo!()
//     }

//     fn message(
//         &self,
//         msg: crate::Message,
//         path: &[super::ViewID],
//         view_state: &mut Self::ViewState,
//         app_state: &mut State,
//     ) -> crate::MessageResult {
//         todo!()
//     }

//     fn collect_nodes(
//         &self,
//         state: &Self::ViewState,
//         nodes: &mut Vec<godot::prelude::Gd<godot::prelude::Node>>,
//     ) {
//         todo!()
//     }
// }
