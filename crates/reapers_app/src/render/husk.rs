//! One husk: a collection's creature, dressed as one NFT. The Flutter
//! `HuskRig`.
//!
//! The rig only ever draws what the rules say. It is told where the husk is
//! with [`HuskRig::place`], what it is doing with [`HuskRig::animate`], and
//! what happened with [`HuskRig::flash`], [`HuskRig::hit`] and
//! [`HuskRig::dissolve`]; it decides nothing about the hunt. How it acts that
//! out is [`super::reaction::HuskReaction`]'s.
//!
//! A creature is built from rigid parts on pivots. `leptoskit_3d` moves a
//! model's parts only by its clips, and the husks have none, so where the
//! Flutter rig swung each leg, this one moves the creature whole: it rises
//! and falls with each step or wingbeat, leans and twists from blows,
//! squashes as it is struck, and is dragged up the reaping arc.

use leptoskit::scene3d::MaterialOverride;
use leptoskit::scene3d::NodeId;
use leptoskit::scene3d::Op;
use leptoskit::scene3d::Rim;
use leptoskit::scene3d::Transform;
use leptoskit::scene3d::client::Scene;
use reapers_rules::Gait;
use reapers_rules::HuskPose;
use reapers_rules::Nft;
use reapers_rules::NftLook;
use reapers_rules::Vessel;
use reapers_rules::wrap_angle;

use super::look::linear;
use super::parts::parts;
use super::reaction::HuskPlight;
use super::reaction::HuskReaction;

/// The model of one vessel.
pub fn husk_model(vessel: Vessel) -> String {
	format!("models/husk_{}.glb", vessel.name())
}

/// What a husk is doing, as far as showing it goes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HuskMood {
	/// How fast it is covering ground, in metres a second.
	pub speed: f64,
	/// How wide its mouth is open, from 0 to 1.
	pub gape: f64,
	/// Whether it is frozen in ice.
	pub frozen: bool,
	/// Whether it is burning.
	pub burning: bool,
}

/// How bright each glowing surface burns at rest.
const ACCENT_GLOW: f32 = 6.0;
const GLOW_GLOW: f32 = 7.0;

/// How much of its eye colour edges a creature's coat. Just enough to lift
/// a dark coat off a dark room, and no more.
const RIM: f32 = 0.12;

/// How fast a strike's flash fades, as a share lost each second.
const FLASH_DECAY: f64 = 7.0;

/// One husk in a scene.
#[derive(Debug)]
pub struct HuskRig {
	/// The group placed where the rules put the husk.
	pub root: NodeId,
	/// The group between the placement and the model, carrying the rise and
	/// fall of a step and the lean of a blow.
	lift: NodeId,
	/// The model.
	pub body: NodeId,
	pub vessel: Vessel,
	look: NftLook,
	/// The colour this husk glows: the light in its eyes.
	pub light: [f32; 3],
	reaction: HuskReaction,
	/// The way from the player to the husk, flat, in the model's frame.
	away: (f64, f64),
	seconds: f64,
	stride: f64,
	effort: f64,
	flash: f64,
	punch: f64,
	frost: f64,
	burn: f64,
	tuck: f64,
	leaping: bool,
	grounded: bool,
	dissolved: f64,
	/// Where the rules last put it, and which way it faces.
	placed: Option<([f32; 3], f32)>,
	/// What was last sent, so a still husk sends nothing.
	sent: Option<(Transform, Transform, [i32; 6])>,
}

impl HuskRig {
	/// Queues the ops that bring a husk of `vessel` dressed as `nft` into
	/// `scene`, hidden until it is placed. A creature is one NFT, its
	/// colours and the thing it wears; a rift's husk is the reliquary stack,
	/// lit in its first relic's light.
	pub fn spawn(scene: &Scene, vessel: Vessel, nft: &Nft) -> Self {
		let look = NftLook::of(nft);
		let model = scene.load(husk_model(vessel));
		let root = scene.group();
		let lift = scene.spawn_in(None, Some(root));
		let body = scene.spawn_in(Some(model), Some(lift));
		// The rules place the husk by the middle of its body, so the
		// renderer's own shadow, which falls at the origin, would fall
		// inside it: the model's own patch under its feet is used instead.
		scene.push(Op::SetShadow {
			node: body,
			cast: false,
		});
		for worn in 1..=NftLook::TRAIT_COUNT {
			scene.set_part_visible(body, format!("trait_{worn}"), worn == look.trait_index);
		}
		let light = linear(look.accent);
		let rig = Self {
			root,
			lift,
			body,
			vessel,
			look,
			light,
			reaction: HuskReaction::default(),
			away: (0.0, 1.0),
			seconds: 0.0,
			stride: 0.0,
			effort: 0.0,
			flash: 0.0,
			punch: 0.0,
			frost: 0.0,
			burn: 0.0,
			tuck: 0.0,
			leaping: false,
			grounded: true,
			dissolved: 0.0,
			placed: None,
			sent: None,
		};
		rig.style(scene, 1.0);
		scene.set_visible(root, false);
		rig
	}

	fn surface(&self, material: &str, eye_glow: f32) -> MaterialOverride {
		let tint = |colour| {
			let [red, green, blue] = linear(colour);
			Some([red, green, blue, 1.0])
		};
		let rim = |strength| {
			Some(Rim {
				color: self.light,
				strength,
				power: 2.5,
			})
		};
		let frost = self.frost as f32;
		let burn = self.burn as f32;
		let flash = self.flash as f32;
		let mut change = match material {
			"body" => {
				MaterialOverride {
					color: tint(self.look.body),
					rim: rim(RIM),
					..MaterialOverride::default()
				}
			}
			"trim" => {
				MaterialOverride {
					color: tint(self.look.trim),
					rim: rim(RIM),
					..MaterialOverride::default()
				}
			}
			"detail" => {
				MaterialOverride {
					color: tint(self.look.detail),
					rim: rim(RIM),
					..MaterialOverride::default()
				}
			}
			"dark" => {
				MaterialOverride {
					color: tint(0x0005_0507),
					rim: rim(0.15),
					..MaterialOverride::default()
				}
			}
			"accent" | "glow" => {
				let strength = if material == "accent" {
					ACCENT_GLOW
				} else {
					GLOW_GLOW
				};
				MaterialOverride {
					color: tint(0x0000_0000),
					emissive: Some(self.light),
					emissive_strength: Some(strength * eye_glow),
					..MaterialOverride::default()
				}
			}
			"canvas" => {
				MaterialOverride {
					rim: rim(0.25),
					..MaterialOverride::default()
				}
			}
			// The soft dark patch under a husk that stands on the floor.
			"shadow" => {
				return MaterialOverride {
					color: Some([0.0, 0.0, 0.0, 1.0]),
					emissive: Some([0.0; 3]),
					opacity: Some((0.55 * (1.0 - self.dissolved)) as f32),
					..MaterialOverride::default()
				};
			}
			_ => {
				MaterialOverride {
					color: tint(0x002B_2018),
					rim: rim(0.9),
					..MaterialOverride::default()
				}
			}
		};
		// Ice whitens and blues it; fire lights it from within; a strike
		// flashes it in its own light.
		if let Some(colour) = change.color.as_mut() {
			for (channel, ice) in colour.iter_mut().zip([0.55, 0.75, 0.95]) {
				*channel += (ice - *channel) * 0.6 * frost;
			}
		}
		if material != "accent" && material != "glow" {
			let ember = [1.0, 0.36, 0.08];
			change.emissive =
				Some([0, 1, 2].map(|channel| ember[channel] * burn + self.light[channel] * flash));
			change.emissive_strength = Some(1.5 * burn + 2.0 * flash);
		}
		change.opacity = Some((1.0 - self.dissolved) as f32);
		change
	}

	fn style(&self, scene: &Scene, eye_glow: f32) {
		for (material, nodes) in parts(&husk_model(self.vessel)) {
			let change = self.surface(material, eye_glow);
			for node in *nodes {
				scene.set_material(self.body, Some(node), change);
			}
		}
	}

	/// Puts the husk where the rules say it is, facing the way they say.
	pub fn place(&mut self, pose: &HuskPose) {
		let at = pose.position;
		self.leaping = pose.leaping;
		self.grounded = pose.ground.is_some() && !pose.leaping;
		let away = (pose.direction().x, pose.direction().z);
		let length = away.0.hypot(away.1);
		if length > 1e-6 {
			// Into the model's frame, which faces −z.
			let turn = std::f64::consts::PI - pose.heading;
			let (sin, cos) = turn.sin_cos();
			let (x, z) = (away.0 / length, away.1 / length);
			self.away = (x * cos + z * sin, -x * sin + z * cos);
		}
		self.placed = Some((
			// The hunt's space has z ahead of the player; the scene's, as
			// glTF's, has the viewer looking down −Z.
			[at.x as f32, at.y as f32, -at.z as f32],
			heading_yaw(pose.heading),
		));
	}

	/// A blow lands: on seal `seal` when it struck one, or the body as a
	/// whole. `strength` runs from 0 to 1, and a `heavy` blow stops time
	/// for an instant.
	pub fn hit(&mut self, seal: Option<usize>, strength: f64, heavy: bool) {
		let seals = self.vessel.seals();
		let widest = seals
			.iter()
			.fold(0.0_f64, |widest, spec| widest.max(spec.offset.x.abs()));
		let side = match seal {
			Some(seal) if widest > 0.0 && seal < seals.len() => seals[seal].offset.x / widest,
			_ => 0.0,
		};
		self.reaction
			.hit(self.away.1.atan2(self.away.0), side, strength, heavy);
	}

	/// Lights the husk up for an instant, as a strike lands. `strength` runs
	/// from 0 to 1.
	pub fn flash(&mut self, strength: f64) {
		self.flash = self.flash.max(strength);
		self.punch = self.punch.max(strength);
	}

	/// How much of the husk has come apart, from 0 (whole) to 1 (gone).
	pub fn dissolve(&mut self, amount: f64) {
		self.dissolved = amount.clamp(0.0, 1.0);
	}

	/// Advances everything about the husk that is only for show, and queues
	/// what changed.
	pub fn animate(&mut self, scene: &Scene, delta: f64, mood: HuskMood, plight: HuskPlight) {
		// Until it has somewhere to be, a husk is not there at all.
		if self.placed.is_none() {
			return;
		}
		let reaction = &mut self.reaction;
		reaction.step(delta, plight);
		let played = delta * reaction.time_scale();
		self.seconds += played;
		let fade = (-FLASH_DECAY * delta).exp();
		self.flash *= fade;
		self.punch *= fade;
		let approach = |value: &mut f64, target: f64, rate: f64| {
			*value += (target - *value) * (1.0 - (-rate * delta).exp());
		};
		approach(
			&mut self.frost,
			f64::from(u8::from(mood.frozen)),
			if mood.frozen { 6.0 } else { 2.5 },
		);
		approach(
			&mut self.burn,
			f64::from(u8::from(mood.burning)),
			if mood.burning { 10.0 } else { 3.0 },
		);
		approach(&mut self.tuck, f64::from(u8::from(self.leaping)), 14.0);
		let bolt = self.vessel.bolt_speed();
		let target = if bolt <= 0.0 {
			0.0
		} else {
			(mood.speed / bolt).clamp(0.0, 1.0)
		};
		approach(&mut self.effort, target, 8.0);
		self.stride += played * (2.2 + 9.0 * self.effort);
		let reaction = &self.reaction;
		let still = 1.0 - self.frost;
		let t = self.seconds * still;
		let half_height = self.vessel.half_height();
		let beats = 2.6 + 2.4 * self.effort + 4.0 * reaction.flail().abs();
		let step = if self.tuck > 0.5 {
			0.0
		} else if self.vessel.gait() == Gait::Air {
			0.02 * (t * std::f64::consts::TAU * beats).sin()
		} else {
			0.03 * self.effort * (self.stride * 2.0).sin().abs()
		};
		let lift = step + reaction.rise() * 2.0 * half_height * still;
		// Tipped by blows and staggering, and pulled over towards the
		// reaper as the reap drags it up the arc; turned about its feet so
		// that it rocks rather than spins about its middle.
		let lean_x = (reaction.lean_x() - self.away.0 * reaction.drag()) * still;
		let lean_z = (reaction.lean_z() - self.away.1 * reaction.drag()) * still;
		let squash = 1.0 + reaction.squash() * still;
		let feet = if self.vessel.gait() == Gait::Air {
			0.0
		} else {
			half_height
		};
		let twist = reaction.twist() * still
			+ 0.08 * self.effort * (self.stride).sin() * still
			+ 0.4 * reaction.flail() * still;
		let lift_transform = sway(lift, feet, lean_x, lean_z, twist, squash);
		let size = 1.0 + self.punch * 0.06;
		let placed = self.placed.unwrap_or(([0.0; 3], 0.0));
		let root = Transform::from_translation(placed.0[0], placed.0[1], placed.0[2])
			.with_yaw(placed.1)
			.with_scale(size as f32);
		let eye_glow = reaction.eye_glow();
		// Materials change in steps fine enough to see, coarse enough that
		// a husk at rest sends nothing.
		let quantum = |value: f64| (value * 40.0).round() as i32;
		let looks = [
			quantum(self.flash),
			quantum(self.frost),
			quantum(self.burn),
			quantum(self.dissolved),
			quantum(eye_glow),
			i32::from(self.grounded && self.dissolved < 1.0),
		];
		let changed = self.sent.map(|(_, _, sent)| sent != looks);
		if self.sent.map(|(sent, ..)| sent) != Some(root) {
			scene.set_transform(self.root, root);
		}
		if self.sent.map(|(_, sent, _)| sent) != Some(lift_transform) {
			scene.set_transform(self.lift, lift_transform);
		}
		if changed != Some(false) {
			self.style(scene, eye_glow as f32);
			scene.set_part_visible(self.body, "shadow", looks[5] == 1);
		}
		if self.sent.is_none() {
			scene.set_visible(self.root, true);
		}
		self.sent = Some((root, lift_transform, looks));
	}

	/// Removes the husk from the scene.
	pub fn remove(&self, scene: &Scene) {
		scene.remove(self.root);
	}
}

/// The turn about the scene's vertical that faces a husk along `heading`,
/// a bearing in the hunt's space (radians to the right of straight ahead).
///
/// The models face −Z. A heading of `h` faces the hunt's `(sin h, 0, cos h)`,
/// which is the scene's `(sin h, 0, −cos h)`, and a right-handed turn of
/// `−h` about +Y takes −Z there. So a husk coming straight at the player
/// (a heading of π) turns half round to face the camera.
pub fn heading_yaw(heading: f64) -> f32 {
	wrap_angle(-heading) as f32
}

/// The body's rise, lean, twist and squash, turned about its feet.
fn sway(lift: f64, feet: f64, lean_x: f64, lean_z: f64, twist: f64, squash: f64) -> Transform {
	let lean = lean_x.hypot(lean_z);
	// Tipping the top towards (x, z) is a turn about up × (x, z).
	let (axis, angle) = if lean > 1e-5 {
		([lean_z / lean, 0.0, -lean_x / lean], lean)
	} else {
		([1.0, 0.0, 0.0], 0.0)
	};
	let tip = axis_angle(axis, angle);
	let turn = axis_angle([0.0, 1.0, 0.0], twist);
	let rotation = multiply(tip, turn);
	// The feet, `feet` below the origin, stay put: the origin moves to
	// R·S·(0, feet, 0) above them.
	let up = super::camera::rotate(rotation, [0.0, (feet * squash) as f32, 0.0]);
	let spread = 1.0 / squash.sqrt();
	Transform {
		translation: [up[0], up[1] - feet as f32 + lift as f32, up[2]],
		rotation,
		scale: [spread as f32, squash as f32, spread as f32],
	}
}

fn axis_angle(axis: [f64; 3], angle: f64) -> [f32; 4] {
	let (sin, cos) = (angle * 0.5).sin_cos();
	[
		(axis[0] * sin) as f32,
		(axis[1] * sin) as f32,
		(axis[2] * sin) as f32,
		cos as f32,
	]
}

fn multiply(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
	let [ax, ay, az, aw] = a;
	let [bx, by, bz, bw] = b;
	[
		aw * bx + ax * bw + ay * bz - az * by,
		aw * by - ax * bz + ay * bw + az * bx,
		aw * bz + ax * by - ay * bx + az * bw,
		aw * bw - ax * bx - ay * by - az * bz,
	]
}

/// The way `heading` points, for the tests.
#[cfg(test)]
fn facing(rotation: [f32; 4]) -> [f32; 3] {
	super::camera::rotate(rotation, [0.0, 0.0, -1.0])
}

#[cfg(test)]
mod tests {
	use reapers_rules::market_listings;

	use super::*;
	use crate::render::model;

	#[test]
	fn every_vessel_has_a_model_with_its_traits_and_shadow() {
		for nft in market_listings() {
			let names = model::node_names(&husk_model(nft.collection.vessel()));
			if nft.collection.vessel().gait() != Gait::Air
				&& nft.collection.vessel() != Vessel::ReliquaryStack
			{
				assert!(names.contains(&"shadow".to_owned()), "{:?}", nft.collection);
			}
		}
	}

	#[test]
	fn a_husk_is_hidden_until_placed_and_then_sends_only_changes() {
		let scene = Scene::new(leptoskit::scene3d::SceneId(40));
		let nft = market_listings()[0];
		let mut rig = HuskRig::spawn(&scene, nft.collection.vessel(), &nft);
		let ops = scene.take_ops();
		assert!(
			ops.iter()
				.any(|op| matches!(op, Op::SetVisible { visible: false, .. }))
		);
		let pose = HuskPose::new(
			nft.collection.vessel(),
			reapers_rules::Vec3::new(0.0, 0.0, -2.0),
			std::f64::consts::PI,
		);
		rig.place(&pose);
		rig.animate(
			&scene,
			1.0 / 30.0,
			HuskMood::default(),
			HuskPlight::default(),
		);
		let first = scene.take_ops();
		assert!(
			first
				.iter()
				.any(|op| matches!(op, Op::SetVisible { visible: true, .. }))
		);
		for _ in 0..90 {
			rig.animate(
				&scene,
				1.0 / 30.0,
				HuskMood::default(),
				HuskPlight::default(),
			);
		}
		scene.take_ops();
		rig.animate(
			&scene,
			1.0 / 30.0,
			HuskMood::default(),
			HuskPlight::default(),
		);
		assert!(
			scene
				.take_ops()
				.iter()
				.all(|op| !matches!(op, Op::SetMaterial { .. })),
			"a husk at rest restyles nothing"
		);
		rig.flash(1.0);
		rig.animate(
			&scene,
			1.0 / 30.0,
			HuskMood::default(),
			HuskPlight::default(),
		);
		assert!(
			scene
				.take_ops()
				.iter()
				.any(|op| matches!(op, Op::SetMaterial { .. }))
		);
	}

	#[test]
	fn the_lean_turns_about_the_feet() {
		let still = sway(0.0, 0.5, 0.0, 0.0, 0.0, 1.0);
		assert_eq!(still.translation, [0.0, 0.0, 0.0]);
		let leaning = sway(0.0, 0.5, 0.3, 0.0, 0.0, 1.0);
		// The top tips towards +x about the feet, so the origin, half a body
		// above them, goes with it.
		assert!(leaning.translation[0] > 0.0);
		assert!(leaning.translation[1] < 0.0);
		let ahead = facing(Transform::IDENTITY.with_yaw(0.0).rotation);
		assert!((ahead[2] + 1.0).abs() < 1e-6);
	}
}
