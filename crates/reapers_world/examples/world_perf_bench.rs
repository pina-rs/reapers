//! Repeatable release performance benchmarks for `reapers_world`.
//!
//! Run with: `cargo run --release -p reapers_world --example perf_bench`

use std::hint::black_box;
use std::time::Instant;

use reapers_rules::Vessel;
use reapers_world::Block;
use reapers_world::DragPose;
use reapers_world::DrawnRoom;
use reapers_world::OcclusionMap;
use reapers_world::PoseSource;
use reapers_world::RoomGridBuilder;
use reapers_world::RoomSurvey;
use reapers_world::Shot;

fn median_ms(samples: &mut [f64]) -> f64 {
	samples.sort_by(f64::total_cmp);
	samples[samples.len() / 2]
}

#[allow(clippy::too_many_lines)]
fn main() {
	let room = DrawnRoom {
		eye_height: 1.45,
		wall: 4.2,
		room_yaw: 0.12,
		blocks: vec![
			Block {
				left: -0.6,
				right: 0.6,
				near: 2.3,
				far: 2.9,
				height: 0.75,
			},
			Block {
				left: -1.3,
				right: -0.7,
				near: 2.2,
				far: 2.8,
				height: 0.45,
			},
			Block {
				left: 1.2,
				right: 2.2,
				near: 1.6,
				far: 2.4,
				height: 0.90,
			},
		],
	};

	let shot = Shot {
		width: 252,
		height: 336,
		noise: 0.01,
		flying: 1,
		..Shot::default()
	};

	let yaws_deg = [-40.0, -20.0, 0.0, 20.0, 40.0, 60.0];
	let frames: Vec<_> = yaws_deg
		.iter()
		.enumerate()
		.map(|(i, &yaw)| {
			room.picture_with(
				yaw,
				-18.0,
				&Shot {
					seed: (i + 1) as i64,
					..shot
				},
			)
		})
		.collect();

	// M9: Full AR depth ingestion (RoomSurvey::add + RoomGridBuilder::add_frame) for 6 frames
	let mut m9_samples = Vec::with_capacity(7);
	for _ in 0..7 {
		let mut survey = RoomSurvey::new(Vessel::Ape);
		let mut grid = RoomGridBuilder::new();
		let t0 = Instant::now();
		for frame in &frames {
			let _ = survey.add_with_grid(frame, Some(&mut grid));
		}
		black_box((&survey, &grid));
		m9_samples.push(t0.elapsed().as_secs_f64() * 1e3 / frames.len() as f64);
	}
	let m9 = median_ms(&mut m9_samples);

	// M10: OcclusionMap::add per depth frame
	let mut m10_samples = Vec::with_capacity(11);
	for _ in 0..11 {
		let mut occ = OcclusionMap::new();
		let t0 = Instant::now();
		occ.add(&frames[2], 1.45, 1.0);
		black_box(&occ);
		m10_samples.push(t0.elapsed().as_secs_f64() * 1e3);
	}
	let m10 = median_ms(&mut m10_samples);

	// M11: RoomGridBuilder::finish latency after 6 frames
	let mut m11_samples = Vec::with_capacity(7);
	for _ in 0..7 {
		let mut grid = RoomGridBuilder::new();
		for frame in &frames {
			grid.add_frame(frame);
		}
		let t0 = Instant::now();
		let finished = grid.finish();
		black_box(finished);
		m11_samples.push(t0.elapsed().as_secs_f64() * 1e3);
	}
	let m11 = median_ms(&mut m11_samples);

	// M12: DragPose::pose + ArPose::project (10,000 iterations)
	let mut drag = DragPose::new();
	drag.drag(0.15, -0.08);
	let target = reapers_rules::Vec3::new(0.4, 0.2, 2.5);
	let mut m12_samples = Vec::with_capacity(15);
	for _ in 0..15 {
		let t0 = Instant::now();
		let mut acc = 0.0;
		for i in 0..10_000 {
			drag.drag(if i % 2 == 0 { 0.001 } else { -0.001 }, 0.0005);
			let pose = drag.pose();
			if let Some((x, y)) = pose.project(target, 1.08, 0.5) {
				acc += x + y;
			}
		}
		black_box(acc);
		m12_samples.push(t0.elapsed().as_secs_f64() * 1e9 / 10_000.0);
	}
	let m12 = median_ms(&mut m12_samples);

	println!("=== reapers_world perf_bench (release) ===");
	println!("M9_depth_ingest_ms:       {m9:.3} ms/frame (survey + grid, 252x336)");
	println!("M10_occlusion_add_ms:     {m10:.3} ms/frame (252x336)");
	println!("M11_room_grid_finish_ms:  {m11:.3} ms (6 frames -> RoomGrid)");
	println!("M12_pose_project_ns:      {m12:.1} ns/iter");
}
