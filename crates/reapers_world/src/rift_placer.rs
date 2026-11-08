//! Where a rift stands, as one reaper's phone sees it.

use reapers_rules::GeoFix;
use reapers_rules::GeoPoint;
use reapers_rules::GeoStanding;
use reapers_rules::RiftFrame;
use reapers_rules::RiftPhase;
use reapers_rules::RiftSighting;
use reapers_rules::Timestamp;
use reapers_rules::wrap_angle;

/// Works out, frame by frame, where a rift stands for one reaper.
///
/// A rift stands at one place on the earth for its whole circle. The first
/// reaper whose phone knows where it is tells the rift, and the rift takes
/// its place from that. After that every phone works out for itself which
/// way that place lies and points its reaper at it, so people standing
/// together are turned the same way.
///
/// How well that holds depends on the phone: a fix is good to a few metres
/// out of doors and worse inside, and a compass to ten degrees or so, less
/// near steel. It is enough to face the same way, not to touch the same
/// stone. See docs/gameplay.md.
#[derive(Clone, Debug, PartialEq)]
pub struct RiftPlacer {
	/// Where the phone last said it was.
	pub fix: Option<GeoFix>,

	/// Where this reaper sees the rift, or `None` while it stands dead
	/// ahead: no place taken yet, nothing known of where the phone is, or
	/// the rift too far off to point at.
	pub sighting: Option<RiftSighting>,

	frames_since_told: u32,
	frames_waited: u32,
}

impl Default for RiftPlacer {
	fn default() -> Self {
		Self::new()
	}
}

impl RiftPlacer {
	/// How far the sighting moves, each frame, toward where the rift is now
	/// seen. Where a phone thinks it is shifts by a few metres from one
	/// second to the next, and the husk should not jump with every guess.
	pub const EASING: f64 = 0.08;
	/// Frames a phone is given to find itself before it is taken not to
	/// know where it is. About eight seconds.
	pub const FRAMES_TO_FIND_ITSELF: u32 = 160;
	/// Frames between one telling of where this reaper stands and the next,
	/// while the rift has yet to take its place. About half a second.
	pub const TELL_EVERY_FRAMES: u32 = 10;

	/// A placer that knows nothing of where the phone is.
	pub const fn new() -> Self {
		Self {
			fix: None,
			sighting: None,
			frames_since_told: Self::TELL_EVERY_FRAMES,
			frames_waited: 0,
		}
	}

	/// Whether the phone is still being given time to find itself.
	pub const fn still_looking(&self) -> bool {
		self.fix.is_none() && self.frames_waited < Self::FRAMES_TO_FIND_ITSELF
	}

	/// Takes in `frame` at the time `now`, with the phone facing `facing`
	/// and north lying at `north` in its view, both as bearings in the
	/// hunt's space.
	///
	/// Answers what to tell the rift about where this reaper stands, or
	/// `None` when there is nothing to tell: the rift already has its place,
	/// the phone does not know where it is, or it was told a moment ago.
	pub fn observe(
		&mut self,
		frame: &RiftFrame,
		facing: f64,
		north: Option<f64>,
		now: Timestamp,
	) -> Option<GeoStanding> {
		let here = self.fix.filter(|fix| fix.is_usable_at(now));

		self.frames_waited = self.frames_waited.saturating_add(1);

		let (Some(here), Some(north)) = (here, north) else {
			self.sighting = None;

			return None;
		};

		self.see(frame.anchor, &here, north);

		let unplaced = frame.phase == RiftPhase::Gathering && frame.anchor.is_none();

		if !unplaced {
			return None;
		}

		self.frames_since_told = self.frames_since_told.saturating_add(1);

		if self.frames_since_told < Self::TELL_EVERY_FRAMES {
			return None;
		}

		self.frames_since_told = 0;

		Some(GeoStanding {
			at: here.at,
			facing: wrap_angle(facing - north),
			accuracy: here.accuracy,
			observed_at: Some(here.observed_at),
		})
	}

	fn see(&mut self, anchor: Option<GeoPoint>, here: &GeoFix, north: f64) {
		let seen = anchor.and_then(|anchor| RiftSighting::of(&anchor, &here.at, north));

		self.sighting = match (seen, self.sighting) {
			(Some(seen), Some(before)) => {
				Some(RiftSighting {
					bearing: wrap_angle(
						before.bearing + wrap_angle(seen.bearing - before.bearing) * Self::EASING,
					),
					metres: before.metres + (seen.metres - before.metres) * Self::EASING,
				})
			}
			(seen, _) => seen,
		};
	}
}
