use crate::{
	archetype::{Archetype, ArchetypeComponentId},
	component::{Component, ComponentId, ComponentStorage, ComponentTicks, StorageType},
	entity::Entity,
	query::{
		debug_checked_unreachable,
		fetch::{Fetch, FetchState, ReadOnlyWorldQuery, WorldQuery, WorldQueryGats},
		Access, FilteredAccess, QueryItem,
	},
	storage::{ComponentSparseSet, Table, Tables},
	world::World,
};

use bevy_ptr::{ThinSlicePtr, UnsafeCellDeref};
use std::{cell::UnsafeCell, marker::PhantomData};

/// [`WorldQuery`] that tracks changes and additions for component `T`.
///
/// Wraps a [`Component`] to track whether the component changed for the corresponding entities in
/// a query since the last time the system that includes these queries ran.
///
/// If you only care about entities that changed or that got added use the
/// [`Changed`](crate::query::Changed) and [`Added`](crate::query::Added) filters instead.
///
/// # Examples
///
/// ```
/// # use bevy_ecs::component::Component;
/// # use bevy_ecs::query::ChangeTrackers;
/// # use bevy_ecs::system::IntoSystem;
/// # use bevy_ecs::system::Query;
/// #
/// # #[derive(Component, Debug)]
/// # struct Name {};
/// # #[derive(Component)]
/// # struct Transform {};
/// #
/// fn print_moving_objects_system(query: Query<(&Name, ChangeTrackers<Transform>)>) {
///     for (name, tracker) in &query {
///         if tracker.is_changed() {
///             println!("Entity moved: {:?}", name);
///         } else {
///             println!("Entity stood still: {:?}", name);
///         }
///     }
/// }
/// # bevy_ecs::system::assert_is_system(print_moving_objects_system);
/// ```
#[derive(Clone)]
pub struct ChangeTrackers<T: Component> {
	pub(crate) component_ticks: ComponentTicks,
	// TODO: Fix code duplication with these!
	pub(crate) last_change_tick: u32,
	pub(crate) change_tick: u32,
	marker: PhantomData<T>,
}

impl<T: Component> std::fmt::Debug for ChangeTrackers<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ChangeTrackers")
			.field("component_ticks", &self.component_ticks)
			.field("last_change_tick", &self.last_change_tick)
			.field("change_tick", &self.change_tick)
			.finish()
	}
}

impl<T: Component> ChangeTrackers<T> {
	/// Returns true if this component has been added since the last execution of this system.
	pub fn is_added(&self) -> bool {
		self
			.component_ticks
			.is_added(self.last_change_tick, self.change_tick)
	}

	/// Returns true if this component has been changed since the last execution of this system.
	pub fn is_changed(&self) -> bool {
		self
			.component_ticks
			.is_changed(self.last_change_tick, self.change_tick)
	}
}

// SAFETY: `ROQueryFetch<Self>` is the same as `QueryFetch<Self>`
unsafe impl<T: Component> WorldQuery for ChangeTrackers<T> {
	type ReadOnly = Self;
	type State = ChangeTrackersState<T>;

	fn shrink<'wlong: 'wshort, 'wshort>(item: QueryItem<'wlong, Self>) -> QueryItem<'wshort, Self> {
		item
	}
}

/// The [`FetchState`] of [`ChangeTrackers`].
#[doc(hidden)]
pub struct ChangeTrackersState<T> {
	component_id: ComponentId,
	marker: PhantomData<T>,
}

impl<T: Component> FetchState for ChangeTrackersState<T> {
	fn init(world: &mut World) -> Self {
		let component_id = world.init_component::<T>();
		Self {
			component_id,
			marker: PhantomData,
		}
	}

	fn matches_component_set(&self, set_contains_id: &impl Fn(ComponentId) -> bool) -> bool {
		set_contains_id(self.component_id)
	}
}

/// The [`Fetch`] of [`ChangeTrackers`].
#[doc(hidden)]
pub struct ChangeTrackersFetch<'w, T> {
	// T::Storage = TableStorage
	table_ticks: Option<ThinSlicePtr<'w, UnsafeCell<ComponentTicks>>>,
	entity_table_rows: Option<ThinSlicePtr<'w, usize>>,
	// T::Storage = SparseStorage
	entities: Option<ThinSlicePtr<'w, Entity>>,
	sparse_set: Option<&'w ComponentSparseSet>,

	marker: PhantomData<T>,
	last_change_tick: u32,
	change_tick: u32,
}

impl<T> Clone for ChangeTrackersFetch<'_, T> {
	fn clone(&self) -> Self {
		Self {
			table_ticks: self.table_ticks,
			entity_table_rows: self.entity_table_rows,
			entities: self.entities,
			sparse_set: self.sparse_set,
			marker: self.marker,
			last_change_tick: self.last_change_tick,
			change_tick: self.change_tick,
		}
	}
}

/// SAFETY: access is read only
unsafe impl<T: Component> ReadOnlyWorldQuery for ChangeTrackers<T> {}

impl<'w, T: Component> WorldQueryGats<'w> for ChangeTrackers<T> {
	type Fetch = ChangeTrackersFetch<'w, T>;
	type _State = ChangeTrackersState<T>;
}

// SAFETY: component access and archetype component access are properly updated to reflect that T is
// read
unsafe impl<'w, T: Component> Fetch<'w> for ChangeTrackersFetch<'w, T> {
	type Item = ChangeTrackers<T>;
	type State = ChangeTrackersState<T>;

	const IS_DENSE: bool = {
		match T::Storage::STORAGE_TYPE {
			StorageType::Table => true,
			StorageType::SparseSet => false,
		}
	};

	const IS_ARCHETYPAL: bool = true;

	unsafe fn init(
		world: &'w World,
		state: &ChangeTrackersState<T>,
		last_change_tick: u32,
		change_tick: u32,
	) -> ChangeTrackersFetch<'w, T> {
		ChangeTrackersFetch {
			table_ticks: None,
			entities: None,
			entity_table_rows: None,
			sparse_set: (T::Storage::STORAGE_TYPE == StorageType::SparseSet).then(|| {
				world
					.storages()
					.sparse_sets
					.get(state.component_id)
					.unwrap()
			}),
			marker: PhantomData,
			last_change_tick,
			change_tick,
		}
	}

	#[inline]
	unsafe fn set_archetype(
		&mut self,
		state: &Self::State,
		archetype: &'w Archetype,
		tables: &'w Tables,
	) {
		match T::Storage::STORAGE_TYPE {
			StorageType::Table => {
				self.entity_table_rows = Some(archetype.entity_table_rows().into());
				let column = tables[archetype.table_id()]
					.get_column(state.component_id)
					.unwrap();
				self.table_ticks = Some(column.get_ticks_slice().into());
			},
			StorageType::SparseSet => self.entities = Some(archetype.entities().into()),
		}
	}

	#[inline]
	unsafe fn set_table(&mut self, state: &Self::State, table: &'w Table) {
		self.table_ticks = Some(
			table
				.get_column(state.component_id)
				.unwrap()
				.get_ticks_slice()
				.into(),
		);
	}

	#[inline]
	unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item {
		match T::Storage::STORAGE_TYPE {
			StorageType::Table => {
				let entity_table_rows = self
					.entity_table_rows
					.unwrap_or_else(|| debug_checked_unreachable());
				let table_row = *entity_table_rows.get(archetype_index);
				ChangeTrackers {
					component_ticks: {
						let table_ticks = self
							.table_ticks
							.unwrap_or_else(|| debug_checked_unreachable());
						table_ticks.get(table_row).read()
					},
					marker: PhantomData,
					last_change_tick: self.last_change_tick,
					change_tick: self.change_tick,
				}
			},
			StorageType::SparseSet => {
				let entities = self
					.entities
					.unwrap_or_else(|| debug_checked_unreachable());
				let entity = *entities.get(archetype_index);
				ChangeTrackers {
					component_ticks: self
						.sparse_set
						.unwrap_or_else(|| debug_checked_unreachable())
						.get_ticks(entity)
						.map(|ticks| &*ticks.get())
						.cloned()
						.unwrap_or_else(|| debug_checked_unreachable()),
					marker: PhantomData,
					last_change_tick: self.last_change_tick,
					change_tick: self.change_tick,
				}
			},
		}
	}

	#[inline]
	unsafe fn table_fetch(&mut self, table_row: usize) -> Self::Item {
		ChangeTrackers {
			component_ticks: {
				let table_ticks = self
					.table_ticks
					.unwrap_or_else(|| debug_checked_unreachable());
				table_ticks.get(table_row).read()
			},
			marker: PhantomData,
			last_change_tick: self.last_change_tick,
			change_tick: self.change_tick,
		}
	}

	fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
		assert!(
			!access.access().has_write(state.component_id),
			"ChangeTrackers<{}> conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
			std::any::type_name::<T>()
		);
		access.add_read(state.component_id);
	}

	fn update_archetype_component_access(
		state: &Self::State,
		archetype: &Archetype,
		access: &mut Access<ArchetypeComponentId>,
	) {
		if let Some(archetype_component_id) = archetype.get_archetype_component_id(state.component_id) {
			access.add_read(archetype_component_id);
		}
	}
}
