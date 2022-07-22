#![warn(missing_docs)]
//! This crate provides core functionality for Bevy Engine.

mod name;

pub use bytemuck::{bytes_of, cast_slice, Pod, Zeroable};
pub use name::*;

pub mod prelude {
	//! The Bevy Core Prelude.
	#[doc(hidden)]
	pub use crate::Name;
}

use bevy_app::prelude::*;
use bevy_ecs::entity::Entity;
use bevy_utils::HashSet;
use bevy_tasks::DefaultTaskPoolOptions;
use std::ops::Range;

/// Adds core functionality to Apps.
#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
	fn build(&self, app: &mut App) {
		// Setup the default bevy task pools
		app
			.world
			.get_resource::<DefaultTaskPoolOptions>()
			.cloned()
			.unwrap_or_default()
			.create_default_pools();

		app.register_type::<Entity>();
		app.register_type::<Name>();

		register_rust_types(app);
		register_math_types(app);
	}
}

fn register_rust_types(app: &mut App) {
	app.register_type::<Range<f32>>();
	app.register_type::<String>();
	app.register_type::<HashSet<String>>();
	app.register_type::<Option<String>>();
}

fn register_math_types(app: &mut App) {
	app.register_type::<bevy_math::IVec2>();
	app.register_type::<bevy_math::IVec3>();
	app.register_type::<bevy_math::IVec4>();
	app.register_type::<bevy_math::UVec2>();
	app.register_type::<bevy_math::UVec3>();
	app.register_type::<bevy_math::UVec4>();
	app.register_type::<bevy_math::Vec2>();
	app.register_type::<bevy_math::Vec3>();
	app.register_type::<bevy_math::Vec4>();
	app.register_type::<bevy_math::Mat2>();
	app.register_type::<bevy_math::Mat3>();
	app.register_type::<bevy_math::Mat4>();
	app.register_type::<bevy_math::Quat>();
}
