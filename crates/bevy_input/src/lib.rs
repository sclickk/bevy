mod axis;
pub mod gamepad;
mod input;
pub mod keyboard;
pub mod mouse;
pub mod touch;

pub use axis::*;
use bevy_ecs::schedule::{ParallelSystemDescriptorCoercion, SystemLabel};
pub use input::*;

pub mod prelude {
	#[doc(hidden)]
	pub use crate::{
		gamepad::{
			Gamepad, GamepadAxis, GamepadAxisType, GamepadButton, GamepadButtonType, GamepadEvent,
			GamepadEventType, Gamepads,
		},
		keyboard::KeyCode,
		mouse::MouseButton,
		touch::{TouchInput, Touches},
		Axis, Input,
	};
}

use bevy_app::prelude::*;
use keyboard::{keyboard_input_system, KeyCode, KeyboardInput};
use mouse::{mouse_button_input_system, MouseButton, MouseButtonInput, MouseMotion, MouseWheel};
use prelude::Gamepads;
use touch::{touch_screen_input_system, TouchInput, Touches};

use gamepad::{
	gamepad_connection_system, gamepad_event_system, GamepadAxis, GamepadButton, GamepadEvent,
	GamepadEventRaw, GamepadSettings,
};

/// Adds keyboard and mouse input to an App
#[derive(Default)]
pub struct InputPlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemLabel)]
pub struct InputSystem;

impl Plugin for InputPlugin {
	fn build(&self, app: &mut App) {
			// keyboard
		app.add_event::<KeyboardInput>();
		app.init_resource::<Input<KeyCode>>();
		app.add_system_to_stage(
				CoreStage::PreUpdate,
				keyboard_input_system.label(InputSystem),
			);
			// mouse
		app.add_event::<MouseButtonInput>();
		app.add_event::<MouseMotion>();
		app.add_event::<MouseWheel>();
		app.init_resource::<Input<MouseButton>>()
			.add_system_to_stage(
				CoreStage::PreUpdate,
				mouse_button_input_system.label(InputSystem),
			);
			// gamepad
		app.add_event::<GamepadEvent>();
		app.add_event::<GamepadEventRaw>();
		app.init_resource::<GamepadSettings>()
			.init_resource::<Gamepads>()
			.init_resource::<Input<GamepadButton>>()
			.init_resource::<Axis<GamepadAxis>>()
			.init_resource::<Axis<GamepadButton>>()
			.add_system_to_stage(
				CoreStage::PreUpdate,
				gamepad_event_system.label(InputSystem),
			)
			.add_system_to_stage(
				CoreStage::PreUpdate,
				gamepad_connection_system.after(InputSystem),
			);
			// touch
		app.add_event::<TouchInput>();
		app.init_resource::<Touches>()
			.add_system_to_stage(
				CoreStage::PreUpdate,
				touch_screen_input_system.label(InputSystem),
			);
	}
}

/// The current "press" state of an element
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum ButtonState {
	Pressed,
	Released,
}

impl ButtonState {
	pub fn is_pressed(&self) -> bool {
		matches!(self, ButtonState::Pressed)
	}
}
