//! Rifts: husks too strong for one reaper, fought by a circle of them.
//!
//! A reaper opens [`enter_rift`] on the live connection and stays in it
//! until the rift ends: it is the rift as they see it, several times a
//! second. What they say of their own fight goes the other way, through
//! [`report_rift`], as it changes. Reports are whole standings, not changes,
//! so one that is lost, repeated or late never miscounts.
//!
//! Who the reaper is and what they fight with is read from their stored
//! books, never from the game. What they report of their own fight is, in
//! this version, taken at their word: a lone hunt is replayed before it
//! pays, a rift is not yet. That is why only a development or test keeper
//! hosts rifts, and why no NFT reaper may enter one.

#[cfg(feature = "server")]
use futures::Stream;
use leptoskit_rpc as rpc;
use reapers_rules::RiftFrame;
use reapers_rules::RiftReport;

use crate::Shared;
#[cfg(feature = "server")]
use crate::keeper::keeper;

/// Joins the rift gathering at `site` (a `RiftSite` name, such as
/// `londonBridge`) and follows it until it ends.
///
/// The first frame is the circle as the reaper found it. The last says how
/// the rift ended; if it was sealed, the reaper has been paid by the time
/// that frame arrives. A report the keeper could not use, such as a stale
/// standing, ends the stream with that refusal.
///
/// With `demo`, the reaper joins the development demo circle at `site`
/// instead, where attunement is waived so several people can meet one
/// enemy without first hunting a collection. Demo circles are kept apart
/// from ordinary ones.
#[rpc::stream]
pub async fn enter_rift(
	cx: &rpc::QueryCx,
	site: String,
	#[serde(default)] demo: bool,
) -> rpc::Result<impl Stream<Item = rpc::Result<RiftFrame>>> {
	keeper(cx.identity())?.enter_rift(cx, site, demo).await
}

/// How the reaper's own fight in their current rift stands.
///
/// Refused as `locationUnusable` when it carries a standing that is coarse,
/// stale or future-dated, which also ends their rift.
#[rpc::action]
pub async fn report_rift(cx: &rpc::ActionCx, report: Shared<RiftReport>) -> rpc::Result<()> {
	keeper(cx.identity())?.report_rift(cx, report.0).await
}
