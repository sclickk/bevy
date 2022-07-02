//! This example demonstrates you can create a custom runner (to update an app manually). It reads
//! lines from stdin and prints them from within the ecs.

use bevy::prelude::*;
use std::io;

struct Input(String);

fn my_runner(mut app: App) {
	println!("Type stuff into the console");
	for line in io::stdin().lines() {
		{
			let mut input = app.world.resource_mut::<Input>();
			input.0 = line.unwrap();
		}
		app.update();
	}
}

fn print_system(input: Res<Input>) {
	println!("You typed: {}", input.0);
}

fn main() {
	let mut app = App::new();
	app.insert_resource(Input(String::new()));
	app.set_runner(my_runner);
	app.add_system(print_system);
	app.run();
}
