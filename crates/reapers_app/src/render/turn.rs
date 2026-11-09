//! Turns as quaternions `[x, y, z, w]`, the way the host takes them, for
//! the parts the game moves itself. Only what the first-person view and its
//! effects need.

use super::camera::Point;

/// The turn by `angle` radians about the unit `axis`.
pub fn axis_angle(axis: Point, angle: f32) -> [f32; 4] {
	let (sin, cos) = (angle * 0.5).sin_cos();
	[axis[0] * sin, axis[1] * sin, axis[2] * sin, cos]
}

/// `a` after `b`: `b` is applied first.
pub fn multiply(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
	let [ax, ay, az, aw] = a;
	let [bx, by, bz, bw] = b;
	[
		aw * bx + ax * bw + ay * bz - az * by,
		aw * by - ax * bz + ay * bw + az * bx,
		aw * bz + ax * by - ay * bx + az * bw,
		aw * bw - ax * bx - ay * by - az * bz,
	]
}

/// The turn that undoes `q`.
pub fn inverse(q: [f32; 4]) -> [f32; 4] {
	[-q[0], -q[1], -q[2], q[3]]
}

/// A view's turn: `yaw` turns to the right, `pitch` up and `roll` clockwise
/// as the eye sees it, in that order, in a frame where the eye looks down
/// -Z with +Y up.
pub fn view_turn(yaw: f32, pitch: f32, roll: f32) -> [f32; 4] {
	let yaw = axis_angle([0.0, 1.0, 0.0], -yaw);
	let pitch = axis_angle([1.0, 0.0, 0.0], pitch);
	let roll = axis_angle([0.0, 0.0, 1.0], -roll);
	multiply(yaw, multiply(pitch, roll))
}

/// Where `point` goes when turned by `q` about the origin.
pub fn rotate(q: [f32; 4], point: Point) -> Point {
	super::camera::rotate(q, point)
}

/// The turn that carries +Y onto the unit vector `to`.
pub fn up_to(to: Point) -> [f32; 4] {
	let dot = to[1].clamp(-1.0, 1.0);
	if dot < -0.9999 {
		return [1.0, 0.0, 0.0, 0.0];
	}
	// Half way between +Y and `to`, as a quaternion: [Y × to, 1 + Y·to].
	let q = [to[2], 0.0, -to[0], 1.0 + dot];
	let length = q.iter().map(|value| value * value).sum::<f32>().sqrt();
	q.map(|value| value / length)
}

pub fn add(a: Point, b: Point) -> Point {
	[a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: Point, b: Point) -> Point {
	[a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn scale(a: Point, by: f32) -> Point {
	a.map(|value| value * by)
}

pub fn length(a: Point) -> f32 {
	(a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

pub fn lerp(a: Point, b: Point, t: f32) -> Point {
	add(a, scale(sub(b, a), t))
}

#[cfg(test)]
mod tests {
	use super::*;

	fn close(a: Point, b: Point) {
		assert!(length(sub(a, b)) < 1.0e-5, "{a:?} is not {b:?}");
	}

	#[test]
	fn view_turns_go_the_way_they_say() {
		let ahead = [0.0, 0.0, -1.0];
		close(
			rotate(view_turn(std::f32::consts::FRAC_PI_2, 0.0, 0.0), ahead),
			[1.0, 0.0, 0.0],
		);
		close(
			rotate(view_turn(0.0, std::f32::consts::FRAC_PI_2, 0.0), ahead),
			[0.0, 1.0, 0.0],
		);
		// Clockwise as the eye sees it carries up to the right.
		close(
			rotate(
				view_turn(0.0, 0.0, std::f32::consts::FRAC_PI_2),
				[0.0, 1.0, 0.0],
			),
			[1.0, 0.0, 0.0],
		);
	}

	#[test]
	fn up_to_carries_up_onto_any_direction() {
		for to in [
			[1.0, 0.0, 0.0],
			[0.0, 0.0, -1.0],
			[0.0, -1.0, 0.0],
			[0.6, 0.0, 0.8],
		] {
			close(rotate(up_to(to), [0.0, 1.0, 0.0]), to);
		}
		let q = view_turn(0.3, -0.2, 0.7);
		close(
			rotate(multiply(inverse(q), q), [0.2, 0.5, 0.1]),
			[0.2, 0.5, 0.1],
		);
	}
}
