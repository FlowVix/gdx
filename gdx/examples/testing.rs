use gdx::{View, view};
use godot::classes::{Button, Label};

fn blob() -> impl View<(Option<i32>, (f32, f32))> + use<> {
    view! {
        become if (Some(a), b: (f32, f32)) => (a, &mut b.0): (i32, f32) {
            Label
        }
    }
}

fn main() {
    // let v = view! {
    //     become (state: (i32, i32)) => (&mut state.0) {
    //         use (state) {
    //             Button[
    //                 pressed = *state,
    //             ]
    //         }
    //     }
    // };
}
