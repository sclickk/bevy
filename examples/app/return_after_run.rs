//! Shows how to return to the calling function after a windowed Bevy app has exited.

use bevy::{prelude::*, winit::WinitSettings};

fn main() {
	println!("Running first App.");
	App::new()
		.insert_resource(WinitSettings {
			return_from_run: true,
			..Default::default()
		})
		.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.8)))
		.add_plugins(DefaultPlugins)
		.add_system(system1)
		.run();
	println!("Running another App.");
	App::new()
		.insert_resource(WinitSettings {
			return_from_run: true,
			..Default::default()
		})
		.insert_resource(ClearColor(Color::rgb(0.2, 0.8, 0.2)))
		.add_plugins_with(DefaultPlugins, |group| {
			group.disable::<bevy::log::LogPlugin>();
			group
		})
		.add_system(system2)
		.run();
	println!("Done.");
}

fn system1() {
	info!("logging from first app");
}

fn system2() {
	info!("logging from second app");
}
