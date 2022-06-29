//! This example illustrates how to customize the thread pool used internally (e.g. to only use a
//! certain number of threads).

use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.insert_resource(DefaultTaskPoolOptions::with_num_threads(4));
	app.add_plugins(DefaultPlugins);
	app.run();
}
