//! The absorption receipt the keeper signs and the Reapers program checks.
//!
//! The layout is `reapers_receipt`'s, the crate the program reads it with,
//! so the keeper and the program cannot disagree about a byte. All this
//! module adds is how an absorption the rules paid becomes a receipt.

use reapers_receipt::Feeding;
use reapers_receipt::Grade;
pub use reapers_receipt::LEN;
pub use reapers_receipt::Receipt;
pub use reapers_receipt::TAG;
use reapers_rules::Absorption;

/// The receipt for `absorption`, made out to `player` for `mint`, or `None`
/// when a figure does not fit its field, which no absorption the rules pay
/// can do.
pub fn of(player: [u8; 32], mint: [u8; 32], absorption: &Absorption) -> Option<Receipt> {
	Some(Receipt {
		player,
		mint,
		grade: Grade::from_byte(u8::try_from(absorption.grade.index()).ok()?)?,
		feeding: Feeding::from_byte(u8::try_from(absorption.feeding.index()).ok()?)?,
		xp: u32::try_from(absorption.xp).ok()?,
		anima: u32::try_from(absorption.anima).ok()?,
		at: absorption.at.unix_seconds(),
		nonce: u64::try_from(absorption.nonce).ok()?,
	})
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
			grade: Grade::from_byte(2).unwrap(),
			feeding: Feeding::from_byte(3).unwrap(),
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
		assert_eq!(Receipt::decode(&expected), Ok(sample()));
	}
}
