//! Demonstrates a startup system (one that runs once when the app starts up).

use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.add_startup_system(startup_system);
	app.add_system(normal_system);
	app.run();
}

/// Startup systems are run exactly once when the app starts up.
/// They run right before "normal" systems run.
fn startup_system() {
	println!("startup system ran first");
}

fn normal_system() {
	println!("normal system ran second");
}
