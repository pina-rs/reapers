# Launch readiness — 8 November 2026

First written on 5 October 2026; revised for the move to one Rust workspace
([decisions.md](decisions.md#2026-11-08-one-rust-workspace)).

**The studio and development game can be reviewed now. A public game with
real NFT acquisition and burning is not ready to launch tomorrow.** Treat the
distinction as a release contract, not a warning hidden in a roadmap.

## Implemented

- Bound Sigil is selected for launcher icons, notification icons and the
  native/animated arrival. All four image masters are retained.
- One leptoskit app for the browser, Android and iOS; no desktop application.
  Every screen is built, and the 3D views and the hunt run on `leptoskit_3d`
  and `leptoskit_xr`: the camera's picture behind the fight in the native
  apps, depth from the phone or, where it measures none (the Seeker), from a
  depth model, and on Meta Quest an immersive session over the panel. The
  launcher icons, launch screens, portraits and mark come from the bundled
  images. Nothing has run on a phone or headset yet: see
  [environment.md](environment.md) for what the first runs should check.
- Six offices, 18 birth gifts, four temperaments, original gift lore, individual
  soul memories and three earned awakenings. Shared deterministic rules drive
  gameplay, preview, settlement and persistence.
- Per-reaper roster, XP, equipment, codex and memories. Canonical Core ownership
  is verified through Solana Kit before online mutations and growth follows
  ownership transfers. Wallet challenges prove an exact, expiring message.
- Location fixes with quality and age validation, expiry and latest-only
  storage on the server. Camera and depth processing stays on the device.
- Combat-log version mismatch is explained as an update requirement. The parser
  refuses impossible aim ranges and stops overlong logs before replay.
- Staging/production refuse the simulated NFT purchase, release, settlement
  and rift endpoints. Prototype transactions cannot masquerade as confirmed chain work.
- An on-chain program that escrows a released husk and burns it against the
  keeper's receipt, with a devnet runbook and a rehearsal on a local Surfpool.
  On devnet and a localnet the game releases and burns real demo husks with
  the player's wallet.
- Research, source links, design decisions and initial values are committed
  beside the implementation. See [identity research](research/reaper-identity-and-growth.md).

`cargo xtask check`, run on every merge and in CI, is the build and test
evidence. It establishes the exercised behaviours, not store approval,
production deployment or hardware comfort. A local browser build is not a
publicly hosted launch.

## Engineering still required for the real NFT game

| Work | Concrete missing behaviour | Dependency |
| --- | --- | --- |
| Device testing | The camera hunt, the depth model's speed on the Seeker, release builds and Quest's immersive switch have only run on recordings, this Mac's GPU and photographs. | A Seeker, an iPhone and a Quest ([environment.md](environment.md)). |
| Husk acquisition | Read supported real NFTs already owned by a proved wallet, with verified collection/trait mapping. Prototype `Nft.mockMint` and fixed prices are not usable. | Approved collections and rights. Start with bring-your-own NFTs rather than marketplace custody. |
| Release and absorption | Escrow, a receipt-bound burn and finishing a burn that did not land work on devnet for the demo husks. Still needed: real collections, rejected and expired transactions under load, and recovery tested against a deployed server. | Chosen asset standards, an audit of the program and deployed addresses. The unaudited program and its devnet runbook exist ([program.md](program.md), [deploy.md](deploy.md)); no mainnet program is deployed. |
| iOS wallet transport | An explicitly supported wallet's sign-message flow, secure return handling, lifecycle recovery and device tests. MWA cannot provide this. | Wallet choice and an iOS distribution plan compatible with NFT functionality. |
| NFT rifts | Replay or authoritatively simulate group results before granting transferable growth. | Server architecture and load testing. NFT reapers remain refused entry. |
| Account lifecycle | Guest recovery, account deletion and a retention policy that distinguishes personal data from transferable asset history. | Product/retention decisions and published privacy/contact details. |
| Production operations | Hosting, TLS, database backup/restore drill, receipt-key management/rotation, monitoring, RPC capacity and end-to-end tests against the deployed system. | Accounts, domains and secrets through Monosecret. |
| Abuse resistance | Rate limits/attestation appropriate to transferable growth. A deterministic replay proves rules compliance, not human motion. GPS does not prove physical presence. | Accepted threat model and real-money economics. |

These are code and operational work, not items that become complete merely by
receiving credentials. Do not remove the production simulation gate to ship.

## What Ifiok needs to provide or decide

1. **First public release:** a studio/closed devnet playtest, or the complete
   real-money NFT game. The latter needs the engineering above. Decide launch
   markets, age audience and whether the core game will remain playable free.
2. **Reaper collection:** Core collection address, network, supply/mint process,
   metadata hosting domain and authority, approved marketplace URL. Each asset's
   permanent genome must use the documented URI contract in [solana.md](solana.md).
3. **Husk collections and rights:** approved collections, written permission to
   use names, metadata/art and the burn mechanic, supported token standards and
   rarity mapping. Current names and invented items are unlicensed prototypes.
4. **Distribution:** Apple developer team and store listing, Google Play/Solana
   dApp Store account, final application identifiers, Android upload keystore,
   signing credentials and iOS provisioning. Keep secrets in Monosecret; never
   send a private key or password in chat.
5. **iOS product decision:** choose supported wallets and resolve how NFT-owned
   powers can fit the intended store/distribution route. Apple review is a
   product constraint, not an adapter bug; see the current rules below.
6. **Online infrastructure:** backend domain/hosting account, production RPC
   provider, backup location and restore owner, the deployment secrets and
   settings `crates/reapers_server/readme.md` lists, and receipt-signing
   custody/rotation.
7. **Public policies:** company/contact address, privacy and terms URLs, data
   retention/deletion decisions, support/removal-request process for real rift
   locations and target age/markets. We cannot invent the operator's identity.
8. **Physical playtest:** access to the Seeker and an iPhone for repeated camera
   opening/closing, motion calibration, denied permissions, foreground/background
   location transitions, battery/thermal behaviour and comfort. Pick testers
   unfamiliar with the controls; report gift usefulness and confusion.

## Current platform evidence

Apple's guideline 3.1.1 permits viewing owned NFTs only when their ownership
does not unlock app functionality; its current text also distinguishes purchase
links by storefront. The present NFT-reaper powers are therefore a concrete
review issue that must be resolved for the chosen iOS distribution route. This
is a reading of the published restriction, not a prediction of approval.
[Apple App Review Guidelines](https://developer.apple.com/app-store/review/guidelines/).

Google requires tokenised-asset declarations and prohibits promoting potential
earnings. Its NFT gamification policy restricts accepting value for a chance at
an NFT of unknown value and using NFTs to wager for real-world prizes. Supply,
mint disclosure and the proposed trading/growth economy need review against
these rules before submission; a purchasable blind shuffle is not assumed safe.
[Google Play Blockchain-based Content](https://support.google.com/googleplay/android-developer/answer/13607354?hl=en).

Solana Mobile's current platform table explicitly excludes iOS from MWA.
Android message signing is implemented; iOS parity requires a different
transport. The browser studio intentionally connects no wallet.
[Solana Mobile MWA documentation](https://docs.solanamobile.com/developers/mobile-wallet-adapter).

## Release signing

`leptoskit build --release --platform android|quest|ios` signs for the
stores from the environment only, and refuses to build a release with the
debug key or none: a missing variable stops it before anything compiles,
and a key file inside the repository is refused. Android and Quest produce
an App Bundle for Google Play and a signed APK for the Solana dApp Store or
sideloading, under `crates/reapers_app/.leptoskit/release/`; iOS produces an
`.ipa` from an archive. `--build-number N` sets the store's build number.

| Variable | For |
| --- | --- |
| `LEPTOSKIT_ANDROID_KEYSTORE_BASE64` (or `LEPTOSKIT_ANDROID_KEYSTORE`, a path outside the repository) | The upload keystore. |
| `LEPTOSKIT_ANDROID_KEYSTORE_PASSWORD`, `LEPTOSKIT_ANDROID_KEY_ALIAS`, `LEPTOSKIT_ANDROID_KEY_PASSWORD` | Its passwords and alias. |
| `LEPTOSKIT_IOS_TEAM_ID` | The Apple team. |
| `LEPTOSKIT_PUBLISH_APPLE_ISSUER_ID`, `LEPTOSKIT_PUBLISH_APPLE_KEY_ID`, `LEPTOSKIT_PUBLISH_APPLE_KEY` | App Store Connect's API key, for automatic signing. |

Supply them through Monosecret around the one build that needs them, and
never commit them:

```sh
cargo xtask fetch-depth-model
cd crates/reapers_app
msr --reason "Sign Reapers release" leptoskit build --release --platform android --build-number 1
```

Nothing here has been signed yet: there is no upload keystore or Apple team
for Reapers. iOS simulator builds do not establish App Store signing.

## Deployment order

Deploy the server and the clients with the same combat-log version together;
an older client receives an update refusal. Back up the database before
deployment. The keeper applies pending migrations when it starts, so a
deployment is a migration: run it against a copy first. Saved books lazily
accept absent soul memory without resetting growth. Do not share a
development database with production. The Rust keeper replaces the first
one with a database of its own (`crates/reapers_db`); players' books from the
first keeper's database are not migrated, so a deployment that has players
needs that import written first.
