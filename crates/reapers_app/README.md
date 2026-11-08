# reapers_app

The Reapers game on [leptoskit](https://github.com/leptoskit/leptoskit): one tree of screens that renders in the browser and, as a Wasm guest, in the iOS and Android apps. It is the port of `reapers_flutter`, minus the 3D and camera views, which plug into the seams described below.

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

`build.rs` mirrors the repository's `assets/{nfts,music,models}` and the Flutter game's fonts and mark into `assets/` before leptoskit bundles it; those mirrors are git-ignored.

## Build flags

The Flutter game's `--dart-define`s are environment variables of `leptoskit build` and `leptoskit dev`, read when the app is compiled (`src/config.rs`):

| Variable | Effect |
| --- | --- |
| `REAPERS_SERVER_URL=http://localhost:8080/` | Play against the server. Without it the game is fully offline. Add the server's origin to `[native.plugin-options.http]` for the apps. |
| `REAPERS_SIMNET=true` | The offline game on a pretend devnet: a burner wallet, and a signed Solana transaction for every purchase, escrow and burn. |
| `REAPERS_SURFPOOL_URL=http://localhost:8899` | With the simnet, a real local Solana RPC in the explorer: health, balance, a faucet, and sending a simnet transaction there. |
| `REAPERS_DEMO_RIFT=true` | With a server, waives attunement for a shared demo rift. Ignored by release builds. |
| `REAPERS_AUTOPLAY=true` | A simulated player fights every hunt. |
| `REAPERS_GALLERY=true`, `REAPERS_DEPTH_PROBE=true` | Debug builds open at the bench (the studio seam for now). |

```sh
REAPERS_SIMNET=true leptoskit dev --platform web
```

A combination that cannot work (the simnet with a server, a demo without one, Surfpool without the simnet) opens a "could not be read" notice and logs the reason, instead of half working.

## Layout

| Path | What it is |
| --- | --- |
| `src/backend/` | `GameBackend`, the seam every screen asks: `LocalBackend` (the device's store, the Flutter save's key and JSON), `RemoteBackend` over `ReapersApi`, and the rifts' sessions. |
| `src/simnet/` | `SimnetChain` (burner wallet, ledger, real wire transactions), `SimnetBackend`, and the Surfpool client and faucet. |
| `src/wallet.rs` | Wallet proofs: Mobile Wallet Adapter on Android through `solana_kit_leptoskit`. |
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

## Integration points

- `TODO(integration): reapers_api`: implement `backend::ReapersApi` over the server's client and hand it to `RemoteBackend::new` in `state.rs`. Until then a server build is unreachable.
- `TODO(integration): reapers_receipt`: `receipt.rs` builds the 98-byte receipt field for field from the Dart layout; swap in the shared crate.
- Music: `sound::Soundtrack` takes a `ScoreEngine`; `leptoskit_plugin_audio` provides the real one.

## Platform notes

- Fonts: leptoskit has no font-family utility. The web shell sets display type (headings) in Fraunces and the rest in DM Sans (`assets/app.css`); native apps use the system face.
- Native images: the mobile hosts bundle only the guest, and `Image` takes `file://` paths the guest cannot name, so portraits and the mark are drawn as vectors there.
- No gradients, no fixed positioning: scrims fall in steps, and the frame is the screen's height with the route scrolling inside it, so overlays cover the screen.
