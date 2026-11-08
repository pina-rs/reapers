//! Where the stage's camera stands, and the small amount of 3D arithmetic
//! the views need to place things: the host does the rest.

use leptoskit::scene3d::Op;
use leptoskit::scene3d::Transform;
use reapers_rules::ReaperClass;

/// A point or a direction, in metres: right-handed, +Y up, as glTF and the
/// renderer have it.
pub type Point = [f32; 3];

fn sub(a: Point, b: Point) -> Point {
	[a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: Point, b: Point) -> Point {
	[
		a[1] * b[2] - a[2] * b[1],
		a[2] * b[0] - a[0] * b[2],
		a[0] * b[1] - a[1] * b[0],
	]
}

fn normalized(a: Point) -> Point {
	let length = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
	if length > 1e-9 {
		[a[0] / length, a[1] / length, a[2] / length]
	} else {
		[0.0, 0.0, 1.0]
	}
}

/// The rotation, as `[x, y, z, w]`, whose axes are the columns `x`, `y` and
/// `z` of an orthonormal basis.
pub fn quaternion_from_axes(x: Point, y: Point, z: Point) -> [f32; 4] {
	let trace = x[0] + y[1] + z[2];
	let [qx, qy, qz, qw] = if trace > 0.0 {
		let s = (trace + 1.0).sqrt() * 2.0;
		[
			(y[2] - z[1]) / s,
			(z[0] - x[2]) / s,
			(x[1] - y[0]) / s,
			0.25 * s,
		]
	} else if x[0] > y[1] && x[0] > z[2] {
		let s = (1.0 + x[0] - y[1] - z[2]).sqrt() * 2.0;
		[
			0.25 * s,
			(y[0] + x[1]) / s,
			(z[0] + x[2]) / s,
			(y[2] - z[1]) / s,
		]
	} else if y[1] > z[2] {
		let s = (1.0 + y[1] - x[0] - z[2]).sqrt() * 2.0;
		[
			(y[0] + x[1]) / s,
			0.25 * s,
			(z[1] + y[2]) / s,
			(z[0] - x[2]) / s,
		]
	} else {
		let s = (1.0 + z[2] - x[0] - y[1]).sqrt() * 2.0;
		[
			(z[0] + x[2]) / s,
			(z[1] + y[2]) / s,
			0.25 * s,
			(x[1] - y[0]) / s,
		]
	};
	let length = (qx * qx + qy * qy + qz * qz + qw * qw).sqrt();
	[qx / length, qy / length, qz / length, qw / length]
}

/// A camera at `eye` looking at `target`, with `up` as near its up as the
/// view allows: a viewer looks down its own −Z.
pub fn look_at(eye: Point, target: Point, up: Point) -> Transform {
	let back = normalized(sub(eye, target));
	let mut right = cross(up, back);
	if right.iter().all(|value| value.abs() < 1e-6) {
		right = [1.0, 0.0, 0.0];
	}
	let right = normalized(right);
	let true_up = cross(back, right);
	Transform::from_translation(eye[0], eye[1], eye[2])
		.with_rotation(quaternion_from_axes(right, true_up, back))
}

/// Where the camera looks on a reaper: about chest height.
pub const STAGE_TARGET: Point = [0.0, 1.42, 0.0];

/// How far back the camera stands for the whole figure, and for the
/// studio's animations, which need room for a lunge.
pub const STAGE_DISTANCE: f32 = 8.4;
pub const STUDIO_DISTANCE: f32 = 7.6;

/// The stage's lens: 30° tall.
pub const STAGE_FOV: f32 = 30.0 * std::f32::consts::PI / 180.0;

/// How high a reaper's face stands at rest, in metres, before stature. The
/// rig's joints are posed by its clips, so the face is where the idle puts
/// it.
pub const FACE_HEIGHT: f32 = 1.92;

/// The face's camera offset at `turn`: in front, a little below, and for the
/// Harvester well below, to clear its drooping hat brim. The face looks
/// along +Z.
pub fn face_orbit(turn: f32, calling: ReaperClass) -> Point {
	let under_hat = calling == ReaperClass::Harvester;
	let distance = if under_hat { 1.15 } else { 1.25 };
	[
		turn.sin() * distance,
		if under_hat { -0.6 } else { -0.04 },
		turn.cos() * distance,
	]
}

/// The stage's camera, turned `turn` radians round the reaper, at
/// `distance`, or close on the face of a reaper of `calling` standing
/// `stature` times its height.
pub fn stage_camera(turn: f32, distance: f32, face: Option<(ReaperClass, f32)>) -> Op {
	let (eye, target) = match face {
		Some((calling, stature)) => {
			let face = [0.0, FACE_HEIGHT * stature, 0.0];
			let offset = face_orbit(turn, calling);
			(
				[
					face[0] + offset[0],
					face[1] + offset[1],
					face[2] + offset[2],
				],
				face,
			)
		}
		None => {
			(
				[
					turn.sin() * distance,
					STAGE_TARGET[1] - 0.75,
					turn.cos() * distance,
				],
				STAGE_TARGET,
			)
		}
	};
	Op::SetCamera {
		transform: look_at(eye, target, [0.0, 1.0, 0.0]),
		fov_y: if face.is_some() {
			STAGE_FOV * 1.4
		} else {
			STAGE_FOV
		},
		near: 0.05,
		far: 40.0,
	}
}

/// Rotates `vector` by the unit quaternion `rotation`.
pub fn rotate(rotation: [f32; 4], vector: Point) -> Point {
	let axis = [rotation[0], rotation[1], rotation[2]];
	let scalar = rotation[3];
	let once = cross(axis, vector);
	let twice = cross(axis, once);
	[0, 1, 2].map(|i| vector[i] + 2.0 * (scalar * once[i] + twice[i]))
}

#[cfg(test)]
mod tests {
	use std::f32::consts::FRAC_PI_2;

	use super::*;

	fn close(a: Point, b: Point) {
		for axis in 0..3 {
			assert!((a[axis] - b[axis]).abs() < 1e-5, "{a:?} != {b:?}");
		}
	}

	#[test]
	fn a_camera_looks_down_its_own_minus_z_at_its_target() {
		let transform = look_at([3.0, 2.0, 4.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]);
		let forward = rotate(transform.rotation, [0.0, 0.0, -1.0]);
		close(forward, normalized([-3.0, -1.0, -4.0]));
		let up = rotate(transform.rotation, [0.0, 1.0, 0.0]);
		assert!(up[1] > 0.9);
		let right = rotate(transform.rotation, [1.0, 0.0, 0.0]);
		assert!(right[1].abs() < 1e-5, "the horizon stays level");
	}

	#[test]
	fn looking_straight_down_still_has_an_answer() {
		let transform = look_at([0.0, 5.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
		close(
			rotate(transform.rotation, [0.0, 0.0, -1.0]),
			[0.0, -1.0, 0.0],
		);
	}

	#[test]
	fn the_face_camera_looks_from_the_front_and_clears_the_harvester_s_brim() {
		close(face_orbit(0.0, ReaperClass::Harvester), [0.0, -0.6, 1.15]);
		close(
			face_orbit(FRAC_PI_2, ReaperClass::Ferryman),
			[1.25, -0.04, 0.0],
		);
	}

	#[test]
	fn the_stage_camera_stands_in_front_of_the_reaper_below_its_chest() {
		let Op::SetCamera { transform, .. } = stage_camera(0.0, STAGE_DISTANCE, None) else {
			unreachable!()
		};
		close(transform.translation, [0.0, 0.67, 8.4]);
		let forward = rotate(transform.rotation, [0.0, 0.0, -1.0]);
		assert!(forward[2] < -0.99 && forward[1] > 0.0);
	}
}
