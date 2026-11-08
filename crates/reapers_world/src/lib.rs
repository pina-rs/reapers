//! Where things are around a Reapers player: which way the phone points.
//!
//! None of it holds a camera, a sensor, a clock or a source of chance.
//! Sensor readings and times are passed in, so that all of it can be
//! tested, and so that it works out the same whether the game runs natively
//! or as Wasm.
//!
//! This is a port of the Flutter game's `hunt/pose`, and keeps its numbers.

mod math;

pub mod pose;

pub use pose::ArPose;
pub use pose::AttitudeFilter;
pub use pose::DragPose;
pub use pose::PoseSource;
pub use pose::SensorPose;
pub use pose::from_platform;
pub use pose::to_platform;
