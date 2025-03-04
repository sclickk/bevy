//! Loads and renders a glTF file as a scene.

use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.insert_resource(AmbientLight {
		color: Color::WHITE,
		brightness: 1.0 / 5.0f32,
	});
	app.add_plugins(DefaultPlugins);
	app.add_startup_system(setup);
	app.add_system(animate_light_direction);
	app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn_bundle(Camera3dBundle {
		transform: Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
		..Default::default()
	});
	const HALF_SIZE: f32 = 1.0;
	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			shadow_projection: OrthographicProjection {
				left: -HALF_SIZE,
				right: HALF_SIZE,
				bottom: -HALF_SIZE,
				top: HALF_SIZE,
				near: -10.0 * HALF_SIZE,
				far: 10.0 * HALF_SIZE,
				..Default::default()
			},
			shadows_enabled: true,
			..Default::default()
		},
		..Default::default()
	});
	commands.spawn_bundle(SceneBundle {
		scene: asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"),
		..Default::default()
	});
}

fn animate_light_direction(
	time: Res<Time>,
	mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
	for mut transform in &mut query {
		transform.rotation = Quat::from_euler(
			EulerRot::ZYX,
			0.0,
			time.seconds_since_startup() as f32 * std::f32::consts::TAU / 10.0,
			-std::f32::consts::FRAC_PI_4,
		);
	}
}
