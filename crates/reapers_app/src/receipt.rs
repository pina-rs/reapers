//! The 98 bytes a keeper signs when a reaper absorbs a husk.
//!
//! The layout is `reapers_receipt`'s, the crate the server and the on-chain
//! program read it with, so the simnet's burns carry exactly what the
//! program checks. All this module adds is how an absorption the rules paid
//! becomes a receipt.

use reapers_receipt::Feeding;
use reapers_receipt::Grade;
pub use reapers_receipt::LEN;
pub use reapers_receipt::Receipt;
pub use reapers_receipt::TAG;
use reapers_rules::Absorption;

/// The receipt for `absorption`, made out to the wallet `player` for the
/// husk `mint`, or `None` when a figure does not fit its field, which no
/// absorption the rules pay can do.
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
	use reapers_rules::CaptureGrade;
	use reapers_rules::Nft;
	use reapers_rules::NftCollection;
	use reapers_rules::Timestamp;

	use super::*;

	#[test]
	fn an_absorption_is_the_shared_layout() {
		let absorption = Absorption {
			nft: Nft::new(NftCollection::ALL[0], 1),
			grade: CaptureGrade::Excellent,
			feeding: reapers_rules::Feeding::Feast,
			xp: 120,
			anima: 30,
			nonce: 0x0102_0304,
			at: Timestamp::from_unix_seconds(1_700_000_000),
			signature: None,
		};
		let bytes = of([1; 32], [2; 32], &absorption).unwrap().encode();

		assert_eq!(bytes.len(), LEN);
		assert_eq!(&bytes[0..8], &TAG);
		assert_eq!(bytes[8..40], [1; 32]);
		assert_eq!(bytes[40..72], [2; 32]);
		assert_eq!(bytes[72], 2);
		assert_eq!(bytes[73], 3);
		assert_eq!(bytes[74..78], 120_u32.to_le_bytes());
		assert_eq!(bytes[78..82], 30_u32.to_le_bytes());
		assert_eq!(bytes[82..90], 1_700_000_000_i64.to_le_bytes());
		assert_eq!(bytes[90..98], 0x0102_0304_u64.to_le_bytes());
		assert_eq!(
			Receipt::decode(&bytes).map(|receipt| receipt.encode()),
			Ok(bytes)
		);
	}
}
