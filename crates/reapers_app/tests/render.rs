//! A reaper and a husk drawn headless through `leptoskit_3d`, from the very
//! ops the game sends, out of the models the app bundles. Skipped where the
//! machine has no GPU adapter.
#![cfg(not(target_arch = "wasm32"))]
#![recursion_limit = "256"]

use std::sync::Arc;

use leptoskit_3d::Asset;
use leptoskit_3d::Renderer;
use leptoskit_3d::client::Scene;
use reapers_app::render::appearance::ReaperAppearance;
use reapers_app::render::camera::STAGE_DISTANCE;
use reapers_app::render::camera::stage_camera;
use reapers_app::render::rig::ReaperRig;
use reapers_app::render::scene::light_sanctum;
use reapers_rules::Outfit;
use reapers_rules::ReaperClass;
use reapers_rules::ReaperGenome;
use reapers_rules::Temper;
use reapers_rules::Weapon;

fn gpu() -> Option<leptoskit_3d::Gpu> {
	match leptoskit_3d::Gpu::headless() {
		Ok(gpu) => Some(gpu),
		Err(error) if error.is_unavailable() => {
			eprintln!("skipping: {error}");
			None
		}
		Err(error) => panic!("{error}"),
	}
}

/// Draws what `scene` queued, loading its models from the app's `assets/`,
/// after `seconds` of animation.
fn draw(gpu: &leptoskit_3d::Gpu, scene: &Scene, seconds: f32, size: (u32, u32)) -> Vec<u8> {
	let mut renderer = Renderer::new(gpu);
	let world = renderer.world_mut();
	world.apply(scene.take_ops());
	for (asset, path) in world.take_loads() {
		let file = format!("{}/assets/{path}", env!("CARGO_MANIFEST_DIR"));
		let bytes = std::fs::read(&file).unwrap_or_else(|error| panic!("{file}: {error}"));
		world.asset_loaded(asset, Ok(Arc::new(Asset::from_slice(&bytes).unwrap())));
	}
	let steps = (seconds * 60.0).round() as u32;
	for _ in 0..steps.max(1) {
		renderer.advance(1.0 / 60.0);
	}
	renderer.render_to_rgba(size.0, size.1).unwrap()
}

fn save(name: &str, pixels: &[u8], (width, height): (u32, u32)) {
	let path = format!("{}/{name}.png", env!("CARGO_TARGET_TMPDIR"));
	let file = std::fs::File::create(&path).unwrap();
	let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
	encoder.set_color(png::ColorType::Rgba);
	encoder.set_depth(png::BitDepth::Eight);
	encoder
		.write_header()
		.unwrap()
		.write_image_data(pixels)
		.unwrap();
	eprintln!("wrote {path}");
}

/// The share of covered pixels that are a loud orange or red: what the
/// models' mask colours looked like, drawn as colours.
fn loud_share(pixels: &[u8]) -> f64 {
	let mut covered = 0;
	let mut loud = 0;
	for pixel in pixels.as_chunks::<4>().0 {
		if pixel[3] < 200 {
			continue;
		}
		covered += 1;
		let (red, green, blue) = (
			i32::from(pixel[0]),
			i32::from(pixel[1]),
			i32::from(pixel[2]),
		);
		if red > 120 && red > 2 * green && red > 2 * blue {
			loud += 1;
		}
	}
	f64::from(loud) / f64::from(covered.max(1))
}

#[test]
fn a_reaper_stands_on_the_stage_in_its_own_colours() {
	let Some(gpu) = gpu() else {
		return;
	};
	let size = (240, 360);
	for calling in ReaperClass::ALL {
		let genome = ReaperGenome {
			reaper_class: calling,
			..ReaperGenome::shuffle(7)
		};
		let scene = Scene::new(leptoskit_3d::SceneId(1));
		light_sanctum(&scene);
		ReaperRig::spawn(
			&scene,
			None,
			genome,
			Weapon::new(calling.weapon(), Temper::Sworn),
			Outfit::Sworn,
			ReaperAppearance::default(),
		);
		scene.push(stage_camera(0.0, STAGE_DISTANCE, None));
		let pixels = draw(&gpu, &scene, 0.5, size);
		save(&format!("stage_{}", calling.name()), &pixels, size);
		let column =
			|x: u32| (0..size.1).any(|y| pixels[((y * size.0 + x) * 4 + 3) as usize] == 255);
		assert!(
			column(size.0 / 2),
			"{calling:?} stands in the middle of the stage"
		);
		assert!(
			!column(2) && !column(size.0 - 3),
			"{calling:?} fits the stage"
		);
		let loud = loud_share(&pixels);
		assert!(
			loud < 0.05,
			"{calling:?} is blotched: {loud:.3} of it is loud red"
		);
	}
}

#[test]
fn the_face_close_up_is_filled_by_the_head() {
	let Some(gpu) = gpu() else {
		return;
	};
	let size = (200, 200);
	let genome = ReaperGenome {
		reaper_class: ReaperClass::Weigher,
		..ReaperGenome::shuffle(3)
	};
	let scene = Scene::new(leptoskit_3d::SceneId(1));
	light_sanctum(&scene);
	ReaperRig::spawn(
		&scene,
		None,
		genome,
		Weapon::new(ReaperClass::Weigher.weapon(), Temper::Shrouded),
		Outfit::Shrouded,
		ReaperAppearance::default(),
	);
	scene.push(stage_camera(
		0.0,
		STAGE_DISTANCE,
		Some((
			genome.reaper_class,
			reapers_app::render::rig::stature(&genome),
		)),
	));
	let pixels = draw(&gpu, &scene, 0.5, size);
	save("face_weigher", &pixels, size);
	// The face sits in the middle of the close-up: the block round the
	// centre is all reaper.
	let middle = (size.0 / 2 - 10..size.0 / 2 + 10)
		.flat_map(|x| (size.1 / 2 - 10..size.1 / 2 + 10).map(move |y| (x, y)))
		.filter(|&(x, y)| pixels[((y * size.0 + x) * 4 + 3) as usize] == 255)
		.count();
	assert_eq!(middle, 400, "the face fills the middle of the close-up");
}

/// The share of covered pixels lit brighter than anything but a husk's own
/// glow: its eyes and seals.
fn glowing(pixels: &[u8]) -> usize {
	pixels
		.as_chunks::<4>()
		.0
		.iter()
		.filter(|pixel| {
			pixel[3] > 200 && u32::from(pixel[0]) + u32::from(pixel[1]) + u32::from(pixel[2]) > 520
		})
		.count()
}

#[test]
fn a_husk_coming_at_the_player_faces_them() {
	use reapers_app::hunt::arena::HUNT_FOV;
	use reapers_app::render::camera::look_at;
	use reapers_app::render::husk::HuskMood;
	use reapers_app::render::husk::HuskRig;
	use reapers_app::render::reaction::HuskPlight;
	use reapers_app::render::scene::light_hunt;
	use reapers_rules::HuskPose;
	use reapers_rules::Vec3;
	use reapers_rules::market_listings;

	let Some(gpu) = gpu() else {
		return;
	};
	let size = (200, 200);
	let nft = market_listings()[0];
	let vessel = nft.collection.vessel();
	let draw_heading = |heading: f64| {
		let scene = Scene::new(leptoskit_3d::SceneId(1));
		light_hunt(&scene, false);
		let mut rig = HuskRig::spawn(&scene, vessel, &nft);
		// Two metres straight ahead of the player, at its own height.
		rig.place(&HuskPose::new(vessel, Vec3::new(0.0, -0.8, 2.0), heading));
		rig.animate(
			&scene,
			1.0 / 30.0,
			HuskMood::default(),
			HuskPlight::default(),
		);
		scene.push(leptoskit_3d::Op::SetCamera {
			transform: look_at([0.0; 3], [0.0, -0.3, -1.0], [0.0, 1.0, 0.0]),
			fov_y: HUNT_FOV as f32,
			near: 0.05,
			far: 50.0,
		});
		draw(&gpu, &scene, 0.1, size)
	};
	let facing = draw_heading(std::f64::consts::PI);
	let away = draw_heading(0.0);
	save("husk_facing", &facing, size);
	save("husk_away", &away, size);
	assert!(
		glowing(&facing) > glowing(&away),
		"the eyes show from the front ({} lit) and not from behind ({} lit)",
		glowing(&facing),
		glowing(&away)
	);
}
