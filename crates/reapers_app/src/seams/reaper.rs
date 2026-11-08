//! The reaper, standing in the dark.

use std::sync::LazyLock;

use leptoskit::prelude::*;
use reapers_rules::Outfit;
use reapers_rules::ReaperGenome;
use reapers_rules::Weapon;

use crate::theme::Rgb;
use crate::theme::reaper_light;

/// Everything the stage needs to draw one reaper: the Flutter
/// `ReaperStage`'s constructor.
///
/// The stage fills the box it is given (lay it `absolute inset-0` or in a
/// sized parent) and draws on transparent ground over the page's ink. The
/// reaper stands centred, looked at from about chest height (the Flutter
/// camera's target is 1.42 m up), lit with [`reaper_light`].
#[derive(Clone, Debug, PartialEq)]
pub struct ReaperStageProps {
	/// The reaper's birth traits: calling, build, colourway, omen.
	pub genome: ReaperGenome,
	/// What it holds. The armory passes what the player is trying on.
	pub weapon: Weapon,
	/// What it wears: the same model, dyed, charred or hollowed.
	pub outfit: Outfit,
	/// Whether it turns slowly by itself (the oath's reveal). A stage also
	/// turns under a drag, and holds still under reduced motion.
	pub turns_on_its_own: bool,
	/// Whether the camera moves in on the face (the studio's expressions).
	pub focus_face: bool,
}

impl ReaperStageProps {
	pub fn new(genome: ReaperGenome, weapon: Weapon, outfit: Outfit) -> Self {
		Self {
			genome,
			weapon,
			outfit,
			turns_on_its_own: false,
			focus_face: false,
		}
	}
}

/// A hooded figure and a blade, drawn in a 100 by 160 box.
static FIGURE: LazyLock<PathData> = LazyLock::new(|| {
	PathData::parse(
		"M50 18 C38 18 31 30 31 45 C31 54 34 60 29 72 L20 146 C32 153 68 153 80 146 L71 72 C66 60 \
		 69 54 69 45 C69 30 62 18 50 18 Z",
	)
	.expect("the figure's path is valid")
});
static FACE: LazyLock<PathData> = LazyLock::new(|| {
	PathData::parse(
		"M50 36 C44 36 41 42 41 49 C41 56 45 60 50 60 C55 60 59 56 59 49 C59 42 56 36 50 36 Z",
	)
	.expect("the face's path is valid")
});
static STAFF: LazyLock<PathData> =
	LazyLock::new(|| PathData::parse("M84 22 L74 156").expect("the staff's path is valid"));
static BLADE: LazyLock<PathData> = LazyLock::new(|| {
	PathData::parse("M84 22 C68 8 40 10 24 26 C42 20 62 21 82 31 Z")
		.expect("the blade's path is valid")
});

fn paint(colour: Rgb, alpha: u8) -> Color {
	Color::rgba(colour.0, colour.1, colour.2, alpha)
}

/// The reaper on its stage.
///
/// Until `leptoskit_3d` draws the reaper's model, this is its silhouette in
/// its own light: hood, robe, and the hollow where a face would be.
pub fn reaper_stage(props: &ReaperStageProps) -> Element {
	let light = reaper_light(&props.genome);
	let line = |alpha| {
		Stroke::new(paint(light, alpha), 1.4)
			.expect("a positive width")
			.round()
	};
	let figure = Vector(ViewBox::new(0.0, 0.0, 100.0, 160.0).expect("a positive size"))
		.path(VectorPath::new(STAFF.clone()).stroke(line(0x99)))
		.path(
			VectorPath::new(FIGURE.clone())
				.fill(paint(Rgb(0x0F, 0x11, 0x16), 0xFF))
				.stroke(line(0xCC)),
		)
		.path(VectorPath::new(FACE.clone()).fill(paint(Rgb(0x02, 0x02, 0x03), 0xFF)))
		.path(
			VectorPath::new(BLADE.clone())
				.fill(paint(light, 0x33))
				.stroke(line(0xDD)),
		)
		.class("w-[46%] max-w-[220px] h-[78%]")
		.aria_hidden(true);
	View()
		.class("absolute inset-0 items-center justify-center overflow-hidden")
		.test_id("reaper-stage")
		.child(View().class(format!(
			"absolute w-[320px] h-[320px] rounded-full bg-[{}]",
			light.hex_alpha(0.06)
		)))
		.child(figure)
}
