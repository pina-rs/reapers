# Reaper animations

Open **Animations · 30 states** in the [Reaper studio](preview.md). Choose a group and a clip, pause it, scrub any pose, replay it, or slow it to quarter speed. **Inspect contact pose** goes straight to an attack's contact frame. Change calling, birth traits and equipment in the same studio. Everything is an unsaved specimen.

The six reapers keep their floating silhouettes. Walk, run and sprint are distinct in-place gliding cycles. Jump, dodge and dash have bounded local movement and return to their starting position. A future third-person controller must own world movement separately.

## The 30 clips

| Group | Clips |
| --- | --- |
| Movement | Idle hover, walk very slowly, walk, run, sprint, jump, land, dodge left, dodge right, phantom dash |
| Reactions | Laugh, cry, angry pose, recoil in fear, bow, victory |
| Memes | GigaChad stance, sigma chin lift, unbothered shrug, absolute cinema, T-pose dominance |
| Combat | Anime power charge, weapon slash, weapon thrust, parry |
| Finishers | Reaping arc, impale, soul crush, palm burst, rising uppercut |

The five gaits loop. Everything else plays once and recovers to rest. Reaping arc uses a scythe or chain whip; impale requires a spear. The other 28 moves work with every weapon. Weapon-free performances hide the equipped weapon while they play. The studio keeps incompatible finishers visible but disables selection and explains the weapon requirement.

These are authored character performances, not new combat rules. Contact markers are presentation metadata for synchronizing an enemy reaction later. The preview does not spawn a victim, run paired enemy choreography, award damage, or trigger a kill in a hunt. Existing AR combat remains controlled by the phone. There is no new movement controller or animation unlock economy.

## Rig and class differences

Every GLB contains 30 named animations and eight shared pivots: `motion`, `head`, `arm_r`, `arm_l`, `elbow_r`, `elbow_l`, `wrist_r`, `wrist_l`. The weapon socket follows the right wrist. `motion` sits at waist height below the stationary `reaper` placement root. The rig uses articulated rigid parts, not a deforming skeletal skin. Overlapping sleeve sections cover the elbows; finger shapes are fixed.

The same local rotations fit all head, mantle and hem variants. The heavier Ferryman, Weigher and Mourner use restrained arm gestures; the Chooser and Hollow use broader ones. The Hollow keeps the skull lantern attached to its left wrist and uses its free right hand for soul crush and palm burst. The Mourner has a second skeletal arm concealed under its veil at rest; its head movement is reduced because the head variants include hanging veils.

Large hats, rigid capes and fixed finger poses limit close-up realism. There is no cloth collision simulation or hand IK. Review a new silhouette at extreme poses before adding it to the catalogue.

## Build and review

`ReaperMotion` in `reapers_shared` owns clip IDs, durations, loop flags, contact fractions and weapon capabilities. `tool/build_art.dart` writes those timings to `art/build/motions.json`; `art/blender/lib/motion.py` authors the poses and bakes them at 30 FPS into matching NLA tracks. Both renderers play the exported GLB clips. Neither reimplements joint animation. `ReaperPlayback` owns playback and seeking; cloth morphs use that same clock and preserve genome traits.

```sh
fvm dart tool/build_art.dart reapers

# Rebuild one class after changing its geometry or choreography.
fvm dart tool/build_art.dart --specs-only
blender --background --python-exit-code 1 --python art/blender/reapers.py -- \
  --out reapers_flutter/assets/models --only mourner

# Review an exported clip at its contact point, from three angles.
blender --background --python-exit-code 1 --python art/blender/review_motions.py -- \
  --class harvester --clip finisherReap --phase 0.52 \
  --out art/build/previews/finisher_reap.png
```

Commit the source and all rebuilt models together. Fully relaunch the app after changing a GLB. Hot reload cannot update compiled model assets. Reduced-motion settings stop automatic character motion as well as rotation; manual timeline scrubbing remains available.

The asset tests read all six committed GLBs, checking the complete clip catalogue, joint tracks, durations, finite samples and loop seams. Playback tests cover one-shot completion, replay, pause, seeking and speed. Studio widget tests exercise controls with and without reduced motion.

## Gesture references

The meme performances borrow readable body gestures, not textures, audio, likenesses or downloaded animation data. [GigaChad](https://knowyourmeme.com/memes/gigachad/) informed the broad, chin-up stance. [Sigma face](https://knowyourmeme.com/editorials/guides/what-is-the-sigma-face-and-who-is-the-sigma-girl-the-meme-explained) informed a slow turn and chin lift adapted to a hidden face. [Absolute Cinema](https://knowyourmeme.com/memes/absolute-cinema-kino-is-served) informed the raised-hand pose. The combat performances use exaggerated anticipation, brief contact and recovery, with no gore.
