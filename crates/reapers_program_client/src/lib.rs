//! The Reapers program's client: what the server, the app and `reapers_ops`
//! build its instructions with and read its accounts and events with.
//!
//! Everything under [`generated`] is written by `pina generate` from the
//! program's IDL (`crates/reapers_program/idl/reapers_program.json`) into
//! `reapers_program/src/generated`, and must not be edited. Run `pina generate`
//! in `crates/reapers_program` after changing the program. This file and
//! [`absorb`] are written by hand.
//!
//! Generated decoders check discriminators, not owners: compare an account's
//! owner with [`REAPERS_PROGRAM_ID`] before trusting what it decodes to.

#[path = "../reapers_program/src/generated/mod.rs"]
#[allow(
	clippy::all,
	clippy::pedantic,
	reason = "the generated sources follow pina's renderer, not this workspace's style"
)]
pub mod generated;

pub mod absorb;

pub use generated::*;
pub use generated::programs::REAPERS_PROGRAM_ID;
