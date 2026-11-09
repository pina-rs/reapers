//! The room a husk moves through, and how it moves through it.
//!
//! A hunt is played inside a real room, read once from the camera before
//! the fight. What the phone saw becomes a [`RoomGrid`]: a gravity-aligned
//! grid of 10 centimetre cells, each holding a level top, an obstacle or a
//! wall. From the grid and a [`LocomotionProfile`] the room's walkable
//! surfaces, the leaps between them, and the places worth standing are
//! derived, once, in [`RoomNav`]. A husk's body is then moved a tick at a
//! time by [`HuskMotion`]: walking surfaces at a gait, and ballistic leaps
//! under real gravity, solved exactly.
//!
//! Everything here is deterministic: integer grid work wherever possible,
//! floating point only through addition, subtraction, multiplication,
//! division and [`crate::math`], ties broken by cell index, and no clock
//! and no chance but a seed passed in. The server replays hunts, so the
//! same room, the same body and the same commands must land on the same
//! bits on any device.
//!
//! Nothing here reads the camera or draws a picture. A [`crate::HuntSim`]
//! whose plan carries a grid moves a husk that walks through it; see
//! [`crate::HuntPlan::room`].

pub mod husk_motion;
pub mod leap;
pub mod locomotion_profile;
pub mod room_grid;
pub mod room_nav;
pub mod room_path;

pub use husk_motion::HuskMotion;
pub use husk_motion::HuskMotionState;
pub use husk_motion::MotionPace;
pub use leap::Leap;
pub use leap::LeapRules;
pub use locomotion_profile::LocomotionProfile;
pub use room_grid::RoomCell;
pub use room_grid::RoomCellKind;
pub use room_grid::RoomGrid;
pub use room_nav::JumpLink;
pub use room_nav::RoomNav;
pub use room_nav::RoomPath;
pub use room_nav::RoomRegion;
pub use room_nav::RoomSpot;
pub use room_nav::RoomSpotKind;
pub use room_nav::RoomStep;
