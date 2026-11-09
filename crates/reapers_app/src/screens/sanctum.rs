//! Home: the player's reaper, standing in the dark, and everything that can
//! be done from here.

use leptoskit::prelude::*;
use reapers_rules::PlayerState;
use reapers_rules::Progression;
use reapers_rules::Sighting;

use super::open;
use crate::backend::KeeperKind;
use crate::l10n::use_l10n;
use crate::lookups::class_name;
use crate::routes::Hunt;
use crate::routes::Route;
use crate::seams::ReaperStageProps;
use crate::seams::reaper_stage;
use crate::state::Game;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::CONTENT_WIDTH;
use crate::theme::roman_numeral;
use crate::theme::seed_for;
use crate::theme::sol_amount;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::display;
use crate::ui::figure;
use crate::ui::foot_scrim;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_heading;
use crate::ui::ledger_meter;
use crate::ui::ledger_pip;
use crate::ui::light;
use crate::ui::margin;
use crate::ui::sketch;

/// The sanctum for `player`.
pub fn page(player: &PlayerState) -> Element {
	let game = Game::get();
	let words = use_l10n();
	let light = light();
	let now = game.now.get();
	let genome = player.genome();
	let Some(weapon) = player.weapon() else {
		return View();
	};

	let mut list = View()
		.class(format!("w-full {CONTENT_WIDTH} self-center px-6 pt-5 pb-7"))
		.child(ledger_meter(
			words.common_level(),
			Progression::level_progress(player.xp()),
			light,
		))
		.child(ledger_heading(
			words.sanctum_loose(),
			Some(format!(
				"{}/{}",
				player.sightings().len(),
				PlayerState::MAX_SIGHTINGS
			)),
		));
	if player.sightings().is_empty() {
		list = list.child(body(words.sanctum_nothing_loose(), 14, ASH));
	}
	for sighting in player.sightings() {
		list = list.child(sighting_row(sighting, now));
	}
	list = list.child(gap(18));

	if game.keeper_kind.get() == Some(KeeperKind::Simnet) {
		let count = game.ledger.get().len() as i64;
		list = list
			.child(
				LedgerButton::new(words.sanctum_simnet())
					.detail(words.sanctum_simnet_detail(count))
					.test_id("sanctum-simnet")
					.on_press(|| open(Route::Simnet)),
			)
			.child(gap(8));
	}
	let entries = [
		(
			words.sanctum_roster(),
			words.roster_count(player.reapers().len() as i64),
			"sanctum-roster",
			Route::Roster,
		),
		(
			words.sanctum_studio(),
			words.sanctum_studio_detail().to_owned(),
			"sanctum-studio",
			Route::Studio,
		),
		(
			words.sanctum_vault(),
			words.sanctum_vault_detail(player.vault().len() as i64),
			"sanctum-vault",
			Route::Vault,
		),
		(
			words.sanctum_armory(),
			words.armory_anima(player.anima()),
			"sanctum-armory",
			Route::Armory,
		),
		(
			words.sanctum_codex(),
			words.sanctum_codex_detail(player.codex().len() as i64),
			"sanctum-codex",
			Route::Codex,
		),
		(
			words.sanctum_rifts(),
			words.sanctum_rifts_detail(player.rifts_sealed()),
			"sanctum-rifts",
			Route::Rifts,
		),
	];
	for (name, detail, id, route) in entries {
		list = list
			.child(
				LedgerButton::new(name)
					.detail(detail)
					.test_id(id)
					.on_press(move || open(route.clone())),
			)
			.child(gap(8));
	}
	list = list
		.child(
			LedgerButton::new(words.sanctum_practice())
				.detail(words.sanctum_practice_detail())
				.quiet()
				.test_id("sanctum-practice")
				.on_press(|| open(Route::Practice)),
		)
		.child(
			LedgerButton::new(words.sanctum_colophon())
				.detail(words.sanctum_colophon_detail())
				.quiet()
				.test_id("sanctum-colophon")
				.on_press(|| open(Route::Colophon)),
		);

	let balance = if game.keeper_kind.get() == Some(KeeperKind::Simnet) {
		words.simnet_balance(&sol_amount(player.lamports()))
	} else {
		words.common_sol(&sol_amount(player.lamports()))
	};
	let masthead =
		View()
			// Decorative text must let native scene touches through.
			.enabled(false)
			.class(format!(
				"flex-1 w-full {CONTENT_WIDTH} self-center px-6 pt-safe-offset-4 pb-[10px]"
			))
			.child(
				View()
					.class("flex-row items-center justify-between")
					.child(margin(words.sanctum_overline()))
					.child(figure(balance, 12, ASH)),
			)
			.child(View().class("flex-1"))
			.child(
				View()
					.class("flex-row items-end gap-4")
					.child(
						View()
							.class("flex-1 gap-[6px]")
							.child(label(class_name(&*words, genome.reaper_class), 12, light))
							.child(display(genome.name(), 30, BONE).test_id("sanctum-name")),
					)
					.child(
						display(roman_numeral(player.level()), 64, light).accessibility_label(
							format!("{} {}", words.common_level(), player.level()),
						),
					),
			);

	let mut screen = View()
		.class("relative flex-1 min-h-screen w-full")
		.test_id("sanctum")
		.child(
			View()
				.class("relative w-full h-[52vh] shrink-0")
				.child(reaper_stage(&ReaperStageProps::new(
					genome,
					weapon,
					player.outfit(),
				)))
				// The foot of the stage falls into the page's ink.
				.child(foot_scrim(40, 0xFF))
				.child(masthead),
		)
		.child(
			ScrollView()
				.class("flex-1 w-full pb-safe")
				.test_id("sanctum-menu")
				.child(list),
		);

	if let Some(sighted) = game.sighted.get()
		&& player
			.sightings()
			.iter()
			.any(|loose| loose.id == sighted.id)
	{
		screen = screen.child(
			View()
				.class("absolute left-0 right-0 top-0 items-center px-4 pt-safe-offset-2")
				.child(sighted_banner(&sighted)),
		);
	}
	screen
}

/// One husk loose in the veil: still surfacing, or sighted and huntable.
fn sighting_row(sighting: &Sighting, now: reapers_rules::Timestamp) -> Element {
	let words = use_l10n();
	let surfaced = sighting.has_surfaced(now);
	let id = sighting.id;
	let button = LedgerButton::new(sighting.nft.title()).test_id(format!("sanctum-sighting-{id}"));
	let button = if surfaced {
		button
			.detail(format!("{}  →", words.sanctum_hunt().to_uppercase()))
			.on_press(move || open(Hunt { sighting: id }))
	} else {
		let wait = sighting.surfaces_at.micros_since(now) / 1_000_000 + 1;
		button.detail(words.sanctum_surfacing(wait)).disabled()
	};
	View().class("pb-2").child(button)
}

/// "Husk sighted": the notice that drops in when a released husk surfaces.
fn sighted_banner(sighting: &Sighting) -> Element {
	let words = use_l10n();
	let light = light();
	let id = sighting.id;
	Pressable(words.sanctum_sighted_banner(&sighting.nft.title()))
		.class("relative flex-row items-center gap-3 px-4 py-[14px] bg-[#0f1116] max-w-[520px]")
		.test_id("sanctum-sighted-banner")
		.aria_live(leptoskit::native::Live::Polite)
		.on_press(move || open(Hunt { sighting: id }))
		.child(sketch(seed_for(&sighting.nft.id()), light, 1.0))
		.child(ledger_pip(true, light))
		.child(body(
			words.sanctum_sighted_banner(&sighting.nft.title()),
			14,
			BONE,
		))
}
