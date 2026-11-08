//! The reaper drawn flat, in its own light: what a stage shows while its
//! model loads, and where 3D cannot be drawn.

use std::sync::LazyLock;

use leptoskit::prelude::*;
use reapers_rules::ReaperGenome;

use crate::theme::Rgb;
use crate::theme::reaper_light;

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

/// The reaper's silhouette in its own light: hood, robe, and the hollow
/// where a face would be.
pub fn silhouette(genome: &ReaperGenome) -> Element {
	let light = reaper_light(genome);
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
		.child(View().class(format!(
			"absolute w-[320px] h-[320px] rounded-full bg-[{}]",
			light.hex_alpha(0.06)
		)))
		.child(figure)
}
