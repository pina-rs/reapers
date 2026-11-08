//! The 98 bytes a keeper signs when a reaper absorbs an NFT.
//!
//! TODO(integration): `reapers_receipt`. The layout belongs to the
//! `reapers_receipt` crate the server and the on-chain program share; it is
//! not in this checkout yet, so the simnet builds the same bytes here, field
//! for field from the Dart `AbsorptionReceipt`. Replace [`message`] with the
//! crate's builder when it lands, and keep the test below as the check that
//! the two agree.
//!
//! | Bytes | Field |
//! | --- | --- |
//! | 0..8 | the ASCII tag `RPRSABS1` |
//! | 8..40 | the player: SHA-256 of their identifier |
//! | 40..72 | the NFT's mint: SHA-256 of its mock mint |
//! | 72 | the reap grade: 0 ragged, 1 clean, 2 perfect |
//! | 73 | the feeding: 0 meagre, 1 fair, 2 rich, 3 feast |
//! | 74..78 | experience paid, little-endian |
//! | 78..82 | anima paid, little-endian |
//! | 82..90 | when, in seconds since 1970, little-endian |
//! | 90..98 | the nonce the feeding was rolled from, little-endian |

use reapers_rules::Absorption;
use sha2::Digest as _;
use sha2::Sha256;

/// The bytes at the start of every message.
pub const TAG: &[u8; 8] = b"RPRSABS1";

/// The length of a message in bytes.
pub const LENGTH: usize = 98;

/// The message that vouches for `absorption` by the player `player_id`.
pub fn message(player_id: &str, absorption: &Absorption) -> [u8; LENGTH] {
	let mut message = [0; LENGTH];
	message[0..8].copy_from_slice(TAG);
	message[8..40].copy_from_slice(&stand_in(player_id));
	message[40..72].copy_from_slice(&stand_in(&absorption.nft.mock_mint()));
	message[72] = absorption.grade.index() as u8;
	message[73] = absorption.feeding.index() as u8;
	message[74..78].copy_from_slice(&(absorption.xp as u32).to_le_bytes());
	message[78..82].copy_from_slice(&(absorption.anima as u32).to_le_bytes());
	message[82..90].copy_from_slice(&absorption.at.unix_seconds().to_le_bytes());
	message[90..98].copy_from_slice(&(absorption.nonce as u64).to_le_bytes());
	message
}

/// The 32 bytes that stand for an identifier until it is a real address.
fn stand_in(identifier: &str) -> [u8; 32] {
	Sha256::digest(identifier.as_bytes()).into()
}

#[cfg(test)]
mod tests {
	use reapers_rules::Feeding;
	use reapers_rules::Nft;
	use reapers_rules::NftCollection;
	use reapers_rules::ReapGrade;
	use reapers_rules::Timestamp;

	use super::*;

	#[test]
	fn the_layout_is_the_dart_one() {
		let absorption = Absorption {
			nft: Nft::new(NftCollection::ALL[0], 1),
			grade: ReapGrade::Perfect,
			feeding: Feeding::Feast,
			xp: 120,
			anima: 30,
			nonce: 0x0102_0304,
			at: Timestamp::from_unix_seconds(1_700_000_000),
			signature: None,
		};
		let bytes = message("player", &absorption);

		assert_eq!(&bytes[0..8], TAG);
		assert_eq!(bytes[8..40], stand_in("player"));
		assert_eq!(bytes[40..72], stand_in(&absorption.nft.mock_mint()));
		assert_eq!(bytes[72], 2);
		assert_eq!(bytes[73], 3);
		assert_eq!(bytes[74..78], 120_u32.to_le_bytes());
		assert_eq!(bytes[78..82], 30_u32.to_le_bytes());
		assert_eq!(bytes[82..90], 1_700_000_000_i64.to_le_bytes());
		assert_eq!(bytes[90..98], 0x0102_0304_u64.to_le_bytes());
	}
}
