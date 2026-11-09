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
#[derive(Clone, Debug)]
pub struct RoomMap {
	/// The husk the room is being read for: how it gets about decides what
	/// counts as in its way, and its size decides where it fits.
	vessel: Vessel,

	clear: Vec<u32>,
	taken: Vec<u32>,
	seen: Vec<u32>,
	floors: Vec<f64>,
	measured_floor: Option<f64>,

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

	frames: usize,
	last_picture_floor: Option<f64>,

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

/// The normal of the surface through one point of a [`Cloud`], before it is
/// scaled to length 1, so that its tilt can be compared without dividing
/// twice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SurfaceNormal {
	pub(crate) x: f64,
	pub(crate) y: f64,
	pub(crate) z: f64,
	pub(crate) length: f64,
}

impl SurfaceNormal {
	/// Whether the surface faces nearly straight up or down: a floor, a
	/// table top, a ceiling.
	pub(crate) fn is_level(self) -> bool {
		self.y.abs() / self.length >= LEVEL_SHARE
	}

	/// How much of this normal lies in the horizontal plane, from 0 for a
	/// level surface to 1 for a vertical one.
	pub(crate) fn horizontal_share(self) -> f64 {
		math::sqrt(self.x * self.x + self.z * self.z) / self.length
	}

	/// The direction this surface faces across the floor, folded into one
	/// quarter turn: a wall and the same wall seen the other way round are
	/// one axis.
	pub(crate) fn folded_angle(self) -> f64 {
		math::atan2(self.x, self.z).rem_euclid(PI / 2.0)
	}
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
			floors: Vec::new(),
			measured_floor: None,
			floor_at: vec![0; cells],
			solid: vec![0; cells],
			heights: vec![0; cells * HEIGHT_BINS],
			level_heights: vec![0; cells * HEIGHT_BINS],
			frames: 0,
			last_picture_floor: None,
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
		self.frames
	}

	/// How far below the eye of the last picture the floor was in that
	/// picture's own metres. Each picture is read against its own floor, and
	/// this is how far to scale it to agree with
	/// [`floor_depth`](Self::floor_depth).
	pub const fn last_picture_floor(&self) -> Option<f64> {
		self.last_picture_floor
	}

	/// How far the floor is below the hunt's origin, in metres: as the
	/// platform measured it, or else the mean of what every accepted picture
	/// made of it, brought back to the hunt's origin.
	///
	/// The mean is the one estimate the plan and the room's grid both use, so
	/// the grid a husk walks is always one the plan agrees with.
	pub fn floor_depth(&self) -> Option<f64> {
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

	/// Whether the floor's depth was measured by the platform rather than
	/// found in the pictures.
	pub const fn floor_is_measured(&self) -> bool {
		self.measured_floor.is_some()
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
		if !(FLOOR_NEAREST..=FLOOR_FARTHEST).contains(&eye_height) {
			return false;
		}

		self.measured_floor = Some(eye_height);
		self.reckon();

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
		let mut cloud = Cloud::of(frame);

		let level = level_points(&cloud, columns, rows);
		let measured = self.measured_floor.filter(|_| !estimated);
		let own = if measured.is_some() {
			None
		} else {
			floor_of(&cloud.ys, &level)
		};
		let agreed = if self.floors.len() < PICTURES_TO_AGREE {
			None
		} else {
			self.picture_floor_depth()
		};

		// The floor below this picture's own eye, in its own metres: the
		// measured floor is below the hunt's origin, so a picture taken from
		// above or below it sees the floor that much farther away.
		let floor = measured.map(|floor| floor + frame.origin.y).or(own);
		// The same floor below the hunt's origin. Every picture is measured
		// against this, so a picture taken from elsewhere still says what the
		// room is.
		let floor_at_origin = floor.map(|floor| floor - frame.origin.y);
		// A picture with a table across it and hardly any floor takes the
		// table for the floor. Once a few pictures agree where the floor is,
		// one that puts it somewhere else altogether is not charted.
		let agrees = floor_at_origin.is_some_and(|floor| {
			measured.is_some()
				|| agreed.is_none_or(|agreed| (floor / agreed - 1.0).abs() <= FLOOR_LEEWAY)
		});

		if self.vessel.gait() == Gait::Ground && !agrees {
			return false;
		}
		// An estimate cannot borrow the measured floor as its own scale.
		if estimated && self.measured_floor.is_some() && !agrees {
			return false;
		}
		// Only accepted pictures contribute to the floor at the hunt's origin.
		if agrees && let Some(own) = own {
			self.floors.push(own - frame.origin.y);
		}
		let chart = floor.filter(|_| agrees);
		let unit = self.floor_depth();
		let scale = chart.map(|floor| {
			(unit.unwrap_or(floor - frame.origin.y) + frame.origin.y) / floor
		});

		if let (Some(floor), Some(scale), Some(unit)) = (chart, scale, unit) {
			if self.vessel.gait() == Gait::Ground {
				self.chart_floor(&cloud, floor, scale, frame.origin, unit);
			}
			self.chart_furniture(&cloud, &level, floor, scale, frame.origin, unit);
		}
		if self.vessel.gait() != Gait::Ground {
			if estimated && self.measured_floor.is_some() {
				// Air is classified in metres; the model's floor calibrates
				// distances from this picture's eye before its origin is added.
				let Some(scale) = scale else { return false; };
				for axis in [&mut cloud.xs, &mut cloud.ys, &mut cloud.zs] {
					for coordinate in axis {
						*coordinate *= scale as f32;
					}
				}
			}
			self.chart_air(&cloud, &level, frame.origin);
		}

		self.last_picture_floor = floor;
		self.frames += 1;
		self.reckon();

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

	fn reckon(&mut self) {
		// One ring of the map in metres.
		let unit = if self.vessel.gait() == Gait::Ground {
			self.floor_depth().unwrap_or(0.0)
		} else {
			1.0
		};
		let clear: Vec<Clear> = (0..HuntPlan::SLICES)
			.map(|slice| self.clear_rings(slice))
			.collect();

		self.read = self
			.seen
			.iter()
			.map(|seen| (f64::from(*seen) / POINTS_TO_READ).clamp(0.0, 1.0))
			.collect();
		self.reach = (0..HuntPlan::SLICES)
			.map(|slice| {
				(self.read[slice] >= 1.0).then(|| clear[slice].rings as f64 * RING_SIZE * unit)
			})
			.collect();
		self.furniture = (0..HuntPlan::SLICES)
			.map(|slice| {
				if self.read[slice] < 1.0 {
					None
				} else {
					self.furniture_along(slice, clear[slice])
				}
			})
			.collect();

		// A husk that flies keeps in front of a ledge it may come down onto,
		// so that it lands on the front of it.
		if self.vessel.gait() != Gait::Ground {
			for (reach, found) in self.reach.iter_mut().zip(&self.furniture) {
				if let (Some(metres), Some(found)) = (*reach, found)
					&& found.front < metres
				{
					*reach = Some(found.front);
				}
			}
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

	/// Charts one picture for a husk that walks. Every length is divided by
	/// `floor`, the picture's own floor depth, before it goes on the map, and
	/// `scale` brings it from the picture's metres to the hunt's. `origin`
	/// is where the picture was taken, so its furniture lands where it is in
	/// the room.
	fn chart_floor(&mut self, cloud: &Cloud, floor: f64, scale: f64, origin: Vec3, unit: f64) {
		let top = 2.0 * self.vessel.half_height() + 0.2;

		for index in 0..cloud.xs.len() {
			let Some((x, y, z, away, ring)) = cloud.point(index, scale, origin, unit) else {
				continue;
			};
			let above = floor + y;
			let slack = FLOOR_SLACK + FLOOR_SLACK_PER_METRE * away;
			let slice = slice_of(x, z);
			let cell = slice * RINGS + ring;

			if above.abs() <= slack {
				self.clear[cell] += 1;
				self.seen[slice] += 1;
			} else if above > slack && above < top {
				self.taken[cell] += 1;
				self.seen[slice] += 1;
			}
		}
	}

	/// Charts what stands on the floor in one picture, for any husk: every
	/// length is divided by `floor`, the picture's own floor depth.
	///
	/// Only what is no higher than the eye is charted. Anything taller is a
	/// wall as far as a husk is concerned, and a ceiling is not furniture.
	fn chart_furniture(
		&mut self,
		cloud: &Cloud,
		level: &[bool],
		floor: f64,
		scale: f64,
		origin: Vec3,
		unit: f64,
	) {
		for (index, level) in level.iter().enumerate() {
			let Some((x, y, z, away, ring)) = cloud.point(index, scale, origin, unit) else {
				continue;
			};

			if y > 0.0 {
				continue;
			}

			let above = floor + y;
			let slack = FLOOR_SLACK + FLOOR_SLACK_PER_METRE * away;
			let cell = slice_of(x, z) * RINGS + ring;

			if above.abs() <= slack {
				self.floor_at[cell] += 1;
				continue;
			}

			if above < slack {
				continue;
			}

			// How high it stands, as a share of the hunt's own floor: the
			// picture's metres scaled to the room's, then taken against the
			// floor the plan stands on. A picture taken from above or below
			// the hunt's origin sees the floor that much farther below its
			// own eye, so its heights must come back to the origin's eye
			// here too, or the room's furniture reads proportionally lower
			// than it is.
			let bin = math::floor(above * scale / unit / HEIGHT_BIN)
				.clamp(0, HEIGHT_BINS as i64 - 1) as usize;

			self.solid[cell] += 1;
			self.heights[cell * HEIGHT_BINS + bin] += 1;

			if *level {
				self.level_heights[cell * HEIGHT_BINS + bin] += 1;
			}
		}
	}

	/// Charts one picture for a husk that flies: what matters is whatever
	/// stands up into the band of air it keeps to.
	///
	/// A level surface is left out. A ceiling or a floor that the band
	/// brushes is not a wall, and would otherwise close off the whole room.
	fn chart_air(&mut self, cloud: &Cloud, level: &[bool], origin: Vec3) {
		let low = self.vessel.low_metres() - self.vessel.half_height() - 0.1;
		let high = self.vessel.high_metres() + self.vessel.half_height() + 0.1;

		for index in 0..cloud.xs.len() {
			let x = f64::from(cloud.xs[index]) + origin.x;
			let y = f64::from(cloud.ys[index]) + origin.y;
			let z = f64::from(cloud.zs[index]) + origin.z;
			let away = math::sqrt(x * x + z * z);
			let in_the_band = away >= SELF_METRES && !level[index] && y >= low && y <= high;

			if !in_the_band {
				continue;
			}

			let slice = slice_of(x, z);
			let ring = math::floor(away / RING_SIZE) as usize;

			self.seen[slice] += 1;

			if ring < RINGS {
				self.taken[slice * RINGS + ring] += 1;
			}
		}
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
		let mut cloud = Self {
			xs: vec![0.0; count],
			ys: vec![0.0; count],
			zs: vec![0.0; count],
		};
		let pose = frame.pose;

		for row in 0..rows {
			for column in 0..columns {
				let sampled_column = column * STRIDE;
				let sampled_row = row * STRIDE;
				let (across, up) = frame.tangents(sampled_column, sampled_row);
				let depth = f64::from(frame.depths[sampled_row * frame.width + sampled_column]);
				let depth = if depth > 0.0 && depth.is_finite() && keep(sampled_column, sampled_row)
				{
					depth
				} else {
					f64::NAN
				};
				let index = row * columns + column;

				cloud.xs[index] =
					(depth * (pose.forward.x + pose.right.x * across + pose.up.x * up)) as f32;
				cloud.ys[index] =
					(depth * (pose.forward.y + pose.right.y * across + pose.up.y * up)) as f32;
				cloud.zs[index] =
					(depth * (pose.forward.z + pose.right.z * across + pose.up.z * up)) as f32;
			}
		}

		cloud
	}

	/// Point `index` in the hunt's frame as `(x, y, z)`, how far it is from
	/// the hunt's origin across the floor, and the ring it falls in when the
	/// floor is `unit` below the origin. `None` for a point that was not
	/// seen, is the player's own hands and feet, or lies beyond the map.
	///
	/// `scale` brings the picture's metres to the hunt's; `origin` is where
	/// its eye was. `y` stays relative to that eye, which is what says how
	/// high the point is above the floor.
	fn point(
		&self,
		index: usize,
		scale: f64,
		origin: Vec3,
		unit: f64,
	) -> Option<(f64, f64, f64, f64, usize)> {
		let y = f64::from(self.ys[index]);
		let x = f64::from(self.xs[index]) * scale + origin.x;
		let z = f64::from(self.zs[index]) * scale + origin.z;
		let away = math::sqrt(x * x + z * z);

		// A point that was not seen is NaN, and fails this too.
		if away.is_nan() || away < SELF_METRES {
			return None;
		}

		let ring = math::floor(away / unit / RING_SIZE);

		(ring < RINGS as i64).then_some((x, y, z, away, ring as usize))
	}
}

/// The surface normal at every point of a cloud, or `None` where no surface
/// can be told: a point next to one that was not seen.
///
/// The surface at a point faces along the cross product of the steps to the
/// next point across and the next point down.
pub(crate) fn surface_normals(
	cloud: &Cloud,
	columns: usize,
	rows: usize,
) -> Vec<Option<SurfaceNormal>> {
	let mut normals = vec![None; cloud.xs.len()];

	for row in 0..rows {
		for column in 0..columns {
			let at = row * columns + column;
			// At the last column and the last row the step is taken
			// backwards. Which way the cross product ends up pointing does
			// not matter.
			let right = if column + 1 < columns {
				at + 1
			} else {
				at.wrapping_sub(1)
			};
			let down = if row + 1 < rows {
				at + columns
			} else {
				at.wrapping_sub(columns)
			};

			if right >= cloud.xs.len() || down >= cloud.xs.len() {
				continue;
			}

			let ax = f64::from(cloud.xs[right]) - f64::from(cloud.xs[at]);
			let ay = f64::from(cloud.ys[right]) - f64::from(cloud.ys[at]);
			let az = f64::from(cloud.zs[right]) - f64::from(cloud.zs[at]);
			let bx = f64::from(cloud.xs[down]) - f64::from(cloud.xs[at]);
			let by = f64::from(cloud.ys[down]) - f64::from(cloud.ys[at]);
			let bz = f64::from(cloud.zs[down]) - f64::from(cloud.zs[at]);
			let nx = ay * bz - az * by;
			let ny = az * bx - ax * bz;
			let nz = ax * by - ay * bx;
			let length = math::length(nx, ny, nz);

			// NaN, from a point not seen, fails this too.
			if length > 0.0 {
				normals[at] = Some(SurfaceNormal {
					x: nx,
					y: ny,
					z: nz,
					length,
				});
			}
		}
	}

	normals
}

/// Which points lie on a level surface: a floor, a table top, a ceiling.
fn level_points(cloud: &Cloud, columns: usize, rows: usize) -> Vec<bool> {
	surface_normals(cloud, columns, rows)
		.into_iter()
		.map(|normal| normal.is_some_and(SurfaceNormal::is_level))
		.collect()
}

/// How far below the eye the floor in one picture is, or `None` when the
/// picture shows none.
///
/// The floor is the lowest level surface that is a fair share of the level
/// surface in the picture: lower than the table tops, and bigger than the
/// stray points below it that a depth model always leaves.
fn floor_of(ys: &[f32], level: &[bool]) -> Option<f64> {
	let steps = math::ceil((FLOOR_FARTHEST - FLOOR_NEAREST) / FLOOR_STEP) as usize;
	let mut tally = vec![0.0; steps];

	for (y, level) in ys.iter().zip(level) {
		let below = -f64::from(*y);

		if *level && (FLOOR_NEAREST..FLOOR_FARTHEST).contains(&below) {
			tally[math::floor((below - FLOOR_NEAREST) / FLOOR_STEP) as usize] += 1.0;
		}
	}

	// A surface's points land a few steps either side of it, so each step is
	// taken together with its neighbours.
	let smooth: Vec<f64> = (0..steps)
		.map(|step| {
			tally[step.saturating_sub(FLOOR_BLUR)..=(step + FLOOR_BLUR).min(steps - 1)]
				.iter()
				.sum()
		})
		.collect();
	let most = smooth.iter().copied().fold(0.0, f64::max);

	if most < FLOOR_POINTS {
		return None;
	}

	for step in (0..steps).rev() {
		let is_peak = smooth[step] >= most * FLOOR_SHARE
			&& smooth[step] >= FLOOR_POINTS
			&& smooth[step] >= smooth[step.saturating_sub(1)]
			&& smooth[step] >= smooth[(step + 1).min(steps - 1)];

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

		for (near, count) in tally.iter().enumerate().take(farthest + 1).skip(nearest) {
			points += count;
			depth += count * (FLOOR_NEAREST + (near as f64 + 0.5) * FLOOR_STEP);
		}

		return Some(depth / points);
	}

	None
}

fn slice_of(x: f64, z: f64) -> usize {
	math::floor((math::atan2(x, z) + PI) / HuntPlan::SLICE_WIDTH)
		.clamp(0, HuntPlan::SLICES as i64 - 1) as usize
}
