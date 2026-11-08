//! The plan a sigil leaves, from `reapers_flutter/test/hunt/scan_overlay_test.dart`,
//! and when the camera can read the room.

mod support;

use reapers_rules::Difficulty;
use reapers_rules::Gait;
use reapers_rules::HuntRules;
use reapers_rules::Rarity;
use reapers_rules::Sigil;
use reapers_rules::SigilTrace;
use reapers_rules::Vec3;
use reapers_rules::Vessel;
use reapers_rules::radians;
use reapers_world::ArPose;
use reapers_world::DrawnRoom;
use reapers_world::RoomSurvey;
use reapers_world::can_read;
use reapers_world::sigil_plan;
use support::assert_close;
use support::slice_at;

fn trace(centre: f64) -> SigilTrace {
	SigilTrace::new(
		Sigil::draw(
			11,
			&Rarity::Uncommon.sigil(),
			Gait::Ground,
			Difficulty::Normal,
		),
		centre,
	)
}

/// A level view tipped `pitch` degrees up, turning at `turn_speed`.
fn looking(pitch: f64, turn_speed: f64) -> ArPose {
	ArPose::looking(Vec3::from_angles(0.0, radians(pitch)), turn_speed)
}

mod the_plan_a_sigil_leaves {
	use super::*;

	#[test]
	fn without_a_survey_it_is_the_arc_the_sigil_read_and_nothing_more() {
		let trace = trace(0.5);
		let plan = sigil_plan(&trace, None);

		assert!(!plan.is_surveyed());
		assert_eq!(plan.eye_centimetres, HuntRules::EYE_CENTIMETRES);
		assert_close(plan.centre(), 0.5, 1e-3);
		assert_close(plan.half_width(), trace.sigil.half_arc(), 1e-3);
		assert!(plan.is_well_formed());
	}

	#[test]
	fn with_one_the_survey_is_asked_about_that_arc() {
		let trace = trace(-1.2);
		let mut survey = RoomSurvey::new(Vessel::Penguin);

		survey.measure_floor(1.2);

		let plan = sigil_plan(&trace, Some(&survey));

		assert_eq!(plan, survey.plan_within(-1.2, trace.sigil.half_arc()));
		assert_close(plan.centre(), -1.2, 1e-3);
		assert_eq!(plan.eye_centimetres, 120);
	}
}

mod reading_the_room {
	use super::*;

	#[test]
	fn a_walker_is_read_looking_down_at_the_floor() {
		assert!(can_read(&looking(-30.0, 0.0), Gait::Ground, 0.7));
		assert!(!can_read(&looking(0.0, 0.0), Gait::Ground, 0.7));
		assert!(!can_read(&looking(-60.0, 0.0), Gait::Ground, 0.7));
	}

	#[test]
	fn a_flier_is_read_about_head_height() {
		for gait in [Gait::Air, Gait::Still] {
			assert!(can_read(&looking(5.0, 0.0), gait, 0.7));
			assert!(!can_read(&looking(-30.0, 0.0), gait, 0.7));
			assert!(!can_read(&looking(45.0, 0.0), gait, 0.7));
		}
	}

	#[test]
	fn nothing_is_read_while_the_phone_turns_fast() {
		assert!(!can_read(&looking(-30.0, 1.0), Gait::Ground, 0.7));
	}

	#[test]
	fn a_survey_wants_what_its_husk_can_be_read_for() {
		let walker = RoomSurvey::new(Vessel::Penguin);
		let flier = RoomSurvey::new(Vessel::Bird);

		assert!(walker.wants(&looking(-30.0, 0.0)));
		assert!(!walker.wants(&looking(-30.0, RoomSurvey::STEADY_TURN_SPEED + 0.1)));
		assert!(flier.wants(&looking(5.0, 0.0)));
		assert!(!flier.wants(&looking(-30.0, 0.0)));
	}
}

#[test]
fn a_survey_keeps_its_map_and_its_occlusion_in_step() {
	let room = DrawnRoom::kitchen();
	let mut survey = RoomSurvey::new(Vessel::Ape);

	// A picture of nothing but wall teaches nothing, and is not added to
	// what hides a husk either.
	assert!(!survey.add(&DrawnRoom::walled(1.0).picture(0.0, 25.0)));
	assert_eq!(survey.pictures(), 0);
	assert_eq!(survey.occlusion().pictures(), 0);

	assert!(survey.add(&room.picture(0.0, -30.0)));
	assert!(survey.add(&room.picture(20.0, -30.0)));
	assert_eq!(survey.pictures(), 2);
	assert_eq!(survey.occlusion().pictures(), 2);

	let ahead = slice_at(0.0);

	assert!(survey.read(ahead) > 0.0);
	assert_eq!(survey.reach(ahead), survey.map().reach(ahead));
	assert_eq!(survey.tight(ahead), survey.map().tight(ahead));
}
