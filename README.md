# Reapers

An augmented reality game where you are a reaper and dead NFTs are what you hunt.

## Why it exists

Most NFT collections are dead: thousands of pictures nobody wants, sitting in wallets at a floor price near zero. Reapers gives them one last use. You buy one, release it into the spirit realm, hunt it down through your phone's camera, and absorb it to grow stronger. Absorbing destroys it, so every hunt takes a dead NFT out of circulation for good.

## What makes it different

You are bound to a reaper you did not choose, and the phone is its blade. The hunt is not won by tapping fast. It is won by keeping still: creep up on a husk without startling it, hold the phone steady while it shrieks at you, and reap it with one slow sweep. The biggest husks are a whole collection piled into one body at a real place, and take a circle of four who all have to hold still at the same moment.

## What is in this version

- The shuffle: six classes, each with its own silhouette, weapon and perk, and a few hundred looks apiece.
- Eighteen permanent birth gifts and four temperaments. Each reaper's remembered successful hunts awaken its own power at levels five, twelve and twenty.
- The full hunt over the camera feed: trace a sigil to draw the husk out (harder for rarer husks) while the phone reads the room, then stalk, strike, shriek, reap.
- Husks that are creatures with a place in the room: walkers on the floor, fliers in the air. They leap onto counters and tables and hide behind them, and the real counter hides them.
- Arts, learned by absorbing the creatures that carry them: Cinder (fire, lobbed over cover), Rime (ice that freezes) and Gale (wind that blows a husk out of hiding). Hold, steady the phone, flick to throw.
- Two heavy weapons for any class: a greatsword that stops a husk in its tracks and an axe that bites the body anywhere.
- A roll of luck on every absorption, decided and signed by the server.
- A vault and a market, an armory of weapons and outfits, a codex, levels and collection sets.
- Rifts: four-reaper raids, with shades filling the seats nobody takes.
- A server that keeps players' books and replays every hunt before paying for it. The game also runs entirely offline.

Purchasing and burning hunt husks remains simulated. The five collections are real ones from Solana's first wave (Degenerate Ape Academy, Thugbirdz, Pesky Penguins, Sollamas, Solarians), each with a creature we modelled ourselves; those stand-ins are not licensed. Wallet proof and verified reaper ownership are separate, as described below. [docs/nfts.md](docs/nfts.md) lists every stand-in and where the real thing plugs in. [docs/gameplay.md](docs/gameplay.md) is the rulebook, and ends with what is not built yet.

## Run it

You need [FVM](https://fvm.app) (Flutter is pinned in `.fvmrc`) and Xcode or Android Studio.

```sh
fvm flutter pub get
cd reapers_flutter
fvm flutter run
```

That is the whole game with no server. On a real phone you look around by turning; on a simulator, drag to look and tap to strike.

On an Android phone the hunt reads the room through the camera before the husk appears: it finds the floor and what stands on it, and the husk keeps to the floor and goes round the furniture. That runs on the phone, with a depth model bundled in the build. [docs/environment.md](docs/environment.md) says how, and how far to trust it.

### Explore reapers without starting a game

Open **Reaper studio** from the sanctum, or run the standalone preview:

```sh
cd reapers_flutter
fvm flutter run -d chrome -t lib/main_preview.dart
```

The studio runs in a browser or on Android and iOS. It shows the actual 3D models: change birth traits, absorb illustrative husks, scrub levels and try unlocked weapons and outfits. It never reads or writes a player save. The camera hunt runs on Android and iOS; desktop apps are not supported targets. See [docs/preview.md](docs/preview.md).

Open **Animations · 32 states** to try gliding gaits, reactions, meme poses, weapon attacks and finishers. Play, pause, slow down or scrub every clip on the customized character. See [the animation catalogue](docs/animations.md) for rig details and weapon restrictions.

Use **Appearance** to swap head and shoulder variations, robe cuts and held objects. Try **Humanoid** for articulated legs, or **Spirit** for floating wisps. Clothing can be a long robe, short tunic or no outer robe. Reactions and memes have expressive eyes and spectral mouths; **Inspect expression** pauses a clip and focuses on the face.

**Your reapers** holds the starter and verified NFT reapers, each with its own experience, equipment and codex. Online Android builds can prove wallet ownership through Mobile Wallet Adapter. Solana Kit verifies assets from a configured Metaplex Core collection; buying happens in the configured marketplace, then the game imports the asset. No collection or marketplace is invented by default. The hunt's five husk collections, prices and burns remain simulated. See [docs/solana.md](docs/solana.md) for setup and the remaining chain work.

To watch it play itself, which is the only way to see a full hunt on a simulator:

```sh
fvm flutter run --dart-define=REAPERS_AUTOPLAY=true
```

### On the simnet: a devnet rehearsal

To show the game's chain story where no cluster has the Reapers program yet, for a demo or a hacker house:

```sh
cd reapers_flutter
fvm flutter run --release --dart-define=REAPERS_SIMNET=true
```

That is the offline game with a pretend devnet on the phone. Binding a reaper makes a burner wallet and airdrops its SOL. Buying a husk, releasing it into escrow, and absorbing it each post a signed transaction, and the burn carries the keeper's Ed25519 receipt, checked on the phone the way the program would check it. A slip confirms each transaction as it lands, and **Simnet ledger** in the sanctum is the explorer. **Start over** there makes a new wallet and reaper for the next player, without touching the offline save. Nothing leaves the phone. See [docs/solana.md](docs/solana.md#the-simnet).

### With the server

```sh
cd reapers_server
cp config/passwords.example.yaml config/passwords.yaml   # once
serverpod start
```

`serverpod start` runs the server with an embedded database. Then point a build at it:

```sh
cd reapers_flutter
fvm flutter run --dart-define=REAPERS_SERVER_URL=http://localhost:8080/
```

## Work on it

```sh
dart tool/hooks.dart install  # once per clone: guard main (see below)
dart tool/check.dart          # format, analysis and every test
dart tool/check.dart --fast   # the same without the server tests
dart tool/build_art.dart      # rebuild every 3D model and NFT portrait (needs Blender 5.1)
dart tool/build_depth_model.dart  # rebuild the depth model that reads rooms (needs uv)
dart tool/build_music.dart    # fetch and re-encode the music (needs ffmpeg and curl)
```

There are no pull requests. Work happens on a branch and is merged into `main` locally:

```sh
git switch -c feat/your-change
dart tool/merge.dart          # run every check, then fast-forward main
dart tool/merge.dart --push   # the same, then push main
```

The hooks make that the only way in: `main` refuses any commit that `tool/merge.dart` has not just checked.

| Read | For |
| --- | --- |
| [docs/gameplay.md](docs/gameplay.md) | The rules and every number in them. |
| [docs/architecture.md](docs/architecture.md) | How the pieces fit, and the known limits. |
| [docs/nfts.md](docs/nfts.md) | What is simulated, the signed receipt, and how Solana comes in. |
| [docs/environment.md](docs/environment.md) | What the game knows about the room, and how it could know more. |
| [docs/research/solana-collections.md](docs/research/solana-collections.md) | 116 old Solana collections, and why these five. |
| [docs/research/dark-games-and-ar-hunting.md](docs/research/dark-games-and-ar-hunting.md) | The research the classes and the hunt came from. |
| [docs/research/reaper-identity-and-growth.md](docs/research/reaper-identity-and-growth.md) | Developer sources and the reasoning behind birth gifts, memory and awakenings. |
| [docs/decisions.md](docs/decisions.md) | The Book of Becoming: interactive evolution, exact randomness, design decisions and the proposed NFT future. |
| [docs/launch-readiness.md](docs/launch-readiness.md) | Completed integration, launch engineering still required and decisions needed from Ifiok. |
| [art/blender/README.md](art/blender/README.md) | How the 3D art is built. |
| [AGENTS.md](AGENTS.md) | Conventions for anyone, human or not, changing the code. |

## Licence

The code, the art scripts and the models and portraits they build are under the [MIT licence](LICENSE).

Three things in the repository are other people's and keep their own licences:

| What | Licence | Where it says so |
| --- | --- | --- |
| The music, by Kevin MacLeod | CC BY 4.0 | [assets/music/CREDITS.md](reapers_flutter/assets/music/CREDITS.md) |
| The depth model, Depth Anything V2 Small | Apache-2.0 | [assets/depth/LICENSE.txt](reapers_flutter/assets/depth/LICENSE.txt) |
| DM Sans and Fraunces | SIL Open Font License 1.1 | [assets/fonts/](reapers_flutter/assets/fonts/) |

The five collections' names belong to their holders and are not covered by the licence; see [docs/nfts.md](docs/nfts.md).
