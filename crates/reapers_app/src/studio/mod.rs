//! The reaper studio: an unsaved specimen on the game's own models and the
//! shared progression rules. The Flutter `studio/`.
//!
//! | Flutter | Here |
//! | --- | --- |
//! | `reaper_studio.dart` | this module, [`specimen`], [`growth`], [`soul`] |
//! | `appearance_controls.dart` | [`appearance`] |
//! | `motion_controls.dart` | [`motion`] |
//! | `decision_ledger.dart` | [`book`] |
//!
//! It opens no save, server, wallet or camera, and nothing done in it
//! changes an owned reaper: the specimen is a copy of the genome it was
//! opened on, kept in this module's own state.
//!
//! The stage is `render::stage`'s, under [`STUDIO`]. Each control changes
//! the [`Specimen`]; the next render shows it on the stage, and a change to
//! the clip or the face is sent at once (see [`act`]).

// A section is one builder, read top to bottom as the page reads.
#![allow(clippy::too_many_lines)]

pub mod appearance;
pub mod book;
pub mod controls;
pub mod growth;
pub mod motion;
pub mod soul;
pub mod specimen;

use std::time::Duration;

use leptoskit::prelude::*;
use reactive_graph::traits::UpdateUntracked;
use reapers_rules::ReaperGenome;

use self::controls::tabs;
use self::controls::text_button;
pub use self::specimen::GALLERY;
pub use self::specimen::Section;
pub use self::specimen::Specimen;
use crate::l10n::use_l10n;
use crate::lookups::class_name;
use crate::lookups::outfit_name;
use crate::platform;
use crate::render::scene::STUDIO;
use crate::render::stage::Loading;
use crate::render::stage::stage;
use crate::render::stage::stage_view;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::CONTENT_WIDTH;
use crate::theme::reaper_light;
use crate::ui::body;
use crate::ui::display;
use crate::ui::foot_scrim;
use crate::ui::gap;
use crate::ui::label;

/// How long the specimen turns once started: one slow revolution, then it
/// holds still. The player can start it again.
const SPIN: Duration = Duration::from_secs(25);

/// The studio's specimen, kept between renders. `None` until a studio
/// opens, and again once it is left.
pub fn specimen() -> RwSignal<Option<Specimen>> {
	thread_local! {
		static SPECIMEN: RwSignal<Option<Specimen>> = crate::ui::lasting(|| RwSignal::new(None));
	}
	SPECIMEN.with(|specimen| *specimen)
}

/// Changes the specimen, and tells the stage at once of a new clip or face.
pub fn act(change: impl FnOnce(&mut Specimen)) {
	let stage = stage(STUDIO);
	let live = stage.borrow().playback;
	let mut cue = None;
	specimen().update(|specimen| {
		let Some(specimen) = specimen else {
			return;
		};
		let before = specimen.cue;
		specimen.clock = live;
		change(specimen);
		if specimen.cue != before {
			cue = Some((specimen.clock, specimen.face()));
		}
	});
	if let Some((clock, face)) = cue {
		let mut stage = stage.borrow_mut();
		stage.playback = clock;
		stage.replay_clock();
		stage.hold_face(face);
	}
}

/// Opens a fresh specimen made by `make` unless one is open on `genome`.
fn open(genome: ReaperGenome, gallery: bool, make: impl FnOnce() -> Specimen) -> Specimen {
	let signal = specimen();
	if let Some(open) = signal.get_untracked()
		&& open.opened == genome
		&& open.gallery.is_some() == gallery
	{
		return signal.get().unwrap_or(open);
	}
	let fresh = make();
	signal.update_untracked(|open| *open = Some(fresh));
	let stage = stage(STUDIO);
	let mut stage = stage.borrow_mut();
	stage.playback = fresh.clock;
	stage.replay_clock();
	stage.hold_face(None);
	drop(stage);
	signal.get().unwrap_or(fresh)
}

/// Leaves the studio, forgetting the specimen: the next opens afresh.
fn leave() {
	specimen().set(None);
	crate::screens::back();
}

/// The studio, opened on `genome`.
pub fn studio(genome: ReaperGenome) -> Element {
	let specimen = open(genome, false, || Specimen::new(genome));
	page(&specimen)
}

/// The gallery bench: the studio stepping through every calling in every
/// outfit, for reviewing the models. Debug builds open at it with
/// `REAPERS_GALLERY`.
pub fn gallery(genome: ReaperGenome) -> Element {
	let specimen = open(genome, true, || Specimen::gallery(genome));
	page(&specimen)
}

fn page(specimen: &Specimen) -> Element {
	let words = use_l10n();
	controls::set_light(reaper_light(&specimen.genome));
	if specimen.book {
		return book::book_view(|| act(|s| s.book = false));
	}
	let content = View()
		.class(format!(
			"w-full {CONTENT_WIDTH} self-center px-6 pt-6 pb-10"
		))
		.child(display(words.studio_title(), 38, BONE))
		.child(gap(12))
		.child(body(words.studio_intro(), 15, ASH))
		.child(motion::motion_controls(specimen))
		.child(gap(12))
		.child(tabs(vec![
			(
				words.studio_traits(),
				"studio-tab-appearance",
				specimen.section == Section::Appearance,
				Box::new(|| act(|s| s.section = Section::Appearance)),
			),
			(
				words.studio_growth(),
				"studio-tab-growth",
				specimen.section == Section::Growth,
				Box::new(|| act(|s| s.section = Section::Growth)),
			),
			(
				words.soul_tab(),
				"studio-tab-soul",
				specimen.section == Section::Soul,
				Box::new(|| act(|s| s.section = Section::Soul)),
			),
		]))
		.child(match specimen.section {
			Section::Appearance => appearance::appearance(specimen),
			Section::Growth => growth::growth(specimen),
			Section::Soul => soul::soul(specimen),
		});
	let mut view = View()
		.class("w-full min-h-screen")
		.test_id("studio-view")
		.child(portrait(specimen));
	if let Some(index) = specimen.gallery {
		view = view.child(gallery_bar(index));
	}
	view.child(content)
}

/// The specimen on its stage, with its name and the stage's controls.
fn portrait(specimen: &Specimen) -> Element {
	let words = use_l10n();
	let genome = specimen.genome;
	let light = reaper_light(&genome);
	let (stage, view) = stage_view(STUDIO, specimen.show());
	let failed = stage.borrow().loading() == Loading::Failed;
	let mut top = View().class(
		"absolute left-0 right-0 top-0 flex-row items-start justify-between px-4 pt-safe-offset-2",
	);
	let mut left = View().class("gap-1");
	if specimen.gallery.is_none() {
		left = left.child(
			Pressable(words.common_back().to_owned())
				.class("min-w-[48px] min-h-[44px] justify-center")
				.test_id("ledger-back")
				.on_press(|| {
					platform::tap();
					leave();
				})
				.child(label("←", 13, BONE)),
		);
	}
	left = left.child(View().class("px-2").child(label(
		class_name(&*words, genome.reaper_class),
		12,
		light,
	)));
	top = top.child(left).child(
		View()
			.class("items-end")
			.child(text_button(
				if specimen.spinning {
					words.studio_pause()
				} else {
					words.studio_rotate()
				},
				"studio-rotate",
				rotate,
			))
			.child(text_button(
				if specimen.focus_face {
					words.studio_full_figure()
				} else {
					words.studio_face_closeup()
				},
				"studio-face",
				|| act(Specimen::toggle_face),
			))
			.child(text_button(words.ledger_open(), "studio-book-open", || {
				book::book().set(book::Book::default());
				act(|s| s.book = true);
			})),
	);
	let mut foot = View()
		.class("absolute left-6 right-6 bottom-6 gap-[10px]")
		.child(display(genome.name(), 34, BONE).test_id("studio-name"))
		.child(body(words.studio_drag(), 13, ASH));
	if failed {
		foot = foot.child(body(words.studio_model_failed(), 13, ASH));
	}
	View()
		.class("relative w-full h-[390px] overflow-hidden")
		.child(view)
		.child(foot_scrim(40, 0xd0))
		.child(top)
		.child(foot)
}

/// Starts or stops the rotation; a started one stops by itself after
/// [`SPIN`].
fn rotate() {
	act(Specimen::toggle_spin);
	let Some(now) = specimen().get_untracked() else {
		return;
	};
	if now.spinning {
		let spin = now.spin;
		platform::spawn(async move {
			if platform::sleep(SPIN).await {
				specimen().update(|specimen| {
					if let Some(specimen) = specimen
						&& specimen.spin == spin
					{
						specimen.spinning = false;
					}
				});
			}
		});
	}
}

/// The gallery bench's strip: where it stands, and the way along.
fn gallery_bar(index: usize) -> Element {
	let words = use_l10n();
	let (calling, outfit) = GALLERY[index];
	View()
		.class(format!(
			"w-full {CONTENT_WIDTH} self-center px-6 pt-4 gap-2"
		))
		.test_id("studio-gallery")
		.child(label(words.studio_gallery(), 11, ASH))
		.child(
			View()
				.class("flex-row items-center justify-between gap-3")
				.child(text_button(
					words.studio_gallery_previous(),
					"gallery-previous",
					|| {
						act(|s| s.step_gallery(-1));
					},
				))
				.child(
					body(
						words.studio_gallery_position(
							class_name(&*words, calling),
							outfit_name(&*words, outfit),
							index as i64 + 1,
							GALLERY.len() as i64,
						),
						13,
						BONE,
					)
					.class("flex-1 text-center")
					.test_id("gallery-position"),
				)
				.child(text_button(
					words.studio_gallery_next(),
					"gallery-next",
					|| {
						act(|s| s.step_gallery(1));
					},
				)),
		)
}

/// A number as Dart's `toString` writes it: `1.0`, `0.85`, `2.2`, as the
/// Flutter studio's strings showed them.
pub fn dart_double(value: f64) -> String {
	if value.fract() == 0.0 {
		format!("{value:.1}")
	} else {
		value.to_string()
	}
}
