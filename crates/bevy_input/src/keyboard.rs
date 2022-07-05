use crate::{ButtonState, Input};
use bevy_ecs::{event::EventReader, system::ResMut};
use std::fmt;

/// A keyboard input event.
///
/// This event is the translated version of the `WindowEvent::KeyboardInput` from the `winit` crate.
/// It is available to the end user and can be used for game logic.
///
/// ## Usage
///
/// The event is consumed inside of the [`keyboard_input_system`](crate::keyboard::keyboard_input_system)
/// to update the [`Input<KeyCode>`](crate::Input<KeyCode>) resource.
#[derive(Debug, Clone)]
pub struct KeyboardInput {
	/// The scan code of the key.
	pub scan_code: u32,
	/// The key code of the key.
	pub key_code: Option<KeyCode>,
	/// The press state of the key.
	pub state: ButtonState,
}

/// Updates the [`Input<KeyCode>`] resource with the latest [`KeyboardInput`] events.
///
/// ## Differences
///
/// The main difference between the [`KeyboardInput`] event and the [`Input<KeyCode>`] resource is that
/// the latter has convenient functions like [`Input::pressed`], [`Input::just_pressed`] and [`Input::just_released`].
pub fn keyboard_input_system(
	mut keyboard_input: ResMut<Input<KeyCode>>,
	mut keyboard_input_events: EventReader<KeyboardInput>,
) {
	keyboard_input.clear();
	for event in keyboard_input_events.iter() {
		if let KeyboardInput {
			key_code: Some(key_code),
			state,
			..
		} = event
		{
			match state {
				ButtonState::Pressed => keyboard_input.press(*key_code),
				ButtonState::Released => keyboard_input.release(*key_code),
			}
		}
	}
}

/// The key code of a [`KeyboardInput`](crate::keyboard::KeyboardInput).
///
/// ## Usage
///
/// It is used as the generic `T` value of an [`Input`](crate::Input) to create a `Res<Input<KeyCode>>`.
/// The resource stores the data of the buttons of a keyboard and can be accessed inside of a system.
///
/// ## Updating
///
/// The resource is updated inside of the [`keyboard_input_system`](crate::keyboard::keyboard_input_system).
#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum KeyCode {
	/// The `1` key over the letters.
	Key1,
	/// The `2` key over the letters.
	Key2,
	/// The `3` key over the letters.
	Key3,
	/// The `4` key over the letters.
	Key4,
	/// The `5` key over the letters.
	Key5,
	/// The `6` key over the letters.
	Key6,
	/// The `7` key over the letters.
	Key7,
	/// The `8` key over the letters.
	Key8,
	/// The `9` key over the letters.
	Key9,
	/// The `0` key over the letters.
	Key0,

	/// The `A` key.
	A,
	/// The `B` key.
	B,
	/// The `C` key.
	C,
	/// The `D` key.
	D,
	/// The `E` key.
	E,
	/// The `F` key.
	F,
	/// The `G` key.
	G,
	/// The `H` key.
	H,
	/// The `I` key.
	I,
	/// The `J` key.
	J,
	/// The `K` key.
	K,
	/// The `L` key.
	L,
	/// The `M` key.
	M,
	/// The `N` key.
	N,
	/// The `O` key.
	O,
	/// The `P` key.
	P,
	/// The `Q` key.
	Q,
	/// The `R` key.
	R,
	/// The `S` key.
	S,
	/// The `T` key.
	T,
	/// The `U` key.
	U,
	/// The `V` key.
	V,
	/// The `W` key.
	W,
	/// The `X` key.
	X,
	/// The `Y` key.
	Y,
	/// The `Z` key.
	Z,

	/// The `Escape` / `ESC` key, next to the `F1` key.
	Escape,

	/// The `F1` key.
	F1,
	/// The `F2` key.
	F2,
	/// The `F3` key.
	F3,
	/// The `F4` key.
	F4,
	/// The `F5` key.
	F5,
	/// The `F6` key.
	F6,
	/// The `F7` key.
	F7,
	/// The `F8` key.
	F8,
	/// The `F9` key.
	F9,
	/// The `F10` key.
	F10,
	/// The `F11` key.
	F11,
	/// The `F12` key.
	F12,
	/// The `F13` key.
	F13,
	/// The `F14` key.
	F14,
	/// The `F15` key.
	F15,
	/// The `F16` key.
	F16,
	/// The `F17` key.
	F17,
	/// The `F18` key.
	F18,
	/// The `F19` key.
	F19,
	/// The `F20` key.
	F20,
	/// The `F21` key.
	F21,
	/// The `F22` key.
	F22,
	/// The `F23` key.
	F23,
	/// The `F24` key.
	F24,

	/// The `Snapshot` / `Print Screen` key.
	Snapshot,
	/// The `Scroll` / `Scroll Lock` key.
	Scroll,
	/// The `Pause` / `Break` key, next to the `Scroll` key.
	Pause,

	/// The `Insert` key, next to the `Backspace` key.
	Insert,
	/// The `Home` key.
	Home,
	/// The `Delete` key.
	Delete,
	/// The `End` key.
	End,
	/// The `PageDown` key.
	PageDown,
	/// The `PageUp` key.
	PageUp,

	/// The `Left` / `Left Arrow` key.
	Left,
	/// The `Up` / `Up Arrow` key.
	Up,
	/// The `Right` / `Right Arrow` key.
	Right,
	/// The `Down` / `Down Arrow` key.
	Down,

	/// The `Back` / `Backspace` key.
	Back,
	/// The `Return` / `Enter` key.
	Return,
	/// The `Space` / `Spacebar` / ` ` key.
	Space,

	/// The `Compose` key on Linux.
	Compose,
	/// The `Caret` / `^` key.
	Caret,

	/// The `Numlock` key.
	Numlock,
	/// The `Numpad0` / `0` key.
	Numpad0,
	/// The `Numpad1` / `1` key.
	Numpad1,
	/// The `Numpad2` / `2` key.
	Numpad2,
	/// The `Numpad3` / `3` key.
	Numpad3,
	/// The `Numpad4` / `4` key.
	Numpad4,
	/// The `Numpad5` / `5` key.
	Numpad5,
	/// The `Numpad6` / `6` key.
	Numpad6,
	/// The `Numpad7` / `7` key.
	Numpad7,
	/// The `Numpad8` / `8` key.
	Numpad8,
	/// The `Numpad9` / `9` key.
	Numpad9,

	/// The `AbntC1` key.
	AbntC1,
	/// The `AbntC2` key.
	AbntC2,

	/// The `NumpadAdd` / `+` key.
	NumpadAdd,
	/// The `Apostrophe` / `'` key.
	Apostrophe,
	/// The `Apps` key.
	Apps,
	/// The `Asterik` / `*` key.
	Asterisk,
	/// The `Plus` / `+` key.
	Plus,
	/// The `At` / `@` key.
	At,
	/// The `Ax` key.
	Ax,
	/// The `Backslash` / `\` key.
	Backslash,
	/// The `Calculator` key.
	Calculator,
	/// The `Capital` key.
	Capital,
	/// The `Colon` / `:` key.
	Colon,
	/// The `Comma` / `,` key.
	Comma,
	/// The `Convert` key.
	Convert,
	/// The `NumpadDecimal` / `.` key.
	NumpadDecimal,
	/// The `NumpadDivide` / `/` key.
	NumpadDivide,
	/// The `Equals` / `=` key.
	Equals,
	/// The `Grave` / `Backtick` / `` ` `` key.
	Grave,
	/// The `Kana` key.
	Kana,
	/// The `Kanji` key.
	Kanji,

	/// The `LAlt` / `Left Alt` key. Maps to `Left Option` on Mac.
	LAlt,
	/// The `LBracket` / `Left Bracket` key.
	LBracket,
	/// The `LControl` / `Left Control` key.
	LControl,
	/// The `LShift` / `Left Shift` key.
	LShift,
	/// The `LWin` / `Left Windows` key. Maps to `Left Command` on Mac.
	LWin,

	/// The `Mail` key.
	Mail,
	/// The `MediaSelect` key.
	MediaSelect,
	/// The `MediaStop` key.
	MediaStop,
	/// The `Minus` / `-` key.
	Minus,
	/// The `NumpadMultiply` / `*` key.
	NumpadMultiply,
	/// The `Mute` key.
	Mute,
	/// The `MyComputer` key.
	MyComputer,
	/// The `NavigateForward` / `Prior` key.
	NavigateForward,
	/// The `NavigateBackward` / `Next` key.
	NavigateBackward,
	/// The `NextTrack` key.
	NextTrack,
	/// The `NoConvert` key.
	NoConvert,
	/// The `NumpadComma` / `,` key.
	NumpadComma,
	/// The `NumpadEnter` key.
	NumpadEnter,
	/// The `NumpadEquals` / `=` key.
	NumpadEquals,
	/// The `Oem102` key.
	Oem102,
	/// The `Period` / `.` key.
	Period,
	/// The `PlayPause` key.
	PlayPause,
	/// The `Power` key.
	Power,
	/// The `PrevTrack` key.
	PrevTrack,

	/// The `RAlt` / `Right Alt` key. Maps to `Right Option` on Mac.
	RAlt,
	/// The `RBracket` / `Right Bracket` key.
	RBracket,
	/// The `RControl` / `Right Control` key.
	RControl,
	/// The `RShift` / `Right Shift` key.
	RShift,
	/// The `RWin` / `Right Windows` key. Maps to `Right Command` on Mac.
	RWin,

	/// The `Semicolon` / `;` key.
	Semicolon,
	/// The `Slash` / `/` key.
	Slash,
	/// The `Sleep` key.
	Sleep,
	/// The `Stop` key.
	Stop,
	/// The `NumpadSubtract` / `-` key.
	NumpadSubtract,
	/// The `Sysrq` key.
	Sysrq,
	/// The `Tab` / `   ` key.
	Tab,
	/// The `Underline` / `_` key.
	Underline,
	/// The `Unlabeled` key.
	Unlabeled,

	/// The `VolumeDown` key.
	VolumeDown,
	/// The `VolumeUp` key.
	VolumeUp,

	/// The `Wake` key.
	Wake,

	/// The `WebBack` key.
	WebBack,
	/// The `WebFavorites` key.
	WebFavorites,
	/// The `WebForward` key.
	WebForward,
	/// The `WebHome` key.
	WebHome,
	/// The `WebRefresh` key.
	WebRefresh,
	/// The `WebSearch` key.
	WebSearch,
	/// The `WebStop` key.
	WebStop,

	/// The `Yen` key.
	Yen,

	/// The `Copy` key.
	Copy,
	/// The `Paste` key.
	Paste,
	/// The `Cut` key.
	Cut,
}

impl fmt::Display for KeyCode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Key1 => "1",
				Self::Key2 => "2",
				Self::Key3 => "3",
				Self::Key4 => "4",
				Self::Key5 => "5",
				Self::Key6 => "6",
				Self::Key7 => "7",
				Self::Key8 => "8",
				Self::Key9 => "9",
				Self::Key0 => "0",

				Self::A => "A",
				Self::B => "B",
				Self::C => "C",
				Self::D => "D",
				Self::E => "E",
				Self::F => "F",
				Self::G => "G",
				Self::H => "H",
				Self::I => "I",
				Self::J => "J",
				Self::K => "K",
				Self::L => "L",
				Self::M => "M",
				Self::N => "N",
				Self::O => "O",
				Self::P => "P",
				Self::Q => "Q",
				Self::R => "R",
				Self::S => "S",
				Self::T => "T",
				Self::U => "U",
				Self::V => "V",
				Self::W => "W",
				Self::X => "X",
				Self::Y => "Y",
				Self::Z => "Z",

				Self::Escape => "Escape",

				Self::F1 => "F1",
				Self::F2 => "F2",
				Self::F3 => "F3",
				Self::F4 => "F4",
				Self::F5 => "F5",
				Self::F6 => "F6",
				Self::F7 => "F7",
				Self::F8 => "F8",
				Self::F9 => "F9",
				Self::F10 => "F10",
				Self::F11 => "F11",
				Self::F12 => "F12",
				Self::F13 => "F13",
				Self::F14 => "F14",
				Self::F15 => "F15",
				Self::F16 => "F16",
				Self::F17 => "F17",
				Self::F18 => "F18",
				Self::F19 => "F19",
				Self::F20 => "F20",
				Self::F21 => "F21",
				Self::F22 => "F22",
				Self::F23 => "F23",
				Self::F24 => "F24",

				Self::Snapshot => "",
				Self::Scroll => "",
				Self::Pause => "",

				Self::Insert => "Insert",
				Self::Delete => "Delete",
				Self::Home => "Home",
				Self::End => "End",
				Self::PageDown => "Page Down",
				Self::PageUp => "Page Up",

				Self::Left => "Left",
				Self::Up => "Up",
				Self::Right => "Right",
				Self::Down => "Down",

				Self::Back => "Back",
				Self::Return => "Return",
				Self::Space => "Space",

				Self::Compose => "",
				Self::Caret => "",

				Self::Numlock => "Number Pad Lock",
				Self::Numpad0 => "Number Pad 0",
				Self::Numpad1 => "Number Pad 1",
				Self::Numpad2 => "Number Pad 2",
				Self::Numpad3 => "Number Pad 3",
				Self::Numpad4 => "Number Pad 4",
				Self::Numpad5 => "Number Pad 5",
				Self::Numpad6 => "Number Pad 6",
				Self::Numpad7 => "Number Pad 7",
				Self::Numpad8 => "Number Pad 8",
				Self::Numpad9 => "Number Pad 9",
				Self::NumpadAdd => "",
				Self::NumpadDivide => "",
				Self::NumpadDecimal => "",
				Self::NumpadComma => "",
				Self::NumpadEnter => "",
				Self::NumpadEquals => "",
				Self::NumpadSubtract => "",

				Self::AbntC1 => "",
				Self::AbntC2 => "",

				Self::Apostrophe => "",
				Self::Apps => "",
				Self::Asterisk => "",
				Self::Plus => "",
				Self::At => "",
				Self::Ax => "",
				Self::Backslash => "",
				Self::Calculator => "",
				Self::Capital => "",
				Self::Colon => "",
				Self::Comma => "",
				Self::Convert => "",
				Self::Equals => "",
				Self::Grave => "",
				Self::Kana => "",
				Self::Kanji => "",

				Self::LAlt => "",
				Self::LBracket => "",
				Self::LControl => "",
				Self::LShift => "",
				Self::LWin => "",

				Self::Mail => "",
				Self::MediaSelect => "",
				Self::MediaStop => "",
				Self::Minus => "",
				Self::NumpadMultiply => "",
				Self::Mute => "",
				Self::MyComputer => "",
				Self::NavigateForward => "",
				Self::NavigateBackward => "",
				Self::NextTrack => "",
				Self::NoConvert => "",
				Self::Oem102 => "",
				Self::Period => "",
				Self::PlayPause => "",
				Self::Power => "",
				Self::PrevTrack => "",

				Self::RAlt => "",
				Self::RBracket => "",
				Self::RControl => "",
				Self::RShift => "",
				Self::RWin => "",

				Self::Semicolon => "",
				Self::Slash => "",
				Self::Sleep => "",
				Self::Stop => "",

				Self::Sysrq => "",
				Self::Tab => "",
				Self::Underline => "",
				Self::Unlabeled => "",

				Self::VolumeDown => "Volume Up",
				Self::VolumeUp => "Volume Down",

				Self::Wake => "Wake",

				Self::WebBack => "",
				Self::WebFavorites => "",
				Self::WebForward => "",
				Self::WebHome => "",
				Self::WebRefresh => "",
				Self::WebSearch => "",
				Self::WebStop => "",

				Self::Yen => "Yen",

				Self::Copy => "Copy",
				Self::Paste => "Paste",
				Self::Cut => "Cut",
				_ => "Unknown Key",
			}
		)
	}
}
