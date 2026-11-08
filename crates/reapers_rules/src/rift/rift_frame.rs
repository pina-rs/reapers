//! What a rift tells its reapers, and what they tell it.

use serde::Deserialize;
use serde::Serialize;

use crate::geo::GeoPoint;
use crate::geo::GeoStanding;
use crate::hunt::hunt_event::HuntOutcome;
use crate::hunt::hunt_event::ReapGrade;
use crate::named::named_enum;
use crate::reaper_class::ReaperClass;
use crate::rift::rift_site::RiftSite;

/// Where a rift is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RiftPhase {
	/// Reapers are arriving and standing ready.
	#[default]
	Gathering,

	/// The circle is breaking the ward.
	Fight,

	/// The ward is broken and the circle is cutting.
	Reap,

	/// Enough of the circle cut. The rift is shut and everyone is paid.
	Sealed,

	/// The circle ran out of time, of reapers, or of cuts.
	Failed,
}

named_enum!(RiftPhase {
	Gathering => "gathering",
	Fight => "fight",
	Reap => "reap",
	Sealed => "sealed",
	Failed => "failed",
});

impl RiftPhase {
	pub const fn is_over(self) -> bool {
		matches!(self, Self::Sealed | Self::Failed)
	}
}

/// One member of a circle, as everyone in it sees them.
#[allow(clippy::struct_excessive_bools, reason = "the wire shape of a member")]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiftMember {
	pub name: String,

	#[serde(rename = "class")]
	pub reaper_class: ReaperClass,

	pub level: i64,

	/// Whether this seat is filled by a shade rather than a person.
	pub shade: bool,

	/// Whether they have said they are ready to begin.
	pub ready: bool,

	/// Whether they are still in the fight: present, and not cast out.
	pub standing: bool,

	/// Ward this member has removed so far.
	pub dealt: f64,

	/// Whether their part is over: cast out, or done with the reap.
	pub finished: bool,

	/// How cleanly they cut, or `None` if they have not, or could not.
	#[serde(default)]
	pub cut: Option<ReapGrade>,
}

/// A rift as one of its reapers sees it at one moment.
///
/// Whoever runs the rift sends these several times a second. Each one is the
/// whole picture, so a frame that never arrives is simply replaced by the
/// next.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiftFrame {
	pub site: RiftSite,

	/// The seed every reaper's own fight is built from.
	pub seed: i64,

	pub phase: RiftPhase,

	/// Which of `members` is the reaper this frame was made for, or -1 for
	/// one who has left.
	pub seat: i64,

	/// Ticks since the fight began. Zero while the circle gathers.
	pub tick: i64,

	/// Ticks until the current phase runs out.
	pub ticks_left: i64,

	/// Whether the circle is ready and the count before the fight has
	/// begun.
	pub counting_down: bool,

	/// What is left of the ward the whole circle is breaking.
	pub ward: f64,

	pub ward_max: f64,

	/// The circle, in seat order.
	pub members: Vec<RiftMember>,

	/// Experience this reaper was paid. Zero until the rift is sealed.
	pub xp: i64,

	/// Anima this reaper was paid. Zero until the rift is sealed.
	pub anima: i64,

	/// Where on the earth the rift stands, the same for everyone in the
	/// circle, or `None` while no reaper in it has said where they are.
	#[serde(default)]
	pub anchor: Option<GeoPoint>,
}

impl RiftFrame {
	/// The member this frame was made for, or `None` for one who has left.
	pub fn you(&self) -> Option<&RiftMember> {
		usize::try_from(self.seat)
			.ok()
			.and_then(|seat| self.members.get(seat))
	}

	/// Cuts the circle needs before the rift counts as sealed: half of those
	/// still standing, rounded up.
	pub fn cuts_needed(&self) -> usize {
		Self::cuts_needed_for(self.members.iter().filter(|member| member.standing).count())
	}

	/// Cuts the circle has made so far.
	pub fn cuts(&self) -> usize {
		self.members
			.iter()
			.filter(|member| member.cut.is_some())
			.count()
	}

	/// Cuts a circle with `standing` reapers left needs.
	pub const fn cuts_needed_for(standing: usize) -> usize {
		if standing <= 1 {
			1
		} else {
			standing.div_ceil(2)
		}
	}
}

/// What one reaper tells the rift about their own fight.
///
/// A report is the whole of that reaper's standing, not a change to it: the
/// damage is a running total and the outcome, once set, stays set. Reports
/// can therefore be repeated, dropped or arrive late without the rift losing
/// count.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RiftReport {
	/// Whether the reaper stands ready to begin.
	pub ready: bool,

	/// Ward this reaper has removed since the fight began.
	pub dealt: f64,

	/// How this reaper's part ended, or `None` while it is still going. A
	/// reaper who cut the husk reports an absorbed outcome with the grade of
	/// the cut.
	#[serde(default)]
	pub outcome: Option<HuntOutcome>,

	/// Where this reaper is standing, when their phone can tell. The first
	/// one a rift hears of decides where the rift stands.
	#[serde(default)]
	pub standing: Option<GeoStanding>,
}

impl RiftReport {
	/// A report that says only whether the reaper is ready.
	pub const fn readiness(ready: bool) -> Self {
		Self {
			ready,
			dealt: 0.0,
			outcome: None,
			standing: None,
		}
	}
}
