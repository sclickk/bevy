//! Uses two windows to visualize a 3D model from different angles.

use bevy::{
	prelude::*,
	render::camera::RenderTarget,
	window::{CreateWindow, PresentMode, WindowId},
};

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_startup_system(setup);
	app.add_system(bevy::window::close_on_esc);
	app.run();
}

fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut create_window_events: EventWriter<CreateWindow>,
) {
	// add entities to the world
	commands.spawn_bundle(SceneBundle {
		scene: asset_server.load("models/monkey/Monkey.gltf#Scene0"),
		..Default::default()
	});
	// light
	commands.spawn_bundle(PointLightBundle {
		transform: Transform::from_xyz(4.0, 5.0, 4.0),
		..Default::default()
	});
	// main camera
	commands.spawn_bundle(Camera3dBundle {
		transform: Transform::from_xyz(0.0, 0.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
		..Default::default()
	});

	let window_id = WindowId::new();

	// sends out a "CreateWindow" event, which will be received by the windowing backend
	create_window_events.send(CreateWindow {
		id: window_id,
		descriptor: WindowDescriptor {
			width: 800.,
			height: 600.,
			present_mode: PresentMode::AutoNoVsync,
			title: "Second window".to_string(),
			..Default::default()
		},
	});

	// second window camera
	commands.spawn_bundle(Camera3dBundle {
		transform: Transform::from_xyz(6.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
		camera: Camera {
			target: RenderTarget::Window(window_id),
			..Default::default()
		},
		..Default::default()
	});
}
