//! What the player did, tick by tick, and the log a hunt is judged from.

use core::fmt;

use thiserror::Error;

use crate::hunt::difficulty::Difficulty;
use crate::hunt::hunt_plan::HuntPlan;
use crate::math;
use crate::vec3::Vec3;
use crate::vec3::degrees;
use crate::vec3::radians;

/// A direction stored as whole hundredths of a degree.
///
/// The simulation only ever sees directions in this form. Whole numbers are
/// what make a hunt replay exactly: the recorded log holds the very values
/// the phone played with, with nothing lost to rounding in between.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Aim {
	/// Hundredths of a degree to the right of straight ahead, in
	/// `[-18000, 18000]`.
	pub yaw_centidegrees: i32,

	/// Hundredths of a degree above the horizon, in `[-9000, 9000]`.
	pub pitch_centidegrees: i32,
}

impl Aim {
	pub const AHEAD: Self = Self::new(0, 0);

	pub const fn new(yaw_centidegrees: i32, pitch_centidegrees: i32) -> Self {
		Self {
			yaw_centidegrees,
			pitch_centidegrees,
		}
	}

	/// Rounds a direction onto the hundredth-of-a-degree grid.
	pub fn from_direction(direction: Vec3) -> Self {
		Self::new(
			math::round(degrees(direction.yaw()) * 100.0) as i32,
			math::round(degrees(direction.pitch()) * 100.0).clamp(-9000, 9000) as i32,
		)
	}

	pub fn direction(&self) -> Vec3 {
		Vec3::from_angles(
			radians(f64::from(self.yaw_centidegrees) / 100.0),
			radians(f64::from(self.pitch_centidegrees) / 100.0),
		)
	}

	/// Whether this is a direction a phone could point.
	pub const fn is_valid(&self) -> bool {
		self.yaw_centidegrees >= -18000
			&& self.yaw_centidegrees <= 18000
			&& self.pitch_centidegrees >= -9000
			&& self.pitch_centidegrees <= 9000
	}
}

impl fmt::Display for Aim {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			formatter,
			"Aim({}, {})",
			self.yaw_centidegrees, self.pitch_centidegrees
		)
	}
}

/// Everything the player did during one tick.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct HuntInput {
	/// Where the middle of the phone's view points.
	pub aim: Aim,

	/// How fast the phone is turning, in thousandths of a radian a second.
	pub turn_speed: i32,

	/// Where each tap this tick points, as a direction out from the player.
	pub taps: Vec<Aim>,

	/// Whether the reaper is holding their art ready: a thumb on the art, to
	/// gather it and throw it with a flick of the phone.
	pub focus: bool,
}

impl HuntInput {
	/// The speed, in thousandths of a radian a second, above which a
	/// recorded turn speed is clamped.
	pub const MAX_TURN_SPEED: i32 = 0xFFFF;

	/// Aiming along `aim`, turning at `turn_speed`, and doing nothing else.
	pub const fn new(aim: Aim, turn_speed: i32) -> Self {
		Self {
			aim,
			turn_speed,
			taps: Vec::new(),
			focus: false,
		}
	}

	/// Rounds what the phone measured this tick onto the grid the rules use.
	pub fn measured(aim: Vec3, turn_radians_per_second: f64, taps: &[Vec3], focus: bool) -> Self {
		Self {
			aim: Aim::from_direction(aim),
			turn_speed: math::round(turn_radians_per_second * 1000.0)
				.clamp(0, i64::from(Self::MAX_TURN_SPEED)) as i32,
			taps: taps.iter().map(|&tap| Aim::from_direction(tap)).collect(),
			focus,
		}
	}

	/// Rounds a tick with no taps and no art held.
	pub fn still(aim: Vec3, turn_radians_per_second: f64) -> Self {
		Self::measured(aim, turn_radians_per_second, &[], false)
	}

	/// `turn_speed` in radians a second.
	pub fn turn_radians_per_second(&self) -> f64 {
		f64::from(self.turn_speed) / 1000.0
	}
}

/// Why a hunt log could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum HuntLogError {
	/// The encoded log belongs to a different combat-rules deployment.
	#[error("Unsupported combat rules")]
	UnsupportedVersion,

	#[error("Not a hunt log")]
	Empty,

	#[error("Hunt log has no plan")]
	NoPlan,

	#[error("Hunt log has no such difficulty")]
	NoSuchDifficulty,

	#[error("Hunt log has a broken plan")]
	BrokenPlan,

	#[error("Hunt log has broken cover")]
	BrokenCover,

	#[error("Hunt log ends mid-plan")]
	EndsMidPlan,

	#[error("Hunt log exceeds the encounter limit")]
	TooLong,

	#[error("Hunt log ends mid-tick")]
	EndsMidTick,

	#[error("Hunt log has unknown input")]
	UnknownInput,

	#[error("Impossible hunt aim")]
	ImpossibleAim,

	#[error("Hunt log ends mid-tap")]
	EndsMidTap,

	#[error("Impossible tap aim")]
	ImpossibleTap,
}

/// The complete input of one hunt, tick by tick.
///
/// A phone records the log as it plays and sends it with the hunt's result.
/// The server feeds the same log to the same rules and so reaches its own
/// verdict; it never takes the phone's word for what happened.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct HuntLog {
	/// One entry for every tick the hunt ran.
	pub inputs: Vec<HuntInput>,

	/// Where the husk was free to go. It is part of the record because the
	/// husk's every move depends on it.
	pub plan: HuntPlan,

	/// How forgiving the hunt was. Part of the record for the same reason.
	pub difficulty: Difficulty,
}

impl HuntLog {
	const FOCUS_FLAG: u8 = 1;
	const HEADER_BYTES: usize = 9;
	const LEDGE_BYTES: usize = HuntPlan::SLICES.div_ceil(8);
	/// The longest log the server accepts: the longest possible hunt, with
	/// room for the stalk, the fight and every reap window.
	pub const MAX_TICKS: usize = 300 * 60;
	const TAP_BYTES: usize = 4;
	const TICK_BYTES: usize = 8;
	// Cover in the room, arts and the heavy weapons change replay results.
	// Refuse old clients rather than silently reinterpret a version-five
	// hunt under the new rules.
	const VERSION: u8 = 6;

	/// `inputs` played in the open with nothing known about the room, at
	/// normal difficulty.
	pub fn new(inputs: Vec<HuntInput>) -> Self {
		Self {
			inputs,
			plan: HuntPlan::OPEN,
			difficulty: Difficulty::Normal,
		}
	}

	/// Reads a log from the bytes written by [`encode`](Self::encode).
	///
	/// Fails when the bytes are not a whole log.
	#[allow(
		clippy::too_many_lines,
		reason = "one pass over the bytes, in their order"
	)]
	pub fn decode(bytes: &[u8]) -> Result<Self, HuntLogError> {
		if bytes.is_empty() {
			return Err(HuntLogError::Empty);
		}

		if bytes.len() < Self::HEADER_BYTES {
			return Err(HuntLogError::NoPlan);
		}

		if bytes[0] != Self::VERSION {
			return Err(HuntLogError::UnsupportedVersion);
		}

		let reach_count = usize::from(bytes[6]);
		let difficulty = usize::from(bytes[7]);
		let cover_count = usize::from(bytes[8]);

		let Some(&difficulty) = Difficulty::ALL.get(difficulty) else {
			return Err(HuntLogError::NoSuchDifficulty);
		};

		if reach_count != 0 && reach_count != HuntPlan::SLICES {
			return Err(HuntLogError::BrokenPlan);
		}

		if cover_count != 0 && cover_count != HuntPlan::SLICES {
			return Err(HuntLogError::BrokenCover);
		}

		let cover_bytes = if cover_count == 0 {
			0
		} else {
			2 * cover_count + Self::LEDGE_BYTES
		};
		let plan_end = Self::HEADER_BYTES + reach_count + cover_bytes;

		if bytes.len() < plan_end {
			return Err(HuntLogError::EndsMidPlan);
		}

		let widen = |slice: &[u8]| {
			slice
				.iter()
				.map(|&byte| i32::from(byte))
				.collect::<Vec<_>>()
		};
		let mut offset = Self::HEADER_BYTES + reach_count;
		let cover = widen(&bytes[offset..offset + cover_count]);
		offset += cover_count;
		let hide = widen(&bytes[offset..offset + cover_count]);
		offset += cover_count;
		let ledges = if cover_count == 0 {
			Vec::new()
		} else {
			(0..HuntPlan::SLICES)
				.map(|slice| bytes[offset + slice / 8] & (1 << (slice % 8)) != 0)
				.collect()
		};

		let plan = HuntPlan {
			centre_centidegrees: i32::from(i16::from_le_bytes([bytes[1], bytes[2]])),
			half_width_centidegrees: i32::from(u16::from_le_bytes([bytes[3], bytes[4]])),
			eye_centimetres: i32::from(bytes[5]),
			reach_decimetres: widen(&bytes[Self::HEADER_BYTES..Self::HEADER_BYTES + reach_count]),
			cover_centimetres: cover,
			hide_decimetres: hide,
			ledges,
		};
		let read_aim = |at: usize| {
			Aim::new(
				i32::from(i16::from_le_bytes([bytes[at], bytes[at + 1]])),
				i32::from(i16::from_le_bytes([bytes[at + 2], bytes[at + 3]])),
			)
		};
		let mut inputs = Vec::new();

		offset = plan_end;

		while offset < bytes.len() {
			if inputs.len() >= Self::MAX_TICKS {
				return Err(HuntLogError::TooLong);
			}

			if offset + Self::TICK_BYTES > bytes.len() {
				return Err(HuntLogError::EndsMidTick);
			}

			let aim = read_aim(offset);
			let turn_speed = i32::from(u16::from_le_bytes([bytes[offset + 4], bytes[offset + 5]]));
			let flags = bytes[offset + 6];
			let tap_count = usize::from(bytes[offset + 7]);

			if flags & !Self::FOCUS_FLAG != 0 {
				return Err(HuntLogError::UnknownInput);
			}

			if !aim.is_valid() {
				return Err(HuntLogError::ImpossibleAim);
			}

			offset += Self::TICK_BYTES;

			if offset + tap_count * Self::TAP_BYTES > bytes.len() {
				return Err(HuntLogError::EndsMidTap);
			}

			let taps: Vec<Aim> = (0..tap_count)
				.map(|tap| read_aim(offset + tap * Self::TAP_BYTES))
				.collect();

			if !taps.iter().all(Aim::is_valid) {
				return Err(HuntLogError::ImpossibleTap);
			}

			offset += tap_count * Self::TAP_BYTES;

			inputs.push(HuntInput {
				aim,
				turn_speed,
				taps,
				focus: flags & Self::FOCUS_FLAG != 0,
			});
		}

		Ok(Self {
			inputs,
			plan,
			difficulty,
		})
	}

	/// Packs the log into bytes: the plan first, then eight for each tick
	/// and four more for each tap.
	///
	/// The plan is its arc, the height of the player's eye, how far the way
	/// is clear along each slice of the room when that was measured, and
	/// what stands at the end of each: its height, where to hide behind it,
	/// and one bit for whether its top is a ledge. Numbers too big for their
	/// field keep their low bits, as Dart's byte writer did.
	pub fn encode(&self) -> Vec<u8> {
		let plan = &self.plan;
		let mut bytes =
			Vec::with_capacity(Self::HEADER_BYTES + 400 + self.inputs.len() * Self::TICK_BYTES);

		bytes.push(Self::VERSION);
		bytes.extend_from_slice(&(plan.centre_centidegrees as i16).to_le_bytes());
		bytes.extend_from_slice(&(plan.half_width_centidegrees as u16).to_le_bytes());
		bytes.push(plan.eye_centimetres as u8);
		bytes.push(plan.reach_decimetres.len() as u8);
		bytes.push(self.difficulty.index() as u8);
		bytes.push(plan.cover_centimetres.len() as u8);
		bytes.extend(plan.reach_decimetres.iter().map(|&value| value as u8));
		bytes.extend(plan.cover_centimetres.iter().map(|&value| value as u8));
		bytes.extend(plan.hide_decimetres.iter().map(|&value| value as u8));

		if plan.knows_cover() {
			let mut ledges = [0_u8; Self::LEDGE_BYTES];

			for slice in 0..HuntPlan::SLICES {
				if plan.ledges.get(slice).copied().unwrap_or(false) {
					ledges[slice / 8] |= 1 << (slice % 8);
				}
			}

			bytes.extend_from_slice(&ledges);
		}

		for input in &self.inputs {
			let taps = &input.taps[..input.taps.len().min(255)];

			bytes.extend_from_slice(&(input.aim.yaw_centidegrees as i16).to_le_bytes());
			bytes.extend_from_slice(&(input.aim.pitch_centidegrees as i16).to_le_bytes());
			bytes.extend_from_slice(
				&(input.turn_speed.clamp(0, HuntInput::MAX_TURN_SPEED) as u16).to_le_bytes(),
			);
			bytes.push(if input.focus { Self::FOCUS_FLAG } else { 0 });
			bytes.push(taps.len() as u8);

			for tap in taps {
				bytes.extend_from_slice(&(tap.yaw_centidegrees as i16).to_le_bytes());
				bytes.extend_from_slice(&(tap.pitch_centidegrees as i16).to_le_bytes());
			}
		}

		bytes
	}
}
