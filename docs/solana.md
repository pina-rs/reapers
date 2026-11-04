# Wallets and reaper NFTs

Reapers uses the Dart Solana Kit SDK, the same SDK repository used by the author's other Flutter projects. Flutter uses Kit address utilities and Mobile Wallet Adapter; the server uses typed RPC, Ed25519 proof verification and Kit's Metaplex Core decoder. No wallet secret or signing authority enters the game or server.

## Deployment settings

These are public server environment values, not secrets:

| Variable | Meaning |
| --- | --- |
| `REAPERS_SOLANA_CHAIN` | `solana:devnet` by default, or `solana:mainnet`. |
| `REAPERS_SOLANA_RPC_URL` | HTTPS RPC; defaults to the corresponding public cluster. Genesis must match the chain. |
| `REAPERS_WALLET_IDENTITY_URI` | Your HTTPS app identity with a trailing slash. Required with a collection. The placeholder `https://reapers.invalid/` disables wallet connection. |
| `REAPERS_REAPER_COLLECTION` | Verified Metaplex Core collection address. Without it, NFT import is unavailable. |
| `REAPERS_REAPER_MARKET_URL` | HTTPS page where users can buy or trade this collection. Omit to disable the marketplace action. |

Each imported asset must be a non-executable account owned by the Core program, decode as AssetV1, belong to that collection and be owned by the proved wallet. Its URI must be exactly the identity URI resolved against `reapers/<genome code>.json`; the genome parser validates every trait. The server does not fetch arbitrary metadata URLs. It freezes that genome on first import, and refuses later collection or genome changes. Publish matching metadata before minting; this code does not mint a collection.

For Android deployment, associate the chosen identity domain with the app's package and signing certificate through `/.well-known/assetlinks.json`, and configure its matching App Links in the manifest. These depend on the final domain and release certificate. Wallet identity verification follows the [Mobile Wallet Adapter specification](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html#native-dapp); supplying a URI alone does not establish that association.

The generated migration adds wallet identities, challenges and canonical reaper records. Apply pending migrations through the running Serverpod service before enabling imports. It does not modify the existing player table or discard saves.

## Player flow

Use an online game build and open **Your reapers**. On Android, connect a wallet using Mobile Wallet Adapter. The wallet signs a domain-, chain- and guest-bound two-minute challenge authorizing sign-in only. The server verifies the exact Ed25519 proof, consumes it once and issues its normal auth tokens. Wrong signatures, another guest's challenge, expiry and replay are rejected. The first proof links the guest account; proving a known wallet restores its account without combining two saves.

The displayed SOL balance comes from that wallet's real finalized RPC balance. The existing husk market's three starting SOL are simulated and remain separate.

Buy or trade through the configured marketplace, using the wallet's own transaction confirmation. Return and import the Core asset address. Import verifies current ownership and starts a newly seen reaper at level one; it never accepts a submitted genome or experience count. Up to 32 NFTs plus the oathbound starter can be held in the roster. Select an owned reaper after finishing its loose hunts.

Before every NFT-bearing game mutation, the server refreshes ownership and locks canonical growth records inside the player transaction. A sold reaper disappears on refresh while retaining its progression for its buyer. Timeouts and malformed chain responses fail the request instead of erasing the roster. Ownership checks are finalized snapshots, so they are not atomic with an on-chain transfer between reads. No offline save can contribute NFT growth.

## Current boundaries

- Wallet connection is implemented on Android. iOS shows an explicit unavailable state. The web studio opens no wallet and remains fully usable as a preview.
- No collection, mint, marketplace listing or purchase transaction is fabricated. No live wallet transaction has been exercised without deployment settings and a test collection.
- Growth follows the NFT through server records, not metadata updates or a new on-chain progression program. Birth traits remain fixed.
- Solo hunt rewards are replayed. NFT reapers cannot enter rifts until rift replay is built, preventing trusted client reports from earning tradable growth.
- The five hunted husk collections, prices and burns remain simulated. The signed receipt is still not submitted on-chain. Escrow, real token acquisition and a verified burn program are outstanding; [nfts.md](nfts.md) describes them.
- Tests use real SDK codecs, RPC transports, Ed25519 signatures and Serverpod's embedded database, with controlled chain responses. They verify proof failures, account restoration, ownership transfer and growth persistence; they do not prove a live marketplace trade.

## The simnet

A build with `--dart-define=REAPERS_SIMNET=true` plays the offline game on a pretend devnet that lives on the phone, so the chain's part can be shown before any program is deployed. It needs no server and no network; it cannot be combined with `REAPERS_SERVER_URL`, and its keeper refuses to open if both are set.

| Step in the game | Simnet transaction | Accounts |
| --- | --- | --- |
| Binding a reaper | **Airdrop** of the starting 3 SOL | faucet, wallet |
| Buying a husk | **Buy** at the rarity's price | wallet, market, mint |
| Releasing it | **Escrow** into the program | wallet, mint, escrow PDA, program |
| Absorbing it | **Burn** out of escrow, carrying the receipt | wallet, mint, escrow PDA, program, receipt key |

A husk that gets away posts nothing: it stays in escrow, still huntable. Weapons, outfits, arts and rifts stay off-chain, as they are meant to.

What is real and what is not:

- **The wallet** is a real Ed25519 key pair, generated on the phone and kept in plain preferences. Its address is a valid Solana address. It is a burner: it holds nothing and must never be used anywhere else.
- **Each transaction's signature** is a real Ed25519 signature by that wallet, over a description of the instruction, its accounts, the amount and a blockhash chained from the previous transaction. It is not a Solana wire transaction and no cluster has seen it.
- **The escrow** is a real program-derived address, `["escrow", sha256(mint)]` under the simnet program. The program, market and faucet addresses are made from names; nobody holds their keys.
- **The burn's receipt** is the same 98-byte `AbsorptionReceipt` the server signs, with the burner wallet as the player. It is signed by a simnet keeper key derived from a public name, so it proves only that the layout and the check work. The ledger runs the Ed25519 check a program would make before burning, and a receipt changed after signing fails it.
- **Slots** count 400 ms from when the wallet was made. Confirmation takes 650 ms; the slip shows the transaction finalized 1.4 s later.
- **The books** are the offline game's rules, kept under a save of their own (`reapers.simnet.player.v2`). The chain is told what happened after the rules allow it; it decides nothing.

The simnet lives in `reapers_flutter/lib/simnet/`: `SimnetChain` is the wallet and the ledger, `SimnetBackend` posts to it around a `LocalBackend`, `SimnetToasts` is the slip and `SimnetScreen` the explorer. Replacing it with devnet means sending the same four instructions to a deployed program through Solana Kit, with the wallet behind Mobile Wallet Adapter and the receipt signed by the server.

