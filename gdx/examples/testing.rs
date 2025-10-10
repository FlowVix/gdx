use gdx::view;
use godot::classes::Button;

fn main() {
    let v = view! {
        become (state: (i32, i32)) => (&mut state.0) {
            use (state) {
                Button[
                    pressed = *state,
                ]
            }
        }
    };
}
