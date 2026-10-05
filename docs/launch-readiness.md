# Launch readiness — 5 October 2026

**The studio and development game can be reviewed now. A public game with
real NFT acquisition and burning is not ready to launch tomorrow.** Treat the
distinction as a release contract, not a warning hidden in a roadmap.

## Implemented

- Bound Sigil is selected for launcher icons, notification icons and the
  native/animated arrival. All four image masters are retained.
- Android/iOS game and responsive browser studio; no desktop application.
- Six offices, 18 birth gifts, four temperaments, original gift lore, individual
  soul memories and three earned awakenings. Shared deterministic rules drive
  gameplay, preview, settlement and persistence.
- Per-reaper roster, XP, equipment, codex and memories. Canonical Core ownership
  is verified through Solana Kit before online mutations and growth follows
  ownership transfers. Wallet challenges prove an exact, expiring message.
- Foreground location permission, quality/age validation, expiry, clear-on-stop
  and latest-only serial uploads. Camera/depth processing stays on the device.
- Combat-log version mismatch is explained as an update requirement. The parser
  refuses impossible aim ranges and stops overlong logs before replay.
- Staging/production refuse the simulated NFT purchase, release, settlement
  and rift endpoints. Prototype transactions cannot masquerade as confirmed chain work.
- Android release tasks require a real keystore supplied through Monosecret.
  They cannot silently sign with the debug key.
- Research, source links, design decisions and initial values are committed
  beside the implementation. See [identity research](research/reaper-identity-and-growth.md).

Tests and build evidence are recorded in the [delivery report](verification/reaper-identities.md). They establish
the exercised behaviours, not store approval, production deployment or hardware
comfort. The local browser preview is not a publicly hosted launch.

## Engineering still required for the real NFT game

| Work | Concrete missing behaviour | Dependency |
| --- | --- | --- |
| Husk acquisition | Read supported real NFTs already owned by a proved wallet, with verified collection/trait mapping. Prototype `Nft.mockMint` and fixed prices are not usable. | Approved collections and rights. Start with bring-your-own NFTs rather than marketplace custody. |
| Release and absorption | Escrow the real NFT before releasing a husk; verify a hunt; post and confirm a receipt-bound, replay-protected burn; handle rejected/expired transactions and recovery without duplicate rewards. | Chosen asset standards, audited escrow program and deployed addresses. No mainnet program is supplied or deployed. |
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
   provider, backup location and restore owner, five deployment secrets listed
   in `reapers_server/lib/server.dart`, and receipt-signing custody/rotation.
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

## Android release signing

Resolve these four values together through the project's Monosecret setup
around one release build, for example `msr --reason "Sign Reapers release"
fvm flutter build appbundle` in `reapers_flutter`:

- `REAPERS_ANDROID_KEYSTORE`: absolute path to the private upload keystore.
- `REAPERS_ANDROID_STORE_PASSWORD`: its store password.
- `REAPERS_ANDROID_KEY_ALIAS`: the upload-key alias.
- `REAPERS_ANDROID_KEY_PASSWORD`: its key password.

No secret was read or created for this change. Missing values or a missing
keystore fail the release task with their names, never their contents. Debug
builds need none. iOS simulator builds do not establish App Store signing.

## Deployment order

Deploy server and clients with combat rules version 5 together. Back up the
database before deployment. JSON records lazily accept absent soul memory
without resetting growth; no table schema changes are needed for this feature.
Do not share a development database with production. Start the server with
`serverpod start` when reviewing live integration; agents do not start it.
The Serverpod MCP connection was unavailable in this session, so live reload,
live migrations and deployment were not represented as completed.
