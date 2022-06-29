//! Shows how to return to the calling function after a windowed Bevy app has exited.

use bevy::{prelude::*, winit::WinitSettings};

fn main() {
	println!("Running first App.");
	let mut first_app = App::new();
	first_app.insert_resource(WinitSettings {
			return_from_run: true,
			..Default::default()
		});
	first_app.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.8)));
	first_app.add_plugins(DefaultPlugins);
	first_app.add_system(system1);
	first_app.run();
	println!("Running another App.");
	let mut second_app = App::new();
	second_app.insert_resource(WinitSettings {
			return_from_run: true,
			..Default::default()
		});
	second_app.insert_resource(ClearColor(Color::rgb(0.2, 0.8, 0.2)));
	second_app.add_plugins_with(DefaultPlugins, |group| {
			group.disable::<bevy::log::LogPlugin>();
			group
		});
	second_app.add_system(system2);
	second_app.run();
	println!("Done.");
}

fn system1() {
	info!("logging from first app");
}

fn system2() {
	info!("logging from second app");
}
