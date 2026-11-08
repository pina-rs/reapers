# reapers_server

The Reapers keeper: the server. It keeps each player's books in PostgreSQL, replays every hunt with `reapers_rules` before it pays, hosts rifts, proves wallets, checks NFT reapers on the chain, and signs the absorption receipts the Reapers program burns husks with.

It is built on leptoskit's server crates: `leptoskit_rpc` for the operations and the live socket, `leptoskit_db` for the tables, `leptoskit_auth` for accounts and sessions, and `leptoskit_axum` for serving. Every operation is declared once, in `crates/reapers_api`, which the game builds for Wasm too. The tables and their migrations are `crates/reapers_db`.

## Running it locally

You need PostgreSQL (the tests run against 17). Nothing else: no Docker.

**1. A database.** Any PostgreSQL you can reach works. To keep one under the repository for this purpose:

```sh
initdb -D .postgres -U postgres --auth=trust --no-locale -E UTF8
pg_ctl -D .postgres -l .postgres/log -o "-p 5432 -c listen_addresses=127.0.0.1" start
createdb -h 127.0.0.1 -U postgres reapers
```

On a Nix profile, run the tools from the PostgreSQL store path (`$(dirname $(readlink -f $(which initdb)))/initdb`), so `initdb` finds the files it was built with.

**2. Secrets.** Copy the example and point `database` at that database:

```sh
cp crates/reapers_server/config/passwords.example.toml crates/reapers_server/config/passwords.toml
```

`config/passwords.toml` is git-ignored. In development and test the keeper reads it, then lets `REAPERS_SECRET_<name>` in the environment override any value. Without one, a development keeper uses `postgres://localhost:5432/reapers`.

**3. Run it.**

```sh
cargo run -p reapers_server
```

It applies any pending migrations, says which key it signs receipts with, and listens on `http://127.0.0.1:8080`. `/readyz` passes once the migrations are applied and the live socket is up.

Try it:

```sh
curl -X POST http://127.0.0.1:8080/auth/guest
curl -X POST -H "authorization: Bearer <access_token>" -H "content-type: application/json" \
  -d '{}' http://127.0.0.1:8080/api/take_oath
```

## Settings

| Variable                      | Default                         | What it is                                                                                                             |
| ----------------------------- | ------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `REAPERS_RUN_MODE`            | `development`                   | `development`, `test`, `staging` or `production`. Only development and test run the simulated economy.                 |
| `REAPERS_LISTEN`              | `127.0.0.1:8080`                | Where to listen.                                                                                                       |
| `REAPERS_ORIGIN`              | none                            | The web game's origin: the live socket checks browser pages against it, and the routes answer it across origins (CORS). Native apps send none. |
| `REAPERS_SOLANA_CHAIN`        | `solana:devnet`                 | Or `solana:mainnet`, or `solana:localnet` (a validator on this machine, development and test only).                    |
| `REAPERS_SOLANA_RPC_URL`      | the chain's public RPC          | HTTPS. Its genesis must match the chain, checked on every read. A localnet's is `http://127.0.0.1:8899` by default, on this machine, and has no genesis check. |
| `REAPERS_DEMO_HUSKS`          | none                            | The path of the manifest `reapers_ops mint-demo --out` writes. `chain_status` lists its husks for the game, and a release with an escrow must escrow the asset its husk stands for. |
| `REAPERS_WALLET_IDENTITY_URI` | `https://reapers.invalid/`      | The HTTPS identity origin, with a trailing slash. The placeholder turns wallets off. Required with a collection.        |
| `REAPERS_REAPER_COLLECTION`   | none                            | The verified Metaplex Core collection NFT reapers come from. Without it, imports are refused.                          |
| `REAPERS_REAPER_MARKET_URL`   | none                            | HTTPS page where players buy and trade the collection.                                                                 |
| `REAPERS_PROGRAM_ID`          | none                            | The Reapers program husks are escrowed with and burned by. Without it, escrowed releases are refused.                  |

The secrets, as `REAPERS_SECRET_<name>`:

| Secret              | What it is                                                                                                                                                                 |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `database`          | The PostgreSQL URL, password included.                                                                                                                                     |
| `authKeys`          | Sign-in's keys, `id:key` with 32 bytes of base64url, comma-separated, newest first. A development keeper derives one from a public name when it is not set.               |
| `receiptSigningKey` | What the keeper's Ed25519 key is derived from (its SHA-256 is the seed). Development and test sign with a key anyone can derive from `reapers-development-receipt-key`, and say so. |

A staging or production keeper refuses to start without all three, naming what is missing, and exits with code 78. There is no flag to get past it, and none to turn the simulated economy on in a deployment.

## Migrations

The tables are `#[table]` structs in `crates/reapers_db/src/lib.rs`, with leptoskit_auth's and leptoskit_rpc's own tables beside them. Each migration is a directory under `crates/reapers_db/migrations/`, listed in order in `registry.txt`, with the SQL that moves the database forward, the SQL that moves it back, and the schema it leaves. The keeper applies pending ones when it starts.

After changing a table:

```sh
cargo run -p reapers_db --bin reapers-migration -- <name>   # writes the next migration
cargo run -p reapers_db --bin reapers-migration -- --check  # fails when a table changed without one
```

This does what `leptoskit db migration create` and `leptoskit db check` do, without the leptoskit CLI. Run it as written, on its own: built together with the keeper, Solana Kit turns on serde_json's `arbitrary_precision`, and leptoskit_db_schema cannot read its snapshots back under it. `tests/process.rs` runs the check in every test run, comparing the snapshot as text so it holds either way.

## Tests

```sh
cargo test -p reapers_server
```

The tests need PostgreSQL's `initdb`, `pg_ctl` and `createdb` on `PATH`, or in `REAPERS_PG_BIN`. Each test binary starts its own cluster under `target/tmp`, migrates a template database once, and gives every test a fresh copy, so tests run side by side. A watchdog stops the cluster and deletes it when the binary exits. The chain is a fake RPC transport each test controls, answering through Solana Kit's real client and codecs.

| File              | What it covers                                                                                                          |
| ----------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `tests/player.rs` | Books, location, replays, receipts, the armory, concurrency, and the simulated economy's gate |
| `tests/solana.rs` | Wallet proof, replay and expiry, restoring an account, ownership transfer; and escrows, and building a burn that did not land again |
| `tests/chain.rs`  | Core decoding, the cluster check, genome URIs, failures; and escrow accounts            |
| `tests/rift.rs`   | One reaper, two at once, demo circles, standings                                         |
| `tests/http.rs`   | The router: guest and wallet sign-in, bearer calls, leptoskit_auth refreshing guest sessions, refusals on the wire     |
| `tests/process.rs`| The binary refusing to start without its secrets, and the migrations matching the tables                               |
