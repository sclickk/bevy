//! This example illustrates how to use [`States`] to control transitioning from a `Menu` state to
//! an `InGame` state.

use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_state(AppState::Menu);
	app.add_startup_system(setup);
	app.add_system_set(SystemSet::when_enter(AppState::Menu, setup_menu));
	app.add_system_set(SystemSet::when_update(AppState::Menu, menu));
	app.add_system_set(SystemSet::when_exit(AppState::Menu, cleanup_menu));
	app.add_system_set(SystemSet::when_enter(AppState::InGame, setup_game));
	app.add_system_set(
		SystemSet::on_update(AppState::InGame)
			.with_system(movement)
			.with_system(change_color),
	);
	app.run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
	Menu,
	InGame,
}

struct MenuData {
	button_entity: Entity,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands) {
	commands.init_bundle::<Camera2dBundle>();
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
	let button_entity = commands
		.spawn_bundle(ButtonBundle {
			style: Style {
				size: Size::new(Val::Px(150.0), Val::Px(65.0)),
				// center button
				margin: UiRect::all(Val::Auto),
				// horizontally center child text
				justify_content: JustifyContent::Center,
				// vertically center child text
				align_items: AlignItems::Center,
				..Default::default()
			},
			color: NORMAL_BUTTON.into(),
			..Default::default()
		})
		.with_children(|parent| {
			parent.spawn_bundle(TextBundle::from_section(
				"Play",
				TextStyle {
					font: asset_server.load("fonts/FiraSans-Bold.ttf"),
					font_size: 40.0,
					color: Color::rgb(0.9, 0.9, 0.9),
				},
			));
		})
		.id();
	commands.insert_resource(MenuData { button_entity });
}

fn menu(
	mut state: ResMut<State<AppState>>,
	mut interaction_query: Query<(&Interaction, &mut UiColor), (Changed<Interaction>, With<Button>)>,
) {
	for (interaction, mut color) in &mut interaction_query {
		match *interaction {
			Interaction::Clicked => {
				*color = PRESSED_BUTTON.into();
				state.set(AppState::InGame).unwrap();
			},
			Interaction::Hovered => {
				*color = HOVERED_BUTTON.into();
			},
			Interaction::None => {
				*color = NORMAL_BUTTON.into();
			},
		}
	}
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
	commands
		.entity(menu_data.button_entity)
		.despawn_recursive();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn_bundle(SpriteBundle {
		texture: asset_server.load("branding/icon.png"),
		..Default::default()
	});
}

const SPEED: f32 = 100.0;
fn movement(
	time: Res<Time>,
	input: Res<Input<KeyCode>>,
	mut query: Query<&mut Transform, With<Sprite>>,
) {
	for mut transform in &mut query {
		let mut direction = Vec3::ZERO;
		if input.pressed(KeyCode::Left) {
			direction.x -= 1.0;
		}
		if input.pressed(KeyCode::Right) {
			direction.x += 1.0;
		}
		if input.pressed(KeyCode::Up) {
			direction.y += 1.0;
		}
		if input.pressed(KeyCode::Down) {
			direction.y -= 1.0;
		}

		if direction != Vec3::ZERO {
			transform.translation += direction.normalize() * SPEED * time.delta_seconds();
		}
	}
}

fn change_color(time: Res<Time>, mut query: Query<&mut Sprite>) {
	for mut sprite in &mut query {
		sprite
			.color
			.set_b((time.seconds_since_startup() * 0.5).sin() as f32 + 2.0);
	}
}
