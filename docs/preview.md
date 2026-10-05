# Reaper studio

The standalone preview is a native Flutter app showing the same GPU models and materials as the game. It opens no save, server, wallet or camera. Nothing done in it alters an owned reaper.

```sh
cd reapers_flutter
fvm flutter run -d macos -t lib/main_preview.dart --enable-impeller
```

macOS needs version 14 or newer because the game's bundled depth runtime requires it. To run on a phone or simulator, substitute that device for `macos`. The same screen is under **Reaper studio** in the sanctum and in **Your reapers**. `serverpod start` also knows a non-automatic `studio` Flutter target.

**Birth traits** lets you try all six callings, head and mantle variants, hems, colourways, omens and auras. Sliders change stature, gauntness, gaze and tatter. Drag the model to turn it or pause its automatic rotation, which makes one slow revolution and then holds. Copy genome preserves a specimen's reproducible code. Owned reapers cannot edit these traits: mint metadata determines their birth.

**Growth** demonstrates the shared level-one-to-thirty progression. Absorb a specimen adds the actual reward for an illustrative clean rare hunt with fair feeding. Scrubbing the level shows strike growth relative to a new reaper of the same calling. Weapon and outfit entries show their level gates and anima prices; select them once unlocked. **See full potential** shows level thirty and hollowed equipment. Outfits affect appearance; levels and weapon temper affect strikes. Capture windows are unchanged.

The preview neither buys gear nor charges currency. Its absorption count resets when it closes. The account roster is separate: it contains the starter and verified NFTs, with independent earned growth. Purchases and ownership setup are described in [solana.md](solana.md).
