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
	/// Compact delta + run-length encoding of the same combat inputs as
	/// versions 10..=12.
	const COMPACT_VERSION: u8 = 13;
	const EYE_TICK_BYTES: usize = 14;
	const EYE_VERSION: u8 = 12;
	const FIRE_FLAG: u8 = 1;
	const HEADER_BYTES: usize = 9;
	const LEDGE_BYTES: usize = HuntPlan::SLICES.div_ceil(8);
	/// The longest log the server accepts: the longest possible hunt, with
	/// room for the stalk, the fight, every broken window and every orb.
	pub const MAX_TICKS: usize = 360 * 60;
	const ORB_FLAG: u8 = 2;
	const PACKED_AIM_BASE: u8 = 0x80;
	const PACKED_AIM_EYE_BASE: u8 = 0x9B;
	const REPEAT_TAG: u8 = 0xFF;
	/// The bytes that say how long the room's grid is.
	const ROOM_LENGTH_BYTES: usize = 4;
	/// The same combat as version 10, with the room's grid after the plan.
	const ROOM_VERSION: u8 = 11;
	const TICK_BYTES: usize = 8;
	// Version 10 lets melee weapons swing at empty space. A missed swing
	// now spends its cooldown and breaks cadence, so versions 7, 8 and 9
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

	/// Reads a log from the bytes written by [`encode`](Self::encode) or
	/// [`encode_compact`](Self::encode_compact).
	///
	/// Fails when the bytes are not a whole log.
	pub fn decode(bytes: &[u8]) -> Result<Self, HuntLogError> {
		if bytes.is_empty() {
			return Err(HuntLogError::Empty);
		}

		if bytes.len() < Self::HEADER_BYTES {
			return Err(HuntLogError::NoPlan);
		}

		let version = bytes[0];
		if version != Self::VERSION
			&& version != Self::ROOM_VERSION
			&& version != Self::EYE_VERSION
			&& version != Self::COMPACT_VERSION
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

		let (room, room_end) = if version == Self::ROOM_VERSION
			|| version == Self::EYE_VERSION
			|| version == Self::COMPACT_VERSION
		{
			Self::decode_room(
				bytes,
				plan_end,
				version == Self::EYE_VERSION || version == Self::COMPACT_VERSION,
			)?
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

		if !plan.is_well_formed() {
			return Err(HuntLogError::BrokenPlan);
		}

		let inputs = if version == Self::COMPACT_VERSION {
			Self::decode_compact_inputs(bytes, offset, &plan)?
		} else {
			Self::decode_inputs(bytes, offset, &plan)?
		};

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
		let has_eye = bytes[0] == Self::EYE_VERSION;
		let tick_bytes = if has_eye {
			Self::EYE_TICK_BYTES
		} else {
			Self::TICK_BYTES
		};
		let remaining = bytes.len() - offset;
		let expected_ticks = remaining / tick_bytes;
		if expected_ticks > Self::MAX_TICKS {
			return Err(HuntLogError::TooLong);
		}
		if !remaining.is_multiple_of(tick_bytes) {
			return Err(HuntLogError::EndsMidTick);
		}

		let mut inputs = Vec::with_capacity(expected_ticks);

		while offset < bytes.len() {
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

			let eye = if has_eye {
				let read =
					|index| i16::from_le_bytes([bytes[offset + index], bytes[offset + index + 1]]);
				let eye = Eye::new(read(8), read(10), read(12));
				if !eye.is_within(plan) {
					return Err(HuntLogError::ImpossibleEye);
				}
				eye
			} else {
				Eye::ORIGIN
			};

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

	fn write_varuint(out: &mut Vec<u8>, mut value: u32) {
		while value >= 0x80 {
			out.push((value as u8) | 0x80);
			value >>= 7;
		}
		out.push(value as u8);
	}

	fn write_varint(out: &mut Vec<u8>, value: i32) {
		let zigzag = ((value << 1) ^ (value >> 31)) as u32;
		Self::write_varuint(out, zigzag);
	}

	fn read_varuint(bytes: &[u8], offset: &mut usize) -> Result<u32, HuntLogError> {
		let mut value = 0_u32;
		let mut shift = 0_u32;
		loop {
			let &byte = bytes.get(*offset).ok_or(HuntLogError::EndsMidTick)?;
			*offset += 1;
			if shift >= 32 || (shift == 28 && byte > 0x0F) {
				return Err(HuntLogError::UnknownInput);
			}
			value |= u32::from(byte & 0x7F) << shift;
			if byte & 0x80 == 0 {
				return Ok(value);
			}
			shift += 7;
		}
	}

	fn read_varint(bytes: &[u8], offset: &mut usize) -> Result<i32, HuntLogError> {
		let zigzag = Self::read_varuint(bytes, offset)?;
		Ok(((zigzag >> 1) as i32) ^ -((zigzag & 1) as i32))
	}

	#[allow(clippy::too_many_lines)]
	fn decode_compact_inputs(
		bytes: &[u8],
		mut offset: usize,
		plan: &HuntPlan,
	) -> Result<Vec<HuntInput>, HuntLogError> {
		let &stream_flags = bytes.get(offset).ok_or(HuntLogError::EndsMidTick)?;
		offset += 1;
		if stream_flags & !0x01 != 0 {
			return Err(HuntLogError::UnknownInput);
		}
		let has_eye = stream_flags & 0x01 != 0;

		let mut inputs = Vec::with_capacity(((bytes.len() - offset) / 2).min(Self::MAX_TICKS));
		let mut prev_yaw = 0_i32;
		let mut prev_pitch = 0_i32;
		let mut prev_turn = 0_i32;
		let mut prev_dyaw = 0_i32;
		let mut prev_dpitch = 0_i32;
		let mut prev_dturn = 0_i32;
		let mut prev_eye = Eye::ORIGIN;
		let mut prev_control = 0_u8;

		while offset < bytes.len() {
			let tag = bytes[offset];
			offset += 1;

			if tag == Self::REPEAT_TAG {
				let Some(last) = inputs.last().cloned() else {
					return Err(HuntLogError::UnknownInput);
				};
				let count = (Self::read_varuint(bytes, &mut offset)? as usize)
					.checked_add(1)
					.ok_or(HuntLogError::TooLong)?;
				if inputs.len().saturating_add(count) > Self::MAX_TICKS {
					return Err(HuntLogError::TooLong);
				}
				inputs.resize(inputs.len() + count, last);
				prev_dyaw = 0;
				prev_dpitch = 0;
				prev_dturn = 0;
				continue;
			}

			if inputs.len() >= Self::MAX_TICKS {
				return Err(HuntLogError::TooLong);
			}

			let (ddyaw, ddpitch, ddturn, dx, dy, dz, control) =
				if (Self::PACKED_AIM_BASE..Self::PACKED_AIM_EYE_BASE).contains(&tag) {
					let code = i32::from(tag - Self::PACKED_AIM_BASE);
					(
						code / 9 - 1,
						(code / 3) % 3 - 1,
						code % 3 - 1,
						0,
						0,
						0,
						prev_control,
					)
				} else if (Self::PACKED_AIM_EYE_BASE..Self::PACKED_AIM_EYE_BASE + 27).contains(&tag)
				{
					let code = i32::from(tag - Self::PACKED_AIM_EYE_BASE);
					let &aux = bytes.get(offset).ok_or(HuntLogError::EndsMidTick)?;
					offset += 1;
					if aux >= 54 {
						return Err(HuntLogError::UnknownInput);
					}
					let toggle_fire = aux >= 27;
					let eye_code = i32::from(if toggle_fire { aux - 27 } else { aux });
					if !has_eye && eye_code != 13 {
						return Err(HuntLogError::UnknownInput);
					}
					let control = if toggle_fire {
						prev_control ^ Self::FIRE_FLAG
					} else {
						prev_control
					};
					(
						code / 9 - 1,
						(code / 3) % 3 - 1,
						code % 3 - 1,
						eye_code / 9 - 1,
						(eye_code / 3) % 3 - 1,
						eye_code % 3 - 1,
						control,
					)
				} else if tag & 0x80 == 0 {
					if !has_eye && tag & 0x38 != 0 {
						return Err(HuntLogError::UnknownInput);
					}
					let ddyaw = if tag & 0x01 != 0 {
						Self::read_varint(bytes, &mut offset)?
					} else {
						0
					};
					let ddpitch = if tag & 0x02 != 0 {
						Self::read_varint(bytes, &mut offset)?
					} else {
						0
					};
					let ddturn = if tag & 0x04 != 0 {
						Self::read_varint(bytes, &mut offset)?
					} else {
						0
					};
					let dx = if tag & 0x08 != 0 {
						Self::read_varint(bytes, &mut offset)?
					} else {
						0
					};
					let dy = if tag & 0x10 != 0 {
						Self::read_varint(bytes, &mut offset)?
					} else {
						0
					};
					let dz = if tag & 0x20 != 0 {
						Self::read_varint(bytes, &mut offset)?
					} else {
						0
					};
					let control = if tag & 0x40 != 0 {
						let &byte = bytes.get(offset).ok_or(HuntLogError::EndsMidTick)?;
						offset += 1;
						byte
					} else {
						prev_control
					};
					(ddyaw, ddpitch, ddturn, dx, dy, dz, control)
				} else {
					return Err(HuntLogError::UnknownInput);
				};

			if control & !0x0F != 0 {
				return Err(HuntLogError::UnknownInput);
			}
			let slot = usize::from(control >> 2);
			if slot >= Arsenal::SLOTS {
				return Err(HuntLogError::UnknownInput);
			}

			let dyaw = prev_dyaw
				.checked_add(ddyaw)
				.ok_or(HuntLogError::ImpossibleAim)?;
			let dpitch = prev_dpitch
				.checked_add(ddpitch)
				.ok_or(HuntLogError::ImpossibleAim)?;
			let dturn = prev_dturn
				.checked_add(ddturn)
				.ok_or(HuntLogError::UnknownInput)?;
			let yaw = prev_yaw
				.checked_add(dyaw)
				.ok_or(HuntLogError::ImpossibleAim)?;
			let pitch = prev_pitch
				.checked_add(dpitch)
				.ok_or(HuntLogError::ImpossibleAim)?;
			let turn_speed = prev_turn
				.checked_add(dturn)
				.ok_or(HuntLogError::UnknownInput)?;

			let aim = Aim::new(yaw, pitch);
			if !aim.is_valid() {
				return Err(HuntLogError::ImpossibleAim);
			}
			if !(0..=HuntInput::MAX_TURN_SPEED).contains(&turn_speed) {
				return Err(HuntLogError::UnknownInput);
			}

			let eye = if has_eye {
				let x = i32::from(prev_eye.x_millimetres)
					.checked_add(dx)
					.and_then(|v| i16::try_from(v).ok())
					.ok_or(HuntLogError::ImpossibleEye)?;
				let y = i32::from(prev_eye.y_millimetres)
					.checked_add(dy)
					.and_then(|v| i16::try_from(v).ok())
					.ok_or(HuntLogError::ImpossibleEye)?;
				let z = i32::from(prev_eye.z_millimetres)
					.checked_add(dz)
					.and_then(|v| i16::try_from(v).ok())
					.ok_or(HuntLogError::ImpossibleEye)?;
				let eye = Eye::new(x, y, z);
				if !eye.is_within(plan) {
					return Err(HuntLogError::ImpossibleEye);
				}
				eye
			} else {
				Eye::ORIGIN
			};

			prev_yaw = yaw;
			prev_pitch = pitch;
			prev_turn = turn_speed;
			prev_dyaw = dyaw;
			prev_dpitch = dpitch;
			prev_dturn = dturn;
			prev_eye = eye;
			prev_control = control;

			inputs.push(HuntInput {
				eye,
				aim,
				turn_speed,
				fire: control & Self::FIRE_FLAG != 0,
				slot: slot as u8,
				orb_held: control & Self::ORB_FLAG != 0,
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

	fn write_header(
		&self,
		version: u8,
		write_room: bool,
		room: Option<&[u8]>,
		bytes: &mut Vec<u8>,
	) {
		let plan = &self.plan;
		bytes.push(version);
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

		if write_room {
			bytes.extend_from_slice(&(room.map_or(0, <[u8]>::len) as u32).to_le_bytes());
			if let Some(room) = room {
				bytes.extend_from_slice(room);
			}
		}
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
		let room = self.plan.room.as_ref().map(RoomGrid::encode);
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

		let version = if has_eye {
			Self::EYE_VERSION
		} else if room.is_some() {
			Self::ROOM_VERSION
		} else {
			Self::VERSION
		};
		self.write_header(
			version,
			has_eye || room.is_some(),
			room.as_deref(),
			&mut bytes,
		);

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

	/// Packs the log into version 13 compact bytes using second-order aim/turn
	/// deltas, first-order eye deltas, single-byte smooth-motion packing, and
	/// run-length encoding for repeated ticks. Decoded transparently by
	/// [`decode`](Self::decode).
	#[allow(clippy::too_many_lines)]
	pub fn encode_compact(&self) -> Vec<u8> {
		let room = self.plan.room.as_ref().map(RoomGrid::encode);
		let has_eye = self.inputs.iter().any(|input| !input.eye.is_origin());
		let mut bytes = Vec::with_capacity(
			Self::HEADER_BYTES + 400 + room.as_ref().map_or(0, Vec::len) + self.inputs.len() * 2,
		);
		self.write_header(Self::COMPACT_VERSION, true, room.as_deref(), &mut bytes);
		bytes.push(u8::from(has_eye));

		let mut prev_yaw = 0_i32;
		let mut prev_pitch = 0_i32;
		let mut prev_turn = 0_i32;
		let mut prev_dyaw = 0_i32;
		let mut prev_dpitch = 0_i32;
		let mut prev_dturn = 0_i32;
		let mut prev_eye = Eye::ORIGIN;
		let mut prev_control = 0_u8;
		let mut index = 0_usize;

		while index < self.inputs.len() {
			let input = &self.inputs[index];
			let yaw = i32::from(input.aim.yaw_centidegrees as i16);
			let pitch = i32::from(input.aim.pitch_centidegrees as i16);
			let turn = input.turn_speed.clamp(0, HuntInput::MAX_TURN_SPEED);
			let slot = usize::from(input.slot).min(Arsenal::SLOTS - 1) as u8;
			let control = (if input.fire { Self::FIRE_FLAG } else { 0 })
				| (if input.orb_held { Self::ORB_FLAG } else { 0 })
				| (slot << 2);
			let eye = if has_eye { input.eye } else { Eye::ORIGIN };

			if index > 0
				&& yaw == prev_yaw
				&& pitch == prev_pitch
				&& turn == prev_turn
				&& eye == prev_eye
				&& control == prev_control
			{
				let mut run = 1_u32;
				while index + (run as usize) < self.inputs.len() {
					let next = &self.inputs[index + (run as usize)];
					let next_yaw = i32::from(next.aim.yaw_centidegrees as i16);
					let next_pitch = i32::from(next.aim.pitch_centidegrees as i16);
					let next_turn = next.turn_speed.clamp(0, HuntInput::MAX_TURN_SPEED);
					let next_slot = usize::from(next.slot).min(Arsenal::SLOTS - 1) as u8;
					let next_control = (if next.fire { Self::FIRE_FLAG } else { 0 })
						| (if next.orb_held { Self::ORB_FLAG } else { 0 })
						| (next_slot << 2);
					let next_eye = if has_eye { next.eye } else { Eye::ORIGIN };
					if next_yaw == prev_yaw
						&& next_pitch == prev_pitch
						&& next_turn == prev_turn
						&& next_eye == prev_eye
						&& next_control == prev_control
					{
						run += 1;
					} else {
						break;
					}
				}
				bytes.push(Self::REPEAT_TAG);
				Self::write_varuint(&mut bytes, run - 1);
				prev_dyaw = 0;
				prev_dpitch = 0;
				prev_dturn = 0;
				index += run as usize;
				continue;
			}

			let dyaw = yaw - prev_yaw;
			let dpitch = pitch - prev_pitch;
			let dturn = turn - prev_turn;
			let ddyaw = dyaw - prev_dyaw;
			let ddpitch = dpitch - prev_dpitch;
			let ddturn = dturn - prev_dturn;
			let dx = i32::from(eye.x_millimetres) - i32::from(prev_eye.x_millimetres);
			let dy = i32::from(eye.y_millimetres) - i32::from(prev_eye.y_millimetres);
			let dz = i32::from(eye.z_millimetres) - i32::from(prev_eye.z_millimetres);

			let small_aim = (-1..=1).contains(&ddyaw)
				&& (-1..=1).contains(&ddpitch)
				&& (-1..=1).contains(&ddturn);
			let same_eye = dx == 0 && dy == 0 && dz == 0;
			let small_eye =
				(-1..=1).contains(&dx) && (-1..=1).contains(&dy) && (-1..=1).contains(&dz);

			if small_aim && same_eye && control == prev_control {
				let code = ((ddyaw + 1) * 9 + (ddpitch + 1) * 3 + (ddturn + 1)) as u8;
				bytes.push(Self::PACKED_AIM_BASE + code);
			} else if small_aim
				&& small_eye
				&& (control == prev_control || (control ^ prev_control) == Self::FIRE_FLAG)
			{
				let code = ((ddyaw + 1) * 9 + (ddpitch + 1) * 3 + (ddturn + 1)) as u8;
				let eye_code = ((dx + 1) * 9 + (dy + 1) * 3 + (dz + 1)) as u8;
				let aux = eye_code + if control == prev_control { 0 } else { 27 };
				bytes.push(Self::PACKED_AIM_EYE_BASE + code);
				bytes.push(aux);
			} else {
				let tag = u8::from(ddyaw != 0)
					| (u8::from(ddpitch != 0) << 1)
					| (u8::from(ddturn != 0) << 2)
					| (u8::from(dx != 0) << 3)
					| (u8::from(dy != 0) << 4)
					| (u8::from(dz != 0) << 5)
					| (u8::from(control != prev_control) << 6);
				bytes.push(tag);
				if ddyaw != 0 {
					Self::write_varint(&mut bytes, ddyaw);
				}
				if ddpitch != 0 {
					Self::write_varint(&mut bytes, ddpitch);
				}
				if ddturn != 0 {
					Self::write_varint(&mut bytes, ddturn);
				}
				if dx != 0 {
					Self::write_varint(&mut bytes, dx);
				}
				if dy != 0 {
					Self::write_varint(&mut bytes, dy);
				}
				if dz != 0 {
					Self::write_varint(&mut bytes, dz);
				}
				if control != prev_control {
					bytes.push(control);
				}
			}

			prev_yaw = yaw;
			prev_pitch = pitch;
			prev_turn = turn;
			prev_dyaw = dyaw;
			prev_dpitch = dpitch;
			prev_dturn = dturn;
			prev_eye = eye;
			prev_control = control;
			index += 1;
		}

		bytes
	}
}

#[cfg(test)]
mod compact_tests {
	use super::*;

	#[test]
	fn compact_round_trips_smooth_and_jumpy_logs() {
		let inputs = vec![
			HuntInput::new(Aim::new(120, -45), 300).with_eye(Eye::new(10, -5, 20)),
			HuntInput::new(Aim::new(130, -40), 301).with_eye(Eye::new(11, -5, 20)),
			HuntInput::new(Aim::new(141, -35), 302).with_eye(Eye::new(12, -4, 19)),
			HuntInput {
				fire: true,
				..HuntInput::new(Aim::new(152, -30), 303).with_eye(Eye::new(12, -4, 19))
			},
			HuntInput::new(Aim::new(152, -30), 303).with_eye(Eye::new(12, -4, 19)),
			HuntInput::new(Aim::new(152, -30), 303).with_eye(Eye::new(12, -4, 19)),
			HuntInput::new(Aim::new(152, -30), 303).with_eye(Eye::new(12, -4, 19)),
			HuntInput {
				fire: false,
				slot: 2,
				orb_held: true,
				..HuntInput::new(Aim::new(-17950, 8950), 65535).with_eye(Eye::new(-250, 40, 250))
			},
		];
		let log = HuntLog::new(inputs);
		let encoded = log.encode_compact();
		let decoded = HuntLog::decode(&encoded).expect("decode compact");
		assert_eq!(decoded, log);
	}

	#[test]
	fn compact_round_trips_empty_and_origin_eye() {
		let empty = HuntLog::new(Vec::new());
		assert_eq!(
			HuntLog::decode(&empty.encode_compact()).expect("empty"),
			empty
		);

		let origin = HuntLog::new(vec![
			HuntInput::new(Aim::AHEAD, 0),
			HuntInput::new(Aim::AHEAD, 0),
			HuntInput::new(Aim::new(1, -1), 1),
		]);
		assert_eq!(
			HuntLog::decode(&origin.encode_compact()).expect("origin"),
			origin
		);
	}

	#[test]
	fn compact_rejects_bad_inputs() {
		let log = HuntLog::new(vec![HuntInput::new(Aim::AHEAD, 0)]);
		let mut bad = log.encode_compact();
		// Truncate stream flags
		assert_eq!(
			HuntLog::decode(&bad[..bad.len() - 2]),
			Err(HuntLogError::EndsMidTick)
		);
		// Invalid repeat at tick 0
		let last = bad.len() - 1;
		bad[last] = HuntLog::REPEAT_TAG;
		bad.push(0);
		assert_eq!(HuntLog::decode(&bad), Err(HuntLogError::UnknownInput));
	}
}
