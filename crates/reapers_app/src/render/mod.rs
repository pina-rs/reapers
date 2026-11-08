//! 3D: the reapers and husks, their looks and motions, and the scenes they
//! stand in, drawn by `leptoskit_3d` on the host.
//!
//! The Flutter game drew on Flutter GPU (`flutter_scene`) with its own
//! shaders. Here a scene lives on the host, which loads the models,
//! animates them on its own display clock and draws them; this module says
//! what is in each scene, with ops, and keeps the little state that must
//! outlive a render.

pub mod appearance;
pub mod camera;
pub mod cloth;
pub mod husk;
pub mod look;
#[cfg(test)]
pub mod model;
pub mod parts;
pub mod playback;
pub mod pointer;
pub mod reaction;
pub mod rig;
pub mod scene;
pub mod silhouette;
pub mod stage;
pub mod ticker;
