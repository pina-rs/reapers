//! How the keeper is configured: its run mode, its public chain settings,
//! and its secrets.
//!
//! Public settings come from the environment. Secrets come from the
//! environment too, as `REAPERS_SECRET_<name>`, and in development and test
//! also from the git-ignored `config/passwords.toml`, which
//! `config/passwords.example.toml` shows the shape of. A deployed keeper
//! reads only the environment, and refuses to start without every secret in
//! [`REQUIRED_SECRETS`].

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use reapers_rules::Refusal;
use solana_kit_core::Address;
use url::Url;

use crate::signer::ReceiptSigner;

/// Which kind of keeper this is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunMode {
	/// A developer's machine: the simulated economy, local secrets.
	Development,
	/// The test suite: the same as development.
	Test,
	/// A deployment that resembles production.
	Staging,
	/// The deployment players use.
	Production,
}

impl RunMode {
	/// The mode `REAPERS_RUN_MODE` names: `development` (the default),
	/// `test`, `staging` or `production`.
	///
	/// # Errors
	///
	/// When it names something else.
	pub fn from_env() -> Result<Self, ConfigError> {
		match std::env::var("REAPERS_RUN_MODE").as_deref() {
			Err(_) | Ok("development") => Ok(Self::Development),
			Ok("test") => Ok(Self::Test),
			Ok("staging") => Ok(Self::Staging),
			Ok("production") => Ok(Self::Production),
			Ok(other) => {
				Err(ConfigError(format!(
					"REAPERS_RUN_MODE: unknown mode `{other}`"
				)))
			}
		}
	}

	/// The mode's name, as `REAPERS_RUN_MODE` and the passwords file write it.
	pub const fn name(self) -> &'static str {
		match self {
			Self::Development => "development",
			Self::Test => "test",
			Self::Staging => "staging",
			Self::Production => "production",
		}
	}

	/// Whether this keeper runs the simulated economy: invented mints, the
	/// stand-in market's balances, and rifts whose reports are taken at their
	/// word. Only development and test do.
	pub const fn simulates(self) -> bool {
		matches!(self, Self::Development | Self::Test)
	}

	/// Refuses, as `marketUnavailable`, anything that belongs only to the
	/// simulated economy on a deployed keeper. A deployed keeper cannot turn
	/// prototype balances and invented mints into tradable NFT progression,
	/// and there is deliberately no flag to let it.
	pub const fn require_simulation(self) -> Result<(), Refusal> {
		if self.simulates() {
			Ok(())
		} else {
			Err(Refusal::MarketUnavailable)
		}
	}
}

impl fmt::Display for RunMode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.name())
	}
}

/// A setting the keeper cannot run with.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct ConfigError(pub String);

/// The chains a keeper may read, with the genesis hash each must answer.
pub const GENESIS_HASHES: [(&str, &str); 2] = [
	(
		"solana:devnet",
		"EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG",
	),
	(
		"solana:mainnet",
		"5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d",
	),
];

/// The genesis hash `chain` must answer.
pub fn genesis_hash(chain: &str) -> Option<&'static str> {
	GENESIS_HASHES
		.iter()
		.find(|(name, _)| *name == chain)
		.map(|(_, hash)| *hash)
}

/// Public deployment settings for the chain. No signing key or wallet secret
/// lives here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolanaConfig {
	/// `solana:devnet` or `solana:mainnet`.
	pub chain: String,
	/// The HTTPS RPC the keeper reads. Its genesis must match `chain`.
	pub rpc_url: String,
	/// The keeper's HTTPS identity origin, with a trailing slash. Wallet
	/// messages are bound to it and reaper metadata lives under it.
	pub identity_uri: Url,
	/// The verified Metaplex Core collection NFT reapers come from.
	pub collection: Option<Address>,
	/// Where players buy and trade the collection.
	pub marketplace_uri: Option<Url>,
	/// The Reapers program husks are escrowed with and burned by.
	pub program: Option<Address>,
}

impl SolanaConfig {
	/// The identity origin that turns wallets off until a real one is set.
	pub const PLACEHOLDER_IDENTITY: &'static str = "https://reapers.invalid/";

	/// Checks the settings: a known chain, HTTPS everywhere, an identity
	/// origin that is a bare origin and path ending in `/`.
	///
	/// # Errors
	///
	/// When any of them is invalid.
	pub fn new(
		chain: impl Into<String>,
		rpc_url: impl Into<String>,
		identity_uri: &str,
		collection: Option<&str>,
		marketplace_uri: Option<&str>,
		program: Option<&str>,
	) -> Result<Self, ConfigError> {
		let invalid =
			|what: &str| ConfigError(format!("Invalid Solana deployment settings: {what}"));
		let chain = chain.into();
		let rpc_url = rpc_url.into();
		// Checked as written, since parsing gives a bare origin the path `/`.
		let bare_origin = !identity_uri.ends_with('/');
		let identity_uri = Url::parse(identity_uri).map_err(|_| invalid("identity URI"))?;
		let address = |value: Option<&str>, what: &str| {
			value
				.map(|value| value.parse::<Address>().map_err(|_| invalid(what)))
				.transpose()
		};
		let https = |value: Option<&str>, what: &str| {
			value
				.map(|value| {
					Url::parse(value)
						.ok()
						.filter(|url| url.scheme() == "https")
						.ok_or_else(|| invalid(what))
				})
				.transpose()
		};

		if genesis_hash(&chain).is_none() {
			return Err(invalid("chain"));
		}

		https(Some(&rpc_url), "RPC URL")?;

		if bare_origin
			|| identity_uri.scheme() != "https"
			|| identity_uri.host_str().is_none_or(str::is_empty)
			|| !identity_uri.username().is_empty()
			|| identity_uri.password().is_some()
			|| identity_uri.query().is_some()
			|| identity_uri.fragment().is_some()
			|| !identity_uri.path().ends_with('/')
		{
			return Err(invalid("identity URI"));
		}

		let program = address(program, "program")?;

		// The keeper builds the program's instructions and finds its accounts
		// with the client generated for one program id; any other id would
		// name accounts and instructions that program does not have.
		if program.is_some_and(|program| program != crate::escrow::program()) {
			return Err(invalid(
				"program: not the program this keeper was built for",
			));
		}

		Ok(Self {
			collection: address(collection, "collection")?,
			marketplace_uri: https(marketplace_uri, "marketplace URI")?,
			program,
			chain,
			rpc_url,
			identity_uri,
		})
	}

	/// The settings in `REAPERS_SOLANA_CHAIN`, `REAPERS_SOLANA_RPC_URL`,
	/// `REAPERS_WALLET_IDENTITY_URI`, `REAPERS_REAPER_COLLECTION`,
	/// `REAPERS_REAPER_MARKET_URL` and `REAPERS_PROGRAM_ID`.
	///
	/// # Errors
	///
	/// When a setting is invalid, or a collection is named without the
	/// identity origin its metadata lives under.
	pub fn from_env() -> Result<Self, ConfigError> {
		Self::from_vars(|name| std::env::var(name).ok())
	}

	/// The settings `var` gives, as [`Self::from_env`] reads them.
	///
	/// # Errors
	///
	/// As [`Self::from_env`].
	pub fn from_vars(var: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
		let chain = var("REAPERS_SOLANA_CHAIN").unwrap_or_else(|| "solana:devnet".to_owned());
		let collection = var("REAPERS_REAPER_COLLECTION");
		let identity = var("REAPERS_WALLET_IDENTITY_URI");

		if collection.is_some() && identity.is_none() {
			return Err(ConfigError(
				"A reaper collection requires its wallet identity origin".to_owned(),
			));
		}

		let rpc_url = var("REAPERS_SOLANA_RPC_URL").unwrap_or_else(|| {
			if chain == "solana:mainnet" {
				"https://api.mainnet-beta.solana.com".to_owned()
			} else {
				"https://api.devnet.solana.com".to_owned()
			}
		});

		Self::new(
			chain,
			rpc_url,
			identity.as_deref().unwrap_or(Self::PLACEHOLDER_IDENTITY),
			collection.as_deref(),
			var("REAPERS_REAPER_MARKET_URL").as_deref(),
			var("REAPERS_PROGRAM_ID").as_deref(),
		)
	}

	/// Whether wallets may be proved: false until a real identity origin is
	/// set.
	pub fn wallets_enabled(&self) -> bool {
		!self
			.identity_uri
			.host_str()
			.is_some_and(|host| host.ends_with(".invalid"))
	}

	/// The genesis hash the RPC must answer.
	pub fn genesis_hash(&self) -> &'static str {
		genesis_hash(&self.chain).unwrap_or_default()
	}
}

/// Every secret a deployed keeper cannot run without: the database's URL
/// (with its password), the keys sign-in seals and hashes with, and the
/// secret the receipt signing key is derived from.
pub const REQUIRED_SECRETS: [&str; 3] = ["database", "authKeys", ReceiptSigner::SECRET];

/// The keeper's secrets.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Secrets {
	values: BTreeMap<String, String>,
}

impl Secrets {
	/// The prefix of a secret's environment variable: `REAPERS_SECRET_database`.
	pub const ENV_PREFIX: &'static str = "REAPERS_SECRET_";

	/// The secrets for `mode`: `config/passwords.toml` under `root`, when
	/// the mode is development or test and the file is there, overridden by
	/// the environment.
	///
	/// # Errors
	///
	/// When the passwords file is there but is not valid TOML.
	pub fn load(mode: RunMode, root: &Path) -> Result<Self, ConfigError> {
		let mut secrets = Self::default();
		let file = root.join("config/passwords.toml");

		if mode.simulates() && file.exists() {
			let text = std::fs::read_to_string(&file)
				.map_err(|error| ConfigError(format!("{}: {error}", file.display())))?;

			secrets = Self::from_toml(&text, mode)?;
		}

		for (name, value) in std::env::vars() {
			if let Some(secret) = name.strip_prefix(Self::ENV_PREFIX) {
				secrets.values.insert(secret.to_owned(), value);
			}
		}

		Ok(secrets)
	}

	/// The secrets a passwords file gives `mode`: its `[shared]` table, then
	/// the mode's own table over it.
	///
	/// # Errors
	///
	/// When `text` is not TOML, or a value is not a string.
	pub fn from_toml(text: &str, mode: RunMode) -> Result<Self, ConfigError> {
		let table: toml::Table = text
			.parse()
			.map_err(|error| ConfigError(format!("config/passwords.toml: {error}")))?;
		let mut values = BTreeMap::new();

		for section in ["shared", mode.name()] {
			let Some(entries) = table.get(section).and_then(toml::Value::as_table) else {
				continue;
			};

			for (name, value) in entries {
				let value = value.as_str().ok_or_else(|| {
					ConfigError(format!(
						"config/passwords.toml: `{section}.{name}` must be a string"
					))
				})?;

				values.insert(name.clone(), value.to_owned());
			}
		}

		Ok(Self { values })
	}

	/// The secrets in `pairs`, for tests.
	pub fn of<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
		Self {
			values: pairs
				.into_iter()
				.map(|(name, value)| (name.to_owned(), value.to_owned()))
				.collect(),
		}
	}

	/// The secret called `name`, unless it is missing or blank.
	pub fn get(&self, name: &str) -> Option<&str> {
		self.values
			.get(name)
			.map(String::as_str)
			.filter(|value| !value.trim().is_empty())
	}

	/// The secrets a keeper in `mode` is missing. Development and test are
	/// exempt: nobody should need production credentials to run the game
	/// locally.
	pub fn missing(&self, mode: RunMode) -> Vec<&'static str> {
		if mode.simulates() {
			return Vec::new();
		}

		REQUIRED_SECRETS
			.into_iter()
			.filter(|name| self.get(name).is_none())
			.collect()
	}
}

impl fmt::Debug for Secrets {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_set().entries(self.values.keys()).finish()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn deployed_keepers_refuse_the_invented_nft_economy() {
		for mode in [RunMode::Staging, RunMode::Production] {
			assert_eq!(mode.require_simulation(), Err(Refusal::MarketUnavailable));
		}

		for mode in [RunMode::Development, RunMode::Test] {
			assert_eq!(mode.require_simulation(), Ok(()));
		}
	}

	#[test]
	fn deployed_keepers_need_every_secret() {
		let none = Secrets::default();
		let all = Secrets::of([
			("database", "postgres://keeper@db/reapers"),
			("authKeys", "2026-11:AAAA"),
			("receiptSigningKey", "a long random value"),
		]);
		let blank = Secrets::of([
			("database", " "),
			("authKeys", "x"),
			("receiptSigningKey", "y"),
		]);

		assert_eq!(none.missing(RunMode::Production), REQUIRED_SECRETS.to_vec());
		assert_eq!(none.missing(RunMode::Staging), REQUIRED_SECRETS.to_vec());
		assert_eq!(none.missing(RunMode::Development), [] as [&str; 0]);
		assert_eq!(none.missing(RunMode::Test), [] as [&str; 0]);
		assert_eq!(all.missing(RunMode::Production), [] as [&str; 0]);
		assert_eq!(blank.missing(RunMode::Production), vec!["database"]);
	}

	#[test]
	fn the_passwords_file_layers_a_mode_over_what_is_shared() {
		let text =
			"[shared]\ndatabase = \"shared\"\n\n[test]\ndatabase = \"test\"\nauthKeys = \"k\"\n";
		let test = Secrets::from_toml(text, RunMode::Test).expect("it parses");
		let development = Secrets::from_toml(text, RunMode::Development).expect("it parses");

		assert_eq!(test.get("database"), Some("test"));
		assert_eq!(test.get("authKeys"), Some("k"));
		assert_eq!(development.get("database"), Some("shared"));
		assert_eq!(development.get("authKeys"), None);
		assert!(Secrets::from_toml("[test]\ndatabase = 1\n", RunMode::Test).is_err());
	}

	#[test]
	fn solana_settings_must_be_https_and_a_bare_identity_origin() {
		let valid = |identity: &str| {
			SolanaConfig::new(
				"solana:devnet",
				"https://api.devnet.solana.com",
				identity,
				None,
				None,
				None,
			)
		};

		assert!(valid("https://reapers.example/").is_ok());

		for identity in [
			"http://reapers.example/",
			"https://reapers.example",
			"https://user@reapers.example/",
			"https://reapers.example/?q=1",
			"https://reapers.example/#top",
		] {
			assert!(valid(identity).is_err(), "{identity}");
		}

		assert!(
			SolanaConfig::new(
				"solana:testnet",
				"https://x.example/",
				"https://reapers.example/",
				None,
				None,
				None
			)
			.is_err()
		);
		assert!(
			SolanaConfig::new(
				"solana:devnet",
				"http://x.example/",
				"https://reapers.example/",
				None,
				None,
				None
			)
			.is_err()
		);
		assert!(
			SolanaConfig::new(
				"solana:devnet",
				"https://x.example/",
				"https://reapers.example/",
				Some("not an address"),
				None,
				None
			)
			.is_err()
		);
	}

	#[test]
	fn a_collection_needs_its_identity_origin() {
		let vars = |pairs: &'static [(&'static str, &'static str)]| {
			move |name: &str| {
				pairs
					.iter()
					.find(|(key, _)| *key == name)
					.map(|(_, value)| (*value).to_owned())
			}
		};

		assert!(
			SolanaConfig::from_vars(vars(&[(
				"REAPERS_REAPER_COLLECTION",
				"11111111111111111111111111111111"
			)]))
			.is_err()
		);

		let defaults = SolanaConfig::from_vars(vars(&[])).expect("the defaults are valid");

		assert_eq!(defaults.chain, "solana:devnet");
		assert_eq!(defaults.rpc_url, "https://api.devnet.solana.com");
		assert!(!defaults.wallets_enabled());

		let mainnet = SolanaConfig::from_vars(vars(&[("REAPERS_SOLANA_CHAIN", "solana:mainnet")]))
			.expect("mainnet is valid");

		assert_eq!(mainnet.rpc_url, "https://api.mainnet-beta.solana.com");
	}
}
