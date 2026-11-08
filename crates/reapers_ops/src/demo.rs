//! The devnet demo husks: one Metaplex Core collection for each of the five
//! supported collections in `docs/nfts.md`, with eight assets each, numbered
//! 1 to 8 like the game's invented NFTs.
//!
//! They are stand-ins on a test network. The names say "Reapers demo", and the
//! metadata URIs point at placeholder JSON under a base URL we can host later.

use solana_kit_core::AccountMeta;
use solana_kit_core::AccountRole;
use solana_kit_core::Address;
use solana_kit_core::Instruction;
use solana_kit_core::codecs::Limits;
use solana_kit_core::crypto::Keypair;
use solana_kit_mpl_core::CreateCollectionV2;
use solana_kit_mpl_core::CreateV2;
use solana_kit_mpl_core::DataState;

use crate::chain::Chain;
use crate::chain::Error;
use crate::chain::random_keypair;
use crate::program::units;

/// The five supported collections: their names, and the slug their metadata
/// lives under.
pub const COLLECTIONS: [(&str, &str); 5] = [
	("Degenerate Ape Academy", "degenerate-ape-academy"),
	("Thugbirdz", "thugbirdz"),
	("Pesky Penguins", "pesky-penguins"),
	("Sollamas", "sollamas"),
	("Solarians", "solarians"),
];

/// Assets in each collection.
pub const ASSETS_PER_COLLECTION: u8 = 8;

/// The Metaplex Core program.
pub const CORE: Address = Address::from_bytes([
	175, 84, 171, 16, 189, 151, 165, 66, 160, 158, 247, 179, 152, 137, 221, 12, 211, 148, 164, 204,
	233, 223, 166, 205, 201, 126, 190, 45, 35, 91, 167, 72,
]);

const SYSTEM: Address = Address::from_bytes([0; 32]);

fn limits() -> Limits {
	Limits::new(4096, 64)
}

/// A minted collection: its address, and its assets in order.
pub struct MintedCollection {
	pub name: &'static str,
	pub slug: &'static str,
	pub address: Address,
	pub assets: Vec<Address>,
}

/// Creates a Core collection, with `payer` as its update authority.
pub async fn create_collection(
	chain: &Chain,
	payer: &Keypair,
	name: &str,
	uri: &str,
) -> Result<Address, Error> {
	let (_, collection) = random_keypair()?;
	let instruction = CreateCollectionV2 {
		name: name.to_owned(),
		uri: uri.to_owned(),
		plugins: None,
		external_plugin_adapters: None,
	}
	.instruction(
		&[
			meta(collection.address(), AccountRole::WritableSigner),
			meta(CORE, AccountRole::Readonly),
			meta(payer.address(), AccountRole::WritableSigner),
			meta(SYSTEM, AccountRole::Readonly),
		],
		limits(),
	)?;

	chain
		.send(&[instruction], &[payer, &collection], units::CORE_MINT)
		.await?;

	Ok(collection.address())
}

/// Creates a Core asset in `collection`, owned by `owner`. `payer` is the
/// collection's update authority.
pub async fn create_asset(
	chain: &Chain,
	payer: &Keypair,
	collection: &Address,
	owner: &Address,
	name: &str,
	uri: &str,
) -> Result<Address, Error> {
	let (_, asset) = random_keypair()?;
	let instruction = create_asset_instruction(
		&asset.address(),
		collection,
		&payer.address(),
		owner,
		name,
		uri,
	)?;

	chain
		.send(&[instruction], &[payer, &asset], units::CORE_MINT)
		.await?;

	Ok(asset.address())
}

fn create_asset_instruction(
	asset: &Address,
	collection: &Address,
	payer: &Address,
	owner: &Address,
	name: &str,
	uri: &str,
) -> Result<Instruction, Error> {
	Ok(CreateV2 {
		data_state: DataState::AccountState,
		name: name.to_owned(),
		uri: uri.to_owned(),
		plugins: None,
		external_plugin_adapters: None,
	}
	.instruction(
		&[
			meta(*asset, AccountRole::WritableSigner),
			meta(*collection, AccountRole::Writable),
			// No separate authority: the payer is the collection's.
			meta(CORE, AccountRole::Readonly),
			meta(*payer, AccountRole::WritableSigner),
			meta(*owner, AccountRole::Readonly),
			// Assets in a collection take its update authority.
			meta(CORE, AccountRole::Readonly),
			meta(SYSTEM, AccountRole::Readonly),
			meta(CORE, AccountRole::Readonly),
		],
		limits(),
	)?)
}

/// Mints the five demo collections and their assets, all owned by `owner`.
pub async fn mint_all(
	chain: &Chain,
	payer: &Keypair,
	owner: &Address,
	uri_base: &str,
	mut progress: impl FnMut(&str),
) -> Result<Vec<MintedCollection>, Error> {
	let mut minted = Vec::new();

	for (name, slug) in COLLECTIONS {
		let collection_name = format!("{name} (Reapers demo)");
		let collection_uri = format!("{uri_base}{slug}/collection.json");
		let address = create_collection(chain, payer, &collection_name, &collection_uri).await?;
		progress(&format!("collection {collection_name}: {address}"));

		let mut assets = Vec::new();
		for number in 1..=ASSETS_PER_COLLECTION {
			let asset_name = format!("{name} husk #{number}");
			let asset_uri = format!("{uri_base}{slug}/{number}.json");
			let asset =
				create_asset(chain, payer, &address, owner, &asset_name, &asset_uri).await?;
			progress(&format!("  #{number} {asset}"));
			assets.push(asset);
		}

		minted.push(MintedCollection {
			name,
			slug,
			address,
			assets,
		});
	}

	Ok(minted)
}

/// The minted collections as JSON, for the server's catalogue and for people.
pub fn manifest(cluster: &str, minted: &[MintedCollection]) -> serde_json::Value {
	serde_json::json!({
		"cluster": cluster,
		"program": crate::program::program_id().to_string(),
		"collections": minted.iter().map(|collection| serde_json::json!({
			"name": collection.name,
			"slug": collection.slug,
			"address": collection.address.to_string(),
			"assets": collection.assets.iter().map(ToString::to_string).collect::<Vec<_>>(),
		})).collect::<Vec<_>>(),
	})
}

fn meta(address: Address, role: AccountRole) -> AccountMeta {
	AccountMeta { address, role }
}

/// The owner and first byte of a Core asset, as `(key, owner)`.
pub fn asset_owner(data: &[u8]) -> Option<(u8, Address)> {
	let key = *data.first()?;
	let owner: [u8; 32] = data.get(1..33)?.try_into().ok()?;

	Some((key, Address::from_bytes(owner)))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn names_core_and_the_system_program() {
		assert_eq!(
			CORE,
			"CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
				.parse::<Address>()
				.unwrap()
		);
		assert_eq!(
			SYSTEM,
			"11111111111111111111111111111111"
				.parse::<Address>()
				.unwrap()
		);
	}

	#[test]
	fn names_fit_core() {
		for (name, slug) in COLLECTIONS {
			let asset = format!("{name} husk #8");
			assert!(asset.len() <= 32, "{asset} is short enough for a wallet");
			assert!(!slug.contains(' '));
		}
	}

	#[test]
	fn builds_a_create_asset_instruction_core_accepts() {
		let instruction = create_asset_instruction(
			&Address::from_bytes([1; 32]),
			&Address::from_bytes([2; 32]),
			&Address::from_bytes([3; 32]),
			&Address::from_bytes([4; 32]),
			"Thugbirdz husk #1",
			"https://example.invalid/thugbirdz/1.json",
		)
		.unwrap();

		assert_eq!(instruction.program_address, CORE);
		assert_eq!(instruction.data[0], 20);
		assert_eq!(instruction.accounts.len(), 8);
	}
}
