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

- Wallet connection is implemented on Android. iOS and desktop show an explicit unavailable state; their studio remains fully usable.
- No collection, mint, marketplace listing or purchase transaction is fabricated. No live wallet transaction has been exercised without deployment settings and a test collection.
- Growth follows the NFT through server records, not metadata updates or a new on-chain progression program. Birth traits remain fixed.
- Solo hunt rewards are replayed. NFT reapers cannot enter rifts until rift replay is built, preventing trusted client reports from earning tradable growth.
- The five hunted husk collections, prices and burns remain simulated. The signed receipt is still not submitted on-chain. Escrow, real token acquisition and a verified burn program are outstanding; [nfts.md](nfts.md) describes them.
- Tests use real SDK codecs, RPC transports, Ed25519 signatures and Serverpod's embedded database, with controlled chain responses. They verify proof failures, account restoration, ownership transfer and growth persistence; they do not prove a live marketplace trade.
