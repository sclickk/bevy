//! Illustrates how to make a single system from multiple functions running in sequence and sharing
//! their inputs and outputs.

use anyhow::Result;
use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.insert_resource(Message("42".to_string()));
	app.add_system(parse_message_system.chain(handler_system));
	app.run();
}

#[derive(Deref)]
struct Message(String);

// this system produces a Result<usize> output by trying to parse the Message resource
fn parse_message_system(message: Res<Message>) -> Result<usize> {
	Ok(message.parse::<usize>()?)
}

// This system takes a Result<usize> input and either prints the parsed value or the error message
// Try changing the Message resource to something that isn't an integer. You should see the error
// message printed.
fn handler_system(In(result): In<Result<usize>>) {
	match result {
		Ok(value) => println!("parsed message: {}", value),
		Err(err) => println!("encountered an error: {:?}", err),
	}
}
