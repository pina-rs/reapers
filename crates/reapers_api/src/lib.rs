//! The contract between the Reapers game and its keeper.
//!
//! Every request a player can make of the keeper is here, once, as a
//! leptoskit operation: a `#[rpc::query]`, `#[rpc::mutation]`,
//! `#[rpc::action]` or `#[rpc::stream]` whose argument struct exists on every
//! target. The game, built for `wasm32-unknown-unknown`, builds those structs
//! and sends them with `leptoskit_rpc::client::call`; the server build
//! (`server`) keeps the bodies, which hand their work to the [`Keeper`] the
//! server installed (see [`keeper`]). No rule lives here: the bodies only
//! pass the request on, and the keeper applies `reapers_rules`.
//!
//! | Module     | What it carries                                                       |
//! | ---------- | --------------------------------------------------------------------- |
//! | [`player`] | A player's own books: the oath, the vault, hunts, the armory          |
//! | [`chain`]  | Wallet proof, the chain's status and NFT reaper imports               |
//! | [`rift`]   | A rift as a stream of frames, and the reports a reaper sends into it  |
//! | [`auth`]   | Signing in as a guest and with a wallet: plain routes, not operations |
//!
//! Every operation finds the player from the request's session, so no
//! argument names a player. A request the rules refuse fails with the
//! [`Refusal`] they raised, written by [`refused`] and read by
//! [`refusal_of`], so an online player is told exactly what an offline one
//! would be.
//!
//! [`Keeper`]: keeper::Keeper

pub mod auth;
pub mod chain;
#[cfg(feature = "server")]
pub mod keeper;
pub mod player;
pub mod rift;

pub use leptoskit_rpc as rpc;
pub use reapers_rules::Refusal;

/// A value of the shared rules as an operation's argument: it crosses the
/// wire exactly as the rules serialize it (the save format), and the spec
/// describes it only as a JSON object, since the rules carry no schema.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Shared<T>(pub T);

impl<T> schemars::JsonSchema for Shared<T> {
	fn schema_name() -> std::borrow::Cow<'static, str> {
		"Shared".into()
	}

	fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
		schemars::json_schema!({
			"type": "object",
			"description": "A value of the shared Reapers rules, as they serialize it."
		})
	}
}

/// What every refused request's message starts with, before the refusal's
/// name.
pub const REFUSED: &str = "refused:";

/// The error a request the rules refused fails with: a conflict, whose
/// message is [`REFUSED`] and the refusal's name, such as
/// `refused:notOwned`.
pub fn refused(refusal: Refusal) -> rpc::Error {
	rpc::Error::conflict(format!("{REFUSED}{}", refusal.name()))
}

/// The refusal an error's message names, if it is one [`refused`] wrote.
///
/// Takes the message rather than the error, so a client can read it from
/// whatever its transport hands back: an `rpc::Error`, an `FnError`, or the
/// body of a failed sign-in route.
pub fn refusal_of(message: &str) -> Option<Refusal> {
	Refusal::from_name(message.strip_prefix(REFUSED)?)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn refusals_cross_the_wire_by_name() {
		for refusal in Refusal::ALL {
			let error = refused(refusal);

			assert_eq!(error.status(), 409);
			assert_eq!(refusal_of(error.message()), Some(refusal));
		}

		assert_eq!(refusal_of("notOwned"), None);
		assert_eq!(refusal_of("refused:noSuchThing"), None);
	}
}
