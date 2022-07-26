//! Displays touch presses, releases, and cancels.

use bevy::{input::touch::*, prelude::*};

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_system(touch_system);
	app.run();
}

fn touch_system(touches: Res<Touches>) {
	for touch in touches.iter_just_pressed() {
		info!(
			"just pressed touch with id: {:?}, at: {:?}",
			touch.id(),
			touch.position()
		);
	}

	for touch in touches.iter_just_released() {
		info!(
			"just released touch with id: {:?}, at: {:?}",
			touch.id(),
			touch.position()
		);
	}

	for touch in touches.iter_just_cancelled() {
		info!("cancelled touch with id: {:?}", touch.id());
	}

	// you can also iterate all current touches and retrieve their state like this:
	for touch in touches.iter() {
		info!("active touch: {:?}", touch);
		info!("  just_pressed: {}", touches.just_pressed(touch.id()));
	}
}
