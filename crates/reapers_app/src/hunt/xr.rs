//! The platform's view of the world, through `leptoskit_xr`: where the phone
//! points, the touches on its screen, the floor it stands over, and, while
//! a sigil is traced, the room's depth.
//!
//! - **Phones with AR** (`ARKit`, `ARCore`): `PhoneAr`, with planes and, where
//!   the phone measures it, depth. The camera's picture and the depth stay
//!   on the host; the game reads the room by casting rays against the depth
//!   ([`reapers_world::RayGrid`]) and takes the floor from the planes.
//! - **Phones without** (the Seeker has no `ARCore`) and the browser:
//!   `Orientation`, the rotation sensor, so the view turns as the phone
//!   does. The browser's `immersive-ar` would take the page away from the
//!   game's own canvas, so it is not asked for.
//! - **Anything else**: nothing, and [`super::pose::LivePose`] follows a
//!   finger. The hunt is told when there is no platform at all, or when a
//!   frame's pose cannot be trusted, so it never assumes a source it does
//!   not have.
//!
//! It all runs on one task per hunt, which stops when the hunt lets go.

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use reapers_world::RoomGridBuilder;
use reapers_world::RoomSurvey;

use super::pose::LivePose;

/// Whether a platform frame can safely supply a camera and room picture.
#[cfg(any(
	test,
	all(target_arch = "wasm32", any(feature = "web", feature = "native"))
))]
fn usable_frame(normal: bool, rotation: [f64; 4], position: [f64; 3], time: f64) -> bool {
	normal
		&& time.is_finite()
		&& position.iter().all(|value| value.is_finite())
		&& super::pose::pose_of(rotation).is_some()
}

/// One look at the room through the platform's depth: the grid's distances
/// along each ray, row by row from the top, and how long it took.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Look {
	pub columns: usize,
	pub rows: usize,
	pub distances: Vec<Option<f64>>,
	pub millis: i64,
}

/// What the hunt shares with the task that follows the platform.
#[derive(Clone, Default)]
pub struct Shared {
	pub environment: Rc<RefCell<super::environment::Environment>>,
	/// The last look at the room, for the depth bench.
	pub last_look: Rc<RefCell<Option<Look>>>,
	/// Whether the room is looked at whether or not a survey wants it: the
	/// depth bench.
	pub probing: Rc<Cell<bool>>,
	pub live: Rc<RefCell<LivePose>>,
	/// The room being read, while a sigil is traced.
	pub survey: Rc<RefCell<Option<RoomSurvey>>>,
	/// The same room read cell by cell, for a husk that walks it: every
	/// picture the survey takes goes into it too.
	pub grid: Rc<RefCell<Option<RoomGridBuilder>>>,
	/// Where the phone's eye was when the first picture the survey accepted
	/// was taken, in the platform's own space. Every later picture is
	/// charted relative to it, and the fight is drawn in the same frame.
	pub room_frame: Rc<Cell<Option<[f64; 3]>>>,
	/// Set when the hunt is over and the platform can be let go.
	pub done: Rc<Cell<bool>>,
	/// The scene a headset draws into its own frames: the hunt's `Scene3d`.
	pub scene: Option<u32>,
}

/// Starts following the platform for `shared`, from a press: browsers only
/// start motion and AR from a gesture.
pub fn follow(shared: &Shared) {
	#[cfg(all(target_arch = "wasm32", any(feature = "web", feature = "native")))]
	{
		shared.live.borrow_mut().expect_platform();
		crate::platform::spawn(platform::run(shared.clone()));
	}
	#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
	shared.live.borrow_mut().no_sensor();
}

#[cfg(all(target_arch = "wasm32", any(feature = "web", feature = "native")))]
mod platform {
	use leptoskit_xr::Availability;
	use leptoskit_xr::DepthSource;
	use leptoskit_xr::Features;
	use leptoskit_xr::Frame;
	use leptoskit_xr::HitKind;
	use leptoskit_xr::HitTargets;
	use leptoskit_xr::HitTestRequest;
	use leptoskit_xr::InputKind;
	use leptoskit_xr::Mode;
	use leptoskit_xr::PlaneLabel;
	use leptoskit_xr::PlaneOrientation;
	use leptoskit_xr::Projection;
	use leptoskit_xr::Ray;
	use leptoskit_xr::SessionConfig;
	use leptoskit_xr::TrackingState;
	use leptoskit_xr::Xr;
	use reapers_rules::Vec3;
	use reapers_world::DepthFrame;
	use reapers_world::RayGrid;
	use reapers_world::from_platform;
	use reapers_world::to_platform;

	use super::Shared;
	use crate::hunt::pose::Tracking;

	/// The modes asked for, best first. A Meta Quest, the only device here
	/// that offers it, draws the hunt into the headset over its passthrough.
	/// The browser's AR would cover the page with its own layer, so the
	/// browser only turns.
	#[cfg(feature = "native")]
	const MODES: [Mode; 3] = [Mode::ImmersiveAr, Mode::PhoneAr, Mode::Orientation];
	#[cfg(not(feature = "native"))]
	const MODES: [Mode; 1] = [Mode::Orientation];

	/// How long a finger rests on the screen before it is holding the art
	/// ready rather than striking, in seconds.
	const HOLD_SECONDS: f64 = 0.35;

	/// How far apart, in seconds, the room is looked at while it is read.
	const LOOK_EVERY: f64 = 0.8;

	/// The rays a look at the room casts: fewer than a picture's pixels,
	/// because each is a question the host answers. See
	/// [`RayGrid::COLUMNS`] for what this many can and cannot see.
	const LOOK_COLUMNS: usize = 12;
	const LOOK_ROWS: usize = 16;

	fn quaternion(frame: &Frame) -> [f64; 4] {
		let rotation = frame.viewer.orientation;
		[
			f64::from(rotation.x),
			f64::from(rotation.y),
			f64::from(rotation.z),
			f64::from(rotation.w),
		]
	}

	/// The vertical field of view of the frame's first view, in radians.
	fn fov(frame: &Frame) -> Option<f64> {
		let view = frame.views.first()?;
		match view.projection {
			Projection::Fov(fov) => Some(f64::from(fov.up - fov.down)),
			Projection::Matrix(matrix) => {
				let focal = f64::from(matrix[5]);
				(focal > 0.0).then(|| 2.0 * (1.0 / focal).atan())
			}
		}
	}

	/// How far above the floor the viewer is: the lowest plane facing up,
	/// or one the platform calls the floor.
	fn eye_height(frame: &Frame, planes: &[leptoskit_xr::Plane]) -> Option<f64> {
		let eye = f64::from(frame.viewer.position.y);
		let floor = planes
			.iter()
			.filter(|plane| plane.orientation == PlaneOrientation::HorizontalUp)
			.filter(|plane| plane.label == PlaneLabel::Floor || plane.label == PlaneLabel::Unknown)
			.map(|plane| f64::from(plane.pose.position.y))
			.filter(|floor| floor.is_finite())
			.filter(|floor| eye - floor > 0.3)
			.fold(None, |lowest: Option<f64>, floor| {
				Some(lowest.map_or(floor, |lowest| lowest.min(floor)))
			})?;
		Some(eye - floor)
	}

	async fn start(xr: Xr) -> Option<(leptoskit_xr::Session, Tracking)> {
		let capabilities = xr.capabilities().await.ok()?;
		for mode in MODES {
			if !capabilities.supports(mode) {
				continue;
			}
			#[cfg_attr(not(feature = "native"), allow(unused_mut))]
			let Ok(mut ready) = xr.prepare(mode, false).await else {
				continue;
			};
			// The rotation sensor answers ready without the camera, which a
			// phone without AR (the Seeker) still shows behind the hunt: ask
			// for it first there too.
			#[cfg(feature = "native")]
			if (ready == Availability::NeedsPermission || mode == Mode::Orientation)
				&& let Ok(permissions) = leptoskit_plugin_permissions::guest::Permissions::new()
			{
				let _ = permissions
					.request(leptoskit_plugin_permissions::Permission::Camera)
					.await;
				let Ok(prepared) = xr.prepare(mode, false).await else {
					continue;
				};
				ready = prepared;
			}
			if ready != Availability::Ready {
				continue;
			}
			let config = SessionConfig::new(mode)
				.prefer(
					Features::PLANES
						| Features::DEPTH
						| Features::HIT_TEST
						| Features::CAMERA_IMAGE,
				)
				.frame_rate(30);
			if let Ok(session) = xr.start(config).await {
				let tracking = if mode == Mode::Orientation {
					Tracking::Turning
				} else {
					Tracking::World
				};
				return Some((session, tracking));
			}
		}
		None
	}

	pub async fn run(shared: Shared) {
		let Ok(xr) = Xr::new() else {
			shared.live.borrow_mut().no_sensor();
			return;
		};
		let Some((mut session, tracking)) = start(xr).await else {
			shared.live.borrow_mut().no_sensor();
			return;
		};
		if tracking == Tracking::World {
			shared.live.borrow_mut().world_source();
		}
		// A headset draws the hunt's scene itself; a phone's session shows
		// it in the hunt's own view and ignores this.
		let _ = session.present(shared.scene).await;
		shared.live.borrow_mut().camera = session.info().features.contains(Features::CAMERA_IMAGE);
		let mut planes = Vec::new();
		let mut planes_revision = u64::MAX;
		let mut touches: Vec<(u32, f64, bool)> = Vec::new();
		let mut last_look = f64::NEG_INFINITY;
		while !shared.done.get() {
			let Ok(frame) = session.next_frame().await else {
				break;
			};
			let at = frame.viewer.position;
			let position = [f64::from(at.x), f64::from(at.y), f64::from(at.z)];
			let normal = super::usable_frame(
				frame.tracking == TrackingState::Normal,
				quaternion(&frame),
				position,
				frame.time,
			);
			let reliable = {
				let mut live = shared.live.borrow_mut();
				let reliable = if tracking == Tracking::World {
					live.world_frame(quaternion(&frame), position, frame.time, normal)
				} else if !normal {
					// The platform says this pose is stale or absent: hold the
					// view where it was rather than follow a pose it cannot
					// vouch for.
					live.untracked();
					false
				} else {
					live.track(quaternion(&frame), frame.time, tracking);
					true
				};
				if reliable {
					if frame.planes_revision != planes_revision {
						planes_revision = frame.planes_revision;
						planes.clone_from(&frame.planes);
					}
					live.fov = fov(&frame);
				}
				live.depth = frame.depth.is_some();
				if reliable && let Some(height) = eye_height(&frame, &planes) {
					live.eye_height = Some(height);
				}
				// A touch that lifts before it is held is a strike along its
				// ray; one that stays is the art held ready.
				let present: Vec<_> = frame
					.inputs
					.iter()
					.filter(|input| input.kind == InputKind::Screen)
					.collect();
				for input in &present {
					if !touches.iter().any(|(id, ..)| *id == input.id) {
						touches.push((input.id, frame.time, false));
					}
				}
				touches.retain(|(id, ..)| present.iter().any(|input| input.id == *id));
				live.holding = touches
					.iter()
					.any(|(_, since, _)| frame.time - since > HOLD_SECONDS);
				reliable
			};
			if reliable && frame.depth.is_some() && frame.time - last_look > LOOK_EVERY {
				impacts(&shared, &session).await;
				last_look = frame.time;
				look(&shared, &session, &frame).await;
			} else if reliable {
				impacts(&shared, &session).await;
			}
		}
		shared.live.borrow_mut().lose_device();
		let _ = session.stop().await;
	}

	async fn impacts(shared: &Shared, session: &leptoskit_xr::Session) {
		let requests = shared.environment.borrow_mut().take_requests();
		for request in requests {
			let point = |v: reapers_rules::Vec3| {
				let [x, y, z] = reapers_world::to_platform(v).map(|v| v as f32);
				leptoskit_xr::Vec3::new(x, y, z)
			};
			let ray = Ray {
				origin: point(request.origin),
				direction: point(request.direction),
			};
			let Ok(hits) = session.hit_test(HitTestRequest::ray(ray)).await else {
				continue;
			};
			if let Some(hit) = hits.first() {
				let at = hit.pose.position;
				let normal = hit.pose.orientation.rotate(leptoskit_xr::Vec3::Y);
				shared.environment.borrow_mut().land(
					request,
					super::super::environment::Surface {
						at: reapers_world::from_platform([at.x, at.y, at.z].map(f64::from)),
						normal: reapers_world::from_platform(
							[normal.x, normal.y, normal.z].map(f64::from),
						),
						distance: f64::from(hit.distance),
					},
				);
			}
		}
	}

	/// Reads the room once, where it is wanted: a grid of rays cast against
	/// the platform's depth, taken as a depth picture.
	async fn look(shared: &Shared, session: &leptoskit_xr::Session, frame: &Frame) {
		let Some(source) = frame.depth else {
			return;
		};
		let (pose, fov, eye_height) = {
			let live = shared.live.borrow();
			(live.pose(), live.fov, live.eye_height)
		};
		let wanted = shared.probing.get()
			|| shared
				.survey
				.borrow()
				.as_ref()
				.is_some_and(|survey| survey.wants(&pose));
		let Some(fov) = fov.filter(|_| wanted) else {
			return;
		};
		let (width, height) = crate::platform::viewport();
		let tan_half_height = (fov / 2.0).tan();
		let grid = RayGrid {
			pose,
			tan_half_width: tan_half_height * width / height,
			tan_half_height,
			columns: LOOK_COLUMNS,
			rows: LOOK_ROWS,
		};
		let eye = frame.viewer.position;
		let anchor = shared.room_frame.get();
		// Where this picture's eye is in the frame the hunt is charted and
		// drawn in: the first picture's eye, or this one when it is first.
		let origin = anchor.map_or(Vec3::ZERO, |anchor| {
			from_platform([
				f64::from(eye.x) - anchor[0],
				f64::from(eye.y) - anchor[1],
				f64::from(eye.z) - anchor[2],
			])
		});
		let started = crate::platform::now();
		let mut distances = Vec::with_capacity(LOOK_COLUMNS * LOOK_ROWS);
		for direction in grid.rays() {
			let [x, y, z] = to_platform(direction);
			let ray = Ray::new(eye, leptoskit_xr::Vec3::new(x as f32, y as f32, z as f32));
			let hit = match ray {
				Some(ray) => {
					session
						.hit_test(
							HitTestRequest::ray(ray)
								.targets(HitTargets::DEPTH)
								.max_results(1),
						)
						.await
						.ok()
						.and_then(|hits| hits.first().copied())
				}
				None => None,
			};
			if let Some(hit) = hit {
				let expected = match source {
					DepthSource::Measured => HitKind::Depth,
					DepthSource::Estimated => HitKind::EstimatedDepth,
				};
				// Each request reads the latest host frame. If the source
				// changes during the grid, try again on the next look.
				if hit.kind != expected {
					return;
				}
			}
			distances.push(hit.map(|hit| f64::from(hit.distance)));
		}
		// Ray casts can outlive the frame they came from. Never chart one
		// after that source has gone silent.
		if !shared.live.borrow().follows_device() {
			return;
		}
		*shared.last_look.borrow_mut() = Some(super::Look {
			columns: LOOK_COLUMNS,
			rows: LOOK_ROWS,
			distances: distances.clone(),
			millis: crate::platform::now().millis() - started.millis(),
		});
		let picture = DepthFrame::from_rays_at(&grid, &distances, origin);
		chart(shared, eye, source, eye_height, picture).await;
	}

	/// Calibrates a picture into the survey and the walking grid together.
	async fn chart(
		shared: &Shared,
		eye: leptoskit_xr::Vec3,
		source: DepthSource,
		eye_height: Option<f64>,
		picture: DepthFrame,
	) {
		let origin = picture.origin;
		let surveyed = {
			let mut survey = shared.survey.borrow_mut();
			let Some(survey) = survey.as_mut() else {
				return;
			};
			if let Some(height) = eye_height {
				// The platform measures the floor below the eye that is
				// looking, which is not where the hunt's frame begins when
				// the player has moved.
				survey.measure_floor(height - origin.y);
			}
			match source {
				DepthSource::Measured => survey.add(&picture),
				DepthSource::Estimated => survey.add_estimated(&picture),
			}
		};
		if surveyed && shared.room_frame.get().is_none() {
			// The first picture that teaches anything fixes the frame. What
			// the survey and the grid learn from it, and from every picture
			// after, is charted relative to it.
			shared
				.room_frame
				.set(Some([f64::from(eye.x), f64::from(eye.y), f64::from(eye.z)]));
		}
		if shared.grid.borrow().is_some() {
			if let Some(height) = eye_height
				&& let Some(cells) = shared.grid.borrow_mut().as_mut()
			{
				cells.measure_floor(height - origin.y);
			}
			if surveyed {
				// Charting the picture into the grid costs about as much
				// again as the survey did, so it waits for the next turn of
				// the event loop, where a frame can be drawn between the two.
				crate::platform::sleep(std::time::Duration::ZERO).await;
				if let Some(cells) = shared.grid.borrow_mut().as_mut() {
					match source {
						DepthSource::Measured => cells.add_frame(&picture),
						DepthSource::Estimated => cells.add_estimated(&picture),
					};
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::usable_frame;

	#[test]
	fn room_capture_requires_a_normal_finite_frame() {
		let rotation = [0.0, 0.0, 0.0, 1.0];
		let position = [1.0, 1.5, 2.0];
		assert!(usable_frame(true, rotation, position, 1.0));
		assert!(!usable_frame(false, rotation, position, 1.0));
		assert!(!usable_frame(true, [0.0; 4], position, 1.0));
		assert!(!usable_frame(true, rotation, [f64::NAN, 0.0, 0.0], 1.0));
		assert!(!usable_frame(true, rotation, position, f64::NAN));
	}
}
