//! Where the 3D and camera views plug in.
//!
//! Every screen that showed a reaper, a husk or the camera in the Flutter
//! game calls one of these components instead. Each has a props struct that
//! is its whole contract, and a 2D stand-in that honours it, so the screens
//! around it are finished and testable now. The port of `render/`,
//! `studio/`, `hunt/` and `rift/` onto `leptoskit_3d` and `leptoskit_xr`
//! replaces the bodies of these functions and nothing else: no screen,
//! keeper or route needs to change.
//!
//! | Seam | Flutter | Replaced by |
//! | --- | --- | --- |
//! | [`reaper_stage`] | `render/reaper_stage*.dart` | a `leptoskit_3d` scene of the reaper's GLB |
//! | [`husk_portrait`] | `widgets/nft_thumbnail.dart`, `render/nft_image.dart` | the bundled portrait, natively too |
//! | [`hunt_view`] | `hunt/hunt_screen.dart`'s arena | `leptoskit_xr`'s camera, pose and planes, and the 3D husk |
//! | [`rift_view`] | `rift/rift_screen.dart` | the same arena in rift mode |
//! | [`studio_view`] | `studio/reaper_studio.dart` | the studio on `leptoskit_3d` |

mod hunt;
mod husk;
mod reaper;
mod rift;
mod studio;

pub use hunt::HuntBrief;
pub use hunt::HuntReward;
pub use hunt::HuntViewProps;
pub use hunt::autoplay_log;
pub use hunt::hunt_view;
pub use husk::HuskPortraitProps;
pub use husk::husk_portrait;
pub use husk::portrait_source;
pub use reaper::ReaperStageProps;
pub use reaper::reaper_stage;
pub use rift::RiftViewProps;
pub use rift::rift_view;
pub use studio::StudioViewProps;
pub use studio::studio_view;
