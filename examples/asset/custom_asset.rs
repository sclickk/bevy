//! Implements loader for a custom asset type.

use bevy::{
	asset::{AssetLoader, LoadContext, LoadedAsset},
	prelude::*,
	reflect::TypeUuid,
	utils::BoxedFuture,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, TypeUuid)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct CustomAsset {
	pub value: i32,
}

#[derive(Default)]
pub struct CustomAssetLoader;

impl AssetLoader for CustomAssetLoader {
	fn load<'a>(
		&'a self,
		bytes: &'a [u8],
		load_context: &'a mut LoadContext,
	) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
		Box::pin(async move {
			let custom_asset = ron::de::from_bytes::<CustomAsset>(bytes)?;
			load_context.set_default_asset(LoadedAsset::from(custom_asset));
			Ok(())
		})
	}

	fn extensions(&self) -> &[&str] {
		&["custom"]
	}
}

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.init_resource::<State>();
	app.add_asset::<CustomAsset>();
	app.init_asset_loader::<CustomAssetLoader>();
	app.add_startup_system(setup);
	app.add_system(print_on_load);
	app.run();
}

#[derive(Default)]
struct State {
	handle: Handle<CustomAsset>,
	printed: bool,
}

fn setup(mut state: ResMut<State>, asset_server: Res<AssetServer>) {
	state.handle = asset_server.load("data/asset.custom");
}

fn print_on_load(mut state: ResMut<State>, custom_assets: ResMut<Assets<CustomAsset>>) {
	let custom_asset = custom_assets.get(&state.handle);
	if state.printed || custom_asset.is_none() {
		return;
	}

	info!("Custom asset loaded: {:?}", custom_asset.unwrap());
	state.printed = true;
}
