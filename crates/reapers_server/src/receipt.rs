//! The absorption receipt the keeper signs and the Reapers program checks.
//!
//! TODO(integration): replace with `reapers_receipt`, which the program
//! shares. This module implements the same version-two layout, byte for
//! byte, so the swap changes no signature; the test vector below is the
//! check that it does not.
//!
//! | Offset | Bytes | Field                                     |
//! | ------ | ----- | ----------------------------------------- |
//! | 0      | 8     | the tag, `RPRSABS2`                       |
//! | 8      | 32    | the player: their wallet's address        |
//! | 40     | 32    | the husk's mint                           |
//! | 72     | 1     | the reap grade, as its index              |
//! | 73     | 1     | the feeding, as its index                 |
//! | 74     | 4     | experience paid, little-endian            |
//! | 78     | 4     | anima paid, little-endian                 |
//! | 82     | 8     | when, in whole seconds since 1970, signed |
//! | 90     | 8     | the nonce the feeding was rolled from     |

use reapers_rules::Absorption;

/// What every version-two receipt starts with.
pub const TAG: &[u8; 8] = b"RPRSABS2";

/// How long a receipt is.
pub const LEN: usize = 98;

/// One absorption, as the receipt states it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipt {
	pub player: [u8; 32],
	pub mint: [u8; 32],
	pub grade: u8,
	pub feeding: u8,
	pub xp: u32,
	pub anima: u32,
	pub at: i64,
	pub nonce: u64,
}

impl Receipt {
	/// The receipt for `absorption`, made out to `player` for `mint`, or
	/// `None` when a figure does not fit its field, which no absorption the
	/// rules pay can do.
	pub fn of(player: [u8; 32], mint: [u8; 32], absorption: &Absorption) -> Option<Self> {
		Some(Self {
			player,
			mint,
			grade: u8::try_from(absorption.grade.index()).ok()?,
			feeding: u8::try_from(absorption.feeding.index()).ok()?,
			xp: u32::try_from(absorption.xp).ok()?,
			anima: u32::try_from(absorption.anima).ok()?,
			at: absorption.at.unix_seconds(),
			nonce: u64::try_from(absorption.nonce).ok()?,
		})
	}

	/// The receipt's bytes, which is what gets signed.
	pub fn encode(&self) -> [u8; LEN] {
		let mut bytes = [0; LEN];

		bytes[0..8].copy_from_slice(TAG);
		bytes[8..40].copy_from_slice(&self.player);
		bytes[40..72].copy_from_slice(&self.mint);
		bytes[72] = self.grade;
		bytes[73] = self.feeding;
		bytes[74..78].copy_from_slice(&self.xp.to_le_bytes());
		bytes[78..82].copy_from_slice(&self.anima.to_le_bytes());
		bytes[82..90].copy_from_slice(&self.at.to_le_bytes());
		bytes[90..98].copy_from_slice(&self.nonce.to_le_bytes());

		bytes
	}

	/// The receipt `bytes` hold, or `None` when they are not a version-two
	/// receipt.
	pub fn decode(bytes: &[u8]) -> Option<Self> {
		let bytes: &[u8; LEN] = bytes.try_into().ok()?;

		if &bytes[0..8] != TAG {
			return None;
		}

		Some(Self {
			player: bytes[8..40].try_into().ok()?,
			mint: bytes[40..72].try_into().ok()?,
			grade: bytes[72],
			feeding: bytes[73],
			xp: u32::from_le_bytes(bytes[74..78].try_into().ok()?),
			anima: u32::from_le_bytes(bytes[78..82].try_into().ok()?),
			at: i64::from_le_bytes(bytes[82..90].try_into().ok()?),
			nonce: u64::from_le_bytes(bytes[90..98].try_into().ok()?),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// A receipt whose every field is distinct, so a field written to the
	/// wrong place shows.
	fn sample() -> Receipt {
		Receipt {
			player: [0x11; 32],
			mint: [0x22; 32],
			grade: 2,
			feeding: 3,
			xp: 0x0102_0304,
			anima: 60,
			at: 1_790_000_000,
			nonce: 0xDEAD_BEEF,
		}
	}

	#[test]
	fn the_layout_matches_the_fixed_vector() {
		let mut expected = Vec::new();

		expected.extend_from_slice(b"RPRSABS2");
		expected.extend_from_slice(&[0x11; 32]);
		expected.extend_from_slice(&[0x22; 32]);
		expected.extend_from_slice(&[0x02, 0x03]);
		expected.extend_from_slice(&[0x04, 0x03, 0x02, 0x01]);
		expected.extend_from_slice(&[0x3C, 0x00, 0x00, 0x00]);
		expected.extend_from_slice(&[0x80, 0x3B, 0xB1, 0x6A, 0x00, 0x00, 0x00, 0x00]);
		expected.extend_from_slice(&[0xEF, 0xBE, 0xAD, 0xDE, 0x00, 0x00, 0x00, 0x00]);

		assert_eq!(expected.len(), LEN);
		assert_eq!(sample().encode().to_vec(), expected);
	}

	#[test]
	fn a_receipt_reads_back_as_written() {
		assert_eq!(Receipt::decode(&sample().encode()), Some(sample()));

		let mut wrong_tag = sample().encode();

		wrong_tag[7] = b'1';

		assert_eq!(Receipt::decode(&wrong_tag), None);
		assert_eq!(Receipt::decode(&sample().encode()[..97]), None);
	}
}
