//! Rules, catalogues and reaper generation shared by the Reapers game and
//! its server.
//!
//! A rule lives here exactly once. The game plays by these rules offline,
//! and the server judges an online player against the same code.
//!
//! The simulation is deterministic: [`HuntSim`], [`PlayerState`] and
//! [`RiftRoom`] read no clock and roll no dice of their own. Seeds and times
//! are passed in, because the server replays hunts and a replay must match.
//! Every sine and square root is taken through one implementation, so the
//! server and the game, built for different targets, land on the same bits.
//!
//! The rules were first written in another language, and were ported here
//! keeping their numbers, their order of operations and their save format,
//! so that every save, hunt log and receipt written before the port reads
//! and replays the same way. `tests/conformance.rs` holds them to fixtures
//! that first implementation wrote.

mod error;
mod math;
mod named;
mod parse;

pub mod armory;
pub mod arts;
pub mod feeding;
pub mod geo;
pub mod hunt;
pub mod nft_catalogue;
pub mod nft_look;
pub mod player_state;
pub mod progression;
pub mod reaper_class;
pub mod reaper_genome;
pub mod reaper_motion;
pub mod reaper_names;
pub mod reaper_soul;
pub mod reaper_state;
pub mod rift;
pub mod seeded_random;
pub mod time;
pub mod vec3;
pub mod vessel;

pub use armory::Outfit;
pub use armory::Temper;
pub use armory::Weapon;
pub use armory::WeaponProfile;
pub use armory::armory_weapons;
pub use armory::weapon_profile;
pub use arts::Art;
pub use arts::ArtRules;
pub use arts::ArtSkill;
pub use error::FormatError;
pub use feeding::Feeding;
pub use geo::GeoFix;
pub use geo::GeoPoint;
pub use geo::GeoStanding;
pub use hunt::art_flight::ArtFlight;
pub use hunt::difficulty::Difficulty;
pub use hunt::hunt_bot::HuntBot;
pub use hunt::hunt_bot::HuntBotStyle;
pub use hunt::hunt_event::EscapeCause;
pub use hunt::hunt_event::HuntEnd;
pub use hunt::hunt_event::HuntEvent;
pub use hunt::hunt_event::HuntOutcome;
pub use hunt::hunt_event::HuntPhase;
pub use hunt::hunt_event::ReapGrade;
pub use hunt::hunt_input::Aim;
pub use hunt::hunt_input::HuntInput;
pub use hunt::hunt_input::HuntLog;
pub use hunt::hunt_input::HuntLogError;
pub use hunt::hunt_plan::HuntPlan;
pub use hunt::hunt_sim::HuntSim;
pub use hunt::hunt_sim::IllFormedPlan;
pub use hunt::hunt_stats::HuntLoadout;
pub use hunt::hunt_stats::HuntRules;
pub use hunt::hunt_stats::HuntStats;
pub use hunt::hunt_stats::InvalidLoadout;
pub use hunt::husk_pose::Footing;
pub use hunt::husk_pose::HuskPose;
pub use hunt::sigil::Sigil;
pub use hunt::sigil::SigilEvent;
pub use hunt::sigil::SigilLight;
pub use hunt::sigil::SigilShape;
pub use hunt::sigil::SigilTrace;
pub use nft_catalogue::LAMPORTS_PER_SOL;
pub use nft_catalogue::Nft;
pub use nft_catalogue::NftCollection;
pub use nft_catalogue::Rarity;
pub use nft_catalogue::market_listings;
pub use nft_look::NftLook;
pub use player_state::Absorption;
pub use player_state::HuntSettlement;
pub use player_state::Judging;
pub use player_state::PlayerState;
pub use player_state::Refusal;
pub use player_state::RuleBreach;
pub use player_state::RuleResult;
pub use player_state::Sighting;
pub use progression::Progression;
pub use reaper_class::ClassPerk;
pub use reaper_class::ReaperClass;
pub use reaper_class::ReaperStats;
pub use reaper_class::WeaponKind;
pub use reaper_genome::Aura;
pub use reaper_genome::GenomeChanges;
pub use reaper_genome::Omen;
pub use reaper_genome::ReaperGenome;
pub use reaper_motion::MotionGroup;
pub use reaper_motion::ReaperMotion;
pub use reaper_names::REAPER_GIVEN_NAMES;
pub use reaper_names::reaper_epithets;
pub use reaper_soul::HuntDeeds;
pub use reaper_soul::ReaperGift;
pub use reaper_soul::ReaperSoul;
pub use reaper_soul::SoulMemory;
pub use reaper_soul::SoulPath;
pub use reaper_soul::SoulRules;
pub use reaper_soul::Temperament;
pub use reaper_state::ReaperState;
pub use rift::rift_frame::RiftFrame;
pub use rift::rift_frame::RiftMember;
pub use rift::rift_frame::RiftPhase;
pub use rift::rift_frame::RiftReport;
pub use rift::rift_room::RiftEntrant;
pub use rift::rift_room::RiftError;
pub use rift::rift_room::RiftReward;
pub use rift::rift_room::RiftRoom;
pub use rift::rift_room::RiftSeat;
pub use rift::rift_room::RiftSighting;
pub use rift::rift_rules::RiftRules;
pub use rift::rift_site::RiftSite;
pub use seeded_random::SeededRandom;
pub use seeded_random::hash_text;
pub use seeded_random::mix_seed;
pub use time::Timestamp;
pub use vec3::Vec3;
pub use vec3::degrees;
pub use vec3::radians;
pub use vec3::smoothstep;
pub use vec3::wrap_angle;
pub use vessel::Gait;
pub use vessel::SealSpec;
pub use vessel::Vessel;
pub use vessel::VesselSpec;
