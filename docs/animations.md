# Reaper animations

Open **Animations · 32 states** in the [Reaper studio](preview.md). Choose a group and a clip, pause it, scrub any pose, replay it, or slow it to quarter speed. **Inspect contact pose** goes straight to an attack's contact frame. Change calling, birth traits and equipment in the same studio. Everything is an unsaved specimen.

The six reapers have separate spirit and humanoid performances. Spirit form keeps the floating silhouette. Humanoid form has shaped thighs, kneecaps, calves and feet, with foot targets and two-bone IK solved in Blender. Walking has heel contact, grounded support, toe-off and swing clearance. Running adds flight. Jump, dodge and dash have bounded local movement and return to their starting position. A future third-person controller must supply matching world speed and terrain adaptation.

## Faces and appearance

Open **Appearance** to choose a calling and change the actual model variants with **Head / face covering**, **Shoulder piece** and **Robe cut**. **Clothing** switches between the class's long robe, a shortened tunic and no outer robe. A spectral torso and arms remain when clothing is removed. The shoulder piece has its own visibility switch.

**Held object** can preview any of the six weapon models on any calling, or turn off **Hold an object** for empty hands. This does not change class equipment rules or saved equipment. The Hollow still carries its own skull in the other hand.

All reactions and meme animations have synchronized eye and mouth morphs. Laugh opens the mouth and narrows the eyes; cry adds a downturned mouth and tear trails; rage tightens the eyes; fear opens them; sigma and GigaChad use an asymmetric smirk; absolute cinema uses an awed expression. Bow uses a restrained sorrowful expression, victory uses joy, and shrug and T-pose use a smirk. All clips return the face to neutral, including when switching to movement.

Choose a reaction or meme and press **Inspect expression** to pause halfway through and focus the camera on its face. The portrait's **Face close-up** and **Full figure** buttons switch framing. The camera follows the animated face, including the Hollow's carried skull. Masks, hats, hoods and the Mourner's veil remain; the features are spectral light rather than human skin.

## The 32 clips

| Group | Clips |
| --- | --- |
| Movement | Idle, walk very slowly, walk, run, sprint, jump, land, dodge left, dodge right, phantom dash, dash left, dash right |
| Reactions | Laugh, cry, angry pose, recoil in fear, bow, victory |
| Memes | GigaChad stance, sigma chin lift, unbothered shrug, absolute cinema, T-pose dominance |
| Combat | Anime power charge, weapon slash, weapon thrust, parry |
| Finishers | Reaping arc, impale, soul crush, palm burst, rising uppercut |

The five gaits loop. Everything else plays once and recovers to rest. Reaping arc uses a scythe or chain whip; impale requires a spear. The other 28 moves work with every weapon. Weapon-free performances hide the equipped weapon while they play. The studio keeps incompatible finishers visible but disables selection and explains the weapon requirement.

These are authored character performances, not new combat rules. Contact markers are presentation metadata for synchronizing an enemy reaction later. The preview does not spawn a victim, run paired enemy choreography, award damage, or trigger a kill in a hunt. Existing AR combat remains controlled by the phone. There is no new movement controller or animation unlock economy.

## Rig and class differences

Every GLB contains 64 clips for the 32 studio states. A state such as `walk` uses `walk` in spirit form and `humanoid_walk` in humanoid form. Fifteen shared pivots include `motion`, `spine`, `head`, shoulders, elbows, wrists, hips, knees and feet. The weapon socket follows the right wrist. The chest moves separately from the waist so the pelvis can lead a strike while the head settles later. Shape-preserving cubic curves carry velocity through breakdown poses and contact instead of braking at every key. The rig uses overlapping rigid anatomical parts, not deforming skin; fingers remain fixed. Facial morphs are baked into both performances.

The [motion review](research/motion-review.md) records references, a visual target for every state, reproducible contact sheets and current limits. Presentation cycle times are 2.4 seconds for creep, 1.2 for walk, 0.8 for run and 0.6 for sprint.

The same local rotations fit all head, mantle and hem variants. The heavier Ferryman, Weigher and Mourner use restrained arm gestures; the Chooser and Hollow use broader ones. The Hollow keeps the skull lantern attached to its left wrist and uses its free right hand for soul crush and palm burst. The Mourner has a second skeletal arm concealed under its veil at rest; its head movement is reduced because the head variants include hanging veils.

Large hats, rigid capes and fixed finger poses limit close-up realism. There is no cloth collision simulation or hand IK. Review a new silhouette at extreme poses before adding it to the catalogue.

## In the leptoskit game

The Rust game (`crates/reapers_app`) plays the same 64 clips from the same GLBs through `leptoskit_3d`, which samples and blends them on the host. Neither side reimplements a joint.

- **Which clip.** `render::appearance::ReaperAppearance::clip_name` picks the spirit or `humanoid_` performance. `render::playback::ReaperPlayback` keeps the Flutter clock (select, toggle, replay, speed, and the loop and one-shot rules) and tells the host what to play: gaits loop and everything else plays once and holds its last frame, which every clip authors as rest. Each state crossfades in: a gait over 0.35 s, a step such as a jump or a dodge over 0.15 s, a strike or a finisher over 0.1 s, a reaction or a meme over 0.25 s.
- **The weapon.** The weapons model follows `socket_weapon` with every weapon but the held one hidden, and weapon-free performances hide it while they play.
- **The cloth.** The robe's genome morphs (gauntness and tatter), its fit for the form and the cut, and its sway are set on the worn robe alone, from the same clock, a few times a second.
- **Faces.** The clips carry their faces. To inspect an expression the studio holds the reaction's morph target (joy, sorrow, anger, fear, smirk or awe) with a weight, which overrides what the clip sets, and moves the camera to the face.
- **What it cannot do.** The host plays a clip on its own clock and cannot seek one, so the studio cannot scrub the timeline or hold the exact contact frame; it says so where the Flutter studio had those controls. The face camera aims where the face stands at rest, not where the animated face is: the host does not report a part's place.

`cargo test -p reapers_app` checks that every one of the 64 clips, the six face targets and the eight cloth targets exist in all six models, and `tests/render.rs` draws all six reapers and a face close-up headless.

## Build and review

`ReaperMotion` in `reapers_rules` owns clip IDs, durations, loop flags, contact fractions and weapon capabilities. `cargo xtask build-art` writes those timings to `art/build/motions.json`; `art/blender/lib/motion.py` authors the poses and bakes them at 30 FPS into matching NLA tracks. The renderer plays the exported GLB clips and never reimplements joint animation; cloth morphs follow the same clock and preserve genome traits.

```sh
cargo xtask build-art reapers

# Rebuild one class after changing its geometry or choreography, for review.
cargo xtask build-art --specs-only
blender --background --python-exit-code 1 --python art/blender/reapers.py -- \
  --out art/build/models --only mourner

# Review an exported clip at its contact point, from three angles.
blender --background --python-exit-code 1 --python art/blender/review_motions.py -- \
  --class harvester --clip finisherReap --phase 0.52 \
  --out art/build/previews/finisher_reap.png
```

Only `cargo xtask build-art` writes `assets/models`, compressed; a model built by hand into `art/build/models` is for review. Commit the source and all rebuilt models together, and rebuild the app after changing one: its `build.rs` mirrors `assets/models` into the bundle. Reduced-motion settings stop automatic character motion as well as rotation; manual timeline scrubbing remains available.

`cargo xtask check` checks that every shipped model comes from a script and was packed by the pinned gltfpack. The checks of the clip catalogue itself (every clip, its joint tracks, durations, finite samples and loop seams) and of playback (one-shot completion, replay, pause, seeking and speed) come back with the renderer on `leptoskit_3d`.

## Gesture references

The meme performances borrow readable body gestures, not textures, audio, likenesses or downloaded animation data. [GigaChad](https://knowyourmeme.com/memes/gigachad/) informed the broad, chin-up stance. [Sigma face](https://knowyourmeme.com/editorials/guides/what-is-the-sigma-face-and-who-is-the-sigma-girl-the-meme-explained) informed a slow turn and chin lift adapted to a hidden face. [Absolute Cinema](https://knowyourmeme.com/memes/absolute-cinema-kino-is-served) informed the raised-hand pose. The combat performances use exaggerated anticipation, brief contact and recovery, with no gore.
