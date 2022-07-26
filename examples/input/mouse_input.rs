//! Prints mouse button events.

use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_system(mouse_click_system);
	app.run();
}

// This system prints messages when you press or release the left mouse button:
fn mouse_click_system(mouse_button_input: Res<Input<MouseButton>>) {
	if mouse_button_input.pressed(MouseButton::Left) {
		info!("left mouse currently pressed");
	}

	if mouse_button_input.just_pressed(MouseButton::Left) {
		info!("left mouse just pressed");
	}

	if mouse_button_input.just_released(MouseButton::Left) {
		info!("left mouse just released");
	}
}
