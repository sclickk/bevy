//! In this example we generate a new texture atlas (sprite sheet) from a folder containing
//! individual sprites.

use bevy::{asset::LoadState, prelude::*, render::texture::ImageSettings};

fn main() {
	let mut app = App::new();
	app.init_resource::<RpgSpriteHandles>();
	app.insert_resource(ImageSettings::default_nearest()); // prevents blurry sprites
	app.add_plugins(DefaultPlugins);
	app.add_state(AppState::Setup);
	app.add_system_set(SystemSet::when_enter(AppState::Setup, load_textures));
	app.add_system_set(SystemSet::when_update(AppState::Setup, check_textures));
	app.add_system_set(SystemSet::when_enter(AppState::Finished, setup));
	app.run();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
	Setup,
	Finished,
}

#[derive(Default)]
struct RpgSpriteHandles {
	handles: Vec<HandleUntyped>,
}

fn load_textures(mut rpg_sprite_handles: ResMut<RpgSpriteHandles>, asset_server: Res<AssetServer>) {
	rpg_sprite_handles.handles = asset_server
		.load_folder("textures/rpg")
		.unwrap();
}

fn check_textures(
	mut state: ResMut<State<AppState>>,
	rpg_sprite_handles: ResMut<RpgSpriteHandles>,
	asset_server: Res<AssetServer>,
) {
	if let LoadState::Loaded = asset_server.get_group_load_state(
		rpg_sprite_handles
			.handles
			.iter()
			.map(|handle| handle.id),
	) {
		state.set(AppState::Finished).unwrap();
	}
}

fn setup(
	mut commands: Commands,
	rpg_sprite_handles: Res<RpgSpriteHandles>,
	asset_server: Res<AssetServer>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
	mut textures: ResMut<Assets<Image>>,
) {
	let mut texture_atlas_builder = TextureAtlasBuilder::default();
	for handle in &rpg_sprite_handles.handles {
		let handle = handle.typed_weak();
		let texture = textures.get(&handle).expect("Textures folder contained a file which way matched by a loader which did not create an `Image` asset");
		texture_atlas_builder.add_texture(handle, texture);
	}

	let texture_atlas = texture_atlas_builder
		.finish(&mut textures)
		.unwrap();
	let texture_atlas_texture = texture_atlas.texture.clone();
	let vendor_handle = asset_server.get_handle("textures/rpg/chars/vendor/generic-rpg-vendor.png");
	let vendor_index = texture_atlas
		.get_texture_index(&vendor_handle)
		.unwrap();
	let atlas_handle = texture_atlases.add(texture_atlas);

	// set up a scene to display our texture atlas
	commands.init_bundle::<Camera2dBundle>();
	// draw a sprite from the atlas
	commands.spawn_bundle(SpriteSheetBundle {
		transform: Transform {
			translation: Vec3::new(150.0, 0.0, 0.0),
			scale: Vec3::splat(4.0),
			..Default::default()
		},
		sprite: TextureAtlasSprite::new(vendor_index),
		texture_atlas: atlas_handle,
		..Default::default()
	});
	// draw the atlas itself
	commands.spawn_bundle(SpriteBundle {
		texture: texture_atlas_texture,
		transform: Transform::from_xyz(-300.0, 0.0, 0.0),
		..Default::default()
	});
}
