use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(Counter(0))
        .add_system(keyboard_input_system)
        .add_plugins(DefaultPlugins)
        .run();
}

fn keyboard_input_system(keyboard_input: Res<Input<KeyCode>>, mut counter: ResMut<Counter>) {
    if keyboard_input.just_pressed(KeyCode::W) {
        let before = counter.0;
        counter.0 += 1;
        info!("Counter incremented from {} to {}", before, counter.0)
    }

    if keyboard_input.just_pressed(KeyCode::S) {
        let before = counter.0;
        // Gotta watch for that underflow...
        if before > 0 {
            counter.0 -= 1;
            info!("Counter decremented from {} to {}", before, counter.0)
        }
    }

    if keyboard_input.just_released(KeyCode::Space) {
        let before = counter.0;
        counter.0 = 0;
        info!("Counter reset from {} to {}", before, counter.0)
    }

    if keyboard_input.pressed(KeyCode::LControl) {
        info!("Left CTRL is being held down...")
    }

    if keyboard_input.any_just_pressed(vec![KeyCode::C, KeyCode::V]) {
        info!("Either C or V was just pressed.")
    }
}

#[derive(Resource)]
struct Counter(u32);
