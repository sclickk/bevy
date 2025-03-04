use crate::{
	mesh::{skinning, Mesh},
	render_asset::RenderAssetPlugin,
};

use bevy_app::{App, Plugin};
use bevy_asset::AddAsset;

/// Adds the [`Mesh`] as an asset and makes sure that they are extracted and prepared for the GPU.
pub struct MeshPlugin;

impl Plugin for MeshPlugin {
	fn build(&self, app: &mut App) {
		app.add_asset::<Mesh>();
		app.add_asset::<skinning::SkinnedMeshInverseBindposes>();
		app.register_type::<skinning::SkinnedMesh>();
		app.init_plugin::<RenderAssetPlugin<Mesh>>();
	}
}
