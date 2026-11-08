# Wallets and reaper NFTs

Reapers uses Solana Kit: the Dart SDK in the Flutter game and Serverpod keeper, and the Rust one in `crates/` (pinned to one revision in the root `Cargo.toml`). The game uses Kit's addresses, transactions and Mobile Wallet Adapter; the server uses typed RPC, Ed25519 proof verification and Kit's Metaplex Core decoder. No wallet secret enters the game or the server: the player's wallet signs everything that moves a husk, and the server signs only receipts.

## Deployment settings

These are public server environment values, not secrets:

| Variable | Meaning |
| --- | --- |
| `REAPERS_SOLANA_CHAIN` | `solana:devnet` by default, or `solana:mainnet`. A development keeper may read `solana:localnet`, a validator on its own machine. |
| `REAPERS_SOLANA_RPC_URL` | HTTPS RPC; defaults to the corresponding public cluster. Genesis must match the chain. A localnet's is plain HTTP on loopback, with no genesis check. |
| `REAPERS_WALLET_IDENTITY_URI` | Your HTTPS app identity with a trailing slash. Required with a collection. The placeholder `https://reapers.invalid/` disables wallet connection. |
| `REAPERS_REAPER_COLLECTION` | Verified Metaplex Core collection address. Without it, NFT import is unavailable. |
| `REAPERS_REAPER_MARKET_URL` | HTTPS page where users can buy or trade this collection. Omit to disable the marketplace action. |
| `REAPERS_PROGRAM_ID` | The Reapers program husks are escrowed with and burned by. Read by the Rust keeper only. Without it, a release that names an escrowed mint is refused. |
| `REAPERS_DEMO_HUSKS` | The path of the demo husk manifest `reapers_ops mint-demo` writes. The game lists the wallet's husks from it, and a release must escrow the very asset its husk stands for. Rust keeper only. |

Each imported asset must be a non-executable account owned by the Core program, decode as AssetV1, belong to that collection and be owned by the proved wallet. Its URI must be exactly the identity URI resolved against `reapers/<genome code>.json`; the genome parser validates every trait. The server does not fetch arbitrary metadata URLs. It freezes that genome on first import, and refuses later collection or genome changes. Publish matching metadata before minting; this code does not mint a collection.

For Android deployment, associate the chosen identity domain with the app's package and signing certificate through `/.well-known/assetlinks.json`, and configure its matching App Links in the manifest. These depend on the final domain and release certificate. Wallet identity verification follows the [Mobile Wallet Adapter specification](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html#native-dapp); supplying a URI alone does not establish that association.

The generated migration adds wallet identities, challenges and canonical reaper records. Apply pending migrations through the running Serverpod service before enabling imports. It does not modify the existing player table or discard saves.

## Player flow

Use an online game build and open **Your reapers**. Connect a wallet: Mobile Wallet Adapter on Android, or, in the Rust game's web build, a browser wallet through the Wallet Standard. The wallet signs a domain-, chain- and guest-bound two-minute challenge authorizing sign-in only. The server verifies the exact Ed25519 proof, consumes it once and issues its normal auth tokens. Wrong signatures, another guest's challenge, expiry and replay are rejected. The first proof links the guest account; proving a known wallet restores its account without combining two saves.

The displayed SOL balance comes from that wallet's real finalized RPC balance. The existing husk market's three starting SOL are simulated and remain separate.

Buy or trade through the configured marketplace, using the wallet's own transaction confirmation. Return and import the Core asset address. Import verifies current ownership and starts a newly seen reaper at level one; it never accepts a submitted genome or experience count. Up to 32 NFTs plus the oathbound starter can be held in the roster. Select an owned reaper after finishing its loose hunts.

Before every NFT-bearing game mutation, the server refreshes ownership and locks canonical growth records inside the player transaction. A sold reaper disappears on refresh while retaining its progression for its buyer. Timeouts and malformed chain responses fail the request instead of erasing the roster. Ownership checks are finalized snapshots, so they are not atomic with an on-chain transfer between reads. No offline save can contribute NFT growth.

## Current boundaries

- Wallet connection is implemented on Android and, in the Rust game, in the browser. iOS shows an explicit unavailable state.
- No collection, mint, marketplace listing or purchase transaction is fabricated. No live wallet transaction has been exercised without deployment settings and a test collection.
- Growth follows the NFT through server records, not metadata updates or a new on-chain progression program. Birth traits remain fixed.
- Solo hunt rewards are replayed. NFT reapers cannot enter rifts until rift replay is built, preventing trusted client reports from earning tradable growth.
- The five hunted husk collections' prices stay simulated. A cluster build of the Rust game escrows and burns the real demo husks with the program ([program.md](program.md), and "Devnet" below); everywhere else, releases and burns stay simulated. Real token acquisition is outstanding; [nfts.md](nfts.md) describes it.
- Tests use real SDK codecs, RPC transports, Ed25519 signatures and Serverpod's embedded database, with controlled chain responses. They verify proof failures, account restoration, ownership transfer and growth persistence; they do not prove a live marketplace trade.

## Devnet

The Rust game (`crates/reapers_app`) built with `REAPERS_SERVER_URL` and `REAPERS_CLUSTER=devnet` plays the demo husks for real. The keeper must run the simulated economy (development or test), read devnet with the program configured (`REAPERS_PROGRAM_ID`), have the demo manifest (`REAPERS_DEMO_HUSKS`), sign receipts with the key the program trusts, and name the web game's origin (`REAPERS_ORIGIN`).

| Step in the game | What happens | Who signs |
| --- | --- | --- |
| Connecting a wallet | The server's challenge, signed; the session moves to the wallet's account | the wallet (a message) |
| Opening the vault | `getMultipleAccounts` over the manifest's assets and their escrows: which the wallet holds, has in escrow, or has burned | nobody |
| Releasing a husk the wallet holds | Its stand-in is bought into the books if it is not there; `ReleaseCore`, sent and followed until finalized; then the server's `release` with the asset as the escrow, which the server checks on-chain at a finalized slot | the wallet |
| Absorbing it | `settle_hunt` replays the hunt and returns a `ChainReceipt` with an unsigned transaction: the Ed25519 check of the keeper's v2 receipt, then `AbsorbCore`. The game has the wallet sign it, sends it, and checks the asset is Core's one-byte tombstone | the wallet (the keeper signed the receipt) |

The slip shows each transaction as it lands, and the transaction sheet links to Solana Explorer. A program error comes back as `ReapersProgramError` and is put in the player's words; a wallet that declines sends nothing. A burn that fails after the hunt was settled leaves the husk in escrow and says so: the receipt is in the codex, but the game does not yet keep the transaction to try again.

`REAPERS_CLUSTER=localnet` plays the same flow on a validator on this machine, which `reapers_ops localnet` brings up with the program and the demo husks; [deploy.md](deploy.md) has the commands, and `crates/reapers_app/tests/devnet.rs` runs the flow end to end against it, and against devnet after a deployment.

## The simnet

A build with `--dart-define=REAPERS_SIMNET=true` plays the offline game on a pretend devnet that lives on the phone, so the chain's part can be shown before any program is deployed. It needs no server and no network; it cannot be combined with `REAPERS_SERVER_URL`, and its keeper refuses to open if both are set.

| Step in the game | Simnet transaction | Accounts |
| --- | --- | --- |
| Binding a reaper | **Airdrop** of the starting 3 SOL | faucet, wallet |
| Buying a husk | **Buy** at the rarity's price | wallet, market, mint |
| Releasing it | **Escrow**: the program's `ReleaseCore` | wallet, mint, escrow PDA, program |
| Absorbing it | **Burn**: the Ed25519 check of the receipt, then the program's `AbsorbCore` | wallet, mint, escrow PDA, program, receipt key |

A husk that gets away posts nothing: it stays in escrow, still huntable. Weapons, outfits, arts and rifts stay off-chain, as they are meant to.

What is real and what is not:

- **The wallet** is a real Ed25519 key pair, generated on the phone and kept in plain preferences. Its address is a valid Solana address. It is a burner: it holds nothing and must never be used anywhere else.
- **Each transaction** but the airdrop is a real Solana wire transaction signed by that wallet. In the Rust game the release and the burn are the program client's own `ReleaseCore` and `AbsorbCore`, exactly what devnet takes; only the husk is a stand-in, the SHA-256 of the NFT's invented mint, which no cluster holds. Its blockhash is chained from the previous transaction, so no cluster would accept it as it stands.
- **The escrow** is the real program's address for that stand-in, `["escrow", sha256(mint)]` under the Reapers program. The market and faucet addresses are made from names; nobody holds their keys.
- **The burn's receipt** is the same 98-byte version 2 receipt (`reapers_receipt`) the server signs, with the burner wallet as the player. It is signed by a simnet keeper key derived from a public name, so it proves only that the layout and the check work. The ledger runs the Ed25519 check a program would make before burning, and a receipt changed after signing fails it.
- **Slots** count 400 ms from when the wallet was made. Confirmation takes 650 ms; the slip shows the transaction finalized 1.4 s later.
- **The books** are the offline game's rules, kept under a save of their own (`reapers.simnet.player.v2`). The chain is told what happened after the rules allow it; it decides nothing.

### A real local validator beside the simnet

A simnet build may also name a real Solana JSON-RPC endpoint, usually a local [Surfpool](https://docs.surfpool.run/rpc/overview) validator:

```sh
surfpool start --offline --no-deploy
cd reapers_flutter
fvm flutter run --dart-define=REAPERS_SIMNET=true \
  --dart-define=REAPERS_SURFPOOL_URL=http://localhost:8899
```

`REAPERS_SURFPOOL_URL` is optional. Without it the simnet is exactly as before: no network. With it, **Simnet ledger** shows the endpoint, `getHealth`, `getGenesisHash` and the burner wallet's real `getBalance`, and offers `requestAirdrop` for one SOL. Those are standard Solana RPC calls. The returned signature is saved per burner wallet and genesis before checking confirmation. After a confirmation timeout or app restart, **Check** resumes that signature instead of requesting another airdrop. A confirmed payment disables the action. If the faucet response itself is lost, or saving its signature fails, a retry can still request another payment. A different genesis permits a new airdrop. The local network SOL is not the game's simulated SOL.

What this does not do:

- It does **not** broadcast `SimnetChain`'s signed descriptions. They are receipts in a rehearsal format, not Solana wire transactions.
- It does **not** buy, escrow or burn an NFT. **Send** puts a simnet release or burn on the validator, where it fails: its husk is a stand-in no cluster holds. A cluster build (`REAPERS_CLUSTER=localnet`) is the way to play real husks on a local validator.
- It does **not** coordinate players or rifts. Surfpool is a validator. Shared enemies are the server's `RiftHall`, as described below and in [gameplay.md](gameplay.md#rifts).

Public hosts need HTTPS. Cleartext HTTP is allowed only for loopback, an emulator's host alias (`10.0.2.2`) and private LAN addresses, which is where a developer's Surfpool runs.

## Demo circles

A development build can waive the collection attunement so several real people join one demo rift:

```sh
fvm flutter run --dart-define=REAPERS_SERVER_URL=http://localhost:8080/ \
  --dart-define=REAPERS_DEMO_RIFT=true
```

Every client that joins the same gathering on the same development or test server shares one server-hosted `RiftRoom`: one seed, one ward, one roster, one timetable of shrieks. Open the rift on each client before pressing ready, so all are present when the count begins. Each seat reports its own fight, and the server pays each player's own books exactly once when the rift is sealed. Empty seats are the ordinary shades. Demo circles are kept apart from ordinary ones, so the flag cannot mix strangers into a real circle.

The flag is ignored by release builds, and `RiftEndpoint.enterDemo` goes through the same development/test gate as the rest of the simulated economy: staging and production refuse it. The rewards are simulated experience and anima, not a chain payout. Nothing in the demo needs a wallet or sends a transaction.

The demo and the simnet are separate launches: `REAPERS_SIMNET` plays offline and refuses `REAPERS_SERVER_URL`, while `REAPERS_DEMO_RIFT` needs a server and refuses the simnet. A build sets one, never both. The simnet's Surfpool validator, where one is configured, is not part of a demo rift.

The simnet lives in `reapers_flutter/lib/simnet/` and, in the Rust game, `crates/reapers_app/src/simnet/`: `SimnetChain` is the wallet and the ledger, `SimnetBackend` posts to it around a `LocalBackend`, `SurfpoolRpc` speaks to an optional local validator, and the slip and the explorer show it. A cluster build replaces it with the real thing ("Devnet" above): the same `ReleaseCore` and burn, signed by the player's wallet, with the receipt signed by the server, at `["escrow", mint]` with the asset's own 32 bytes. **Airdrop** and **Buy** have no program instruction; buying happens in a marketplace.

