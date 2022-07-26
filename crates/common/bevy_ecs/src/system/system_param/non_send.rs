use crate::{
	change_detection::{NonSendMut, Ticks},
	component::{ComponentId, ComponentTicks},
	system::{ReadOnlySystemParamFetch, SystemMeta, SystemParam, SystemParamFetch, SystemParamState},
	world::World,
};
use bevy_ptr::UnsafeCellDeref;
use std::{fmt::Debug, marker::PhantomData, ops::Deref};

/// Shared borrow of a non-[`Send`] resource.
///
/// Only `Send` resources may be accessed with the [`Res`] [`SystemParam`]. In case that the
/// resource does not implement `Send`, this `SystemParam` wrapper can be used. This will instruct
/// the scheduler to instead run the system on the main thread so that it doesn't send the resource
/// over to another thread.
///
/// # Panics
///
/// Panics when used as a `SystemParameter` if the resource does not exist.
///
/// Use `Option<NonSend<T>>` instead if the resource might not always exist.
pub struct NonSend<'w, T: 'static> {
	pub(crate) value: &'w T,
	ticks: ComponentTicks,
	last_change_tick: u32,
	change_tick: u32,
}

// SAFETY: Only reads a single World non-send resource
unsafe impl<T> ReadOnlySystemParamFetch for NonSendState<T> {}

impl<'w, T> Debug for NonSend<'w, T>
where
	T: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("NonSend")
			.field(&self.value)
			.finish()
	}
}

impl<'w, T: 'static> NonSend<'w, T> {
	/// Returns `true` if the resource was added after the system last ran.
	pub fn is_added(&self) -> bool {
		self
			.ticks
			.is_added(self.last_change_tick, self.change_tick)
	}

	/// Returns `true` if the resource was added or mutably dereferenced after the system last ran.
	pub fn is_changed(&self) -> bool {
		self
			.ticks
			.is_changed(self.last_change_tick, self.change_tick)
	}
}

impl<'w, T> Deref for NonSend<'w, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.value
	}
}
impl<'a, T> From<NonSendMut<'a, T>> for NonSend<'a, T> {
	fn from(nsm: NonSendMut<'a, T>) -> Self {
		Self {
			value: nsm.value,
			ticks: nsm.ticks.component_ticks.to_owned(),
			change_tick: nsm.ticks.change_tick,
			last_change_tick: nsm.ticks.last_change_tick,
		}
	}
}

/// The [`SystemParamState`] of [`NonSend<T>`].
#[doc(hidden)]
pub struct NonSendState<T> {
	component_id: ComponentId,
	marker: PhantomData<fn() -> T>,
}

impl<'a, T: 'static> SystemParam for NonSend<'a, T> {
	type Fetch = NonSendState<T>;
}

// SAFETY: NonSendComponentId and ArchetypeComponentId access is applied to SystemMeta. If this
// NonSend conflicts with any prior access, a panic will occur.
unsafe impl<T: 'static> SystemParamState for NonSendState<T> {
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
		system_meta.set_non_send();

		let component_id = world.initialize_non_send_resource::<T>();
		let combined_access = system_meta
			.component_access_set
			.combined_access_mut();
		assert!(
			!combined_access.has_write(component_id),
			"error[B0002]: NonSend<{}> in system {} conflicts with a previous mutable resource access ({0}). Consider removing the duplicate access.",
			std::any::type_name::<T>(),
			system_meta.name,
		);
		combined_access.add_read(component_id);

		let resource_archetype = world.archetypes.resource();
		let archetype_component_id = resource_archetype
			.get_archetype_component_id(component_id)
			.unwrap();
		system_meta
			.archetype_component_access
			.add_read(archetype_component_id);
		Self {
			component_id,
			marker: PhantomData,
		}
	}
}

impl<'w, 's, T: 'static> SystemParamFetch<'w, 's> for NonSendState<T> {
	type Item = NonSend<'w, T>;

	#[inline]
	unsafe fn get_param(
		state: &'s mut Self,
		system_meta: &SystemMeta,
		world: &'w World,
		change_tick: u32,
	) -> Self::Item {
		world.validate_non_send_access::<T>();
		let column = world
			.get_populated_resource_column(state.component_id)
			.unwrap_or_else(|| {
				panic!(
					"Non-send resource requested by {} does not exist: {}",
					system_meta.name,
					std::any::type_name::<T>()
				)
			});

		NonSend {
			value: column.get_data_ptr().deref::<T>(),
			ticks: column.get_ticks_unchecked(0).read(),
			last_change_tick: system_meta.last_change_tick,
			change_tick,
		}
	}
}

/// The [`SystemParamState`] of [`Option<NonSend<T>>`].
/// See: [`NonSend<T>`]
#[doc(hidden)]
pub struct OptionNonSendState<T>(NonSendState<T>);

impl<'w, T: 'static> SystemParam for Option<NonSend<'w, T>> {
	type Fetch = OptionNonSendState<T>;
}

// SAFETY: Only reads a single non-send resource
unsafe impl<T: 'static> ReadOnlySystemParamFetch for OptionNonSendState<T> {}

// SAFETY: this impl defers to `NonSendState`, which initializes
// and validates the correct world access
unsafe impl<T: 'static> SystemParamState for OptionNonSendState<T> {
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
		Self(NonSendState::init(world, system_meta))
	}
}

impl<'w, 's, T: 'static> SystemParamFetch<'w, 's> for OptionNonSendState<T> {
	type Item = Option<NonSend<'w, T>>;

	#[inline]
	unsafe fn get_param(
		state: &'s mut Self,
		system_meta: &SystemMeta,
		world: &'w World,
		change_tick: u32,
	) -> Self::Item {
		world.validate_non_send_access::<T>();
		world
			.get_populated_resource_column(state.0.component_id)
			.map(|column| NonSend {
				value: column.get_data_ptr().deref::<T>(),
				ticks: column.get_ticks_unchecked(0).read(),
				last_change_tick: system_meta.last_change_tick,
				change_tick,
			})
	}
}

/// The [`SystemParamState`] of [`NonSendMut<T>`].
#[doc(hidden)]
pub struct NonSendMutState<T> {
	component_id: ComponentId,
	marker: PhantomData<fn() -> T>,
}

impl<'a, T: 'static> SystemParam for NonSendMut<'a, T> {
	type Fetch = NonSendMutState<T>;
}

// SAFETY: NonSendMut ComponentId and ArchetypeComponentId access is applied to SystemMeta. If this
// NonSendMut conflicts with any prior access, a panic will occur.
unsafe impl<T: 'static> SystemParamState for NonSendMutState<T> {
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
		system_meta.set_non_send();

		let component_id = world.initialize_non_send_resource::<T>();
		let combined_access = system_meta
			.component_access_set
			.combined_access_mut();
		if combined_access.has_write(component_id) {
			panic!(
				"error[B0002]: NonSendMut<{}> in system {} conflicts with a previous mutable resource access ({0}). Consider removing the duplicate access.",
				std::any::type_name::<T>(), system_meta.name);
		} else if combined_access.has_read(component_id) {
			panic!(
				"error[B0002]: NonSendMut<{}> in system {} conflicts with a previous immutable resource access ({0}). Consider removing the duplicate access.",
				std::any::type_name::<T>(), system_meta.name);
		}
		combined_access.add_write(component_id);

		let resource_archetype = world.archetypes.resource();
		let archetype_component_id = resource_archetype
			.get_archetype_component_id(component_id)
			.unwrap();
		system_meta
			.archetype_component_access
			.add_write(archetype_component_id);
		Self {
			component_id,
			marker: PhantomData,
		}
	}
}

impl<'w, 's, T: 'static> SystemParamFetch<'w, 's> for NonSendMutState<T> {
	type Item = NonSendMut<'w, T>;

	#[inline]
	unsafe fn get_param(
		state: &'s mut Self,
		system_meta: &SystemMeta,
		world: &'w World,
		change_tick: u32,
	) -> Self::Item {
		world.validate_non_send_access::<T>();
		let column = world
			.get_populated_resource_column(state.component_id)
			.unwrap_or_else(|| {
				panic!(
					"Non-send resource requested by {} does not exist: {}",
					system_meta.name,
					std::any::type_name::<T>()
				)
			});
		NonSendMut {
			value: column
				.get_data_ptr()
				.assert_unique()
				.deref_mut::<T>(),
			ticks: Ticks {
				component_ticks: column.get_ticks_unchecked(0).deref_mut(),
				last_change_tick: system_meta.last_change_tick,
				change_tick,
			},
		}
	}
}

/// The [`SystemParamState`] of [`Option<NonSendMut<T>>`].
/// See: [`NonSendMut<T>`]
#[doc(hidden)]
pub struct OptionNonSendMutState<T>(NonSendMutState<T>);

impl<'a, T: 'static> SystemParam for Option<NonSendMut<'a, T>> {
	type Fetch = OptionNonSendMutState<T>;
}

// SAFETY: this impl defers to `NonSendMutState`, which initializes
// and validates the correct world access
unsafe impl<T: 'static> SystemParamState for OptionNonSendMutState<T> {
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
		Self(NonSendMutState::init(world, system_meta))
	}
}

impl<'w, 's, T: 'static> SystemParamFetch<'w, 's> for OptionNonSendMutState<T> {
	type Item = Option<NonSendMut<'w, T>>;

	#[inline]
	unsafe fn get_param(
		state: &'s mut Self,
		system_meta: &SystemMeta,
		world: &'w World,
		change_tick: u32,
	) -> Self::Item {
		world.validate_non_send_access::<T>();
		world
			.get_populated_resource_column(state.0.component_id)
			.map(|column| NonSendMut {
				value: column
					.get_data_ptr()
					.assert_unique()
					.deref_mut::<T>(),
				ticks: Ticks {
					component_ticks: column.get_ticks_unchecked(0).deref_mut(),
					last_change_tick: system_meta.last_change_tick,
					change_tick,
				},
			})
	}
}
