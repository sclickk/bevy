use crate::{
	archetype::Archetype,
	change_detection::Mut,
	component::{Component, ComponentId, ComponentStorage, StorageType},
	entity::Entity,
	query::{
		debug_checked_unreachable,
		fetch::{ArchetypeComponentId, ComponentTicks, Fetch, Ticks},
		Access, ComponentIdState, FilteredAccess,
	},
	storage::{ComponentSparseSet, Table, Tables},
	world::World,
};

use bevy_ptr::{ThinSlicePtr, UnsafeCellDeref};

use std::cell::UnsafeCell;

/// The [`Fetch`] of `&mut T`.
#[doc(hidden)]
pub struct WriteFetch<'w, T> {
	// T::Storage = TableStorage
	table_components: Option<ThinSlicePtr<'w, UnsafeCell<T>>>,
	table_ticks: Option<ThinSlicePtr<'w, UnsafeCell<ComponentTicks>>>,
	entity_table_rows: Option<ThinSlicePtr<'w, usize>>,
	// T::Storage = SparseStorage
	entities: Option<ThinSlicePtr<'w, Entity>>,
	sparse_set: Option<&'w ComponentSparseSet>,

	// TODO: Fix code duplication in Ticks in change_detection.rs
	last_change_tick: u32,
	change_tick: u32,
}

impl<T> Clone for WriteFetch<'_, T> {
	fn clone(&self) -> Self {
		Self {
			table_components: self.table_components,
			table_ticks: self.table_ticks,
			entities: self.entities,
			entity_table_rows: self.entity_table_rows,
			sparse_set: self.sparse_set,
			last_change_tick: self.last_change_tick,
			change_tick: self.change_tick,
		}
	}
}

/// SAFETY: component access and archetype component access are properly updated to reflect that `T` is
/// read and write
unsafe impl<'w, T: Component> Fetch<'w> for WriteFetch<'w, T> {
	type Item = Mut<'w, T>;
	type State = ComponentIdState<T>;

	const IS_DENSE: bool = {
		match T::Storage::STORAGE_TYPE {
			StorageType::Table => true,
			StorageType::SparseSet => false,
		}
	};

	const IS_ARCHETYPAL: bool = true;

	unsafe fn init(
		world: &'w World,
		state: &ComponentIdState<T>,
		last_change_tick: u32,
		change_tick: u32,
	) -> Self {
		Self {
			table_components: None,
			entities: None,
			entity_table_rows: None,
			sparse_set: (T::Storage::STORAGE_TYPE == StorageType::SparseSet).then(|| {
				world
					.storages()
					.sparse_sets
					.get(state.component_id)
					.unwrap()
			}),
			table_ticks: None,
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
				self.table_components = Some(column.get_data_slice().into());
				self.table_ticks = Some(column.get_ticks_slice().into());
			},
			StorageType::SparseSet => self.entities = Some(archetype.entities().into()),
		}
	}

	#[inline]
	unsafe fn set_table(&mut self, state: &Self::State, table: &'w Table) {
		let column = table.get_column(state.component_id).unwrap();
		self.table_components = Some(column.get_data_slice().into());
		self.table_ticks = Some(column.get_ticks_slice().into());
	}

	#[inline]
	unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item {
		match T::Storage::STORAGE_TYPE {
			StorageType::Table => {
				let (entity_table_rows, (table_components, table_ticks)) = self
					.entity_table_rows
					.zip(self.table_components.zip(self.table_ticks))
					.unwrap_or_else(|| debug_checked_unreachable());
				let table_row = *entity_table_rows.get(archetype_index);
				Mut {
					value: table_components.get(table_row).deref_mut(),
					ticks: Ticks {
						component_ticks: table_ticks.get(table_row).deref_mut(),
						change_tick: self.change_tick,
						last_change_tick: self.last_change_tick,
					},
				}
			},
			StorageType::SparseSet => {
				let (entities, sparse_set) = self
					.entities
					.zip(self.sparse_set)
					.unwrap_or_else(|| debug_checked_unreachable());
				let entity = *entities.get(archetype_index);
				let (component, component_ticks) = sparse_set
					.get_with_ticks(entity)
					.unwrap_or_else(|| debug_checked_unreachable());
				Mut {
					value: component.assert_unique().deref_mut(),
					ticks: Ticks {
						component_ticks: component_ticks.deref_mut(),
						change_tick: self.change_tick,
						last_change_tick: self.last_change_tick,
					},
				}
			},
		}
	}

	#[inline]
	unsafe fn table_fetch(&mut self, table_row: usize) -> Self::Item {
		let (table_components, table_ticks) = self
			.table_components
			.zip(self.table_ticks)
			.unwrap_or_else(|| debug_checked_unreachable());
		Mut {
			value: table_components.get(table_row).deref_mut(),
			ticks: Ticks {
				component_ticks: table_ticks.get(table_row).deref_mut(),
				change_tick: self.change_tick,
				last_change_tick: self.last_change_tick,
			},
		}
	}

	fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
		assert!(
			!access.access().has_read(state.component_id),
			"&mut {} conflicts with a previous access in this query. Mutable component access must be unique.",
			std::any::type_name::<T>(),
		);
		access.add_write(state.component_id);
	}

	fn update_archetype_component_access(
		state: &Self::State,
		archetype: &Archetype,
		access: &mut Access<ArchetypeComponentId>,
	) {
		if let Some(archetype_component_id) = archetype.get_archetype_component_id(state.component_id) {
			access.add_write(archetype_component_id);
		}
	}
}
