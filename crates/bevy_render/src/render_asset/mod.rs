mod plugin;
pub use plugin::*;

use crate::Extract;
use bevy_asset::{Asset, AssetEvent, Assets, Handle};
use bevy_ecs::{
	prelude::*,
	system::{StaticSystemParam, SystemParam, SystemParamItem},
};
use bevy_utils::{HashMap, HashSet};

pub enum PrepareAssetError<E: Send + Sync + 'static> {
	RetryNextUpdate(E),
}

/// Describes how an asset gets extracted and prepared for rendering.
///
/// In the [`RenderStage::Extract`](crate::RenderStage::Extract) step the asset is transferred
/// from the "app world" into the "render world".
/// Therefore it is converted into a [`RenderAsset::ExtractedAsset`], which may be the same type
/// as the render asset itself.
///
/// After that in the [`RenderStage::Prepare`](crate::RenderStage::Prepare) step the extracted asset
/// is transformed into its GPU-representation of type [`RenderAsset::PreparedAsset`].
pub trait RenderAsset: Asset {
	/// The representation of the asset in the "render world".
	type ExtractedAsset: Send + Sync + 'static;
	/// The GPU-representation of the asset.
	type PreparedAsset: Send + Sync + 'static;
	/// Specifies all ECS data required by [`RenderAsset::prepare_asset`].
	/// For convenience use the [`lifetimeless`](bevy_ecs::system::lifetimeless) [`SystemParam`].
	type Param: SystemParam;
	/// Converts the asset into a [`RenderAsset::ExtractedAsset`].
	fn extract_asset(&self) -> Self::ExtractedAsset;
	/// Prepares the `extracted asset` for the GPU by transforming it into
	/// a [`RenderAsset::PreparedAsset`]. Therefore ECS data may be accessed via the `param`.
	fn prepare_asset(
		extracted_asset: Self::ExtractedAsset,
		param: &mut SystemParamItem<Self::Param>,
	) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>>;
}

#[derive(Clone, Hash, Debug, Default, PartialEq, Eq, SystemLabel)]
pub enum PrepareAssetLabel {
	PreAssetPrepare,
	#[default]
	AssetPrepare,
	PostAssetPrepare,
}

/// Temporarily stores the extracted and removed assets of the current frame.
pub struct ExtractedAssets<A: RenderAsset> {
	extracted: Vec<(Handle<A>, A::ExtractedAsset)>,
	removed: Vec<Handle<A>>,
}

impl<A: RenderAsset> Default for ExtractedAssets<A> {
	fn default() -> Self {
		Self {
			extracted: Default::default(),
			removed: Default::default(),
		}
	}
}

/// Stores all GPU representations ([`RenderAsset::PreparedAssets`](RenderAsset::PreparedAsset))
/// of [`RenderAssets`](RenderAsset) as long as they exist.
pub type RenderAssets<A> = HashMap<Handle<A>, <A as RenderAsset>::PreparedAsset>;

/// This system extracts all crated or modified assets of the corresponding [`RenderAsset`] type
/// into the "render world".
pub(crate) fn extract_render_asset<A: RenderAsset>(
	mut commands: Commands,
	mut events: Extract<EventReader<AssetEvent<A>>>,
	assets: Extract<Res<Assets<A>>>,
) {
	let mut changed = HashSet::default();
	let mut removed = Vec::new();
	for event in events.iter() {
		match event {
			AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
				changed.insert(handle.clone_weak());
			},
			AssetEvent::Removed { handle } => {
				changed.remove(handle);
				removed.push(handle.clone_weak());
			},
		}
	}

	let mut extracted = Vec::new();
	for handle in changed.drain() {
		if let Some(asset) = assets.get(&handle) {
			extracted.push((handle, asset.extract_asset()));
		}
	}

	commands.insert_resource(ExtractedAssets { extracted, removed });
}

// TODO: consider storing inside system?
/// All assets that should be prepared next frame.
pub struct PrepareNextFrameAssets<A: RenderAsset> {
	assets: Vec<(Handle<A>, A::ExtractedAsset)>,
}

impl<A: RenderAsset> Default for PrepareNextFrameAssets<A> {
	fn default() -> Self {
		Self {
			assets: Default::default(),
		}
	}
}

/// This system prepares all assets of the corresponding [`RenderAsset`] type
/// which where extracted this frame for the GPU.
pub(crate) fn prepare_assets<R: RenderAsset>(
	mut extracted_assets: ResMut<ExtractedAssets<R>>,
	mut render_assets: ResMut<RenderAssets<R>>,
	mut prepare_next_frame: ResMut<PrepareNextFrameAssets<R>>,
	param: StaticSystemParam<<R as RenderAsset>::Param>,
) {
	let mut param = param.into_inner();
	let mut queued_assets = std::mem::take(&mut prepare_next_frame.assets);
	// TODO: Code duplication here!

	for (handle, extracted_asset) in queued_assets.drain(..) {
		match R::prepare_asset(extracted_asset, &mut param) {
			Ok(prepared) => {
				render_assets.insert(handle, prepared);
			},
			Err(PrepareAssetError::RetryNextUpdate(extracted_asset)) => {
				prepare_next_frame
					.assets
					.push((handle, extracted_asset));
			},
		}
	}

	for removed in std::mem::take(&mut extracted_assets.removed) {
		render_assets.remove(&removed);
	}

	for (handle, extracted_asset) in std::mem::take(&mut extracted_assets.extracted) {
		match R::prepare_asset(extracted_asset, &mut param) {
			Ok(prepared) => {
				render_assets.insert(handle, prepared);
			},
			Err(PrepareAssetError::RetryNextUpdate(extracted_asset)) => {
				prepare_next_frame
					.assets
					.push((handle, extracted_asset));
			},
		}
	}
}
