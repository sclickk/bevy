use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.add_system(hello_world_system);
	app.run();
}

fn hello_world_system() {
	println!("hello world");
}
