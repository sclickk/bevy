use crate::{
	component::{Component, ComponentId, ComponentStorage, StorageType},
	entity::Entity,
	query::{
		debug_checked_unreachable,
		fetch::{Archetype, ArchetypeComponentId, Fetch},
		Access, ComponentIdState, FilteredAccess,
	},
	storage::{ComponentSparseSet, Table, Tables},
	world::World,
};

use bevy_ptr::{ThinSlicePtr, UnsafeCellDeref};

use std::cell::UnsafeCell;

/// The [`Fetch`] of `&T`.
#[doc(hidden)]
pub struct ReadFetch<'w, T> {
	// T::Storage = TableStorage
	table_components: Option<ThinSlicePtr<'w, UnsafeCell<T>>>,
	entity_table_rows: Option<ThinSlicePtr<'w, usize>>,
	// T::Storage = SparseStorage
	entities: Option<ThinSlicePtr<'w, Entity>>,
	sparse_set: Option<&'w ComponentSparseSet>,
}

impl<T> Clone for ReadFetch<'_, T> {
	fn clone(&self) -> Self {
		Self {
			table_components: self.table_components,
			entity_table_rows: self.entity_table_rows,
			entities: self.entities,
			sparse_set: self.sparse_set,
		}
	}
}

// SAFETY: component access and archetype component access are properly updated to reflect that T is
// read
unsafe impl<'w, T: Component> Fetch<'w> for ReadFetch<'w, T> {
	type Item = &'w T;
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
		_last_change_tick: u32,
		_change_tick: u32,
	) -> ReadFetch<'w, T> {
		ReadFetch {
			table_components: None,
			entity_table_rows: None,
			entities: None,
			sparse_set: (T::Storage::STORAGE_TYPE == StorageType::SparseSet).then(|| {
				world
					.storages()
					.sparse_sets
					.get(state.component_id)
					.unwrap()
			}),
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
			},
			StorageType::SparseSet => self.entities = Some(archetype.entities().into()),
		}
	}

	#[inline]
	unsafe fn set_table(&mut self, state: &Self::State, table: &'w Table) {
		self.table_components = Some(
			table
				.get_column(state.component_id)
				.unwrap()
				.get_data_slice()
				.into(),
		);
	}

	#[inline]
	unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item {
		match T::Storage::STORAGE_TYPE {
			StorageType::Table => {
				let (entity_table_rows, table_components) = self
					.entity_table_rows
					.zip(self.table_components)
					.unwrap_or_else(|| debug_checked_unreachable());
				let table_row = *entity_table_rows.get(archetype_index);
				table_components.get(table_row).deref()
			},
			StorageType::SparseSet => {
				let (entities, sparse_set) = self
					.entities
					.zip(self.sparse_set)
					.unwrap_or_else(|| debug_checked_unreachable());
				let entity = *entities.get(archetype_index);
				sparse_set
					.get(entity)
					.unwrap_or_else(|| debug_checked_unreachable())
					.deref::<T>()
			},
		}
	}

	#[inline]
	unsafe fn table_fetch(&mut self, table_row: usize) -> Self::Item {
		let components = self
			.table_components
			.unwrap_or_else(|| debug_checked_unreachable());
		components.get(table_row).deref()
	}

	fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
		assert!(
			!access.access().has_write(state.component_id),
			"&{} conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
			std::any::type_name::<T>(),
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
