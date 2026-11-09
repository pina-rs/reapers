//! A reading of the room, made while the player traces the husk's sigil.

use reapers_rules::Gait;
use reapers_rules::HuntPlan;
use reapers_rules::SigilTrace;
use reapers_rules::Vessel;
use reapers_rules::radians;

use crate::depth_frame::DepthFrame;
use crate::occlusion_map::OcclusionMap;
use crate::pose::ArPose;
use crate::room_map::RoomMap;

/// A reading of the room, made while the player traces the husk's sigil,
/// that ends in a plan of where the husk may go.
///
/// The circle around the player is divided into the slices of a
/// [`HuntPlan`], and a survey fills them in as the camera looks into them.
/// The sigil decides when the reading is over and which arc the husk keeps
/// to. The survey says what it learnt of that arc.
///
/// It holds the map of the floor and the map of what hides a husk, and
/// takes each picture into both, scaled to the same floor. Where the
/// pictures come from is up to whoever feeds it.
#[derive(Clone, Debug)]
pub struct RoomSurvey {
	map: RoomMap,
	occlusion: OcclusionMap,
}

impl RoomSurvey {
	/// Turning faster than this, in radians a second, no picture is taken. A
	/// picture is pinned to where the phone pointed when it arrived, and the
	/// faster the phone turns the further out that is.
	pub const STEADY_TURN_SPEED: f64 = 0.7;

	/// A survey that has read nothing yet, for a husk shaped as `vessel` is.
	pub fn new(vessel: Vessel) -> Self {
		Self {
			map: RoomMap::new(vessel),
			occlusion: OcclusionMap::new(),
		}
	}

	/// The map of the floor and what stands on it.
	pub const fn map(&self) -> &RoomMap {
		&self.map
	}

	/// The room as it looks from where the player stands, for hiding the
	/// husk behind what stands in it.
	pub const fn occlusion(&self) -> &OcclusionMap {
		&self.occlusion
	}

	/// Pictures read so far.
	pub const fn pictures(&self) -> usize {
		self.map.frames()
	}

	/// Whether a picture taken with the phone pointing as `pose` does is
	/// worth reading: see [`can_read`].
	pub fn wants(&self, pose: &ArPose) -> bool {
		can_read(pose, self.map.vessel().gait(), Self::STEADY_TURN_SPEED)
	}

	/// Takes the floor as measured by the platform: see
	/// [`RoomMap::measure_floor`]. It is the floor below the hunt's origin;
	/// a picture taken from above or below it has the origin's height taken
	/// into account.
	pub fn measure_floor(&mut self, eye_height: f64) -> bool {
		self.map.measure_floor(eye_height)
	}

	/// Takes in one picture. Answers whether anything was learnt from it.
	pub fn add(&mut self, frame: &DepthFrame) -> bool {
		let learnt = self.map.add(frame);

		// Each picture is scaled to the floor the hunt will stand on, as the
		// map of the room is. A picture taken from above or below the hunt's
		// origin saw the floor that much farther away, so its own expected
		// floor is the hunt's plus its height.
		if learnt
			&& let (Some(picture_floor), Some(floor)) =
				(self.map.last_picture_floor(), self.map.floor_depth())
		{
			let scale = (floor + frame.origin.y) / picture_floor;

			self.occlusion.add(frame, picture_floor, scale);
		}

		learnt
	}

	/// How well slice `index` has been read, from 0 to 1.
	pub fn read(&self, index: usize) -> f64 {
		self.map.read(index)
	}

	/// How far the way is clear along slice `index`, in metres, or `None`
	/// until it has been read.
	pub fn reach(&self, index: usize) -> Option<f64> {
		self.map.reach(index)
	}

	/// Whether slice `index` has been read and found tight: so little clear
	/// floor that the husk can only stand there as close as it ever comes.
	pub fn tight(&self, index: usize) -> bool {
		self.map.tight(index)
	}

	/// The plan for a hunt kept to `half_width` radians either side of the
	/// bearing `centre`, with whatever has been learnt of the room so far.
	pub fn plan_within(&self, centre: f64, half_width: f64) -> HuntPlan {
		self.map.plan_within(centre, half_width)
	}
}

/// The plan for a hunt inside the arc tracing `trace` read, with what
/// `survey` learnt of the room, or nothing of it when there is no survey.
pub fn sigil_plan(trace: &SigilTrace, survey: Option<&RoomSurvey>) -> HuntPlan {
	let half_arc = trace.sigil.half_arc();

	match survey {
		Some(survey) => survey.plan_within(trace.centre, half_arc),
		None => HuntPlan::between(trace.centre - half_arc, trace.centre + half_arc),
	}
}

/// Whether the camera, pointing as `pose` does, can read the room for a
/// husk with `gait`: whether the phone is turning slower than
/// `steady_turn_speed` radians a second and pointing at the height that
/// needs reading.
///
/// The floor a walker will stand on lies two to five metres out, which is
/// well below the horizon from standing height. A flier keeps to about head
/// height. A sigil's lights hang inside these bands.
pub fn can_read(pose: &ArPose, gait: Gait, steady_turn_speed: f64) -> bool {
	let (low, high) = match gait {
		Gait::Ground => (radians(-52.0), radians(-7.0)),
		Gait::Air | Gait::Still => (radians(-18.0), radians(38.0)),
	};
	let pitch = pose.forward.pitch();

	pose.turn_speed <= steady_turn_speed && pitch >= low && pitch <= high
}
