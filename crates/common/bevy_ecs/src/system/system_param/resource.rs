use crate::{
	change_detection::ResMut,
	change_detection::Ticks,
	component::{ComponentId, ComponentTicks},
	system::{ReadOnlySystemParamFetch, SystemMeta, SystemParam, SystemParamFetch, SystemParamState},
	world::World,
};
use bevy_ptr::UnsafeCellDeref;
use std::{fmt::Debug, marker::PhantomData, ops::Deref};

pub trait Resource: Send + Sync + 'static {}

impl<T> Resource for T where T: Send + Sync + 'static {}

/// Shared borrow of a resource.
///
/// See the [`World`] documentation to see the usage of a resource.
///
/// If you need a unique mutable borrow, use [`ResMut`] instead.
///
/// # Panics
///
/// Panics when used as a [`SystemParameter`](SystemParam) if the resource does not exist.
///
/// Use `Option<Res<T>>` instead if the resource might not always exist.
pub struct Res<'w, T: Resource> {
	value: &'w T,
	ticks: &'w ComponentTicks,
	last_change_tick: u32,
	change_tick: u32,
}

// SAFETY: Res only reads a single World resource
unsafe impl<T: Resource> ReadOnlySystemParamFetch for ResState<T> {}

impl<'w, T: Resource> Debug for Res<'w, T>
where
	T: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("Res").field(&self.value).finish()
	}
}

impl<'w, T: Resource> Res<'w, T> {
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

	pub fn into_inner(self) -> &'w T {
		self.value
	}
}

impl<'w, T: Resource> Deref for Res<'w, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.value
	}
}

impl<'w, T: Resource> AsRef<T> for Res<'w, T> {
	#[inline]
	fn as_ref(&self) -> &T {
		self.deref()
	}
}

impl<'w, T: Resource> From<ResMut<'w, T>> for Res<'w, T> {
	fn from(res: ResMut<'w, T>) -> Self {
		Self {
			value: res.value,
			ticks: res.ticks.component_ticks,
			change_tick: res.ticks.change_tick,
			last_change_tick: res.ticks.last_change_tick,
		}
	}
}

/// The [`SystemParamState`] of [`Res<T>`].
#[doc(hidden)]
pub struct ResState<T> {
	component_id: ComponentId,
	marker: PhantomData<T>,
}

impl<'a, T: Resource> SystemParam for Res<'a, T> {
	type Fetch = ResState<T>;
}

// SAFETY: Res ComponentId and ArchetypeComponentId access is applied to SystemMeta. If this Res
// conflicts with any prior access, a panic will occur.
unsafe impl<T: Resource> SystemParamState for ResState<T> {
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
		let component_id = world.initialize_resource::<T>();
		let combined_access = system_meta
			.component_access_set
			.combined_access_mut();
		assert!(
			!combined_access.has_write(component_id),
			"error[B0002]: Res<{}> in system {} conflicts with a previous ResMut<{0}> access. Consider removing the duplicate access.",
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

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResState<T> {
	type Item = Res<'w, T>;

	#[inline]
	unsafe fn get_param(
		state: &'s mut Self,
		system_meta: &SystemMeta,
		world: &'w World,
		change_tick: u32,
	) -> Self::Item {
		let column = world
			.get_populated_resource_column(state.component_id)
			.unwrap_or_else(|| {
				panic!(
					"Resource requested by {} does not exist: {}",
					system_meta.name,
					std::any::type_name::<T>()
				)
			});
		Res {
			value: column.get_data_ptr().deref::<T>(),
			ticks: column.get_ticks_unchecked(0).deref(),
			last_change_tick: system_meta.last_change_tick,
			change_tick,
		}
	}
}

/// The [`SystemParamState`] of [`Option<Res<T>>`].
/// See: [`Res<T>`]
#[doc(hidden)]
pub struct OptionResState<T>(ResState<T>);

impl<'a, T: Resource> SystemParam for Option<Res<'a, T>> {
	type Fetch = OptionResState<T>;
}

// SAFETY: Only reads a single World resource
unsafe impl<T: Resource> ReadOnlySystemParamFetch for OptionResState<T> {}

// SAFETY: this impl defers to `ResState`, which initializes
// and validates the correct world access
unsafe impl<T: Resource> SystemParamState for OptionResState<T> {
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
		Self(ResState::init(world, system_meta))
	}
}

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for OptionResState<T> {
	type Item = Option<Res<'w, T>>;

	#[inline]
	unsafe fn get_param(
		state: &'s mut Self,
		system_meta: &SystemMeta,
		world: &'w World,
		change_tick: u32,
	) -> Self::Item {
		world
			.get_populated_resource_column(state.0.component_id)
			.map(|column| Res {
				value: column.get_data_ptr().deref::<T>(),
				ticks: column.get_ticks_unchecked(0).deref(),
				last_change_tick: system_meta.last_change_tick,
				change_tick,
			})
	}
}

/// The [`SystemParamState`] of [`ResMut<T>`].
#[doc(hidden)]
pub struct ResMutState<T> {
	component_id: ComponentId,
	marker: PhantomData<T>,
}

impl<'a, T: Resource> SystemParam for ResMut<'a, T> {
	type Fetch = ResMutState<T>;
}

// SAFETY: Res ComponentId and ArchetypeComponentId access is applied to SystemMeta. If this Res
// conflicts with any prior access, a panic will occur.
unsafe impl<T: Resource> SystemParamState for ResMutState<T> {
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
		let component_id = world.initialize_resource::<T>();
		let combined_access = system_meta
			.component_access_set
			.combined_access_mut();
		if combined_access.has_write(component_id) {
			panic!(
				"error[B0002]: ResMut<{}> in system {} conflicts with a previous ResMut<{0}> access. Consider removing the duplicate access.",
				std::any::type_name::<T>(), system_meta.name);
		} else if combined_access.has_read(component_id) {
			panic!(
				"error[B0002]: ResMut<{}> in system {} conflicts with a previous Res<{0}> access. Consider removing the duplicate access.",
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

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for ResMutState<T> {
	type Item = ResMut<'w, T>;

	#[inline]
	unsafe fn get_param(
		state: &'s mut Self,
		system_meta: &SystemMeta,
		world: &'w World,
		change_tick: u32,
	) -> Self::Item {
		let value = world
			.get_resource_unchecked_mut_with_id(state.component_id)
			.unwrap_or_else(|| {
				panic!(
					"Resource requested by {} does not exist: {}",
					system_meta.name,
					std::any::type_name::<T>()
				)
			});
		ResMut {
			value: value.value,
			ticks: Ticks {
				component_ticks: value.ticks.component_ticks,
				last_change_tick: system_meta.last_change_tick,
				change_tick,
			},
		}
	}
}

/// The [`SystemParamState`] of [`Option<ResMut<T>>`].
/// See: [`ResMut<T>`]
#[doc(hidden)]
pub struct OptionResMutState<T>(ResMutState<T>);

impl<'a, T: Resource> SystemParam for Option<ResMut<'a, T>> {
	type Fetch = OptionResMutState<T>;
}

// SAFETY: this impl defers to `ResMutState`, which initializes
// and validates the correct world access
unsafe impl<T: Resource> SystemParamState for OptionResMutState<T> {
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
		Self(ResMutState::init(world, system_meta))
	}
}

impl<'w, 's, T: Resource> SystemParamFetch<'w, 's> for OptionResMutState<T> {
	type Item = Option<ResMut<'w, T>>;

	#[inline]
	unsafe fn get_param(
		state: &'s mut Self,
		system_meta: &SystemMeta,
		world: &'w World,
		change_tick: u32,
	) -> Self::Item {
		world
			.get_resource_unchecked_mut_with_id(state.0.component_id)
			.map(|value| ResMut {
				value: value.value,
				ticks: Ticks {
					component_ticks: value.ticks.component_ticks,
					last_change_tick: system_meta.last_change_tick,
					change_tick,
				},
			})
	}
}
