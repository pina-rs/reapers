//! The Reapers keeper: player books, hunt replays, rifts, wallets and NFT
//! reapers, served on leptoskit over PostgreSQL.
//!
//! [`server::start`] builds everything the keeper serves; the binary in
//! `main.rs` configures it from the environment and serves it. Every rule
//! it applies is `reapers_rules`'; the keeper decides only what the rules
//! are handed: who is asking, the time, a seed, and what the chain says.

pub mod absorb;
pub mod books;
pub mod chain;
pub mod config;
pub mod escrow;
pub mod keeper;
pub mod receipt;
pub mod rift;
pub mod server;
pub mod sign_in;
pub mod signer;
pub mod wallet;

pub use reapers_db as tables;
