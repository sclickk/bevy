//! Demonstrates how reflection is used with generic Rust types.

use bevy::{prelude::*, reflect::TypeRegistry};
use std::any::TypeId;

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	// You must manually register each instance of a generic type
	app.register_type::<MyType<u32>>();
	app.add_startup_system(setup);
	app.run();
}

#[derive(Reflect)]
struct MyType<T: Reflect> {
	value: T,
}

fn setup(type_registry: Res<TypeRegistry>) {
	let type_registry = type_registry.read();

	let registration = type_registry
		.get(TypeId::of::<MyType<u32>>())
		.unwrap();
	info!("Registration for {} exists", registration.short_name());

	// MyType<String> was not manually registered, so it does not exist
	assert!(type_registry
		.get(TypeId::of::<MyType<String>>())
		.is_none());
}
