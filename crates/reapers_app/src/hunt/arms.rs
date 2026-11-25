//! The reaper's own arms in a fight: what the viewmodel shows, worked out
//! from what the rules did and are doing.
//!
//! [`Arms`] hears every event the rules report, on the tick it happened,
//! and follows the rules' state between them: a heavy dive's swing that is
//! due, a dive's way back out, ORB held. It decides nothing about the fight;
//! it only says which motion the arms are in and since when, how hard each
//! blow landed ([`Juice`]), and where the latest one struck.
//!
//! The seam's actions map from the rules as `seams::viewmodel` says. Two
//! are timed to the rules' tick rather than the event's: a heavy dive's
//! swing starts [`CONTACT_TICKS`] before the rules land it, and the knife's
//! thrust reaches the husk [`STAB_CONTACT_TICKS`] after the stab.

use reapers_rules::Archetype;
use reapers_rules::Arsenal;
use reapers_rules::BladeRules;
use reapers_rules::HeavyRules;
use reapers_rules::HuntEvent;
use reapers_rules::HuntSim;
use reapers_rules::Vec3;
use reapers_rules::WeaponKind;

use crate::render::arms::CONTACT_TICKS;
use crate::render::arms::JAM_SECONDS;
use crate::render::arms::STAB_SECONDS;
use crate::render::arms::THROW_RELEASE;
use crate::render::arms::TICKS_PER_SECOND;
use crate::render::camera::Point;
use crate::seams::HitView;
use crate::seams::Jolt;
use crate::seams::Juice;
use crate::seams::ViewAction;
use crate::seams::ViewmodelProps;
use crate::seams::WorldView;
use crate::theme::Rgb;

/// Ticks from a stab to its thrust reaching the husk: where the blow is
/// felt and its shards fly.
pub const STAB_CONTACT_TICKS: i64 = seconds_to_ticks(STAB_SECONDS);
/// Ticks from the start of a throw to the orb leaving the hand.
const RELEASE_TICKS: i64 = seconds_to_ticks(THROW_RELEASE);
/// Ticks a jam rattles before the reload is worked again.
const JAM_TICKS: i64 = seconds_to_ticks(JAM_SECONDS);
/// How far in front of the husk's body a blow's shards burst, in metres:
/// on its near side, not inside it.
const NEAR_SIDE: f64 = 0.3;

const fn seconds_to_ticks(seconds: f32) -> i64 {
	(seconds * TICKS_PER_SECOND + 0.5) as i64
}

/// A dive under way, as its events told it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Dive {
	weapon: Archetype,
	/// The tick it began.
	start: i64,
}

/// What the arms are doing in a fight, between renders.
#[derive(Clone, Debug, PartialEq)]
pub struct Arms {
	/// The class weapon in the heavy slot.
	heavy: WeaponKind,
	action: ViewAction,
	action_tick: i64,
	juice: Juice,
	hit: Option<HitView>,
	/// Which side the next stab comes in from.
	next_stab: u8,
	/// The latest stab's tick, so a dive comes out once it has landed.
	last_stab: i64,
	dive: Option<Dive>,
	/// The tick the reload under way began.
	reload_start: Option<i64>,
}

impl Arms {
	/// The class weapon whose swing the arms show.
	pub const fn weapon(&self) -> WeaponKind {
		self.heavy
	}

	/// Arms drawing their first weapon as the fight in `sim` begins, with
	/// `heavy` in the heavy slot.
	pub fn new(sim: &HuntSim, heavy: WeaponKind) -> Self {
		Self {
			heavy,
			action: ViewAction::Draw,
			action_tick: sim.tick(),
			juice: Juice::default(),
			hit: None,
			next_stab: 0,
			last_stab: i64::MIN / 2,
			dive: None,
			reload_start: None,
		}
	}

	/// What the arms are doing, and the tick it began.
	pub const fn action(&self) -> (ViewAction, i64) {
		(self.action, self.action_tick)
	}

	/// Every blow's weight so far.
	pub const fn juice(&self) -> &Juice {
		&self.juice
	}

	fn play(&mut self, action: ViewAction, tick: i64) {
		self.action = action;
		self.action_tick = tick;
	}

	/// Hears `event`, which happened on the rules' `tick`. `husk` is how the
	/// husk is drawn: where blows on it land, and its light.
	pub fn hear(&mut self, sim: &HuntSim, tick: i64, event: &HuntEvent, husk: &HuskMarks) {
		match event {
			HuntEvent::WeaponDrawn(_) => {
				self.play(ViewAction::Draw, tick);
				self.reload_start = None;
			}
			HuntEvent::ShotFired { .. } => {
				self.play(ViewAction::Fire, tick);
				self.juice.land(Jolt::Shot, tick);
			}
			HuntEvent::ArtThrown(_) => self.play(ViewAction::Fire, tick),
			HuntEvent::ReloadStarted => {
				self.play(ViewAction::Reload, tick);
				self.reload_start = Some(tick);
			}
			HuntEvent::ReloadJammed => self.play(ViewAction::Jam, tick),
			HuntEvent::ReloadFinished { perfect } => {
				self.reload_start = None;
				if *perfect {
					self.play(ViewAction::ActiveReload, tick);
				} else if self.action == ViewAction::Jam {
					// The jam is worked free: the drum goes home.
					self.play(ViewAction::Idle, tick);
				}
			}
			HuntEvent::DiveStarted(weapon) => {
				self.dive = Some(Dive {
					weapon: *weapon,
					start: tick,
				});
				self.play(ViewAction::DiveIn, tick);
			}
			HuntEvent::DiveEnded => {
				self.dive = None;
				if self.in_dive() {
					self.play(ViewAction::DiveOut, tick);
				}
			}
			HuntEvent::HitLanded {
				weapon,
				damage,
				cadence,
				critical,
				seal,
				..
			} => {
				self.strike(*weapon, tick);
				let landed = Self::landing(*weapon, tick);
				let jolt = match weapon {
					Archetype::Gun if seal.is_some() || *critical => Some(Jolt::SealShot),
					Archetype::Gun | Archetype::Art => None,
					Archetype::Blade => Some(Jolt::Stab),
					Archetype::Heavy => Some(Jolt::Heavy(self.heavy)),
				};
				if let Some(jolt) = jolt {
					self.juice.land(jolt, landed);
				}
				let at = seal.map_or(sim.pose().position, |seal| sim.pose().seal_position(seal));
				self.hit = Some(husk.hit(landed, at, *damage, *critical, seal.is_some(), *cadence));
			}
			HuntEvent::Missed(weapon) => self.strike(*weapon, tick),
			HuntEvent::ArtLanded {
				hit: true,
				at,
				damage,
				..
			} => {
				self.juice.land(Jolt::ArtHit, tick);
				self.hit = Some(husk.hit(tick, *at, *damage, false, false, 1));
			}
			HuntEvent::Parried => {
				self.play(ViewAction::Parry, tick);
				self.juice.land(Jolt::Parry, tick);
			}
			HuntEvent::Struck { .. } => {
				self.play(ViewAction::Struck, tick);
				self.juice.land(Jolt::Struck, tick);
			}
			HuntEvent::Broken { .. } => self.juice.land(Jolt::Kill, tick),
			HuntEvent::OrbThrown { .. } => {
				// ORB is let go on the rules' tick, and the orb leaves the
				// hand on it: the throw is already that far along.
				self.play(ViewAction::Throw, tick - RELEASE_TICKS);
			}
			HuntEvent::OrbImpact { .. } => self.juice.land(Jolt::OrbImpact, tick),
			HuntEvent::OrbBrokeFree(_) => self.juice.land(Jolt::OrbBreak, tick),
			_ => {}
		}
	}

	/// A stab or a swing that the rules landed, or that met nothing, on
	/// `tick`: the knife's thrust, or a heavy swing in arm's reach struck at
	/// once. A heavy dive's swing is already under way.
	fn strike(&mut self, weapon: Archetype, tick: i64) {
		let dived = self.dive.is_some_and(|dive| dive.weapon == weapon);
		match weapon {
			Archetype::Blade => {
				let nth = self.next_stab;
				self.next_stab = (nth + 1) % 2;
				self.last_stab = tick;
				self.play(ViewAction::Stab { nth, dived }, tick);
			}
			Archetype::Heavy if !dived => self.play(ViewAction::Strike, tick),
			_ => {}
		}
	}

	/// The tick a blow the rules landed on `tick` is seen to strike: a
	/// knife's thrust a little after the stab, everything else on it.
	const fn landing(weapon: Archetype, tick: i64) -> i64 {
		match weapon {
			Archetype::Blade => tick + STAB_CONTACT_TICKS,
			_ => tick,
		}
	}

	/// Whether the arms are held in a dive.
	const fn in_dive(&self) -> bool {
		matches!(
			self.action,
			ViewAction::DiveIn | ViewAction::Stab { dived: true, .. }
		)
	}
}

impl Arms {
	/// Follows the rules' state at their tick now: a heavy dive's swing
	/// begins so that its blow lands on the rules' tick, a knife's dive comes
	/// back out as the rules' does, a jammed reload is worked again, and ORB
	/// held raises the orb.
	pub fn follow(&mut self, sim: &HuntSim) {
		let now = sim.tick();
		if let Some(dive) = self.dive {
			match dive.weapon {
				Archetype::Heavy if self.action == ViewAction::DiveIn => {
					let blow = dive.start + HeavyRules::IN_TICKS + sim.stats().swing_ticks;
					let swing = blow - CONTACT_TICKS;
					if now >= swing {
						self.play(ViewAction::Swing { dived: true }, swing);
					}
				}
				Archetype::Blade if self.in_dive() => {
					// Out once the window has closed and the last thrust is
					// home.
					let window = dive.start + BladeRules::IN_TICKS + BladeRules::WINDOW_TICKS;
					let out = window.max(self.last_stab + STAB_CONTACT_TICKS * 2);
					if now >= out {
						self.play(ViewAction::DiveOut, out);
					}
				}
				_ => {}
			}
		}
		if self.action == ViewAction::Jam
			&& now >= self.action_tick + JAM_TICKS
			&& let Some(start) = self.reload_start
			&& sim.reloading()
		{
			// Worked again from the start, so the drum goes home as the
			// rules finish the reload.
			self.play(ViewAction::Reload, start + JAM_TICKS);
		}
		let raised = sim.orb_ring().is_some() && sim.orb_block().is_none();
		if raised && self.action != ViewAction::Raise {
			self.play(ViewAction::Raise, now);
		} else if !raised && self.action == ViewAction::Raise {
			self.play(ViewAction::Idle, now);
		}
	}

	/// The viewmodel's props at the rules' tick now, in the room `world`,
	/// with the view turning at `turn_rate` (radians a second, right and up),
	/// for a reaper whose marks print in `spot`.
	pub fn props(
		&self,
		sim: &HuntSim,
		world: Option<WorldView>,
		turn_rate: [f32; 2],
		spot: Rgb,
	) -> ViewmodelProps {
		let archetype = Arsenal::archetype_of(sim.selected_slot()).unwrap_or(Archetype::Gun);
		ViewmodelProps {
			archetype,
			heavy: self.heavy,
			action: self.action,
			action_tick: self.action_tick,
			now: sim.tick(),
			rate: TICKS_PER_SECOND,
			turn_rate,
			juice: self.juice,
			orb: sim.orb(),
			world,
			hit: self.hit,
			spot,
			sight: None,
		}
	}
}

/// How the husk is drawn, for the blows that strike it: the player's place
/// in the scene, and the husk's own light.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HuskMarks {
	/// Where the rules' origin, the player's eye, is in the scene.
	pub origin: Point,
	/// The camera in hunt coordinates.
	pub eye: Vec3,
	/// The husk's light, in linear RGB.
	pub light: [f32; 3],
}

impl HuskMarks {
	/// A blow seen to strike on `tick` at `at`, a point in the hunt's space
	/// on the husk, drawn on its near side: on an open seal or not, and the
	/// `cadence`th hit in a row.
	fn hit(
		&self,
		tick: i64,
		at: Vec3,
		damage: f64,
		crit: bool,
		seal: bool,
		cadence: i64,
	) -> HitView {
		let way = at - self.eye;
		let distance = way.length();
		let near = if distance > NEAR_SIDE * 2.0 {
			at - way * (NEAR_SIDE / distance)
		} else {
			at
		};
		HitView {
			tick,
			at: self.scene_point(near),
			damage: damage.round() as i64,
			crit,
			seal,
			cadence,
			color: self.light,
		}
	}

	/// A point in the hunt's space, in the scene's: the hunt has z ahead of
	/// the eye, the scene looks down -z.
	pub fn scene_point(&self, at: Vec3) -> Point {
		[
			self.origin[0] + at.x as f32,
			self.origin[1] + at.y as f32,
			self.origin[2] - at.z as f32,
		]
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// What a fought hunt showed: every event on the rules' tick it happened,
	/// the tick each action that strikes something began, and every blow's
	/// landing.
	struct Fought {
		events: Vec<(i64, String)>,
		actions: Vec<(ViewAction, i64)>,
		blows: Vec<i64>,
	}

	/// Fights a hunt the rules' own player plays, drawn `rate` frames a
	/// second.
	fn fought_at(rate: f64, seed: i64) -> Fought {
		use reapers_rules::Difficulty;
		use reapers_rules::HuntLoadout;
		use reapers_rules::HuntPlan;
		use reapers_rules::HuntStats;
		use reapers_rules::market_listings;
		use reapers_world::ArPose;

		use crate::hunt::controller::Fight;
		use crate::hunt::controller::FrameEye;
		use crate::hunt::controller::HuntController;

		let nft = market_listings()[seed as usize % 8];
		let stats = HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap();
		let sim = HuntSim::new(
			seed,
			nft.collection.vessel(),
			nft.rarity(),
			nft.rarity().ward(),
			&stats,
			HuntPlan::OPEN,
			Difficulty::default(),
		)
		.unwrap();
		let mut controller = HuntController::autoplay(Fight::Hunt(sim));
		let mut arms = Arms::new(controller.sim(), WeaponKind::Scythe);
		let marks = HuskMarks {
			origin: [0.0; 3],
			eye: Vec3::ZERO,
			light: [1.0; 3],
		};
		let (mut events, mut actions, mut blows) = (Vec::new(), Vec::new(), Vec::new());
		let mut last = arms.action();
		let mut note = |arms: &Arms, actions: &mut Vec<(ViewAction, i64)>| {
			let now = arms.action();
			if now != last
				&& !matches!(
					now.0,
					ViewAction::Idle | ViewAction::Raise | ViewAction::Draw
				) {
				actions.push(now);
			}
			last = now;
		};
		let frames = (40.0 * rate) as usize;
		for _ in 0..frames {
			if controller.sim().is_over() {
				break;
			}
			let heard = controller.frame(1.0 / rate, &ArPose::AHEAD, FrameEye::Origin);
			for (tick, event) in &heard {
				arms.hear(controller.sim(), *tick, event, &marks);
				events.push((*tick, format!("{event:?}")));
				note(&arms, &mut actions);
				if let Some(hit) = arms.hit
					&& blows.last() != Some(&hit.tick)
				{
					blows.push(hit.tick);
				}
			}
			arms.follow(controller.sim());
			note(&arms, &mut actions);
		}
		Fought {
			events,
			actions,
			blows,
		}
	}

	#[test]
	fn the_rules_and_every_blow_land_on_the_same_tick_at_any_frame_rate() {
		// Two hunts the rules' player fights with the pistol, the knife's
		// dives and its parries, and the orb.
		for seed in [2, 4] {
			let sixty = fought_at(60.0, seed);
			assert!(
				sixty.blows.len() > 20,
				"the hunt landed blows: {:?}",
				sixty.blows
			);
			for wanted in ["Fire", "Stab", "Parry", "Throw"] {
				assert!(
					sixty
						.actions
						.iter()
						.any(|(action, _)| format!("{action:?}").starts_with(wanted)),
					"seed {seed} played no {wanted}"
				);
			}
			// The knife's thrust is seen to land the moment it reaches the husk.
			for (action, tick) in &sixty.actions {
				if matches!(action, ViewAction::Stab { .. })
					&& sixty
						.events
						.iter()
						.any(|(at, event)| at == tick && event.starts_with("HitLanded"))
				{
					assert!(sixty.blows.contains(&(tick + STAB_CONTACT_TICKS)));
				}
			}
			for rate in [30.0, 120.0, 144.0] {
				let other = fought_at(rate, seed);
				assert_eq!(
					other.events, sixty.events,
					"the rules' events moved at {rate} Hz"
				);
				assert_eq!(
					other.actions, sixty.actions,
					"an action's start moved at {rate} Hz"
				);
				assert_eq!(
					other.blows, sixty.blows,
					"a blow's landing moved at {rate} Hz"
				);
			}
		}
	}

	#[test]
	fn impact_is_on_the_side_facing_the_moved_eye() {
		let marks = HuskMarks {
			origin: [1.0, 2.0, 3.0],
			eye: Vec3::new(1.0, 0.0, 0.0),
			light: [1.0; 3],
		};
		let hit = marks.hit(0, Vec3::new(0.0, 0.0, 2.0), 1.0, false, false, 1);
		assert!(hit.at[0] > 1.0);
		assert!(hit.at[2] > 1.0);
	}
}
