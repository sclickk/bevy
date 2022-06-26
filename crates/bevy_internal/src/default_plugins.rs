use bevy_app::{PluginGroup, PluginGroupBuilder};

/// This plugin group will add all the default plugins:
/// * [`LogPlugin`](bevy_log::LogPlugin)
/// * [`CorePlugin`](bevy_core::CorePlugin)
/// * [`TimePlugin`](bevy_time::TimePlugin)
/// * [`TransformPlugin`](bevy_transform::TransformPlugin)
/// * [`HierarchyPlugin`](bevy_hierarchy::HierarchyPlugin)
/// * [`DiagnosticsPlugin`](bevy_diagnostic::DiagnosticsPlugin)
/// * [`InputPlugin`](bevy_input::InputPlugin)
/// * [`WindowPlugin`](bevy_window::WindowPlugin)
/// * [`AssetPlugin`](bevy_asset::AssetPlugin)
/// * [`ScenePlugin`](bevy_scene::ScenePlugin)
/// * [`RenderPlugin`](bevy_render::RenderPlugin) - with feature `bevy_render`
/// * [`SpritePlugin`](bevy_sprite::SpritePlugin) - with feature `bevy_sprite`
/// * [`PbrPlugin`](bevy_pbr::PbrPlugin) - with feature `bevy_pbr`
/// * [`UiPlugin`](bevy_ui::UiPlugin) - with feature `bevy_ui`
/// * [`TextPlugin`](bevy_text::TextPlugin) - with feature `bevy_text`
/// * [`AudioPlugin`](bevy_audio::AudioPlugin) - with feature `bevy_audio`
/// * [`GilrsPlugin`](bevy_gilrs::GilrsPlugin) - with feature `bevy_gilrs`
/// * [`GltfPlugin`](bevy_gltf::GltfPlugin) - with feature `bevy_gltf`
/// * [`WinitPlugin`](bevy_winit::WinitPlugin) - with feature `bevy_winit`
///
/// See also [`MinimalPlugins`] for a slimmed down option
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
	fn build(&mut self, group: &mut PluginGroupBuilder) {
		group.init::<bevy_log::LogPlugin>();
		group.init::<bevy_core::CorePlugin>();
		group.init::<bevy_time::TimePlugin>();
		group.init::<bevy_transform::TransformPlugin>();
		group.init::<bevy_hierarchy::HierarchyPlugin>();
		group.init::<bevy_diagnostic::DiagnosticsPlugin>();
		group.init::<bevy_input::InputPlugin>();
		group.init::<bevy_window::WindowPlugin>();
		group.init::<bevy_asset::AssetPlugin>();
		#[cfg(feature = "debug_asset_server")]
		group.init::<bevy_asset::debug_asset_server>();
		group.init::<bevy_scene::ScenePlugin>();

		#[cfg(feature = "bevy_winit")]
		group.init::<bevy_winit::WinitPlugin>();

		#[cfg(feature = "bevy_render")]
		group.init::<bevy_render::RenderPlugin>();

		#[cfg(feature = "bevy_core_pipeline")]
		group.init::<bevy_core_pipeline::CorePipelinePlugin>();

		#[cfg(feature = "bevy_sprite")]
		group.init::<bevy_sprite::SpritePlugin>();

		#[cfg(feature = "bevy_text")]
		group.init::<bevy_text::TextPlugin>();

		#[cfg(feature = "bevy_ui")]
		group.init::<bevy_ui::UiPlugin>();

		#[cfg(feature = "bevy_pbr")]
		group.init::<bevy_pbr::PbrPlugin>();

		// NOTE: Load this after renderer initialization so that it knows about the supported
		// compressed texture formats
		#[cfg(feature = "bevy_gltf")]
		group.init::<bevy_gltf::GltfPlugin>();

		#[cfg(feature = "bevy_audio")]
		group.init::<bevy_audio::AudioPlugin>();

		#[cfg(feature = "bevy_gilrs")]
		group.init::<bevy_gilrs::GilrsPlugin>();

		#[cfg(feature = "bevy_animation")]
		group.init::<bevy_animation::AnimationPlugin>();
	}
}

/// Minimal plugin group that will add the following plugins:
/// * [`CorePlugin`](bevy_core::CorePlugin)
/// * [`TimePlugin`](bevy_time::TimePlugin)
/// * [`ScheduleRunnerPlugin`](bevy_app::ScheduleRunnerPlugin)
///
/// See also [`DefaultPlugins`] for a more complete set of plugins
pub struct MinimalPlugins;

impl PluginGroup for MinimalPlugins {
	fn build(&mut self, group: &mut PluginGroupBuilder) {
		group.add(bevy_core::CorePlugin::default());
		group.add(bevy_time::TimePlugin::default());
		group.add(bevy_app::ScheduleRunnerPlugin::default());
	}
}
