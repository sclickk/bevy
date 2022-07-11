//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::WindowMode;

fn main() {
	let mut app = App::new();
	app.insert_resource(Msaa { samples: 4 });
	app.init_plugin::<LogDiagnosticsPlugin>();
	app.init_plugin::<FrameTimeDiagnosticsPlugin>();
	app.init_resource::<CameraTracker>();
	app.add_plugins(DefaultPlugins);
	app.add_startup_system(setup_camera);
	app.add_startup_system(setup_scene);
	app.add_system(toggle_fullscreen);
	app.add_system(camera_controller);
	app.add_system(camera_tracker);
	app.run();
}

fn setup_camera(mut commands: Commands) {
	// camera
	commands
		.spawn_bundle(Camera3dBundle {
			transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
			..Default::default()
		})
		.insert(CameraController::default());
}

/// set up a simple 3D scene
fn setup_scene(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// ground plane
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
		material: materials.add(StandardMaterial {
			base_color: Color::WHITE,
			perceptual_roughness: 1.0,
			..Default::default()
		}),
		..Default::default()
	});

	// left wall
	let mut transform = Transform::from_xyz(2.5, 2.5, 0.0);
	transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Box::new(5.0, 0.15, 5.0))),
		transform,
		material: materials.add(StandardMaterial {
			base_color: Color::INDIGO,
			perceptual_roughness: 1.0,
			..Default::default()
		}),
		..Default::default()
	});
	// back (right) wall
	let mut transform = Transform::from_xyz(0.0, 2.5, -2.5);
	transform.rotate(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Box::new(5.0, 0.15, 5.0))),
		transform,
		material: materials.add(StandardMaterial {
			base_color: Color::INDIGO,
			perceptual_roughness: 1.0,
			..Default::default()
		}),
		..Default::default()
	});

	// cube
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
		material: materials.add(StandardMaterial {
			base_color: Color::PINK,
			..Default::default()
		}),
		transform: Transform::from_xyz(0.0, 0.5, 0.0),
		..Default::default()
	});
	// .insert(Movable);
	// sphere
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::UVSphere {
			radius: 0.5,
			..Default::default()
		})),
		material: materials.add(StandardMaterial {
			base_color: Color::LIME_GREEN,
			..Default::default()
		}),
		transform: Transform::from_xyz(1.5, 1.0, 1.5),
		..Default::default()
	});
	// .insert(Movable);

	// ambient light
	commands.insert_resource(AmbientLight {
		color: Color::ORANGE_RED,
		brightness: 0.02,
	});

	// red point light
	commands
		.spawn_bundle(PointLightBundle {
			// transform: Transform::from_xyz(5.0, 8.0, 2.0),
			transform: Transform::from_xyz(1.0, 2.0, 0.0),
			point_light: PointLight {
				intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
				color: Color::RED,
				shadows_enabled: true,
				..Default::default()
			},
			..Default::default()
		})
		.with_children(|builder| {
			builder.spawn_bundle(PbrBundle {
				mesh: meshes.add(Mesh::from(shape::UVSphere {
					radius: 0.1,
					..Default::default()
				})),
				material: materials.add(StandardMaterial {
					base_color: Color::RED,
					emissive: Color::rgba_linear(100.0, 0.0, 0.0, 0.0),
					..Default::default()
				}),
				..Default::default()
			});
		});

	// green point light
	commands
		.spawn_bundle(PointLightBundle {
			// transform: Transform::from_xyz(5.0, 8.0, 2.0),
			transform: Transform::from_xyz(-1.0, 2.0, 0.0),
			point_light: PointLight {
				intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
				color: Color::GREEN,
				shadows_enabled: true,
				..Default::default()
			},
			..Default::default()
		})
		.with_children(|builder| {
			builder.spawn_bundle(PbrBundle {
				mesh: meshes.add(Mesh::from(shape::UVSphere {
					radius: 0.1,
					..Default::default()
				})),
				material: materials.add(StandardMaterial {
					base_color: Color::GREEN,
					emissive: Color::rgba_linear(0.0, 100.0, 0.0, 0.0),
					..Default::default()
				}),
				..Default::default()
			});
		});

	// blue point light
	commands
		.spawn_bundle(PointLightBundle {
			// transform: Transform::from_xyz(5.0, 8.0, 2.0),
			transform: Transform::from_xyz(0.0, 4.0, 0.0),
			point_light: PointLight {
				intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
				color: Color::BLUE,
				shadows_enabled: true,
				..Default::default()
			},
			..Default::default()
		})
		.with_children(|builder| {
			builder.spawn_bundle(PbrBundle {
				mesh: meshes.add(Mesh::from(shape::UVSphere {
					radius: 0.1,
					..Default::default()
				})),
				material: materials.add(StandardMaterial {
					base_color: Color::BLUE,
					emissive: Color::rgba_linear(0.0, 0.0, 100.0, 0.0),
					..Default::default()
				}),
				..Default::default()
			});
		});

	// directional 'sun' light
	const HALF_SIZE: f32 = 10.0;
	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			// Configure the projection to better fit the scene
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
		transform: Transform {
			translation: Vec3::new(0.0, 2.0, 0.0),
			rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
			..Default::default()
		},
		..Default::default()
	});
}

fn toggle_fullscreen(input: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
	if input.just_pressed(KeyCode::F11) {
		let window = windows.primary_mut();
		window.set_mode(match window.mode() {
			WindowMode::Windowed => WindowMode::Fullscreen,
			_ => WindowMode::Windowed,
		});
		// window.set_cursor_lock_mode(!window.cursor_locked());
		// window.set_cursor_visibility(!window.cursor_visible());
	}
}

#[derive(Default)]
struct CameraTracker {
	active_index: Option<usize>,
	cameras: Vec<Entity>,
}

impl CameraTracker {
	fn track_camera(&mut self, entity: Entity) -> bool {
		self.cameras.push(entity);
		if self.active_index.is_none() {
			self.active_index = Some(self.cameras.len() - 1);
			true
		} else {
			false
		}
	}

	fn active_camera(&self) -> Option<Entity> {
		self.active_index.map(|i| self.cameras[i])
	}

	fn set_next_active(&mut self) -> Option<Entity> {
		let active_index = self.active_index?;
		let new_i = (active_index + 1) % self.cameras.len();
		self.active_index = Some(new_i);
		Some(self.cameras[new_i])
	}
}

fn camera_tracker(
	mut camera_tracker: ResMut<CameraTracker>,
	keyboard_input: Res<Input<KeyCode>>,
	mut queries: ParamSet<(
		Query<(Entity, &mut Camera), (Added<Camera>, Without<CameraController>)>,
		Query<(Entity, &mut Camera), (Added<Camera>, With<CameraController>)>,
		Query<&mut Camera>,
	)>,
) {
	// track added scene camera entities first, to ensure they are preferred for the
	// default active camera
	for (entity, mut camera) in queries.p0().iter_mut() {
		camera.is_active = camera_tracker.track_camera(entity);
	}

	// iterate added custom camera entities second
	for (entity, mut camera) in queries.p1().iter_mut() {
		camera.is_active = camera_tracker.track_camera(entity);
	}

	if keyboard_input.just_pressed(KeyCode::C) {
		// disable currently active camera
		if let Some(e) = camera_tracker.active_camera() {
			if let Ok(mut camera) = queries.p2().get_mut(e) {
				camera.is_active = false;
			}
		}

		// enable next active camera
		if let Some(e) = camera_tracker.set_next_active() {
			if let Ok(mut camera) = queries.p2().get_mut(e) {
				camera.is_active = true;
			}
		}
	}
}

#[derive(Component)]
struct CameraController {
	pub enabled: bool,
	pub initialized: bool,
	pub sensitivity: f32,
	pub key_forward: KeyCode,
	pub key_back: KeyCode,
	pub key_left: KeyCode,
	pub key_right: KeyCode,
	pub key_up: KeyCode,
	pub key_down: KeyCode,
	pub key_run: KeyCode,
	pub mouse_key_enable_mouse: MouseButton,
	pub keyboard_key_enable_mouse: KeyCode,
	pub walk_speed: f32,
	pub run_speed: f32,
	pub friction: f32,
	pub pitch: f32,
	pub yaw: f32,
	pub velocity: Vec3,
}

impl Default for CameraController {
	fn default() -> Self {
		Self {
			enabled: true,
			initialized: false,
			sensitivity: 0.1,
			key_forward: KeyCode::W,
			key_back: KeyCode::S,
			key_left: KeyCode::A,
			key_right: KeyCode::D,
			key_up: KeyCode::E,
			key_down: KeyCode::Q,
			key_run: KeyCode::LShift,
			mouse_key_enable_mouse: MouseButton::Left,
			keyboard_key_enable_mouse: KeyCode::M,
			walk_speed: 5.0,
			run_speed: 15.0,
			friction: 0.5,
			pitch: 0.0,
			yaw: 0.0,
			velocity: Vec3::ZERO,
		}
	}
}

fn camera_controller(
	time: Res<Time>,
	mut mouse_events: EventReader<MouseMotion>,
	mouse_button_input: Res<Input<MouseButton>>,
	key_input: Res<Input<KeyCode>>,
	mut move_toggled: Local<bool>,
	mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
	let dt = time.delta_seconds();

	if let Ok((mut transform, mut options)) = query.get_single_mut() {
		if !options.initialized {
			let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
			options.yaw = yaw;
			options.pitch = pitch;
			options.initialized = true;
		}
		if !options.enabled {
			return;
		}

		// Handle key input
		let mut axis_input = Vec3::ZERO;
		if key_input.pressed(options.key_forward) {
			axis_input.z += 1.0;
		}
		if key_input.pressed(options.key_back) {
			axis_input.z -= 1.0;
		}
		if key_input.pressed(options.key_right) {
			axis_input.x += 1.0;
		}
		if key_input.pressed(options.key_left) {
			axis_input.x -= 1.0;
		}
		if key_input.pressed(options.key_up) {
			axis_input.y += 1.0;
		}
		if key_input.pressed(options.key_down) {
			axis_input.y -= 1.0;
		}
		if key_input.just_pressed(options.keyboard_key_enable_mouse) {
			*move_toggled = !*move_toggled;
		}

		// Apply movement update
		if axis_input != Vec3::ZERO {
			let max_speed = if key_input.pressed(options.key_run) {
				options.run_speed
			} else {
				options.walk_speed
			};
			options.velocity = axis_input.normalize() * max_speed;
		} else {
			let friction = options.friction.clamp(0.0, 1.0);
			options.velocity *= 1.0 - friction;
			if options.velocity.length_squared() < 1e-6 {
				options.velocity = Vec3::ZERO;
			}
		}
		let forward = transform.forward();
		let right = transform.right();
		transform.translation += options.velocity.x * dt * right
			+ options.velocity.y * dt * Vec3::Y
			+ options.velocity.z * dt * forward;

		// Handle mouse input
		let mut mouse_delta = Vec2::ZERO;
		if mouse_button_input.pressed(options.mouse_key_enable_mouse) || *move_toggled {
			for mouse_event in mouse_events.iter() {
				mouse_delta += mouse_event.delta;
			}
		}

		if mouse_delta != Vec2::ZERO {
			// Apply look update
			let (pitch, yaw) = (
				(options.pitch - mouse_delta.y * 0.5 * options.sensitivity * dt).clamp(
					-0.99 * std::f32::consts::FRAC_PI_2,
					0.99 * std::f32::consts::FRAC_PI_2,
				),
				options.yaw - mouse_delta.x * options.sensitivity * dt,
			);
			transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, yaw, pitch);
			options.pitch = pitch;
			options.yaw = yaw;
		}
	}
}
