//! How this build was made: the `REAPERS_*` variables in the environment of
//! `leptoskit build` or `leptoskit dev`, read when the app is compiled.
//!
//! They replace the Flutter game's `--dart-define`s one for one:
//!
//! | Variable | Effect |
//! | --- | --- |
//! | `REAPERS_SERVER_URL=http://localhost:8080/` | Play against the server. Without it the game is fully offline. |
//! | `REAPERS_CLUSTER=devnet` | With a server, real demo husks on devnet (or `localnet`, a validator on this machine), moved by the player's wallet. |
//! | `REAPERS_SOLANA_RPC_URL=https://…` | With a cluster, the RPC the game sends through. Defaults to the cluster's public one. |
//! | `REAPERS_SIMNET=true` | The offline game on a pretend devnet: a burner wallet and a signed transaction for every airdrop, purchase, escrow and burn. |
//! | `REAPERS_SURFPOOL_URL=http://localhost:8899` | With the simnet, a real local Solana RPC in the explorer. |
//! | `REAPERS_DEMO_RIFT=true` | With a server, waives attunement so several clients share a demo rift. Ignored by release builds. |
//! | `REAPERS_AUTOPLAY=true` | A simulated player fights every hunt and rift. |
//! | `REAPERS_GALLERY=true` | Debug builds open at the reaper bench. |
//! | `REAPERS_DEPTH_PROBE=true` | Debug builds open at the depth model's bench. |
//!
//! ```sh
//! REAPERS_SIMNET=true leptoskit dev --platform web
//! REAPERS_SERVER_URL=http://10.0.2.2:8080/ leptoskit build --platform android
//! REAPERS_SERVER_URL=https://keeper.example/ REAPERS_CLUSTER=devnet leptoskit build --platform web
//! ```
//!
//! A combination that cannot work (the simnet with a server, a demo rift
//! without one, a Surfpool without the simnet, a cluster without a server)
//! is refused when the keeper is chosen, with the reason, instead of
//! starting a game that half works.

use crate::chain::ClusterName;

/// Every build flag, as one value, so tests can make any build they like.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools, reason = "each is one build flag")]
pub struct BuildFlags {
	/// The Reapers server, when the build plays against one.
	pub server_url: Option<&'static str>,
	/// The cluster real husks live on, by name: `devnet` or `localnet`.
	pub cluster: Option<&'static str>,
	/// The cluster's RPC, when not its public one.
	pub solana_rpc_url: Option<&'static str>,
	/// Whether the build rehearses the chain on the device.
	pub simnet: bool,
	/// A real Solana JSON-RPC for the simnet's explorer.
	pub surfpool_url: Option<&'static str>,
	/// Whether the build offers the development demo rift.
	pub demo_rift: bool,
	/// Whether a simulated player fights every hunt and rift.
	pub autoplay: bool,
	/// Whether the build opens at the reaper bench.
	pub gallery: bool,
	/// Whether the build opens at the depth model's bench.
	pub depth_probe: bool,
	/// Whether this is a release build, which ignores the developers' flags.
	pub release: bool,
}

/// Which keeper holds the player's books in this build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeeperChoice {
	/// The device itself, with no server.
	Local,
	/// The device, posting each step the chain would take to the simnet.
	Simnet,
	/// The Reapers server at this URL, with the cluster real husks live on
	/// and its RPC, when the build plays on one.
	Server(&'static str, Option<(ClusterName, &'static str)>),
}

/// A build whose flags cannot make a working game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
	#[error("REAPERS_DEMO_RIFT needs a server: leave REAPERS_SIMNET out.")]
	DemoWithSimnet,
	#[error("REAPERS_DEMO_RIFT needs REAPERS_SERVER_URL pointing at a development server.")]
	DemoWithoutServer,
	#[error("REAPERS_SURFPOOL_URL belongs to the simnet: add REAPERS_SIMNET=true.")]
	SurfpoolWithoutSimnet,
	#[error("REAPERS_SIMNET plays offline: build it without REAPERS_SERVER_URL.")]
	SimnetWithServer,
	#[error(
		"REAPERS_CLUSTER needs REAPERS_SERVER_URL: the server keeps the books and signs the burns."
	)]
	ClusterWithoutServer,
	#[error("REAPERS_CLUSTER is `devnet` or `localnet`.")]
	UnknownCluster,
	#[error("REAPERS_SOLANA_RPC_URL belongs to a cluster: add REAPERS_CLUSTER.")]
	RpcWithoutCluster,
}

/// Reads a yes from a flag's text.
const fn yes(value: Option<&str>) -> bool {
	match value {
		Some(text) => matches!(text.as_bytes(), b"true" | b"1" | b"yes" | b"TRUE" | b"True"),
		None => false,
	}
}

/// An unset or empty text flag is no flag.
const fn text(value: Option<&'static str>) -> Option<&'static str> {
	match value {
		Some(text) if !text.is_empty() => Some(text),
		_ => None,
	}
}

impl BuildFlags {
	/// The flags this build was compiled with.
	pub const fn current() -> Self {
		let release = !cfg!(debug_assertions);
		Self {
			server_url: text(option_env!("REAPERS_SERVER_URL")),
			cluster: text(option_env!("REAPERS_CLUSTER")),
			solana_rpc_url: text(option_env!("REAPERS_SOLANA_RPC_URL")),
			simnet: yes(option_env!("REAPERS_SIMNET")),
			surfpool_url: text(option_env!("REAPERS_SURFPOOL_URL")),
			demo_rift: !release && yes(option_env!("REAPERS_DEMO_RIFT")),
			autoplay: yes(option_env!("REAPERS_AUTOPLAY")),
			gallery: !release && yes(option_env!("REAPERS_GALLERY")),
			depth_probe: !release && yes(option_env!("REAPERS_DEPTH_PROBE")),
			release,
		}
	}

	/// Who keeps the books in a build with these flags, or why no one can.
	pub fn keeper(&self) -> Result<KeeperChoice, ConfigError> {
		if self.demo_rift {
			if self.simnet {
				return Err(ConfigError::DemoWithSimnet);
			}
			if self.server_url.is_none() {
				return Err(ConfigError::DemoWithoutServer);
			}
		}
		if self.surfpool_url.is_some() && !self.simnet {
			return Err(ConfigError::SurfpoolWithoutSimnet);
		}
		if self.simnet {
			return match self.server_url {
				Some(_) => Err(ConfigError::SimnetWithServer),
				None => Ok(KeeperChoice::Simnet),
			};
		}
		let cluster = match self.cluster {
			Some(name) => {
				let name = ClusterName::from_name(name).ok_or(ConfigError::UnknownCluster)?;
				Some((name, self.solana_rpc_url.unwrap_or(name.default_rpc())))
			}
			None if self.solana_rpc_url.is_some() => return Err(ConfigError::RpcWithoutCluster),
			None => None,
		};
		match (self.server_url, cluster) {
			(Some(url), cluster) => Ok(KeeperChoice::Server(url, cluster)),
			(None, Some(_)) => Err(ConfigError::ClusterWithoutServer),
			(None, None) => Ok(KeeperChoice::Local),
		}
	}

	/// Whether hunts may be made easier from their threshold: every build but
	/// a release, so whoever tries the game can see a whole hunt.
	pub const fn testing_difficulty(&self) -> bool {
		!self.release
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn no_flags_is_the_offline_game() {
		assert_eq!(BuildFlags::default().keeper(), Ok(KeeperChoice::Local));
	}

	#[test]
	fn a_server_url_plays_against_the_server() {
		let flags = BuildFlags {
			server_url: Some("http://localhost:8080/"),
			..BuildFlags::default()
		};
		assert_eq!(
			flags.keeper(),
			Ok(KeeperChoice::Server("http://localhost:8080/", None))
		);
	}

	#[test]
	fn a_cluster_plays_against_the_server_on_its_rpc() {
		let flags = BuildFlags {
			server_url: Some("http://localhost:8080/"),
			cluster: Some("devnet"),
			..BuildFlags::default()
		};
		assert_eq!(
			flags.keeper(),
			Ok(KeeperChoice::Server(
				"http://localhost:8080/",
				Some((ClusterName::Devnet, "https://api.devnet.solana.com"))
			))
		);
		assert_eq!(
			BuildFlags {
				cluster: Some("localnet"),
				solana_rpc_url: Some("http://127.0.0.1:18899"),
				..flags
			}
			.keeper(),
			Ok(KeeperChoice::Server(
				"http://localhost:8080/",
				Some((ClusterName::Localnet, "http://127.0.0.1:18899"))
			))
		);
		assert_eq!(
			BuildFlags {
				cluster: Some("mainnet"),
				..flags
			}
			.keeper(),
			Err(ConfigError::UnknownCluster)
		);
		assert_eq!(
			BuildFlags {
				server_url: None,
				..flags
			}
			.keeper(),
			Err(ConfigError::ClusterWithoutServer)
		);
		assert_eq!(
			BuildFlags {
				solana_rpc_url: Some("http://127.0.0.1:8899"),
				..BuildFlags::default()
			}
			.keeper(),
			Err(ConfigError::RpcWithoutCluster)
		);
	}

	#[test]
	fn the_simnet_refuses_a_server() {
		let flags = BuildFlags {
			simnet: true,
			server_url: Some("http://localhost:8080/"),
			..BuildFlags::default()
		};
		assert_eq!(flags.keeper(), Err(ConfigError::SimnetWithServer));
	}

	#[test]
	fn the_demo_needs_a_server_and_no_simnet() {
		let demo = BuildFlags {
			demo_rift: true,
			..BuildFlags::default()
		};
		assert_eq!(demo.keeper(), Err(ConfigError::DemoWithoutServer));
		assert_eq!(
			BuildFlags {
				simnet: true,
				..demo
			}
			.keeper(),
			Err(ConfigError::DemoWithSimnet)
		);
		assert_eq!(
			BuildFlags {
				server_url: Some("http://localhost:8080/"),
				..demo
			}
			.keeper(),
			Ok(KeeperChoice::Server("http://localhost:8080/", None))
		);
	}

	#[test]
	fn surfpool_belongs_to_the_simnet() {
		let flags = BuildFlags {
			surfpool_url: Some("http://localhost:8899"),
			..BuildFlags::default()
		};
		assert_eq!(flags.keeper(), Err(ConfigError::SurfpoolWithoutSimnet));
		assert_eq!(
			BuildFlags {
				simnet: true,
				..flags
			}
			.keeper(),
			Ok(KeeperChoice::Simnet)
		);
	}

	#[test]
	fn flags_read_yes_in_the_usual_spellings() {
		assert!(yes(Some("true")));
		assert!(yes(Some("1")));
		assert!(!yes(Some("false")));
		assert!(!yes(None));
		assert_eq!(text(Some("")), None);
	}
}
