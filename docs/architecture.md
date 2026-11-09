# How Reapers is built

## The shape

```
reapers/
├── crates/
│   ├── reapers_rules/           Every rule, catalogue and number.
│   ├── reapers_receipt/         The absorption receipt, signed by the keeper and checked on-chain.
│   ├── reapers_program/         The on-chain program, on pina.
│   ├── reapers_program_client/  Its generated client.
│   ├── reapers_api/             Every request and answer, shared by the keeper and the app.
│   ├── reapers_db/              The keeper's tables and migrations.
│   ├── reapers_server/          The keeper, on leptoskit and PostgreSQL.
│   ├── reapers_app/             The game, on leptoskit: browser, iOS and Android.
│   ├── reapers_landing/         The landing page, on leptoskit: a static site.
│   ├── reapers_ops/             Deploys and rehearses the program; brings up a localnet.
│   └── xtask/                   The checks, the guard on main, the art and music builds.
├── assets/       The models, music, portraits, typefaces and mark the game ships.
├── art/blender/  The scripts every 3D model is built from.
├── docs/
└── leptoskit.workspace.toml  The two leptoskit frontends: the game and the landing page.
```

It is one Cargo workspace, pinned by `rust-toolchain.toml` to the nightly leptoskit uses. leptoskit comes from git at one revision, patched in by name under `[patch.crates-io]`, and Solana Kit at one revision too, both in the root `Cargo.toml`. The on-chain program is the exception to the toolchain: `pina build` compiles it with Solana's platform tools.

The game was first written on another stack, and everything here is its port; [decisions.md](decisions.md#2026-11-08-one-rust-workspace) records why and how.

## One rule, written once

Everything that decides an outcome lives in `reapers_rules` and nowhere else: how a hunt plays, what a class is worth, what a refusal is, how a rift is sealed. The game runs those rules, compiled to Wasm, to play offline. The server runs the very same code, natively, to judge an online player. There is no second implementation to drift.

Three types carry most of it:

- **`PlayerState`** is everything a player owns and has done, as one value. Each thing a player can ask for is a method that returns the next state or a `RuleBreach` naming a `Refusal`. It reads no clock and rolls no dice: seeds and times are passed in, so the server decides them for an online player.
- **`ReaperState`** owns one reaper's fixed genome, experience, equipment, codex, soul memories and records. `ReaperSoul` derives the permanent birth gift and temperament from R1 fields. `SoulRules` combines the gift's preferred memory with level to derive its awakening; `HuntStats` and `HuntSim` execute it. `PlayerState` owns the roster, selection and account currency. The starter is non-tradable; an NFT reaper is identified by its Core asset address.
- **`HuntSim`** is one encounter, advanced a tick at a time from quantised inputs (aim in hundredths of a degree, turn speed in thousandths of a radian a second). A husk has a real position around the player, in metres, and moves through it: along the floor or through the air, inside the `HuntPlan` it was given. The same seed, the same plan and the same inputs always end the same way, on any device.

The server and the game are built for different targets, so every sine, arc tangent and square root the rules take goes through one implementation (`libm`), and lands on the same bits on both. The rules keep the numbers, order of operations and save format they had before the port; `tests/conformance.rs` holds them, to the bit, to fixtures that first implementation wrote, so saves, hunt logs and receipts written before the port read and replay the same way.

That determinism prevents a client from inventing a result or submitting its own growth counters. It does not prove a human performed the inputs: a bot can produce a valid log. The game records the plan and each tick's input into a version-seven `HuntLog` (9 bytes for the plan and the difficulty, 72 more when the room was surveyed and 153 more for what stands in it, then 8 a tick: where the crosshair points, how fast the phone turns, whether FIRE was pressed and ORB held, and which weapon is in hand) and sends that, not a result. A hunt played in a room read as a grid is version eight: the same, with the grid's length in 4 bytes and the grid after the plan. A log without a grid is still written as version seven, byte for byte, so every log written before replays as it did. The orb's aim is the one window that is not widened, and its roll comes from the hunt's seed, so the replay rolls what the phone rolled. The server replays it and pays whatever the replay earns, including bounded soul marks on successful absorption. It replays with windows 3% wider than the phone played with, so a hunt won by a hair is never lost to rounding. Impossible aim ranges, overlong logs and a plan too narrow to be fair are refused. A different combat-log version receives an update refusal.

Staging and production also refuse the simulated acquire, release, hunt-settlement and rift endpoints. The invented mints, balances and unverified group reports cannot become live NFT growth accidentally. Replacing this gate requires verified husk ownership, escrow, confirmed burns and authoritative rift results, as described in [nfts.md](nfts.md). [The soul research](research/reaper-identity-and-growth.md) records the design evidence; [launch readiness](launch-readiness.md) separates implemented behaviour from the dependencies still missing.

## The game

`crates/reapers_app` is one tree of screens on leptoskit. In the browser it is a Wasm client rendered into the page; on iOS and Android it is the same code as a Wasm guest inside a native host, which leptoskit builds from `crates/reapers_app/ios` and `android`. Its layout, build flags and seams are in [its readme](../crates/reapers_app/README.md).

```
src/
├── app.rs, lib.rs, guest.rs  The app, its routes and the native guest's entry.
├── routes/                   One file per screen; leptoskit writes routes.rs from them.
├── screens/                  Threshold, oath, sanctum, vault, armory, codex, roster, rifts, colophon.
├── ui/, theme.rs             The "funeral ledger" kit: hand-drawn boxes over straight rules, rows and marks.
├── state.rs                  The state every screen reads.
├── backend/                  Who keeps the books: the device, or the server.
├── simnet/                   The pretend devnet on the device.
├── chain.rs, wallet*         A real cluster, and the player's wallet.
├── seams/                    Where the 3D and camera views plug in.
├── sound.rs                  Which piece plays.
└── l10n.rs                   Generated from locales/en.arb. No screen hardcodes text.
```

The screens are written builder-style (`View().class(…).child(…)`), which `leptoskit.toml` enforces.

**Backends.** Screens read the player from one `Game` state, which asks a `GameBackend`. `LocalBackend` keeps the books in the device's store (the browser's `localStorage`, the `store` plugin natively) and is the whole game with no server. `RemoteBackend` sends the same requests to the keeper through `ServerApi`, `reapers_api`'s operations over the game's own HTTP. Which one a build uses is decided by `REAPERS_SERVER_URL` when it is compiled; without it the build is fully offline.

**Seams.** Every view that is 3D or the camera is a function of one props struct: `reaper_stage`, `husk_portrait`, `hunt_view`, `rift_view` and `studio_view`. Their bodies draw on `leptoskit_3d` and follow the phone with `leptoskit_xr`; the screens around them, and their tests, know only the props. `REAPERS_AUTOPLAY` lets the rules' own careful player fight through the real hunt view, so a hunt's log is real and the keeper settles it end to end.

**Wallets and NFT reapers.** The keeper's wallet challenge is short-lived, single-use and scoped to the authenticated guest, domain, chain and address. Solana Kit verifies the exact Ed25519 signature, strictly. In the browser the wallet is reached through the Wallet Standard; on Android, through Mobile Wallet Adapter. iOS has no wallet transport yet. The first proof links the guest; subsequent proofs restore that wallet's existing account without merging saves. The real wallet balance is separate from the husk market's simulated SOL.

**Drawing the husk out.** Before a hunt, the player traces the husk's `Sigil`: lights around them that catch in order. The sigil is a rule, so it lives in `reapers_rules`: `Sigil::draw` lays one out from the hunt's seed and the rarity's `SigilShape`, and `SigilTrace` follows the phone across it, reading no clock.

**Reading the room.** A hunt is played inside a `HuntPlan`: how far down the floor is, how far the way is clear in each direction, and what stands at the end of each. Whatever reads the room hands the hunt a plan; the rules never see a camera. The camera hunt takes the room from the platform's own depth, through `leptoskit_xr`; a device without it plays in `HuntPlan::OPEN`, an unsurveyed room. See [environment.md](environment.md).

**The room a husk walks through.** Beyond the polar plan, `crates/reapers_rules/src/room` holds the room as a `RoomGrid` of 10 cm gravity-aligned cells (a level top, an obstacle height, a wall or hidden floor each) that travels as a run-length-encoded byte string. From the grid and a vessel's `LocomotionProfile`, `RoomNav` derives the standable mask, regions, edges, jump links and the perch, peek, hide and open spots; `RoomPath` finds a route by Dijkstra over the links and A* inside a region; `Leap` solves the ballistic arc at 9.81 m/s² a tick at a time; and `HuskMotion` moves one husk's root through idle, turn, walk, edge, crouch, air, land and recover, exposing its position, heading, state, fraction, support height and next foothold. When the plan carries a grid (`HuntPlan::room`), `HuntSim` moves a husk that walks through it: the hunt still decides when it moves and why, from the same streams of chance, and a walker (`hunt_sim/walker.rs`, holding a `HuskMotion` over a shared `RoomNav` built with `RoomNav::for_hunt`) decides where and how, and copies its place back into the fields the rest of the hunt reads. The pose it hands the view carries the body's `scale` and its `HuskLocomotion` (state, fraction, support height, next foothold). A husk that flies, and a hunt without a grid, move as before. The research is [husks-in-the-room.md](research/husks-in-the-room.md).

**Where a rift stands.** A rift's `RiftRoom` holds one `GeoPoint` for its husk and every `RiftFrame` carries it, so whoever hosts the room is the one place that knows where the enemy is. `GeoFix` carries the device observation time and accuracy alongside its point; the shared rules validate it with a supplied clock, and the server validates again before accepting a location or a rift standing.

**Music.** `Soundtrack` decides which piece plays and whether it is heard, and a `ScoreEngine` makes the sound. Until the audio plugin hands in a real engine, the engine is silent, as it is in every test. The recordings are other people's, built by `cargo xtask build-music` and credited in `assets/music/CREDITS.md` and on the colophon page.

**Simnet.** `REAPERS_SIMNET=true` swaps the keeper for a `SimnetBackend`: a `LocalBackend` under its own save, which posts an airdrop, a buy, an escrow and a receipt-carrying burn to a `SimnetChain` on the device after the rules allow each one. They are real Solana wire transactions, signed by a burner wallet, and the release and burn are the program's own instructions from `reapers_program_client`. A slip confirms each transaction; the simnet screen is its explorer. A simnet build may also set `REAPERS_SURFPOOL_URL`, which adds a client for a local validator to the explorer: health, genesis, balance and a once-per-burner faucet. See [solana.md](solana.md#the-simnet).

**A real cluster.** With `REAPERS_SERVER_URL` and `REAPERS_CLUSTER=devnet` or `localnet`, the vault lists the demo husks the player's wallet holds, releasing one has the wallet sign and send `ReleaseCore`, and a won hunt comes back with the keeper's receipt and an unsigned absorb transaction for the wallet to sign and send (`src/chain.rs`).

**Autoplay.** `REAPERS_AUTOPLAY=true` has a simulated player fight every hunt and rift.

## The landing page

`crates/reapers_landing` is the page that says what Reapers is and leads into the game. It is a second leptoskit frontend, the `landing` application of the root `leptoskit.workspace.toml` beside the game's `product`, with its own build and its own port (3011; the game's is 3010). It is web only and renders in the browser, so `leptoskit build` makes a static site in `dist/` for any static host. Release builds compile the Wasm for size, run `wasm-opt`, and write brotli and gzip copies of every file.

```
src/
├── lib.rs           Mounts the page in the browser.
├── config.rs        Where the game is: REAPERS_APP_URL when the page is built, or the game's leptoskit dev.
├── routes/          One page, and the same page for any other address.
├── page/            The sections, in reading order: the hook, why, the hunt in four beats, the capture,
│                    the six callings, the husks, the chain, the way in, the colophon.
└── l10n.rs          Generated from locales/en.arb.
web/                 The page's HTML shell; its title, description and noscript line are held to en.arb by a test.
assets/app.css       What leptoskit has no utility for: the two typefaces, line height, the glows,
                     and the capture's scroll-pinned stage.
```

It shares the game's look (the funeral ledger: ink, bone and ash, Fraunces over DM Sans) but none of its code: it depends on `reapers_rules` only in its tests, to hold the callings it shows to the rules'. Its drawings are leptoskit vectors and its pictures are the husks' portraits from the art build, which `build.rs` mirrors from `assets/` with the typefaces and the mark; no screenshot of a camera is shown, since a camera's picture is someone's home. Its motion is CSS, driven by the scroll where the browser can, and still for anyone who asks for reduced motion. Its strings are in `locales/en.arb`, and the music's credit lines on it are `assets/music/CREDITS.md`'s, held word for word by a test.

`tests/page.rs` builds the page headlessly at a phone's and a desk's width with leptoskit's layout engine, which also checks every Tailwind class: in the browser, a class leptoskit cannot draw is a blank page.

## Art

There are no hand-made model files. Every reaper, weapon and husk is built by a Python script run inside headless Blender, driven by `cargo xtask build-art`, then compressed by gltfpack (meshopt, quantized, every named node, material and extra kept) into `assets/models`. The husk scripts read their dimensions and seal positions from `reapers_rules`' `Vessel`, written to `art/build/vessels.json`, and fail the build if a creature does not fit them, so the model and the hit-testing cannot disagree. The same build renders every NFT's portrait from its creature and its look, into `assets/nfts` as WebP. See `art/blender/README.md`.

## Tools and continuous integration

`cargo xtask` (`crates/xtask`, aliased in `.cargo/config.toml`) is every tool the repository has:

- `check` runs, in order and stopping at the first failure: `cargo fmt --check`; clippy over every target, denying warnings; the Wasm builds of `reapers_rules`, `reapers_receipt`, `reapers_api` and `reapers_program_client`; `pina build` and a 56 KiB ceiling on the program; Metaplex Core fetched once for the program's tests; every test of the workspace's default members (`--fast` leaves out `reapers_server`, whose tests start PostgreSQL); and, when the leptoskit CLI is installed, `leptoskit workspace check` and `leptoskit build --platform web` for the app and for the landing page. Before it starts, it fails with the fix when a tool it needs is missing.
- `hooks install` writes the `reference-transaction` and `pre-commit` hooks that let `main` move only to a commit with a verification record.
- `merge` refuses on `main`, with uncommitted work, without the hooks or when `main` has moved on; pushes the branch and opens a pull request into `main`, or reuses the open one; runs every check; refuses if the checks changed the tree; writes the record, fast-forwards `main` and deletes the record whether or not that worked; then pushes `main`, which is what merges the pull request, comments the check report on it, confirms GitHub marked it merged and deletes the branch on origin. It needs the GitHub CLI signed in, and says so before the checks start.
- `build-art` and `build-music` build the assets.

`.github/workflows/ci.yml` runs `cargo xtask check` on every pull request, every push to `main`, and by hand. It reads the repository with no write permission and keeps no credentials in the checkout; every action is pinned by commit. It installs the pinned toolchain, PostgreSQL 17's binaries (the server's tests start their own cluster with `initdb` and `pg_ctl`), Agave for `cargo-build-sbf` and Metaplex Core, `pina` 0.24.0 and the leptoskit CLI, caching the tools, Agave and Metaplex Core by version and the build by `Cargo.lock`. Only `main` writes the build cache. The job has a two-hour ceiling and cancels a superseded run on the same ref.

## The server

The keeper is `crates/reapers_server`, on leptoskit's server crates and PostgreSQL. How to run it is in its [readme](../crates/reapers_server/readme.md).

- **One contract.** Every request is declared once, in `crates/reapers_api`, as a leptoskit operation: a query, a mutation, an action or a stream. Each is an argument struct the game builds on any target, Wasm included, and sends to `POST /api/<name>` or over the live socket. Only the keeper's build has the bodies, and they only pass the request to the `Keeper`, which the server puts in each request's context beside the session.
- **Sign-in.** A fresh install plays at once as a guest: `POST /auth/guest` opens an account and its books and answers a token pair, so nobody signs up before their first hunt. Accounts and sessions are leptoskit_auth's: its session layer recognizes the tokens on every request and its `/auth/refresh` trades a refresh token for a new pair. Wallet proof (`POST /auth/wallet`) links the guest, or restores the wallet's existing account.
- **The books.** Each player's `PlayerState` is stored whole, as its save format, in one `player_books` row. Every change is one mutation: a serializable transaction that reads the books, checks NFT reapers on the chain, applies one shared rule and stores the result, run again when another transaction gets in its way. So two requests that would spend the same anima cannot both commit. No operation takes a player id from the client.
- **NFT reapers.** Before an NFT-bearing player's mutation, the keeper reads finalized ownership through Solana Kit's typed RPC and decodes Core accounts with its Metaplex Core client. It validates program, collection, owner and the configured genome URI, and checks the chain's genesis. Canonical `reaper_records` rows are read in address order inside the same transaction as the player's books, so growth and player state commit together or not at all. Import starts at level one, never with submitted or offline growth; a transfer retains that asset's canonical growth. A failed RPC leaves the books intact and fails the action. Ownership is a finalized snapshot, not atomic with a later on-chain transfer.
- **Location.** `update_location` and `location` require a session and use the same row. Only the latest `GeoFix` is kept. A later observation replaces an earlier one, and two of the same instant keep the more precise, so a phone's refinement lands and a delayed coarser copy does not. `None` clears it, and stale readings are never returned. This is positioning, not proof of physical presence: coordinates reported by a modified client remain untrusted.
- **Receipts.** When a hunt is absorbed, the keeper rolls how well the reaper fed from the operating system's secure random numbers and signs the result with Ed25519, in the 98-byte version-two receipt layout the Reapers program checks. When the husk was escrowed on-chain at its release, the receipt is made out to the player's proved wallet for the husk's real mint, after the keeper has read at a finalized slot that the escrow is still there and is that wallet's; the answer then carries the receipt and an unsigned transaction (the Ed25519 check, then the program's `Absorb`) for the wallet to sign. See [nfts.md](nfts.md).
- **Rifts.** `enter_rift` is a stream on the live socket: frames come down several times a second, and the reaper's reports go up through `report_rift`. `RiftHall` keeps the open rifts in memory and moves each on ten times a second.
- **Refusals** fail an operation as a conflict whose message is `refused:` and the shared `Refusal`'s name, so an online player is told exactly what an offline one would be.

Secrets are never committed. `crates/reapers_server/config/passwords.toml` is git-ignored and created from `passwords.example.toml`, and is read only in development and test. A deployed keeper reads `REAPERS_SECRET_<name>` from its environment and refuses to start in staging or production without the database, sign-in's keys and the receipt signing secret. Staging and production also refuse the simulated acquire, release, hunt settlement and rifts, with no flag to allow them.

## The program

`crates/reapers_program` escrows a released husk at `["escrow", mint]` and burns it when a transaction carries the keeper's receipt for it, checked through the Ed25519 program. [program.md](program.md) says what it checks and why; [deploy.md](deploy.md) is the devnet runbook, and `reapers_ops rehearse` runs that runbook against a local Surfpool.

## Rifts

`RiftRoom` (in the rules) is the keeper of what a circle shares: who is in it, the ward, the phases, the verdict. Like the hunt it reads no clock; its host calls `advance` as time passes. The game hosts a room itself when offline, and the server hosts rooms for many (`RiftHall`). Both are thin.

Each reaper runs their own `HuntSim` in rift mode on their own device and reports a running total of the damage they have dealt. Reports are whole standings, not changes, so one that is lost, repeated or late never miscounts. The shrieks come from a seed-only timetable, so every phone in the circle is told to hold still at the same moment.

Shades are the same simulated player the tests use, stepped by the room.

A development build may set `REAPERS_DEMO_RIFT=true` with `REAPERS_SERVER_URL`. It uses the same server rift: two real clients that open the same site are seated in one `RiftRoom`, share one ward, and are each paid once from their own books. The demo waives only attunement. Demo rooms are keyed apart from ordinary rooms, release builds ignore the flag, and `enter_rift` with `demo` passes the same development/test gate as the rest of the simulated economy. See [solana.md](solana.md#demo-circles).

## Known limits of this design

- **The camera's picture is not behind the hunt on native phones yet.** The generated iOS and Android hosts do not hand the XR session's camera picture and depth to the 3D view, so the hunt is drawn over the veil while the husk is tracked into the real room. The `leptoskit_xr` adapter that does it exists (`leptoskit_xr::host::scene3d::draw`); a host has to call it.
- **`leptoskit_3d` draws only its own materials.** No custom shaders (the Flutter char, hollowing and sketched outline are gone), no clip seeking, and husks move as one piece. A husk behind real furniture fades as a whole rather than being cut pixel by pixel.
- **The room is read once, from where you stand.** A plan describes the floor and what stands on it around the player before the hunt. It does not follow you walking or notice anything moved. See [environment.md](environment.md).
- **The plan is the device's word.** The server checks a plan is wide enough to be fair and has room for the husk, not that the room is really that shape. A made-up room cannot win a hunt, but it can bring the husk closer, where its seals are easier to hit.
- **Rifts trust the client.** A hunt is replayed; a rift is not. NFT reapers cannot enter until rift input replay is implemented, so trusted reports cannot mint tradable growth. The starter can still play. Making it verified means sending each seat's input log and replaying it against the shared ward's history.
- **Rifts live in one process.** Every reaper in a circle must be connected to the same server instance. Running more than one needs the room moved to shared state or the circle pinned to an instance.
- **The player is one row.** That is the right size now and will not be once a codex holds thousands of entries.
- **Receipts are posted by the wallet, not the keeper.** For a husk escrowed on-chain, the keeper signs the receipt and builds the burn transaction; the player's wallet signs and sends it, and the program checks the receipt. A husk from the stand-in market still has a receipt only the books keep.
- **Guest sessions are written beside leptoskit_auth, not by it.** leptoskit_auth has no guest sign-in, so the keeper writes guest accounts and their sessions into leptoskit_auth's tables as its own sign-in writes them. leptoskit_auth then treats them as its own; when it gains an anonymous provider, that code goes.
