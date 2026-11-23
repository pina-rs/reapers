//! An NFT's portrait: the husk it becomes, in its own colours.

use leptoskit::prelude::*;
use reapers_rules::Nft;

use crate::theme::ASH;
use crate::ui::sketch;

/// One NFT's portrait, squared off and ruled like a plate in a catalogue:
/// the Flutter `NftThumbnail`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HuskPortraitProps {
	pub nft: Nft,
	/// The plate's side, in layout units.
	pub size: u32,
	/// Whether the NFT is gone (absorbed), and the plate faded.
	pub spent: bool,
}

/// Where `nft`'s portrait is bundled: `assets/nfts/<image key>.webp`, built
/// from the NFT's own creature by the art build. No collection's artwork is
/// used; see docs/nfts.md. An `asset://` source names the same file in the
/// browser and in the native apps, which bundle the project's `assets/`.
pub fn portrait_source(nft: Nft) -> String {
	format!("asset://nfts/{}.webp", nft.image_key())
}

/// FNV-1a 32-bit hash matching `seed_for(&nft.id())` without allocating `nft.id()`.
fn nft_seed(nft: Nft) -> u32 {
	let mut hash: u32 = 0x811C_9DC5;
	let mut feed = |bytes: &[u8]| {
		for &b in bytes {
			hash ^= u32::from(b);
			hash = hash.wrapping_mul(0x0100_0193);
		}
	};
	feed(nft.collection.name().as_bytes());
	feed(b":");
	let mut buf = [0_u8; 20];
	let mut n = nft.serial;
	let mut i = buf.len();
	if n == 0 {
		i -= 1;
		buf[i] = b'0';
	} else {
		while n > 0 {
			i -= 1;
			buf[i] = b'0' + (n % 10) as u8;
			n /= 10;
		}
	}
	feed(&buf[i..]);
	hash
}

/// The portrait.
pub fn husk_portrait(props: HuskPortraitProps) -> Element {
	let HuskPortraitProps { nft, size, spent } = props;
	let plate = match size {
		48 => View().class("relative w-[48px] h-[48px] overflow-hidden bg-[#0f1116]"),
		44 => View().class("relative w-[44px] h-[44px] overflow-hidden bg-[#0f1116]"),
		_ => {
			View().class(format!(
				"relative w-[{size}px] h-[{size}px] overflow-hidden bg-[#0f1116]"
			))
		}
	}
	.accessibility_label(nft.title());
	// The plate carries the name; the picture is its decoration.
	let picture = Image(portrait_source(nft))
		.class("absolute inset-0 w-full h-full")
		.aria_hidden(true);
	let plate = plate.child(picture).child(sketch(nft_seed(nft), ASH, 0.5));
	if spent { plate.opacity(0.35) } else { plate }
}
