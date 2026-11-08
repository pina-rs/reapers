# reapers_program

The Reapers on-chain program, built with [pina](https://github.com/pina-rs/pina). Program id `HuskN1zeKqNaWUZbxwymyoW3S8CxMNneycZWDoXc98XP`. What it does and why it is safe is in [docs/program.md](../../docs/program.md); how to deploy it is in [docs/deploy.md](../../docs/deploy.md).

## Commands

Run them in this directory. `pina` is `cargo install pina_cli --version 0.24.0 --locked`; `cargo-build-sbf` comes with the Agave CLI.

| Command | What it does |
| --- | --- |
| `pina build` | The SBF program at `target/deploy/reapers_program.so`, and the IDL. |
| `cargo test -p reapers_program` | Unit tests, and every instruction and failure path on the built program in Mollusk. Needs `pina build` first and Metaplex Core at `target/fixtures/mpl_core.so` (`cargo run -p reapers_ops -- fixtures`). |
| `pina test` | The built program on an isolated, offline Surfnet (`tests/surfpool`). |
| `RUSTUP_TOOLCHAIN=nightly-2026-09-15 pina lint` | Pina's security lints. Its prebuilt driver exists only for that nightly. |
| `pina migrations check` | The account, instruction and event layouts still match `migrations/manifest.json`. |
| `pina generate` | The IDL in `idl/` and the Rust client's sources in `../reapers_program_client/reapers_program/src/generated`. |

Everything under `migrations/`, `idl/`, `tests/abi_layout.rs` and the client's `generated` directory is written by pina. Never edit it by hand.
