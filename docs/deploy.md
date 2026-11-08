# Deploying the program to devnet

The runbook for putting the Reapers program (`HuskN1zeKqNaWUZbxwymyoW3S8CxMNneycZWDoXc98XP`) on devnet, configuring it, minting the demo husks and proving it works. What the program does is in [program.md](program.md).

Every command here has been run, in this order, against a local Surfpool: `reapers_ops rehearse` does it all in one go (step 1). Run the rehearsal first, every time.

Commands run from the repository root, in bash or zsh.

## What you need

| Tool | Version used | For |
| --- | --- | --- |
| Agave Solana CLI (`solana`, `solana-keygen`, `cargo-build-sbf`) | 4.2.2, platform tools v1.54 | Building, deploying, inspecting |
| `pina` | 0.24.0 (`cargo install pina_cli --version 0.24.0 --locked`) | Building the program and its IDL |
| `surfpool` | 1.7.0 | The rehearsal |
| Rust | the nightly in `rust-toolchain.toml` | `reapers_ops` |

Keys, none of which ever go in the repository:

| Key | Where | What it is |
| --- | --- | --- |
| Program keypair | `~/.config/solana/reapers/program-devnet-keypair.json` | The program's address. Needed for the first deployment only. Back it up: without it the address cannot be deployed to again from scratch. |
| Deployer | `~/.config/solana/reapers/devnet-deployer.json`, made in step 2 | Pays, and is the upgrade authority and the program's authority until you hand them on. |
| Keeper secret | the server's `REAPERS_SECRET_receiptSigningKey` | The secret the server derives its receipt key from. Use one for devnet only, never the mainnet one: a receipt does not name its cluster. |

Set these once per shell:

```sh
export KEYS=~/.config/solana/reapers
export PROGRAM_KEYPAIR=$KEYS/program-devnet-keypair.json
export DEPLOYER=$KEYS/devnet-deployer.json
export PROGRAM=HuskN1zeKqNaWUZbxwymyoW3S8CxMNneycZWDoXc98XP
ops() { cargo run -q --release -p reapers_ops -- "$@"; }
```

`ops` is the Rust tool in `crates/reapers_ops`. `ops --help` lists its commands. Every command that writes takes `--url`, and none reaches mainnet without `--allow-mainnet`.

## 1. Build and rehearse

```sh
git status                                  # build from a clean, committed tree
(cd crates/reapers_program && pina build)   # target/deploy/reapers_program.so
shasum -a 256 target/deploy/reapers_program.so | tee target/deploy/reapers_program.sha256
ls -l target/deploy/reapers_program.so      # 49,352 bytes at the time of writing

ops fixtures                                # Metaplex Core, for the tests: target/fixtures/mpl_core.so
cargo test -p reapers_receipt -p reapers_program -p reapers_program_client -p reapers_ops
(cd crates/reapers_program && pina test)    # the program on an isolated Surfnet
ops rehearse                                # the whole runbook on a local Surfpool
```

`ops rehearse` starts an offline Surfpool on port 18899, loads Metaplex Core into it, deploys with the Solana CLI exactly as step 3 does, then runs `status`, `initialize`, `mint-demo`, `smoke` and `set-paused` as below, and tries a replayed and a forged receipt. It must end with `==> rehearsal passed`. Its keys and the minted addresses are left in `target/rehearsal/`.

Do not go on if any of these fail.

## 2. Make and fund the deployer

```sh
solana-keygen new --outfile $DEPLOYER       # write down the seed phrase, offline
chmod 600 $DEPLOYER
solana address -k $DEPLOYER
solana airdrop 2 -k $DEPLOYER -u devnet     # rate limited; or https://faucet.solana.com
solana balance -k $DEPLOYER -u devnet
```

Have at least 2 SOL. The program data account costs 0.2516 SOL of rent, the deploy's write buffer holds about as much again until the deploy finishes, and the 40 demo husks cost about 0.15 SOL with Core's fees. The rest is margin for priority fees and a second attempt.

## 3. Deploy

```sh
solana program deploy \
  --url devnet \
  --keypair $DEPLOYER \
  --program-id $PROGRAM_KEYPAIR \
  --upgrade-authority $DEPLOYER \
  --with-compute-unit-price 50000 \
  --max-sign-attempts 20 \
  --use-rpc \
  target/deploy/reapers_program.so
```

It prints `Program Id: HuskN1zeKqNaWUZbxwymyoW3S8CxMNneycZWDoXc98XP` and a signature.

- `--with-compute-unit-price` is the priority fee, in micro-lamports per compute unit. Devnet rarely needs it; 50,000 costs well under 0.01 SOL for the roughly fifty write transactions. Raise it if writes keep expiring.
- `--max-sign-attempts` lets the CLI re-sign writes whose blockhash expired.
- `--use-rpc` sends through the RPC node instead of straight to validators, which is what works from behind a NAT.

### If the deploy stops half way

The CLI writes the program into a buffer account first, and leaves the buffer behind if it fails. It prints a 12-word seed phrase for the buffer's keypair. Either resume, or close the buffer and start again.

To resume, writing only what is missing:

```sh
solana-keygen recover --outfile target/deploy/buffer.json   # paste the 12 words
solana program deploy --url devnet --keypair $DEPLOYER \
  --program-id $PROGRAM_KEYPAIR --upgrade-authority $DEPLOYER \
  --buffer target/deploy/buffer.json \
  --with-compute-unit-price 50000 --max-sign-attempts 20 --use-rpc \
  target/deploy/reapers_program.so
```

To close it and get its rent back:

```sh
solana program show --buffers --url devnet --keypair $DEPLOYER
solana program close <BUFFER_ADDRESS> --url devnet --keypair $DEPLOYER --bypass-warning
# or every buffer this key owns:
solana program close --buffers --url devnet --keypair $DEPLOYER
```

## 4. Check what is deployed

```sh
solana program show $PROGRAM --url devnet --keypair $DEPLOYER
solana program dump $PROGRAM target/deploy/deployed.so --url devnet
shasum -a 256 target/deploy/deployed.so target/deploy/reapers_program.so
ops status --url devnet
```

`show` must name the deployer as `Authority` and a data length of 49,352 bytes. The two hashes must be identical: the cluster runs exactly the file you built. `status` must say the program is executable and the config is not initialized.

The explorer: <https://explorer.solana.com/address/HuskN1zeKqNaWUZbxwymyoW3S8CxMNneycZWDoXc98XP?cluster=devnet>.

## 5. Initialize, with the keeper

The program trusts receipts from one Ed25519 key, the keeper. The server derives it from its secret: the key's seed is the secret's SHA-256 (`ReceiptSigner` in `crates/reapers_server/src/signer.rs`, `keeper_from_secret` in `reapers_ops`). Make a devnet secret, give it to the devnet server as `REAPERS_SECRET_receiptSigningKey`, and derive the public key from it:

```sh
export REAPERS_SECRET_receiptSigningKey="$(openssl rand -base64 32)"   # keep it where the server's secrets live
export KEEPER=$(ops keeper)
echo $KEEPER
```

`ops keeper` reads the secret from the environment and prints only the public key.

Initialize. Only the upgrade authority can, so this has to come before the upgrade authority is handed on (step 9):

```sh
ops initialize --url devnet --upgrade-authority $DEPLOYER --keeper $KEEPER
ops status --url devnet
```

Add `--authority <ADDRESS>` to make someone other than the deployer the program's authority. `status` must show the authority, the keeper you derived, and `paused false`.

## 6. Mint the demo husks

Five Metaplex Core collections, one for each supported collection in [nfts.md](nfts.md), with eight assets each, numbered 1 to 8. The deployer is each collection's update authority; `--owner` receives the husks.

```sh
ops mint-demo --url devnet --payer $DEPLOYER \
  --owner <WALLET_THAT_WILL_PLAY> \
  --uri-base https://reapers.invalid/husks/ \
  --out target/deploy/demo-husks-devnet.json
```

It prints each collection and asset as it goes and writes their addresses to the `--out` file, which the server's catalogue will need. Keep the file. The metadata URIs are `<uri-base><slug>/collection.json` and `<uri-base><slug>/<n>.json`, with the slugs `degenerate-ape-academy`, `thugbirdz`, `pesky-penguins`, `sollamas` and `solarians`. They point at nothing yet: host placeholder JSON there later, or pass the real base URL now if it exists. Collection and asset names say "Reapers demo" and "husk"; they are stand-ins on a test network and copy nothing of the real collections.

If it stops part way, run it again: it makes new collections rather than resuming, and the earlier ones stay on devnet, unused.

## 7. Smoke test

Release one demo husk and absorb it with a receipt signed by the real keeper, on devnet. It burns that husk. The player must own it and hold a little SOL; with `--owner` left as the deployer in step 6, the deployer is the player.

```sh
ASSET=$(jq -r '.collections[] | select(.slug=="thugbirdz") | .assets[0]' target/deploy/demo-husks-devnet.json)
COLLECTION=$(jq -r '.collections[] | select(.slug=="thugbirdz") | .address' target/deploy/demo-husks-devnet.json)
ops smoke --url devnet --player $DEPLOYER --asset $ASSET --collection $COLLECTION
```

It checks first that the keeper derived from `REAPERS_SECRET_receiptSigningKey` is the one the program trusts. It must end with `==> smoke test passed`, after saying the asset is a one-byte tombstone and the escrow is closed. Look the asset up in the explorer: it is owned by Core and holds one byte.

The server does not yet sign version 2 receipts or post them; this test signs from your machine with the same secret. Clear it from the shell afterwards:

```sh
unset REAPERS_SECRET_receiptSigningKey
```

## 8. Upgrading

Build, rehearse (step 1), then deploy again with the same command as step 3. With an existing program it writes a new buffer and swaps it in; the program id does not change, and the program keypair is not needed, only the upgrade authority:

```sh
(cd crates/reapers_program && pina build)
ops rehearse
solana program deploy --url devnet --keypair $DEPLOYER \
  --program-id $PROGRAM --upgrade-authority $DEPLOYER \
  --with-compute-unit-price 50000 --max-sign-attempts 20 --use-rpc \
  target/deploy/reapers_program.so
```

Then check it as in step 4. Or in two steps, so the write and the switch are separate:

```sh
solana program write-buffer --url devnet --keypair $DEPLOYER \
  --with-compute-unit-price 50000 --max-sign-attempts 20 --use-rpc \
  target/deploy/reapers_program.so                     # prints Buffer: <BUFFER_ADDRESS>
solana program upgrade <BUFFER_ADDRESS> $PROGRAM --url devnet --keypair $DEPLOYER \
  --upgrade-authority $DEPLOYER
```

A larger program needs a larger data account; the CLI extends it automatically unless told not to.

An upgrade that changes an account, instruction or event layout must go through `pina migrations create` first; `pina migrations check` must pass before building. `pina rehearse --network devnet --build` replays the deployed program's recent transactions against the new build before you upgrade.

## 9. The upgrade authority

Whoever holds it can replace the program. On devnet the deployer is fine. Before anything holds value, move it to a key that lives somewhere safer, or a multisig.

```sh
# to a key you hold, which must sign:
solana program set-upgrade-authority $PROGRAM --url devnet --keypair $DEPLOYER \
  --upgrade-authority $DEPLOYER --new-upgrade-authority <NEW_AUTHORITY_KEYPAIR>

# to an address that cannot sign here, such as a multisig vault:
solana program set-upgrade-authority $PROGRAM --url devnet --keypair $DEPLOYER \
  --upgrade-authority $DEPLOYER --new-upgrade-authority <ADDRESS> \
  --skip-new-upgrade-authority-signer-check

solana program show $PROGRAM --url devnet --keypair $DEPLOYER   # Authority: the new one
```

Check the address twice before using `--skip-new-upgrade-authority-signer-check`: a wrong one cannot be undone. Never pass `--final` to a deploy or to `set-upgrade-authority`; it makes the program immutable.

## 10. Running it

| To | Run |
| --- | --- |
| See the configuration | `ops status --url devnet` |
| Stop releases and absorptions at once | `ops set-paused --url devnet --authority $DEPLOYER --paused true` |
| Resume | `ops set-paused --url devnet --authority $DEPLOYER --paused false` |
| Rotate the keeper (a new server secret) | `ops set-keeper --url devnet --authority $DEPLOYER --keeper <NEW_KEEPER>` |
| Hand the program's authority on | `ops set-authority --url devnet --authority $DEPLOYER --new-authority <NEW_KEYPAIR>` |

Rotating the keeper makes every receipt the old key signed useless at once, including ones a player has not yet posted: the server must sign those again with the new key. If the keeper secret leaks, pause first, rotate, then resume.

The program's authority (in the config) and the upgrade authority (in the loader) are different keys that start out the same. Handing one on does not hand on the other. `set-authority` needs both the old and the new key to sign.

## When something goes wrong

| Symptom | Cause |
| --- | --- |
| `Error: Account ... has insufficient funds` | The deployer needs more SOL (step 2). |
| Writes keep expiring, `max sign attempts reached` | Congestion: raise `--with-compute-unit-price`, then resume from the buffer (step 3). |
| `custom program error: 0x2` on `initialize` | Not signed by the upgrade authority. Check `solana program show`. |
| `Account already initialized` on `initialize` | It is already initialized. `ops status` shows by whom. |
| `custom program error: 0x8` on absorbing | The receipt's key is not the configured keeper: check `ops keeper` against `ops status`. |
| `custom program error: 0x0` | The program is paused. |
| `Invalid account owner` on absorbing | No escrow for that husk: it was never released, or it is already burned. |

The full list of error codes is in [program.md](program.md#errors). `pina explain <SIGNATURE> --network devnet`, run in `crates/reapers_program`, names the check that failed.
