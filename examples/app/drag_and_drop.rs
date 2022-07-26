//! An example that shows how to handle drag and drop of files in an app.

use bevy::prelude::*;

fn main() {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins);
	app.add_system(file_drag_and_drop_system);
	app.run();
}

fn file_drag_and_drop_system(mut events: EventReader<FileDragAndDrop>) {
	for event in events.iter() {
		info!("{:?}", event);
	}
}
