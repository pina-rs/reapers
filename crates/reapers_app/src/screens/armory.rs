//! Where anima is spent: weapons that change how a hunt plays, and outfits
//! that only change how the reaper looks. Its arts are kept here too,
//! though nothing buys one: they are learned by absorbing husks.
//!
//! Whatever is tapped is tried on at once, on the reaper standing above the
//! list, before anything is bought.

use leptoskit::prelude::*;
use reapers_rules::Art;
use reapers_rules::ArtRules;
use reapers_rules::HuntRules;
use reapers_rules::NftCollection;
use reapers_rules::Outfit;
use reapers_rules::PlayerState;
use reapers_rules::ReaperClass;
use reapers_rules::ReaperState;
use reapers_rules::Temper;
use reapers_rules::Weapon;
use reapers_rules::WeaponKind;

use crate::l10n::use_l10n;
use crate::lookups::art_line;
use crate::lookups::art_name;
use crate::lookups::outfit_line;
use crate::lookups::outfit_name;
use crate::lookups::temper_name;
use crate::lookups::weapon_name;
use crate::lookups::weapon_trait;
use crate::seams::ReaperStageProps;
use crate::seams::reaper_stage;
use crate::state::Game;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::CONTENT_WIDTH;
use crate::theme::Rgb;
use crate::theme::class;
use crate::theme::roman_numeral;
use crate::ui::aside;
use crate::ui::body;
use crate::ui::figure;
use crate::ui::label;
use crate::ui::ledger_back;
use crate::ui::ledger_heading;
use crate::ui::ledger_pip;
use crate::ui::light;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
	Weapons,
	Arts,
	Outfits,
}

/// The armory's own state: which tab is open, and what the reaper above is
/// trying on. `None` means what it wields and wears.
#[derive(Clone, Copy)]
struct Fitting {
	tab: RwSignal<Tab>,
	weapon: RwSignal<Option<Weapon>>,
	outfit: RwSignal<Option<Outfit>>,
}

fn fitting() -> Fitting {
	thread_local! {
		static FITTING: Fitting = Fitting {
			tab: RwSignal::new(Tab::Weapons),
			weapon: RwSignal::new(None),
			outfit: RwSignal::new(None),
		};
	}
	FITTING.with(|fitting| *fitting)
}

/// Every kind of weapon, with the one this class was born holding first.
fn weapon_order(reaper_class: ReaperClass) -> Vec<WeaponKind> {
	std::iter::once(reaper_class.weapon())
		.chain(
			WeaponKind::ALL
				.into_iter()
				.filter(|kind| *kind != reaper_class.weapon()),
		)
		.collect()
}

/// The armory.
pub fn page() -> Element {
	let game = Game::get();
	let words = use_l10n();
	let Some(player) = game.player() else {
		return View();
	};
	let (Some(reaper), Some(wielded)) = (player.active_reaper().cloned(), player.weapon()) else {
		return View();
	};
	let fitting = fitting();
	let tried_weapon = fitting.weapon.get().unwrap_or(wielded);
	let tried_outfit = fitting.outfit.get().unwrap_or(player.outfit());
	let tab = fitting.tab.get();
	let light = light();

	let tab_button = |name: &str, id: &str, which: Tab| {
		let chosen = tab == which;
		let underline = if chosen {
			light.hex()
		} else {
			"#00000000".to_owned()
		};
		Pressable(name.to_owned())
			.class(format!(
				"min-h-[48px] justify-center border-b-2 border-[{underline}]"
			))
			.test_id(id.to_owned())
			.aria_selected(chosen)
			.child(label(name, 13, if chosen { BONE } else { ASH }))
			.on_press(move || fitting.tab.set(which))
	};

	let rows: Vec<Element> = match tab {
		Tab::Arts => {
			std::iter::once(
				View()
					.class("pt-[14px]")
					.child(aside(words.armory_art_note())),
			)
			.chain(
				Art::ALL
					.into_iter()
					.map(|art| art_row(game, art, &reaper, light)),
			)
			.collect()
		}
		Tab::Outfits => {
			std::iter::once(
				View()
					.class("pt-[14px]")
					.child(aside(words.armory_outfit_note())),
			)
			.chain(
				Outfit::ALL
					.into_iter()
					.map(|outfit| outfit_row(game, &player, outfit, tried_outfit == outfit, light)),
			)
			.collect()
		}
		Tab::Weapons => {
			let mut rows = Vec::new();
			for kind in weapon_order(player.genome().reaper_class) {
				let tempo = HuntRules::TICK_RATE as f64
					/ reapers_rules::weapon_profile(kind).swing_ticks as f64;
				rows.push(ledger_heading(
					weapon_name(&*words, kind),
					Some(words.armory_tempo(&format!("{tempo:.1}"))),
				));
				if let Some(trait_text) = weapon_trait(&*words, kind) {
					rows.push(View().class("pb-1").child(body(trait_text, 12, ASH)));
				}
				for temper in Temper::ALL {
					let weapon = Weapon::new(kind, temper);
					rows.push(weapon_row(
						game,
						&player,
						weapon,
						tried_weapon == weapon,
						light,
					));
				}
			}
			rows
		}
	};

	View()
		.class("flex-1 min-h-screen w-full")
		.test_id("armory")
		.child(
			View()
				.class("relative w-full h-[45vh] min-h-[300px]")
				.child(reaper_stage(&ReaperStageProps::new(
					player.genome(),
					tried_weapon,
					tried_outfit,
				)))
				.child(
					View()
						.class(
							"absolute left-0 right-0 top-0 flex-row items-start justify-between px-6 pt-safe-offset-2",
						)
						.child(ledger_back(Some(words.armory_overline())))
						.child(
							View().class("pt-4").child(
								figure(words.armory_anima(player.anima()), 14, BONE)
									.test_id("armory-anima"),
							),
						),
				),
		)
		.child(
			View()
				.class(format!("w-full {CONTENT_WIDTH} self-center flex-1"))
				.child(
					View()
						.class("flex-row gap-[22px] px-6")
						.child(tab_button(
							words.armory_weapons(),
							"armory-tab-weapons",
							Tab::Weapons,
						))
						.child(tab_button(
							words.armory_arts(),
							"armory-tab-arts",
							Tab::Arts,
						))
						.child(tab_button(
							words.armory_outfits(),
							"armory-tab-outfits",
							Tab::Outfits,
						)),
				)
				.child(View().class(format!("h-px w-full {}", class::HAIRLINE_BG)))
				.child(
					ScrollView()
						.class("flex-1 w-full")
						.child(View().class("px-6 pt-1 pb-7 pb-safe").children(rows)),
				),
		)
}

/// One thing the armory sells: press the row to try it on, press the action
/// to buy or take it up. A price the player cannot meet is still pressable,
/// so the refusal can say why, but it is not lit.
#[allow(
	clippy::too_many_arguments,
	reason = "one row, laid out from its parts"
)]
fn armory_row(
	id: String,
	title: &str,
	detail: String,
	action: String,
	tried: bool,
	on_try: impl Fn() + Send + Sync + 'static,
	on_action: Option<Box<dyn Fn() + Send + Sync>>,
	lit: bool,
	light: Rgb,
) -> Element {
	let action_colour = if on_action.is_some() && lit {
		light
	} else {
		ASH
	};
	let action_button = Pressable(action.clone())
		.class("min-h-[48px] min-w-[96px] items-end justify-center")
		.child(label(&action, 12, action_colour));
	let action_button = match on_action {
		Some(on_action) => action_button.on_press(move || on_action()),
		None => action_button.enabled(false),
	};
	View()
		.class("flex-row items-center gap-[14px] py-1")
		.test_id(id)
		.child(
			Pressable(title.to_owned())
				.class("flex-1 flex-row items-center gap-[14px]")
				.child(ledger_pip(tried, light))
				.child(
					View()
						.class("flex-1 gap-[2px]")
						.child(body(title, 15, BONE))
						.child(body(detail, 12, ASH)),
				)
				.on_press(on_try),
		)
		.child(action_button)
}

fn weapon_row(
	game: Game,
	player: &PlayerState,
	weapon: Weapon,
	tried: bool,
	light: Rgb,
) -> Element {
	let words = use_l10n();
	let owned = player.weapons().contains(&weapon);
	let price = weapon.price_for(player.genome().reaper_class);
	let reachable = player.level() >= weapon.level_required();
	let strike = format!("{:.2}", weapon.profile().strike * weapon.temper.strike());
	let buy = move || {
		game.change(
			move |keeper| async move { keeper.buy_weapon(weapon).await },
			|_| {},
		)
	};
	let wield = move || {
		game.change(
			move |keeper| async move { keeper.wield(weapon).await },
			|_| {},
		)
	};
	let (action, on_action): (String, Option<Box<dyn Fn() + Send + Sync>>) =
		if player.weapon() == Some(weapon) {
			(words.armory_wielded().to_owned(), None)
		} else if owned {
			(words.armory_wield().to_owned(), Some(Box::new(wield)))
		} else if !reachable {
			(
				words.armory_needs_level(&roman_numeral(weapon.level_required())),
				None,
			)
		} else if price == 0 {
			(words.armory_free().to_owned(), Some(Box::new(buy)))
		} else {
			(words.armory_anima(price), Some(Box::new(buy)))
		};
	armory_row(
		format!("armory-weapon-{}", weapon.id()),
		temper_name(&*words, weapon.temper),
		words.armory_strike(&strike),
		action,
		tried,
		move || fitting().weapon.set(Some(weapon)),
		on_action,
		owned || player.anima() >= price,
		light,
	)
}

fn outfit_row(
	game: Game,
	player: &PlayerState,
	outfit: Outfit,
	tried: bool,
	light: Rgb,
) -> Element {
	let words = use_l10n();
	let owned = player.outfits().contains(&outfit);
	let reachable = player.level() >= outfit.level_required();
	let buy = move || {
		game.change(
			move |keeper| async move { keeper.buy_outfit(outfit).await },
			|_| {},
		)
	};
	let wear = move || {
		game.change(
			move |keeper| async move { keeper.wear(outfit).await },
			|_| {},
		)
	};
	let (action, on_action): (String, Option<Box<dyn Fn() + Send + Sync>>) =
		if player.outfit() == outfit {
			(words.armory_worn().to_owned(), None)
		} else if owned {
			(words.armory_wear().to_owned(), Some(Box::new(wear)))
		} else if !reachable {
			(
				words.armory_needs_level(&roman_numeral(outfit.level_required())),
				None,
			)
		} else {
			(words.armory_anima(outfit.price()), Some(Box::new(buy)))
		};
	View().class("pt-2").child(armory_row(
		format!("armory-outfit-{}", outfit.name()),
		outfit_name(&*words, outfit),
		outfit_line(&*words, outfit).to_owned(),
		action,
		tried,
		move || fitting().outfit.set(Some(outfit)),
		on_action,
		owned || player.anima() >= outfit.price(),
		light,
	))
}

/// One art: what it does, how deep the reaper knows it, and whether it is
/// the one the reaper throws.
fn art_row(game: Game, art: Art, reaper: &ReaperState, light: Rgb) -> Element {
	let words = use_l10n();
	let rank = reaper.art_rank(art);
	let absorbed = reaper
		.codex()
		.iter()
		.filter(|entry| art.vessels().contains(&entry.nft.collection.vessel()))
		.count();
	// How many more it takes to go deeper, once the art is known at all.
	let next = (rank > 0 && rank < ArtRules::MAX_RANK)
		.then(|| ArtRules::ABSORBED_FOR_RANK[rank as usize] as i64 - absorbed as i64);
	let carriers = NftCollection::ALL
		.into_iter()
		.filter(|collection| art.vessels().contains(&collection.vessel()))
		.map(NftCollection::title)
		.collect::<Vec<_>>()
		.join(", ");
	let mut facts = Vec::new();
	if rank > 0 {
		facts.push(words.armory_art_rank(rank));
	}
	if let Some(next) = next {
		facts.push(words.armory_art_next(next));
	}
	facts.push(words.armory_art_source(&carriers));

	let ready = reaper.art() == Some(art);
	let (action, colour) = if ready {
		(words.armory_art_ready(), ASH)
	} else if rank > 0 {
		(words.armory_art_take(), light)
	} else {
		(words.armory_art_unlearned(), ASH)
	};
	let action_button = Pressable(action.to_owned())
		.class("min-h-[48px] min-w-[96px] items-end pt-1")
		.child(label(action, 12, colour));
	let action_button = if !ready && rank > 0 {
		action_button.on_press(move || {
			game.change(
				move |keeper| async move { keeper.attune(art).await },
				|_| {},
			)
		})
	} else {
		action_button.enabled(false)
	};

	View()
		.class("flex-row items-start gap-[14px] pt-4")
		.test_id(format!("armory-art-{}", art.name()))
		.child(View().class("pt-[6px]").child(ledger_pip(ready, light)))
		.child(
			View()
				.class("flex-1 gap-[2px]")
				.child(body(
					art_name(&*words, art),
					15,
					if rank > 0 { BONE } else { ASH },
				))
				.child(body(art_line(&*words, art), 12, ASH))
				.child(body(facts.join(" · "), 11, ASH)),
		)
		.child(action_button)
}
