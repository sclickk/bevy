#[warn(missing_docs)]
mod cursor;
mod event;
mod plugin;
mod raw_window_handle;
mod system;
mod window;
mod windows;

pub use crate::raw_window_handle::*;
pub use cursor::*;
pub use event::*;
pub use plugin::*;
pub use system::*;
pub use window::*;
pub use windows::*;

pub mod prelude {
	#[doc(hidden)]
	pub use crate::{
		CursorEntered, CursorIcon, CursorLeft, CursorMoved, FileDragAndDrop, MonitorSelection,
		ReceivedCharacter, Window, WindowDescriptor, WindowMoved, WindowPosition, Windows,
	};
}

use bevy_ecs::schedule::SystemLabel;

/// The configuration information for the [`WindowPlugin`].
///
/// It can be added as a [`Resource`](bevy_ecs::system::Resource) before the [`WindowPlugin`]
/// runs, to configure how it behaves.
#[derive(Clone)]
pub struct WindowSettings {
	/// Whether to create a window when added.
	///
	/// Note that if there are no windows, by default the App will exit,
	/// due to [`exit_on_all_closed`].
	pub add_primary_window: bool,
	/// Whether to exit the app when there are no open windows.
	///
	/// If disabling this, ensure that you send the [`bevy_app::AppExit`]
	/// event when the app should exit. If this does not occur, you will
	/// create 'headless' processes (processes without windows), which may
	/// surprise your users. It is recommended to leave this setting as `true`.
	///
	/// If true, this plugin will add [`exit_on_all_closed`] to [`CoreStage::Update`].
	pub exit_on_all_closed: bool,
	/// Whether to close windows when they are requested to be closed (i.e.
	/// when the close button is pressed).
	///
	/// If true, this plugin will add [`close_when_requested`] to [`CoreStage::Update`].
	/// If this system (or a replacement) is not running, the close button will have no effect.
	/// This may surprise your users. It is recommended to leave this setting as `true`.
	pub close_when_requested: bool,
}

impl Default for WindowSettings {
	fn default() -> Self {
		WindowSettings {
			add_primary_window: true,
			exit_on_all_closed: true,
			close_when_requested: true,
		}
	}
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub struct ModifiesWindows;
