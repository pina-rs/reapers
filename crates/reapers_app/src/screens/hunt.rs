//! A hunt, as the books see it: the brief it is played from, the keeper's
//! verdict when it ends, and what it came to.
//!
//! The hunt itself is [`hunt_view`], the seam the camera and the 3D husk
//! plug into. This screen gives it a brief and settles what it reports.

use std::sync::Arc;

use leptoskit::prelude::*;
use reapers_rules::Art;
use reapers_rules::EscapeCause;
use reapers_rules::Feeding;
use reapers_rules::HuntEnd;
use reapers_rules::HuntLoadout;
use reapers_rules::HuntLog;
use reapers_rules::HuntOutcome;
use reapers_rules::PlayerState;
use reapers_rules::Rarity;
use reapers_rules::Refusal;
use reapers_rules::SoulPath;
use reapers_rules::SoulRules;
use reapers_rules::market_listings;

use super::back;
use crate::backend::KeeperKind;
use crate::l10n::use_l10n;
use crate::lookups::art_name;
use crate::lookups::feeding_name;
use crate::lookups::gift_name;
use crate::lookups::grade_name;
use crate::lookups::refusal_message;
use crate::lookups::soul_path_name;
use crate::platform;
use crate::seams::HuntBrief;
use crate::seams::HuntReward;
use crate::seams::HuntViewProps;
use crate::seams::HuskPortraitProps;
use crate::seams::autoplay_log;
use crate::seams::hunt_view;
use crate::seams::husk_portrait;
use crate::sound::Score;
use crate::state::Game;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::CONTENT_WIDTH;
use crate::theme::WOUND;
use crate::theme::roman_numeral;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::display;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_notice;
use crate::ui::ledger_row;
use crate::ui::light;

/// Where a hunt has got to.
#[derive(Clone, Debug, PartialEq)]
enum Stage {
	/// Being fought, from this brief.
	Fighting(HuntBrief),
	/// Over, and with the keeper.
	Settling(HuntBrief),
	/// What it came to.
	Done(HuntBrief, HuntReward),
}

fn stage() -> RwSignal<Option<Stage>> {
	thread_local! {
		static STAGE: RwSignal<Option<Stage>> = RwSignal::new(None);
	}
	STAGE.with(|stage| *stage)
}

const fn brief_of(stage: &Stage) -> &HuntBrief {
	match stage {
		Stage::Fighting(brief) | Stage::Settling(brief) | Stage::Done(brief, _) => brief,
	}
}

/// The hunt for the loose husk `sighting_id`.
pub fn page(sighting_id: i64) -> Element {
	let game = Game::get();
	let words = use_l10n();
	let stage = stage();
	let current = stage.get();
	if let Some(current) =
		current.filter(|current| brief_of(current).sighting_id == Some(sighting_id))
	{
		return show(game, current);
	}
	let Some(player) = game.player() else {
		return View();
	};
	let Some(sighting) = player
		.sightings()
		.iter()
		.find(|sighting| sighting.id == sighting_id)
		.copied()
	else {
		return ledger_notice(
			words.common_refused_title(),
			refusal_message(&*words, Refusal::UnknownSighting),
			words.common_back(),
			back,
		);
	};
	let brief = HuntBrief {
		nft: sighting.nft,
		seed: sighting.seed,
		loadout: player.loadout(),
		sighting_id: Some(sighting.id),
	};
	stage.set(Some(Stage::Fighting(brief)));
	show(game, Stage::Fighting(brief))
}

/// A practice hunt: a stray husk, the reaper's own strength, and nothing
/// gained or lost.
pub fn practice() -> Element {
	let game = Game::get();
	let stage = stage();
	if let Some(current) = stage
		.get()
		.filter(|current| brief_of(current).is_practice())
	{
		return show(game, current);
	}
	let brief = stray(&game.player());
	stage.set(Some(Stage::Fighting(brief)));
	show(game, Stage::Fighting(brief))
}

/// A stray for a practice hunt, drawn from the time: a practice decides
/// nothing, so it needs no secure dice.
fn stray(player: &Option<PlayerState>) -> HuntBrief {
	let strays: Vec<_> = market_listings()
		.into_iter()
		.filter(|nft| nft.rarity() <= Rarity::Uncommon)
		.collect();
	let draw = reapers_rules::mix_seed(&[platform::now().micros(), 0x5EED]);
	HuntBrief {
		nft: strays[(draw.unsigned_abs() as usize) % strays.len()],
		seed: draw & 0xFFFF_FFFF,
		loadout: player
			.as_ref()
			.filter(|player| player.oath_taken())
			.map_or(HuntLoadout::BORROWED, PlayerState::loadout),
		sighting_id: None,
	}
}

fn show(game: Game, stage: Stage) -> Element {
	game.play(Score::Hunt);
	match stage {
		Stage::Fighting(brief) => {
			let on_difficulty = game
				.flags
				.testing_difficulty()
				.then(|| Arc::new(move || game.cycle_difficulty()) as Arc<dyn Fn() + Send + Sync>);
			let props = HuntViewProps {
				brief,
				difficulty: game.difficulty.get(),
				autoplay: game.flags.autoplay,
				on_finished: Arc::new(move |log, outcome| settle(game, brief, log, outcome)),
				on_difficulty,
			};
			if props.autoplay {
				let difficulty = props.difficulty;
				platform::spawn(async move {
					if let Some((log, outcome)) = autoplay_log(&brief, difficulty) {
						settle(game, brief, log, outcome);
					}
				});
			}
			hunt_view(&props)
		}
		Stage::Settling(_) => {
			View()
				.class("flex-1 min-h-screen items-center justify-center")
				.child(
					Status(use_l10n().common_loading())
						.class(format!("text-[15px] text-[{}]", ASH.hex())),
				)
		}
		Stage::Done(brief, reward) => result(game, &brief, &reward),
	}
}

/// Hands a finished hunt to the keeper, which judges it by replaying the
/// log, and keeps what it says.
fn settle(game: Game, brief: HuntBrief, log: HuntLog, outcome: HuntOutcome) {
	let stage = stage();
	let Some(sighting_id) = brief.sighting_id else {
		stage.set(Some(Stage::Done(brief, HuntReward::practice(outcome))));
		return;
	};
	let Some(before) = game.player() else {
		return;
	};
	stage.set(Some(Stage::Settling(brief)));
	let art = Art::carried_by(brief.nft.collection.vessel());
	game.attempt(
		move |keeper| async move { keeper.settle_hunt(sighting_id, &log).await },
		move |settlement| {
			let Some(settlement) = settlement else {
				stage.set(Some(Stage::Fighting(brief)));
				return;
			};
			let after = &settlement.state;
			let signed = settlement.outcome.absorbed()
				&& after
					.active_reaper()
					.and_then(|reaper| reaper.codex().last())
					.is_some_and(|entry| entry.signature.is_some());
			let rank_of = |player: &PlayerState, art: Option<Art>| {
				art.zip(player.active_reaper())
					.map_or(0, |(art, reaper)| reaper.art_rank(art))
			};
			let reward = HuntReward {
				outcome: settlement.outcome,
				xp: settlement.xp,
				anima: settlement.anima,
				feeding: settlement.feeding,
				signed,
				level_before: before.level(),
				level_after: after.level(),
				gift: before.active_reaper().map(|reaper| reaper.soul().gift),
				soul_rank_before: before
					.active_reaper()
					.map_or(0, reapers_rules::ReaperState::soul_rank),
				soul_rank_after: after
					.active_reaper()
					.map_or(0, reapers_rules::ReaperState::soul_rank),
				art,
				art_rank_before: rank_of(&before, art),
				art_rank_after: rank_of(after, art),
			};
			game.books.set(crate::state::Books::Open(settlement.state));
			stage.set(Some(Stage::Done(brief, reward)));
		},
	);
}

/// Leaves the hunt, and forgets it, so the next is fought afresh.
fn leave(game: Game) {
	stage().set(None);
	game.play(Score::Sanctum);
	back();
}

/// What the hunt came to.
fn result(game: Game, brief: &HuntBrief, reward: &HuntReward) -> Element {
	let words = use_l10n();
	let light = light();
	let outcome = reward.outcome;
	let (title, detail) = match outcome.end {
		HuntEnd::Absorbed(grade) => {
			(
				grade_name(&*words, grade).to_owned(),
				if brief.is_practice() {
					words.hunt_practice_done().to_owned()
				} else {
					words.hunt_absorbed(&brief.nft.title())
				},
			)
		}
		HuntEnd::Escaped(cause) => {
			(
				words.hunt_escaped_title().to_owned(),
				match cause {
					EscapeCause::Resolve => words.hunt_escaped_resolve(),
					EscapeCause::Time => words.hunt_escaped_time(),
					EscapeCause::Reap => words.hunt_escaped_reap(),
				}
				.to_owned(),
			)
		}
	};
	let absorbed = outcome.absorbed();

	let mut column = View().class(format!(
		"flex-1 min-h-screen w-full {CONTENT_WIDTH} self-center justify-end p-7 pt-safe-offset-7 pb-safe-offset-7"
	));
	if absorbed && !brief.is_practice() {
		column = column.child(View().class("pb-7").child(husk_portrait(HuskPortraitProps {
			nft: brief.nft,
			size: 168,
			spent: true,
		})));
	}
	column = column
		.child(label(
			if absorbed {
				words.hunt_reaped_label()
			} else {
				words.hunt_lost_label()
			},
			11,
			if absorbed { ASH } else { WOUND },
		))
		.child(gap(10))
		.child(display(title, 52, BONE).test_id("hunt-result"))
		.child(gap(14))
		.child(body(detail, 15, ASH));

	if brief.is_practice() {
		column = column
			.child(gap(20))
			.child(ledger_row(
				words.hunt_practice_holds(),
				outcome.deeds.holds.to_string(),
				false,
			))
			.child(ledger_row(
				words.hunt_practice_parries(),
				outcome.deeds.parries.to_string(),
				false,
			))
			.child(ledger_row(
				words.hunt_practice_cadence(),
				outcome.deeds.max_cadence.to_string(),
				false,
			));
	}
	if reward.xp > 0 {
		column = column
			.child(gap(26))
			.child(ledger_row(
				words.common_experience(),
				format!("+{}", reward.xp),
				false,
			))
			.child(ledger_row(
				words.common_anima(),
				format!("+{}", reward.anima),
				false,
			));
		if let Some(feeding) = reward.feeding {
			column = column
				.child(ledger_row(
					words.hunt_feeding(),
					feeding_name(&*words, feeding),
					feeding == Feeding::Feast,
				))
				.child(ledger_row(
					words.hunt_receipt(),
					if !reward.signed {
						words.hunt_receipt_unsigned()
					} else if game.keeper_kind.get() == Some(KeeperKind::Simnet) {
						words.hunt_receipt_simnet()
					} else {
						words.hunt_receipt_signed()
					},
					false,
				));
		}
		if outcome.ambushed {
			column = column.child(ledger_row(
				words.hunt_ambush(),
				words.hunt_ambush_bonus(),
				false,
			));
		}
		if reward.levelled_up() {
			column = column.child(ledger_row(
				words.common_level(),
				roman_numeral(reward.level_after),
				true,
			));
		}
		if let Some(gift) = reward.gift
			&& reward.soul_rank_after > reward.soul_rank_before
		{
			column = column.child(gap(14)).child(
				Status(words.soul_rank_earned(gift_name(&*words, gift), reward.soul_rank_after))
					.class(format!("text-[15px] text-[{}]", light.hex())),
			);
		}
		if let Some(art) = reward.art
			&& reward.art_rank_after > reward.art_rank_before
		{
			let learned = if reward.art_rank_before == 0 {
				words.hunt_art_learned(art_name(&*words, art))
			} else {
				words.hunt_art_deepened(art_name(&*words, art), reward.art_rank_after)
			};
			column = column.child(gap(14)).child(
				Status(learned)
					.class(format!("text-[15px] text-[{}]", light.hex()))
					.test_id("hunt-art-learned"),
			);
		}
		let mut paths = Vec::new();
		if outcome.ambushed {
			paths.push(soul_path_name(&*words, SoulPath::Unseen));
		}
		if outcome.deeds.holds > 0 {
			paths.push(soul_path_name(&*words, SoulPath::Stillness));
		}
		if outcome.deeds.max_cadence >= SoulRules::CADENCE_FOR_MARK {
			paths.push(soul_path_name(&*words, SoulPath::Rhythm));
		}
		if !paths.is_empty() {
			column = column.child(gap(12)).child(body(
				words.soul_deeds_earned(&paths.join(", ")),
				13,
				ASH,
			));
		}
	}
	column = column.child(gap(30));
	if brief.is_practice() {
		column = column
			.child(
				LedgerButton::new(words.hunt_practice_again())
					.test_id("hunt-practice-again")
					.on_press(move || {
						let brief = stray(&game.player());
						stage().set(Some(Stage::Fighting(brief)));
					}),
			)
			.child(gap(8))
			.child(
				LedgerButton::new(words.hunt_leave())
					.quiet()
					.test_id("hunt-leave")
					.on_press(move || leave(game)),
			);
	} else {
		column = column.child(
			LedgerButton::new(words.hunt_leave())
				.test_id("hunt-leave")
				.on_press(move || leave(game)),
		);
	}
	ScrollView().class("flex-1 w-full").child(column)
}
