# Reapers

An augmented reality game where you are a reaper and dead NFTs are what you hunt.

## Why it exists

Most NFT collections are dead: thousands of pictures nobody wants, sitting in wallets at a floor price near zero. Reapers gives them one last use. You buy one, release it into the spirit realm, hunt it down through your phone's camera, and absorb it to grow stronger. Absorbing destroys it, so every hunt takes a dead NFT out of circulation for good.

## What makes it different

You are bound to a reaper you did not choose, and the phone is its blade. The hunt is not won by tapping fast. It is won by keeping still: creep up on a husk without startling it, hold the phone steady while it shrieks at you, and reap it with one slow sweep. The biggest husks are a whole collection piled into one body at a real place, and take a circle of four who all have to hold still at the same moment.

## What is in this version

- The shuffle: six classes, each with its own silhouette, weapon and perk, and a few hundred looks apiece.
- The full hunt over the camera feed: read the room, stalk, strike, shriek, reap.
- Husks that are creatures with a place in the room: walkers on the floor, fliers in the air.
- A roll of luck on every absorption, decided and signed by the server.
- A vault and a market, an armory of weapons and outfits, a codex, levels and collection sets.
- Rifts: four-reaper raids, with shades filling the seats nobody takes.
- A server that keeps players' books and replays every hunt before paying for it. The game also runs entirely offline.

The NFT side is a simulation on purpose. The five collections are real ones from Solana's first wave (Degenerate Ape Academy, Thugbirdz, Pesky Penguins, Sollamas, Solarians), each with a creature we modelled ourselves, but there is no wallet and no chain, and nothing is licensed. [docs/nfts.md](docs/nfts.md) lists every stand-in and where the real thing plugs in. [docs/gameplay.md](docs/gameplay.md) is the rulebook, and ends with what is not built yet.

## Run it

You need [FVM](https://fvm.app) (Flutter is pinned in `.fvmrc`) and Xcode or Android Studio.

```sh
fvm flutter pub get
cd reapers_flutter
fvm flutter run
```

That is the whole game with no server. On a real phone you look around by turning; on a simulator, drag to look and tap to strike.

To watch it play itself, which is the only way to see a full hunt on a simulator:

```sh
fvm flutter run --dart-define=REAPERS_AUTOPLAY=true
```

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
| [art/blender/README.md](art/blender/README.md) | How the 3D art is built. |
| [AGENTS.md](AGENTS.md) | Conventions for anyone, human or not, changing the code. |
