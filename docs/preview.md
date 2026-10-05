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

**Growth** demonstrates the shared level-one-to-thirty progression. Absorb a specimen adds the actual reward for an illustrative clean rare hunt with fair feeding. Scrubbing the level shows strike growth relative to a new reaper of the same calling and temperament. Weapon and outfit entries show their level gates and anima prices; select them once unlocked. **See full potential** shows level thirty, hollowed equipment and all three gift awakenings. Outfits affect appearance; levels and weapon temper affect strikes. Some birth gifts and temperaments also change the capture window.

**Soul** shows the specimen's permanent birth gift, its temperament and its original lore. Try the three gifts of a calling, inspect every awakening's concrete effect, scrub levels and add illustrative successful hunts along **Unseen**, **Stillness** and **Rhythm**. A level-thirty reaper with no remembered deeds still has its birth power: awakening requires both level and the gift's favoured memory. Real memories come from server-replayed hunt inputs, with at most one mark per path per successful hunt. Preview controls cannot change the books. See [the rules](gameplay.md) and [identity research](research/reaper-identity-and-growth.md).

Run the renderer and soul journey in separate phone app containers, preserving the normal install and its saves:

```sh
fvm dart tool/test_studio.dart android <device-id>
fvm dart tool/test_studio.dart ios <simulator-id>
```

Run these commands from the repository root. The helper temporarily isolates the application identifier, restores the project afterwards, and saves screenshots under `.impeccable/review/`.

The preview neither buys gear nor charges currency. Its absorption count resets when it closes. The account roster is separate: it contains the starter and verified NFTs, with independent earned growth. Purchases and ownership setup are described in [solana.md](solana.md).
