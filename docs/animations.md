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

## The reaper's own arms

In a hunt the reaper sees its own arms: a skeletal right hand low in the bottom right of the view holding the weapon, and a left hand that comes up to reload and to throw the capture orb. They come from `art/blender/viewmodel.py` as three models: `viewmodel.glb` (both arms, the grave pistol, the bone knife, the scythe, the spare soul drum, the held orb, the muzzle flash and the swing trail), `viewmodel_orb.glb` (the orb as it flies and lies in the room) and `viewmodel_fx.glb` (every light the room shows: halos, sparks, the ring wave, the soul stream and the tracer).

None of it is a clip. Each moving part is one mesh about its own pivot, and every tick the game works out the pose from pure functions of time (`render::arms::pose`) and sends it as part offsets, which the host eases over the tick on its own display clock. A pose that is a function of time can be held still for a hit-stop, slowed to a quarter, or frozen on any frame.

The seam is `seams::viewmodel_view`: the selected slot's `Archetype` and the reaper's class `WeaponKind`, the action and the tick it began, how fast the view is turning, the juice, the rules' own `OrbView` and the latest blow. The weapon and the orb cross as the rules' types, and the seam's documentation says which of the rules' events begins which action; the timings the rules own (the draw, the reload, the jam, the knife's dive, the orb's ticks) are read from them. The arms are drawn in a scene of their own from a fixed eye at 55°, over the hunt. The orb, the tracers and the sparks belong to the room, so they are drawn in the hunt's scene. Open the bench with `REAPERS_ARMS_BENCH=true` to play every action, throw the orb to each ending, and see it all over a dark, a bright or a grey background, at full speed or a quarter.

| Action | What it does |
| --- | --- |
| Draw | Brought up from below over 21 ticks (0.35 s), as long as the rules take to let a new weapon fire, the hand rolling level. |
| Rest | Breathing: a slow rise and fall, and a wander slower still. The arms trail the view's turn. |
| Fire | The muzzle kicks 6° and recovers in about 120 ms; the drum turns a chamber in 80 ms; the flash shows three frames, one each thirtieth of a second. |
| Empty | The hammer falls on nothing: a twitch, no kick. |
| Reload | 84 ticks (1.4 s): the pistol tips up and rolls in toward the middle, the spent drum drops away, and the left hand carries a glowing drum up into sight and seats it. |
| Active reload | A well-timed press snaps the drum home from 62% of the way and spins the pistol once over 0.3 s. |
| Jam | A press too early: the drum sticks halfway and rattles for 30 ticks. |
| Dive in, stab, dive out | The knife lunges in over 13 ticks and is held there. Each stab drives 25 cm in about 90 ms, from alternating sides, and the knife comes back out over 18 ticks. |
| Swing | A heavy swing: a 120 ms wind-up, then 120° in 180 ms, with a trail at its head. The strike eases in and out, so the blade is at its fastest as it crosses the middle of the view. |
| Parry | The weapon comes up and in toward the middle, turned flat across the body, for 0.42 s. |
| Struck | A jolt back for 0.45 s. |
| Raise, throw | The left hand holds the orb up while the throw is timed, then throws it underarm: it leaves the hand 0.3 s into the 0.62 s throw. |

Every class weapon swings in one of five manners (`render::arms::manner`, curves in `render::swings`): the scythe and chain whip **sweep** wide and flat; the spear and lantern pole **thrust**; the bell **tolls**, raised and brought down; the flail **lashes** with a snap; the greatsword and axe **chop** down across. Each manner is a set of curves with the same contact point (halfway through the strike), so the hunt can time a blow's landing without knowing the weapon. Only the scythe is modelled so far: every other class weapon is drawn as the scythe, swung in its own manner. Bringing one into the hand is a merged mesh under `socket_weapon` in `viewmodel.py`, a grip in `render::viewmodel`'s `hold`, and its part in `weapon_part`. The art slot's hand is empty: the art is thrown, not held, and its throw is not drawn yet.

### The light of a blow

| Effect | How it looks |
| --- | --- |
| Muzzle flash | Three frames of a crossed star, additive, in the soul's green. |
| Tracer | Four frames: its head races to what was struck in two, its tail follows, and it fades. |
| Shards | A burst of soul shards from where the blow struck, in the husk's own light, never white. Two bursts can fly at once. |
| Hit flash | The husk flashes in its own colour. |
| Damage number | Pops, rises 44 px and fades over 0.8 s. A critical one is bigger, and gold. |

### The capture orb

The orb follows the rules' timeline (`CaptureRules`, design v2's numbers), in the hunt's ticks from the moment it leaves the hand. The view reads the rules' `OrbView` as it is:

| Tick | Stage | What shows |
| --- | --- | --- |
| 0–27 | Flying | A low arc and a tumble. Its core is unlit light; a dark halo sits just behind a bright additive one, so it reads against a picture of any brightness. |
| 27–48 | Absorbing | It strikes the husk with a flare, its talons open, and the soul streams in as a spiral of motes while the husk comes apart. |
| 48–66 | Falling | Shut, it drops to the ground under where the husk was. |
| 66, 114, 168 | Pulses 1–3 | It rocks, the soul swells against the talons, and a ring wave spreads over the ground. Each pulse is stronger. |
| 168–222 | Breath | The third pulse dies away and nothing more comes. |
| 222 | Sealed | The gold sigil burns in and the light settles. |
| The pulse it breaks on | Broken | In place of that pulse the shell cracks, the talons are flung out, and the soul bursts free. The husk gathers itself again. |
| 27, missed | Missed | It comes down shut where it was thrown, with a dull knock of light, and goes out over 30 ticks. |

### Juice

A blow holds the arms still for a few frames (a seal shot 2, a stab 5, a heavy swing 6 to 8 by how hard the rules say its weapon strikes, a parry 4, a kill or a broken orb 10) by holding their clock; the world, the husk and the orb keep moving. Every blow adds trauma, which drains at 1.5 a second. The shake is trauma squared on smooth noise, so small blows barely move and big ones throw. It turns the arms and the world's 3D, never the camera's picture, and never more than 6 px. A dive with the knife zooms the lens toward the husk, never past 3×.

### Budget and limits

The view draws at most six meshes and 6,000 triangles at its busiest moment, a reload or a throw (5,650 with the pistol). The room's effects are 11 meshes in all. `render::viewmodel_budget`'s tests read the built models and hold every frame of every action to that, and check every part, pivot and morph target the code names. Nothing bakes an environment map.

What leptoskit cannot do yet:

- **The dive lens on the camera's picture.** The 3D lens zooms, and the bench zooms its background with it. A hunt over the camera needs the platform to zoom its picture with the scene.
- **A soul glow on a textured surface.** An emissive override replaces the palette's emissive texture with one flat colour, so the glowing drum and sigil cannot be brightened alone.
- **A pixel ratio cap.** The host draws at the display's full pixel ratio, three on most iPhones; the budget asks for two.

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
