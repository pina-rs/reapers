# Reapers art sources

Every 3D model in the game is built by the scripts in this folder. There are no hand-edited model files: change a script, rebuild, and commit the script together with the models it produced.

```sh
cargo xtask build-art                 # build every model into assets/models, and every portrait into assets/nfts
cargo xtask build-art husks           # build one script: reapers, weapons, husks or viewmodel
cargo xtask build-art --preview       # also render review sheets into art/build/previews
cargo xtask build-art --preview --cel # and every sheet again in the cel look (cel.toml)
cargo xtask build-art --specs-only    # write art/build/vessels.json and motions.json, and stop
cargo xtask build-art --report        # what every shipped model costs, against its budget
```

Blender's authoring API is Python, so these scripts are Python. They only ever run inside Blender; `cargo xtask build-art` (`crates/xtask/src/art.rs`) is what runs them.

## What the build does

1. It writes the numbers the scripts shape the models to, from `reapers_rules`: every vessel's gait, half-width, half-height, seals and the look of every NFT that wears it (`art/build/vessels.json`), and every reaper clip's name and timing (`art/build/motions.json`).
2. It runs Blender 5.1 on each script. Blender writes full-size models into `art/build/models`, and the husks script renders every NFT's portrait into `art/build/portraits`.
3. It compresses each model with gltfpack 0.25 into `assets/models`: `gltfpack -i <model> -o <packed> -cc -kn -km -ke`, which is meshopt compression with quantized attributes, keeping every named node (`-kn`), material (`-km`) and extra (`-ke`) the game looks things up by. The sixteen shipped models come to about 77 MiB as Blender writes them and 16 MiB packed (`--report` prints today's numbers), with every node, morph target and clip kept. The build checks that each packed model names gltfpack 0.25 as its generator.
4. It turns each portrait into WebP in `assets/nfts`, with ImageMagick (`magick`, quality 80).
5. It holds every shipped model to its budget (see [Budgets](#budgets)), and fails if one is over.

Nothing under `art/build` is committed: it is the full-size, rebuildable output, and what the review scripts read.

### The tools

| Tool | Version | Where to get it |
| --- | --- | --- |
| Blender | 5.1 | [blender.org](https://www.blender.org/download/). A different release may triangulate or export differently. |
| gltfpack | 0.25 | The `gltfpack-<platform>.zip` asset of [meshoptimizer v0.25](https://github.com/zeux/meshoptimizer/releases/tag/v0.25); put `gltfpack` on the PATH. |
| ImageMagick | 7 | Any package manager; it provides `magick`. |

The versions are pinned in `crates/xtask/src/art.rs` (`BLENDER_RELEASE`, `GLTFPACK_VERSION`), and the build refuses any other. Changing one is an upgrade done on purpose: change the constant, rebuild every model, and commit them together.

## Layout

| Path | What it is |
| --- | --- |
| `lib/core.py` | Scene, mesh and export helpers: `loft`, `revolve`, `tube`, `plate`, `blade`, `sculpt`, `finish`. |
| `lib/cloth.py` | `robe`, `sleeve` and `drape`: cloth that is drawn, not simulated. |
| `lib/anatomy.py` | Skeletal hands and forearms. |
| `lib/figure.py` | What every reaper shares: landmarks, the hidden face, both arms. |
| `lib/motion.py` | The 32 shared performances, class adaptations and NLA baking. |
| `lib/armoury.py` | Parts every weapon is assembled from. |
| `lib/preview.py` | Renders a review sheet: rows of views, each row framed on what it shows, with a silhouette and its phone-thumbnail. |
| `lib/review.py` | What each kind of model's sheets show, shared by the drivers and `review.py`. |
| `lib/cel.py`, `cel.toml` | The cel look a sheet can be rendered in, and its palette. |
| `review.py` | Renders every sheet again from the built models, without rebuilding. |
| `review_motions.py` | One pose of an exported clip, from three angles. |
| `review_catalogue.py` | Every clip of a class in five timed poses. |
| `review_trail.py` | One clip's weapon tip and wrist paths over ghosted poses. |
| `review_check.py` | Checks the review harness itself against the built models. |
| `lib/creature.py` | What every husk creature shares: pivots, seals, the shadow, and the checks a creature must pass. |
| `reaper_classes/<class>.py` | One module per reaper class. `build()` returns the model's root. |
| `creature_kinds/<vessel>.py` | One module per husk creature. `build(spec)` returns the model's root. |
| `weapon_kinds/<kind>.py` | One module per weapon kind. `build()` returns the weapon's root. |
| `viewmodel_parts/<part>.py` | What the reaper sees of itself: its arms, the grave pistol, the bone knife, the scythe in hand, the capture orb, and the light they throw, painted from one palette (`common.py`). |
| `reapers.py`, `weapons.py`, `husks.py`, `viewmodel.py` | The drivers `cargo xtask build-art` runs. |

## Building one thing while you work

The drivers take `--only`, so one class or weapon can be rebuilt and reviewed without touching the game's assets. Write the specs first (`cargo xtask build-art --specs-only`), then:

```sh
blender --background --python-exit-code 1 --python art/blender/reapers.py -- \
  --out art/build/models --preview art/build/previews --only harvester

blender --background --python-exit-code 1 --python art/blender/weapons.py -- \
  --out art/build/models --preview art/build/previews --only scythe
```

Add `--preview art/build/previews` to any driver for its review sheets (see [Reviewing a model](#reviewing-a-model)), and `--cel` for the cel look as well.

## Reviewing a model

A model is judged by eye, from its review sheets. Every sheet is a grid: a row for each thing to compare, framed on what that row shows so nothing leaves the frame, at one scale across the row. Most rows show the front, three-quarter, side and back, then the model in black with the same silhouette beside it at the size a phone shows it in a list (96 pixels tall), since that is where a shape is hardest to read. A silhouette leaves out the floor shadow, which is not the model's shape.

A model is reviewed at its bind pose: every node and morph where the file exported it, before any clip moves it. The glTF importer leaves each node at the first frame of the first clip it reads, so `review.py` puts every node a clip moves back where the file has it. To see a class in its idle stance instead, render `review_motions.py --clip idle --phase 0`.

| Sheet | What it shows |
| --- | --- |
| `reaper_<class>_spirit.png`, `reaper_<class>_humanoid.png` | The class at its bind pose, holding its own weapon: a row for each robe, with the matching head and mantle (the last of them again where a class has fewer heads or mantles than robes; the row's label says which). |
| `reaper_<class>_armoury.png` | The class holding each of the eight weapons, then each in silhouette, so a weapon is judged in a hand at its true size. |
| `weapon_<kind>.png` | One weapon alone, framed on itself. |
| `weapons.png` | All eight side by side at one scale, and in silhouette. |
| `viewmodel.png` | A row for each moment of the first-person view (fire, reload, throw, throw with the scythe, stab, swing), showing what that moment shows, lit by the palette it is painted from, with every part placed where the game's rig places it. The first view is the game's own camera at the eye, at a phone's shape; then the front, three-quarter, side and back, each framed on its own (an arm seen from its end is a fraction of its length), and the silhouette. |
| `viewmodel_parts.png` | Every first-person part alone, at rest, framed on itself, so what a hand hides in play (the orb in the palm, the spare drum) is seen too. |
| `viewmodel_orb.png` | The capture orb, its core glowing in the palette's `glow`. |
| `viewmodel_fx.png` | Every light the room shows, a row each, drawn unlit and added to a dusk ground as the game draws it, from the front, three-quarter, side and above. The halos and the flare are cards the game turns to the camera, so they are lines from the side. |
| `husk_<vessel>.png` | A row for each thing the creature can wear, in an NFT's look. |

The drivers render them with `--preview`, after building. To render them again from the models already in `art/build/models`, without rebuilding (a minute rather than fifteen for the reapers):

```sh
blender --background --python-exit-code 1 --python art/blender/review.py -- \
  --out art/build/previews [--cel] [--only reaper_harvester,weapons,husk_ape]
```

`--only` takes model names without `.glb`; `--models` and `--specs` read from somewhere other than `art/build/models` and `art/build/vessels.json`.

The first-person sheets are staged from the rig itself: `lib/review.py` reads the arms' rest place and turn, each weapon's hold, the camera's field of view and the reload and throw constants out of `crates/reapers_app/src/render/viewmodel.rs` and `arms.rs`, so they follow the game. The few numbers a pose works out inside a function (where the left hand is at the top of a reload and while a throw is held) are copied into `review.moment_stages`, which says where each comes from. If the rig's source changes shape, the review stops and names the constant it could not find.

`review_check.py` checks the harness itself against the built models: every reaper reviewed at exactly its bind pose, the trail finding the off-hand flaps in thrust, slash and parry and none in idle or a walk, the first-person moments staged where the game puts them, and a face found for every variant. It prints a `CHECK` line for each and fails if any did, in about three minutes:

```sh
blender --background --python-exit-code 1 --python art/blender/review_check.py [-- --models art/build/models]
```

### The cel look

`--cel`, on `cargo xtask build-art --preview`, on any driver and on `review.py`, renders every sheet a second time in the look the game is moving to, beside the first, as `<sheet>_cel.png`:

- flat colour bands from one key light, fixed to the camera so every view is lit alike, with the darkest band the colour multiplied by a cool shade tint;
- an ink outline drawn as an inverted hull: each mesh is welded by position and pushed out along the welded, smooth normals, so the line never tears where the shading normals split at a hard edge, and its width wobbles along the model as a drawn line does.

The look is a preview for judging shapes in that style before the game's renderer draws them; nothing it does reaches a model. Its art direction is all in [`cel.toml`](cel.toml): the ink's colour, width (in pixels of a sheet) and wobble, the number of bands, the shade tint, the key light's direction, the paper, and each material's flat colour. Change it and run `review.py --cel` to see the result.

### Motion

These read the built reapers in `art/build/models` and the clip timings in `art/build/motions.json` (`cargo xtask build-art --specs-only`), and take `--cel`, `--models` and `--motions`.

```sh
# One pose of a clip, from three angles; --phase runs from 0 (its start) to 1 (its end).
blender --background --python-exit-code 1 --python art/blender/review_motions.py -- \
  --class harvester --clip finisherReap --phase 0.52 --out art/build/previews/finisher_reap.png
# Add --face for the face close-up, --humanoid, --clothing tunic|none, --weapon <kind>, --variant <n>.

# Every clip of a class in five timed poses, eight clips to a page.
blender --background --python-exit-code 1 --python art/blender/review_catalogue.py -- \
  --class harvester --form humanoid --out art/build/motion-review [--only slash,thrust]

# How fluid a clip is, in a still: the weapon tip's and both wrists' paths over ghosted poses.
blender --background --python-exit-code 1 --python art/blender/review_trail.py -- \
  --class harvester --clip finisherReap --out art/build/previews/trail_harvester_finisherReap.png
```

On a trail sheet (front, side and top), the pose at the clip's contact is solid and the ghosts before and after it run from cold to warm. The tip (orange; the point of the weapon farthest from its grip), the right wrist (cyan) and the left wrist (pink) each have a dot at every key: the model is imported at the 30 keys a second the clips are baked at and the game plays, so the gaps are the speed. A green dot is a dead stop in the middle of the clip. A red one is a flap of a joint of that arm (shoulder, elbow or wrist), found from the joint's turn rather than the path: a drop, the joint at least 25° out within the last five keys landing on its bind pose at 15° a key or faster (the snap a key pose that leaves a joint out makes), or a jitter, out by 25° and back to within a third of that inside five keys. A flap of the head or spine is printed but not drawn. A fluid arc is smooth, opens its gaps into the strike and bunches them only where it means to hold. It prints each path's length, top speed, stops and flaps, and a `FLAP` line for each flap, such as `FLAP elbow_l: drops 65° onto its bind pose in 100 ms, keys 8-11` for the harvester's thrust.

### Budgets

```sh
cargo xtask build-art --report
```

prints, for every model in `assets/models` (it takes no script names: it always measures them all), its triangles, vertices, draw calls, materials, morph targets and clips (counted as if every node were shown), its size as Blender wrote it (when a build has left it in `art/build/models`) and packed, and the triangles of each weapon in it. It fails when a model is over its budget, and so do every build and `cargo xtask check` (a test in `crates/xtask`).

The budgets are in `crates/xtask/src/art/report.rs`. Triangles follow the guidelines above where today's models meet them: 6,000 a weapon, 14,000 a creature, 3,000 for the orb and 1,500 for the lights. Where a model was already over when the budgets were set (every reaper class, at 129,000 to 142,000 against 110,000; the lantern pole, at 6,328; the whole first-person file, which has no guideline beyond the 6,000 a moment `viewmodel.py` holds it to), its ceiling is what it measured, so it can only get better. Each packed file's size ceiling is what it measured, rounded up to the next 8 KiB. Lower a ceiling when a model slims; raise one only on purpose.

## Conventions

- **Units and axes.** Metres, Z up. A reaper faces -Y, so its right hand (the weapon hand) is at -X. The exporter converts to glTF's Y up.
- **Two forms share one character.** Spirits float with quiet arms and trailing cloth. Humanoids have articulated legs and planted walking steps; their long robes hang outside the hips and stride, while short tunics expose the knees.
- **Landmarks.** `lib/figure.py` holds the shared heights: shoulders 1.64, neck 1.73, head centre 1.88. A class that hunches carries all of them forward (toward -Y) by the same amount.
- **Masks, not colours.** Every mesh that should react to an outfit must be passed through `finish()`. It paints four mask channels (grime, char, wear, hollow) into the `Color` attribute, exported as `COLOR_0`, and writes one UV channel, `UVMap` (`TEXCOORD_0`): a box projection of each vertex's object-space position at 3.5 per metre, which the procedural surface textures are laid on. The app's build strips `COLOR_0` from every model today (`crates/reapers_app/build.rs`), so the masks show only on the review sheets. Image-bearing parts keep their own UVs (`finish(..., rest_position=False)`), and the first-person parts point theirs at a palette instead (`viewmodel_parts/common.py`). Parts that are pure light (`accent`) or pure dark (`void`) are not finished.
- **Materials are names.** The game throws the Blender material away and keeps its name, which picks a surface in the renderer's reaper look. Use only these: `cloth`, `cloth_alt`, `bone`, `metal`, `wood`, `rope`, `feather`, `gold`, `paper`, `accent`, `void`.
- **One light per reaper.** `accent` is the only thing that glows: eyes, a blade edge, a lantern flame, a bell's mouth. Keep it small.
- **Triangles.** About 110,000 for a whole class file, with all its variants. About 6,000 for a weapon. The build enforces what each model may cost: see [Budgets](#budgets).

## The node layout of a class

The game assembles any genome from a class file by showing some nodes and hiding others, so every class must have exactly this layout:

```text
reaper                      the root; an empty named "reaper"
  robe_0 .. robe_3          four hems; one is shown. Made with cloth.robe, which adds the
                            shape keys the game animates: sway_x, sway_y, billow, gaunt, long_hem
  mantle_0 .. mantle_2      three things worn on the shoulders; one is shown
  head                      an empty at the neck, so the game can turn the head
    head_0 .. head_2        three heads; one is shown. Each is an empty holding its parts
  arm_r                     an empty at the weapon-side shoulder (-X)
    elbow_r                 the elbow, carrying the lower sleeve
      wrist_r               hand pivot
        socket_weapon       an empty where the weapon's grip sits, haft along Z
  arm_l                     an empty at the other shoulder (+X)
    elbow_l
      wrist_l
```

Before export, `lib/motion.py` inserts a `motion` pivot at waist height and a separate `spine` pivot for the chest, preserving rest positions. Each of the 32 states has a spirit clip and a `humanoid_` clip, for 64 clips on fifteen pivots. The exported model uses rigid parts, with overlapping sleeve sections at the elbows. The Hollow's head lantern follows its left wrist. The Mourner's concealed second arm emerges for gestures. See [the animation guide](../../docs/animations.md) for clip names, class differences and review commands.

The reaper driver reads `art/build/motions.json`, written from `ReaperMotion` in `reapers_rules` by `cargo xtask build-art --specs-only`. Rebuild that spec before invoking Blender directly after changing the shared animation catalogue.

`lib/body.py` adds a spectral torso and arms, mutually exclusive `spirit_form` and `humanoid_form` groups, and shaped thighs, kneecaps, calves and feet. `lib/locomotion.py` solves the humanoid hip/knee/ankle chain from world-space foot targets before baking. Spirit travel keeps the arms steady, with speed expressed through lean and hover. Garment targets `short_robe`, `humanoid_robe` and `humanoid_tunic` are mutually exclusive: humanoid fits add clearance around the pelvis and legs without pulling the cloth into the hip joints. Both renderers select these targets through `reaperClothWeights`. Clothing controls are independent of the genome; the original spirit tailoring remains the basis shape.

`lib/face.py` creates one `face_<variant>` mesh per head, or one on the Hollow's carried skull. Its six expression morphs are baked into the same named NLA tracks as the body, including neutral reset tracks. Keep eyes and mouths in the head's local frame, with readable glyphs in front of face coverings. The browser and native renderer play the exported weight tracks without reimplementing expressions.

Inside a head, name the parts that should disappear when an outfit hollows the reaper out `void_<n>` (the darkness where a face would be) and `wrap_<n>` (whatever covers the lower face). Authoring eyes named `eyes_<n>` are replaced by facial glyphs before export. The genome's gaze scales glyph width and the Hollow's remaining collar embers.

## The layout of a weapon

A weapon's root is an empty named `weapon_<kind>`, with `<kind>` spelled as the game's `WeaponKind` enum spells it: `scythe`, `lanternPole`, `balanceFlail`, `spear`, `chainWhip`, `bell`, `greatsword`, `axe`. The grip is at the origin and the haft runs up +Z. +X points across the reaper's body, which is where a blade or a hanging lantern should lean.

A weapon's temper is not a second model. The shader chars and hollows the same mesh.

## What makes a class read

The classes come from [the design research](../../docs/research/dark-games-and-ar-hunting.md). The rules that matter when modelling:

- A class is recognised as a black shape by three things only: its head, its shoulder line and its weapon. Check the silhouette panel of the review sheet first.
- Menace comes from dignity and from hiding the face, never from gore or clutter. Keep shapes clean and deliberate.
- Give every class one asymmetric element.
- At most three materials on show at once.
- Hands are larger than life.

## Creatures

A husk is what is left of an NFT. Every supported collection has a creature of its own: an ape for a collection of apes, a llama for a collection of llamas. The creature is our own model. It follows the spirit of a collection (its animal, its attitude, the kind of thing it wears) and copies none of its artwork, logos or named traits.

One model serves every NFT of a collection. What tells one NFT from another is its look: three colours, the colour of its eyes, and at most one thing it wears. The looks are defined in `crates/reapers_rules/src/nft_look.rs` and reach the build through `art/build/vessels.json`.

### Building one while you work

```sh
cargo xtask build-art --specs-only           # write art/build/vessels.json

blender --background --python-exit-code 1 --python art/blender/husks.py -- \
  --out art/build/models --specs art/build/vessels.json \
  --preview art/build/previews --portraits art/build/portraits --only penguin
```

That leaves two things to look at:

- `art/build/previews/husk_<vessel>.png`: a row for each thing it can wear, each in one NFT's look: front, three-quarter, side, back, and the silhouette with its phone-size thumbnail.
- `art/build/portraits/<collection>_<n>.png`: the eight NFTs' portraits, each in its own colours. These are the pictures the game shows, so they have to read at thumbnail size.

The build fails, with a list of what to fix, if a creature breaks any rule below. `creature_kinds/penguin.py` is the reference: start by reading it.

### The spec

`build(spec)` is given the numbers the rules play by. Shape the model to them, never the other way round.

- `spec["gait"]`: `"ground"` for a creature that walks, `"air"` for one that flies.
- `spec["halfWidth"]`, `spec["halfHeight"]`: half the body's width and height in metres. The middle of the body is the origin. A walker's feet stand on the floor at `z = -halfHeight` (`creature.floor(spec)`), and the top of its head is near `z = +halfHeight`. Width and length should stay within about one and a half half-widths of the middle.
- `spec["seals"]`: where strikes land. `creature.studs(root, spec)` puts a small glowing stud at each. Model the body so every stud sits on its surface or just proud of it, never buried and never floating.

### Axes

Metres, Z up. The creature faces -Y, so its left side is +X and its right side is -X, the same as a reaper.

### Parts and pivots

A creature is not skinned. It is built from rigid parts, and the game moves them. Each moving part hangs from a pivot made with `creature.pivot(name, parent, at)`: an empty at the joint, with the part's meshes parented to it and modelled in place. A pivot must never be rotated or scaled, because the game turns it about its own axes.

| Pivot | What the game does with it | Where to put it |
| --- | --- | --- |
| `head` | Turns it to watch the player. Required. | The base of the skull or the top of the neck. |
| `neck` | Bobs it as the creature walks. Holds `head`. | The base of a long neck. |
| `jaw` | Drops it open when the creature shrieks. | The hinge of the jaw, as a child of `head`. |
| `leg_l`, `leg_r` | Swings them about the X axis, opposite each other. | The hips of a two-legged creature. |
| `leg_fl`, `leg_fr`, `leg_bl`, `leg_br` | Swings them in diagonal pairs. | The shoulders and hips of a four-legged one. |
| `arm_l`, `arm_r` | Swings them against the legs. | The shoulders. |
| `wing_l`, `wing_r` | Beats them about the Y axis. | The wing roots. |
| `tail` | Sways it about the Z axis. | The base of the tail. |
| `eye_l`, `eye_r` | Scales it: wide in fear, narrowed in fury, squeezed shut at a blow. Made by `creature.eye`. | The middle of each eye, as a child of `head`. |
| `eyes` | Scales a single band of eyes in height only, so that it never stands off the face. | The middle of the band, as a child of `head`. |
| `brow_l`, `brow_r` | Lifts the inner end in fear and drives it down in fury, turning it about Z, and raises or lowers it. Made by `creature.brow`. | The middle of each brow, as a child of `head`. Model it level. |

Use the ones the body has. A swinging limb needs its pivot at the joint and its length hanging below it, so that turning the pivot about X swings the foot forward and back.

Those names are reserved for pivots. A mesh may not share one, and the build says so if it does.

How far each part moves, so that parts can be shaped not to pass through each other:

| Part | Range |
| --- | --- |
| Legs and arms | about 47° either way at a full run, 5° at rest |
| Head | 20° to either side and 14° up or down. It is kept small because a seal on a head stays where the rules put it and the head must not leave it behind. |
| Jaw | drops 43° |
| Wings | from 20° below rest to 37° above |
| Tail | 27° either way |
| Neck | 9° forward and back |
| Eyes | from 0.08 to 2.4 times their height, and 0.4 to 2.2 times their width |
| Brows | about 34° either way, and lifted or lowered by a twentieth of the body's half-height |

When a husk is struck, frightened or reaped the whole body moves on top of all of this: it rocks up to about 9° away from a blow and twists about its feet, cowers lower as its ward fails, and is dragged up to a third of its height off the floor by the reap. Parts that only just clear each other at rest will cross then, so leave room.

These numbers are the rig's ranges as the first game played them. The husk rig on `leptoskit_3d` must keep within them, and when it changes one, this table changes with it.

Two more nodes are required on every creature: `trait_1` and `trait_2`, pivots that each hold one thing the creature can wear (a cap, a crown, a chain). The game shows at most one. Parent them to `head` if the thing is worn on the head, so it follows when the head turns. Keep them generic: no brand, logo, team, band or real person.

A walker also needs `creature.shadow(root, spec)`, the dark patch under it that makes it sit on the floor.

### Materials

Use only these. The game keeps the name and throws the rest away.

| Material | What it is | In the game |
| --- | --- | --- |
| `body` | Fur, feathers, skin, plate. | The NFT's main colour. |
| `trim` | The second colour: a belly, a face, a blanket. | The NFT's second colour. |
| `detail` | Small hard things: beak, claws, teeth, and whatever it wears. | The NFT's third colour. |
| `accent` | The light in its eyes. Keep it small. | Glows in the NFT's eye colour. |
| `dark` | Sockets, a mouth, the inside of anything hollow. | Near black. |
| `glow` | Seal studs. Made by `creature.studs`. | Glows. |
| `shadow` | The floor shadow. Made by `creature.shadow`. | A soft dark patch. |

Make organic parts with `creature.form(name, blobs, material, parent, triangles=...)`, which fuses spheres and cubes into one shape, trims it to a triangle count and paints its shading. Use `core.revolve`, `core.loft`, `core.tube` and `core.plate` for hard or regular shapes, and pass what they return through `creature.finish`.

### What makes a creature read

- It has to be the animal at a glance, as a black shape, from the side as well as the front. Check the silhouette panel and the side view first. An ape is heavy in the shoulders and walks on its knuckles; a llama is mostly neck.
- It is a husk: what is left of something. The eyes are dark sockets with a small light far back in them. Nothing else about it needs to look dead.
- Stylised and chunky, not realistic. Big clear forms, no fine detail that vanishes at three metres on a phone.
- About 14,000 triangles at most for the whole creature. The build enforces it.
- The portrait matters as much as the model. If the eight portraits do not look like a collection of the same creature in different colours, it is not done.
