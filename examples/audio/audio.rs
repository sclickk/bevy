//! This example illustrates how to load and play an audio file.

use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_startup_system(setup);
	app.run();
}

fn setup(asset_server: Res<AssetServer>, audio: Res<Audio>) {
	let music = asset_server.load("sounds/Windless Slopes.ogg");
	audio.play(music);
}
