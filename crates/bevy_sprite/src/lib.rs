mod bundle;
mod dynamic_texture_atlas_builder;
mod mesh2d;
mod rect;
mod render;
mod sprite;
mod texture_atlas;
mod texture_atlas_builder;

pub mod collide_aabb;

pub mod prelude {
	#[doc(hidden)]
	pub use crate::{
		bundle::{SpriteBundle, SpriteSheetBundle},
		sprite::Sprite,
		texture_atlas::{TextureAtlas, TextureAtlasSprite},
		ColorMaterial, ColorMesh2dBundle, TextureAtlasBuilder,
	};
}

pub use bundle::*;
pub use dynamic_texture_atlas_builder::*;
pub use mesh2d::*;
pub use rect::*;
pub use render::*;
pub use sprite::*;
pub use texture_atlas::*;
pub use texture_atlas_builder::*;

use bevy_app::prelude::*;
use bevy_asset::{AddAsset, Assets, HandleUntyped};
use bevy_core_pipeline::core_2d::Transparent2d;
use bevy_ecs::schedule::{ParallelSystemDescriptorCoercion, SystemLabel};
use bevy_reflect::TypeUuid;
use bevy_render::{
	render_phase::AddRenderCommand,
	render_resource::{Shader, SpecializedRenderPipelines},
	RenderApp, RenderStage,
};

#[derive(Default)]
pub struct SpritePlugin;

pub const SPRITE_SHADER_HANDLE: HandleUntyped =
	HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597127);

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum SpriteSystem {
	ExtractSprites,
}

impl Plugin for SpritePlugin {
	fn build(&self, app: &mut App) {
		let mut shaders = app.world.resource_mut::<Assets<Shader>>();
		let sprite_shader = Shader::from_wgsl(include_str!("render/sprite.wgsl"));
		shaders.set_untracked(SPRITE_SHADER_HANDLE, sprite_shader);
		app.add_asset::<TextureAtlas>();
		app.register_type::<Sprite>();
		app.register_type::<Mesh2dHandle>();
		app.add_plugin(Mesh2dRenderPlugin);
		app.add_plugin(ColorMaterialPlugin);

		if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
			render_app.init_resource::<ImageBindGroups>();
			render_app.init_resource::<SpritePipeline>();
			render_app.init_resource::<SpecializedRenderPipelines<SpritePipeline>>();
			render_app.init_resource::<SpriteMeta>();
			render_app.init_resource::<ExtractedSprites>();
			render_app.init_resource::<SpriteAssetEvents>();
			render_app.add_render_command::<Transparent2d, DrawSprite>();
			render_app.add_system_to_stage(
				RenderStage::Extract,
				render::extract_sprites.label(SpriteSystem::ExtractSprites),
			);
			render_app.add_system_to_stage(RenderStage::Extract, render::extract_sprite_events);
			render_app.add_system_to_stage(RenderStage::Queue, queue_sprites);
		};
	}
}
