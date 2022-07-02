//! This example creates a custom [`SystemParam`] struct that counts the number of players.

use bevy::{ecs::system::SystemParam, prelude::*};

fn main() {
	let mut app = App::new();
	app.insert_resource(PlayerCount(0));
	app.add_startup_system(spawn);
	app.add_system(count_players);
	app.run();
}

#[derive(Component)]
pub struct Player;
#[derive(Component)]
pub struct PlayerCount(usize);

/// The [`SystemParam`] struct can contain any types that can also be included in a
/// system function signature.
///
/// In this example, it includes a query and a mutable resource.
#[derive(SystemParam)]
struct PlayerCounter<'w, 's> {
	players: Query<'w, 's, &'static Player>,
	count: ResMut<'w, PlayerCount>,
}

impl<'w, 's> PlayerCounter<'w, 's> {
	fn count(&mut self) {
		self.count.0 = self.players.into_iter().len();
	}
}

/// Spawn some players to count
fn spawn(mut commands: Commands) {
	commands.spawn().insert(Player);
	commands.spawn().insert(Player);
	commands.spawn().insert(Player);
}

/// The [`SystemParam`] can be used directly in a system argument.
fn count_players(mut counter: PlayerCounter) {
	counter.count();

	println!("{} players in the game", counter.count.0);
}
