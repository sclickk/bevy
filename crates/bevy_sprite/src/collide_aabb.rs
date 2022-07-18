//! Utilities for detecting if and on which side two axis-aligned bounding boxes (AABB) collide.

use bevy_math::{Vec2, Vec3};

#[derive(Debug)]
pub enum Collision {
	Left,
	Right,
	Top,
	Bottom,
	Inside,
}

// TODO: ideally we can remove this once bevy gets a physics system
/// Axis-aligned bounding box collision with "side" detection
/// * `a_pos` and `b_pos` are the center positions of the rectangles, typically obtained by
/// extracting the `translation` field from a `Transform` component
/// * `a_size` and `b_size` are the dimensions (width and height) of the rectangles.
pub fn collide(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> Option<Collision> {
	let a = (a_pos.truncate() - a_size / 2.0)..(a_pos.truncate() + a_size / 2.0);
	let b = (b_pos.truncate() - b_size / 2.0)..(b_pos.truncate() + b_size / 2.0);

	let cmp_1 = [
		a.start.x < b.end.x,
		b.start.x < a.end.x,
		a.start.y < b.end.y,
		b.start.y < a.end.y,
	];

	// check to see if the two rectangles are intersecting
	(!cmp_1.into_iter().any(|x| !x)).then(|| {
		// check to see if we hit on the left or right side
		let (x_collision, x_depth) = if a.start.x < b.start.x && a.end.x < b.end.x {
			(Collision::Left, b.start.x - a.end.x)
		} else if a.start.x > b.start.x && a.end.x > b.end.x {
			(Collision::Right, a.start.x - b.end.x)
		} else {
			(Collision::Inside, -f32::INFINITY)
		};

		// check to see if we hit on the top or bottom side
		let (y_collision, y_depth) = if a.start.y < b.start.y && a.end.y < b.end.y {
			(Collision::Bottom, b.start.y - a.end.y)
		} else if a.start.y > b.start.y && a.end.y > b.end.y {
			(Collision::Top, a.start.y - b.end.y)
		} else {
			(Collision::Inside, -f32::INFINITY)
		};

		// if we had an "x" and a "y" collision, pick the "primary" side using penetration depth
		if y_depth.abs() < x_depth.abs() {
			y_collision
		} else {
			x_collision
		}
	})
}
