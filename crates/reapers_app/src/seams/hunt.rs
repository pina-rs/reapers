//! One hunt, from the sigil that draws the husk out to the reap.

use std::sync::Arc;

use leptoskit::prelude::*;
use reapers_rules::Art;
use reapers_rules::Difficulty;
use reapers_rules::Feeding;
use reapers_rules::HuntBot;
use reapers_rules::HuntLoadout;
use reapers_rules::HuntLog;
use reapers_rules::HuntOutcome;
use reapers_rules::HuntPlan;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::Nft;
use reapers_rules::ReaperGift;

use super::HuskPortraitProps;
use super::husk_portrait;
use crate::l10n::use_l10n;
use crate::lookups::difficulty_name;
use crate::lookups::rarity_name;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::ui::LedgerButton;
use crate::ui::aside;
use crate::ui::display;
use crate::ui::gap;
use crate::ui::ledger_back;
use crate::ui::ledger_row;
use crate::ui::margin;

/// What a hunt is played from: the Flutter `HuntBrief`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HuntBrief {
	/// The husk.
	pub nft: Nft,
	/// The seed of every hunt for this husk: where it hides, how it moves,
	/// when it shrieks, and the shape of its sigil.
	pub seed: i64,
	/// The reaper's strength, weapon and gifts as the rules read them.
	pub loadout: HuntLoadout,
	/// The loose husk being hunted, or `None` for a practice hunt that leaves
	/// the books alone.
	pub sighting_id: Option<i64>,
}

impl HuntBrief {
	pub const fn is_practice(&self) -> bool {
		self.sighting_id.is_none()
	}
}

/// What a finished hunt came to, as the keeper of the books judged it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HuntReward {
	/// The verdict: the keeper's, which is the one that counts if it ever
	/// differs from what the device saw.
	pub outcome: HuntOutcome,
	pub xp: i64,
	pub anima: i64,
	/// How well the reaper fed, or `None` when it fed on nothing.
	pub feeding: Option<Feeding>,
	/// Whether the keeper signed a receipt for the absorption.
	pub signed: bool,
	pub level_before: i64,
	pub level_after: i64,
	pub gift: Option<ReaperGift>,
	pub soul_rank_before: i64,
	pub soul_rank_after: i64,
	/// The art the husk carried, and how deep the reaper knew it before and
	/// after: absorbing a husk is how an art is learned.
	pub art: Option<Art>,
	pub art_rank_before: i64,
	pub art_rank_after: i64,
}

impl HuntReward {
	/// A practice hunt: the device's own verdict, and nothing changes hands.
	pub const fn practice(outcome: HuntOutcome) -> Self {
		Self {
			outcome,
			xp: 0,
			anima: 0,
			feeding: None,
			signed: false,
			level_before: 0,
			level_after: 0,
			gift: None,
			soul_rank_before: 0,
			soul_rank_after: 0,
			art: None,
			art_rank_before: 0,
			art_rank_after: 0,
		}
	}

	pub const fn levelled_up(&self) -> bool {
		self.level_after > self.level_before
	}
}

/// Everything the hunt's view is given: the Flutter `HuntScreen`'s
/// constructor, with the books' half (settling) left to the route.
///
/// The view owns the hunt from the threshold to the last tick. It builds
/// the `HuntSim` from `brief` (`HuntSim::new` with the room's `HuntPlan`,
/// at `difficulty`), feeds it a `HuntInput` every tick from the camera and
/// the pose, records each into a `HuntLog`, and when the sim ends calls
/// `on_finished` once with the log and the outcome it saw. The route
/// settles that with the keeper and shows the result; the view draws
/// nothing after it. `on_leave` walks away before the end, which a hunt
/// for a loose husk does not record.
#[derive(Clone)]
pub struct HuntViewProps {
	pub brief: HuntBrief,
	pub difficulty: Difficulty,
	/// Whether a simulated player fights (`REAPERS_AUTOPLAY`).
	pub autoplay: bool,
	pub on_finished: Arc<dyn Fn(HuntLog, HuntOutcome) + Send + Sync>,
	/// Moves on to the next testing difficulty, in a build that has one.
	pub on_difficulty: Option<Arc<dyn Fn() + Send + Sync>>,
}

/// The rules' own careful player, fighting `brief` in the open at
/// `difficulty`: the log a hunt would record, and the outcome it saw.
pub fn autoplay_log(brief: &HuntBrief, difficulty: Difficulty) -> Option<(HuntLog, HuntOutcome)> {
	let stats = HuntStats::derive(&brief.loadout, 1.0).ok()?;
	let vessel = brief.nft.collection.vessel();
	let mut sim = HuntSim::new(
		brief.seed,
		vessel,
		brief.nft.rarity().ward(),
		&stats,
		HuntPlan::OPEN,
		difficulty,
	)
	.ok()?;
	let mut bot = HuntBot::careful(&sim);
	let outcome = bot.play(&mut sim)?;
	Some((
		HuntLog {
			inputs: bot.recorded,
			plan: HuntPlan::OPEN,
			difficulty,
		},
		outcome,
	))
}

/// The hunt.
///
/// Until `leptoskit_xr` brings the camera and `leptoskit_3d` the husk, this
/// is the hunt's threshold: the husk, how hard the hunt is, and a reaper
/// that hunts it by itself, with the rules' own careful player. Its log is
/// a real one, settled by the same keeper, so everything after the hunt is
/// the game's own.
pub fn hunt_view(props: &HuntViewProps) -> Element {
	let words = use_l10n();
	let brief = props.brief;
	let difficulty = props.difficulty;
	let finished = props.on_finished.clone();
	let fight = move || {
		if let Some((log, outcome)) = autoplay_log(&brief, difficulty) {
			finished(log, outcome);
		}
	};

	let mut column = View()
		.class("flex-1 w-full max-w-[520px] self-center px-7 pt-2 pb-7")
		.test_id("hunt-view")
		.child(ledger_back(None))
		.child(gap(24))
		.child(margin(if brief.is_practice() {
			words.hunt_practice_label()
		} else {
			words.hunt_sighted_label()
		}))
		.child(gap(10))
		.child(display(brief.nft.title(), 40, BONE))
		.child(gap(6))
		.child(margin(rarity_name(&*words, brief.nft.rarity())))
		.child(gap(22))
		.child(
			View()
				.class("items-center")
				.child(husk_portrait(HuskPortraitProps {
					nft: brief.nft,
					size: 168,
					spent: false,
				})),
		)
		.child(gap(22))
		.child(ledger_row(
			words.hunt_ward(),
			brief.nft.rarity().ward().to_string(),
			false,
		))
		.child(ledger_row(
			words.hunt_difficulty(),
			difficulty_name(&*words, difficulty),
			false,
		));
	if let Some(cycle) = props.on_difficulty.clone() {
		column = column.child(
			LedgerButton::new(words.hunt_difficulty())
				.detail(difficulty_name(&*words, difficulty))
				.quiet()
				.test_id("hunt-difficulty")
				.on_press(move || cycle()),
		);
	}
	column = column
		.child(gap(18))
		.child(aside(words.hunt_how_sigil()).class(format!("text-[{}]", ASH.hex())))
		.child(gap(22))
		.child(
			LedgerButton::new(words.hunt_begin())
				.test_id("hunt-begin")
				.on_press(fight),
		);
	ScrollView()
		.class("flex-1 w-full pt-safe pb-safe")
		.child(column)
}
