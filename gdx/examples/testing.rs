use gdx::{View, view};
use godot::classes::{Button, Label};

fn blob() -> impl View<(Option<i32>, (f32, f32))> + use<> {
    view! {
        become (a, b: (f32, f32)) => (a, &mut b.0) {

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
