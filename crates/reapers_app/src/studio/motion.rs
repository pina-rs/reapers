//! Animations · 32 states: the authored clips, played on the specimen. The
//! Flutter `MotionControls`.
//!
//! The host plays a clip on its own clock and cannot seek, so where Flutter
//! had a timeline and "Inspect contact pose" the studio says why there is
//! none (`motionNoSeek`). Play, pause, replay, speed and "Inspect
//! expression" all work.

use leptoskit::prelude::*;
use reapers_rules::MotionGroup;
use reapers_rules::ReaperMotion;

use super::act;
use super::controls::Action;
use super::controls::chip;
use super::controls::chips;
use super::controls::help;
use super::controls::text_button;
use super::dart_double;
use super::specimen::SPEEDS;
use super::specimen::Specimen;
use crate::l10n::Messages;
use crate::l10n::use_l10n;
use crate::platform;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::ui::body;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_heading;

/// Every group, in the order the studio offers them.
pub const GROUPS: [MotionGroup; 5] = [
	MotionGroup::Movement,
	MotionGroup::Reaction,
	MotionGroup::Meme,
	MotionGroup::Combat,
	MotionGroup::Finisher,
];

pub fn motion_name(words: &dyn Messages, motion: ReaperMotion) -> &'static str {
	match motion {
		ReaperMotion::Idle => words.motion_idle(),
		ReaperMotion::Creep => words.motion_creep(),
		ReaperMotion::Walk => words.motion_walk(),
		ReaperMotion::Run => words.motion_run(),
		ReaperMotion::Sprint => words.motion_sprint(),
		ReaperMotion::Jump => words.motion_jump(),
		ReaperMotion::Land => words.motion_land(),
		ReaperMotion::DodgeLeft => words.motion_dodge_left(),
		ReaperMotion::DodgeRight => words.motion_dodge_right(),
		ReaperMotion::Dash => words.motion_dash(),
		ReaperMotion::DashLeft => words.motion_dash_left(),
		ReaperMotion::DashRight => words.motion_dash_right(),
		ReaperMotion::Laugh => words.motion_laugh(),
		ReaperMotion::Cry => words.motion_cry(),
		ReaperMotion::Rage => words.motion_rage(),
		ReaperMotion::Fear => words.motion_fear(),
		ReaperMotion::Bow => words.motion_bow(),
		ReaperMotion::Victory => words.motion_victory(),
		ReaperMotion::Gigachad => words.motion_gigachad(),
		ReaperMotion::Sigma => words.motion_sigma(),
		ReaperMotion::Shrug => words.motion_shrug(),
		ReaperMotion::AbsoluteCinema => words.motion_absolute_cinema(),
		ReaperMotion::TPose => words.motion_t_pose(),
		ReaperMotion::AnimeCharge => words.motion_anime_charge(),
		ReaperMotion::Slash => words.motion_slash(),
		ReaperMotion::Thrust => words.motion_thrust(),
		ReaperMotion::Parry => words.motion_parry(),
		ReaperMotion::FinisherReap => words.motion_finisher_reap(),
		ReaperMotion::FinisherImpale => words.motion_finisher_impale(),
		ReaperMotion::FinisherSoulCrush => words.motion_finisher_soul_crush(),
		ReaperMotion::FinisherPalm => words.motion_finisher_palm(),
		ReaperMotion::FinisherUppercut => words.motion_finisher_uppercut(),
	}
}

pub fn motion_group_name(words: &dyn Messages, group: MotionGroup) -> &'static str {
	match group {
		MotionGroup::Movement => words.motion_movement(),
		MotionGroup::Reaction => words.motion_reaction(),
		MotionGroup::Meme => words.motion_meme(),
		MotionGroup::Combat => words.motion_combat(),
		MotionGroup::Finisher => words.motion_finisher(),
	}
}

/// The animations, folded until opened.
pub fn motion_controls(specimen: &Specimen) -> Element {
	let words = use_l10n();
	let open = specimen.motions_open;
	let mut column = View().class("w-full pt-4").child(
		Pressable(words.motion_title().to_owned())
			.class("min-h-[52px] flex-row items-center justify-between")
			.test_id("studio-motions")
			.aria_expanded(open)
			.on_press(|| {
				platform::tap();
				act(|specimen| specimen.motions_open = !specimen.motions_open);
			})
			.child(body(words.motion_title(), 16, BONE))
			.child(label(if open { "−" } else { "+" }, 16, BONE)),
	);
	if !open {
		return column;
	}
	let reduced = platform::reduced_motion();
	let clock = specimen.clock;
	let motion = clock.motion();
	let weapon = specimen.visible_weapon();
	let group = specimen.group;

	column = column
		.child(body(words.motion_intro(), 13, ASH))
		.child(gap(12))
		.child(chips(
			GROUPS
				.into_iter()
				.map(|value| {
					chip(
						motion_group_name(&*words, value),
						value == group,
						format!("motion-group-{}", group_id(value)),
						Some(Box::new(move || {
							act(move |specimen| specimen.group = value);
						})),
					)
				})
				.collect(),
		))
		.child(ledger_heading(words.motion_select(), None))
		.child(chips(
			ReaperMotion::ALL
				.into_iter()
				.filter(|value| value.group() == group)
				.map(|value| {
					let press: Option<Action> = if value.supports(weapon) {
						Some(Box::new(move || {
							act(move |specimen| specimen.select_motion(value, reduced));
						}))
					} else {
						None
					};
					chip(
						motion_name(&*words, value),
						value == motion,
						format!("motion-{}", value.name()),
						press,
					)
				})
				.collect(),
		));
	if group == MotionGroup::Finisher {
		column = column.child(help(words.motion_weapon_hint()));
	}
	column = column
		.child(gap(14))
		.child(body(motion_name(&*words, motion), 15, BONE).test_id("motion-current"))
		.child(body(
			words.motion_timing(
				&format!("{:.1}", motion.seconds()),
				if motion.loops() {
					words.motion_loop()
				} else {
					words.motion_once()
				},
			),
			12,
			ASH,
		));
	if motion.unarmed() {
		column = column.child(body(words.motion_unarmed(), 12, ASH));
	}

	let playing = clock.playing() && !reduced;
	let mut buttons = View()
		.class("flex-row flex-wrap items-center gap-x-2 pt-2")
		.child(text_button(
			if playing {
				words.motion_pause()
			} else {
				words.motion_play()
			},
			"motion-play",
			move || act(move |specimen| specimen.press_play(reduced)),
		))
		.child(text_button(
			words.motion_replay(),
			"motion-replay",
			move || {
				act(move |specimen| specimen.replay(reduced));
			},
		));
	if matches!(motion.group(), MotionGroup::Reaction | MotionGroup::Meme) {
		buttons = buttons.child(text_button(
			words.studio_inspect_expression(),
			"motion-inspect",
			|| act(Specimen::inspect_expression),
		));
	}
	column = column.child(buttons);
	column = column
		.child(ledger_heading(words.motion_speed(), None))
		.child(chips(
			SPEEDS
				.into_iter()
				.map(|speed| {
					chip(
						&words.motion_rate(&dart_double(speed)),
						(clock.speed() - speed).abs() < 1e-9,
						format!("motion-speed-{}", dart_double(speed)),
						Some(Box::new(move || {
							act(move |specimen| specimen.set_speed(speed));
						})),
					)
				})
				.collect(),
		))
		.child(help(if reduced {
			words.motion_reduced_held()
		} else {
			words.motion_no_seek()
		}))
		.child(gap(16));
	column
}

const fn group_id(group: MotionGroup) -> &'static str {
	match group {
		MotionGroup::Movement => "movement",
		MotionGroup::Reaction => "reaction",
		MotionGroup::Meme => "meme",
		MotionGroup::Combat => "combat",
		MotionGroup::Finisher => "finisher",
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::l10n::En;

	#[test]
	fn every_motion_and_group_is_named_and_every_group_offered() {
		for motion in ReaperMotion::ALL {
			assert_ne!(motion_name(&En, motion), "");
			assert!(GROUPS.contains(&motion.group()));
		}
		assert_eq!(
			motion_name(&En, ReaperMotion::FinisherImpale),
			"Finisher · Impale"
		);
		assert_eq!(motion_group_name(&En, MotionGroup::Finisher), "Finishers");
	}

	#[test]
	fn speeds_read_as_the_flutter_studio_wrote_them() {
		assert_eq!(dart_double(0.25), "0.25");
		assert_eq!(dart_double(1.0), "1.0");
		assert_eq!(En.motion_rate(&dart_double(0.5)), "0.5×");
	}
}
