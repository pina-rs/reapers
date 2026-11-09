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
//!   finger.
//!
//! It all runs on one task per hunt, which stops when the hunt lets go.

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use reapers_world::RoomGridBuilder;
use reapers_world::RoomSurvey;

use super::pose::LivePose;

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
	crate::platform::spawn(platform::run(shared.clone()));
	#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
	let _ = shared;
}

#[cfg(all(target_arch = "wasm32", any(feature = "web", feature = "native")))]
mod platform {
	use leptoskit_xr::Availability;
	use leptoskit_xr::Features;
	use leptoskit_xr::Frame;
	use leptoskit_xr::HitTargets;
	use leptoskit_xr::HitTestRequest;
	use leptoskit_xr::InputKind;
	use leptoskit_xr::Mode;
	use leptoskit_xr::PlaneLabel;
	use leptoskit_xr::PlaneOrientation;
	use leptoskit_xr::Projection;
	use leptoskit_xr::Ray;
	use leptoskit_xr::SessionConfig;
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
			let mut ready = xr.prepare(mode, false).await.ok()?;
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
				ready = xr.prepare(mode, false).await.ok()?;
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
			return;
		};
		let Some((mut session, tracking)) = start(xr).await else {
			return;
		};
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
			if frame.planes_revision != planes_revision {
				planes_revision = frame.planes_revision;
				planes.clone_from(&frame.planes);
			}
			{
				let mut live = shared.live.borrow_mut();
				live.track(quaternion(&frame), frame.time, tracking);
				live.fov = fov(&frame);
				live.depth = frame.depth;
				let at = frame.viewer.position;
				live.position = Some([f64::from(at.x), f64::from(at.y), f64::from(at.z)]);
				if let Some(height) = eye_height(&frame, &planes) {
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
			}
			if frame.depth && frame.time - last_look > LOOK_EVERY {
				last_look = frame.time;
				look(&shared, &session, &frame).await;
			}
		}
		shared.live.borrow_mut().lose_device();
		let _ = session.stop().await;
	}

	/// Reads the room once, where it is wanted: a grid of rays cast against
	/// the platform's depth, taken as a depth picture.
	async fn look(shared: &Shared, session: &leptoskit_xr::Session, frame: &Frame) {
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
						.and_then(|hits| hits.first().map(|hit| f64::from(hit.distance)))
				}
				None => None,
			};
			distances.push(hit);
		}
		*shared.last_look.borrow_mut() = Some(super::Look {
			columns: LOOK_COLUMNS,
			rows: LOOK_ROWS,
			distances: distances.clone(),
			millis: crate::platform::now().millis() - started.millis(),
		});
		let picture = DepthFrame::from_rays_at(&grid, &distances, origin);
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
			survey.add(&picture)
		};
		if surveyed && shared.room_frame.get().is_none() {
			// The first picture that teaches anything fixes the frame. What
			// the survey and the grid learn from it, and from every picture
			// after, is charted relative to it.
			shared
				.frame
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
					cells.add_frame(&picture);
				}
			}
		}
	}
}
