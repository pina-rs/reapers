# Reaper studio

The studio is where a reaper can be looked at and played with without a game: the actual models, looks and shared progression rules, with no save, server, wallet or camera opened. Nothing done in it alters an owned reaper. It runs wherever the game runs: the browser, iOS and Android.

## Open it

The studio is the app's `/studio` route. It opens on the player's reaper, or on one the shuffle draws for a player who has none, and it is under **Reaper studio** in the sanctum.

```sh
cd crates/reapers_app
leptoskit dev --platform web                   # then open http://127.0.0.1:3010/studio
REAPERS_GALLERY=true leptoskit dev --platform web   # a debug build that opens at the bench
leptoskit dev --platform ios                   # or android: the same route in the app
```

For a static build to put on any web host, `leptoskit build --release --platform web` writes `crates/reapers_app/dist`; publish the whole directory.

## Where it stands

The studio is in the game, under **Reaper studio** in the sanctum and in **Your reapers**, at `/studio`. It is `crates/reapers_app/src/studio/`, opened through the `studio_view` seam (given only the genome it opens on, `StudioViewProps`), and draws the specimen on the game's own models with `leptoskit_3d`: WebGPU or WebGL2 in the browser, Metal on iOS, OpenGL ES on Android.

Everything below holds for it, with two differences that come from `leptoskit_3d` playing clips on the host's own clock: the timeline cannot be scrubbed, and **Inspect contact pose** is replaced by a note saying why. **Inspect expression** holds the reaction's face with its morph target and moves the camera to the face, which it frames where the face stands at rest. The browser turns the specimen under a drag; the native apps turn it with the portrait button. `crates/reapers_app/tests/studio.rs` drives the controls and checks what the stage shows, and that the books are untouched.

## What it holds

**Animations · 32 states** opens movement, reaction, meme, combat and finisher previews on the current specimen. Select a clip, play or pause, replay, scrub the timeline, or change speed. Attacks have an **Inspect contact pose** shortcut. Weapon-specific finishers explain which held object they need. With reduced motion enabled, scrub by hand. The full [animation catalogue and rig guide](animations.md) covers the 32 states, class adaptations and current limits.

**Appearance** lets you try all six callings, head and shoulder variants, robe cuts, colourways, omens and auras. Select **Spirit** or **Humanoid**, choose a long robe, short tunic or no robe, and swap any of the six held objects. Turn off **Hold an object** for empty hands. These wardrobe and form choices are unsaved previews; mint metadata and class equipment rules are unchanged. Sliders change stature, gauntness, gaze and tatter. Drag to turn the model, or start its rotation with the portrait button. Copy genome preserves birth traits only.

For facial animation, select a reaction or meme and use **Inspect expression**. It pauses the clip and moves into a face close-up. Use **Full figure** to return to the body. The [animation guide](animations.md#faces-and-appearance) describes each expression and the rigs.

**Growth** demonstrates the shared level-one-to-thirty progression. Absorb a specimen adds the actual reward for an illustrative clean rare hunt with fair feeding. Scrubbing the level shows strike growth relative to a new reaper of the same calling and temperament. Weapon and outfit entries show their level gates and anima prices; select them once unlocked. **See full potential** shows level thirty, hollowed equipment and all three gift awakenings. Outfits affect appearance; levels and weapon temper affect strikes. Some birth gifts and temperaments also change the capture window.

**Soul** shows the specimen's permanent birth gift, its temperament and its original lore. Try the three gifts of a calling, inspect every awakening's concrete effect, scrub levels and add illustrative successful hunts along **Unseen**, **Stillness** and **Rhythm**. A level-thirty reaper with no remembered deeds still has its birth power: awakening requires both level and the gift's favoured memory. Real memories come from server-replayed hunt inputs, with at most one mark per path per successful hunt. Preview controls cannot change the books. See [the rules](gameplay.md) and [identity research](research/reaper-identity-and-growth.md).

**The Book of Becoming** opens from the book button in the studio. Its four chapters explain evolution, chance, the NFT future and the reasons behind the rules. Explore all gifts and temperaments, level and memory gates, exact feeding odds and unsaved random examples. Planned commerce and publicly verifiable randomness are labelled as future work. The durable record is [decisions.md](decisions.md).

Every number the studio shows comes from `reapers_rules`: `Progression`, the armory's prices and gates, `HuntStats` for strikes, `SoulRules` for awakenings and `Feeding` for the odds. It keeps no second calculator.

The studio neither buys gear nor charges currency. Its absorption count resets when it closes. The account roster is separate: it holds the starter and verified NFT reapers, with independent earned growth. Purchases and ownership setup are described in [solana.md](solana.md).
