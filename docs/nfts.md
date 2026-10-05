# NFTs: what is simulated today, and where the real thing goes

Reaper ownership now connects to Solana through Solana Kit: Android wallet proof, real balances and verified Metaplex Core imports. Each NFT reaper has canonical off-chain progression that follows its asset when traded. Buying happens in a configured external marketplace and is followed by an ownership check; the game does not invent listings or hold keys. See [solana.md](solana.md).

The hunted husks are still stand-ins. Their collections, prices, releases, receipts and burns below remain the prototype simulation; no escrow or burn program is deployed.

## The collections are real. Everything else about them is simulated.

The game supports five collections from Solana's first wave, each old and long since quiet:

| Collection | Launched | Minted | Creature |
| --- | --- | --- | --- |
| Degenerate Ape Academy | August 2021 | 10,000 | Ape |
| Thugbirdz | August 2021 | 3,333 | Bird |
| Pesky Penguins | October 2021 | 8,888 | Penguin |
| Sollamas | August 2021 | 8,888 planned | Llama |
| Solarians | April 2021 | 10,000 | Robot |

They were picked from a researched list of 116: see [research/solana-collections.md](research/solana-collections.md), which also says how well each figure above is sourced.

What that means, and does not:

- **We use their names.** Nothing else of theirs is in the game: no image, no logo, no metadata, no named trait.
- **The creatures are our own models**, built from scratch by the scripts in `art/blender/creature_kinds/`. Each follows the spirit of its collection (a hunched bird in a visor, a llama under a striped blanket) and copies none of its artwork.
- **An NFT's picture in the game is a render of our model** in that NFT's colours.
- **The eight NFTs of each collection are invented.** They are numbered 1 to 8 and correspond to no real token.
- **Nothing is licensed.** The research found no collection under an open licence. Using a collection's name in a private prototype is one thing; shipping it needs the agreement of whoever holds the rights, and for several of these that is a holders' DAO. The vault screen says in words that this is a simulation.
- **Avoid list.** The research flags collections not to touch: copies of other people's work, ones tied to brands or real people, and ones that ended in fraud. Catalina Whale Mixer and Galactic Geckos were first choices here and were dropped on that evidence.

## What is a stand-in

| In the game | What it is today | Where it lives |
| --- | --- | --- |
| Collections | The five above, eight invented items each. | `NftCollection` in `reapers_shared/lib/src/nft_catalogue.dart` |
| An NFT's look | Three colours, an eye colour and one worn thing, drawn from a palette for its collection. | `NftLook` in `reapers_shared/lib/src/nft_look.dart` |
| NFT pictures | 40 portraits rendered by Blender from our own models. | `art/blender/husks.py`, `reapers_flutter/assets/nfts/` |
| Mint addresses | A string with the shape of a Solana address, derived from the item's id. It belongs to no account. | `Nft.mockMint` |
| The market | Every item of every collection is always for sale at a price fixed by its rarity. | `marketListings`, `Rarity.lamports` |
| The wallet | A number. Every player starts with 3 SOL of it, which is enough to buy the whole market. | `PlayerState.lamports` |
| Buying | Subtracts the price and adds the item to the vault. | `PlayerState.acquire` |
| Releasing | Moves the item from the vault into the veil as a husk. | `PlayerState.release` |
| Absorbing | Removes the husk, rolls the feeding and writes the codex. | `PlayerState.settleHunt` |
| The receipt | A real signature, by a development key, that is posted nowhere. | `AbsorptionReceipt` in `reapers_server` |
| Anima | A plain counter. It is not a token and is not meant to become one. | `PlayerState.anima` |

## The design the stand-ins assume

These are the decisions the simulation already bakes in. They are worth agreeing before any chain work starts, because they are the product, not the plumbing.

1. **Absorbing burns.** The NFT is destroyed, not transferred. That is what makes the game a sink for dead collections rather than another place to flip them: every hunt removes supply.
2. **The player owns the NFT before it is hunted.** Buying and releasing are separate so that a real purchase (which can fail, or take time) never happens in the middle of a hunt.
3. **Releasing is the point of no return.** Once released, the NFT can only be hunted. In the real version this is where it should move into escrow, so a player cannot release an NFT, start the hunt and sell it meanwhile.
4. **A supported collection has a creature of its own.** Supporting a collection means modelling its animal, so it is a deliberate act and not something any collection gets for free.
5. **Luck is decided off-chain and signed.** How well a reaper feeds is rolled by our server. Its signature authenticates the stated result; it does not prove the operator sampled fairly. Publicly verifiable randomness is not implemented. See the next section and [the decision record](decisions.md).
6. **Anima and experience stay off-chain.** Anima belongs to the account and cannot be traded. Experience is earned by play and cannot be purchased directly; an NFT reaper's earned growth, codex and equipment follow the asset when its ownership changes. This changes the earlier single-reaper design to support trading grown reapers.
7. **Rifts drive demand.** A collection's rift only opens to someone who has absorbed one of that collection, so a rift is a reason to buy the floor.

If any of these is wrong, the change is in `PlayerState`, not in the chain code.

## Feeding, and the signed receipt

When a reaper absorbs an NFT, the server decides three things the player cannot: the verdict of the hunt (by replaying it), how well the reaper fed (a weighted roll from the server's secure random numbers), and when it happened. It then signs all of it.

- **The roll** is `Feeding.roll(seed)` in the shared rules: meagre ×0.85, fair ×1, rich ×1.25, a feast ×2, at 25, 50, 20 and 5 in a hundred. It scales experience only. The seed is the server's and is recorded as the receipt's nonce, so anyone can check that the feeding follows from the nonce. This does not prove an unbiased draw: an operator could choose or withhold a seed. A public randomness or commitment protocol remains future work.
- **The message** is 98 bytes in a fixed layout, documented on `AbsorptionReceipt`: a tag and version, the player, the NFT's mint, the grade, the feeding, the experience and anima paid, the time and the nonce.
- **The signature** is Ed25519, the scheme Solana itself uses. A Solana program can check it with the built-in Ed25519 program, so a transaction can carry the server's receipt and the program can trust the roll without doing any randomness of its own.
- **The key** is derived from one secret, `receiptSigningKey`. Development and tests run without it and sign with a key anyone can derive, so their receipts prove nothing. A deployed server refuses to start without the real secret. The public key is served by `player.receiptKey`.
- **The receipt is kept** with the absorption in the player's codex, so it can be posted, or posted again, later.
- **Offline play signs nothing.** The phone rolls its own feeding and the codex entry carries no signature. An offline absorption could never be posted on-chain, which is right.

What is not built: anything that posts a receipt. The intended shape is that the same transaction that burns the escrowed NFT carries the receipt, and the program refuses the burn unless the signature checks, the mint and player match, and the nonce has not been seen.

Two stand-ins inside the message itself: until players have wallets and NFTs have real mints, the two 32-byte address fields hold the SHA-256 of the stand-in identifiers. When the real addresses exist they go in as they are and the tag's version goes up.

## Where the real thing plugs in

The game talks to one interface, `GameBackend` (`reapers_flutter/lib/src/backend/game_backend.dart`), and the server applies the same shared rules. The chain work lands in four places:

1. **Wallet.** Sign in with a wallet (Mobile Wallet Adapter on Android, a deep-link wallet on iOS) and link it to the guest account the server already creates on first launch. `PlayerState.lamports` goes away: the wallet's balance is read, never stored.
2. **Catalogue.** `NftCollection` becomes data instead of an enum: a list of supported collections (verified collection address, creature, rarity mapping) kept by the server. An NFT's look is then read from its real traits instead of drawn from a palette: `NftLook.of` is the one place that changes.
3. **Acquire.** Today the server just debits the mock wallet. For real it either lists floor items through a marketplace API and hands the wallet a transaction to sign, or simply reads the wallet's holdings and lets the player bring NFTs they already own. The second is far less work and has no custody risk; start there.
4. **Release and absorb.** Release becomes a transfer into a program-owned escrow. A hunt the server has replayed and judged absorbed produces a signed receipt, and the burn transaction carries it; a husk that escapes stays in escrow, still huntable. This is the one piece that needs an on-chain program, and the only thing it should trust is the server's receipt key.

The server already replays every lone hunt from its recorded inputs before it pays (`PlayerState.judgeHunt`), which is the property a burn needs: the chain acts on the server's verdict, never on the phone's claim.

The implemented Dart stack uses `solana_kit`, its typed RPC API and Metaplex Core codecs, plus its Mobile Wallet Adapter packages. The wallet and reaper-ownership seams above are implemented; real husk acquisition, escrow and burning remain work for the future. Wallet connection on iOS requires a separate adapter before it is offered there.

## What has to be true before real money

The server now refuses simulated purchases, releases, hunt settlements and rifts in
staging and production. Development and test retain the prototype economy.
This prevents invented mints and the mock wallet from granting live NFT growth;
it does not implement a burn. See [launch-readiness.md](launch-readiness.md).

Reaper NFTs carry their earned soul memories as well as XP, equipment and
codex in the canonical server record. A transferred reaper retains its birth
gift and temperament, which are derived from its permanent R1 genome.

- Rift results are trusted, not replayed. NFT reapers are refused entry until verification is built; the non-tradable starter can enter in development and test.
- Rifts are free to enter and have no cooldown.
- The simulated prices are invented. Real floors move.
- Supported collections need the consent of whoever holds their rights. Burning someone's art for sport is a decision to make with them, and so is using their name.
- The receipt key becomes the most valuable secret in the system: whoever holds it can sign any absorption. It wants a hardware-backed key and rotation before launch.
- Real-money mechanics need legal review in each market before launch, and app store rules on NFTs differ between Apple and Google.
