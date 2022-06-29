//! This example only enables a minimal set of plugins required for bevy to run.
//! You can also completely remove rendering / windowing Plugin code from bevy
//! by making your import look like this in your Cargo.toml.
//!
//! [dependencies]
//! bevy = { version = "*", default-features = false }
//! # replace "*" with the most recent version of bevy

use bevy::{app::ScheduleRunnerSettings, prelude::*, utils::Duration};

fn main() {
	// this app runs once
	let mut first_app = App::new();
	first_app.insert_resource(ScheduleRunnerSettings::run_once());
	first_app.add_plugins(MinimalPlugins);
	first_app.add_system(hello_world_system);
	first_app.run();

	// this app loops forever at 60 fps
	let mut second_app = App::new();
	second_app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
			1.0 / 60.0,
		)));
	second_app.add_plugins(MinimalPlugins);
	second_app.add_system(counter);
	second_app.run();
}

fn hello_world_system() {
	println!("hello world");
}

fn counter(mut state: Local<CounterState>) {
	if state.count % 60 == 0 {
		println!("{}", state.count);
	}
	state.count += 1;
}

#[derive(Default)]
struct CounterState {
	count: u32,
}
