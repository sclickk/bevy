//! Simple benchmark to test rendering many point lights.
//! Run with `WGPU_SETTINGS_PRIO=webgl2` to restrict to uniform buffers and max 256 lights.

use bevy::{
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
	math::{DVec2, DVec3},
	pbr::{ExtractedPointLight, GlobalLightMeta},
	prelude::*,
	render::{camera::ScalingMode, Extract, RenderApp, RenderStage},
	window::PresentMode,
};
use rand::{thread_rng, Rng};

fn main() {
	let mut app = App::new();
	app.insert_resource(WindowDescriptor {
		width: 1024.0,
		height: 768.0,
		title: "many_lights".to_string(),
		present_mode: PresentMode::AutoNoVsync,
		..Default::default()
	});
	app.add_plugins(DefaultPlugins);
	app.add_plugin(FrameTimeDiagnosticsPlugin::default());
	app.add_plugin(LogDiagnosticsPlugin::default());
	app.add_startup_system(setup);
	app.add_system(move_camera);
	app.add_system(print_light_count);
	app.add_plugin(LogVisibleLights);
	app.run();
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	warn!(include_str!("warning_string.txt"));

	const LIGHT_RADIUS: f32 = 0.3;
	const LIGHT_INTENSITY: f32 = 5.0;
	const RADIUS: f32 = 50.0;
	const N_LIGHTS: usize = 100_000;

	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Icosphere {
			radius: RADIUS,
			subdivisions: 9,
		})),
		material: materials.add(StandardMaterial::from(Color::WHITE)),
		transform: Transform::from_scale(Vec3::NEG_ONE),
		..Default::default()
	});

	let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
	let material = materials.add(StandardMaterial {
		base_color: Color::PINK,
		..Default::default()
	});

	// NOTE: This pattern is good for testing performance of culling as it provides roughly
	// the same number of visible meshes regardless of the viewing angle.
	// NOTE: f64 is used to avoid precision issues that produce visual artifacts in the distribution
	let golden_ratio = 0.5f64 * (1.0f64 + 5.0f64.sqrt());
	let mut rng = thread_rng();
	for i in 0..N_LIGHTS {
		let spherical_polar_theta_phi = fibonacci_spiral_on_sphere(golden_ratio, i, N_LIGHTS);
		let unit_sphere_p = spherical_polar_to_cartesian(spherical_polar_theta_phi);
		commands.spawn_bundle(PointLightBundle {
			point_light: PointLight {
				range: LIGHT_RADIUS,
				intensity: LIGHT_INTENSITY,
				color: Color::hsl(rng.gen_range(0.0..360.0), 1.0, 0.5),
				..Default::default()
			},
			transform: Transform::from_translation((RADIUS as f64 * unit_sphere_p).as_vec3()),
			..Default::default()
		});
	}

	// camera
	match std::env::args().nth(1).as_deref() {
		Some("orthographic") => commands.spawn_bundle(Camera3dBundle {
			projection: OrthographicProjection {
				scale: 20.0,
				scaling_mode: ScalingMode::FixedHorizontal(1.0),
				..Default::default()
			}
			.into(),
			..Default::default()
		}),
		_ => commands.spawn_bundle(Camera3dBundle::default()),
	};

	// add one cube, the only one with strong handles
	// also serves as a reference point during rotation
	commands.spawn_bundle(PbrBundle {
		mesh,
		material,
		transform: Transform {
			translation: Vec3::new(0.0, RADIUS as f32, 0.0),
			scale: Vec3::splat(5.0),
			..Default::default()
		},
		..Default::default()
	});
}

// NOTE: This epsilon value is apparently optimal for optimizing for the average
// nearest-neighbor distance. See:
// http://extremelearning.com.au/how-to-evenly-distribute-points-on-a-sphere-more-effectively-than-the-canonical-fibonacci-lattice/
// for details.
const EPSILON: f64 = 0.36;
fn fibonacci_spiral_on_sphere(golden_ratio: f64, i: usize, n: usize) -> DVec2 {
	DVec2::new(
		2.0 * std::f64::consts::PI * (i as f64 / golden_ratio),
		(1.0 - 2.0 * (i as f64 + EPSILON) / (n as f64 - 1.0 + 2.0 * EPSILON)).acos(),
	)
}

fn spherical_polar_to_cartesian(p: DVec2) -> DVec3 {
	let (sin_theta, cos_theta) = p.x.sin_cos();
	let (sin_phi, cos_phi) = p.y.sin_cos();
	DVec3::new(cos_theta * sin_phi, sin_theta * sin_phi, cos_phi)
}

// System for rotating the camera
fn move_camera(time: Res<Time>, mut camera_query: Query<&mut Transform, With<Camera>>) {
	let mut camera_transform = camera_query.single_mut();
	let delta = time.delta_seconds() * 0.15;
	camera_transform.rotate_z(delta);
	camera_transform.rotate_x(delta);
}

// System for printing the number of meshes on every tick of the timer
fn print_light_count(time: Res<Time>, mut timer: Local<PrintingTimer>, lights: Query<&PointLight>) {
	timer.0.tick(time.delta());

	if timer.0.just_finished() {
		info!("Lights: {}", lights.into_iter().len(),);
	}
}

struct LogVisibleLights;

impl Plugin for LogVisibleLights {
	fn build(&self, app: &mut App) {
		if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
			render_app.add_system_to_stage(RenderStage::Extract, extract_time);
			render_app.add_system_to_stage(RenderStage::Prepare, print_visible_light_count);
		};
	}
}

// System for printing the number of meshes on every tick of the timer
fn print_visible_light_count(
	time: Res<ExtractedTime>,
	mut timer: Local<PrintingTimer>,
	visible: Query<&ExtractedPointLight>,
	global_light_meta: Res<GlobalLightMeta>,
) {
	timer.0.tick(time.delta());

	if timer.0.just_finished() {
		info!(
			"Visible Lights: {}, Rendered Lights: {}",
			visible.into_iter().len(),
			global_light_meta.entity_to_index.len()
		);
	}
}

#[derive(Deref, DerefMut)]
pub struct ExtractedTime(Time);

fn extract_time(mut commands: Commands, time: Extract<Res<Time>>) {
	commands.insert_resource(ExtractedTime(time.clone()));
}

struct PrintingTimer(Timer);

impl Default for PrintingTimer {
	fn default() -> Self {
		Self(Timer::from_seconds(1.0, true))
	}
}
