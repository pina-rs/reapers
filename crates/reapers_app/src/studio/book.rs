//! The Book of Becoming: the design decisions, read beside the very rules
//! they explain, with unsaved illustrations. The Flutter `DecisionLedger`.
//!
//! Its draws take fresh entropy from the platform; every figure is the
//! rules' own.

use leptoskit::prelude::*;
use reapers_rules::CaptureGrade;
use reapers_rules::Feeding;
use reapers_rules::GenomeChanges;
use reapers_rules::HuntDeeds;
use reapers_rules::HuntOutcome;
use reapers_rules::Omen;
use reapers_rules::Outfit;
use reapers_rules::PlayerState;
use reapers_rules::Progression;
use reapers_rules::Rarity;
use reapers_rules::ReaperClass;
use reapers_rules::ReaperGenome;
use reapers_rules::ReaperGift;
use reapers_rules::ReaperSoul;
use reapers_rules::SoulMemory;
use reapers_rules::SoulPath;
use reapers_rules::SoulRules;
use reapers_rules::Temper;
use reapers_rules::Temperament;

use super::controls::chip;
use super::controls::chips;
use super::controls::slider_row;
use super::controls::switch_row;
use super::dart_double;
use crate::l10n::use_l10n;
use crate::lookups::class_line;
use crate::lookups::class_name;
use crate::lookups::feeding_name;
use crate::lookups::gift_name;
use crate::lookups::grade_name;
use crate::lookups::omen_name;
use crate::lookups::outfit_name;
use crate::lookups::perk_line;
use crate::lookups::perk_name;
use crate::lookups::rarity_name;
use crate::lookups::soul_path_name;
use crate::lookups::temper_name;
use crate::lookups::temperament_name;
use crate::platform;
use crate::screens::soul::soul_ledger;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::class;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::display;
use crate::ui::figure;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_heading;
use crate::ui::ledger_row;
use crate::ui::margin;
use crate::ui::prose;

/// The book's four chapters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Chapter {
	#[default]
	Evolution,
	Chance,
	Nfts,
	Decisions,
}

impl Chapter {
	pub const ALL: [Self; 4] = [Self::Evolution, Self::Chance, Self::Nfts, Self::Decisions];
}

/// Where the reader has got to, and what they are trying.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Book {
	pub chapter: Chapter,
	pub calling: ReaperClass,
	pub gift: ReaperGift,
	pub level: i64,
	pub marks: i64,
	pub temperament: Temperament,
	pub rarity: Rarity,
	pub grade: CaptureGrade,
	pub ambush: bool,
	/// The feeding last rolled for this scenario.
	pub feeding: Option<Feeding>,
	/// The reaper last drawn.
	pub birth: Option<ReaperGenome>,
}

impl Default for Book {
	fn default() -> Self {
		Self {
			chapter: Chapter::Evolution,
			calling: ReaperClass::Harvester,
			gift: ReaperGift::Furrow,
			level: 1,
			marks: 0,
			temperament: Temperament::Patient,
			rarity: Rarity::Common,
			grade: CaptureGrade::Great,
			ambush: false,
			feeding: None,
			birth: None,
		}
	}
}

impl Book {
	/// The borrowed reaper, born of the chosen calling, gift and
	/// temperament. Its given name and epithet are zero, so the gaze picks
	/// the gift.
	pub fn genome(&self) -> ReaperGenome {
		let borrowed = ReaperGenome::BORROWED;
		borrowed
			.with(GenomeChanges {
				reaper_class: Some(self.calling),
				gaze: Some((self.gift.index() % SoulRules::GIFTS_PER_CLASS) as u32),
				tatter: Some(borrowed.tatter + self.temperament.index() as u32),
				..GenomeChanges::default()
			})
			.unwrap_or(borrowed)
	}

	/// The marks chosen, all along the gift's own path.
	pub fn memory(&self) -> SoulMemory {
		let path = self.gift.path();
		let marks = |of| if path == of { self.marks } else { 0 };
		SoulMemory {
			unseen: marks(SoulPath::Unseen),
			stillness: marks(SoulPath::Stillness),
			rhythm: marks(SoulPath::Rhythm),
		}
	}

	pub fn choose_calling(&mut self, calling: ReaperClass) {
		self.calling = calling;
		if let Some(gift) = ReaperGift::ALL
			.into_iter()
			.find(|gift| gift.reaper_class() == calling)
		{
			self.gift = gift;
		}
	}

	/// The scenario's absorption.
	pub const fn outcome(&self) -> HuntOutcome {
		HuntOutcome::absorbed_with(
			self.grade,
			self.ambush,
			0,
			HuntDeeds {
				holds: 0,
				parries: 0,
				max_cadence: 0,
			},
		)
	}

	/// Each feeding's chance in percent, its share and the XP it pays.
	pub fn feedings(&self) -> Vec<(Feeding, f64, i64)> {
		let total: u32 = Feeding::ALL.iter().map(|feeding| feeding.weight()).sum();
		let outcome = self.outcome();
		Feeding::ALL
			.into_iter()
			.map(|feeding| {
				(
					feeding,
					f64::from(feeding.weight()) * 100.0 / f64::from(total),
					outcome.xp_for(self.rarity, feeding),
				)
			})
			.collect()
	}

	/// The XP the scenario pays on average: each rounded reward by its
	/// weight.
	pub fn expected_xp(&self) -> f64 {
		self.feedings()
			.into_iter()
			.map(|(_, chance, xp)| xp as f64 * chance / 100.0)
			.sum()
	}

	pub fn anima(&self) -> i64 {
		self.outcome().anima_for(self.rarity)
	}

	/// Each omen's chance at birth, in percent.
	pub fn omens() -> Vec<(Omen, f64)> {
		let total: u32 = Omen::ALL.iter().map(|omen| omen.weight()).sum();
		Omen::ALL
			.into_iter()
			.map(|omen| (omen, f64::from(omen.weight()) * 100.0 / f64::from(total)))
			.collect()
	}

	/// Changes the scenario, which forgets the feeding rolled for the last.
	pub fn scenario(&mut self, change: impl FnOnce(&mut Self)) {
		change(self);
		self.feeding = None;
	}
}

/// The book's state, kept between renders.
pub fn book() -> RwSignal<Book> {
	thread_local! {
		static BOOK: RwSignal<Book> = crate::ui::lasting(|| RwSignal::new(Book::default()));
	}
	BOOK.with(|book| *book)
}

fn turn(change: impl FnOnce(&mut Book) + 'static) {
	book().update(change);
}

/// Draws fresh entropy, then hands it to `change`.
fn draw(change: impl FnOnce(&mut Book, i64) + 'static) {
	platform::spawn(async move {
		match platform::random_seed().await {
			Ok(seed) => turn(move |book| change(book, seed)),
			Err(error) => platform::log(&format!("the book could not draw: {error}")),
		}
	});
}

/// The book, with `on_close` its way back to the studio.
pub fn book_view(on_close: impl Fn() + Send + Sync + 'static) -> Element {
	let words = use_l10n();
	let state = book().get();
	let names = [
		words.ledger_evolution(),
		words.ledger_chance(),
		words.ledger_nfts(),
		words.ledger_decisions(),
	];
	let chapter = match state.chapter {
		Chapter::Evolution => evolution(&state),
		Chapter::Chance => chance(&state),
		Chapter::Nfts => nft_future(),
		Chapter::Decisions => decisions(),
	};
	View()
		.class("w-full max-w-[720px] self-center px-6 pt-safe-offset-2 pb-16")
		.test_id("studio-book")
		.child(
			Pressable(words.common_back().to_owned())
				.class("min-w-[48px] min-h-[48px] justify-center self-start")
				.test_id("book-back")
				.on_press(move || {
					platform::tap();
					on_close();
				})
				.child(label(
					&format!("←  {}", words.studio_app_title().to_uppercase()),
					13,
					BONE,
				)),
		)
		.child(gap(10))
		.child(display(words.ledger_title(), 40, BONE))
		.child(gap(18))
		.child(body(words.ledger_intro(), 17, BONE))
		.child(gap(24))
		.child(chips(
			Chapter::ALL
				.into_iter()
				.zip(names)
				.enumerate()
				.map(|(index, (which, name))| {
					chip(
						name,
						state.chapter == which,
						format!("ledger-chapter-{index}"),
						Some(Box::new(move || turn(move |book| book.chapter = which))),
					)
				})
				.collect(),
		))
		.child(gap(18))
		.child(body(words.ledger_current(), 13, ASH))
		.child(View().class(format!("h-px w-full my-5 {}", class::HAIRLINE_BG)))
		.child(chapter)
}

/// A heading and a readable entry.
fn entry(title: &str, text: impl Into<String>) -> Element {
	View()
		.class("pt-6 pb-4 gap-3")
		.child(display(title, 28, BONE))
		.child(prose(text))
}

fn evolution(state: &Book) -> Element {
	let words = use_l10n();
	let gift = state.gift;
	let calling = state.calling;
	let strike = |level: i64| 1.0 + Progression::STRIKE_PER_LEVEL * (level - 1) as f64;

	let mut column = View()
		.child(prose(words.ledger_traits()))
		.child(ledger_heading(words.ledger_calling(), None))
		.child(chips(
			ReaperClass::ALL
				.into_iter()
				.map(|office| {
					chip(
						class_name(&*words, office),
						calling == office,
						format!("ledger-calling-{}", office.name()),
						Some(Box::new(move || {
							turn(move |book| book.choose_calling(office));
						})),
					)
				})
				.collect(),
		))
		.child(entry(
			class_name(&*words, calling),
			format!(
				"{}\n\n{}: {}",
				class_line(&*words, calling),
				perk_name(&*words, calling.perk()),
				perk_line(&*words, calling.perk()),
			),
		))
		.child(ledger_heading(words.ledger_gift(), None))
		.child(chips(
			ReaperGift::ALL
				.into_iter()
				.filter(|candidate| candidate.reaper_class() == calling)
				.map(|candidate| {
					chip(
						gift_name(&*words, candidate),
						gift == candidate,
						format!("ledger-gift-{}", candidate.name()),
						Some(Box::new(move || turn(move |book| book.gift = candidate))),
					)
				})
				.collect(),
		))
		.child(gap(22))
		.child(chips(
			Temperament::ALL
				.into_iter()
				.map(|item| {
					chip(
						temperament_name(&*words, item),
						state.temperament == item,
						format!("ledger-temperament-{}", item.name()),
						Some(Box::new(move || turn(move |book| book.temperament = item))),
					)
				})
				.collect(),
		))
		.child(gap(22))
		.child(body(words.ledger_level_help(), 15, ASH))
		.child(slider_row(
			words.ledger_level(),
			Some(state.level.to_string()),
			state.level as f64,
			(1.0, Progression::LEVEL_CAP as f64, 1.0),
			"ledger-level",
			words.studio_level_value(state.level),
			|level| turn(move |book| book.level = level.round() as i64),
		))
		.child(prose(words.ledger_path_marks()))
		.child(gap(8))
		.child(chips(
			SoulRules::RANK_MARKS
				.into_iter()
				.map(|count| {
					chip(
						&format!("{count} {}", soul_path_name(&*words, gift.path())),
						state.marks == count,
						format!("ledger-marks-{count}"),
						Some(Box::new(move || turn(move |book| book.marks = count))),
					)
				})
				.collect(),
		))
		.child(gap(16))
		.child(ledger_row(
			words.ledger_total_xp(),
			Progression::xp_for_level(state.level).to_string(),
			false,
		))
		.child(ledger_row(
			words.ledger_base_strike(),
			format!("×{:.2}", strike(state.level)),
			false,
		))
		.child(body(words.ledger_tradeoff(), 15, ASH))
		.child(soul_ledger(&state.genome(), state.level, &state.memory()))
		.child(entry(
			words.ledger_equipment(),
			words.ledger_equipment_help(),
		));
	for temper in Temper::ALL {
		column = column.child(gate(
			temper_name(&*words, temper),
			words.ledger_weapon_gate(
				temper.level_required(),
				temper.price(),
				&dart_double(temper.strike()),
			),
		));
	}
	for outfit in Outfit::ALL {
		column = column.child(gate(
			outfit_name(&*words, outfit),
			words.ledger_outfit_gate(outfit.level_required(), outfit.price()),
		));
	}
	column = column
		.child(gap(12))
		.child(prose(words.ledger_set_bonus(
			(Progression::STRIKE_PER_SET * 100.0).round() as i64,
		)))
		.child(entry(
			words.ledger_table(),
			words
				.ledger_progression_formula((Progression::STRIKE_PER_LEVEL * 100.0).round() as i64),
		))
		.child(
			View()
				.class(format!("flex-row py-2 border-b {}", class::HAIRLINE_BORDER))
				.child(
					View()
						.class("flex-1")
						.child(margin(words.ledger_level_column())),
				)
				.child(
					View()
						.class("flex-1 items-end")
						.child(margin(words.ledger_xp_column())),
				)
				.child(
					View()
						.class("flex-1 items-end")
						.child(margin(words.ledger_strike_column())),
				),
		);
	for level in 1..=Progression::LEVEL_CAP {
		column = column.child(
			View()
				.class("flex-row py-[6px]")
				.child(
					View()
						.class("flex-1")
						.child(figure(level.to_string(), 14, BONE)),
				)
				.child(View().class("flex-1 items-end").child(figure(
					Progression::xp_for_level(level).to_string(),
					14,
					BONE,
				)))
				.child(View().class("flex-1 items-end").child(figure(
					format!("×{:.2}", strike(level)),
					14,
					BONE,
				))),
		);
	}
	column
}

/// A temper or outfit, with its gate.
fn gate(name: &str, text: String) -> Element {
	View()
		.class("py-3 gap-1")
		.child(display(name, 22, BONE))
		.child(body(text, 14, ASH))
}

fn chance(state: &Book) -> Element {
	let words = use_l10n();
	let mut column = View()
		.child(entry(words.ledger_luck_heading(), words.ledger_luck_body()))
		.child(entry(words.ledger_birth_odds(), words.ledger_birth_body()))
		.child(
			LedgerButton::new(words.ledger_draw())
				.test_id("ledger-draw-birth")
				.on_press(|| draw(|book, seed| book.birth = Some(ReaperGenome::shuffle(seed)))),
		);
	if let Some(genome) = state.birth {
		let soul = ReaperSoul::from_genome(&genome);
		column = column.child(gap(16)).child(
			prose(words.ledger_birth_preview(
				&genome.name(),
				class_name(&*words, genome.reaper_class),
				gift_name(&*words, soul.gift),
				temperament_name(&*words, soul.temperament),
				omen_name(&*words, genome.omen),
			))
			.aria_live(leptoskit::native::Live::Polite)
			.test_id("ledger-birth"),
		);
	}
	column = column.child(ledger_heading(words.ledger_omen_odds(), None));
	for (omen, percent) in Book::omens() {
		column = column.child(ledger_row(
			omen_name(&*words, omen),
			words.ledger_omen_row(&format!("{percent:.0}")),
			false,
		));
	}
	column = column
		.child(entry(words.ledger_scenario(), words.ledger_roll_empty()))
		.child(chips(
			Rarity::ALL
				.into_iter()
				.map(|item| {
					chip(
						rarity_name(&*words, item),
						state.rarity == item,
						format!("ledger-rarity-{}", item.name()),
						Some(Box::new(move || {
							turn(move |book| book.scenario(|book| book.rarity = item));
						})),
					)
				})
				.collect(),
		))
		.child(ledger_heading(words.ledger_grade(), None))
		.child(chips(
			CaptureGrade::ALL
				.into_iter()
				.map(|item| {
					chip(
						grade_name(&*words, item),
						state.grade == item,
						format!("ledger-grade-{}", item.name()),
						Some(Box::new(move || {
							turn(move |book| book.scenario(|book| book.grade = item));
						})),
					)
				})
				.collect(),
		))
		.child(switch_row(
			words.ledger_ambush(),
			None,
			state.ambush,
			"ledger-ambush",
			|ambush| turn(move |book| book.scenario(|book| book.ambush = ambush)),
		))
		.child(ledger_heading(words.ledger_feeding_odds(), None));
	for (feeding, chance, xp) in state.feedings() {
		column = column.child(gate(
			feeding_name(&*words, feeding),
			words.ledger_chance_row(&format!("{chance:.0}"), &dart_double(feeding.share()), xp),
		));
	}
	column = column
		.child(gap(16))
		.child(prose(words.ledger_average(
			&format!("{:.2}", state.expected_xp()),
			state.anima(),
		)))
		.child(gap(20))
		.child(
			LedgerButton::new(words.ledger_roll())
				.test_id("ledger-roll-feeding")
				.on_press(|| draw(|book, seed| book.feeding = Some(Feeding::roll(seed)))),
		);
	if let Some(rolled) = state.feeding {
		column = column.child(gap(16)).child(
			display(
				words.ledger_roll_result(
					feeding_name(&*words, rolled),
					state.outcome().xp_for(state.rarity, rolled),
					state.anima(),
				),
				24,
				BONE,
			)
			.aria_live(leptoskit::native::Live::Polite)
			.test_id("ledger-rolled"),
		);
	}
	column
		.child(gap(14))
		.child(body(words.ledger_roll_notice(), 15, ASH))
		.child(entry(
			words.ledger_encounter(),
			words.ledger_encounter_body(
				PlayerState::SURFACE_MIN_SECONDS,
				PlayerState::SURFACE_MAX_SECONDS,
			),
		))
		.child(entry(words.ledger_trust(), words.ledger_trust_body()))
		.child(body(words.ledger_no_mutation(), 15, ASH))
}

fn nft_future() -> Element {
	let words = use_l10n();
	View()
		.child(entry(words.ledger_nft_heading(), words.ledger_nft_body()))
		.child(entry(
			words.ledger_nft_current_title(),
			words.ledger_nft_current_body(),
		))
		.child(entry(
			words.ledger_nft_next_title(),
			words.ledger_nft_next_body(),
		))
		.child(entry(
			words.ledger_nft_rifts_title(),
			words.ledger_nft_rifts_body(),
		))
		.child(entry(
			words.ledger_nft_later_title(),
			words.ledger_nft_later_body(),
		))
		.child(View().class(format!("h-px w-full my-5 {}", class::HAIRLINE_BG)))
		.child(body(words.ledger_nft_status(), 15, ASH))
		.child(entry(
			words.ledger_nft_economy(),
			words.ledger_nft_economy_body(),
		))
}

fn decisions() -> Element {
	let words = use_l10n();
	View()
		.child(entry(
			words.ledger_design_heading(),
			words.ledger_decision_body(),
		))
		.child(entry(
			words.ledger_physical_title(),
			words.ledger_physical_body(),
		))
		.child(entry(
			words.ledger_identity_title(),
			words.ledger_identity_body(),
		))
		.child(entry(
			words.ledger_memory_title(),
			words.ledger_memory_body(),
		))
		.child(entry(words.ledger_fair_title(), words.ledger_fair_body()))
		.child(entry(
			words.ledger_research_title(),
			words.ledger_research_body(),
		))
		.child(entry(
			words.ledger_balance_title(),
			words.ledger_balance_body(),
		))
		.child(entry(
			words.ledger_sources_title(),
			words.ledger_sources_body(),
		))
		.child(entry(words.ledger_needs_title(), words.ledger_needs_body()))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::l10n::En;
	use crate::l10n::Messages;

	#[test]
	fn the_borrowed_reaper_is_born_with_the_gift_and_temperament_chosen() {
		let mut book = Book::default();
		for calling in ReaperClass::ALL {
			book.choose_calling(calling);
			assert_eq!(book.gift.reaper_class(), calling);
			for gift in ReaperGift::ALL
				.into_iter()
				.filter(|gift| gift.reaper_class() == calling)
			{
				for temperament in Temperament::ALL {
					book.gift = gift;
					book.temperament = temperament;
					let soul = ReaperSoul::from_genome(&book.genome());
					assert_eq!(soul.gift, gift);
					assert_eq!(soul.temperament, temperament);
				}
			}
		}
	}

	#[test]
	fn an_awakening_needs_both_the_level_and_the_marks() {
		let mut book = Book::default();
		book.choose_calling(ReaperClass::Ferryman);
		book.gift = ReaperGift::Anchor;
		book.level = Progression::LEVEL_CAP;
		let rank = |book: &Book| SoulRules::rank(book.level, &book.memory(), book.gift);
		assert_eq!(rank(&book), 0, "Awakening 0 / 3");
		book.marks = 30;
		assert_eq!(rank(&book), SoulRules::MAX_RANK, "Awakening 3 / 3");
		book.level = 1;
		assert_eq!(rank(&book), 0);
	}

	#[test]
	fn the_odds_are_the_rules_own_and_a_new_scenario_forgets_its_roll() {
		let mut book = Book::default();
		let row = |book: &Book, feeding: Feeding| {
			let (_, chance, xp) = book
				.feedings()
				.into_iter()
				.find(|(item, ..)| *item == feeding)
				.unwrap();
			En.ledger_chance_row(&format!("{chance:.0}"), &dart_double(feeding.share()), xp)
		};
		assert_eq!(row(&book, Feeding::Meagre), "25% · ×0.85 XP · 43 XP");
		assert_eq!(row(&book, Feeding::Feast), "5% · ×2.0 XP · 100 XP");
		book.feeding = Some(Feeding::Fair);
		assert_eq!(book.anima(), 20);
		book.scenario(|book| book.rarity = Rarity::Rare);
		assert_eq!(book.feeding, None);
		assert_eq!(row(&book, Feeding::Meagre), "25% · ×0.85 XP · 128 XP");
		assert_eq!(
			En.ledger_average(&format!("{:.2}", book.expected_xp()), book.anima()),
			"Expected XP for this scenario: 159.60. Anima: 60. Expectation uses each rounded reward and its weight."
		);
		let total: f64 = Book::omens().into_iter().map(|(_, percent)| percent).sum();
		assert!((total - 100.0).abs() < 1e-9);
	}
}
