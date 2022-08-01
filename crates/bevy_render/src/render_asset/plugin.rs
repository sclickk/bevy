use crate::{
	render_asset::{
		extract_render_asset, prepare_assets, ExtractedAssets, PrepareAssetLabel,
		PrepareNextFrameAssets, RenderAsset, RenderAssets,
	},
	RenderApp, RenderStage,
};

use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;

use std::marker::PhantomData;

/// This plugin extracts the changed assets from the "app world" into the "render world"
/// and prepares them for the GPU. They can then be accessed from the [`RenderAssets`] resource.
///
/// Therefore it sets up the [`RenderStage::Extract`](crate::RenderStage::Extract) and
/// [`RenderStage::Prepare`](crate::RenderStage::Prepare) steps for the specified [`RenderAsset`].
pub struct RenderAssetPlugin<A: RenderAsset> {
	prepare_asset_label: PrepareAssetLabel,
	phantom: PhantomData<fn() -> A>,
}

impl<A: RenderAsset> From<PrepareAssetLabel> for RenderAssetPlugin<A> {
	fn from(prepare_asset_label: PrepareAssetLabel) -> Self {
		Self {
			prepare_asset_label,
			phantom: PhantomData,
		}
	}
}

impl<A: RenderAsset> Default for RenderAssetPlugin<A> {
	fn default() -> Self {
		Self {
			prepare_asset_label: Default::default(),
			phantom: PhantomData,
		}
	}
}

impl<A: RenderAsset> Plugin for RenderAssetPlugin<A> {
	fn build(&self, app: &mut App) {
		if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
			let prepare_asset_system = prepare_assets::<A>.label(self.prepare_asset_label.clone());

			let prepare_asset_system = match self.prepare_asset_label {
				PrepareAssetLabel::PreAssetPrepare => prepare_asset_system,
				PrepareAssetLabel::AssetPrepare => {
					prepare_asset_system.after(PrepareAssetLabel::PreAssetPrepare)
				},
				PrepareAssetLabel::PostAssetPrepare => {
					prepare_asset_system.after(PrepareAssetLabel::AssetPrepare)
				},
			};

			render_app.init_resource::<ExtractedAssets<A>>();
			render_app.init_resource::<RenderAssets<A>>();
			render_app.init_resource::<PrepareNextFrameAssets<A>>();
			render_app.add_system_to_stage(RenderStage::Extract, extract_render_asset::<A>);
			render_app.add_system_to_stage(RenderStage::Prepare, prepare_asset_system);
		}
	}
}
