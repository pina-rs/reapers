# Reapers: notes for agents

Reapers is an AR game for phones: you are a reaper, and you hunt dead NFTs through the camera. It is one Cargo workspace: the rules, a leptoskit app for the browser, iOS and Android, a leptoskit server on PostgreSQL, and an on-chain program on pina. Read [README.md](README.md) for what it is and [docs/architecture.md](docs/architecture.md) for how it fits together before changing anything.

## Layout

| Path | What it is |
| --- | --- |
| `crates/reapers_rules/` | Every rule, catalogue and number. The game (as Wasm) and the server both run it. |
| `crates/reapers_receipt/` | The absorption receipt: what the server signs and the program checks. |
| `crates/reapers_program/` | The on-chain program, built with pina. |
| `crates/reapers_program_client/` | The program's Rust client. Its `src/generated` is written by `pina generate`. |
| `crates/reapers_api/` | Every request between the game and the server, as leptoskit operations. |
| `crates/reapers_db/` | The server's tables and migrations. |
| `crates/reapers_server/` | The keeper: the server. |
| `crates/reapers_app/` | The game, on leptoskit. |
| `crates/reapers_ops/` | Deploys and rehearses the program; brings up a localnet. |
| `crates/xtask/` | `cargo xtask`: the checks, the hooks, the merge, the art and music builds. |
| `assets/` | What the game ships: compressed models, music, portraits, typefaces, the mark. |
| `art/blender/` | Python scripts that build every 3D model inside Blender. |
| `docs/` | The rulebook, the architecture, the program, the NFT plan, the research. |

## Rules that are easy to break

- **A rule is written once, in `reapers_rules`.** If the game and the server would both need to know it, it does not belong in either of them. A number that changes how the game plays goes in `HuntRules`, `RiftRules` or the enum it describes, and in `docs/gameplay.md`.
- **The simulation is deterministic.** `HuntSim`, `PlayerState` and `RiftRoom` read no clock and roll no dice of their own. Seeds and times are passed in: the server replays hunts and a replay must match. Do not add `SystemTime::now()`, `Instant`, `rand` or `getrandom` to the rules. Take every sine, arc tangent and square root through the crate's `math` module, never `f64::sin` and friends: the server runs natively and the game as Wasm, and both must land on the same bits.
- **The rules are held to their fixtures.** `tests/conformance.rs` replays fixtures written before the rules were Rust, to the last bit. A rule changed on purpose changes its test; the fixtures are never regenerated (`crates/reapers_rules/tests/fixtures/README.md` says why).
- **No hardcoded strings in the app.** Every string a player reads is in `crates/reapers_app/locales/en.arb`. `leptoskit build` writes `src/l10n.rs` from it; never edit that file.
- **leptoskit is written builder-style.** `leptoskit.toml` sets `markup = "builder"` and denies markup: write `View().class(…).child(…)`, never `view!`. Screens are files under `src/routes`; `leptoskit build` writes `src/routes.rs`.
- **The app's 3D and camera views live behind seams.** `src/seams/` holds one function per view and a props struct that is its whole contract. A seam's body may change; its props, and the screens around it, should not need to.
- **3D models are built, never edited.** Change a script under `art/blender/`, run `cargo xtask build-art`, and commit the script with the models it produced in `assets/models` (and the portraits in `assets/nfts`). Blender 5.1 and gltfpack 0.25 are pinned in `crates/xtask/src/art.rs`; another version is an upgrade done on purpose, with every model rebuilt. See `art/blender/README.md`.
- **The music is other people's, and credited.** `assets/music/*.ogg` comes from `cargo xtask build-music`, which fetches each recording from its maker, checks its SHA-256 and re-encodes it as Vorbis q2. All three are Kevin MacLeod's, under CC BY 4.0: their credit lines on the colophon page and in `assets/music/CREDITS.md` are the composer's own words and a condition of using them. Change a piece and you change both; a test in `xtask` fails when they disagree. Only CC0 or CC BY recordings; never anything generated.
- **The depth model is fetched, never committed.** Phones without platform depth read the room with Depth Anything V2 Small (Apache-2.0, 27 MB), which `cargo xtask fetch-depth-model` puts in `assets/depth` from the repository's `depth-model-v1` release, checking its SHA-256. `build.rs` bundles it and `.gitignore` refuses it. A build without it runs; those phones then read no room. See `docs/environment.md`.
- **Keys are never committed.** The program's keypair, the deployer and every other `*-keypair.json` live outside the repository (`docs/deploy.md` says where); `.gitignore` refuses them. So does the server's `config/passwords.toml`.
- **leptoskit and Solana Kit are pinned in one place.** The root `Cargo.toml` names every leptoskit crate at one revision under `[patch.crates-io]`, and every Solana Kit crate at one revision under `[workspace.dependencies]`. Crates take them with `workspace = true`, never a git dependency of their own. The leptoskit CLI is installed at that same revision (README, and `LEPTOSKIT_REV` in CI), and the app pins `wasm-bindgen` to the version the CLI embeds. Move them together.
- **The program is built by `pina build`, and stays small.** Its profile (`opt-level = 2`, fat LTO) is in the root `Cargo.toml`; `cargo xtask check` fails a program over 56 KiB. Everything under `crates/reapers_program/{idl,migrations}`, its `tests/abi_layout.rs` and the client's `src/generated` is written by pina: change the program, then run `pina generate` and `pina migrations check`.
- **Migrations come from the tables.** After changing a `#[table]` in `crates/reapers_db`, write the migration with `cargo run -p reapers_db --bin reapers-migration -- <name>`, and never edit an applied one.
- **The Seeker's GPU is fragile.** A camera closing and per-scene environment maps once reset its GPU and broke every texture until a restart. Bake an environment map once and share it, and open and close hunts on the Seeker while reading its log before changing how Android draws. See `docs/environment.md`, "The Seeker's GPU".

## Landing a change

There are no pull requests here. Work is merged into `main` locally, and `main` is guarded by git hooks so that the only way in is through the checks.

```sh
cargo xtask hooks install        # once per clone: guard main
git switch -c feat/your-change   # work and commit on a branch
cargo xtask merge                # run every check, then fast-forward main
cargo xtask merge --push         # the same, then push main to origin
```

- Never commit on `main`, and never move it by hand with `git merge`, `git reset` or `git pull`. The hooks refuse all of these.
- `main` only fast-forwards. If it has moved since the branch was cut, `git rebase main` first.
- Clean up `wip:` commits into conventional commits before merging: what lands on `main` is what gets pushed.
- Do not open pull requests, and do not use `gh pr`.
- `.github/workflows/ci.yml` runs the same `cargo xtask check` on GitHub for pull requests and pushes to `main`. It is a mirror of the local gate, never a replacement for running the checks before merging.
- `REAPERS_SKIP_HOOKS=1` bypasses the hooks. It is for recovery work the user has asked for, never for getting past a failing check.

## Commands

Rust is pinned by `rust-toolchain.toml`; rustup installs it on first use.

```sh
cargo xtask check             # fmt, clippy, Wasm and SBF builds, every test, the web build
cargo xtask check --fast      # the same without the server's PostgreSQL tests
cargo xtask merge             # the full checks, then merge this branch into main

cargo test -p reapers_rules                                  # the rules
cargo run --release -p reapers_rules --example pacing        # how long hunts take, by class and level
cargo run --release -p reapers_rules --example rift_pacing
cargo test -p reapers_server                                 # needs PostgreSQL's binaries
cargo test -p reapers_app                                    # the screens, keepers and clients
(cd crates/reapers_program && pina build)                    # the program, for its tests
(cd crates/reapers_app && leptoskit dev --platform web)      # the game in a browser
cargo run -p reapers_server                                  # the server, on the PostgreSQL in passwords.toml
```

`cargo xtask check` needs `pina` 0.24.0 and Agave's `cargo-build-sbf`, the `solana` CLI the first time (to fetch Metaplex Core for the program's tests), and PostgreSQL's `initdb`, `pg_ctl` and `createdb` on `PATH` or in `REAPERS_PG_BIN`. The app's web build runs when the leptoskit CLI is installed, and is reported as skipped when it is not.

The app's build flags are environment variables of `leptoskit dev` and `leptoskit build`; the README lists them.

## Secrets

`crates/reapers_server/config/passwords.toml` is git-ignored and must never be committed. Create it from `passwords.example.toml`, which holds local placeholders only. A deployed server reads `REAPERS_SECRET_<name>` from its environment and refuses to start in staging or production without them. Deploy keys and the keeper's receipt secret are handled as `docs/deploy.md` says, outside the repository.

## Working with the server

The server's tests need no Docker and no running database: each test binary starts its own PostgreSQL cluster under `target/tmp` with `initdb` and `pg_ctl`, and removes it when it exits. To run the server itself, give it a PostgreSQL in `config/passwords.toml`; `crates/reapers_server/readme.md` shows how to keep one under the repository. It applies pending migrations when it starts.

Types from `reapers_rules` that cross the wire are serde types of the rules themselves, used by `reapers_api` as they are. Do not describe one a second time in the API.

## Checklist after a change

1. `cargo fmt --all`.
2. `cargo clippy --all-targets` in what you touched, with no warnings.
3. A migration, if a table changed.
4. `cargo xtask check`.
5. Look at the result running. Tests do not show a screen that looks wrong.
6. Update `docs/gameplay.md` if a rule or a number changed.
7. `cargo xtask merge` when the branch is ready for `main`.
