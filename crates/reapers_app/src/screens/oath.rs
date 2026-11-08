//! Where a player is bound to a reaper they did not choose.
//!
//! Three moments: the oath is read, the shuffle turns, and the reaper it
//! drew steps out of the dark.

use std::time::Duration;

use leptoskit::prelude::*;
use reapers_rules::Omen;
use reapers_rules::PlayerState;
use reapers_rules::REAPER_GIVEN_NAMES;

use super::home;
use crate::l10n::use_l10n;
use crate::lookups::class_line;
use crate::lookups::class_name;
use crate::lookups::omen_name;
use crate::lookups::perk_name;
use crate::lookups::weapon_name;
use crate::platform;
use crate::seams::ReaperStageProps;
use crate::seams::reaper_stage;
use crate::state::Game;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::CONTENT_WIDTH;
use crate::theme::reaper_light;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::display;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_back;
use crate::ui::ledger_row;
use crate::ui::margin;
use crate::widgets::reaper_stat_rows;

/// The shuffle turns for at least this long, however fast the keeper
/// answers, so the draw has weight.
const SUSPENSE: Duration = Duration::from_millis(1900);

/// How often the shuffle shows another name.
const SHUFFLE: Duration = Duration::from_millis(90);

/// What the oath screen is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Moment {
	Reading,
	/// The shuffle is turning, showing the name at this index.
	Turning(usize),
	Drawn,
}

/// The oath.
pub fn page() -> Element {
	let game = Game::get();
	let moment = moment_signal();
	match (moment.get(), game.player()) {
		(Moment::Drawn, Some(player)) if player.oath_taken() => drawn(&player),
		(Moment::Turning(index), _) => reading(game, moment, Some(index)),
		_ => reading(game, moment, None),
	}
}

/// The screen's own state, kept for as long as the app runs: a player who
/// leaves mid-shuffle comes back to it.
fn moment_signal() -> RwSignal<Moment> {
	thread_local! {
		static MOMENT: RwSignal<Moment> = RwSignal::new(Moment::Reading);
	}
	MOMENT.with(|moment| *moment)
}

/// Speaks the oath: asks the keeper to draw a reaper, and turns the shuffle
/// for at least [`SUSPENSE`] meanwhile.
fn speak(game: Game, moment: RwSignal<Moment>) {
	moment.set(Moment::Turning(0));
	platform::spawn(async move {
		let mut index = 0;
		let mut turned = Duration::ZERO;
		while turned < SUSPENSE && matches!(moment.get_untracked(), Moment::Turning(_)) {
			if !platform::sleep(SHUFFLE).await {
				break;
			}
			turned += SHUFFLE;
			index = (index + 7) % REAPER_GIVEN_NAMES.len();
			moment.set(Moment::Turning(index));
		}
	});
	game.change(
		|keeper| async move { keeper.take_oath().await },
		move |bound| {
			if bound {
				platform::thud();
				platform::spawn(async move {
					platform::sleep(SUSPENSE).await;
					moment.set(Moment::Drawn);
				});
			} else {
				moment.set(Moment::Reading);
			}
		},
	);
}

fn reading(game: Game, moment: RwSignal<Moment>, turning: Option<usize>) -> Element {
	let words = use_l10n();
	let mut column = View().class(format!(
		"flex-1 min-h-screen w-full {CONTENT_WIDTH} self-center px-7 pt-safe-offset-2 pb-safe-offset-7"
	));
	column = match turning {
		None => column.child(ledger_back(None)),
		Some(_) => column.child(gap(48)),
	};
	column = column
		.child(View().class("flex-1"))
		.child(margin(words.oath_overline()))
		.child(gap(12));
	column = match turning {
		Some(index) => {
			column.child(display(REAPER_GIVEN_NAMES[index], 64, ASH).test_id("oath-shuffle"))
		}
		None => {
			column
				.child(display(words.oath_title(), 44, BONE))
				.child(gap(18))
				.child(body(words.oath_body(), 15, ASH))
		}
	};
	column.child(gap(34)).child(match turning {
		Some(_) => {
			LedgerButton::new(words.oath_turning())
				.test_id("oath-speak")
				.disabled()
		}
		None => {
			LedgerButton::new(words.oath_speak())
				.test_id("oath-speak")
				.on_press(move || speak(game, moment))
		}
	})
}

/// The reaper the shuffle drew, shown for the first time.
fn drawn(player: &PlayerState) -> Element {
	let words = use_l10n();
	let genome = player.genome();
	let light = reaper_light(&genome);
	let reaper_class = genome.reaper_class;
	let rising = |delay: u32, element: Element| {
		element.class(format!(
			"starting:opacity-0 transition-opacity duration-700 delay-[{delay}ms]"
		))
	};
	let Some(weapon) = player.weapon() else {
		return View();
	};

	let mut details = View()
		.child(reaper_stat_rows(reaper_class.stats(), light))
		.child(ledger_row(
			perk_name(&*words, reaper_class.perk()),
			weapon_name(&*words, reaper_class.weapon()),
			false,
		));
	if genome.omen != Omen::None {
		details = details.child(ledger_row(
			words.oath_omen(),
			omen_name(&*words, genome.omen),
			true,
		));
	}

	View()
		.class("relative flex-1 min-h-screen w-full")
		.test_id("oath-drawn")
		.child(rising(
			0,
			View()
				.class("absolute inset-0")
				.child(reaper_stage(&ReaperStageProps {
					turns_on_its_own: true,
					..ReaperStageProps::new(genome.clone(), weapon, player.outfit())
				})),
		))
		// The foot of the stage darkens so the ledger over it stays
		// readable.
		.child(View().class("absolute left-0 right-0 bottom-0 h-[58%] bg-[#07080bf2]"))
		.child(
			View()
				.class(format!(
					"flex-1 w-full {CONTENT_WIDTH} self-center px-7 pt-safe-offset-6 pb-safe-offset-7"
				))
				.child(rising(700, margin(words.oath_drawn_overline())))
				.child(View().class("flex-1"))
				.child(rising(
					1100,
					label(class_name(&*words, reaper_class), 13, light),
				))
				.child(gap(8))
				.child(rising(
					1300,
					display(genome.name(), 40, BONE).test_id("oath-name"),
				))
				.child(gap(10))
				.child(rising(
					1500,
					body(class_line(&*words, reaper_class), 15, ASH),
				))
				.child(gap(14))
				.child(rising(1650, details))
				.child(gap(22))
				.child(rising(
					1700,
					LedgerButton::new(words.oath_enter())
						.test_id("oath-enter")
						.on_press(home),
				)),
		)
}
