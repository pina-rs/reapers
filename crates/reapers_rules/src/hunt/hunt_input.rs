//! What the player did, tick by tick, and the log a hunt is judged from.

use core::fmt;

use thiserror::Error;

use crate::hunt::arsenal::Arsenal;
use crate::hunt::difficulty::Difficulty;
use crate::hunt::hunt_plan::HuntPlan;
use crate::hunt::hunt_stats::HuntRules;
use crate::math;
use crate::room::RoomGrid;
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

/// The phone's offset from the hunt origin, in whole millimetres.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Eye {
	/// Displacement to the right.
	pub x_millimetres: i16,
	/// Displacement above the original eye.
	pub y_millimetres: i16,
	/// Displacement ahead.
	pub z_millimetres: i16,
}

impl Eye {
	/// The eye at the hunt's origin.
	pub const ORIGIN: Self = Self::new(0, 0, 0);

	/// Constructs a wire-representable eye, without applying stance bounds.
	pub const fn new(x_millimetres: i16, y_millimetres: i16, z_millimetres: i16) -> Self {
		Self {
			x_millimetres,
			y_millimetres,
			z_millimetres,
		}
	}

	/// Quantises a finite offset, rejecting values outside the wire range.
	pub fn rounded(offset: Vec3) -> Option<Self> {
		fn component(value: f64) -> Option<i16> {
			if !value.is_finite() {
				return None;
			}
			let rounded = math::round(value * 1000.0);
			i16::try_from(rounded).ok()
		}
		Some(Self::new(
			component(offset.x)?,
			component(offset.y)?,
			component(offset.z)?,
		))
	}

	/// Whether the eye lies within the provisional stance and height limits.
	pub fn is_within(&self, plan: &HuntPlan) -> bool {
		let x = i64::from(self.x_millimetres);
		let z = i64::from(self.z_millimetres);
		let radius = i64::from(HuntRules::STANCE_MILLIMETRES);
		let height = i64::from(plan.eye_centimetres) * 10 + i64::from(self.y_millimetres);
		x * x + z * z <= radius * radius
			&& height >= i64::from(HuntPlan::MIN_EYE_CENTIMETRES) * 10
			&& height <= i64::from(HuntPlan::MAX_EYE_CENTIMETRES) * 10
	}

	/// The offset in metres, in hunt coordinates.
	pub fn point(&self) -> Vec3 {
		Vec3::new(
			f64::from(self.x_millimetres) / 1000.0,
			f64::from(self.y_millimetres) / 1000.0,
			f64::from(self.z_millimetres) / 1000.0,
		)
	}

	/// Whether the eye is exactly at the hunt origin.
	pub const fn is_origin(&self) -> bool {
		self.x_millimetres == 0 && self.y_millimetres == 0 && self.z_millimetres == 0
	}
}

/// Everything the player did during one tick.
///
/// The crosshair is the middle of the screen, so [`aim`](HuntInput::aim) is
/// where every weapon and every orb goes. Nothing the player touches says
/// where to strike.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct HuntInput {
	/// Where the phone was relative to the hunt origin.
	pub eye: Eye,

	/// Where the middle of the phone's view points: the crosshair.
	pub aim: Aim,

	/// How fast the phone is turning, in thousandths of a radian a second.
	pub turn_speed: i32,

	/// Whether FIRE was pressed this tick. Holding it down does not fire
	/// again: each press is one.
	pub fire: bool,

	/// The weapon slot selected, from 0 to [`Arsenal::SLOTS`] - 1.
	pub slot: u8,

	/// Whether ORB is held down. The tick it comes up is the throw.
	pub orb_held: bool,
}

impl HuntInput {
	/// The speed, in thousandths of a radian a second, above which a
	/// recorded turn speed is clamped.
	pub const MAX_TURN_SPEED: i32 = 0xFFFF;

	/// Aiming along `aim`, turning at `turn_speed`, and doing nothing else.
	pub const fn new(aim: Aim, turn_speed: i32) -> Self {
		Self {
			eye: Eye::ORIGIN,
			aim,
			turn_speed,
			fire: false,
			slot: Arsenal::GUN_SLOT as u8,
			orb_held: false,
		}
	}

	/// Rounds what the phone measured this tick onto the grid the rules use.
	pub fn measured(
		aim: Vec3,
		turn_radians_per_second: f64,
		fire: bool,
		slot: usize,
		orb_held: bool,
	) -> Self {
		Self {
			eye: Eye::ORIGIN,
			aim: Aim::from_direction(aim),
			turn_speed: math::round(turn_radians_per_second * 1000.0)
				.clamp(0, i64::from(Self::MAX_TURN_SPEED)) as i32,
			fire,
			slot: (slot % Arsenal::SLOTS) as u8,
			orb_held,
		}
	}

	/// Rounds a tick with nothing pressed and ORB up.
	pub fn still(aim: Vec3, turn_radians_per_second: f64) -> Self {
		Self::measured(
			aim,
			turn_radians_per_second,
			false,
			Arsenal::GUN_SLOT,
			false,
		)
	}

	/// Records the eye used for this tick.
	#[must_use]
	pub const fn with_eye(mut self, eye: Eye) -> Self {
		self.eye = eye;
		self
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

	#[error("Hunt log has a broken room")]
	BrokenRoom,

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

	/// The recorded eye is outside the stance or height limits.
	#[error("Impossible hunt eye")]
	ImpossibleEye,
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
	const EYE_TICK_BYTES: usize = 14;
	const EYE_VERSION: u8 = 12;
	const FIRE_FLAG: u8 = 1;
	const HEADER_BYTES: usize = 9;
	const LEDGE_BYTES: usize = HuntPlan::SLICES.div_ceil(8);
	/// The longest log the server accepts: the longest possible hunt, with
	/// room for the stalk, the fight, every broken window and every orb.
	pub const MAX_TICKS: usize = 360 * 60;
	const ORB_FLAG: u8 = 2;
	/// The bytes that say how long the room's grid is.
	const ROOM_LENGTH_BYTES: usize = 4;
	/// The same combat as version 9, with the room's grid after the plan.
	const ROOM_VERSION: u8 = 11;
	const TICK_BYTES: usize = 8;
	// Version 9 lets melee weapons swing at empty space. A missed swing
	// now spends its cooldown and breaks cadence, so versions 7 and 8
	// cannot be replayed under these rules.
	const VERSION: u8 = 10;

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
	pub fn decode(bytes: &[u8]) -> Result<Self, HuntLogError> {
		if bytes.is_empty() {
			return Err(HuntLogError::Empty);
		}

		if bytes.len() < Self::HEADER_BYTES {
			return Err(HuntLogError::NoPlan);
		}

		if bytes[0] != Self::VERSION
			&& bytes[0] != Self::ROOM_VERSION
			&& bytes[0] != Self::EYE_VERSION
		{
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

		let (room, room_end) = if bytes[0] == Self::ROOM_VERSION || bytes[0] == Self::EYE_VERSION {
			Self::decode_room(bytes, plan_end, bytes[0] == Self::EYE_VERSION)?
		} else {
			(None, plan_end)
		};

		offset = room_end;

		let plan = HuntPlan {
			centre_centidegrees: i32::from(i16::from_le_bytes([bytes[1], bytes[2]])),
			half_width_centidegrees: i32::from(u16::from_le_bytes([bytes[3], bytes[4]])),
			eye_centimetres: i32::from(bytes[5]),
			reach_decimetres: widen(&bytes[Self::HEADER_BYTES..Self::HEADER_BYTES + reach_count]),
			cover_centimetres: cover,
			hide_decimetres: hide,
			ledges,
			room,
		};
		let inputs = Self::decode_inputs(bytes, offset, &plan)?;

		Ok(Self {
			inputs,
			plan,
			difficulty,
		})
	}

	fn decode_inputs(
		bytes: &[u8],
		mut offset: usize,
		plan: &HuntPlan,
	) -> Result<Vec<HuntInput>, HuntLogError> {
		let mut inputs = Vec::new();

		let tick_bytes = if bytes[0] == Self::EYE_VERSION {
			Self::EYE_TICK_BYTES
		} else {
			Self::TICK_BYTES
		};

		while offset < bytes.len() {
			if inputs.len() >= Self::MAX_TICKS {
				return Err(HuntLogError::TooLong);
			}

			if offset + tick_bytes > bytes.len() {
				return Err(HuntLogError::EndsMidTick);
			}

			let aim = Aim::new(
				i32::from(i16::from_le_bytes([bytes[offset], bytes[offset + 1]])),
				i32::from(i16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]])),
			);
			let turn_speed = i32::from(u16::from_le_bytes([bytes[offset + 4], bytes[offset + 5]]));
			let flags = bytes[offset + 6];
			let slot = usize::from(bytes[offset + 7]);

			if flags & !(Self::FIRE_FLAG | Self::ORB_FLAG) != 0 || slot >= Arsenal::SLOTS {
				return Err(HuntLogError::UnknownInput);
			}

			if !aim.is_valid() {
				return Err(HuntLogError::ImpossibleAim);
			}

			let eye = if bytes[0] == Self::EYE_VERSION {
				let read =
					|index| i16::from_le_bytes([bytes[offset + index], bytes[offset + index + 1]]);
				Eye::new(read(8), read(10), read(12))
			} else {
				Eye::ORIGIN
			};

			if bytes[0] == Self::EYE_VERSION && !eye.is_within(plan) {
				return Err(HuntLogError::ImpossibleEye);
			}

			offset += tick_bytes;

			inputs.push(HuntInput {
				eye,
				aim,
				turn_speed,
				fire: flags & Self::FIRE_FLAG != 0,
				slot: slot as u8,
				orb_held: flags & Self::ORB_FLAG != 0,
			});
		}

		Ok(inputs)
	}

	/// Reads the room's grid that starts at `offset`: its length in four
	/// bytes, then the grid as [`RoomGrid::encode`] writes it. Returns the
	/// grid and where the bytes after it begin.
	fn decode_room(
		bytes: &[u8],
		offset: usize,
		empty_allowed: bool,
	) -> Result<(Option<RoomGrid>, usize), HuntLogError> {
		let start = offset + Self::ROOM_LENGTH_BYTES;

		if bytes.len() < start {
			return Err(HuntLogError::EndsMidPlan);
		}

		let length = u32::from_le_bytes([
			bytes[offset],
			bytes[offset + 1],
			bytes[offset + 2],
			bytes[offset + 3],
		]) as usize;
		let Some(end) = start.checked_add(length).filter(|&end| end <= bytes.len()) else {
			return Err(HuntLogError::EndsMidPlan);
		};
		if length == 0 && empty_allowed {
			return Ok((None, end));
		}

		let room = RoomGrid::decode(&bytes[start..end]).map_err(|_| HuntLogError::BrokenRoom)?;

		Ok((Some(room), end))
	}

	/// Packs the log into bytes: the plan first, then eight or fourteen bytes per tick:
	/// the aim, the turn speed, FIRE and ORB, and the slot selected.
	///
	/// The plan is its arc, the height of the player's eye, how far the way
	/// is clear along each slice of the room when that was measured, and
	/// what stands at the end of each: its height, where to hide behind it,
	/// and one bit for whether its top is a ledge. Numbers too big for their
	/// field keep their low bits, as they always have.
	///
	/// A plan with a room's grid is written as version 11, with the grid's
	/// length and the grid itself after the rest of the plan. One without
	/// is version 10. A non-origin eye selects version 12, which adds a
	/// room length (zero without a grid) and three i16 millimetre offsets
	/// per tick. Earlier combat versions are refused.
	pub fn encode(&self) -> Vec<u8> {
		let plan = &self.plan;
		let room = plan.room.as_ref().map(RoomGrid::encode);
		let has_eye = self.inputs.iter().any(|input| !input.eye.is_origin());
		let mut bytes = Vec::with_capacity(
			Self::HEADER_BYTES
				+ 400
				+ room.as_ref().map_or(0, Vec::len)
				+ self.inputs.len()
					* if has_eye {
						Self::EYE_TICK_BYTES
					} else {
						Self::TICK_BYTES
					},
		);

		bytes.push(if has_eye {
			Self::EYE_VERSION
		} else if room.is_some() {
			Self::ROOM_VERSION
		} else {
			Self::VERSION
		});
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

		if has_eye || room.is_some() {
			bytes.extend_from_slice(&(room.as_ref().map_or(0, Vec::len) as u32).to_le_bytes());
			if let Some(room) = room {
				bytes.extend_from_slice(&room);
			}
		}

		for input in &self.inputs {
			bytes.extend_from_slice(&(input.aim.yaw_centidegrees as i16).to_le_bytes());
			bytes.extend_from_slice(&(input.aim.pitch_centidegrees as i16).to_le_bytes());
			bytes.extend_from_slice(
				&(input.turn_speed.clamp(0, HuntInput::MAX_TURN_SPEED) as u16).to_le_bytes(),
			);
			bytes.push(
				(if input.fire { Self::FIRE_FLAG } else { 0 })
					| (if input.orb_held { Self::ORB_FLAG } else { 0 }),
			);
			bytes.push(usize::from(input.slot).min(Arsenal::SLOTS - 1) as u8);
			if has_eye {
				for coordinate in [
					input.eye.x_millimetres,
					input.eye.y_millimetres,
					input.eye.z_millimetres,
				] {
					bytes.extend_from_slice(&coordinate.to_le_bytes());
				}
			}
		}

		bytes
	}
}
