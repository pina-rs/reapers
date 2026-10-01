# NFTs: what is fake today, and where the real thing goes

Nothing in this version touches a chain, a wallet or a marketplace. That was the brief: get the hunt feeling right first. This page lists every stand-in and the seam it sits behind, so wiring Solana in later is a matter of replacing these pieces rather than finding them.

## What is a stand-in

| In the game | What it is today | Where it lives |
| --- | --- | --- |
| Collections | Four invented ones: Tin Saints, Mothlight Parade, Static Choir, Brine Hounds. Eight items each. No real collection's art or name is used. | `NftCollection` in `reapers_shared/lib/src/nft_catalogue.dart` |
| NFT images | 32 PNGs drawn by a script and bundled with the app. | `tool/build_nft_art.dart`, `reapers_flutter/assets/nfts/` |
| Mint addresses | A string with the shape of a Solana address, derived from the item's id. It belongs to no account. | `Nft.mockMint` |
| The market | Every item of every collection is always for sale at a price fixed by its rarity. | `marketListings`, `Rarity.lamports` |
| The wallet | A number. Every player starts with 2.5 SOL of it, which is enough to buy the whole market. | `PlayerState.lamports` |
| Buying | Subtracts the price and adds the item to the vault. | `PlayerState.acquire` |
| Releasing | Moves the item from the vault into the veil as a husk. | `PlayerState.release` |
| Absorbing | Removes the husk and writes the codex. | `PlayerState.settleHunt` |
| Anima | A plain counter. It is not a token and is not meant to become one. | `PlayerState.anima` |

The vault screen says in words that the market is a stand-in, so nobody playing this build is misled.

## The design the stand-ins assume

These are the decisions the mock already bakes in. They are worth agreeing before any chain work starts, because they are the product, not the plumbing.

1. **Absorbing burns.** The NFT is destroyed, not transferred. That is what makes the game a sink for dead collections rather than another place to flip them: every hunt removes supply.
2. **The player owns the NFT before it is hunted.** Buying and releasing are separate so that a real purchase (which can fail, or take time) never happens in the middle of a hunt.
3. **Releasing is the point of no return.** Once released, the NFT can only be hunted. In the real version this is where it should move into escrow, so a player cannot release an NFT, start the hunt and sell it meanwhile.
4. **A husk wears the NFT's own image on an original body.** The vessel is ours; the picture is theirs. Any square image works, so any collection can be supported without modelling it.
5. **Anima and experience stay off-chain.** They are earned by play and cannot be bought or traded. Putting them on-chain would turn the game into a farm.
6. **Rifts drive demand.** A collection's rift only opens to someone who has absorbed one of that collection, so a rift is a reason to buy the floor.

If any of these is wrong, the change is in `PlayerState`, not in the chain code.

## Where the real thing plugs in

The game talks to one interface, `GameBackend` (`reapers_flutter/lib/src/backend/game_backend.dart`), and the server applies the same shared rules. The chain work lands in four places:

1. **Wallet.** Sign in with a wallet (Mobile Wallet Adapter on Android, a deep-link wallet on iOS) and link it to the guest account the server already creates on first launch. `PlayerState.lamports` goes away: the wallet's balance is read, never stored.
2. **Catalogue.** `NftCollection` becomes data instead of an enum: a list of supported collections (verified collection address, vessel, rarity mapping) kept by the server. `nftImageAsset` becomes a fetch of the item's image URI, cached on the device. `HuskRig` already takes any decoded image.
3. **Acquire.** Today the server just debits the mock wallet. For real it either lists floor items through a marketplace API and hands the wallet a transaction to sign, or simply reads the wallet's holdings and lets the player bring NFTs they already own. The second is far less work and has no custody risk; start there.
4. **Release and absorb.** Release becomes a transfer into a program-owned escrow. A hunt the server has replayed and judged absorbed triggers the burn; a husk that escapes stays in escrow, still huntable. This is the one piece that needs an on-chain program, and its only authority should be the server's judging key.

The server already replays every lone hunt from its recorded inputs before it pays (`PlayerState.judgeHunt`), which is the property a burn needs: the chain acts on the server's verdict, never on the phone's claim.

On the Dart side the realistic choices are the `solana` package for transactions and the Mobile Wallet Adapter client for signing. Neither is a dependency yet: adding them before there is a program to call would be dead weight.

## What has to be true before real money

- Rift results are trusted, not replayed. Fix that before a rift pays anything of value.
- Rifts are free to enter and have no cooldown.
- The stand-in prices are invented. Real floors move.
- Supported collections need the holders' and creators' consent. Burning someone's art for sport is a decision to make with them.
- Real-money mechanics need legal review in each market before launch, and app store rules on NFTs differ between Apple and Google.
