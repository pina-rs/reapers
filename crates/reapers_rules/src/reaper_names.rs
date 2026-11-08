//! The names the shuffle draws from.

use crate::reaper_class::ReaperClass;

/// Given names the shuffle draws from.
///
/// A genome stores an index into this list, so entries are only ever
/// appended: reordering or removing one would rename reapers that already
/// exist.
pub const REAPER_GIVEN_NAMES: [&str; 40] = [
	"Vesper", "Morrow", "Thren", "Quill", "Oswin", "Nerys", "Calder", "Ilse", "Bram", "Sable",
	"Wren", "Corvin", "Maud", "Hesper", "Lorn", "Tamsin", "Garrick", "Isolde", "Fenwick", "Odile",
	"Rook", "Sorrel", "Thorne", "Vigil", "Wick", "Yarrow", "Dirge", "Knell", "Cairn", "Hollis",
	"Mire", "Neve", "Orrin", "Pall", "Silas", "Umbra", "Brannoc", "Eira", "Haldis", "Jorun",
];

/// Epithets by class. The same append-only rule as [`REAPER_GIVEN_NAMES`]
/// applies to each list.
pub const fn reaper_epithets(class: ReaperClass) -> &'static [&'static str] {
	match class {
		ReaperClass::Harvester => {
			&[
				"of the Last Furrow",
				"the Gleaner",
				"of the Long Field",
				"who Walks the Rows",
				"the Late Harvest",
				"of the Bent Stalk",
				"the Winnower",
				"of the Stubble Dark",
			]
		}
		ReaperClass::Ferryman => {
			&[
				"of the Slow Water",
				"the Unmoved",
				"who Takes the Fare",
				"of the Far Bank",
				"the Tarred",
				"of the Low Tide",
				"the Mooring",
				"who Poles the Dark",
			]
		}
		ReaperClass::Weigher => {
			&[
				"of the True Measure",
				"the Even Hand",
				"who Tips the Pan",
				"of the Feather Weight",
				"the Assayer",
				"of the Final Count",
				"the Unbribed",
				"who Reads the Beam",
			]
		}
		ReaperClass::Chooser => {
			&[
				"who Marks the Fallen",
				"of the Single Cast",
				"the Raven-Sworn",
				"of the Red Thread",
				"who Does Not Miss",
				"the Named Hour",
				"of the Quiet Wing",
				"the Singled Out",
			]
		}
		ReaperClass::Hollow => {
			&[
				"the Unheaded",
				"of the Empty Collar",
				"who Carries the Light",
				"of the Night Road",
				"the Riderless",
				"of the Rattling Chain",
				"who Lost the First Thing",
				"the Open Gate",
			]
		}
		ReaperClass::Mourner => {
			&[
				"of the First Toll",
				"the Keening",
				"who Washes at the Ford",
				"of the Grey Veil",
				"the Hush",
				"of the Long Lament",
				"who Names the Hour",
				"the Unconsoled",
			]
		}
	}
}
