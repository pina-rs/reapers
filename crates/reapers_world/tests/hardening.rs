//! Regression tests for `reapers_world` hardening, normal-based ledge detection, and scale accuracy.

use reapers_rules::GeoFix;
use reapers_rules::GeoPoint;
use reapers_rules::HuntLoadout;
use reapers_rules::RiftEntrant;
use reapers_rules::RiftRoom;
use reapers_rules::RiftSite;
use reapers_rules::Timestamp;
use reapers_rules::Vec3;
use reapers_rules::Vessel;
use reapers_rules::radians;
use reapers_world::ArPose;
use reapers_world::AttitudeFilter;
use reapers_world::Block;
use reapers_world::DepthFrame;
use reapers_world::DrawnRoom;
use reapers_world::RayGrid;
use reapers_world::RayHit;
use reapers_world::RiftPlacer;
use reapers_world::RoomGridBuilder;
use reapers_world::RoomMap;
use reapers_world::Shot;

#[test]
fn elevated_origin_never_panics_with_negative_ring_index() {
	let room = DrawnRoom::default();
	// Camera eye lifted above the room's own floor height: previously produced a
	// negative floor unit and wrapped `ring as usize` to `usize::MAX`.
	let mut frame = room.picture(0.0, -30.0);
	frame.origin = Vec3::new(0.0, 2.5, 0.0);

	let mut map = RoomMap::new(Vessel::Penguin);
	assert!(!map.add(&frame));
	assert_eq!(map.frames(), 0);

	// Also ensure measured_floor + origin.y <= 0 is safely rejected.
	assert!(map.measure_floor(1.45));
	let mut low_frame = room.picture(0.0, -30.0);
	low_frame.origin = Vec3::new(0.0, -2.0, 0.0);
	assert!(!map.add(&low_frame));
}

#[test]
fn scaled_depth_model_does_not_drop_counters_in_room_map() {
	// A 0.90 m kitchen counter 2.2 m ahead: for a Penguin (top = 1.10 m), a depth
	// model reading 1.45x long used to compute unscaled `above = 1.305 m > 1.10 m`
	// and drop the counter from `taken`.
	let room = DrawnRoom::with(&[Block {
		left: -1.5,
		right: 1.5,
		near: 2.2,
		far: 2.8,
		height: 0.90,
	}]);
	let mut map = RoomMap::new(Vessel::Penguin);
	assert!(map.measure_floor(room.eye_height));

	for yaw in [-30.0, 0.0, 30.0] {
		let mut frame = room.picture(yaw, -28.0);
		for depth in &mut frame.depths {
			*depth *= 1.45;
		}
		assert!(map.add_estimated(&frame));
	}

	let straight_slice = reapers_rules::HuntPlan::slice_of(0.0);
	let reach = map
		.reach(straight_slice)
		.expect("straight-ahead slice must be read");
	assert!(
		reach < 2.6,
		"counter must stop clear floor near 2.2 m, got reach {reach:.2} m"
	);
}

#[test]
fn sparse_ray_grid_with_surface_normals_reconstructs_horizontal_ledge() {
	let pose = ArPose::looking(Vec3::from_angles(0.0, radians(-28.0)), 0.0);
	// A 12x16 sparse grid cannot pass 3x3 slope continuity on a narrow 1-row ledge,
	// so `from_rays` drops it while `from_ray_hits` with normals reconstructs it.
	let grid = RayGrid {
		pose,
		tan_half_width: 0.41,
		tan_half_height: 0.56,
		columns: 12,
		rows: 16,
	};

	let mut hits: Vec<Option<RayHit>> = vec![None; grid.columns * grid.rows];
	let ledge_row = 7;
	for col in 3..9 {
		let ray = grid.ray(col, ledge_row);
		// Horizontal counter top 0.55 m below eye (0.90 m above a 1.45 m floor).
		let dist = (-0.55) / ray.y;
		hits[ledge_row * grid.columns + col] = Some(RayHit {
			distance: dist,
			normal: Some(Vec3::UP),
		});
	}

	let distances: Vec<Option<f64>> = hits.iter().map(|h| h.map(|r| r.distance)).collect();
	let without_normals = DepthFrame::from_rays(&grid, &distances);
	let with_normals = DepthFrame::from_ray_hits(&grid, &hits);

	let count_finite = |f: &DepthFrame| f.depths.iter().filter(|d| d.is_finite()).count();
	assert_eq!(count_finite(&without_normals), 0);
	assert!(
		count_finite(&with_normals) > 200,
		"expected single-row horizontal ledge reconstruction with normals, got {} finite pixels",
		count_finite(&with_normals)
	);
}

#[test]
fn pose_and_filter_reject_zero_quaternions_invalid_fov_and_nan_dt() {
	let zero_pose = ArPose::from_quaternion([0.0, 0.0, 0.0, 0.0], 0.0);
	assert!(zero_pose.forward.length().is_finite());
	assert!((zero_pose.forward.length() - 1.0).abs() < 1e-6);

	assert!(zero_pose.project(Vec3::FORWARD, 0.0, 1.0).is_none());
	assert!(zero_pose.project(Vec3::FORWARD, f64::NAN, 1.0).is_none());
	assert!(zero_pose.project(Vec3::FORWARD, 1.0, 0.0).is_none());
	assert!(zero_pose.project(Vec3::FORWARD, 1.0, -1.0).is_none());

	let mut filter = AttitudeFilter::new();
	filter.add_acceleration(0.0, AttitudeFilter::STANDARD_GRAVITY, 0.0);
	let before = filter.pose();
	filter.add_rotation(1.0, 0.0, 0.0, f64::NAN);
	filter.add_rotation(1.0, 0.0, 0.0, -0.1);
	filter.add_rotation(f64::NAN, 0.0, 0.0, 0.016);
	assert_eq!(filter.pose(), before);
}

#[test]
fn rift_placer_stays_looking_when_stored_fix_is_stale_or_coarse() {
	let now = Timestamp::utc(2026, 10, 5, 12, 0, 0);
	let mut room = RiftRoom::new(RiftSite::LondonBridge, 3);
	let seat = room
		.join(RiftEntrant {
			name: "Vesper".to_owned(),
			loadout: HuntLoadout::BORROWED,
		})
		.unwrap();
	let frame = room.frame_for(seat).unwrap();

	let mut placer = RiftPlacer::new();
	placer.fix = Some(GeoFix {
		at: GeoPoint::new(52.2, 0.12),
		accuracy: 200.0, // too coarse
		observed_at: now,
	});
	assert_eq!(placer.observe(&frame, 0.0, Some(0.0), now), None);
	assert!(
		placer.still_looking(),
		"placer must keep looking when the only fix received is unusable"
	);
}

#[test]
fn drawn_room_smear_flying_handles_multi_pixel_vertical_blur_without_corrupting_floor() {
	let room = DrawnRoom::with(&[Block {
		left: -0.6,
		right: 0.6,
		near: 2.2,
		far: 2.8,
		height: 0.75,
	}]);
	let shot = Shot {
		flying: 3,
		..Shot::default()
	};
	let frame = room.picture_with(0.0, -28.0, &shot);
	let mut builder = RoomGridBuilder::new();
	assert!(builder.add_frame(&frame));
	let grid = builder.finish();
	assert!(grid.is_well_formed());
}
