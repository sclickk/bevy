use crate::{
	system::{ReadOnlySystemParamFetch, SystemMeta, SystemParam, SystemParamFetch, SystemParamState},
	world::World,
};

/// A [`SystemParam`] that reads the previous and current change ticks of the system.
///
/// A system's change ticks are updated each time it runs:
/// - `last_change_tick` copies the previous value of `change_tick`
/// - `change_tick` copies the current value of [`World::read_change_tick`]
///
/// Component change ticks that are more recent than `last_change_tick` will be detected by the system.
/// Those can be read by calling [`last_changed`](crate::change_detection::DetectChanges::last_changed)
/// on a [`Mut<T>`](crate::change_detection::Mut) or [`ResMut<T>`](crate::change_detection::ResMut).
#[derive(Debug)]
pub struct SystemChangeTick {
	last_change_tick: u32,
	change_tick: u32,
}

impl SystemChangeTick {
	/// Returns the current [`World`] change tick seen by the system.
	#[inline]
	pub fn change_tick(&self) -> u32 {
		self.change_tick
	}

	/// Returns the [`World`] change tick seen by the system the previous time it ran.
	#[inline]
	pub fn last_change_tick(&self) -> u32 {
		self.last_change_tick
	}
}

/// The [`SystemParamState`] of [`SystemChangeTick`].
#[doc(hidden)]
pub struct SystemChangeTickState {}

// SAFETY: Only reads internal system state
unsafe impl ReadOnlySystemParamFetch for SystemChangeTickState {}

impl SystemParam for SystemChangeTick {
	type Fetch = SystemChangeTickState;
}

// SAFETY: `SystemParamTickState` doesn't require any world access
unsafe impl SystemParamState for SystemChangeTickState {
	fn init(_world: &mut World, _system_meta: &mut SystemMeta) -> Self {
		Self {}
	}
}

impl<'w, 's> SystemParamFetch<'w, 's> for SystemChangeTickState {
	type Item = SystemChangeTick;

	unsafe fn get_param(
		_state: &'s mut Self,
		system_meta: &SystemMeta,
		_world: &'w World,
		change_tick: u32,
	) -> Self::Item {
		SystemChangeTick {
			last_change_tick: system_meta.last_change_tick,
			change_tick,
		}
	}
}
