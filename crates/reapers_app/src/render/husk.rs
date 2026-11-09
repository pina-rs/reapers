//! One husk: a collection's creature, dressed as one NFT. The Flutter
//! `HuskRig`.
//!
//! The rig only ever draws what the rules say. It is told where the husk is
//! with [`HuskRig::place`], what it is doing with [`HuskRig::animate`], and
//! what happened with [`HuskRig::flash`], [`HuskRig::hit`] and
//! [`HuskRig::dissolve`]; it decides nothing about the hunt. How it acts that
//! out is [`super::reaction::HuskReaction`]'s.
//!
//! A creature is built from rigid parts on pivots, and has no clips. The
//! creature moves whole: it rises and falls with each step or wingbeat,
//! leans and twists from blows, squashes as it is struck, and is dragged up
//! the reaping arc. Its own pivots move by part offsets, which the host
//! eases between ticks: its legs keep step with the ground it covers, it
//! crouches, stretches, tucks and reaches through a leap, and its head
//! looks where it is going ([`super::gait`]). Its shadow stays on the
//! surface under it, whatever the body does above.

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

use super::gait::Carriage;
use super::gait::Footwork;
use super::gait::MAX_LIMBS;
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

/// How dark a shadow is at its darkest, under a body standing on it.
const SHADOW_OPACITY: f64 = 0.55;
/// The shadow's strength and width under feet that touch the ground,
/// and under a body [`SHADOW_FADE_HEIGHT`] or more above it: tight and dark
/// at contact, broad and faint in the air.
const SHADOW_CONTACT: f64 = 1.15;
const SHADOW_FAINTEST: f64 = 0.35;
const SHADOW_TIGHTEST: f64 = 0.9;
const SHADOW_WIDEST: f64 = 1.5;
const SHADOW_FADE_HEIGHT: f64 = 0.8;

/// No turn at all.
const IDENTITY: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

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
	/// The model's shadow, a second instance under the root so the lift
	/// cannot tip or squash it. Every part but its own patch is hidden.
	shadow: NodeId,
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
	/// How strong its shadow is against [`SHADOW_OPACITY`], by how high the
	/// body is above it.
	shadow_fade: f64,
	/// Where the rules last put it, and which way it faces.
	placed: Option<([f32; 3], f32)>,
	/// What the rules last said of it.
	pose: Option<HuskPose>,
	/// Its walk, its leaps and its gaze.
	gait: Footwork,
	/// The last way the body was carried, kept while frozen so its legs and
	/// gaze hold where the ice caught them.
	carriage: Carriage,
	/// Whether its model has a shadow to keep on the ground.
	has_shadow: bool,
	/// The shadow's place, kept for the tests that hold it flat.
	shadow_placed: Transform,
	/// The part offsets last sent, so a still husk sends none: each limb,
	/// the head, the neck, and the shadow's place and look.
	posed: Posed,
	/// Where the player stands in the scene's space: zero, unless the
	/// platform draws the scene from its own view of the room.
	origin: [f32; 3],
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
		// It hangs from the root, not the lift, so a lean or a squash never
		// tips it, and its instance draws nothing but that patch.
		let shadow = scene.spawn_in(Some(model), Some(root));
		scene.push(Op::SetShadow {
			node: body,
			cast: false,
		});
		scene.push(Op::SetShadow {
			node: shadow,
			cast: false,
		});
		scene.set_part_visible(body, "shadow", false);
		for (material, nodes) in parts(&husk_model(vessel)) {
			if *material == "shadow" {
				continue;
			}
			for node in *nodes {
				scene.set_part_visible(shadow, *node, false);
			}
		}
		for worn in 1..=NftLook::TRAIT_COUNT {
			scene.set_part_visible(body, format!("trait_{worn}"), worn == look.trait_index);
		}
		let light = linear(look.accent);
		let rig = Self {
			root,
			lift,
			body,
			shadow,
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
			shadow_fade: 1.0,
			placed: None,
			pose: None,
			gait: Footwork::new(vessel),
			carriage: Carriage::REST,
			has_shadow: parts(&husk_model(vessel))
				.iter()
				.any(|(material, _)| *material == "shadow"),
			shadow_placed: Transform::IDENTITY,
			posed: Posed::default(),
			origin: [0.0; 3],
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
					opacity: Some(
						(SHADOW_OPACITY * self.shadow_fade * (1.0 - self.dissolved)).min(1.0)
							as f32,
					),
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
				// The body's own patch is hidden: the shadow is its own
				// instance, out of the lift's way.
				if *material == "shadow" {
					scene.set_material(self.shadow, Some(node), change);
				} else {
					scene.set_material(self.body, Some(node), change);
				}
			}
		}
	}

	/// Puts the husk where the rules say it is, facing the way they say.
	pub fn place(&mut self, pose: &HuskPose) {
		self.pose = Some(*pose);
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

	/// Stands the player at `origin` in the scene's space. The rules place a
	/// husk around a player at the origin; where the platform draws the
	/// scene over its camera picture, from where the phone really is, the
	/// husk is placed around the phone instead.
	pub fn set_origin(&mut self, origin: [f32; 3]) {
		self.origin = origin;
	}

	/// Where the player stands in the scene's space: see
	/// [`set_origin`](Self::set_origin).
	pub const fn origin(&self) -> [f32; 3] {
		self.origin
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

	/// How the body is carried about its feet this frame: its step's rise
	/// and fall, the lean and twist of blows and of its own gait, and its
	/// squash and stretch.
	fn carry(&self, carriage: &Carriage) -> Transform {
		let walks = self
			.gait
			.build()
			.is_some_and(|build| !build.limbs.is_empty());
		let reaction = &self.reaction;
		let still = 1.0 - self.frost;
		let t = self.seconds * still;
		let half_height = self.vessel.half_height();
		let beats = 2.6 + 2.4 * self.effort + 4.0 * reaction.flail().abs();
		let step = if self.tuck > 0.5 {
			0.0
		} else if self.vessel.gait() == Gait::Air {
			0.02 * (t * std::f64::consts::TAU * beats).sin()
		} else if walks {
			carriage.bob * half_height
		} else {
			0.03 * self.effort * (self.stride * 2.0).sin().abs()
		};
		let lift = step + reaction.rise() * 2.0 * half_height * still;
		// Tipped by blows and staggering, and pulled over towards the
		// reaper as the reap drags it up the arc; turned about its feet so
		// that it rocks rather than spins about its middle. Its own gait
		// rolls it from foot to foot and tips it into a leap.
		let lean_x = (reaction.lean_x() - self.away.0 * reaction.drag() + carriage.roll) * still;
		let lean_z = (reaction.lean_z() - self.away.1 * reaction.drag() + carriage.pitch) * still;
		let squash = (1.0 + reaction.squash() * still) * carriage.squash;
		let feet = if self.vessel.gait() == Gait::Air {
			0.0
		} else {
			half_height
		};
		let twist = reaction.twist() * still
			+ 0.08 * self.effort * (self.stride).sin() * still
			+ 0.4 * reaction.flail() * still;
		sway(lift, feet, lean_x, lean_z, twist, squash)
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
		let frozen = mood.frozen;
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
		if !frozen {
			// Ice holds the body as it was: nothing that only time or a
			// settling foot swings goes on while it is frozen. The host
			// finishes easing the last part offsets it was given and holds
			// them there.
			self.seconds += played;
			approach(&mut self.tuck, f64::from(u8::from(self.leaping)), 14.0);
			let bolt = self.vessel.bolt_speed();
			let target = if bolt <= 0.0 {
				0.0
			} else {
				(mood.speed / bolt).clamp(0.0, 1.0)
			};
			approach(&mut self.effort, target, 8.0);
			self.stride += played * (2.2 + 9.0 * self.effort);
		}
		let carriage = match self.pose {
			Some(pose) if !frozen => {
				let carriage = self.gait.step(&pose, played);

				self.carriage = carriage;

				carriage
			}
			Some(_) => self.carriage,
			None => Carriage::REST,
		};
		let lift_transform = self.carry(&carriage);
		let reaction = &self.reaction;
		// The rules' size for the body: a husk on a room's grid is the
		// smaller creature of the v2 table.
		let size = self.pose.map_or(1.0, |pose| pose.scale) * (1.0 + self.punch * 0.06);
		let placed = self.placed.unwrap_or(([0.0; 3], 0.0));
		let root = Transform::from_translation(
			self.origin[0] + placed.0[0],
			self.origin[1] + placed.0[1],
			self.origin[2] + placed.0[2],
		)
		.with_yaw(placed.1)
		.with_scale(size as f32);
		let eye_glow = reaction.eye_glow();
		let ground = self.pose.and_then(|pose| pose.ground);
		// A husk walking the room's grid keeps its shadow through a leap,
		// on the surface it left until it crosses onto the next.
		let shadowed = match self.pose.and_then(|pose| pose.locomotion) {
			Some(_) => ground.is_some(),
			None => self.grounded,
		};
		let shadow = self.has_shadow
			&& shadowed
			&& self.dissolved < 1.0
			&& ground.is_some_and(|ground| self.pin_shadow(scene, ground, placed.0[1], size));
		// Materials change in steps fine enough to see, coarse enough that
		// a husk at rest sends nothing.
		let quantum = |value: f64| (value * 40.0).round() as i32;
		let looks = [
			quantum(self.flash),
			quantum(self.frost),
			quantum(self.burn),
			quantum(self.dissolved),
			quantum(eye_glow),
			i32::from(shadow),
		];
		let changed = self.sent.map(|(_, _, sent)| sent != looks);
		if self.sent.map(|(sent, ..)| sent) != Some(root) {
			scene.set_transform(self.root, root);
		}
		if self.sent.map(|(_, sent, _)| sent) != Some(lift_transform) {
			scene.set_transform(self.lift, lift_transform);
		}
		let shadow_look = quantum(self.shadow_fade);
		if changed != Some(false) {
			self.style(scene, eye_glow as f32);
			scene.set_visible(self.shadow, looks[5] == 1);
		} else if shadow && self.posed.shadow_look != shadow_look {
			scene.set_material(
				self.shadow,
				Some("shadow"),
				self.surface("shadow", eye_glow as f32),
			);
		}
		self.posed.shadow_look = shadow_look;
		if !frozen {
			self.pose_parts(scene, &carriage, delta);
		}
		if self.sent.is_none() {
			scene.set_visible(self.root, true);
		}
		self.sent = Some((root, lift_transform, looks));
	}

	/// Sends the limbs' swings and the head's gaze that changed, for the
	/// host to ease to over `delta`.
	fn pose_parts(&mut self, scene: &Scene, carriage: &Carriage, delta: f64) {
		let Some(build) = self.gait.build() else {
			return;
		};
		let seconds = delta.clamp(0.0, 0.1) as f32;
		for (slot, limb) in build.limbs.iter().take(MAX_LIMBS).enumerate() {
			let (swing, spread) = carriage.limbs[slot];
			let rotation = multiply(
				axis_angle([1.0, 0.0, 0.0], swing),
				axis_angle([0.0, 0.0, 1.0], spread),
			);
			let key = quantize(rotation);
			if self.posed.limbs[slot] != key {
				self.posed.limbs[slot] = key;
				scene.set_part_offset(self.body, limb.part, turn(rotation), seconds);
			}
		}
		let (right, down) = carriage.look;
		// A long neck takes the tip and the head the turn; otherwise the
		// head takes both.
		let (head, neck) = match build.neck {
			Some(_) => {
				(
					axis_angle([0.0, 1.0, 0.0], -right),
					axis_angle([1.0, 0.0, 0.0], down),
				)
			}
			None => {
				(
					multiply(
						axis_angle([0.0, 1.0, 0.0], -right),
						axis_angle([1.0, 0.0, 0.0], down),
					),
					IDENTITY,
				)
			}
		};
		let key = quantize(head);
		if self.posed.head != key {
			self.posed.head = key;
			scene.set_part_offset(self.body, build.head, turn(head), seconds);
		}
		if let Some(part) = build.neck {
			let key = quantize(neck);
			if self.posed.neck != key {
				self.posed.neck = key;
				scene.set_part_offset(self.body, part, turn(neck), seconds);
			}
		}
	}

	/// Keeps the model's shadow flat on the surface `ground` metres below the
	/// hunt's origin, wherever the body above it is: a husk `centre` metres
	/// up, drawn `size` times its model. The shadow hangs from the root, so
	/// the lift's tip, twist and squash never reach it; it grows larger and
	/// fainter as the body rises off it. Answers whether it is drawn.
	fn pin_shadow(&mut self, scene: &Scene, ground: f64, centre: f32, size: f64) -> bool {
		if size <= 1e-6 {
			return false;
		}
		let feet = self.vessel.half_height();
		// The shadow's patch lies `feet` below the model's origin, a little
		// clear of it so it is not lost in the surface. The root-local place
		// its origin must take to put the patch on the ground.
		let down = (ground - f64::from(centre)) / size;
		let above = (-(down + feet)).max(0.0) * size;
		let risen = (above / SHADOW_FADE_HEIGHT).min(1.0);
		self.shadow_fade = SHADOW_CONTACT - (SHADOW_CONTACT - SHADOW_FAINTEST) * risen;
		let grow = SHADOW_TIGHTEST + (SHADOW_WIDEST - SHADOW_TIGHTEST) * risen;
		let place = Transform {
			translation: [0.0, (down + feet) as f32, 0.0],
			rotation: IDENTITY,
			scale: [grow as f32, 1.0, grow as f32],
		};
		self.shadow_placed = place;
		let key = quantize_transform(&place);
		if self.posed.shadow != Some(key) {
			self.posed.shadow = Some(key);
			// The surface under a husk changes at once, never in between.
			scene.set_transform(self.shadow, place);
		}
		true
	}

	/// Removes the husk from the scene.
	pub fn remove(&self, scene: &Scene) {
		scene.remove(self.root);
	}
}

/// The turn about the scene's vertical that faces a husk along `heading`,
/// a bearing in the hunt's space (radians to the right of straight ahead).
///
/// The models face +Z, as glTF's do. A heading of `h` faces the hunt's
/// `(sin h, 0, cos h)`, which is the scene's `(sin h, 0, −cos h)`, and a
/// right-handed turn of `π − h` about +Y takes +Z there. So a husk coming
/// straight at the player (a heading of π) needs no turn at all.
pub fn heading_yaw(heading: f64) -> f32 {
	wrap_angle(std::f64::consts::PI - heading) as f32
}

/// The part offsets a rig last sent, in the steps [`quantize`] takes them
/// to: a limb, the head or the shadow is sent again only when it moved by
/// more than a step.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Posed {
	limbs: [[i32; 4]; MAX_LIMBS],
	head: [i32; 4],
	neck: [i32; 4],
	shadow: Option<[i32; 10]>,
	shadow_look: i32,
}

impl Default for Posed {
	/// Every pivot at rest, as the model has it: nothing needs sending.
	fn default() -> Self {
		let rest = quantize(IDENTITY);
		Self {
			limbs: [rest; MAX_LIMBS],
			head: rest,
			neck: rest,
			shadow: None,
			shadow_look: -1,
		}
	}
}

/// A turn, in steps of about a twentieth of a degree.
fn quantize(rotation: [f32; 4]) -> [i32; 4] {
	rotation.map(|value| (value * 2000.0).round() as i32)
}

/// A whole offset, in steps of half a millimetre, a twentieth of a degree
/// and a thousandth of its size.
fn quantize_transform(transform: &Transform) -> [i32; 10] {
	let mut key = [0; 10];
	for (slot, value) in key.iter_mut().zip(
		transform
			.translation
			.iter()
			.chain(&transform.rotation)
			.chain(&transform.scale),
	) {
		*slot = (value * 2000.0).round() as i32;
	}
	key
}

/// A part turned by `rotation` about its own pivot, and moved no further.
const fn turn(rotation: [f32; 4]) -> Transform {
	Transform {
		translation: [0.0; 3],
		rotation,
		scale: [1.0; 3],
	}
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

	/// Where `point`, in the model's own frame, ends up in the scene.
	fn apply(transform: &Transform, point: [f32; 3]) -> [f32; 3] {
		let scaled = [0, 1, 2].map(|axis| point[axis] * transform.scale[axis]);
		let turned = crate::render::camera::rotate(transform.rotation, scaled);
		[0, 1, 2].map(|axis| turned[axis] + transform.translation[axis])
	}

	/// A penguin of the room's size, `state` and `fraction` through it,
	/// with its middle `centre` metres from the eye over a surface at
	/// `support`.
	fn walker(
		state: reapers_rules::HuskMotionState,
		fraction: f64,
		centre: f64,
		support: f64,
	) -> HuskPose {
		HuskPose {
			scale: 0.53,
			ground: Some(support),
			leaping: state == reapers_rules::HuskMotionState::Air,
			locomotion: Some(reapers_rules::HuskLocomotion {
				state,
				state_fraction: fraction,
				support_height: support,
				next_foothold: None,
			}),
			..HuskPose::new(
				Vessel::Penguin,
				reapers_rules::Vec3::new(0.3, centre, 2.4),
				std::f64::consts::PI,
			)
		}
	}

	/// Where three separated points of the shadow's patch lie in the scene,
	/// after `rig` was placed at `pose` and animated.
	fn shadow_plane(scene: &Scene, rig: &mut HuskRig, pose: &HuskPose) -> [f32; 3] {
		rig.place(pose);
		rig.animate(
			scene,
			1.0 / 30.0,
			HuskMood::default(),
			HuskPlight::default(),
		);
		scene.take_ops();
		let root = rig.sent.expect("the husk was placed").0;
		// Three separated corners of the patch, which lies flat at the
		// model's feet, a little clear of the ground.
		let at = |x: f32, z: f32| apply(&root, apply(&rig.shadow_placed, [x, -0.446, z]))[1];

		[at(0.3, 0.2), at(-0.25, 0.25), at(0.05, -0.3)]
	}

	#[test]
	fn the_shadow_stays_flat_on_the_surface_under_the_husk() {
		use reapers_rules::HuskMotionState::Air;
		use reapers_rules::HuskMotionState::Crouch;

		let scene = Scene::new(leptoskit::scene3d::SceneId(41));
		let nft = market_listings()
			.into_iter()
			.find(|nft| nft.collection.vessel() == Vessel::Penguin)
			.unwrap();
		let mut rig = HuskRig::spawn(&scene, Vessel::Penguin, &nft);
		scene.take_ops();
		let size = 0.53;
		let clear = 0.004 * size;
		// High over the desk it left: every point of the patch is on the
		// desk, a little clear of it.
		let heights = shadow_plane(&scene, &mut rig, &walker(Air, 0.5, -0.2, -0.7));
		assert!(
			heights
				.iter()
				.all(|y| (*y - (-0.7 + clear as f32)).abs() < 1e-4),
			"{heights:?} over the desk"
		);
		// The clear space is its own thing: the surface offset is real, and
		// not something flatness could hide.
		assert!(clear > 1e-3, "{clear} m clear of the surface");
		// Over the edge, the surface under it is the floor, at once.
		let heights = shadow_plane(&scene, &mut rig, &walker(Air, 0.6, -0.4, -1.45));
		assert!(
			heights
				.iter()
				.all(|y| (*y - (-1.45 + clear as f32)).abs() < 1e-4),
			"{heights:?} over the floor"
		);
		// Gathered, squashed and tipped into a leap, it neither leans nor
		// sinks with the body: the lift's pitch and squash cannot reach it.
		for _ in 0..10 {
			shadow_plane(&scene, &mut rig, &walker(Crouch, 0.9, -1.25, -1.45));
		}
		let heights = shadow_plane(&scene, &mut rig, &walker(Crouch, 1.0, -1.25, -1.45));
		let lowest = heights.iter().copied().fold(f32::INFINITY, f32::min);
		let highest = heights.iter().copied().fold(f32::NEG_INFINITY, f32::max);
		assert!(
			highest - lowest < 1e-4,
			"a crouched, pitched husk tilts its shadow by {} m",
			highest - lowest
		);
		assert!(
			(lowest - (-1.45 + clear as f32)).abs() < 1e-4,
			"{heights:?} over the floor"
		);
		// Its middle stays under the husk, whatever the lean did to the body.
		let root = rig.sent.expect("the husk was placed").0;
		let middle = apply(&root, apply(&rig.shadow_placed, [0.0, -0.446, 0.0]));
		assert!((middle[0] - root.translation[0]).abs() < 1e-4);
		assert!((middle[2] - root.translation[2]).abs() < 1e-4);
	}

	#[test]
	fn a_frozen_husk_holds_its_limbs_and_gaze_until_it_thaws() {
		use reapers_rules::HuskLocomotion;
		use reapers_rules::Vec3;

		let scene = Scene::new(leptoskit::scene3d::SceneId(43));
		let nft = market_listings()
			.into_iter()
			.find(|nft| nft.collection.vessel() == Vessel::Llama)
			.unwrap();
		let mut rig = HuskRig::spawn(&scene, Vessel::Llama, &nft);
		scene.take_ops();
		let scale = Vessel::Llama.locomotion().body_height / (2.0 * Vessel::Llama.half_height());
		let walking = |z: f64| {
			HuskPose {
				scale,
				ground: Some(-1.45),
				locomotion: Some(HuskLocomotion {
					state: reapers_rules::HuskMotionState::Walk,
					state_fraction: 0.0,
					support_height: -1.45,
					next_foothold: Some(Vec3::new(0.9, -1.0, 3.2)),
				}),
				..HuskPose::new(Vessel::Llama, Vec3::new(0.0, -1.2, z), std::f64::consts::PI)
			}
		};
		let mut z = 2.2;
		let (mut moved_limbs, mut moved_gaze, mut eased) = (false, false, false);
		for _ in 0..45 {
			z += 0.02;
			let pose = walking(z);
			rig.place(&pose);
			rig.animate(
				&scene,
				1.0 / 30.0,
				HuskMood::default(),
				HuskPlight::default(),
			);
			for op in scene.take_ops() {
				if let Op::SetPartOffset { part, seconds, .. } = op {
					let gait = part.starts_with("leg_") || part.starts_with("arm_");
					let gaze = part == "head" || part == "neck";

					moved_limbs |= gait;
					moved_gaze |= gaze;
					eased |= (gait || gaze) && seconds > 0.0;
				}
			}
		}
		assert!(moved_limbs && moved_gaze, "the llama walks and looks");
		assert!(eased, "the host was easing the offsets when the ice came");

		let frozen = HuskMood {
			frozen: true,
			..HuskMood::default()
		};
		let ice = HuskPlight {
			frozen: true,
			..HuskPlight::default()
		};
		let held = walking(z);
		for frame in 0..60 {
			rig.place(&held);
			rig.animate(&scene, 1.0 / 30.0, frozen, ice);
			let ops = scene.take_ops();
			assert!(
				ops.iter().all(|op| !matches!(op, Op::SetPartOffset { .. })),
				"frame {frame} moved a limb or the gaze while frozen"
			);
		}
		let mut resumed = false;
		for _ in 0..20 {
			rig.place(&held);
			rig.animate(
				&scene,
				1.0 / 30.0,
				HuskMood::default(),
				HuskPlight::default(),
			);
			resumed |= scene
				.take_ops()
				.iter()
				.any(|op| matches!(op, Op::SetPartOffset { .. }));
		}
		assert!(resumed, "the gait resumes once the ice thaws");
	}

	#[test]
	fn a_walking_husk_moves_its_legs_and_a_still_one_does_not() {
		let scene = Scene::new(leptoskit::scene3d::SceneId(42));
		let nft = market_listings()
			.into_iter()
			.find(|nft| nft.collection.vessel() == Vessel::Penguin)
			.unwrap();
		let mut rig = HuskRig::spawn(&scene, Vessel::Penguin, &nft);
		scene.take_ops();
		let mut moved_legs = false;
		for frame in 0..30 {
			let mut pose = walker(reapers_rules::HuskMotionState::Walk, 0.0, -1.21, -1.45);
			pose.position.x += 0.015 * f64::from(frame);
			rig.place(&pose);
			rig.animate(
				&scene,
				1.0 / 30.0,
				HuskMood::default(),
				HuskPlight::default(),
			);
			moved_legs |= scene.take_ops().iter().any(
				|op| matches!(op, Op::SetPartOffset { part, seconds, .. } if part == "leg_l" && *seconds > 0.0),
			);
		}
		assert!(moved_legs, "the legs keep step");
		let still = walker(reapers_rules::HuskMotionState::Idle, 0.0, -1.21, -1.45);
		for _ in 0..90 {
			rig.place(&still);
			rig.animate(
				&scene,
				1.0 / 30.0,
				HuskMood::default(),
				HuskPlight::default(),
			);
		}
		scene.take_ops();
		rig.place(&still);
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
				.all(|op| !matches!(op, Op::SetPartOffset { .. })),
			"a husk standing still sends no part"
		);
	}

	#[test]
	fn every_pivot_a_creature_walks_on_is_in_its_model() {
		for vessel in [
			Vessel::Ape,
			Vessel::Bird,
			Vessel::Llama,
			Vessel::Penguin,
			Vessel::Robot,
		] {
			let build = crate::render::gait::build(vessel).unwrap();
			let names = model::node_names(&husk_model(vessel));
			let pivots = build
				.limbs
				.iter()
				.map(|limb| limb.part)
				.chain([build.head])
				.chain(build.neck);
			for pivot in pivots {
				assert!(
					names.iter().any(|name| name == pivot),
					"{vessel:?} has no {pivot}"
				);
			}
		}
	}
}
