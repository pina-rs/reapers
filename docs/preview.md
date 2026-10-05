# Reaper studio

The standalone preview runs on web, Android and iOS. It uses the game's actual models, looks and shared progression rules. It opens no save, server, wallet or camera. Nothing done in it alters an owned reaper. Desktop apps are outside the supported targets.

```sh
cd reapers_flutter
fvm flutter run -d chrome -t lib/main_preview.dart
```

To run on an Android or iOS device or simulator, substitute its device ID for `chrome`. The same screen is under **Reaper studio** in the sanctum and in **Your reapers**. `serverpod start` also knows a non-automatic `studio` Flutter target.

The Android SDK needs CMake 3.31.4 for the rendering dependency's native plugin, even though the mobile portrait uses Flutter GPU. Install that SDK component when setting up a fresh Android toolchain.

Build a static browser preview for any web host:

```sh
fvm flutter build web -t lib/main_preview.dart
```

Publish the complete `build/web` directory, including its local WebGL bindings. The browser needs WebGL 2. Mobile uses Flutter GPU; web uses a separate WebGL renderer so native GPU code does not enter the browser build. Both load the same GLBs and `reaper.fmat` surface arithmetic. Lighting differs between renderers; web does not promise identical pixels or the mobile camera hunt.

**Birth traits** lets you try all six callings, head and mantle variants, hems, colourways, omens and auras. Sliders change stature, gauntness, gaze and tatter. Drag the model to turn it or pause its automatic rotation, which makes one slow revolution and then holds. Copy genome preserves a specimen's reproducible code. Owned reapers cannot edit these traits: mint metadata determines their birth.

**Growth** demonstrates the shared level-one-to-thirty progression. Absorb a specimen adds the actual reward for an illustrative clean rare hunt with fair feeding. Scrubbing the level shows strike growth relative to a new reaper of the same calling. Weapon and outfit entries show their level gates and anima prices; select them once unlocked. **See full potential** shows level thirty and hollowed equipment. Outfits affect appearance; levels and weapon temper affect strikes. Capture windows are unchanged.

The preview neither buys gear nor charges currency. Its absorption count resets when it closes. The account roster is separate: it contains the starter and verified NFTs, with independent earned growth. Purchases and ownership setup are described in [solana.md](solana.md).
