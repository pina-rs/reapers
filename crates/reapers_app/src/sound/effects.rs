//! Short, recorded combat cues. They follow accepted rule events, so an
//! empty trigger and a real pistol shot never sound the same.

use reapers_rules::Archetype;
use reapers_rules::HuntEvent;
use reapers_rules::Vessel;
use reapers_rules::WeaponKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Effect {
	PistolA,
	PistolB,
	PistolC,
	Reload,
	Loaded,
	Blade,
	Heavy,
	Impact,
	Parry,
	Ape,
	Bird,
	Penguin,
	Llama,
	Robot,
	Boss,
	Shriek,
	Hurt,
	Catch,
}

impl Effect {
	pub const ALL: [Self; 18] = [
		Self::PistolA,
		Self::PistolB,
		Self::PistolC,
		Self::Reload,
		Self::Loaded,
		Self::Blade,
		Self::Heavy,
		Self::Impact,
		Self::Parry,
		Self::Ape,
		Self::Bird,
		Self::Penguin,
		Self::Llama,
		Self::Robot,
		Self::Boss,
		Self::Shriek,
		Self::Hurt,
		Self::Catch,
	];

	pub const fn asset(self) -> &'static str {
		match self {
			Self::PistolA => "sfx/pistol-a.ogg",
			Self::PistolB => "sfx/pistol-b.ogg",
			Self::PistolC => "sfx/pistol-c.ogg",
			Self::Reload => "sfx/reload.ogg",
			Self::Loaded => "sfx/loaded.ogg",
			Self::Blade => "sfx/blade.ogg",
			Self::Heavy => "sfx/heavy.ogg",
			Self::Impact => "sfx/impact.ogg",
			Self::Parry => "sfx/parry.ogg",
			Self::Ape => "sfx/ape.ogg",
			Self::Bird => "sfx/bird.ogg",
			Self::Penguin => "sfx/penguin.ogg",
			Self::Llama => "sfx/llama.ogg",
			Self::Robot => "sfx/robot.ogg",
			Self::Boss => "sfx/boss.ogg",
			Self::Shriek => "sfx/shriek.ogg",
			Self::Hurt => "sfx/hurt.ogg",
			Self::Catch => "sfx/catch.ogg",
		}
	}

	pub const fn volume(self) -> f32 {
		match self {
			Self::PistolA | Self::PistolB | Self::PistolC => 0.8,
			Self::Reload | Self::Loaded | Self::Catch => 0.55,
			Self::Shriek => 0.65,
			_ => 0.75,
		}
	}

	pub const fn enemy(vessel: Vessel) -> Self {
		match vessel {
			Vessel::Ape => Self::Ape,
			Vessel::Bird => Self::Bird,
			Vessel::Penguin => Self::Penguin,
			Vessel::Llama => Self::Llama,
			Vessel::Robot => Self::Robot,
			Vessel::ReliquaryStack => Self::Boss,
		}
	}

	pub fn event(event: &HuntEvent, tick: i64, vessel: Vessel, heavy: WeaponKind) -> Option<Self> {
		let swing = || {
			match heavy {
				WeaponKind::Scythe
				| WeaponKind::Greatsword
				| WeaponKind::Axe
				| WeaponKind::Spear => Self::Blade,
				WeaponKind::Bell => Self::Parry,
				_ => Self::Heavy,
			}
		};
		Some(match event {
			HuntEvent::ShotFired { .. } => {
				match tick.rem_euclid(3) {
					0 => Self::PistolA,
					1 => Self::PistolB,
					_ => Self::PistolC,
				}
			}
			HuntEvent::ReloadStarted => Self::Reload,
			HuntEvent::ReloadFinished { .. } => Self::Loaded,
			HuntEvent::HitLanded {
				weapon: Archetype::Blade,
				..
			}
			| HuntEvent::Missed(Archetype::Blade) => Self::Blade,
			HuntEvent::HitLanded {
				weapon: Archetype::Heavy,
				..
			}
			| HuntEvent::Missed(Archetype::Heavy) => swing(),
			HuntEvent::HitLanded {
				weapon: Archetype::Gun,
				..
			} => Self::Impact,
			HuntEvent::ShriekWindup | HuntEvent::ChargeBegan => Self::enemy(vessel),
			HuntEvent::ShriekBegan => Self::Shriek,
			HuntEvent::Struck { .. }
			| HuntEvent::Flinched {
				forgiven: false, ..
			} => Self::Hurt,
			HuntEvent::Parried | HuntEvent::IceShattered => Self::Parry,
			HuntEvent::HuskCaptured(_) | HuntEvent::OrbAbsorbed => Self::Catch,
			_ => return None,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn only_a_real_shot_plays_the_pistol_and_enemy_tells_are_distinct() {
		let cue =
			|event: &HuntEvent| Effect::event(event, 3, Vessel::Robot, WeaponKind::BalanceFlail);
		assert_eq!(
			cue(&HuntEvent::ShotFired {
				rounds_left: 5,
				boosted: false
			}),
			Some(Effect::PistolA)
		);
		assert_eq!(cue(&HuntEvent::Missed(Archetype::Gun)), None);
		assert_eq!(cue(&HuntEvent::ChargeBegan), Some(Effect::Robot));
		assert_eq!(
			cue(&HuntEvent::Struck {
				resolve_left: 2,
				diving: false
			}),
			Some(Effect::Hurt)
		);
		let mut sounds = std::collections::HashSet::new();
		for vessel in Vessel::ALL {
			assert!(sounds.insert(Effect::enemy(vessel)));
		}
	}
}
