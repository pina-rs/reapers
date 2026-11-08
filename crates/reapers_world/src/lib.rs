//! Where things are around a Reapers player: which way the phone points,
//! what stands in the room around them, and where a rift stands on the
//! earth.
//!
//! None of it holds a camera, a sensor, a clock or a source of chance.
//! Sensor readings, depth pictures and times are passed in, so that all of
//! it can be tested against rooms drawn by hand, and so that the room a
//! phone reads becomes the same [`HuntPlan`] whether the game runs natively
//! or as Wasm.
//!
//! This is a port of the Flutter game's `hunt/pose`, `hunt/scan` and
//! `rift/place`, and keeps their numbers. One thing is new: depth comes
//! from the platform's own AR, a ray at a time, instead of a depth model
//! (see [`RayGrid`] and [`RoomMap::measure_floor`]).
//!
//! [`HuntPlan`]: reapers_rules::HuntPlan

mod math;

pub mod depth_frame;
pub mod drawn_room;
pub mod occlusion_map;
pub mod pose;
pub mod rift_placer;
pub mod room_map;
pub mod room_survey;

pub use depth_frame::DepthFrame;
pub use depth_frame::RayGrid;
pub use drawn_room::Block;
pub use drawn_room::DrawnRoom;
pub use drawn_room::Shot;
pub use occlusion_map::OcclusionMap;
pub use pose::ArPose;
pub use pose::AttitudeFilter;
pub use pose::DragPose;
pub use pose::PoseSource;
pub use pose::SensorPose;
pub use pose::from_platform;
pub use pose::to_platform;
pub use rift_placer::RiftPlacer;
pub use room_map::Furniture;
pub use room_map::RoomMap;
pub use room_survey::RoomSurvey;
pub use room_survey::can_read;
pub use room_survey::sigil_plan;
