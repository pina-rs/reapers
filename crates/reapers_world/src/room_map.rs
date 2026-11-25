//! The room as seen from above, put together from depth pictures.

use reapers_rules::Gait;
use reapers_rules::HuntPlan;
use reapers_rules::HuntRules;
use reapers_rules::Vec3;
use reapers_rules::Vessel;

use crate::depth_frame::DepthFrame;
use crate::math;
use crate::math::PI;

/// Rings of the map, counted out from the player. For a walker each is
/// [`RING_SIZE`] of the eye's height above the floor wide; for a flier,
/// that many metres.
const RINGS: usize = 90;
const RING_SIZE: f64 = 0.07;

/// Every other pixel each way is enough to map a room with.
pub(crate) const STRIDE: usize = 2;

/// Points a slice needs before it counts as read.
const POINTS_TO_READ: f64 = 60.0;

/// Points in one ring of one slice that make it floor, or make it taken.
const POINTS_OF_FLOOR: u32 = 3;
const POINTS_IN_THE_WAY: u32 = 5;

/// Nearer than this, in metres, is the player's own feet and hands.
pub(crate) const SELF_METRES: f64 = 0.6;

/// How far off the floor a point may be and still be floor: a little, and a
/// little more for every metre away, where depth is less sure.
const FLOOR_SLACK: f64 = 0.07;
const FLOOR_SLACK_PER_METRE: f64 = 0.045;

/// The floor is looked for in 2 cm steps between the lowest and highest a
/// phone can be held above one.
const FLOOR_STEP: f64 = 0.02;
const FLOOR_NEAREST: f64 = HuntPlan::MIN_EYE_CENTIMETRES as f64 / 100.0;
const FLOOR_FARTHEST: f64 = HuntPlan::MAX_EYE_CENTIMETRES as f64 / 100.0;

/// How high points are charted, in bins of [`HEIGHT_BIN`] of the floor's
/// depth.
const HEIGHT_BINS: usize = 40;
const HEIGHT_BIN: f64 = 0.04;

/// Points in the highest bin that make it the top of something.
const POINTS_OF_TOP: u32 = 3;

/// Level points around the top that make it a ledge, and the share of the
/// points there they must be.
const POINTS_OF_LEDGE: u32 = 12;
const LEDGE_SHARE: f64 = 0.35;

/// How far, in metres, a hiding husk keeps from the back of its cover and
/// from what stands behind it.
const HIDE_GAP: f64 = 0.1;

/// How nearly straight up a surface has to face to be level.
const LEVEL_SHARE: f64 = 0.9;

/// Steps either side that are counted together with each step.
const FLOOR_BLUR: usize = 2;

/// Pictures that have to have found a floor before one that disagrees with
/// them is set aside, and how far, as a share, it may disagree.
const PICTURES_TO_AGREE: usize = 3;
const FLOOR_LEEWAY: f64 = 0.25;

/// The fewest level points that make a floor, and the share of the biggest
/// level surface a lower one must have to be the floor instead.
const FLOOR_POINTS: f64 = 120.0;
const FLOOR_SHARE: f64 = 0.15;

/// The room as seen from above, put together from depth pictures: where the
/// floor is, how far the way is clear in each direction, and what stands at
/// the end of each way: how tall it is, whether its top is level enough to
/// stand on, and whether there is room behind it to hide.
///
/// Each picture becomes a cloud of points around the player. For a husk
/// that walks, the floor is found in every picture on its own: the lowest
/// level surface with any size to it. Everything in the picture is then
/// measured in multiples of how far that floor is below the eye, and only
/// the floor's depth, taken across all the pictures, is trusted in metres.
/// A depth model reads a whole picture long or short together, so this
/// keeps one picture's floor and furniture in step with the next one's, and
/// it puts a husk's feet on the floor the camera shows even when the
/// model's metres are off.
///
/// A picture carries where its eye was when it was taken
/// ([`DepthFrame::origin`]), so a player who moves while the room is read
/// charts one room rather than one room per place they stood.
///
/// Where the platform has measured the floor itself, in real metres, that
/// is trusted instead: see [`measure_floor`](Self::measure_floor).
///
/// This holds no camera and no model, only arithmetic, so that it can be
/// tested against rooms drawn by hand.
/// Tracks the room's floor height and per-picture floor scale without
/// allocating the 2 MiB slice/furniture tables of a full [`RoomMap`].
/// Shared between [`RoomMap`] and [`RoomGridBuilder`](crate::RoomGridBuilder)
/// so the plan and the 10 cm grid always agree on the floor to the bit.
#[derive(Clone, Debug, Default)]
pub(crate) struct FloorTracker {
	floors: Vec<f64>,
	measured_floor: Option<f64>,
	frames: usize,
	last_picture_floor: Option<f64>,
}

impl FloorTracker {
	pub(crate) const fn new() -> Self {
		Self {
			floors: Vec::new(),
			measured_floor: None,
			frames: 0,
			last_picture_floor: None,
		}
	}

	pub(crate) const fn frames(&self) -> usize {
		self.frames
	}

	pub(crate) const fn last_picture_floor(&self) -> Option<f64> {
		self.last_picture_floor
	}

	pub(crate) const fn floor_is_measured(&self) -> bool {
		self.measured_floor.is_some()
	}

	pub(crate) fn floor_depth(&self) -> Option<f64> {
		if self.measured_floor.is_some() {
			return self.measured_floor;
		}

		self.picture_floor_depth()
	}

	fn picture_floor_depth(&self) -> Option<f64> {
		if self.floors.is_empty() {
			return None;
		}

		Some(self.floors.iter().sum::<f64>() / self.floors.len() as f64)
	}

	pub(crate) fn measure_floor(&mut self, eye_height: f64) -> bool {
		if !(FLOOR_NEAREST..=FLOOR_FARTHEST).contains(&eye_height) {
			return false;
		}

		self.measured_floor = Some(eye_height);
		true
	}

	/// Resolves `(picture_floor, scale, unit)` for `frame`, updating floor
	/// tracking state if accepted. Returns `None` if the picture is rejected.
	pub(crate) fn evaluate(
		&mut self,
		cloud: &Cloud,
		level: &[bool],
		origin: Vec3,
		gait: Gait,
		estimated: bool,
	) -> Option<(Option<f64>, Option<f64>, Option<f64>)> {
		let measured = self.measured_floor.filter(|_| !estimated);
		let own = if measured.is_some() {
			None
		} else {
			floor_of(&cloud.ys, level)
		};
		let agreed = if self.floors.len() < PICTURES_TO_AGREE {
			None
		} else {
			self.picture_floor_depth()
		};

		// The floor below this picture's own eye, in its own metres.
		let floor = measured
			.map(|floor| floor + origin.y)
			.or(own)
			.filter(|floor| floor.is_finite() && *floor > 0.0);
		let floor_at_origin = floor
			.map(|floor| floor - origin.y)
			.filter(|at_origin| (FLOOR_NEAREST..=FLOOR_FARTHEST).contains(at_origin));

		let agrees = floor_at_origin.is_some_and(|floor| {
			measured.is_some()
				|| agreed.is_none_or(|agreed| (floor / agreed - 1.0).abs() <= FLOOR_LEEWAY)
		});

		if gait == Gait::Ground && !agrees {
			return None;
		}
		if estimated && self.measured_floor.is_some() && !agrees {
			return None;
		}
		if agrees
			&& let Some(at_origin) = floor_at_origin
			&& own.is_some()
		{
			self.floors.push(at_origin);
		}
		let chart = floor.filter(|_| agrees);
		let unit = self.floor_depth().filter(|u| u.is_finite() && *u > 0.0);
		let scale = chart.and_then(|floor| {
			let numerator = unit.unwrap_or(floor - origin.y) + origin.y;
			let scale = numerator / floor;
			(scale.is_finite() && scale > 0.0).then_some(scale)
		});

		if gait != Gait::Ground && estimated && self.measured_floor.is_some() && scale.is_none() {
			return None;
		}

		self.last_picture_floor = chart;
		self.frames += 1;

		Some((chart, scale, unit))
	}
}

#[derive(Clone, Debug)]
pub struct RoomMap {
	/// The husk the room is being read for: how it gets about decides what
	/// counts as in its way, and its size decides where it fits.
	vessel: Vessel,

	clear: Vec<u32>,
	taken: Vec<u32>,
	seen: Vec<u32>,
	floor: FloorTracker,

	// What stands on the floor, whatever the husk. Every length is in
	// multiples of the floor's depth in its own picture, like the rings.
	/// Points of floor in each cell.
	floor_at: Vec<u32>,

	/// Points of anything standing on the floor in each cell.
	solid: Vec<u32>,

	/// How high those points are, cell by cell, in bins of [`HEIGHT_BIN`],
	/// and the same for the points among them that lie on a level surface.
	heights: Vec<u32>,
	level_heights: Vec<u32>,

	// What the pictures so far add up to. Worked out once for each picture
	// taken in, not each time it is asked for.
	/// How well each slice has been read, from 0 to 1.
	read: Vec<f64>,

	/// Each slice's reach in metres, or `None` where it has not been read.
	reach: Vec<Option<f64>>,

	/// What stands at the end of each slice's reach.
	furniture: Vec<Option<Furniture>>,
}

/// Something standing in the room, at the end of one slice's way.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Furniture {
	/// How far its front is from the player, in metres.
	pub front: f64,

	/// How high its top is above the floor, in metres.
	pub cover: f64,

	/// Whether its top is level enough to stand on.
	pub ledge: bool,

	/// How far out a husk could hide behind it, in metres, or `None` where
	/// there is no room behind it, or nothing behind it was seen.
	pub hide: Option<f64>,
}

/// How far a slice is clear, in rings, and whether something stands in the
/// next one.
#[derive(Clone, Copy)]
struct Clear {
	rings: usize,
	stopped: bool,
}

/// One picture as a cloud of points around the player's eye. Points that
/// were not seen are NaN.
pub(crate) struct Cloud {
	pub(crate) xs: Vec<f32>,
	pub(crate) ys: Vec<f32>,
	pub(crate) zs: Vec<f32>,
}

impl RoomMap {
	/// An empty map, for a husk shaped as `vessel` is.
	pub fn new(vessel: Vessel) -> Self {
		let cells = HuntPlan::SLICES * RINGS;

		Self {
			vessel,
			clear: vec![0; cells],
			taken: vec![0; cells],
			seen: vec![0; HuntPlan::SLICES],
			floor: FloorTracker::new(),
			floor_at: vec![0; cells],
			solid: vec![0; cells],
			heights: vec![0; cells * HEIGHT_BINS],
			level_heights: vec![0; cells * HEIGHT_BINS],
			read: vec![0.0; HuntPlan::SLICES],
			reach: vec![None; HuntPlan::SLICES],
			furniture: vec![None; HuntPlan::SLICES],
		}
	}

	/// The husk the room is being read for.
	pub const fn vessel(&self) -> Vessel {
		self.vessel
	}

	/// Pictures taken in so far.
	pub const fn frames(&self) -> usize {
		self.floor.frames()
	}

	/// How far below the eye of the last picture the floor was in that
	/// picture's own metres. Each picture is read against its own floor, and
	/// this is how far to scale it to agree with
	/// [`floor_depth`](Self::floor_depth).
	pub const fn last_picture_floor(&self) -> Option<f64> {
		self.floor.last_picture_floor()
	}

	/// How far the floor is below the hunt's origin, in metres: as the
	/// platform measured it, or else the mean of what every accepted picture
	/// made of it, brought back to the hunt's origin.
	///
	/// The mean is the one estimate the plan and the room's grid both use, so
	/// the grid a husk walks is always one the plan agrees with.
	pub fn floor_depth(&self) -> Option<f64> {
		self.floor.floor_depth()
	}

	/// Whether the floor's depth was measured by the platform rather than
	/// found in the pictures.
	pub const fn floor_is_measured(&self) -> bool {
		self.floor.floor_is_measured()
	}

	/// Takes the floor as `eye_height` metres below the hunt's origin, as the
	/// platform measured it. Answers whether it was taken: a height no
	/// phone could be held at is not.
	///
	/// A platform that tracks the floor (a plane it labels as floor, below
	/// the camera) knows exactly how far below the eye it is: the camera's
	/// height less the plane's. Its depth is in real metres too, unlike a
	/// depth model's. So once the floor is measured:
	///
	/// - [`floor_depth`](Self::floor_depth) is the measured height, and the
	///   plan's eye height is exactly it;
	/// - every measured-depth picture is charted against it, with no floor of its own
	///   looked for and none set aside. A picture with no floor in view,
	///   such as one of a table top, now charts the table: its height is
	///   still known;
	/// - what was charted before is scaled to it. The map keeps every length
	///   in multiples of the floor's depth, so a picture read against its own
	///   floor and one read against the measured floor land in the same
	///   rings, and only the metres they come to change.
	///
	/// A picture taken from `origin` elsewhere has the measured floor below
	/// its own eye, so [`add`](Self::add) takes the origin's height into
	/// account before charting it.
	///
	/// It may be measured again, as the platform refines the plane; the last
	/// measurement stands. Only depth in real metres should be read against
	/// a measured floor: use [`add_estimated`](Self::add_estimated) for a
	/// depth model's pictures, which must first find their own floors.
	pub fn measure_floor(&mut self, eye_height: f64) -> bool {
		if !self.floor.measure_floor(eye_height) {
			return false;
		}

		self.reckon(u128::MAX);

		true
	}

	/// Takes in one picture. Answers whether anything was learnt from it: a
	/// picture with no floor in it teaches nothing about where a walker can
	/// stand, unless the floor has been measured.
	pub fn add(&mut self, frame: &DepthFrame) -> bool {
		self.add_frame(frame, false)
	}

	/// Takes a picture inferred by a depth model. Even with a measured AR
	/// floor, its own floor must be found: the model can read everything
	/// long or short together. Lengths are charted in that picture's floor
	/// units, then converted to the measured floor's metres.
	pub fn add_estimated(&mut self, frame: &DepthFrame) -> bool {
		self.add_frame(frame, true)
	}

	fn add_frame(&mut self, frame: &DepthFrame, estimated: bool) -> bool {
		let columns = frame.width / STRIDE;
		let rows = frame.height / STRIDE;
		let cloud = Cloud::of(frame);
		let level = level_points(&cloud, columns, rows);
		self.add_prepared(&cloud, &level, frame.origin, estimated)
	}

	pub(crate) fn add_prepared(
		&mut self,
		cloud: &Cloud,
		level: &[bool],
		origin: Vec3,
		estimated: bool,
	) -> bool {
		let prev_floor = self.floor.floor_depth();
		let Some((chart, scale, unit)) =
			self.floor
				.evaluate(cloud, level, origin, self.vessel.gait(), estimated)
		else {
			return false;
		};

		let mut dirty_slices = 0_u128;

		if let (Some(floor), Some(scale), Some(unit)) = (chart, scale, unit) {
			if self.vessel.gait() == Gait::Ground {
				dirty_slices |=
					self.chart_ground_and_furniture(cloud, level, floor, scale, origin, unit);
			} else {
				dirty_slices |= self.chart_furniture(cloud, level, floor, scale, origin, unit);
			}
		}
		if self.vessel.gait() != Gait::Ground {
			let air_scale = if estimated && self.floor.floor_is_measured() {
				let Some(scale) = scale else {
					return false;
				};
				scale
			} else {
				1.0
			};
			dirty_slices |= self.chart_air(cloud, level, origin, air_scale);
		}

		let floor_changed = self.floor.floor_depth() != prev_floor;
		self.reckon(if floor_changed {
			u128::MAX
		} else {
			dirty_slices
		});

		true
	}

	/// What stands at the end of the way along slice `index`: how tall it
	/// is, where a husk could hide behind it, and whether its top is a
	/// ledge. `None` where the way ends for want of floor, or the slice has
	/// not been read.
	pub fn furniture(&self, index: usize) -> Option<Furniture> {
		self.furniture[index]
	}

	/// How well slice `index` has been seen, from 0 to 1.
	pub fn read(&self, index: usize) -> f64 {
		self.read[index]
	}

	/// How far the way is clear along slice `index`, in metres: to the first
	/// thing standing in it, or else to the end of the floor that was seen.
	/// `None` until the slice has been read.
	pub fn reach(&self, index: usize) -> Option<f64> {
		self.reach[index]
	}

	/// Whether slice `index` has been read and found tight: so little clear
	/// floor that the husk can only stand there as close as it ever comes.
	pub fn tight(&self, index: usize) -> bool {
		self.reach[index]
			.is_some_and(|metres| metres - self.vessel.half_width() < HuntRules::CLOSEST_METRES)
	}

	/// How much of the circle has been read at all, in radians.
	pub fn coverage(&self) -> f64 {
		self.read.iter().filter(|read| **read >= 1.0).count() as f64 * HuntPlan::SLICE_WIDTH
	}

	/// The plan for a hunt kept to `half_width` radians either side of the
	/// bearing `centre`: the floor where it was found, and how far the way
	/// is clear along every slice that has been read.
	///
	/// A room is never too small to hunt in. Where there is little clear
	/// floor the husk stands closer, and the plan says how much there is.
	/// Nothing is known to be in the way along a slice that has not been
	/// read, so it is given the most a plan can say. A plan made before any
	/// slice was read says nothing about the room but the floor.
	pub fn plan_within(&self, centre: f64, half_width: f64) -> HuntPlan {
		let mut plan = HuntPlan::between(centre - half_width, centre + half_width);

		plan.eye_centimetres =
			math::round(self.floor_depth().unwrap_or(HuntRules::EYE_HEIGHT) * 100.0) as i32;

		if self.reach.iter().any(Option::is_some) {
			plan.reach_decimetres = self
				.reach
				.iter()
				.map(|reach| {
					match reach {
						None => HuntPlan::MAX_REACH_DECIMETRES,
						Some(metres) => {
							math::floor(metres * 10.0)
								.clamp(0, i64::from(HuntPlan::MAX_REACH_DECIMETRES))
								as i32
						}
					}
				})
				.collect();
			plan.cover_centimetres = self
				.furniture
				.iter()
				.map(|found| {
					match found {
						None => 0,
						Some(found) => {
							math::round(found.cover * 100.0)
								.clamp(1, i64::from(HuntPlan::MAX_COVER_CENTIMETRES))
								as i32
						}
					}
				})
				.collect();
			plan.hide_decimetres = (0..HuntPlan::SLICES)
				.map(|slice| self.hide_decimetres(slice))
				.collect();
			plan.ledges = self
				.furniture
				.iter()
				.map(|found| found.is_some_and(|found| found.ledge))
				.collect();
		}

		plan
	}

	/// Where a husk hides behind the cover along `slice`, as a plan records
	/// it: always beyond the reach, or nowhere.
	fn hide_decimetres(&self, slice: usize) -> i32 {
		let (Some(hide), Some(reach)) = (
			self.furniture[slice].and_then(|found| found.hide),
			self.reach[slice],
		) else {
			return 0;
		};
		let decimetres = math::round(hide * 10.0);

		if decimetres > math::floor(reach * 10.0)
			&& decimetres <= i64::from(HuntPlan::MAX_REACH_DECIMETRES)
		{
			decimetres as i32
		} else {
			0
		}
	}

	fn reckon(&mut self, dirty_slices: u128) {
		let ground = self.vessel.gait() == Gait::Ground;
		// One ring of the map in metres.
		let unit = if ground {
			self.floor_depth().unwrap_or(0.0)
		} else {
			1.0
		};

		for slice in 0..HuntPlan::SLICES {
			if (dirty_slices & (1_u128 << slice)) == 0 {
				continue;
			}
			let read = (f64::from(self.seen[slice]) / POINTS_TO_READ).clamp(0.0, 1.0);
			self.read[slice] = read;
			if read < 1.0 {
				self.reach[slice] = None;
				self.furniture[slice] = None;
				continue;
			}
			let clear = self.clear_rings(slice);
			let mut reach = clear.rings as f64 * RING_SIZE * unit;
			let furniture = self.furniture_along(slice, clear);
			// A husk that flies keeps in front of a ledge it may come down onto,
			// so that it lands on the front of it.
			if !ground
				&& let Some(found) = furniture
				&& found.front < reach
			{
				reach = found.front;
			}
			self.reach[slice] = Some(reach);
			self.furniture[slice] = furniture;
		}
	}

	/// What stands at the end of slice `slice`'s way, given how far the way
	/// is `clear`.
	///
	/// For a husk that walks, that is whatever stopped the way. For one that
	/// flies, it is the nearest level top below its air, which it may come
	/// down and perch on; anything else under it does not matter to it.
	fn furniture_along(&self, slice: usize, clear: Clear) -> Option<Furniture> {
		let unit = self.floor_depth()?;
		let ground = self.vessel.gait() == Gait::Ground;
		let front = if ground {
			clear.stopped.then_some(clear.rings)
		} else {
			self.first_solid(slice, 0)
		}?;

		// It runs back from its front for as long as there is something
		// there, over a gap of a ring at most.
		let mut back = front;
		let mut gap = 0;

		for ring in front + 1..RINGS {
			if self.is_solid(slice, ring) {
				back = ring;
				gap = 0;
			} else {
				gap += 1;

				if gap > 1 {
					break;
				}
			}
		}

		let top = self.top_bin(slice, front, back)?;
		let cover = (top + 1) as f64 * HEIGHT_BIN * unit;
		let ledge = self.is_ledge(slice, front, back, top);

		if !ground {
			return ledge.then_some(Furniture {
				front: front as f64 * RING_SIZE * unit,
				cover,
				ledge: true,
				hide: None,
			});
		}

		Some(Furniture {
			front: front as f64 * RING_SIZE * unit,
			cover,
			ledge,
			hide: self.hide_behind(slice, back, unit),
		})
	}

	/// Where a husk can hide behind something whose far side is in ring
	/// `back` of slice `slice`: in the gap before whatever stands behind it,
	/// or just behind it where floor shows beyond it. Nowhere when nothing
	/// behind it was seen at all: a husk is not put somewhere unknown.
	fn hide_behind(&self, slice: usize, back: usize, unit: f64) -> Option<f64> {
		let behind = (back + 1) as f64 * RING_SIZE * unit;
		let next = self.first_solid(slice, back + 2);
		let floor_beyond =
			(back + 1..RINGS).any(|ring| self.floor_at[slice * RINGS + ring] >= POINTS_OF_FLOOR);
		let half_width = self.vessel.half_width();
		let width = 2.0 * half_width;
		let end = match next {
			Some(next) => next as f64 * RING_SIZE * unit,
			None if floor_beyond => behind + width + 3.0 * HIDE_GAP,
			None => return None,
		};

		if end - behind < width + HIDE_GAP {
			return None;
		}

		Some((behind + half_width + HIDE_GAP).min(end - half_width - HIDE_GAP / 2.0))
	}

	fn is_solid(&self, slice: usize, ring: usize) -> bool {
		let cell = slice * RINGS + ring;

		self.solid[cell] >= POINTS_IN_THE_WAY && self.solid[cell] >= self.floor_at[cell]
	}

	fn first_solid(&self, slice: usize, from: usize) -> Option<usize> {
		(from..RINGS).find(|ring| self.is_solid(slice, *ring))
	}

	/// The highest height bin with enough points in it, over rings `front`
	/// to `back` of `slice`.
	fn top_bin(&self, slice: usize, front: usize, back: usize) -> Option<usize> {
		(0..HEIGHT_BINS).rev().find(|bin| {
			let points: u32 = (front..=back)
				.map(|ring| self.heights[(slice * RINGS + ring) * HEIGHT_BINS + bin])
				.sum();

			points >= POINTS_OF_TOP
		})
	}

	/// Whether the top of what stands over rings `front` to `back` of
	/// `slice`, in bin `top`, is a level surface: a table or a counter.
	fn is_ledge(&self, slice: usize, front: usize, back: usize, top: usize) -> bool {
		let mut level = 0;
		let mut all = 0;

		for ring in front..=back {
			for bin in top.saturating_sub(1)..=top {
				let at = (slice * RINGS + ring) * HEIGHT_BINS + bin;

				level += self.level_heights[at];
				all += self.heights[at];
			}
		}

		level >= POINTS_OF_LEDGE && f64::from(level) >= f64::from(all) * LEDGE_SHARE
	}

	/// How many rings out from the player slice `index` is clear, and
	/// whether that is because something stands in the next one. Otherwise
	/// it is as far as floor has been seen.
	fn clear_rings(&self, index: usize) -> Clear {
		let mut floor_end = 0;

		for ring in 0..RINGS {
			let cell = index * RINGS + ring;

			// Something stands here, and more of it than there is floor.
			if self.taken[cell] >= POINTS_IN_THE_WAY && self.taken[cell] >= self.clear[cell] {
				return Clear {
					rings: ring,
					stopped: true,
				};
			}

			if self.clear[cell] >= POINTS_OF_FLOOR {
				floor_end = ring + 1;
			}
		}

		// In the air nothing has to be underfoot: clear is clear, as far as
		// the map goes.
		Clear {
			rings: if self.vessel.gait() == Gait::Ground {
				floor_end
			} else {
				RINGS
			},
			stopped: false,
		}
	}

	/// Single-pass ground + furniture charting for walking husks: computes
	/// radial distance, ring, and slice once per point instead of twice.
	fn chart_ground_and_furniture(
		&mut self,
		cloud: &Cloud,
		level: &[bool],
		floor: f64,
		scale: f64,
		origin: Vec3,
		unit: f64,
	) -> u128 {
		if !unit.is_finite() || unit <= 0.0 || !scale.is_finite() || scale <= 0.0 {
			return 0;
		}
		let top = 2.0 * self.vessel.half_height() + 0.2;
		let inv_ring = 1.0 / (unit * RING_SIZE);
		let inv_height = 1.0 / (unit * HEIGHT_BIN);
		let self_sq = SELF_METRES * SELF_METRES;
		let max_away = RINGS as f64 * unit * RING_SIZE;
		let max_away_sq = max_away * max_away;
		let mut dirty = 0_u128;

		for (index, &is_level) in level.iter().enumerate() {
			let y = f64::from(cloud.ys[index]);
			if !y.is_finite() {
				continue;
			}
			let x = f64::from(cloud.xs[index]) * scale + origin.x;
			let z = f64::from(cloud.zs[index]) * scale + origin.z;
			let away_sq = x * x + z * z;
			if !(away_sq >= self_sq && away_sq < max_away_sq) {
				continue;
			}
			let away = math::sqrt(away_sq);
			let ring = (away * inv_ring) as usize;
			if ring >= RINGS {
				continue;
			}

			let above = (floor + y) * scale;
			let slack = FLOOR_SLACK + FLOOR_SLACK_PER_METRE * away;
			let slice = slice_of(x, z);
			let cell = slice * RINGS + ring;
			let on_floor = above.abs() <= slack;
			let mut touched = false;

			if on_floor {
				self.clear[cell] += 1;
				self.seen[slice] += 1;
				touched = true;
			} else if above > slack && above < top {
				self.taken[cell] += 1;
				self.seen[slice] += 1;
				touched = true;
			}

			if y * scale + origin.y <= 0.0 {
				if on_floor {
					self.floor_at[cell] += 1;
					touched = true;
				} else if above >= slack {
					let bin = ((above * inv_height) as usize).min(HEIGHT_BINS - 1);
					self.solid[cell] += 1;
					self.heights[cell * HEIGHT_BINS + bin] += 1;
					if is_level {
						self.level_heights[cell * HEIGHT_BINS + bin] += 1;
					}
					touched = true;
				}
			}

			if touched {
				dirty |= 1_u128 << slice;
			}
		}

		dirty
	}

	/// Charts what stands on the floor in one picture, for flying/still husks.
	fn chart_furniture(
		&mut self,
		cloud: &Cloud,
		level: &[bool],
		floor: f64,
		scale: f64,
		origin: Vec3,
		unit: f64,
	) -> u128 {
		if !unit.is_finite() || unit <= 0.0 || !scale.is_finite() || scale <= 0.0 {
			return 0;
		}
		let inv_ring = 1.0 / (unit * RING_SIZE);
		let inv_height = 1.0 / (unit * HEIGHT_BIN);
		let self_sq = SELF_METRES * SELF_METRES;
		let max_away = RINGS as f64 * unit * RING_SIZE;
		let max_away_sq = max_away * max_away;
		let mut dirty = 0_u128;

		for (index, &is_level) in level.iter().enumerate() {
			let y = f64::from(cloud.ys[index]);
			if !y.is_finite() || y * scale + origin.y > 0.0 {
				continue;
			}
			let x = f64::from(cloud.xs[index]) * scale + origin.x;
			let z = f64::from(cloud.zs[index]) * scale + origin.z;
			let away_sq = x * x + z * z;
			if !(away_sq >= self_sq && away_sq < max_away_sq) {
				continue;
			}
			let away = math::sqrt(away_sq);
			let ring = (away * inv_ring) as usize;
			if ring >= RINGS {
				continue;
			}

			let above = (floor + y) * scale;
			let slack = FLOOR_SLACK + FLOOR_SLACK_PER_METRE * away;
			let slice = slice_of(x, z);
			let cell = slice * RINGS + ring;

			if above.abs() <= slack {
				self.floor_at[cell] += 1;
				dirty |= 1_u128 << slice;
				continue;
			}

			if above < slack {
				continue;
			}

			// How high it stands, as a share of the hunt's own floor.
			let bin = ((above * inv_height) as usize).min(HEIGHT_BINS - 1);

			self.solid[cell] += 1;
			self.heights[cell * HEIGHT_BINS + bin] += 1;

			if is_level {
				self.level_heights[cell * HEIGHT_BINS + bin] += 1;
			}
			dirty |= 1_u128 << slice;
		}

		dirty
	}

	/// Charts one picture for a husk that flies: what matters is whatever
	/// stands up into the band of air it keeps to.
	///
	/// A level surface is left out. A ceiling or a floor that the band
	/// brushes is not a wall, and would otherwise close off the whole room.
	fn chart_air(&mut self, cloud: &Cloud, level: &[bool], origin: Vec3, scale: f64) -> u128 {
		let low = self.vessel.low_metres() - self.vessel.half_height() - 0.1;
		let high = self.vessel.high_metres() + self.vessel.half_height() + 0.1;
		let self_sq = SELF_METRES * SELF_METRES;
		let inv_ring = 1.0 / RING_SIZE;
		let mut dirty = 0_u128;

		for (index, &is_level) in level.iter().enumerate() {
			if is_level {
				continue;
			}
			let y = f64::from(cloud.ys[index]) * scale + origin.y;
			if !(low..=high).contains(&y) {
				continue;
			}
			let x = f64::from(cloud.xs[index]) * scale + origin.x;
			let z = f64::from(cloud.zs[index]) * scale + origin.z;
			let away_sq = x * x + z * z;
			if !(away_sq >= self_sq && away_sq.is_finite()) {
				continue;
			}
			let away = math::sqrt(away_sq);
			let slice = slice_of(x, z);
			let ring = (away * inv_ring) as usize;

			self.seen[slice] += 1;
			dirty |= 1_u128 << slice;

			if ring < RINGS {
				self.taken[slice * RINGS + ring] += 1;
			}
		}

		dirty
	}
}

impl Cloud {
	/// One picture's points at every other pixel each way.
	pub(crate) fn of(frame: &DepthFrame) -> Self {
		Self::of_filtered(frame, |_, _| true)
	}

	/// One picture's points at every other pixel each way, leaving out the
	/// pixels `keep` refuses. `keep` is asked about the sampled pixel's own
	/// column and row.
	pub(crate) fn of_filtered(frame: &DepthFrame, keep: impl Fn(usize, usize) -> bool) -> Self {
		let columns = frame.width / STRIDE;
		let rows = frame.height / STRIDE;
		let count = columns * rows;
		let mut xs = Vec::with_capacity(count);
		let mut ys = Vec::with_capacity(count);
		let mut zs = Vec::with_capacity(count);
		let pose = frame.pose;

		if count == 0 {
			return Self { xs, ys, zs };
		}

		let inv_w = 2.0 / frame.width as f64;
		let inv_h = 2.0 / frame.height as f64;
		let across_step = (STRIDE as f64) * inv_w * frame.tan_half_width;
		let across_start = (0.5 * inv_w - 1.0) * frame.tan_half_width;
		let step_x = pose.right.x * across_step;
		let step_y = pose.right.y * across_step;
		let step_z = pose.right.z * across_step;

		for row in 0..rows {
			let sampled_row = row * STRIDE;
			let up = (1.0 - (sampled_row as f64 + 0.5) * inv_h) * frame.tan_half_height;
			let mut dir_x = pose.forward.x + pose.up.x * up + pose.right.x * across_start;
			let mut dir_y = pose.forward.y + pose.up.y * up + pose.right.y * across_start;
			let mut dir_z = pose.forward.z + pose.up.z * up + pose.right.z * across_start;
			let row_base = sampled_row * frame.width;

			for column in 0..columns {
				let sampled_column = column * STRIDE;
				let depth = f64::from(frame.depths[row_base + sampled_column]);
				if depth > 0.0 && depth.is_finite() && keep(sampled_column, sampled_row) {
					xs.push((depth * dir_x) as f32);
					ys.push((depth * dir_y) as f32);
					zs.push((depth * dir_z) as f32);
				} else {
					xs.push(f32::NAN);
					ys.push(f32::NAN);
					zs.push(f32::NAN);
				}
				dir_x += step_x;
				dir_y += step_y;
				dir_z += step_z;
			}
		}

		Self { xs, ys, zs }
	}
}

/// Which points lie on a level surface: a floor, a table top, a ceiling.
/// Computes the cross-product levelness directly without allocating an
/// intermediate `Vec<Option<SurfaceNormal>>` or calling `sqrt`.
pub(crate) fn level_points(cloud: &Cloud, columns: usize, rows: usize) -> Vec<bool> {
	let len = cloud.xs.len();
	let mut level = vec![false; len];
	if columns < 2 || rows < 2 {
		return level;
	}
	let level_share_sq = LEVEL_SHARE * LEVEL_SHARE;
	let xs = cloud.xs.as_slice();
	let ys = cloud.ys.as_slice();
	let zs = cloud.zs.as_slice();

	for row in 0..rows {
		let row_base = row * columns;
		let down_offset: isize = if row + 1 < rows {
			columns as isize
		} else {
			-(columns as isize)
		};
		for column in 0..columns {
			let at = row_base + column;
			let right = if column + 1 < columns { at + 1 } else { at - 1 };
			let down = (at as isize + down_offset) as usize;

			let x0 = f64::from(xs[at]);
			let y0 = f64::from(ys[at]);
			let z0 = f64::from(zs[at]);
			let ax = f64::from(xs[right]) - x0;
			let ay = f64::from(ys[right]) - y0;
			let az = f64::from(zs[right]) - z0;
			let bx = f64::from(xs[down]) - x0;
			let by = f64::from(ys[down]) - y0;
			let bz = f64::from(zs[down]) - z0;
			let nx = ay * bz - az * by;
			let ny = az * bx - ax * bz;
			let nz = ax * by - ay * bx;
			let len_sq = nx * nx + ny * ny + nz * nz;

			if len_sq > 0.0 && ny * ny >= level_share_sq * len_sq {
				level[at] = true;
			}
		}
	}

	level
}

/// How far below the eye the floor in one picture is, or `None` when the
/// picture shows none.
///
/// The floor is the lowest level surface that is a fair share of the level
/// surface in the picture: lower than the table tops, and bigger than the
/// stray points below it that a depth model always leaves.
fn floor_of(ys: &[f32], level: &[bool]) -> Option<f64> {
	const MAX_STEPS: usize = 160;
	let steps = math::ceil((FLOOR_FARTHEST - FLOOR_NEAREST) / FLOOR_STEP) as usize;
	debug_assert!(steps <= MAX_STEPS);
	let mut tally = [0.0_f64; MAX_STEPS];
	let inv_step = 1.0 / FLOOR_STEP;

	for (&y, &is_level) in ys.iter().zip(level) {
		if !is_level {
			continue;
		}
		let below = -f64::from(y);
		if (FLOOR_NEAREST..FLOOR_FARTHEST).contains(&below) {
			let bin = ((below - FLOOR_NEAREST) * inv_step) as usize;
			if bin < steps {
				tally[bin] += 1.0;
			}
		}
	}

	// A surface's points land a few steps either side of it, so each step is
	// taken together with its neighbours.
	let mut smooth = [0.0_f64; MAX_STEPS];
	let mut most = 0.0_f64;
	for (step, slot) in smooth.iter_mut().enumerate().take(steps) {
		let low = step.saturating_sub(FLOOR_BLUR);
		let high = (step + FLOOR_BLUR).min(steps - 1);
		let mut sum = 0.0;
		for &count in &tally[low..=high] {
			sum += count;
		}
		*slot = sum;
		if sum > most {
			most = sum;
		}
	}

	if most < FLOOR_POINTS {
		return None;
	}

	let threshold = (most * FLOOR_SHARE).max(FLOOR_POINTS);
	for step in (0..steps).rev() {
		let cur = smooth[step];
		let is_peak = cur >= threshold
			&& cur >= smooth[step.saturating_sub(1)]
			&& cur >= smooth[(step + 1).min(steps - 1)];

		if !is_peak {
			continue;
		}

		// The blur spreads a surface over the steps either side of it, so
		// the peak says roughly where the floor is and the points around it
		// say exactly.
		let mut points = 0.0;
		let mut depth = 0.0;

		let nearest = step.saturating_sub(2 * FLOOR_BLUR);
		let farthest = (step + 2 * FLOOR_BLUR).min(steps - 1);

		for (near, &count) in tally.iter().enumerate().take(farthest + 1).skip(nearest) {
			points += count;
			depth += count * (FLOOR_NEAREST + (near as f64 + 0.5) * FLOOR_STEP);
		}

		return Some(depth / points);
	}

	None
}

#[inline]
fn slice_of(x: f64, z: f64) -> usize {
	let idx = ((math::fast_atan2(x, z) + PI) * (1.0 / HuntPlan::SLICE_WIDTH)) as i64;
	idx.clamp(0, HuntPlan::SLICES as i64 - 1) as usize
}
