use crate::{
	close_when_requested,
	event::{
		CreateWindow, CursorEntered, CursorLeft, CursorMoved, FileDragAndDrop, ReceivedCharacter,
		RequestRedraw, WindowBackendScaleFactorChanged, WindowCloseRequested, WindowClosed,
		WindowCreated, WindowFocused, WindowMoved, WindowResized, WindowScaleFactorChanged,
	},
	exit_on_all_closed, WindowDescriptor, WindowId, WindowSettings, Windows,
};

use bevy_app::{App, Plugin};

use bevy_ecs::event::Events;

/// A [`Plugin`] that defines an interface for windowing support in Bevy.
#[derive(Default)]
pub struct WindowPlugin;

impl Plugin for WindowPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<WindowResized>();
		app.add_event::<CreateWindow>();
		app.add_event::<WindowCreated>();
		app.add_event::<WindowClosed>();
		app.add_event::<WindowCloseRequested>();
		app.add_event::<RequestRedraw>();
		app.add_event::<CursorMoved>();
		app.add_event::<CursorEntered>();
		app.add_event::<CursorLeft>();
		app.add_event::<ReceivedCharacter>();
		app.add_event::<WindowFocused>();
		app.add_event::<WindowScaleFactorChanged>();
		app.add_event::<WindowBackendScaleFactorChanged>();
		app.add_event::<FileDragAndDrop>();
		app.add_event::<WindowMoved>();
		app.init_resource::<Windows>();

		let settings = app
			.world
			.get_resource::<WindowSettings>()
			.cloned()
			.unwrap_or_default();

		if settings.add_primary_window {
			let event = CreateWindow {
				id: WindowId::PRIMARY,
				descriptor: app
					.world
					.get_resource::<WindowDescriptor>()
					.cloned()
					.unwrap_or_default(),
			};

			app
				.world
				.resource_mut::<Events<CreateWindow>>()
				.send(event);
		}

		if settings.exit_on_all_closed {
			app.add_system(exit_on_all_closed);
		}
		if settings.close_when_requested {
			app.add_system(close_when_requested);
		}
	}
}
