//! The light the reaper's weapons and orb throw into the world: the orb
//! itself with its halos, the soul streaming in, ring waves, the sigil,
//! tracers and soul-shard sparks.
//!
//! [`WorldFx`] is two models spawned once into the world's scene, the orb
//! (`viewmodel_orb.glb`) and the effects (`viewmodel_fx.glb`). Every tick it
//! is handed a [`WorldFrame`], what should show at that moment, and moves,
//! fades and hides their parts to match; the host eases each move over the
//! tick. Nothing is spawned or removed as effects come and go: the sparks
//! are a pool of two bursts.
//!
//! Every glow is additive and unlit, in the soul's colour; behind the orb's
//! bright halo is a dark one, blended normally, so the orb reads against a
//! camera picture of any brightness.

use leptoskit::scene3d::Blend;
use leptoskit::scene3d::MaterialOverride;
use leptoskit::scene3d::NodeId;
use leptoskit::scene3d::Op;
use leptoskit::scene3d::Transform;
use leptoskit::scene3d::client::Scene;

use super::camera::Point;
use super::camera::look_at;
use super::orb::OrbLook;
use super::puppet::Puppet;
use super::turn::add;
use super::turn::axis_angle;
use super::turn::length;
use super::turn::multiply;
use super::turn::scale;
use super::turn::sub;
use super::turn::up_to;
use super::viewmodel::SOUL;
use super::viewmodel::glow;

pub const ORB_MODEL: &str = "models/viewmodel_orb.glb";
pub const FX_MODEL: &str = "models/viewmodel_fx.glb";

/// How big the orb is in the world, against its model.
const ORB_SCALE: f32 = 1.8;
/// The halos' radii, in metres.
const HALO: f32 = 0.3;
const SHADE: f32 = 0.4;
/// The sigil's gold, burning.
const GOLD: [f32; 3] = [1.0, 0.68, 0.24];

const ADDITIVE: [&str; 7] = [
	"fx_halo",
	"fx_flare",
	"fx_sparks_0",
	"fx_sparks_1",
	"fx_ring",
	"fx_stream",
	"fx_tracer",
];

/// A tracer from the muzzle to what the shot hit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tracer {
	pub from: Point,
	pub to: Point,
	pub brightness: f32,
}

/// A burst of soul shards.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Burst {
	pub at: Point,
	/// How far it has flown out, from 0 to 1.
	pub spread: f32,
	pub brightness: f32,
	/// How big it is when fully out, in metres.
	pub size: f32,
	pub color: [f32; 3],
}

/// Everything the world's effects show at one moment.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WorldFrame {
	/// Where the camera is, for the halos to face.
	pub eye: Point,
	pub orb: Option<(OrbLook, Point)>,
	pub tracer: Option<Tracer>,
	pub sparks: [Option<Burst>; 2],
}

/// The orb and the effects, in one scene.
pub struct WorldFx {
	orb: Puppet,
	fx: Puppet,
}

impl WorldFx {
	/// Spawns the orb and the effects in `scene`, under `parent`, all hidden.
	pub fn spawn(scene: &Scene, parent: Option<NodeId>) -> Self {
		let mut orb = Puppet::spawn(scene, ORB_MODEL, parent);
		let mut fx = Puppet::spawn(scene, FX_MODEL, parent);
		for part in ADDITIVE {
			scene.push(Op::SetBlend {
				node: fx.node,
				part: Some(part.to_owned()),
				blend: Blend::Additive,
			});
			fx.visible(scene, part, false);
		}
		fx.visible(scene, "fx_shade", false);
		orb.visible(scene, "capture_orb", false);
		Self { orb, fx }
	}

	/// Shows `frame`, eased over `seconds`.
	pub fn show(&mut self, scene: &Scene, frame: &WorldFrame, seconds: f32) {
		self.orb(scene, frame, seconds);
		let fx = &mut self.fx;
		match frame.tracer {
			Some(tracer) if tracer.brightness > 0.01 => {
				let way = sub(tracer.to, tracer.from);
				let reach = length(way).max(0.01);
				let line = Transform {
					translation: tracer.from,
					rotation: up_to(scale(way, 1.0 / reach)),
					scale: [0.025, reach, 0.025],
				};
				fx.visible(scene, "fx_tracer", true);
				// A tracer jumps; it is never eased from the last one.
				fx.offset(scene, "fx_tracer", line, 0.0);
				fx.material(scene, "fx_tracer", glow(tracer.brightness));
			}
			_ => fx.visible(scene, "fx_tracer", false),
		}
		for (index, part) in ["fx_sparks_0", "fx_sparks_1"].into_iter().enumerate() {
			match frame.sparks[index] {
				Some(burst) if burst.brightness > 0.01 => {
					let starting = !fx.shown(part);
					fx.visible(scene, part, true);
					let size = burst.size * (0.15 + 0.85 * burst.spread);
					let spin = axis_angle([0.0, 1.0, 0.0], burst.spread * 1.2);
					let at = Transform {
						translation: burst.at,
						rotation: spin,
						scale: [size; 3],
					};
					fx.offset(scene, part, at, if starting { 0.0 } else { seconds });
					fx.material(scene, part, tinted(burst.color, burst.brightness));
				}
				_ => fx.visible(scene, part, false),
			}
		}
	}

	fn orb(&mut self, scene: &Scene, frame: &WorldFrame, seconds: f32) {
		let (orb, fx) = (&mut self.orb, &mut self.fx);
		let Some((look, husk)) = frame.orb.filter(|(look, _)| look.shown) else {
			orb.visible(scene, "capture_orb", false);
			for part in ["fx_halo", "fx_shade", "fx_flare", "fx_stream", "fx_ring"] {
				fx.visible(scene, part, false);
			}
			return;
		};
		let starting = !orb.shown("capture_orb");
		let ease = if starting { 0.0 } else { seconds };
		orb.visible(scene, "capture_orb", true);
		let body = Transform {
			translation: look.at,
			rotation: look.turn,
			scale: [ORB_SCALE; 3],
		};
		orb.offset(scene, "capture_orb", body, ease);
		orb.morph(scene, "orb_body", "open", look.open);
		orb.morph(scene, "orb_body", "burst", look.burst);
		let core_size = Transform {
			scale: [look.core_size; 3],
			..Transform::IDENTITY
		};
		orb.offset(scene, "orb_core", core_size, ease);
		orb.visible(scene, "orb_core", look.core > 0.01);
		// The core is light itself: unlit, and brighter than white.
		orb.material(
			scene,
			"orb_core",
			MaterialOverride {
				color: Some([SOUL[0] * 0.6, SOUL[1] * 0.6, SOUL[2] * 0.6, 1.0]),
				emissive: Some(SOUL),
				emissive_strength: Some(look.core * 1.4),
				..MaterialOverride::default()
			},
		);
		orb.material(
			scene,
			"orb_sigil",
			MaterialOverride {
				emissive: Some(GOLD),
				emissive_strength: Some(look.sigil * 5.0),
				..MaterialOverride::default()
			},
		);
		halos(scene, fx, &look, frame.eye, ease);
		stream(scene, fx, &look, husk, ease);
		ring(scene, fx, &look, seconds);
	}

	pub fn remove(&self, scene: &Scene) {
		self.orb.remove(scene);
		self.fx.remove(scene);
	}
}

/// The orb's halos, facing the eye: a dark one a little behind a bright
/// one, so the orb reads against a picture of any brightness, and a flare.
fn halos(scene: &Scene, fx: &mut Puppet, look: &OrbLook, eye: Point, ease: f32) {
	let facing = look_at(look.at, sub(scale(look.at, 2.0), eye), [0.0, 1.0, 0.0]).rotation;
	let away = scale(
		sub(look.at, eye),
		0.04 / length(sub(look.at, eye)).max(0.01),
	);
	let halo = look.halo.clamp(0.0, 1.5);
	disc(
		scene,
		fx,
		"fx_shade",
		add(look.at, away),
		facing,
		SHADE * halo,
		ease,
	);
	fx.material(
		scene,
		"fx_shade",
		MaterialOverride {
			color: Some([0.0, 0.0, 0.0, 1.0]),
			opacity: Some((0.6 * halo).min(1.0)),
			..MaterialOverride::default()
		},
	);
	disc(
		scene,
		fx,
		"fx_halo",
		look.at,
		facing,
		HALO * halo * (0.7 + 0.3 * look.core.min(2.0)),
		ease,
	);
	fx.material(scene, "fx_halo", glow((0.45 * look.core).min(1.0)));
	if look.flare > 0.01 {
		disc(
			scene,
			fx,
			"fx_flare",
			look.at,
			facing,
			0.25 + 0.5 * look.flare,
			ease,
		);
		fx.material(scene, "fx_flare", glow(look.flare.min(1.0)));
	} else {
		fx.visible(scene, "fx_flare", false);
	}
}

/// The soul streaming in from the husk to the orb.
fn stream(scene: &Scene, fx: &mut Puppet, look: &OrbLook, husk: Point, ease: f32) {
	match look.stream {
		Some((gathered, brightness)) if brightness > 0.01 => {
			let way = sub(look.at, husk);
			let reach = length(way).max(0.01);
			let twist = axis_angle([0.0, 1.0, 0.0], gathered * 9.0);
			let stream = Transform {
				translation: husk,
				rotation: multiply(up_to(scale(way, 1.0 / reach)), twist),
				scale: [reach; 3],
			};
			fx.visible(scene, "fx_stream", true);
			fx.offset(scene, "fx_stream", stream, ease);
			fx.morph(scene, "fx_stream", "gather", gathered);
			// Turning `flow` up over and over draws the motes along.
			fx.morph(
				scene,
				"fx_stream",
				"flow",
				(gathered * 6.0).fract() * (1.0 - gathered),
			);
			fx.material(scene, "fx_stream", glow(brightness));
		}
		_ => fx.visible(scene, "fx_stream", false),
	}
}

/// The latest ring wave, spreading over the ground from the orb.
fn ring(scene: &Scene, fx: &mut Puppet, look: &OrbLook, seconds: f32) {
	match look.ring {
		Some((spread, brightness)) if brightness > 0.01 => {
			let starting = !fx.shown("fx_ring");
			fx.visible(scene, "fx_ring", true);
			let ring = Transform {
				translation: add(look.at, [0.0, 0.01 - 0.07 * ORB_SCALE, 0.0]),
				rotation: [0.0, 0.0, 0.0, 1.0],
				scale: [0.1 + 0.8 * spread; 3],
			};
			fx.offset(scene, "fx_ring", ring, if starting { 0.0 } else { seconds });
			fx.material(scene, "fx_ring", glow(brightness));
		}
		_ => fx.visible(scene, "fx_ring", false),
	}
}

fn disc(
	scene: &Scene,
	fx: &mut Puppet,
	part: &'static str,
	at: Point,
	facing: [f32; 4],
	radius: f32,
	seconds: f32,
) {
	let starting = !fx.shown(part);
	fx.visible(scene, part, radius > 0.005);
	let disc = Transform {
		translation: at,
		rotation: facing,
		scale: [radius; 3],
	};
	fx.offset(scene, part, disc, if starting { 0.0 } else { seconds });
}

/// Light of `color` at `brightness`.
pub fn tinted(color: [f32; 3], brightness: f32) -> MaterialOverride {
	MaterialOverride {
		color: Some([color[0], color[1], color[2], 1.0]),
		opacity: Some(brightness.clamp(0.0, 1.0)),
		..MaterialOverride::default()
	}
}
