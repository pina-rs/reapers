# The Reapers program

The on-chain half of [the NFT design](nfts.md): a husk goes into escrow when it is released, and comes out only as ash, burned on the strength of a receipt the server's keeper signed. It is written with [pina](https://github.com/pina-rs/pina) and lives in `crates/reapers_program`.

| | |
| --- | --- |
| Program id | `HuskN1zeKqNaWUZbxwymyoW3S8CxMNneycZWDoXc98XP` |
| Keypair | `~/.config/solana/reapers/program-devnet-keypair.json`, outside the repository |
| Size | 49,352 bytes (`opt-level = 2`, fat LTO) |
| Rent | 0.2516 SOL on devnet for the program data account (49,397 bytes), and about as much again for the write buffer while deploying |
| Upgradeable | yes, by the upgrade authority |

How to deploy it is in [deploy.md](deploy.md).

## What it holds

Two kinds of account, both program-derived addresses. Every account the program writes starts with a discriminator byte and a migration version byte, which pina uses to evolve layouts (`crates/reapers_program/migrations/`).

**The configuration**, at `["config"]`, 68 bytes. There is one.

| Field | Meaning |
| --- | --- |
| `authority` | Rotates the keeper, pauses, and hands itself on. |
| `keeper` | The Ed25519 public key whose receipts `Absorb` accepts. |
| `paused` | While set, nothing can be released or absorbed. |
| `bump` | The address's bump. |

**An escrow**, at `["escrow", mint]`, 68 bytes (0.00136 SOL of rent, paid by the player and returned on absorbing), one per released husk. `mint` is the husk's address: the mint of a token NFT, or the asset itself for Metaplex Core. The simnet hashed its stand-in mint into the seed because the stand-in was a string; a real address is already 32 bytes and goes in as it is.

| Field | Meaning |
| --- | --- |
| `player` | The wallet that released the husk, and the only one that can absorb it. |
| `mint` | The husk. |
| `kind` | 1 a Core asset, 2 an SPL Token NFT, 3 a Token-2022 NFT. |
| `bump` | The address's bump. |

The escrow is what holds the husk: it is the Core asset's owner, or the owner of the token account that holds the NFT (its associated token account, the *vault*). The program has no instruction that gives a husk back.

## Instructions

Compute units are those the Mollusk suite measured for a whole transaction, cross-program calls included, but without the compute budget instructions. The ops tool asks for more (`crates/reapers_ops/src/program.rs`).

| # | Instruction | Signers | What it does | CU |
| --- | --- | --- | --- | --- |
| 0 | `Initialize { authority, keeper }` | upgrade authority | Creates the configuration. Only the program's upgrade authority can, so nobody can initialize a freshly deployed program before its owner does. | 6,112 |
| 1 | `SetKeeper { keeper }` | authority | Replaces the keeper. Receipts the old one signed stop working at once. | 328 |
| 2 | `SetAuthority` | authority and the new authority | Hands the administration on. Both sign, so a mistyped address cannot lock the owner out. | 361 |
| 3 | `SetPaused { paused }` | authority | Pauses or resumes releasing and absorbing. | 320 |
| 4 | `ReleaseCore` | player | Moves a Core asset the player owns into its escrow. | 11,037 |
| 5 | `ReleaseToken` | player | Moves a token NFT the player owns into its escrow's vault. | 22,033 (Token), 29,005 (Token-2022) |
| 6 | `AbsorbCore` | player | Burns an escrowed Core asset and closes its escrow. | 10,306 |
| 7 | `AbsorbToken` | player | Burns an escrowed token NFT, closes the vault and the escrow. | 3,930 (Token), 6,506 (Token-2022) |

pina also routes a reserved `Migrate` instruction, for moving accounts to a newer layout. It does nothing until a layout has a second version.

The accounts, in order:

| Instruction | Accounts |
| --- | --- |
| `Initialize` | upgrade authority (signer, writable), config (writable), program data, system program |
| `SetKeeper`, `SetPaused` | authority (signer), config (writable) |
| `SetAuthority` | authority (signer), new authority (signer), config (writable) |
| `ReleaseCore` | player (signer, writable), config, escrow (writable), asset (writable), collection, Core program, system program |
| `ReleaseToken` | player (signer, writable), config, escrow (writable), mint, player's token account (writable), vault (writable), token program, associated token program, system program |
| `AbsorbCore` | player (signer, writable), config, escrow (writable), asset (writable), collection (writable when it is one), Core program, instructions sysvar |
| `AbsorbToken` | player (signer, writable), config, escrow (writable), mint (writable), vault (writable), token program, instructions sysvar |

`collection` is the asset's Metaplex Core collection, or the Core program's own address when it has none, which is Core's convention for an absent account. Core checks that it is the right one.

The Rust client generated from the IDL (`crates/reapers_program_client`) builds all of these, and `reapers_program_client::absorb` builds the pair of instructions an absorption needs. It builds for `wasm32-unknown-unknown`.

### Events

Both are written to the transaction log as `Program data:` lines, decoded by the generated client.

| Event | Fields |
| --- | --- |
| `HuskReleased` | player, mint, kind |
| `HuskAbsorbed` | player, mint, grade, feeding, xp, anima, at, nonce: everything the receipt said |

An event proves nothing by itself: any program can log the same bytes. Attribute one to this program by its place in the logs, inside `Program HuskN1ze… invoke`, as the generated decoders do.

### Errors

| Code | Error | When |
| --- | --- | --- |
| 0 | `Paused` | Releasing or absorbing while paused. |
| 1 | `NotAuthority` | An administrative instruction not signed by the configured authority. |
| 2 | `NotUpgradeAuthority` | `Initialize` not signed by the upgrade authority, or the wrong program data account. |
| 3 | `NotAnNft` | Not a Core asset, or a token whose supply is not one or which has decimals. |
| 4 | `NotOwner` | The player does not own the husk (a delegate is not an owner). |
| 5 | `UnsupportedExtension` | A Token-2022 extension outside the allowed list. |
| 6 | `MissingReceipt` | The instruction before `Absorb` is not the Ed25519 program's. |
| 7 | `UnsupportedReceiptInstruction` | The Ed25519 instruction holds other than one signature, or points outside itself. |
| 8 | `WrongKeeper` | The receipt is signed by another key. |
| 9 | `MalformedReceipt` | The signed message is not a 98-byte version 2 receipt. |
| 10 | `WrongPlayer` | The receipt names another player, or the escrow belongs to another. |
| 11 | `WrongHusk` | The receipt names another husk. |
| 12 | `WrongKind` | `AbsorbCore` on a token escrow, or the other way round. |
| 13 | `EscrowEmpty` | The husk did not arrive in, or is no longer in, escrow. |
| 14 | `SupplyNotFixed` | The mint could still mint or freeze. |

## The receipt, version 2

`crates/reapers_receipt` is the one description of it, shared by the program, the server and the app. It has no dependencies and no `std`.

| Bytes | Field |
| --- | --- |
| 0..8 | the ASCII tag `RPRSABS2` |
| 8..40 | the player's wallet address |
| 40..72 | the husk's mint (token) or asset (Core) address |
| 72 | the reap grade: 0 ragged, 1 clean, 2 perfect |
| 73 | the feeding: 0 meagre, 1 fair, 2 rich, 3 feast |
| 74..78 | experience paid, unsigned, little-endian |
| 78..82 | anima paid, unsigned, little-endian |
| 82..90 | when, in seconds since 1970, signed, little-endian |
| 90..98 | the nonce the feeding was rolled from, little-endian |

It is version 1's layout (the first keeper's `AbsorptionReceipt`) with real addresses where version 1 had the SHA-256 of stand-ins. A version 1 receipt is refused by name. The crate's tests hold a fixed vector of the bytes and of the development keeper's signature over them; another signer must reproduce both.

An absorption is one transaction:

1. compute budget instructions, if any;
2. the Ed25519 program's instruction, verifying the keeper's signature over the receipt, with the public key, the signature and the message all inside its own data (`reapers_receipt::ed25519_instruction_data`);
3. `AbsorbCore` or `AbsorbToken`, immediately after it.

## Why it is safe

### Only the right people configure it

`Initialize` reads the program's own data account, which the loader owns, and requires its upgrade authority to sign. A program deployed and not yet initialized cannot be claimed by whoever sends the first transaction. Every other administrative instruction loads the configuration from its canonical address and compares the signer with the stored authority. Handing the authority on takes both keys.

### Only the owner releases, and only an NFT

- **Core.** The asset must be an account Core owns whose first byte is `AssetV1`, and its recorded owner must be the signer. Core would let a transfer delegate move it too; the program does not, so a delegate cannot release someone else's asset. After the transfer the program reads the owner again and requires it to be the escrow.
- **Tokens.** The mint must be owned by the token program passed, have a supply of exactly one and no decimals. The token account must hold that one token and be owned, not merely delegated, by the signer. After the transfer the vault must have received exactly one token and hold it alone.
- **Fixed supply.** The mint's mint authority and freeze authority must each be absent or be the mint's Token Metadata master edition, which is what a classic NFT has and which never mints more of its own mint. Otherwise a burned husk could be minted again, released again and absorbed again with its old receipt, or its vault frozen. Check a real collection's mints before supporting it: one whose freeze authority is still a wallet is refused.
- **Token-2022 extensions.** Only `MetadataPointer`, `TokenMetadata`, `GroupMemberPointer` and `TokenGroupMember` are allowed. Each describes the NFT. Everything else is refused because it can move the token out of escrow (`PermanentDelegate`), stop it moving or burning (`NonTransferable`, `DefaultAccountState`, `Pausable`, `PermissionedBurn`, `TransferHook`), take a cut (`TransferFeeConfig`), hide it (`ConfidentialTransferMint`), or let the mint be closed and made again at the same address (`MintCloseAuthority`).
- **The vault** is the escrow's associated token account, created idempotently. Anyone may create it first; if they do it is still the escrow's, because only the escrow could set a delegate or a close authority on it.
- **One escrow per husk.** The escrow's address is derived from the husk, and creating it fails if it exists.

### Only the keeper's receipt burns

The Ed25519 program checks signatures before anything else in the transaction runs, and fails the transaction if one is bad, so an Ed25519 instruction that is present has a good signature. What it does not check is *what* was signed, by *whom*, or *where* the bytes it verified came from. `Absorb` checks those, in `crates/reapers_program/src/receipt.rs`:

1. The instructions sysvar is the real one.
2. The instruction immediately before `Absorb` is the Ed25519 program's.
3. It verifies exactly one signature.
4. Its signature, public key and message offsets all point into its own data: each instruction index is `u16::MAX` ("this instruction") or its own position. This closes the known attack where the offsets point the Ed25519 program at a genuine signature somewhere else in the transaction while the bytes a careless program reads say something else. A test builds exactly that attack.
5. The public key is the configured keeper.
6. The message is 98 bytes and tagged `RPRSABS2`, with a grade and a feeding that exist.
7. The receipt's player is the signer, and the escrow's player.
8. The receipt's husk is the escrow's, and the escrow holds the kind of husk this instruction burns.

Then it burns: Core's `BurnV1` with the escrow signing as the asset's owner, or the token program's `Burn` and `CloseAccount` with the escrow signing as the vault's owner. The escrow is closed to the player, who gets back its rent, and the vault's.

### A receipt can be used once

A receipt names one husk. A husk has at most one escrow, and `Absorb` needs that escrow, belonging to that player. After an absorption:

- the escrow is closed, so the same receipt finds no escrow to absorb. A second `Absorb` in the same transaction fails the same way, and could not borrow the first one's Ed25519 instruction anyway, because it must be immediately before it;
- a Core asset is a one-byte tombstone (`Key::Uninitialized`) that Core will never turn into an asset again, so the husk cannot be released again;
- a token mint has a supply of zero, no mint authority but its master edition, which does not mint, and cannot be closed and remade, so the husk cannot come back either.

So there is never a second escrow for a receipt to match, and the program needs no record of the nonces it has seen. The tests try each of these: absorbing twice, absorbing twice in one transaction, releasing a burned husk, and releasing mints that could come back.

### What it does not protect against

- **The keeper key.** Whoever holds it can burn any escrowed husk with any receipt, including one that pays a player well. That is the trust the design places in the server ([nfts.md](nfts.md), decision 5). `SetKeeper` retires a leaked key at once; `SetPaused` stops everything while that happens.
- **Fair rolls.** The signature authenticates the server's stated feeding. It does not prove the server rolled it fairly.
- **The admin key.** The authority can pause the program and rotate the keeper. The upgrade authority can replace the program. Neither can take a husk out of escrow with the program as written, but a new version could.
- **Collection plugins on Core assets.** A Core collection with a permanent transfer, burn or freeze delegate lets that delegate move, burn or freeze an asset in escrow. The demo collections have no plugins. Check each real collection before supporting it.
- **Programmable NFTs.** Token Metadata pNFTs keep their token accounts frozen and move only through Token Metadata, so `ReleaseToken` fails for them. Supporting them needs Token Metadata's own transfer and burn.
- **Classic NFTs' metadata.** Burning with the token program leaves the Token Metadata accounts behind, with their rent. The NFT is gone (its supply is zero); reclaiming that rent needs Token Metadata's `BurnV1`.
- **Cross-cluster receipts.** A receipt does not name the cluster or the program, so the same keeper on devnet and mainnet would make devnet receipts valid on mainnet. Use a different keeper for each cluster. A version 3 receipt could add the program id.
- **Collections at all.** The program escrows any NFT a player releases; it does not know which collections the game supports. The server decides that, and signs receipts only for husks it supports. A husk released from another collection stays in escrow for good.

## How it is tested

| Where | What |
| --- | --- |
| `crates/reapers_receipt` | The layout, a fixed vector, the development keeper's signature, refusals of every malformed receipt, the Ed25519 instruction builder. |
| `crates/reapers_program/src/lib.rs` | Discriminators, program ids, and every shape of Ed25519 instruction data the reader must refuse. |
| `crates/reapers_program/tests/program.rs` | The built program in Mollusk with the real Metaplex Core, SPL Token, Token-2022 and associated token account programs and the Ed25519 precompile: every instruction's success, and every failure above, including the offset attack, two signatures, a tampered receipt, a rotated keeper, pausing, and replays. |
| `crates/reapers_program/tests/surfpool` | The built program on an isolated Surfnet through `pina test`: bootstrapping and administration. |
| `cargo run -p reapers_ops -- rehearse` | The whole runbook on a local Surfpool: deploy with the Solana CLI, initialize, mint the demo husks, release, absorb, and refuse a replay and a forged receipt. |

Run them as `crates/reapers_program/README.md` says. The Mollusk suite needs the built program and Metaplex Core (`cargo run -p reapers_ops -- fixtures`); it fails, rather than skipping, without them.

## Size and compute

`opt-level` was measured on the same source, with fat LTO, before the fixed-supply check was added:

| `opt-level` | Bytes | `ReleaseCore` CU | `AbsorbCore` CU |
| --- | --- | --- | --- |
| `"z"` | 50,824 | 12,332 | 11,479 |
| `"s"` | 54,752 | 11,232 | 10,483 |
| `3` | 50,088 | 10,820 | 10,117 |
| `2` | **48,232** | 11,037 | 10,306 |
| `1` | 93,960 | 11,645 | 10,975 |

Level 2 is both the smallest and within 2% of level 3's compute units, so the workspace builds the program at level 2 (`[profile.release.package.reapers_program]` in the root `Cargo.toml`). `"z"` is larger and slower. The program is 49,352 bytes with the fixed-supply check. No build reported a stack frame over the 4 KB limit.

## Candidates for pina

Two pieces of this program are general and would be better in pina:

- **Ed25519 introspection** (`src/receipt.rs`): finding the Ed25519 instruction before the current one and returning its single, self-contained message and key, refusing offsets into other instructions. `pina::introspection` has the sysvar plumbing but not this.
- **Metaplex Core `TransferV1` and `BurnV1`** (`src/mpl_core.rs`): two fixed-shape CPIs and an `AssetV1` owner read. A generated CPI crate for all of Core would be much larger than these.
