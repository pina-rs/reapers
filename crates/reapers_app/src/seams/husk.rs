//! An NFT's portrait: the husk it becomes, in its own colours.

use leptoskit::prelude::*;
use reapers_rules::Nft;

use crate::theme::ASH;
use crate::theme::seed_for;
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

/// The portrait.
pub fn husk_portrait(props: HuskPortraitProps) -> Element {
	let HuskPortraitProps { nft, size, spent } = props;
	let plate = View()
		.class(format!(
			"relative w-[{size}px] h-[{size}px] overflow-hidden bg-[#0f1116]"
		))
		.accessibility_label(nft.title());
	// The plate carries the name; the picture is its decoration.
	let picture = Image(portrait_source(nft))
		.class("absolute inset-0 w-full h-full")
		.aria_hidden(true);
	let plate = plate
		.child(picture)
		.child(sketch(seed_for(&nft.id()), ASH, 0.5));
	if spent { plate.opacity(0.35) } else { plate }
}
