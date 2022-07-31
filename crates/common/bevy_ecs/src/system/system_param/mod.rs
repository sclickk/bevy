mod change_tick;
pub use change_tick::*;

mod local;
pub use local::*;

mod non_send;
pub use non_send::*;

mod resource;
pub use resource::*;

mod static_param;
pub use static_param::*;

pub use crate::change_detection::{NonSendMut, ResMut};
use crate::{
	archetype::{Archetype, Archetypes},
	bundle::Bundles,
	component::{Component, ComponentId, Components},
	entity::{Entities, Entity},
	query::{Access, FilteredAccess, FilteredAccessSet, QueryState, ReadOnlyWorldQuery, WorldQuery},
	system::{CommandQueue, Commands, Query, SystemMeta},
	world::World,
};
pub use bevy_ecs_macros::SystemParam;
use bevy_ecs_macros::{all_tuples, impl_param_set};
use std::marker::PhantomData;

/// A parameter that can be used in a [`System`](super::System).
///
/// # Derive
///
/// This trait can be derived with the [`derive@super::SystemParam`] macro.
/// This macro only works if each field on the derived struct implements [`SystemParam`].
/// Note: There are additional requirements on the field types.
/// See the *Generic `SystemParam`s* section for details and workarounds of the probable
/// cause if this derive causes an error to be emitted.
///
///
/// The struct for which `SystemParam` is derived must (currently) have exactly
/// two lifetime parameters.
/// The first is the lifetime of the world, and the second the lifetime
/// of the parameter's state.
///
/// ## Attributes
///
/// `#[system_param(ignore)]`:
/// Can be added to any field in the struct. Fields decorated with this attribute
/// will created with the default value upon realisation.
/// This is most useful for `PhantomData` fields, to ensure that the required lifetimes are
/// used, as shown in the example.
///
/// # Example
///
/// ```
/// # use bevy_ecs::prelude::*;
/// use std::marker::PhantomData;
/// use bevy_ecs::system::SystemParam;
///
/// #[derive(SystemParam)]
/// struct MyParam<'w, 's> {
///     foo: Res<'w, usize>,
///     #[system_param(ignore)]
///     marker: PhantomData<&'s usize>,
/// }
///
/// fn my_system(param: MyParam) {
///     // Access the resource through `param.foo`
/// }
///
/// # bevy_ecs::system::assert_is_system(my_system);
/// ```
///
/// # Generic `SystemParam`s
///
/// When using the derive macro, you may see an error in the form of:
///
/// ```text
/// expected ... [ParamType]
/// found associated type `<<[ParamType] as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item`
/// ```
/// where `[ParamType]` is the type of one of your fields.
/// To solve this error, you can wrap the field of type `[ParamType]` with [`StaticSystemParam`]
/// (i.e. `StaticSystemParam<[ParamType]>`).
///
/// ## Details
///
/// The derive macro requires that the [`SystemParam`] implementation of
/// each field `F`'s [`Fetch`](`SystemParam::Fetch`)'s [`Item`](`SystemParamFetch::Item`) is itself `F`
/// (ignoring lifetimes for simplicity).
/// This assumption is due to type inference reasons, so that the derived [`SystemParam`] can be
/// used as an argument to a function system.
/// If the compiler cannot validate this property for `[ParamType]`, it will error in the form shown above.
///
/// This will most commonly occur when working with `SystemParam`s generically, as the requirement
/// has not been proven to the compiler.
pub trait SystemParam: Sized {
	type Fetch: for<'w, 's> SystemParamFetch<'w, 's>;
}

pub type SystemParamItem<'w, 's, P> = <<P as SystemParam>::Fetch as SystemParamFetch<'w, 's>>::Item;

/// The state of a [`SystemParam`].
///
/// # Safety
///
/// It is the implementor's responsibility to ensure `system_meta` is populated with the _exact_
/// [`World`] access used by the [`SystemParamState`] (and associated [`SystemParamFetch`]).
/// Additionally, it is the implementor's responsibility to ensure there is no
/// conflicting access across all [`SystemParam`]'s.
pub unsafe trait SystemParamState: Send + Sync + 'static {
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self;
	#[inline]
	fn new_archetype(&mut self, _archetype: &Archetype, _system_meta: &mut SystemMeta) {}
	#[inline]
	fn apply(&mut self, _world: &mut World) {}
}

/// A [`SystemParamFetch`] that only reads a given [`World`].
///
/// # Safety
/// This must only be implemented for [`SystemParamFetch`] impls that exclusively read the World passed in to [`SystemParamFetch::get_param`]
pub unsafe trait ReadOnlySystemParamFetch {}

pub trait SystemParamFetch<'world, 'state>: SystemParamState {
	type Item: SystemParam<Fetch = Self>;
	/// # Safety
	///
	/// This call might access any of the input parameters in an unsafe way. Make sure the data
	/// access is safe in the context of the system scheduler.
	unsafe fn get_param(
		state: &'state mut Self,
		system_meta: &SystemMeta,
		world: &'world World,
		change_tick: u32,
	) -> Self::Item;
}

impl<'w, 's, Q: WorldQuery + 'static, F: WorldQuery + 'static> SystemParam for Query<'w, 's, Q, F> {
	type Fetch = QueryState<Q, F>;
}

// SAFETY: QueryState is constrained to read-only fetches, so it only reads World.
unsafe impl<Q: ReadOnlyWorldQuery, F: WorldQuery> ReadOnlySystemParamFetch for QueryState<Q, F> {}

// SAFETY: Relevant query ComponentId and ArchetypeComponentId access is applied to SystemMeta. If
// this QueryState conflicts with any prior access, a panic will occur.
unsafe impl<Q: WorldQuery + 'static, F: WorldQuery + 'static> SystemParamState
	for QueryState<Q, F>
{
	fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self {
		let state = QueryState::new(world);
		assert_component_access_compatibility(
			&system_meta.name,
			std::any::type_name::<Q>(),
			std::any::type_name::<F>(),
			&system_meta.component_access_set,
			&state.component_access,
			world,
		);
		system_meta
			.component_access_set
			.add(state.component_access.clone());
		system_meta
			.archetype_component_access
			.extend(&state.archetype_component_access);
		state
	}

	fn new_archetype(&mut self, archetype: &Archetype, system_meta: &mut SystemMeta) {
		self.new_archetype(archetype);
		system_meta
			.archetype_component_access
			.extend(&self.archetype_component_access);
	}
}

impl<'w, 's, Q: WorldQuery + 'static, F: WorldQuery + 'static> SystemParamFetch<'w, 's>
	for QueryState<Q, F>
{
	type Item = Query<'w, 's, Q, F>;

	#[inline]
	unsafe fn get_param(
		state: &'s mut Self,
		system_meta: &SystemMeta,
		world: &'w World,
		change_tick: u32,
	) -> Self::Item {
		Query::new(world, state, system_meta.last_change_tick, change_tick)
	}
}

fn assert_component_access_compatibility(
	system_name: &str,
	query_type: &'static str,
	filter_type: &'static str,
	system_access: &FilteredAccessSet<ComponentId>,
	current: &FilteredAccess<ComponentId>,
	world: &World,
) {
	let mut conflicts = system_access.get_conflicts_single(current);
	if conflicts.is_empty() {
		return;
	}
	let conflicting_components = conflicts
		.drain(..)
		.map(|component_id| {
			world
				.components
				.get_info(component_id)
				.unwrap()
				.name()
		})
		.collect::<Vec<&str>>();
	let accesses = conflicting_components.join(", ");
	panic!(
		"error[B0001]: Query<{}, {}> in system {} accesses component(s) {} in a way that conflicts with a previous system parameter. Consider using `Without<T>` to create disjoint Queries or merging conflicting Queries into a `ParamSet`.",
		query_type, filter_type, system_name, accesses
	);
}

pub struct ParamSet<'w, 's, T: SystemParam> {
	param_states: &'s mut T::Fetch,
	world: &'w World,
	system_meta: SystemMeta,
	change_tick: u32,
}
/// The [`SystemParamState`] of [`ParamSet<T::Item>`].
pub struct ParamSetState<T: for<'w, 's> SystemParamFetch<'w, 's>>(T);

impl_param_set!();

impl<'w, 's> SystemParam for Commands<'w, 's> {
	type Fetch = CommandQueue;
}

// SAFETY: Commands only accesses internal state
unsafe impl ReadOnlySystemParamFetch for CommandQueue {}

// SAFETY: only local state is accessed
unsafe impl SystemParamState for CommandQueue {
	fn init(_world: &mut World, _system_meta: &mut SystemMeta) -> Self {
		Default::default()
	}

	fn apply(&mut self, world: &mut World) {
		self.apply(world);
	}
}

impl<'w, 's> SystemParamFetch<'w, 's> for CommandQueue {
	type Item = Commands<'w, 's>;

	#[inline]
	unsafe fn get_param(
		state: &'s mut Self,
		_system_meta: &SystemMeta,
		world: &'w World,
		_change_tick: u32,
	) -> Self::Item {
		Commands::new(state, world)
	}
}

/// SAFETY: only reads world
unsafe impl ReadOnlySystemParamFetch for WorldState {}

/// The [`SystemParamState`] of [`&World`](crate::world::World).
#[doc(hidden)]
pub struct WorldState;

impl<'w> SystemParam for &'w World {
	type Fetch = WorldState;
}

// SAFETY: `read_all` access is set and conflicts result in a panic
unsafe impl SystemParamState for WorldState {
	fn init(_world: &mut World, system_meta: &mut SystemMeta) -> Self {
		fn compatability_error() -> ! {
			panic!("&World conflicts with a previous mutable system parameter. Allowing this would break Rust's mutability rules")
		}

		let mut access = Access::default();
		access.read_all();

		(!system_meta
			.archetype_component_access
			.is_compatible(&access))
		.then(compatability_error);

		system_meta
			.archetype_component_access
			.extend(&access);

		let mut filtered_access = FilteredAccess::default();

		filtered_access.read_all();
		(!system_meta
			.component_access_set
			.get_conflicts_single(&filtered_access)
			.is_empty())
		.then(compatability_error);

		system_meta
			.component_access_set
			.add(filtered_access);

		WorldState
	}
}

impl<'w, 's> SystemParamFetch<'w, 's> for WorldState {
	type Item = &'w World;
	unsafe fn get_param(
		_state: &'s mut Self,
		_system_meta: &SystemMeta,
		world: &'w World,
		_change_tick: u32,
	) -> Self::Item {
		world
	}
}

/// A [`SystemParam`] that grants access to the entities that had their `T` [`Component`] removed.
///
/// Note that this does not allow you to see which data existed before removal.
/// If you need this, you will need to track the component data value on your own,
/// using a regularly scheduled system that requests `Query<(Entity, &T), Changed<T>>`
/// and stores the data somewhere safe to later cross-reference.
///
/// If you are using `bevy_ecs` as a standalone crate,
/// note that the `RemovedComponents` list will not be automatically cleared for you,
/// and will need to be manually flushed using [`World::clear_trackers`]
///
/// For users of `bevy` itself, this is automatically done in a system added by `MinimalPlugins`
/// or `DefaultPlugins` at the end of each pass of the game loop during the `CoreStage::Last`
/// stage. As such `RemovedComponents` systems should be scheduled after the stage where
/// removal occurs but before `CoreStage::Last`.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// # use bevy_ecs::component::Component;
/// # use bevy_ecs::system::IntoSystem;
/// # use bevy_ecs::system::RemovedComponents;
/// #
/// # #[derive(Component)]
/// # struct MyComponent;
///
/// fn react_on_removal(removed: RemovedComponents<MyComponent>) {
///     removed.iter().for_each(|removed_entity| println!("{:?}", removed_entity));
/// }
///
/// # bevy_ecs::system::assert_is_system(react_on_removal);
/// ```
pub struct RemovedComponents<'a, T: Component> {
	world: &'a World,
	component_id: ComponentId,
	marker: PhantomData<T>,
}

impl<'a, T: Component> RemovedComponents<'a, T> {
	/// Returns an iterator over the entities that had their `T` [`Component`] removed.
	pub fn iter(&self) -> std::iter::Cloned<std::slice::Iter<'_, Entity>> {
		self.world.removed_with_id(self.component_id)
	}
}

// SAFETY: Only reads World components
unsafe impl<T: Component> ReadOnlySystemParamFetch for RemovedComponentsState<T> {}

/// The [`SystemParamState`] of [`RemovedComponents<T>`].
#[doc(hidden)]
pub struct RemovedComponentsState<T> {
	component_id: ComponentId,
	marker: PhantomData<T>,
}

impl<'a, T: Component> SystemParam for RemovedComponents<'a, T> {
	type Fetch = RemovedComponentsState<T>;
}

// SAFETY: no component access. removed component entity collections can be read in parallel and are
// never mutably borrowed during system execution
unsafe impl<T: Component> SystemParamState for RemovedComponentsState<T> {
	fn init(world: &mut World, _system_meta: &mut SystemMeta) -> Self {
		Self {
			component_id: world.init_component::<T>(),
			marker: PhantomData,
		}
	}
}

impl<'w, 's, T: Component> SystemParamFetch<'w, 's> for RemovedComponentsState<T> {
	type Item = RemovedComponents<'w, T>;

	#[inline]
	unsafe fn get_param(
		state: &'s mut Self,
		_system_meta: &SystemMeta,
		world: &'w World,
		_change_tick: u32,
	) -> Self::Item {
		RemovedComponents {
			world,
			component_id: state.component_id,
			marker: PhantomData,
		}
	}
}

impl<'a> SystemParam for &'a Archetypes {
	type Fetch = ArchetypesState;
}

// SAFETY: Only reads World archetypes
unsafe impl ReadOnlySystemParamFetch for ArchetypesState {}

/// The [`SystemParamState`] of [`Archetypes`].
#[doc(hidden)]
pub struct ArchetypesState;

// SAFETY: no component value access
unsafe impl SystemParamState for ArchetypesState {
	fn init(_world: &mut World, _system_meta: &mut SystemMeta) -> Self {
		Self
	}
}

impl<'w, 's> SystemParamFetch<'w, 's> for ArchetypesState {
	type Item = &'w Archetypes;

	#[inline]
	unsafe fn get_param(
		_state: &'s mut Self,
		_system_meta: &SystemMeta,
		world: &'w World,
		_change_tick: u32,
	) -> Self::Item {
		world.archetypes()
	}
}

impl<'a> SystemParam for &'a Components {
	type Fetch = ComponentsState;
}

// SAFETY: Only reads World components
unsafe impl ReadOnlySystemParamFetch for ComponentsState {}

/// The [`SystemParamState`] of [`Components`].
#[doc(hidden)]
pub struct ComponentsState;

// SAFETY: no component value access
unsafe impl SystemParamState for ComponentsState {
	fn init(_world: &mut World, _system_meta: &mut SystemMeta) -> Self {
		Self
	}
}

impl<'w, 's> SystemParamFetch<'w, 's> for ComponentsState {
	type Item = &'w Components;

	#[inline]
	unsafe fn get_param(
		_state: &'s mut Self,
		_system_meta: &SystemMeta,
		world: &'w World,
		_change_tick: u32,
	) -> Self::Item {
		world.components()
	}
}

impl<'a> SystemParam for &'a Entities {
	type Fetch = EntitiesState;
}

// SAFETY: Only reads World entities
unsafe impl ReadOnlySystemParamFetch for EntitiesState {}

/// The [`SystemParamState`] of [`Entities`].
#[doc(hidden)]
pub struct EntitiesState;

// SAFETY: no component value access
unsafe impl SystemParamState for EntitiesState {
	fn init(_world: &mut World, _system_meta: &mut SystemMeta) -> Self {
		Self
	}
}

impl<'w, 's> SystemParamFetch<'w, 's> for EntitiesState {
	type Item = &'w Entities;

	#[inline]
	unsafe fn get_param(
		_state: &'s mut Self,
		_system_meta: &SystemMeta,
		world: &'w World,
		_change_tick: u32,
	) -> Self::Item {
		world.entities()
	}
}

impl<'a> SystemParam for &'a Bundles {
	type Fetch = BundlesState;
}

// SAFETY: Only reads World bundles
unsafe impl ReadOnlySystemParamFetch for BundlesState {}

/// The [`SystemParamState`] of [`Bundles`].
#[doc(hidden)]
pub struct BundlesState;

// SAFETY: no component value access
unsafe impl SystemParamState for BundlesState {
	fn init(_world: &mut World, _system_meta: &mut SystemMeta) -> Self {
		Self
	}
}

impl<'w, 's> SystemParamFetch<'w, 's> for BundlesState {
	type Item = &'w Bundles;

	#[inline]
	unsafe fn get_param(
		_state: &'s mut Self,
		_system_meta: &SystemMeta,
		world: &'w World,
		_change_tick: u32,
	) -> Self::Item {
		world.bundles()
	}
}

macro_rules! impl_system_param_tuple {
	($($param: ident),*) => {
		impl<$($param: SystemParam),*> SystemParam for ($($param,)*) {
			type Fetch = ($($param::Fetch,)*);
		}

		// SAFETY: tuple consists only of ReadOnlySystemParamFetches
		unsafe impl<$($param: ReadOnlySystemParamFetch),*> ReadOnlySystemParamFetch for ($($param,)*) {}

		#[allow(unused_variables)]
		#[allow(non_snake_case)]
		impl<'w, 's, $($param: SystemParamFetch<'w, 's>),*> SystemParamFetch<'w, 's> for ($($param,)*) {
			type Item = ($($param::Item,)*);

			#[inline]
			#[allow(clippy::unused_unit)]
			unsafe fn get_param(
				state: &'s mut Self,
				system_meta: &SystemMeta,
				world: &'w World,
				change_tick: u32,
			) -> Self::Item {

				let ($($param,)*) = state;
				($($param::get_param($param, system_meta, world, change_tick),)*)
			}
		}

		// SAFETY: implementors of each `SystemParamState` in the tuple have validated their impls
		#[allow(clippy::undocumented_unsafe_blocks)] // false positive by clippy
		#[allow(non_snake_case)]
		unsafe impl<$($param: SystemParamState),*> SystemParamState for ($($param,)*) {
			#[inline]
			fn init(_world: &mut World, _system_meta: &mut SystemMeta) -> Self {
				(($($param::init(_world, _system_meta),)*))
			}

			#[inline]
			fn new_archetype(&mut self, _archetype: &Archetype, _system_meta: &mut SystemMeta) {
				let ($($param,)*) = self;
				$($param.new_archetype(_archetype, _system_meta);)*
			}

			#[inline]
			fn apply(&mut self, _world: &mut World) {
				let ($($param,)*) = self;
				$($param.apply(_world);)*
			}
		}
	};
}

all_tuples!(impl_system_param_tuple, 0, 16, P);

pub mod lifetimeless {
	pub type SQuery<Q, F = ()> = super::Query<'static, 'static, Q, F>;
	pub type Read<T> = &'static T;
	pub type Write<T> = &'static mut T;
	pub type SRes<T> = super::Res<'static, T>;
	pub type SResMut<T> = super::ResMut<'static, T>;
	pub type SCommands = crate::system::Commands<'static, 'static>;
}

#[cfg(test)]
mod tests {
	use super::SystemParam;
	use crate::{
		self as bevy_ecs, // Necessary for the `SystemParam` Derive when used inside `bevy_ecs`.
		query::WorldQuery,
		system::Query,
	};

	// Compile test for #2838
	#[derive(SystemParam)]
	pub struct SpecialQuery<
		'w,
		's,
		Q: WorldQuery + Send + Sync + 'static,
		F: WorldQuery + Send + Sync + 'static = (),
	> {
		_query: Query<'w, 's, Q, F>,
	}
}
