# reapers_app

The Reapers game on [leptoskit](https://github.com/leptoskit/leptoskit): one tree of screens that renders in the browser and, as a Wasm guest, in the iOS, Android and Meta Quest apps. The reapers, the husks and the hunt are drawn in 3D by `leptoskit_3d`, and the room around the player comes from `leptoskit_xr`; both plug into the seams described below.

Every rule is `reapers_rules`'; this crate keeps the books, draws the ledger, and talks to the device, the simnet and the server.

## Run it

You need the leptoskit CLI at the revision the workspace pins:

```sh
cargo install --locked --git https://github.com/leptoskit/leptoskit --rev 996f9b1814996e5f6b3f565803d53266c3359e78 leptoskit_cli
cd crates/reapers_app
leptoskit dev --platform web             # the browser, live
leptoskit build --platform web           # dist/, for any static host
leptoskit build --platform ios           # the iOS app (Xcode)
leptoskit build --platform android       # the Android app (SDK, NDK, CMake)
leptoskit build --platform quest         # the Meta Quest app, as a panel (the Android SDK)
```

`build.rs` mirrors the repository's `assets/{nfts,music,models,fonts,branding}` into `assets/` before leptoskit bundles it; those mirrors are git-ignored. The models' copies leave out the vertex colours: the art build paints mask channels into them for a custom shader, and `leptoskit_3d` would draw them as colours. leptoskit serves `assets/` to the browser at `/assets/` and bundles it into the native apps.

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
| `REAPERS_AUTOPLAY=true` | The rules' simulated player, with a person's limits, fights every hunt and rift through the real view. |
| `REAPERS_GALLERY=true` | Debug builds open at the gallery: every reaper and outfit. |
| `REAPERS_DEPTH_PROBE=true` | Debug builds open at the depth bench: what the platform's AR measures of the room. |
| `REAPERS_DRAWN_ROOM=true` | Debug builds hunt in a kitchen drawn by hand, read as if the platform saw it. |

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
| `src/render/` | 3D on `leptoskit_3d`: the reaper rig, its look and appearance, cloth and playback, the stage, the husk rig and how it reacts, and the scenes, ticker and pointer they share. |
| `src/hunt/` | The hunt: the arena, the controller that keeps the log, the pose (`leptoskit_xr`, or a finger), the HUD and what is drawn over the scene. |
| `src/rift/` | A seat in a rift, in the hunt's arena. |
| `src/studio/` | The reaper studio and the gallery. |
| `src/dev/` | The depth bench and the drawn room. |
| `../reapers_world` | Where things are around the player, with no camera in it: the pose, the room read from depth, and where a rift stands. |
| `locales/en.arb` | Every string a player reads; `src/l10n.rs` is generated from it. |

## Seams

Each seam is a function of one props struct. The screens around them only ever call the seam.

| Seam | Props | What it is now |
| --- | --- | --- |
| `reaper_stage` | `ReaperStageProps { genome, weapon, outfit, turns_on_its_own, focus_face }` | The class's model on a `Scene3d`, on transparent ground, seen from chest height and lit as the sanctum was. Each line that shows a stage keeps its own scene and sends only what changed. Its silhouette stands in while the model loads, and for good where 3D cannot draw. |
| `husk_portrait` | `HuskPortraitProps { nft, size, spent }` | The bundled WebP plate in the browser; the husk's outline in its colour in the native apps, whose hosts cannot yet name a bundled image. |
| `hunt_view` | `HuntViewProps { brief, difficulty, autoplay, on_finished, on_difficulty }` | The whole hunt: the threshold, the sigil traced while the room is read, and the fight on `HuntSim::new(brief…, plan, difficulty)`. It calls `on_finished(log, outcome)` once, after the husk has come apart, with the log it recorded. |
| `rift_view` | `RiftViewProps { site, frame, report, on_leave, autoplay }` | The circle gathering over the husk at rest, then this reaper's `RiftSeat` fought in the hunt's arena against the frames, reporting whole standings. |
| `studio_view` | `StudioViewProps { genome }` | The studio; opens no save, server, wallet or camera. |

### The 3D

A scene lives on the host (the page's renderer, or the native app's), which loads the models, animates them on its own display clock and draws them; the game says what is in it with ops (`leptoskit_3d::client::Scene`). Screens are drawn afresh whenever what they read changes, so no render makes a scene: `render::scene` keeps one per place that shows one. Everything the game moves itself (a stage turning, a robe stirring, a hunt's tick) runs on one ticker, at the rules' thirty a second, only while it is on screen.

What `leptoskit_3d` cannot do, and what the port does instead:

- **No custom shaders.** The Flutter game drew every reaper surface with `reaper.fmat`. A look here is a material override per part (tint, a light of its own, a rim); `render/parts.rs` lists each model's parts by material, kept whole by a test that reads the models. The burn front, the hollowing and the sketched outline are gone; the hollowed outfit still loses its void parts.
- **No seeking a clip.** The host plays clips on its own clock, with crossfades. The studio plays, replays and changes speed, and holds a reaction's face with morph weights; it cannot hold an exact frame.
- **No moving a model's own parts.** The husks are rigid parts on pivots with no clips. The rig moves the creature whole (a step's rise and fall, the lean and twist of a blow, the reap dragging it up) where the Flutter rig also swung each leg and turned the head.
- **No per-material textures.** A rift's reliquary stack shows its frames, not the collection's portraits.

### The room

`hunt_view` and `rift_view` follow the phone through `leptoskit_xr`: `PhoneAr` with planes and depth on phones with ARKit or ARCore, the rotation sensor on phones without, and a dragged finger in a browser or a simulator. Taps strike along their ray. While the sigil is traced, the room is read by casting a grid of rays against the platform's depth and taking the floor from its planes (`reapers_world`). See [docs/environment.md](../../docs/environment.md), which also says what has not yet run on a device.

## The server

`ServerApi` sends `reapers_api`'s operations the way `leptoskit_rpc`'s client does (`POST /api/<name>`, an idempotency key on every mutation, the access token as a bearer header) through the game's own `HttpTransport`, so the browser, the apps and the desktop test run one client. A fresh install opens a guest account at `/auth/guest`; the refresh token is kept in the platform's secure store (the `secure-store` plugin; `localStorage` in a browser, which has none) and traded at `leptoskit_auth`'s `/auth/refresh`; proving a wallet at `/auth/wallet` switches to the wallet's account. A rift is `enter_rift` on `leptoskit_rpc`'s live socket (`/live`, the `websocket` plugin on the apps), with reports going up as `report_rift`.

The web game is served from its own origin and signs in as a native app does, with a bearer token rather than a cookie, so the server must name its origin as `REAPERS_ORIGIN` to answer it across origins.

## Music

`sound::Soundtrack` takes a `ScoreEngine`; `sound::device_engine` is the real one, on `leptoskit_plugin_audio` (Web Audio in the browser, the `audio` plugin in the apps), handed in by `Game::start` and silent everywhere else. Screens say `game.play(Score::…)` and, each tick of a hunt, `game.shape(hunt_mix(&sim))`, or `game.shape(sigil_mix(&trace))` while its sigil is traced. A browser holds the music back until the page is first pressed, and while it is hidden. leptoskit stages `assets/` into the native apps because the app draws 3D scenes, and the music comes with it.


## Tests

```sh
cargo test -p reapers_app                                           # the screens, keepers and clients
cargo test -p reapers_app --test devnet -- --ignored --nocapture    # the cluster flow, end to end
cargo test -p reapers_app --test render -- --nocapture              # reapers and a husk drawn headless (skips without a GPU)
cargo test -p reapers_world                                         # the pose, the room and a rift's place
```

`tests/render.rs` writes what it drew under `target/tmp/`.

The second builds the server and `reapers_ops`, starts PostgreSQL, a localnet and the server, and runs the game's backend through the whole devnet flow against them; `docs/deploy.md` says what it needs, and how to point it at devnet.

## Platform notes

- Wallets: the browser reaches its wallets through the Wallet Standard (`src/wallet/browser.rs`), written as `leptoskit_auth`'s web client does it. `solana_kit_browser`'s wallet would do, but at the pinned Solana Kit revision it requires wasm-bindgen 0.2.128 and leptoskit's CLI embeds 0.2.129, and one build holds only one.

- Fonts: leptoskit has no font-family utility. The web shell sets display type (headings) in Fraunces and the rest in DM Sans (`assets/app.css`); native apps use the system face.
- Native images: `Image` takes `file://` paths the guest cannot name, so portraits and the mark are drawn as vectors there.
- Meta Quest: `leptoskit build --platform quest` packages the Android app for Horizon OS as a panel (`[native.quest] immersive = false`): the screens are ordinary views, which an immersive session would not show. The hunt follows the headset through OpenXR where the `xr` plugin offers it; drawing the hunt into the headset's own frames needs a host that runs `leptoskit_xr::host::scene3d::draw`, which the generated host does not yet.
- The iOS app's `Info.plist` is app-owned (`ios/Info.plist`): ARKit needs its camera usage text.
- No gradients, no fixed positioning: scrims fall in steps, and the frame is the screen's height with the route scrolling inside it, so overlays cover the screen.
