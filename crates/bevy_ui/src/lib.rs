//! This crate contains Bevy's UI system, which can be used to create UI for both 2D and 3D games
//! # Basic usage
//! Spawn UI elements with [`entity::ButtonBundle`], [`entity::ImageBundle`], [`entity::TextBundle`] and [`entity::NodeBundle`]
//! This UI is laid out with the Flexbox paradigm (see <https://cssreference.io/flexbox/> ) except the vertical axis is inverted
mod flex;
mod focus;
mod geometry;
mod render;
mod ui_node;

pub mod entity;
pub mod update;
pub mod widget;

use bevy_render::extract_component::ExtractComponentPlugin;
pub use flex::*;
pub use focus::*;
pub use geometry::*;
pub use render::*;
pub use ui_node::*;

#[doc(hidden)]
pub mod prelude {
	#[doc(hidden)]
	pub use crate::{entity::*, geometry::*, ui_node::*, widget::Button, Interaction};
}

use crate::Size;
use bevy_app::prelude::*;
use bevy_ecs::schedule::{ParallelSystemDescriptorCoercion, SystemLabel};
use bevy_input::InputSystem;
use bevy_transform::TransformSystem;
use bevy_window::ModifiesWindows;
use update::{ui_z_system, update_clipping_system};

use crate::prelude::CameraUi;

/// The basic plugin for Bevy UI
#[derive(Default)]
pub struct UiPlugin;

/// The label enum labeling the types of systems in the Bevy UI
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum UiSystem {
	/// After this label, the ui flex state has been updated
	Flex,
	/// After this label, input interactions with UI entities have been updated for this frame
	Focus,
}

impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
		app.init_plugin::<ExtractComponentPlugin<CameraUi>>();
		app.init_resource::<FlexSurface>();
		app.register_type::<AlignContent>();
		app.register_type::<AlignItems>();
		app.register_type::<AlignSelf>();
		app.register_type::<CalculatedSize>();
		app.register_type::<Direction>();
		app.register_type::<Display>();
		app.register_type::<FlexDirection>();
		app.register_type::<FlexWrap>();
		app.register_type::<FocusPolicy>();
		app.register_type::<Interaction>();
		app.register_type::<JustifyContent>();
		app.register_type::<Node>();
		// NOTE: used by Style::aspect_ratio;
		app.register_type::<Option<f32>>();
		app.register_type::<Overflow>();
		app.register_type::<PositionType>();
		app.register_type::<Size<f32>>();
		app.register_type::<Size<Val>>();
		app.register_type::<UiRect<Val>>();
		app.register_type::<Style>();
		app.register_type::<UiColor>();
		app.register_type::<UiImage>();
		app.register_type::<Val>();
		app.register_type::<widget::Button>();
		app.register_type::<widget::ImageMode>();
		app.add_system_to_stage(
			CoreStage::PreUpdate,
			ui_focus_system
				.label(UiSystem::Focus)
				.after(InputSystem),
		);
		// add these stages to front because these must run before transform update systems
		app.add_system_to_stage(
			CoreStage::PostUpdate,
			widget::text_system
				.before(UiSystem::Flex)
				.after(ModifiesWindows),
		);
		app.add_system_to_stage(
			CoreStage::PostUpdate,
			widget::image_node_system.before(UiSystem::Flex),
		);
		app.add_system_to_stage(
			CoreStage::PostUpdate,
			flex_node_system
				.label(UiSystem::Flex)
				.before(TransformSystem::TransformPropagate)
				.after(ModifiesWindows),
		);
		app.add_system_to_stage(
			CoreStage::PostUpdate,
			ui_z_system
				.after(UiSystem::Flex)
				.before(TransformSystem::TransformPropagate),
		);
		app.add_system_to_stage(
			CoreStage::PostUpdate,
			update_clipping_system.after(TransformSystem::TransformPropagate),
		);

		crate::render::build_ui_render(app);
	}
}
