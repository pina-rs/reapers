//! Home: the bound reaper on its stage before its lamp, who it is and how
//! far it has come, what it has taken and what is loose, and one way out
//! into the hunt (the art bible, §9, `ui-sanctum.png`).
//!
//! The stage keeps the top of the screen and the reaper keeps the stage: no
//! word is printed over the model. Below the floor stroke the page scrolls
//! when it must (a short or turned phone); the primary and the tab bar stay
//! where the hand finds them.

use leptoskit::prelude::*;
use reapers_rules::PlayerState;
use reapers_rules::Progression;
use reapers_rules::Sighting;
use reapers_rules::Timestamp;

use super::open;
use crate::backend::KeeperKind;
use crate::l10n::use_l10n;
use crate::lookups::class_name;
use crate::platform;
use crate::routes::Hunt;
use crate::routes::Route;
use crate::seams::HuskPortraitProps;
use crate::seams::ReaperStageProps;
use crate::seams::husk_portrait;
use crate::seams::reaper_stage;
use crate::state::Game;
use crate::theme::ASH;
use crate::theme::KEY;
use crate::theme::PAPER;
use crate::theme::Rgb;
use crate::theme::Step;
use crate::theme::class;
use crate::theme::roman_numeral;
use crate::theme::seed_for;
use crate::theme::sol_amount;
use crate::theme::space;
use crate::theme::stroke;
use crate::ui::body;
use crate::ui::figure;
use crate::ui::fitted_display;
use crate::ui::ink::Mark;
use crate::ui::ink::Plate;
use crate::ui::ink::Recipe;
use crate::ui::kit::Primary;
use crate::ui::kit::Row;
use crate::ui::kit::Slip;
use crate::ui::kit::State;
use crate::ui::kit::Tab;
use crate::ui::kit::TabIcon;
use crate::ui::kit::class_mark;
use crate::ui::kit::divider;
use crate::ui::kit::lamp;
use crate::ui::kit::measure;
use crate::ui::kit::stamp;
use crate::ui::kit::tab_bar;
use crate::ui::kit::tally;
use crate::ui::label;
use crate::ui::light;

/// The stage's share of the screen's height. With the page below it and the
/// tab bar this puts the floor stroke where the board does on a phone, and
/// leaves the reaper well over the 340 points a phone must give it.
const STAGE_HEIGHT: &str = "h-[52vh]";

/// The least the page under the stage keeps when the screen is short (a
/// phone on its side): the stage gives way first.
const PAGE_LEAST: u32 = 128;

/// A husk ghost's side (§9).
const GHOST: u32 = 46;

/// The sanctum for `player`.
pub fn page(player: &PlayerState) -> Element {
	let game = Game::get();
	game.play(crate::sound::Score::Sanctum);
	let light = light();
	// The page follows the clock only while a husk is still surfacing.
	let untracked_now = game.now.get_untracked();
	let now = if player
		.sightings()
		.iter()
		.any(|sighting| !sighting.has_surfaced(untracked_now))
	{
		game.now.get()
	} else {
		untracked_now
	};
	let Some(weapon) = player.weapon() else {
		return View();
	};

	let column = View()
		.class(format!(
			"w-full max-w-[{}px] self-center px-5 pt-1 pb-4",
			space::COLUMN
		))
		.child(identity(player, light))
		.child(View().class("w-full pt-2 pb-2").child(divider(7301)))
		.child(taken(player, light))
		.child(loose(player, now))
		.child(elsewhere(player));

	let mut screen = View()
		.class(format!(
			"relative flex-1 min-h-screen w-full {}",
			class::VIGIL_BG
		))
		.test_id("sanctum")
		.child(
			View()
				.class(format!(
					"relative w-full {STAGE_HEIGHT} shrink overflow-hidden"
				))
				.child(lamp_behind(player, light))
				.child(floor(seed_for("sanctum-floor")))
				.child(reaper_stage(&ReaperStageProps::new(
					player.genome(),
					weapon,
					player.outfit(),
				)))
				.child(balance(player)),
		)
		.child(
			ScrollView()
				.class(format!("flex-1 w-full min-h-[{PAGE_LEAST}px]"))
				// The page opens at its top. The browser can hand this view the
				// scroll the screen before it left (the oath, scrolled to its
				// last button at a large text size); the offset is set once, so
				// the player's own scrolling is left alone after.
				.scroll_offset(leptoskit::native::ScrollOffset::new(0.0, 0.0))
				.test_id("sanctum-menu")
				.child(column),
		)
		.child(
			View()
				.class(format!(
					"w-full max-w-[{}px] self-center shrink-0 px-5 pt-2 pb-3",
					space::COLUMN
				))
				.child(go_hunting(player, now, light)),
		)
		.child(View().class("w-full shrink-0").child(tabs(light)));

	if let Some(sighted) = game.sighted.get()
		&& player
			.sightings()
			.iter()
			.any(|loose| loose.id == sighted.id)
	{
		screen = screen.child(
			View()
				.class("absolute left-0 right-0 top-0 items-center px-5 pt-safe-offset-14")
				.child(sighted_slip(&sighted)),
		);
	}
	screen
}

/// The lamp disc behind the reaper, a little right of the stage's centre
/// and a little above it, as the board sets it (centre 204, 224 of a
/// 390-point phone). Waxing with the experience to the next level.
fn lamp_behind(player: &PlayerState, light: Rgb) -> Element {
	let (width, height) = platform::viewport();
	// The board's disc on its stage: 132 points of a 462-point stage, and
	// never wider than a third of the screen either side.
	let radius = space::LAMP_RADIUS
		.min(height as f32 * 0.52 * 0.286)
		.min(width as f32 * 0.34);
	View()
		.class("absolute inset-0 items-center justify-center pl-[18px] pb-[14px]")
		.aria_hidden(true)
		.child(lamp(
			radius.round(),
			Progression::level_progress(player.xp()),
			light,
			seed_for("sanctum-lamp"),
		))
}

/// The floor the reaper stands on: a key stroke across the stage and, at
/// its feet, a cast shadow hatched in key.
fn floor(seed: u32) -> Element {
	const HEIGHT: f32 = 22.0;
	const LINE: f32 = 16.0;
	let (width, _) = platform::viewport();
	let span = (width as f32).min((space::COLUMN + 2 * space::MARGIN) as f32);
	// Half an ellipse standing on the floor, from 29% to 80% of the way
	// across, as the board's shadow falls.
	let (left, right) = (span * 0.29, span * 0.8);
	let (centre, half) = (left.midpoint(right), (right - left) / 2.0);
	let shadow: Vec<(f32, f32)> = (0..=16)
		.map(|step| {
			let angle = std::f32::consts::PI * step as f32 / 16.0;
			(centre - half * angle.cos(), LINE - 9.0 * angle.sin())
		})
		.collect();
	let plate = Plate::new(span, HEIGHT)
		.stroke(
			&Recipe::once(Mark::hatch(&shadow, 2.4, 60), seed.wrapping_add(1)),
			KEY,
			1.0,
		)
		.stroke(
			&Recipe::once(Mark::line(28.0, LINE, span - 30.0, LINE + 0.4), seed),
			KEY,
			stroke::EMPHASIS,
		);
	View()
		.class("absolute left-0 right-0 bottom-0 items-center")
		.aria_hidden(true)
		.child(
			View()
				.class(format!("relative w-[{span}px] h-[{HEIGHT}px]"))
				.child(plate.over()),
		)
}

/// The stage's top right corner: the SOL balance in Figures s3 (§9), as the
/// board prints it. It takes no press, so the stage still turns the reaper
/// under a finger everywhere else.
fn balance(player: &PlayerState) -> Element {
	let words = use_l10n();
	let balance = sol_amount(player.lamports());
	let balance = if Game::get().keeper_kind.get() == Some(KeeperKind::Simnet) {
		words.simnet_balance(&balance)
	} else {
		words.common_sol(&balance)
	};
	View()
		.class("absolute right-0 top-0 pr-5 pt-safe-offset-4")
		.child(figure(balance, Step::S3.size(), PAPER))
}

/// Who the reaper is: its class mark and class, its name in fitted Display
/// beside its level stamp, and how far its lamp has waxed.
fn identity(player: &PlayerState, light: Rgb) -> Element {
	const STAMP: f32 = 70.0;
	let words = use_l10n();
	let genome = player.genome();
	let level = player.level();
	let name_room = measure() - STAMP - space::S3 as f32;
	let lamp_line = if level >= Progression::LEVEL_CAP {
		words.sanctum_lamp_full(&roman_numeral(level))
	} else {
		let earned = player.xp() - Progression::xp_for_level(level);
		words.sanctum_lamp_waxing(
			earned,
			Progression::xp_to_next(level),
			&roman_numeral(level + 1),
		)
	};
	View()
		.class("w-full gap-2")
		.child(
			View()
				.class("w-full flex-row items-center gap-3")
				.child(
					View()
						.class("flex-1 gap-1")
						.child(
							View()
								.class("flex-row items-center gap-[6px]")
								.child(class_mark(light, seed_for("sanctum-class")))
								.child(label(
									class_name(&*words, genome.reaper_class),
									Step::S2.size(),
									PAPER,
								)),
						)
						.child(
							fitted_display(genome.name(), name_room, Step::S7, PAPER)
								.test_id("sanctum-name"),
						),
				)
				.child(stamp(
					&roman_numeral(level),
					light,
					seed_for("sanctum-stamp"),
				)),
		)
		.child(label(&lamp_line, Step::S2.size(), ASH))
}

/// What the reaper has taken: the tally in gates of five, as many as the
/// line holds, and the whole count beside it.
fn taken(player: &PlayerState, light: Rgb) -> Element {
	/// A gate of five and the room after it.
	const GATE: f32 = 27.5;
	/// The word "Taken", the count and the gaps between them.
	const BESIDE: f32 = 116.0;
	let words = use_l10n();
	let count = player.codex().len();
	let gates = ((measure() - BESIDE) / GATE).floor().max(1.0) as u32;
	View()
		.class("w-full flex-row items-center gap-4 min-h-[28px]")
		.test_id("sanctum-taken")
		.child(label(words.sanctum_taken(), Step::S2.size(), ASH))
		.child(
			View()
				.class("flex-1 overflow-hidden")
				.aria_hidden(true)
				.child(tally(
					u32::try_from(count).unwrap_or(u32::MAX),
					gates,
					light,
					seed_for("sanctum-tally"),
				)),
		)
		.child(figure(count.to_string(), Step::S4.size(), PAPER))
}

/// What is loose in the veil: up to three husk ghosts, a sentence naming
/// them, and a dashed ring for the next empty place.
fn loose(player: &PlayerState, now: Timestamp) -> Element {
	let words = use_l10n();
	let sightings = player.sightings();
	let names: Vec<String> = sightings
		.iter()
		.map(|sighting| sighting.nft.title())
		.collect();
	let sentence = match names.as_slice() {
		[] => words.sanctum_nothing_loose().to_owned(),
		[first] => words.sanctum_loose_one(first),
		[first, second] => words.sanctum_loose_two(first, second),
		[first, second, third, ..] => words.sanctum_loose_three(first, second, third),
	};
	let mut row = View().class("w-full flex-row items-center gap-3 pt-1");
	for sighting in sightings {
		row = row.child(ghost(sighting, now));
	}
	row = row.child(
		View()
			.class("flex-1")
			.child(body(sentence, Step::S3.size(), PAPER)),
	);
	if sightings.len() < PlayerState::MAX_SIGHTINGS {
		row = row.child(empty_place());
	}
	View()
		.class("w-full pt-3")
		.child(
			View()
				.class("w-full flex-row items-center gap-3")
				.child(label(words.sanctum_loose(), Step::S2.size(), ASH))
				.child(
					View()
						.class("flex-1 h-[4px] overflow-hidden")
						.aria_hidden(true)
						.child(divider(7302)),
				)
				.child(figure(
					format!("{} / {}", sightings.len(), PlayerState::MAX_SIGHTINGS),
					Step::S3.size(),
					PAPER,
				)),
		)
		.child(row)
}

/// One husk loose in the veil, as its portrait at 46 points: sighted, a
/// press hunts it; still surfacing, it says how long it has left.
fn ghost(sighting: &Sighting, now: Timestamp) -> Element {
	let words = use_l10n();
	let id = sighting.id;
	let title = sighting.nft.title();
	let portrait = husk_portrait(HuskPortraitProps {
		nft: sighting.nft,
		size: GHOST,
		spent: false,
	});
	let ghost = Pressable(title.clone())
		.class(format!("relative w-[{GHOST}px] h-[{GHOST}px]"))
		.test_id(format!("sanctum-sighting-{id}"))
		.child(portrait);
	if sighting.has_surfaced(now) {
		return ghost.on_press(move || {
			platform::tap();
			open(Hunt { sighting: id });
		});
	}
	let wait = sighting.surfaces_at.micros_since(now) / 1_000_000 + 1;
	ghost
		.accessibility_label(format!("{title}, {}", words.sanctum_surfacing(wait)))
		.enabled(false)
		.child(
			View()
				.class(format!(
					"absolute left-0 right-0 bottom-0 items-center bg-[{}]",
					KEY.hex_alpha(0.8)
				))
				.child(figure(
					words.sanctum_surfacing_soon(wait),
					Step::S1.size(),
					PAPER,
				)),
		)
}

/// The next empty place in the veil: a dashed ash ring with a plus. A press
/// opens the vault, where an NFT is released to fill it.
fn empty_place() -> Element {
	let words = use_l10n();
	let side = GHOST as f32;
	let centre = side / 2.0;
	let seed = seed_for("sanctum-empty-place");
	let ring = Plate::new(side, side)
		.dashed(
			&Recipe::once(Mark::circle(centre, centre, 15.0), seed),
			ASH,
			1.4,
			3.5,
			3.0,
		)
		.stroke(
			&Recipe::once(
				Mark::line(centre - 7.0, centre, centre + 7.0, centre),
				seed.wrapping_add(1),
			),
			ASH,
			1.4,
		)
		.stroke(
			&Recipe::once(
				Mark::line(centre, centre - 7.0, centre, centre + 7.0),
				seed.wrapping_add(2),
			),
			ASH,
			1.4,
		);
	Pressable(words.sanctum_loose_slot())
		.class(format!("relative w-[{GHOST}px] h-[{GHOST}px]"))
		.test_id("sanctum-release")
		.on_press(|| {
			platform::tap();
			open(Route::Vault);
		})
		.child(ring.over())
}

/// The primary (§9): "Go hunting", with how many husks are sighted nearby.
/// It hunts the first of them. With none sighted it sends the reaper after
/// a stray, so the one way out of the sanctum is never shut.
fn go_hunting(player: &PlayerState, now: Timestamp, light: Rgb) -> Element {
	let words = use_l10n();
	let sighted: Vec<_> = player
		.sightings()
		.iter()
		.filter(|sighting| sighting.has_surfaced(now))
		.map(|sighting| sighting.id)
		.collect();
	let primary = Primary::new(words.sanctum_go_hunting(), light).test_id("sanctum-hunt");
	match sighted.first().copied() {
		Some(first) => {
			primary
				.detail(words.sanctum_nearby(sighted.len() as i64))
				.on_press(move || open(Hunt { sighting: first }))
		}
		None => {
			primary
				.detail(words.sanctum_practice_detail())
				.on_press(|| open(Route::Practice))
		}
	}
}

/// The rest of what can be done from here, as rows under the page's fold:
/// the studio, the roster (a figure of one reaper or many), a practice hunt,
/// the colophon, and the simnet ledger in a simnet build. The board leaves
/// no room for them above the primary, and the tab bar carries the five
/// places.
fn elsewhere(player: &PlayerState) -> Element {
	let game = Game::get();
	let words = use_l10n();
	let mut rows = View()
		.class("w-full pt-4")
		.child(
			Row::new(words.sanctum_studio())
				.status(words.sanctum_studio_detail())
				.test_id("sanctum-studio")
				.on_press(|| open(Route::Studio)),
		)
		.child(
			Row::new(words.roster_title())
				.figure(words.sanctum_roster(player.reapers().len() as i64))
				.test_id("sanctum-roster")
				.on_press(|| open(Route::Roster)),
		)
		.child(
			Row::new(words.sanctum_practice())
				.status(words.sanctum_practice_detail())
				.test_id("sanctum-practice")
				.on_press(|| open(Route::Practice)),
		)
		.child(
			Row::new(words.sanctum_colophon())
				.test_id("sanctum-colophon")
				.on_press(|| open(Route::Colophon)),
		);
	if game.keeper_kind.get() == Some(KeeperKind::Simnet) {
		let count = game.ledger.get().len() as i64;
		rows = rows.child(
			Row::new(words.sanctum_simnet())
				.figure(words.sanctum_simnet_detail(count))
				.test_id("sanctum-simnet")
				.on_press(|| open(Route::Simnet)),
		);
	}
	rows
}

/// The tab bar, with the sanctum the page it is on.
fn tabs(light: Rgb) -> Element {
	let words = use_l10n();
	let tab = |icon, text: &str, id: &str, route: Option<Route>| {
		Tab {
			icon,
			label: text.to_owned(),
			test_id: id.to_owned(),
			active: route.is_none(),
			state: State::Rest,
			open: Box::new(move || {
				if let Some(route) = route.clone() {
					open(route);
				}
			}),
		}
	};
	tab_bar(
		vec![
			tab(TabIcon::Lamp, words.sanctum_overline(), "sanctum-tab", None),
			tab(
				TabIcon::Box,
				words.sanctum_vault(),
				"sanctum-vault",
				Some(Route::Vault),
			),
			tab(
				TabIcon::Blade,
				words.sanctum_armory(),
				"sanctum-armory",
				Some(Route::Armory),
			),
			tab(
				TabIcon::Book,
				words.sanctum_codex(),
				"sanctum-codex",
				Some(Route::Codex),
			),
			tab(
				TabIcon::Crack,
				words.sanctum_rifts(),
				"sanctum-rifts",
				Some(Route::Rifts),
			),
		],
		light,
	)
}

/// "Husk sighted": the slip that drops in when a released husk surfaces. A
/// press hunts it.
fn sighted_slip(sighting: &Sighting) -> Element {
	let words = use_l10n();
	let id = sighting.id;
	Slip::new(words.sanctum_sighted_banner(&sighting.nft.title()))
		.test_id("sanctum-sighted-banner")
		.on_press(move || open(Hunt { sighting: id }))
		.aria_live(leptoskit::native::Live::Polite)
}
