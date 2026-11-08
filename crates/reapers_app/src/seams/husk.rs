//! An NFT's portrait: the husk it becomes, in its own colours.

use std::sync::LazyLock;

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
/// used; see docs/nfts.md.
pub fn portrait_source(nft: Nft) -> String {
	format!("assets/nfts/{}.webp", nft.image_key())
}

/// A husk's outline, for a host that cannot load the bundled picture.
static HUSK: LazyLock<PathData> = LazyLock::new(|| {
	PathData::parse(
		"M20 78 C14 60 18 34 34 24 C42 18 58 18 66 24 C82 34 86 60 80 78 C70 86 30 86 20 78 Z M38 46 \
		 C35 46 34 50 36 52 C38 54 41 52 41 49 C41 47 40 46 38 46 Z M62 46 C59 46 58 49 59 51 C60 54 64 \
		 53 64 50 C64 47 63 46 62 46 Z",
	)
	.expect("the husk's path is valid")
});

/// The portrait.
///
/// The browser shows the bundled WebP. The native apps cannot yet name a
/// bundled file to an `Image` (their hosts take `file://` paths and bundle
/// only the guest), so there the plate holds the husk's outline in the
/// NFT's own colour; see the report's leptoskit gaps.
pub fn husk_portrait(props: HuskPortraitProps) -> Element {
	let HuskPortraitProps { nft, size, spent } = props;
	let plate = View()
		.class(format!(
			"relative w-[{size}px] h-[{size}px] overflow-hidden bg-[#0f1116]"
		))
		.accessibility_label(nft.title());
	let picture = if cfg!(all(target_arch = "wasm32", feature = "web")) {
		// The plate carries the name; the picture is its decoration.
		Image(portrait_source(nft))
			.class("absolute inset-0 w-full h-full")
			.aria_hidden(true)
	} else {
		let accent = nft.look().accent;
		let colour = Color::rgba(
			(accent >> 16) as u8,
			(accent >> 8) as u8,
			accent as u8,
			0xCC,
		);
		Vector(ViewBox::new(0.0, 0.0, 100.0, 100.0).expect("a positive size"))
			.path(VectorPath::new(HUSK.clone()).fill(colour))
			.class("absolute inset-0")
	};
	let plate = plate
		.child(picture)
		.child(sketch(seed_for(&nft.id()), ASH, 0.5));
	if spent { plate.opacity(0.35) } else { plate }
}
