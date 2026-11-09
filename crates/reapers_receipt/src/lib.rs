//! The absorption receipt: what the keeper signs when a reaper absorbs a husk.
//!
//! How well a reaper feeds is rolled off-chain by the server, because
//! randomness on a chain is slow and costly. The roll still has to be something
//! the chain can trust, so the server's keeper key signs what it decided, and
//! the Reapers program checks that signature with Solana's Ed25519 program
//! before it burns the husk.
//!
//! This crate is the one description of those bytes. The program, the server
//! and the app all depend on it, so it has no dependencies and no `std`.
//!
//! # Layout
//!
//! A receipt is 98 bytes, laid out so that a program can read it without
//! parsing anything:
//!
//! | Bytes | Field |
//! | --- | --- |
//! | 0..8 | the ASCII tag `RPRSABS2`: what this is, and its version |
//! | 8..40 | the player's wallet address |
//! | 40..72 | the husk's mint (SPL Token) or asset (Metaplex Core) address |
//! | 72 | the capture grade: 0 plain, 1 great, 2 excellent |
//! | 73 | the feeding: 0 meagre, 1 fair, 2 rich, 3 feast |
//! | 74..78 | experience paid, unsigned, little-endian |
//! | 78..82 | anima paid, unsigned, little-endian |
//! | 82..90 | when, in seconds since 1970, signed, little-endian |
//! | 90..98 | the nonce the feeding was rolled from, little-endian |
//!
//! Version 1 (`RPRSABS1`, the first keeper's `AbsorptionReceipt`) had the same
//! layout but held the SHA-256 of stand-in identifiers in the two address
//! fields. It can never be posted, and [`Receipt::decode`] refuses it with
//! [`ReceiptError::Version1`].
//!
//! # Posting a receipt
//!
//! The transaction that absorbs a husk carries two instructions in a row: an
//! Ed25519 program instruction that verifies the keeper's signature over the
//! receipt, then the program's `Absorb`. [`ed25519_instruction_data`] builds the
//! first one in the only shape the program accepts: one signature, with the
//! public key, signature and message all inside that instruction's own data.

#![no_std]

/// The bytes at the start of every version 2 receipt.
pub const TAG: [u8; 8] = *b"RPRSABS2";

/// The tag of the first version, which held stand-ins instead of addresses.
pub const TAG_V1: [u8; 8] = *b"RPRSABS1";

/// The length of a receipt in bytes.
pub const LEN: usize = 98;

/// Where each field starts.
pub mod offset {
	/// The tag.
	pub const TAG: usize = 0;
	/// The player's wallet address.
	pub const PLAYER: usize = 8;
	/// The husk's mint or asset address.
	pub const MINT: usize = 40;
	/// The capture grade.
	pub const GRADE: usize = 72;
	/// The feeding.
	pub const FEEDING: usize = 73;
	/// Experience paid.
	pub const XP: usize = 74;
	/// Anima paid.
	pub const ANIMA: usize = 78;
	/// When.
	pub const AT: usize = 82;
	/// The nonce.
	pub const NONCE: usize = 90;
}

/// How well the capturing orb was thrown. The values are
/// `CaptureGrade::index` in `reapers_rules`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Grade {
	/// Thrown without a hold, with the ring in its outer third, or off the
	/// husk.
	Plain = 0,
	/// Let go with the ring in its middle third.
	Great = 1,
	/// Let go with the ring in its inner third.
	Excellent = 2,
}

impl Grade {
	/// The grade stored as `byte`, if there is one.
	pub const fn from_byte(byte: u8) -> Option<Self> {
		match byte {
			0 => Some(Self::Plain),
			1 => Some(Self::Great),
			2 => Some(Self::Excellent),
			_ => None,
		}
	}
}

/// How well the reaper fed. The values are `Feeding::index` in `reapers_rules`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Feeding {
	/// There was less in it than it looked.
	Meagre = 0,
	/// What the husk was worth, no more and no less.
	Fair = 1,
	/// More than expected.
	Rich = 2,
	/// Once in a while a husk gives twice what it should.
	Feast = 3,
}

impl Feeding {
	/// The feeding stored as `byte`, if there is one.
	pub const fn from_byte(byte: u8) -> Option<Self> {
		match byte {
			0 => Some(Self::Meagre),
			1 => Some(Self::Fair),
			2 => Some(Self::Rich),
			3 => Some(Self::Feast),
			_ => None,
		}
	}
}

/// Why a run of bytes is not a version 2 receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiptError {
	/// It is not 98 bytes long.
	Length,
	/// It is a version 1 receipt, whose addresses are stand-ins.
	Version1,
	/// It does not start with `RPRSABS2`.
	Tag,
	/// The grade byte is not 0, 1 or 2.
	Grade,
	/// The feeding byte is not 0, 1, 2 or 3.
	Feeding,
}

impl core::fmt::Display for ReceiptError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.write_str(match self {
			Self::Length => "a receipt is 98 bytes",
			Self::Version1 => "a version 1 receipt holds stand-ins and cannot be posted",
			Self::Tag => "the receipt does not start with RPRSABS2",
			Self::Grade => "the receipt's grade is not 0, 1 or 2",
			Self::Feeding => "the receipt's feeding is not 0, 1, 2 or 3",
		})
	}
}

impl core::error::Error for ReceiptError {}

/// What the keeper vouches for when a reaper absorbs a husk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Receipt {
	/// The player's wallet address.
	pub player: [u8; 32],
	/// The husk's mint (SPL Token) or asset (Metaplex Core) address.
	pub mint: [u8; 32],
	/// How well the capturing orb was thrown.
	pub grade: Grade,
	/// How well the reaper fed.
	pub feeding: Feeding,
	/// Experience paid.
	pub xp: u32,
	/// Anima paid.
	pub anima: u32,
	/// When, in seconds since 1970.
	pub at: i64,
	/// The nonce the feeding was rolled from.
	pub nonce: u64,
}

impl Receipt {
	/// The 98 bytes the keeper signs.
	pub fn encode(&self) -> [u8; LEN] {
		let mut bytes = [0; LEN];

		bytes[offset::TAG..offset::PLAYER].copy_from_slice(&TAG);
		bytes[offset::PLAYER..offset::MINT].copy_from_slice(&self.player);
		bytes[offset::MINT..offset::GRADE].copy_from_slice(&self.mint);
		bytes[offset::GRADE] = self.grade as u8;
		bytes[offset::FEEDING] = self.feeding as u8;
		bytes[offset::XP..offset::ANIMA].copy_from_slice(&self.xp.to_le_bytes());
		bytes[offset::ANIMA..offset::AT].copy_from_slice(&self.anima.to_le_bytes());
		bytes[offset::AT..offset::NONCE].copy_from_slice(&self.at.to_le_bytes());
		bytes[offset::NONCE..LEN].copy_from_slice(&self.nonce.to_le_bytes());

		bytes
	}

	/// Reads a version 2 receipt, checking its length, tag, grade and feeding.
	pub fn decode(bytes: &[u8]) -> Result<Self, ReceiptError> {
		let Ok(bytes) = <&[u8; LEN]>::try_from(bytes) else {
			return Err(ReceiptError::Length);
		};
		let tag = &bytes[offset::TAG..offset::PLAYER];

		if tag == TAG_V1 {
			return Err(ReceiptError::Version1);
		}
		if tag != TAG {
			return Err(ReceiptError::Tag);
		}

		Ok(Self {
			player: field(bytes, offset::PLAYER),
			mint: field(bytes, offset::MINT),
			grade: Grade::from_byte(bytes[offset::GRADE]).ok_or(ReceiptError::Grade)?,
			feeding: Feeding::from_byte(bytes[offset::FEEDING]).ok_or(ReceiptError::Feeding)?,
			xp: u32::from_le_bytes(field(bytes, offset::XP)),
			anima: u32::from_le_bytes(field(bytes, offset::ANIMA)),
			at: i64::from_le_bytes(field(bytes, offset::AT)),
			nonce: u64::from_le_bytes(field(bytes, offset::NONCE)),
		})
	}
}

/// The `N` bytes of `bytes` starting at `start`. Every caller passes an offset
/// from [`offset`], so the range is always inside a receipt.
fn field<const N: usize>(bytes: &[u8; LEN], start: usize) -> [u8; N] {
	let mut out = [0; N];
	out.copy_from_slice(&bytes[start..start + N]);
	out
}

/// Solana's Ed25519 signature verification program,
/// `Ed25519SigVerify111111111111111111111111111`.
pub const ED25519_PROGRAM_ID: [u8; 32] = [
	3, 125, 70, 214, 124, 147, 251, 190, 18, 249, 66, 143, 131, 141, 64, 255, 5, 112, 116, 73, 39,
	244, 138, 100, 252, 202, 112, 68, 128, 0, 0, 0,
];

/// The Ed25519 program instruction's layout, as far as a receipt uses it.
pub mod ed25519 {
	/// An instruction index meaning "this instruction".
	pub const THIS_INSTRUCTION: u16 = u16::MAX;
	/// The count byte and one padding byte.
	pub const HEADER: usize = 2;
	/// One signature's offsets: seven little-endian `u16`s.
	pub const OFFSETS: usize = 14;
	/// Where the public key starts in an instruction built for a receipt.
	pub const PUBLIC_KEY: usize = HEADER + OFFSETS;
	/// Where the signature starts.
	pub const SIGNATURE: usize = PUBLIC_KEY + 32;
	/// Where the receipt starts.
	pub const MESSAGE: usize = SIGNATURE + 64;
	/// The length of an instruction built for a receipt.
	pub const LEN: usize = MESSAGE + super::LEN;
}

/// The data of the Ed25519 program instruction that verifies `signature`, by
/// `keeper`, over `receipt`.
///
/// It holds one signature, and every offset points into this same instruction
/// (index `u16::MAX`), which is the only shape the Reapers program accepts.
pub fn ed25519_instruction_data(
	keeper: &[u8; 32],
	signature: &[u8; 64],
	receipt: &[u8; LEN],
) -> [u8; ed25519::LEN] {
	let mut data = [0; ed25519::LEN];
	let offsets: [u16; 7] = [
		ed25519::SIGNATURE as u16,
		ed25519::THIS_INSTRUCTION,
		ed25519::PUBLIC_KEY as u16,
		ed25519::THIS_INSTRUCTION,
		ed25519::MESSAGE as u16,
		LEN as u16,
		ed25519::THIS_INSTRUCTION,
	];

	data[0] = 1;
	for (index, value) in offsets.iter().enumerate() {
		let at = ed25519::HEADER + index * 2;
		data[at..at + 2].copy_from_slice(&value.to_le_bytes());
	}
	data[ed25519::PUBLIC_KEY..ed25519::SIGNATURE].copy_from_slice(keeper);
	data[ed25519::SIGNATURE..ed25519::MESSAGE].copy_from_slice(signature);
	data[ed25519::MESSAGE..].copy_from_slice(receipt);

	data
}

#[cfg(test)]
mod tests {
	extern crate std;

	use std::fmt::Write;
	use std::string::String;

	use ed25519_dalek::Signer;
	use ed25519_dalek::SigningKey;
	use ed25519_dalek::Verifier;
	use sha2::Digest;
	use sha2::Sha256;

	use super::*;

	fn hex(bytes: &[u8]) -> String {
		bytes.iter().fold(String::new(), |mut out, byte| {
			let _ = write!(out, "{byte:02x}");
			out
		})
	}

	fn sample() -> Receipt {
		Receipt {
			player: core::array::from_fn(|i| i as u8),
			mint: core::array::from_fn(|i| 0xff - i as u8),
			grade: Grade::Excellent,
			feeding: Feeding::Feast,
			xp: 300,
			anima: 60,
			at: 1_793_836_800,
			nonce: 0x0123_4567_89ab_cdef,
		}
	}

	/// The bytes of a known receipt. The server and the app must produce
	/// exactly these for the same fields.
	#[test]
	fn encodes_the_fixed_vector() {
		let bytes = sample().encode();

		assert_eq!(
			hex(&bytes),
			concat!(
				"5250525341425332",
				"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
				"fffefdfcfbfaf9f8f7f6f5f4f3f2f1f0efeeedecebeae9e8e7e6e5e4e3e2e1e0",
				"02",
				"03",
				"2c010000",
				"3c000000",
				"00c7eb6a00000000",
				"efcdab8967452301",
			)
		);
	}

	#[test]
	fn round_trips() {
		let receipt = sample();
		assert_eq!(Receipt::decode(&receipt.encode()), Ok(receipt));

		let early = Receipt {
			grade: Grade::Plain,
			feeding: Feeding::Meagre,
			at: -1,
			..receipt
		};
		assert_eq!(Receipt::decode(&early.encode()), Ok(early));
	}

	#[test]
	fn refuses_the_wrong_length() {
		let bytes = sample().encode();
		assert_eq!(Receipt::decode(&bytes[..97]), Err(ReceiptError::Length));

		let mut longer = [0; 99];
		longer[..98].copy_from_slice(&bytes);
		assert_eq!(Receipt::decode(&longer), Err(ReceiptError::Length));
	}

	#[test]
	fn refuses_version_1_and_other_tags() {
		let mut bytes = sample().encode();
		bytes[..8].copy_from_slice(&TAG_V1);
		assert_eq!(Receipt::decode(&bytes), Err(ReceiptError::Version1));

		bytes[..8].copy_from_slice(b"RPRSABS3");
		assert_eq!(Receipt::decode(&bytes), Err(ReceiptError::Tag));
	}

	#[test]
	fn refuses_unknown_grades_and_feedings() {
		let mut bytes = sample().encode();
		bytes[offset::GRADE] = 3;
		assert_eq!(Receipt::decode(&bytes), Err(ReceiptError::Grade));

		let mut bytes = sample().encode();
		bytes[offset::FEEDING] = 4;
		assert_eq!(Receipt::decode(&bytes), Err(ReceiptError::Feeding));
	}

	#[test]
	fn grades_and_feedings_match_the_shared_rules() {
		for (byte, grade) in [Grade::Plain, Grade::Great, Grade::Excellent]
			.into_iter()
			.enumerate()
		{
			assert_eq!(Grade::from_byte(byte as u8), Some(grade));
		}
		for (byte, feeding) in [
			Feeding::Meagre,
			Feeding::Fair,
			Feeding::Rich,
			Feeding::Feast,
		]
		.into_iter()
		.enumerate()
		{
			assert_eq!(Feeding::from_byte(byte as u8), Some(feeding));
		}
	}

	/// The development keeper is derived the way `ReceiptSigner` in
	/// `reapers_server` derives it: the seed is the SHA-256 of the secret. Its
	/// signature over the fixed receipt is a vector for any other signer.
	#[test]
	fn the_development_keeper_signs_the_fixed_vector() {
		let seed: [u8; 32] = Sha256::digest(b"reapers-development-receipt-key").into();
		let keeper = SigningKey::from_bytes(&seed);
		let message = sample().encode();
		let signature = keeper.sign(&message);

		assert_eq!(
			bs58::encode(keeper.verifying_key().as_bytes()).into_string(),
			"5tTh2Cc1DkXv1KDhB2XnCkwsN1GuvuZU9C21yCo7BCrd",
		);
		assert!(keeper.verifying_key().verify(&message, &signature).is_ok());
		assert_eq!(
			hex(&signature.to_bytes()),
			concat!(
				"23caa588ebb6d4e8f44c590777b003dbafd95e95ef3c9663073dfa5e8ba3d370",
				"b1926a5eb329af296db130e675e11e2186c8c40084815b6f875d92512f575f04",
			),
		);
	}

	#[test]
	fn builds_a_self_contained_ed25519_instruction() {
		let keeper = [7; 32];
		let signature = [9; 64];
		let receipt = sample().encode();
		let data = ed25519_instruction_data(&keeper, &signature, &receipt);
		let read = |at: usize| u16::from_le_bytes([data[at], data[at + 1]]);

		assert_eq!(data.len(), 210);
		assert_eq!(data[0], 1);
		assert_eq!(data[1], 0);
		assert_eq!(read(2), 48);
		assert_eq!(read(4), u16::MAX);
		assert_eq!(read(6), 16);
		assert_eq!(read(8), u16::MAX);
		assert_eq!(read(10), 112);
		assert_eq!(read(12), 98);
		assert_eq!(read(14), u16::MAX);
		assert_eq!(&data[16..48], &keeper);
		assert_eq!(&data[48..112], &signature);
		assert_eq!(&data[112..], &receipt);
	}

	#[test]
	fn names_the_ed25519_program() {
		// `Ed25519SigVerify111111111111111111111111111`, decoded from base58.
		let decoded = bs58::decode("Ed25519SigVerify111111111111111111111111111")
			.into_vec()
			.unwrap();
		assert_eq!(decoded, ED25519_PROGRAM_ID);
	}
}
