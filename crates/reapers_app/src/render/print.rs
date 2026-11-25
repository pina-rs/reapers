//! Second Ink as the renderer prints it (art bible §4 and §5): how a scene
//! is lit, lined and hatched for the way its figures are framed, and what
//! every part of every model prints as.
//!
//! A model prints by material. Each material is named for a family
//! ([`crate::ink::Family`]) and the parts drawn in it are listed in
//! [`super::parts`], read from the models themselves, so a model rebuilt
//! with new parts or materials prints them as soon as its table is listed
//! again. The first-person models are the one exception: their parts share
//! one palette material, a strip of colour cells that print cannot read, so
//! each part prints whole in the family most of it is made of ([`palette`]).

use leptoskit::scene3d::Ink;
use leptoskit::scene3d::Light;
use leptoskit::scene3d::MAX_INK_WIDTH;
use leptoskit::scene3d::NodeId;
use leptoskit::scene3d::Print;
use leptoskit::scene3d::PrintMode;
use leptoskit::scene3d::PrintOverride;
use leptoskit::scene3d::PrintTarget;
use leptoskit::scene3d::Shading;
use leptoskit::scene3d::Style;
use leptoskit::scene3d::client::Scene;

use super::parts::parts;
use crate::ink;
use crate::ink::Family;
use crate::ink::Printed;
use crate::ink::ReaperInk;
use crate::theme::Rgb;

// Framing ----------------------------------------------------------------------

/// How a scene frames its figures, which sets its light, line and hatch
/// (§4.2, §5.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Framing {
	/// The sanctum's hero: one reaper about 400 pt tall, lit from behind so
	/// the rim carries the silhouette.
	Hero,
	/// A reaper about 300 pt tall, lit from the front so the costume reads:
	/// the armoury, the oath and the studio.
	Figure,
	/// A husk in the hunt or a rift, 200 to 300 pt tall, lit over the
	/// player's shoulder so its mouth and seals always read.
	Hunt,
	/// The first-person arms and what they hold.
	FirstPerson,
}

/// One framing's numbers, from cel.toml's `[scenes]`.
#[derive(Clone, Copy, Debug)]
struct Plate {
	/// The way to the light, fixed to the camera: right, up, and toward the
	/// viewer.
	toward: [f32; 3],
	t_lit: f32,
	t_deep: f32,
	t_cross: f32,
	rim: f32,
	/// The key line, in points on the phone the bible measures.
	line_pt: f32,
	hatch_per_m: f32,
	/// How tall the view the figure is drawn in is, as a share of the
	/// screen's height: the sanctum's stage is `h-[52vh]`; the armoury's,
	/// the oath's and the studio's are 44 to 46vh.
	view: f32,
}

/// How tall the bible's phone is, in points.
const PHONE_PT: f32 = 844.0;
/// The most a line grows on a taller screen, against its width on the
/// phone (§4.2).
const MOST_GROWTH: f32 = 1.5;
/// The paper knockout outside every figure's key line (§4.1).
const KNOCKOUT_PT: f32 = 1.75;
/// How tall a reaper and a husk stand, in metres, for the hatch to land
/// its pitch apart at its framing (cel.toml).
const REAPER_M: f32 = 2.2;
const HUSK_M: f32 = 0.9;
/// How far the key line wanders along its length (§4.3).
const WOBBLE: f32 = 0.22;

/// Hatch lines a metre on a figure `size` metres tall drawn `figure_pt`
/// tall, so they land `pitch_pt` apart (§5.2).
const fn hatch(figure_pt: f32, size: f32, pitch_pt: f32) -> f32 {
	figure_pt / (size * pitch_pt)
}

impl Framing {
	pub const ALL: [Self; 4] = [Self::Hero, Self::Figure, Self::Hunt, Self::FirstPerson];

	const fn plate(self) -> Plate {
		match self {
			Self::Hero => {
				Plate {
					toward: [-0.80, 0.50, -0.32],
					t_lit: 0.25,
					t_deep: -0.12,
					t_cross: -0.55,
					rim: 0.34,
					line_pt: 2.6,
					hatch_per_m: hatch(400.0, REAPER_M, 2.6),
					view: 0.52,
				}
			}
			Self::Figure => {
				Plate {
					toward: [-0.86, 0.40, 0.32],
					t_lit: 0.30,
					t_deep: -0.10,
					t_cross: -0.55,
					rim: 0.30,
					line_pt: 2.4,
					hatch_per_m: hatch(300.0, REAPER_M, 2.7),
					view: 0.45,
				}
			}
			// No crosshatch and no rim on the dead.
			Self::Hunt => {
				Plate {
					toward: [-0.78, 0.48, 0.40],
					t_lit: 0.38,
					t_deep: 0.0,
					t_cross: -2.0,
					rim: 0.0,
					line_pt: 2.25,
					hatch_per_m: hatch(250.0, HUSK_M, 2.6),
					view: 1.0,
				}
			}
			Self::FirstPerson => {
				Plate {
					toward: [-0.55, 0.75, 0.35],
					t_lit: 0.30,
					t_deep: -0.12,
					t_cross: -0.55,
					rim: 0.30,
					line_pt: 3.3,
					hatch_per_m: 480.0,
					view: 1.0,
				}
			}
		}
	}

	/// The one light, fixed to the camera and never to the room (§5.4). In
	/// print its colour and strength are the bands' thresholds, so they are
	/// plain; it casts no soft shadow, which print has no tone for.
	pub fn light(self) -> Light {
		let [right, up, toward] = self.plate().toward;
		let length = (right * right + up * up + toward * toward).sqrt();
		Light {
			// The way the light travels: away from where it is.
			direction: [-right / length, -up / length, -toward / length],
			color: [1.0; 3],
			intensity: 1.0,
			ambient: 0.0,
			shadow: 0.0,
			camera_space: true,
		}
	}

	/// `pt` points on the phone, in thousandths of this framing's view's
	/// height on a screen `screen_pt` points tall: the same share of the
	/// screen on every phone, never more than half again as wide on a tablet
	/// (§4.2).
	pub fn permille(self, pt: f32, screen_pt: f32) -> f32 {
		let screen = screen_pt.max(1.0);
		let drawn = pt * (screen / PHONE_PT).min(MOST_GROWTH);
		(drawn / (self.plate().view * screen) * 1000.0).min(MAX_INK_WIDTH)
	}

	/// The print and the line, on a screen `screen_pt` points tall.
	pub fn style(self, screen_pt: f32) -> Style {
		let plate = self.plate();
		Style {
			shading: Shading::Print(Print {
				t_lit: plate.t_lit,
				t_deep: plate.t_deep,
				t_cross: plate.t_cross,
				rim: plate.rim,
				paper: ink::linear(ink::PAPER),
				knockout_width: self.permille(KNOCKOUT_PT, screen_pt),
				hatch_per_m: plate.hatch_per_m,
				..Print::default()
			}),
			ink: Some(Ink {
				color: ink::linear(ink::KEY),
				width: self.permille(plate.line_pt, screen_pt),
				facing: 0.0,
				shade_weight: 0.0,
				wobble: WOBBLE,
				boil: 0.0,
				seed: 0,
			}),
		}
	}

	/// Lights and styles `scene` for this framing, on this screen.
	pub fn apply(self, scene: &Scene) {
		let screen_pt = crate::platform::viewport().1 as f32;
		scene.set_style(self.style(screen_pt));
		scene.set_light(self.light());
	}
}

// Surfaces ---------------------------------------------------------------------

/// What one material, or one palette part, prints as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
	/// A family, in the figure's light.
	Family(Family),
	/// One of a husk's own colours, washed (§3.6).
	Own(Own),
	/// A relic's portrait. Print shows no picture, so it prints as paper.
	Image,
	/// The patch under a husk: flat key, see-through, with no line and out
	/// of the knockout.
	Shadow,
	/// Light an effect throws: flat in the figure's light, with no line.
	Light,
	/// The dark disc an effect lays behind a glow, which the printed halo
	/// replaces (§5.5): never shown in print.
	Shade,
}

/// A husk's own colours, from its NFT's look.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Own {
	Body,
	Trim,
	Detail,
}

/// What a material called `material` prints as; on a husk, `body`, `trim`
/// and `detail` are its NFT's colours. A name the bible does not know
/// prints as cloth (§3.3).
pub fn surface(material: &str, husk: bool) -> Surface {
	let name = material.split('.').next().unwrap_or(material);
	// Names that print as another family.
	let name = match name {
		"frame" => "metal",
		"shroud" => "cloth",
		"core" => "void",
		other => other,
	};
	match name {
		"canvas" | "paper_image" | "shard" | "ribbon" => Surface::Image,
		"shadow" => Surface::Shadow,
		"fx_glow" | "fx_light" => Surface::Light,
		"fx_shade" => Surface::Shade,
		"body" if husk => Surface::Own(Own::Body),
		"trim" if husk => Surface::Own(Own::Trim),
		"detail" if husk => Surface::Own(Own::Detail),
		other => Surface::Family(Family::named(other).unwrap_or(Family::Cloth)),
	}
}

/// The first-person models' palette material.
pub const PALETTE_MATERIAL: &str = "viewmodel";

/// Each part drawn in the palette, whole, in the family most of it is made
/// of (`art/blender/viewmodel_parts`). The bone forearms print as their
/// sleeves, and the pistol's bone grip, gilt band and chambers as its iron.
const PALETTE: [(&str, Family); 11] = [
	// The bone hand.
	("arm_r", Family::Bone),
	// The sleeve, and the forearm's bones inside it.
	("arm_r_forearm", Family::Cloth),
	("arm_l", Family::Cloth),
	("pistol", Family::Metal),
	("pistol_drum", Family::Metal),
	// One long fang.
	("knife", Family::Bone),
	("scythe", Family::Bone),
	// The orb's iron cup, held and thrown.
	("orb_held", Family::Metal),
	("orb_body", Family::Metal),
	("orb_sigil_ring", Family::Gold),
	("soul_cylinder", Family::Metal),
];

/// The family palette part `part` prints in; cloth for one not listed.
pub fn palette(part: &str) -> Family {
	PALETTE
		.iter()
		.find(|(listed, _)| *listed == part)
		.map_or(Family::Cloth, |(_, family)| *family)
}

/// Where each print override on the model at `model` goes, and what it
/// prints: by material, and part by part on the palette.
pub fn plan(model: &str) -> Vec<(PrintTarget, Surface)> {
	let husk = model.starts_with("models/husk_");
	let mut plan = Vec::new();
	for (material, parts) in parts(model) {
		if *material == PALETTE_MATERIAL {
			plan.extend(parts.iter().map(|part| {
				(
					PrintTarget::Part((*part).to_owned()),
					Surface::Family(palette(part)),
				)
			}));
		} else {
			plan.push((
				PrintTarget::Material((*material).to_owned()),
				surface(material, husk),
			));
		}
	}
	plan
}

// Inking -----------------------------------------------------------------------

/// Whose figure a model is, which decides how its surfaces print.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Inking {
	/// A reaper, or the arms and the light they throw, in its light.
	Reaper(ReaperInk),
	/// A husk.
	Husk(HuskInk),
}

/// How one husk prints: its NFT's colours, washed for whoever hunts it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HuskInk {
	pub body: Rgb,
	pub trim: Rgb,
	pub detail: Rgb,
	/// The light of the reaper who sees it, for the clash rule.
	pub player: Option<Rgb>,
	/// A blow prints it in this spot, the light touching the dead, and
	/// presses its line (§10.3, §11.2).
	pub struck: Option<Rgb>,
}

/// How much heavier a struck husk's line is: a pressed frame (§11.2).
const STRUCK_LINE: f64 = 1.25;

impl Inking {
	/// The spot its rim prints in.
	fn spot(&self) -> Rgb {
		match self {
			Self::Reaper(reaper) => reaper.spot(),
			Self::Husk(_) => ink::PAPER,
		}
	}

	/// What `surface` prints as on this figure; nothing for a surface print
	/// never shows.
	pub fn printed(&self, surface: Surface) -> Option<Printed> {
		let flat = |colour: Rgb| {
			Printed {
				lit: colour,
				shade: colour,
				deep: colour,
				flat: true,
				hatch: 0.0,
				cross: false,
				specular: 0.0,
				rim: false,
				line: 0.0,
			}
		};
		let printed = match (self, surface) {
			(_, Surface::Shade) => return None,
			// The dead have no light: what would glow prints key.
			(_, Surface::Shadow) | (Self::Husk(_), Surface::Light) => flat(ink::KEY),
			(Self::Reaper(reaper), Surface::Light) => flat(reaper.spot()),
			(Self::Reaper(reaper), Surface::Family(family)) => reaper.print(family),
			// A reaper has no colours of its own: they print as its robe.
			(Self::Reaper(reaper), Surface::Own(_)) => reaper.print(Family::Cloth),
			(Self::Reaper(reaper), Surface::Image) => reaper.print(Family::Paper),
			(Self::Husk(husk), surface) => return Some(husk.printed(surface)),
		};
		Some(printed)
	}
}

impl HuskInk {
	fn printed(&self, surface: Surface) -> Printed {
		let family = match surface {
			Surface::Family(family) => family,
			Surface::Image => Family::Paper,
			_ => Family::Cloth,
		};
		if let Some(spot) = self.struck {
			// Every surface but its eyes and seals prints as gilt in the
			// reaper's light: spot, with key over it for shade and deep.
			if !matches!(
				family,
				Family::Accent | Family::Glow | Family::Void | Family::Dark
			) {
				let gilt = ReaperInk {
					light: spot,
					robe: ink::Robe::Night,
					omen: reapers_rules::Omen::None,
					outfit: reapers_rules::Outfit::Sworn,
					first_person: false,
				}
				.print(Family::Gold);
				return Printed {
					cross: false,
					rim: false,
					specular: 0.0,
					line: STRUCK_LINE,
					..gilt
				};
			}
		}
		match surface {
			Surface::Own(own) => {
				let colour = match own {
					Own::Body => self.body,
					Own::Trim => self.trim,
					Own::Detail => self.detail,
				};
				ink::ghost(colour, self.player)
			}
			_ => ink::husk(family),
		}
	}
}

/// The override that prints `printed`, its rim in `spot`.
pub fn print_override(printed: &Printed, spot: Rgb) -> PrintOverride {
	PrintOverride {
		lit: Some(ink::linear(printed.lit)),
		shade: Some(ink::linear(printed.shade)),
		deep: Some(ink::linear(printed.deep)),
		spot: Some(ink::linear(spot)),
		mode: Some(if printed.flat {
			PrintMode::Flat
		} else {
			PrintMode::Banded
		}),
		hatch_min: Some(printed.hatch as f32),
		hatch_per_m: None,
		cross_hatch: Some(printed.cross),
		specular: Some(printed.specular as f32),
		rim: Some(printed.rim),
		ink_weight: Some(printed.line as f32),
		knockout: None,
		boil_step: None,
	}
}

/// Prints every part of `node`, an instance of the model at `model`, as
/// `inking` says. A shadow or an effect's light stays out of the paper
/// knockout: they are not the figure.
pub fn print(scene: &Scene, node: NodeId, model: &str, inking: &Inking) {
	let spot = inking.spot();
	for (target, surface) in plan(model) {
		let Some(printed) = inking.printed(surface) else {
			continue;
		};
		let mut print = print_override(&printed, spot);
		if matches!(surface, Surface::Shadow | Surface::Light) {
			print.knockout = Some(false);
		}
		scene.set_print(node, target, print);
	}
}

#[cfg(test)]
mod tests {
	use reapers_rules::Omen;
	use reapers_rules::Outfit;
	use reapers_rules::ReaperClass;
	use reapers_rules::ReaperGenome;

	use super::*;
	use crate::render::parts::MODELS;

	#[test]
	fn every_part_of_every_shipped_model_prints_in_a_family() {
		for model in MODELS {
			let plan = plan(model);
			assert!(!plan.is_empty(), "{model} prints nothing");
			for (material, parts) in parts(model) {
				if *material == PALETTE_MATERIAL {
					for part in *parts {
						assert!(
							PALETTE.iter().any(|(listed, _)| listed == part),
							"{model}'s {part} is on the palette but has no family"
						);
					}
					continue;
				}
				let husk = model.starts_with("models/husk_");
				let name = material.split('.').next().unwrap_or(material);
				let known = matches!(
					surface(material, husk),
					Surface::Own(_)
						| Surface::Image
						| Surface::Shadow
						| Surface::Light
						| Surface::Shade
				) || Family::named(name).is_some()
					|| ["frame", "shroud", "core"].contains(&name);
				assert!(
					known,
					"{model}'s {material} names no family: it would print as cloth"
				);
			}
		}
	}

	#[test]
	fn the_palette_lists_only_parts_the_models_have() {
		let palette_parts: Vec<&str> = MODELS
			.iter()
			.flat_map(|model| parts(model).iter())
			.filter(|(material, _)| *material == PALETTE_MATERIAL)
			.flat_map(|(_, parts)| parts.iter().copied())
			.collect();
		for (part, _) in PALETTE {
			assert!(palette_parts.contains(&part), "no model has {part}");
		}
	}

	#[test]
	fn names_print_as_the_bible_says() {
		assert_eq!(surface("metal.001", false), Surface::Family(Family::Metal));
		assert_eq!(surface("frame", true), Surface::Family(Family::Metal));
		assert_eq!(surface("core", false), Surface::Family(Family::Void));
		assert_eq!(surface("anything", false), Surface::Family(Family::Cloth));
		assert_eq!(surface("body", true), Surface::Own(Own::Body));
		assert_eq!(surface("body", false), Surface::Family(Family::Cloth));
		assert_eq!(surface("canvas", true), Surface::Image);
		assert_eq!(surface("fx_glow", false), Surface::Light);
		assert_eq!(surface("fx_shade", false), Surface::Shade);
		assert_eq!(surface("shadow", true), Surface::Shadow);
	}

	#[test]
	fn each_framing_is_the_bibles() {
		// §4.2: the ‰ of a full-screen view on the phone.
		let phone = PHONE_PT;
		let hunt = Framing::Hunt.style(phone);
		let Shading::Print(print) = hunt.shading else {
			panic!("the hunt prints")
		};
		assert!((hunt.ink.unwrap().width - 2.67).abs() < 0.01);
		assert!((print.knockout_width - 2.07).abs() < 0.01);
		assert!(
			print.t_cross <= -1.5 && print.rim == 0.0,
			"none on the dead"
		);
		assert!((Framing::FirstPerson.permille(3.3, phone) - 3.91).abs() < 0.01);
		// The hero's 2.6 pt line, drawn in a view 52% of the screen tall.
		assert!((Framing::Hero.permille(2.6, phone) * 0.52 - 3.08).abs() < 0.01);
		// A taller phone keeps the share of the screen; a tablet stops at
		// half again the phone's width.
		assert_eq!(
			Framing::Hunt.permille(2.25, 932.0),
			Framing::Hunt.permille(2.25, phone)
		);
		let tablet = Framing::Hunt.permille(2.25, 1366.0) / 1000.0 * 1366.0;
		assert!((tablet - 2.25 * 1.5).abs() < 0.01, "{tablet} pt");
		// The hatch lands 2.6 pt apart on a 250 pt, 0.9 m husk.
		assert!((print.hatch_per_m - 106.8).abs() < 0.1);
	}

	#[test]
	fn each_light_is_fixed_to_the_camera_and_falls_from_where_the_bible_puts_it() {
		for framing in Framing::ALL {
			let light = framing.light();
			assert!(light.camera_space);
			let toward = framing.plate().toward;
			let length = toward.iter().map(|axis| axis * axis).sum::<f32>().sqrt();
			for axis in 0..3 {
				assert!((light.direction[axis] + toward[axis] / length).abs() < 1e-6);
			}
		}
		// The hunt's light comes over the left shoulder, toward the viewer.
		let hunt = Framing::Hunt.light().direction;
		assert!(hunt[0] > 0.0 && hunt[1] < 0.0 && hunt[2] < 0.0);
		// The hero is lit from behind.
		assert!(Framing::Hero.light().direction[2] > 0.0);
	}

	#[test]
	fn a_reaper_prints_its_own_light_and_a_husk_none() {
		let genome = ReaperGenome {
			reaper_class: ReaperClass::Ferryman,
			palette: 0,
			omen: Omen::None,
			..ReaperGenome::BORROWED
		};
		let reaper = Inking::Reaper(ReaperInk::of(&genome, Outfit::Sworn));
		let cloth = reaper.printed(Surface::Family(Family::Cloth)).unwrap();
		assert_eq!(cloth.lit, Rgb::from_hex(0x008C_4D28));
		let light = reaper.printed(Surface::Light).unwrap();
		assert!(light.flat && light.line == 0.0);
		assert_eq!(light.lit, ink::LANTERN);
		assert_eq!(reaper.printed(Surface::Shade), None);
		let husk = HuskInk {
			body: Rgb::from_hex(0x00C9_722E),
			trim: Rgb::from_hex(0x00F2_EEE6),
			detail: Rgb::from_hex(0x003B_3F46),
			player: Some(ink::LANTERN),
			struck: None,
		};
		let dead = Inking::Husk(husk);
		let body = dead.printed(Surface::Own(Own::Body)).unwrap();
		assert_eq!(body, ink::ghost(husk.body, Some(ink::LANTERN)));
		assert_eq!(
			dead.printed(Surface::Family(Family::Accent)).unwrap().lit,
			ink::PAPER
		);
		// Struck, it prints in the reaper's spot with a pressed line, but its
		// eyes stay paper.
		let struck = Inking::Husk(HuskInk {
			struck: Some(ink::LANTERN),
			..husk
		});
		let body = struck.printed(Surface::Own(Own::Body)).unwrap();
		assert_eq!(body.lit, ink::LANTERN);
		assert!(body.line > 1.0);
		assert_eq!(
			struck.printed(Surface::Family(Family::Accent)).unwrap().lit,
			ink::PAPER
		);
	}

	#[test]
	fn printing_a_model_sets_every_material_and_keeps_effects_out_of_the_knockout() {
		let scene = Scene::new(leptoskit::scene3d::SceneId(90));
		let node = scene.group();
		let inking = Inking::Reaper(ReaperInk {
			first_person: true,
			..ReaperInk::of(&ReaperGenome::BORROWED, Outfit::Sworn)
		});
		print(&scene, node, "models/viewmodel.glb", &inking);
		let ops = scene.take_ops();
		let prints: Vec<(PrintTarget, PrintOverride)> = ops
			.into_iter()
			.filter_map(|op| {
				match op {
					leptoskit::scene3d::Op::SetPrint { target, print, .. } => Some((target, print)),
					_ => None,
				}
			})
			.collect();
		// The flashes and the trail, and each of the nine palette parts.
		assert_eq!(prints.len(), 10);
		let (_, glow) = prints
			.iter()
			.find(|(target, _)| *target == PrintTarget::Material("fx_glow".to_owned()))
			.unwrap();
		assert_eq!(glow.knockout, Some(false));
		assert_eq!(glow.mode, Some(PrintMode::Flat));
		let (_, hand) = prints
			.iter()
			.find(|(target, _)| *target == PrintTarget::Part("arm_r".to_owned()))
			.unwrap();
		assert_eq!(
			hand.lit,
			Some(ink::linear(ink::PAPER)),
			"a bone hand is paper"
		);
	}
}
