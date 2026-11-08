//! Pieces more than one screen shows.

use leptoskit::prelude::*;
use reapers_rules::ReaperStats;

use crate::l10n::use_l10n;
use crate::theme::Rgb;
use crate::ui::ledger_pip;
use crate::ui::margin;

/// A calling's four ratings, each as five pips.
pub fn reaper_stat_rows(stats: ReaperStats, light: Rgb) -> Element {
	let words = use_l10n();
	let rows = [
		(words.sanctum_stat_strike(), stats.strike),
		(words.sanctum_stat_reach(), stats.reach),
		(words.sanctum_stat_capture(), stats.capture),
		(words.sanctum_stat_resilience(), stats.resilience),
	];
	View().children(rows.into_iter().map(|(name, rating)| {
		View()
			.class("flex-row items-center py-[7px]")
			.accessibility_label(name.to_owned())
			.aria_value_text(rating.to_string())
			.child(View().class("flex-1").child(margin(name)))
			.children((1..=5).map(|pip| {
				View()
					.class("ml-[9px]")
					.child(ledger_pip(pip <= rating, light))
			}))
	}))
}
