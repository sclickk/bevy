#[allow(clippy::module_inception)]
mod camera;
mod camera_driver_node;
mod projection;

pub use camera::*;
pub use camera_driver_node::*;
pub use projection::*;

use crate::{
	primitives::Aabb,
	render_graph::RenderGraph,
	view::{ComputedVisibility, Visibility, VisibleEntities},
	RenderApp, RenderStage,
};
use bevy_app::{App, Plugin};

#[derive(Default)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<Camera>();
		app.register_type::<Visibility>();
		app.register_type::<ComputedVisibility>();
		app.register_type::<VisibleEntities>();
		app.register_type::<WindowOrigin>();
		app.register_type::<ScalingMode>();
		app.register_type::<DepthCalculation>();
		app.register_type::<Aabb>();
		app.register_type::<CameraRenderGraph>();
		app.init_plugin::<CameraProjectionPlugin<Projection>>();
		app.init_plugin::<CameraProjectionPlugin<OrthographicProjection>>();
		app.init_plugin::<CameraProjectionPlugin<PerspectiveProjection>>();

		if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
			render_app.add_system_to_stage(RenderStage::Extract, extract_cameras);

			let camera_driver_node = CameraDriverNode::new(&mut render_app.world);
			let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
			render_graph.add_node(crate::main_graph::node::CAMERA_DRIVER, camera_driver_node);
		}
	}
}
