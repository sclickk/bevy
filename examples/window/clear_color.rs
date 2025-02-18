//! Shows how to set the solid color that is used to paint the window before the frame gets drawn.
//!
//! Acts as background color, since pixels that are not drawn in a frame remain unchanged.

use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.9)));
	app.add_plugins(DefaultPlugins);
	app.add_startup_system(setup);
	app.add_system(change_clear_color);
	app.run();
}

fn setup(mut commands: Commands) {
	commands.init_bundle::<Camera2dBundle>();
}

fn change_clear_color(input: Res<Input<KeyCode>>, mut clear_color: ResMut<ClearColor>) {
	if input.just_pressed(KeyCode::Space) {
		clear_color.0 = Color::PURPLE;
	}
}
