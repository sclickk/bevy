use bevy::{
	prelude::*,
	render::settings::{WgpuSettings, WgpuSettingsPriority},
};

// the `bevy_main` proc_macro generates the required android boilerplate
#[bevy_main]
fn main() {
	let mut app = App::new();
	// This configures the app to use the most compatible rendering settings.
	// They help with compatibilty with as many devices as possible.
	app.insert_resource(WgpuSettings {
		priority: WgpuSettingsPriority::Compatibility,
		..Default::default()
	});
	app.add_plugins(DefaultPlugins);
	app.add_startup_system(setup);
	app.run();
}

/// set up a simple 3D scene
fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// plane
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
		material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
		..Default::default()
	});
	// cube
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
		transform: Transform::from_xyz(0.0, 0.5, 0.0),
		..Default::default()
	});
	// light
	commands.spawn_bundle(PointLightBundle {
		transform: Transform::from_xyz(4.0, 8.0, 4.0),
		..Default::default()
	});
	// camera
	commands.spawn_bundle(Camera3dBundle {
		transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
		..Default::default()
	});
}
