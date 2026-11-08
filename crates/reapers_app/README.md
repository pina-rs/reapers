# reapers_app

The Reapers game on [leptoskit](https://github.com/leptoskit/leptoskit): one tree of screens that renders in the browser and, as a Wasm guest, in the iOS and Android apps. Every screen is here; the 3D and camera views plug into the seams described below.

Every rule is `reapers_rules`'; this crate keeps the books, draws the ledger, and talks to the device, the simnet and the server.

## Run it

You need the leptoskit CLI at the revision the workspace pins (`53ca7405`):

```sh
cargo install --locked --git https://github.com/leptoskit/leptoskit --rev 53ca7405 leptoskit_cli
cd crates/reapers_app
leptoskit dev --platform web             # the browser, live
leptoskit build --platform web           # dist/, for any static host
leptoskit build --platform ios           # the iOS app (Xcode)
leptoskit build --platform android       # the Android app (SDK, NDK, CMake)
```

`build.rs` mirrors the repository's `assets/{nfts,music,models,fonts,branding}` into `assets/` before leptoskit bundles it; those mirrors are git-ignored.

`leptoskit dev --platform ios` runs the app in every booted iOS Simulator, and `--platform android` on every attached Android phone or emulator, swapping each rebuilt guest in without reinstalling.

## Build flags

The build flags are environment variables of `leptoskit build` and `leptoskit dev`, read when the app is compiled (`src/config.rs`):

| Variable | Effect |
| --- | --- |
| `REAPERS_SERVER_URL=http://localhost:8080/` | Play against the server. Without it the game is fully offline. Add the server's origin to `[native.plugin-options.http]` for the apps, and give the server the web game's origin as `REAPERS_ORIGIN`. |
| `REAPERS_CLUSTER=devnet` | With a server, real demo husks on devnet, moved by the player's wallet. `localnet` is a validator on this machine (`reapers_ops localnet`). |
| `REAPERS_SOLANA_RPC_URL=https://…` | With a cluster, the RPC the game reads and sends through. Defaults to devnet's public RPC, or `http://127.0.0.1:8899` for `localnet`. The apps need its origin in `[native.plugin-options.http]`. |
| `REAPERS_SIMNET=true` | The offline game on a pretend devnet: a burner wallet, and a signed Solana transaction for every purchase, escrow and burn. |
| `REAPERS_SURFPOOL_URL=http://localhost:8899` | With the simnet, a real local Solana RPC in the explorer: health, balance, a faucet, and sending a simnet transaction there. |
| `REAPERS_DEMO_RIFT=true` | With a server, waives attunement for a shared demo rift. Ignored by release builds. |
| `REAPERS_AUTOPLAY=true` | A simulated player fights every hunt. |
| `REAPERS_GALLERY=true`, `REAPERS_DEPTH_PROBE=true` | Debug builds open at the bench (the studio seam for now). |

```sh
REAPERS_SIMNET=true leptoskit dev --platform web
```

A combination that cannot work (the simnet with a server, a demo without one, Surfpool without the simnet, a cluster without a server) opens a "could not be read" notice and logs the reason, instead of half working.

## Playing on devnet

A cluster build plays against the server and moves real husks with the player's wallet. The server keeps the books and signs the receipts; the wallet holds the husks and signs every transaction that moves one; the game sends them (`src/chain.rs`).

```sh
REAPERS_SERVER_URL=https://keeper.example/ REAPERS_CLUSTER=devnet leptoskit build --platform web
```

1. **Connect a wallet** in **Your reapers**: a browser wallet through the Wallet Standard, or Mobile Wallet Adapter on Android. iOS has none. The wallet signs the server's challenge, and the server links it.
2. **The vault** lists the demo husks the wallet holds, as "In your wallet": the Core assets of the five demo collections `reapers_ops mint-demo` makes, which the server names from its manifest (`REAPERS_DEMO_HUSKS`, in `chain_status`).
3. **Releasing** one buys its stand-in into the books if it is not there (the simulated market's price), has the wallet sign `ReleaseCore` (from `reapers_program_client`), sends it, waits until it is finalized, and then calls the server's `release` with the asset as the escrow. A husk already in escrow ("In escrow") skips the chain.
4. **A won hunt** comes back from `settle_hunt` with a `ChainReceipt` and its unsigned absorb transaction. The wallet signs it, the game sends it, waits for it to confirm, checks the asset is Core's one-byte tombstone, and slips the burn in front of the player; the transaction sheet links to Solana Explorer.

The Reapers program's errors, and a wallet that declines, are explained in the player's words (`lookups::chain_failure_message`). A burn that fails after its hunt was settled leaves the husk in escrow, and says so. The codex and the vault then show it as not yet burned, with **Finish the burn**: the server's `absorb_transaction` builds the same receipt again on a fresh blockhash, and the wallet signs and sends it as the first time (`GameBackend::finish_burn`).

To play the whole flow on this machine, bring up a localnet and a server on it (`docs/deploy.md`, "The game on a localnet"), then:

```sh
REAPERS_SERVER_URL=http://127.0.0.1:8080/ REAPERS_CLUSTER=localnet leptoskit dev --platform web
```

## Layout

| Path | What it is |
| --- | --- |
| `src/backend/` | `GameBackend`, the seam every screen asks: `LocalBackend` (the device's store, under the save's long-standing key and JSON), `RemoteBackend` over `ReapersApi`, `ServerApi` (the server's client over `reapers_api`), and the rifts' sessions. |
| `src/chain.rs` | A real cluster: the wallet's demo husks, releasing with `ReleaseCore`, and signing and sending the server's burn. |
| `src/program.rs`, `src/receipt.rs` | The Reapers program's instructions from `reapers_program_client`, and the receipt from `reapers_receipt`. |
| `src/simnet/` | `SimnetChain` (burner wallet, ledger, real wire transactions, the program's own release and burn), `SimnetBackend`, and the Surfpool client and faucet. |
| `src/wallet.rs`, `src/wallet/` | The wallet: Mobile Wallet Adapter on Android through `solana_kit_leptoskit`, the Wallet Standard in the browser. |
| `src/state.rs` | `Game`: the state every screen reads, as `Copy` signals, provided as context. |
| `src/screens/`, `src/routes/` | The screens, and the file routes that show them (`routes.rs` is generated). |
| `src/ui/`, `src/theme.rs` | The funeral-ledger kit: hand-drawn boxes, rows, meters, pips, sheets and dialogs. |
| `src/seams/` | Where the 3D and camera views plug in. |
| `locales/en.arb` | Every string a player reads; `src/l10n.rs` is generated from it. |

## Seams

Each seam is a function of one props struct, with a 2D stand-in honouring it. The 3D port replaces the function bodies and nothing else.

| Seam | Props | Contract |
| --- | --- | --- |
| `reaper_stage` | `ReaperStageProps { genome, weapon, outfit, turns_on_its_own, focus_face }` | Fills its box on transparent ground; the reaper stands centred, seen from chest height, lit with `theme::reaper_light`. |
| `husk_portrait` | `HuskPortraitProps { nft, size, spent }` | A `size`-square plate of the NFT's portrait, faded when `spent`. |
| `hunt_view` | `HuntViewProps { brief, difficulty, autoplay, on_finished, on_difficulty }` | Owns the hunt from the threshold to its last tick; records a `HuntLog` from `HuntSim::new(brief…, plan, difficulty)` and calls `on_finished(log, outcome)` once. The route settles it with the keeper and shows the result. |
| `rift_view` | `RiftViewProps { site, frame, report, on_leave, autoplay }` | Runs this reaper's `HuntSim` in rift mode against the frames and reports whole standings. |
| `studio_view` | `StudioViewProps { genome }` | The studio; opens no save, server, wallet or camera. |

The stand-in hunt lets the rules' own careful player fight, so a hunt settles end to end before the camera exists.

## The server

`ServerApi` sends `reapers_api`'s operations the way `leptoskit_rpc`'s client does (`POST /api/<name>`, an idempotency key on every mutation, the access token as a bearer header) through the game's own `HttpTransport`, so the browser, the apps and the desktop test run one client. A fresh install opens a guest account at `/auth/guest`; the refresh token is kept in the platform's secure store (the `secure-store` plugin; `localStorage` in a browser, which has none) and traded at `leptoskit_auth`'s `/auth/refresh`; proving a wallet at `/auth/wallet` switches to the wallet's account. A rift is `enter_rift` on `leptoskit_rpc`'s live socket (`/live`, the `websocket` plugin on the apps), with reports going up as `report_rift`.

The web game is served from its own origin and signs in as a native app does, with a bearer token rather than a cookie, so the server must name its origin as `REAPERS_ORIGIN` to answer it across origins.

## Integration points

- Music: `sound::Soundtrack` takes a `ScoreEngine`; `leptoskit_plugin_audio` provides the real one.

## Tests

```sh
cargo test -p reapers_app                                           # the screens, keepers and clients
cargo test -p reapers_app --test devnet -- --ignored --nocapture    # the cluster flow, end to end
```

The second builds the server and `reapers_ops`, starts PostgreSQL, a localnet and the server, and runs the game's backend through the whole devnet flow against them; `docs/deploy.md` says what it needs, and how to point it at devnet.

## Platform notes

- Wallets: the browser reaches its wallets through the Wallet Standard (`src/wallet/browser.rs`), written as `leptoskit_auth`'s web client does it. `solana_kit_browser`'s wallet would do, but at the pinned Solana Kit revision it requires wasm-bindgen 0.2.128 and leptoskit's CLI embeds 0.2.129, and one build holds only one.

- Fonts: leptoskit has no font-family utility. The web shell sets display type (headings) in Fraunces and the rest in DM Sans (`assets/app.css`); native apps use the system face.
- Native images: the mobile hosts bundle only the guest, and `Image` takes `file://` paths the guest cannot name, so portraits and the mark are drawn as vectors there.
- No gradients, no fixed positioning: scrims fall in steps, and the frame is the screen's height with the route scrolling inside it, so overlays cover the screen.
