# Reapers art sources

Every 3D model in the game is built by the scripts in this folder. There are no hand-edited model files: change a script, rebuild, and commit the script together with the models it produced.

The look every model is built for is **Second Ink**: a two-plate relief print, near-black key ink on bone paper plus one spot colour, the reaper's own light. [The art bible](../../docs/art-bible.md) is the authority on it; [`cel.toml`](cel.toml) holds its numbers, and the review sheets draw every model in it (see [Second Ink on the sheets](#second-ink-on-the-sheets)).

```sh
cargo xtask build-art                 # build every model into assets/models, and every portrait into assets/nfts
cargo xtask build-art husks           # build one script: reapers, weapons, husks or viewmodel
cargo xtask build-art --preview       # also render review sheets into art/build/previews
cargo xtask build-art --preview --cel # and every sheet again in Second Ink (cel.toml)
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
| `lib/ink.py` | The ink weight every vertex carries, and the one function that writes it into an exported model. |
| `lib/preview.py` | Renders a review sheet: rows of views, each row framed on what it shows, with a silhouette and its phone-thumbnail. |
| `lib/review.py` | What each kind of model's sheets show, and the lineups, shared by the drivers and `review.py`. |
| `lib/cel.py`, `cel.toml` | Second Ink on the review sheets, and every number of the look. |
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

### Who owns what

| Files | Owner | A change there |
| --- | --- | --- |
| [`docs/art-bible.md`](../../docs/art-bible.md) | Art direction | Changes the look. `cel.toml` and the sheets follow it. |
| `lib/**`, `cel.toml`, the four drivers, `review*.py`, this README | The art foundation | Changes every model at once: rebuild them all and look at every sheet before landing it. |
| `reaper_classes/<class>.py`, `creature_kinds/<vessel>.py`, `weapon_kinds/<kind>.py`, `viewmodel_parts/<part>.py` | The team making that model | Changes that model alone. Anything a second model would need belongs in `lib`: ask for it there rather than copying it. |

The contracts in this README (node names and layout, reserved pivots and their ranges, material names, axes and units, the creature spec, the ink weight's carrier) bind both sides: a model module keeps them, and the foundation changes one only with every model that relies on it.

## Building one thing while you work

The drivers take `--only`, so one class or weapon can be rebuilt and reviewed without touching the game's assets. Write the specs first (`cargo xtask build-art --specs-only`), then:

```sh
blender --background --python-exit-code 1 --python art/blender/reapers.py -- \
  --out art/build/models --preview art/build/previews --only harvester

blender --background --python-exit-code 1 --python art/blender/weapons.py -- \
  --out art/build/models --preview art/build/previews --only scythe
```

Add `--preview art/build/previews` to any driver for its review sheets (see [Reviewing a model](#reviewing-a-model)), and `--cel` for Second Ink as well (`--cel-only` for Second Ink alone).

### The fast path

`--fast` on any driver builds the geometry and renders its review sheets straight from it, and stops there: no clips are baked, no faces baked, nothing is exported and no portrait is rendered. It needs `--preview`. A reaper class takes about half a minute instead of fifteen, a husk or all eight weapons a quarter of a minute, so the build-look-fix loop is one command:

```sh
cargo xtask build-art --specs-only    # once: vessels.json and motions.json

blender --background --python-exit-code 1 --python art/blender/reapers.py -- \
  --out art/build/models --preview art/build/previews --fast --cel-only --only harvester --sheets spirit

blender --background --python-exit-code 1 --python art/blender/husks.py -- \
  --out art/build/models --specs art/build/vessels.json --preview art/build/previews --fast --cel --only ape

blender --background --python-exit-code 1 --python art/blender/weapons.py -- \
  --out art/build/models --preview art/build/previews --fast --cel --only scythe

blender --background --python-exit-code 1 --python art/blender/viewmodel.py -- \
  --out art/build/models --preview art/build/previews --fast --cel-only
```

- A reaper on the fast path is posed by hand in the first key of each form's idle performance (`motion.hold`), the pose the default path's sheets show from the baked clip; the two can differ by a hair where a clip's bake smooths a key.
- `--sheets spirit,humanoid,armoury` picks a reaper's sheets (all three by default), on the drivers and on `review.py`.
- `--scene stage|roster|hunt|first_person` draws every Second Ink sheet in that scene of `cel.toml` instead of the one `[sheets]` names.
- The ink weight is computed just as an export would compute it, so the line on a fast sheet is the line the model will ship with.
- `--fast --lineup` on `reapers.py` or `husks.py` builds every class or creature (`--only` takes a comma-separated list) into one scene and renders them side by side, as `lineup_reapers.png` or `lineup_husks.png` (see the lineups under [Reviewing a model](#reviewing-a-model)). The reliquary stack, a tower several storeys high, joins a husk lineup only when `--only` names it.
- Without `--fast` every driver does exactly what it did: builds, bakes, exports, then renders the sheets from the exported file. Use that before committing, and for anything `review.py` and the motion reviews read, since they read exported models.

## Reviewing a model

A model is judged by eye, from its review sheets. Every sheet is a grid: a row for each thing to compare, framed on what that row shows so nothing leaves the frame, at one scale across the row. Most rows show the front, three-quarter, side and back, then the model in key ink with the same silhouette beside it 64 pixels tall, the art bible's 64 pt test (6.1): the size a phone shows a figure in a list, where a shape is hardest to read. Judge that strip first. A silhouette leaves out the floor shadow, which is not the model's shape.

The glTF importer leaves each node at the first frame of whichever clip it reads first, so the review puts every node a clip moves back at the transform and morph weights the file exported (its bind pose), and then plays any pose it shows on purpose. A reaper is shown in the first key of its idle clip, `idle` for the spirit and `humanoid_idle` for the humanoid (`review.STANCES`): a reaper file's own rest transforms put every joint at its parent's origin, which is no pose at all, and the game always plays a clip. Every other model has no clips and is shown as exported.

| Sheet | What it shows |
| --- | --- |
| `reaper_<class>_spirit.png`, `reaper_<class>_humanoid.png` | The class in its idle stance, holding its own weapon: a row for each robe, with the matching head and mantle (the last of them again where a class has fewer heads or mantles than robes; the row's label says which). |
| `reaper_<class>_armoury.png` | The class holding each of the eight weapons, then each in silhouette, so a weapon is judged in a hand at its true size. |
| `weapon_<kind>.png` | One weapon alone, framed on itself. |
| `weapons.png` | All eight side by side at one scale, and in silhouette. |
| `viewmodel.png` | A row for each moment of the first-person view (fire, reload, throw, throw with the scythe, stab, swing), showing what that moment shows, lit by the palette it is painted from, with every part placed where the game's rig places it. The first view is the game's own camera at the eye, at a phone's shape; then the front, three-quarter, side and back, each framed on its own (an arm seen from its end is a fraction of its length), and the silhouette. |
| `viewmodel_parts.png` | Every first-person part alone, at rest, framed on itself, so what a hand hides in play (the orb in the palm, the spare drum) is seen too. |
| `viewmodel_orb.png` | The capture orb, its core glowing in the palette's `glow`. |
| `viewmodel_fx.png` | Every light the room shows, a row each, drawn unlit and added to a dusk ground as the game draws it, from the front, three-quarter, side and above. The halos and the flare are cards the game turns to the camera, so they are lines from the side. |
| `husk_<vessel>.png` | A row for each thing the creature can wear, in an NFT's look. |
| `lineup_reapers.png`, `lineup_husks.png` | Several models side by side at one scale, in Second Ink and then in silhouette (`review.py --lineup`). |

The drivers render them with `--preview`, after building. To render them again from the models already in `art/build/models`, without rebuilding (a minute rather than fifteen for the reapers):

```sh
blender --background --python-exit-code 1 --python art/blender/review.py -- \
  --out art/build/previews [--cel] [--only reaper_harvester,weapons,husk_ape]
```

`--only` takes model names without `.glb`; `--models` and `--specs` read from somewhere other than `art/build/models` and `art/build/vessels.json`.

A lineup sets models side by side at one scale, each turned to its three-quarter view and standing on one floor, in Second Ink and then in key-only silhouette with the 64-pixel strip: every reaper in its class's light, in robe 0, holding its own weapon, or every husk in its collection's first look. A driver builds one fast (`--fast --lineup`, above); `review.py` builds one from exported models:

```sh
blender --background --python-exit-code 1 --python art/blender/review.py -- \
  --out art/build/previews --lineup reapers [--scene stage]
# --lineup husks, or a comma-separated list: --lineup reaper_harvester,reaper_hollow
```

The first-person sheets are staged from the rig itself: `lib/review.py` reads the arms' rest place and turn, each weapon's hold, the camera's field of view and the reload and throw constants out of `crates/reapers_app/src/render/viewmodel.rs` and `arms.rs`, so they follow the game. The few numbers a pose works out inside a function (where the left hand is at the top of a reload and while a throw is held) are copied into `review.moment_stages`, which says where each comes from. If the rig's source changes shape, the review stops and names the constant it could not find.

`review_check.py` checks the harness itself against the built models: every reaper imported at exactly its bind pose whatever clip the importer read first, and shown in the first key of its form's idle clip, the trail finding the off-hand flaps in thrust, slash and parry and none in idle or a walk, the first-person moments staged where the game puts them, and a face found for every variant. It prints a `CHECK` line for each and fails if any did, in about three minutes:

```sh
blender --background --python-exit-code 1 --python art/blender/review_check.py [-- --models art/build/models]
```

### Second Ink on the sheets

`--cel`, on `cargo xtask build-art --preview`, on any driver and on `review.py`, renders every sheet a second time in Second Ink, beside the first, as `<sheet>_cel.png` (`--cel-only` renders that alone). It draws a model as the game's renderer will (art bible, sections 3 to 5), so a shape can be judged in the look before the renderer exists:

- **Colour** is an overprint: key at coverage k and the spot at coverage s on paper, per family (`[families]`). A reaper prints in its class's light, a weapon alone in its owner's (`[weapons]`), the first-person view in the Ferryman's. A husk's `body`, `trim` and `detail` are its NFT's colours in the ghost wash (`[ghost]`): hue kept, chroma capped at 16, value clamped, key over it for shade and deep; its eyes print paper and its seals key.
- **Three hard bands** from N·L against each scene's thresholds (`[scenes]`), one light fixed to the camera.
- **Hatching** in the deep band, in object space so it turns with the form, thickening as the form darkens, with a crosshatch below `t_cross` on reapers. Its pitch is set so lines land the scene's `hatch_pt` apart at the scene's framing, and it fades to its mean tone where the pitch falls under 2.5 to 4 device pixels, so it never moirés.
- **Paper specular** on turning edges of bone, metal and gold, and the **spot rim** on a reaper's far edge.
- **The key line**, an inverted hull pushed out by the vertex's [ink weight](#the-ink-weight) times the two terms the game adds (heavier on the side away from the light, and a wobble fixed to the surface), at the scene's `line_pt` on a phone. Families with `line = 0` draw none.
- **The paper knockout**, a margin of paper round each whole figure outside the key line, so it stands off the vigil ground.
- A floor shadow prints as key hatching.

Everything is in [`cel.toml`](cel.toml): inks, lights, the class of each light, families, the robe's colourway, the ghost wash, the line, the hatch, the scenes and which scene each kind of sheet is drawn in. The model build reads only its `[weight]`. Change anything else and run `review.py --cel-only` (or a driver with `--fast`) to see it; no model is rebuilt.

A sheet's sizes are a phone's: each scene's `figure_pt` is how tall the framed figure stands on a 390 x 844 pt screen, and the line and hatch are drawn at their sizes in points at that height. The eye tile of `viewmodel_cel.png` is the phone's own view.

### The ink weight

Every vertex of every model carries an ink weight: how heavy the key line is there, as a multiple of the line's width (art bible, 4.3). `lib/ink.py` bakes it at build time (`ink.weigh`) from four terms, numbered in `cel.toml` `[weight]`:

| Term | From | Value |
| --- | --- | --- |
| Part | The largest dimension of the connected piece the vertex is on | 0.45 (fine) under `sizes[0]`, 0.70 (interior) under `sizes[1]`, 1.0 (silhouette) above, per kind of model; never under `minimum_pt` |
| Contact | How near the floor (a reaper's z = 0, a walking husk's lowest point) | x1.2 within 8 cm |
| Paint | The vertex group `ink_weight`, which a model script may paint (`ink.paint(obj, multiplier)`) | x2 the group's weight: 0.5 leaves it, 0 draws no line, 1 doubles it |
| Clamp | A ray cast inward along the normal | never more than 40% of the local thickness, at the heaviest the game's side term makes it |

A mesh drawn only in families whose `line` is 0 (`void`, `dark`, `accent`, `glow`) weighs 0. The game multiplies the weight by the terms the build leaves to it: the side (x1.30 away from the light, down to x0.62 where it is lit), the wobble, and the line's width on screen. The review sheets draw with the same weights: a fast sheet reads them from the build, a sheet of an exported model from the file.

**How it reaches the game.** `core.export_glb` calls `ink.carry`, the one function that writes the weight into a model, and `ink.CARRIER` names where:

```python
CARRIER = "TEXCOORD_1"
```

| `CARRIER` | Where the weight is |
| --- | --- |
| `"TEXCOORD_1"` (today) | U of the second UV set, `ink` (V is 0). A mesh with no UV set gets an empty first one, so the weight is always `TEXCOORD_1`. |
| `"COLOR_0.r"` | Red of `COLOR_0`, as weight / 2.5, over the grime mask. The app's build strips `COLOR_0` today (`crates/reapers_app/build.rs`), so it would have to keep it. |
| `"_INK_W"` | A custom attribute, `_INK_W`, written by the exporter's `export_attributes`. gltfpack drops it. |

gltfpack drops a UV set no material reads. With the flags in `crates/xtask/src/art.rs` today, line 64,

```rust
pub const GLTFPACK_FLAGS: [&str; 4] = ["-cc", "-kn", "-km", "-ke"];
```

the packed models in `assets/models` carry `TEXCOORD_0` only; the full-size ones in `art/build/models` carry the weight. When the game's renderer reads it, add `"-kv"` (keep vertex attributes) to that line, making it `[&str; 5]`, and rebuild every model. Measured on the penguin with gltfpack 0.25: `TEXCOORD_1` kept, 235 KiB packed becoming 244 KiB, so the size ceilings in `crates/xtask/src/art/report.rs` rise by about 4%.

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
- **Masks, not colours.** Every mesh that should react to an outfit must be passed through `finish()`. It paints four mask channels (grime, char, wear, hollow) into the `Color` attribute, exported as `COLOR_0`, and writes one UV channel, `UVMap` (`TEXCOORD_0`): a box projection of each vertex's object-space position at 3.5 per metre (`core.box_uvs`). The ink weight follows it as `TEXCOORD_1`. The app's build strips `COLOR_0` from every model today (`crates/reapers_app/build.rs`), so the masks show only on the review sheets. Image-bearing parts keep their own UVs (`finish(..., rest_position=False)`), and the first-person parts point theirs at a palette instead (`viewmodel_parts/common.py`). Parts that are pure light (`accent`) or pure dark (`void`) are not finished.
- **Materials are names.** The game throws the Blender material away and keeps its name, which picks a surface in the renderer's reaper look and a family in Second Ink (`cel.toml` `[families]`). Use only these: `cloth`, `cloth_alt`, `bone`, `metal`, `wood`, `rope`, `feather`, `gold`, `paper`, `accent`, `void`.
- **No surface textures.** Second Ink draws every material in flat bands, so no model carries a texture (art bible, 5.6). The procedural normal, ORM and albedo maps every material once had are gone, and so is `lib/textures.py`. What is kept: `TEXCOORD_0` on every finished mesh, the reliquary's portraits (the one exception the bible makes), and the first-person palette (`viewmodel_parts/common.py`), whose cells are how the game colours those parts.
- **One light per reaper.** `accent` is the only thing that glows: eyes, a blade edge, a lantern flame, a bell's mouth. Keep it small.
- **Triangles.** About 110,000 for a whole class file, with all its variants. About 6,000 for a weapon. The build enforces what each model may cost: see [Budgets](#budgets).

### What the library does to every model

These are the art bible's form rules (6.1) where the shared code can keep them, so every model gets them without asking:

- **Hands at 1.4x.** `reapers.py` grows every class's `arm_r_hand` and `arm_l_hand` by `figure.HAND_GROWTH` after the class is built (`figure.grow_hands`), about the haft the hand holds: the fingers grow longer and thicker and close on the same haft, and the pivots, the weapon and every clip stay where they are. Build a hand at whatever scale suits the class; the driver makes it larger. Knuckles step out of the bone (`anatomy.KNUCKLE`), so a finger reads as bone and not as a rod.
- **Round things look round.** `core.tube` and `core.revolve` take more sides than asked for when a section is wide enough for its facets to show past the key line (`core.sides_for`: within 2 mm of a true circle, at most 16 sides). A thin rod keeps the count it was given, and so does a section asked for with fewer than 8 sides, which is taken to be faceted on purpose.
- **Hard edges are bevelled.** `core.plate` chamfers sheet (2 cm thick or less) a third of its thickness wide, at most 3 mm, bevels any plate given `bevel=` in two segments, and leaves a thicker slab to its caller; a bevelled plate gets one normal a vertex, weighted by the faces round it (`core.crisp`), so flat faces stay flat, the bevel catches the paper highlight, and the key line has a closed surface to push. Use `core.crisp` on any other hard-edged mesh.
- **Budgets come first.** The two lifts that spend triangles, rounder sections and chamfered sheet, are switched off by `core.LIFTS_ADD_TRIANGLES`; `viewmodel.py` turns it off, since a first-person moment may show 6,000 triangles and its parts are within a few dozen of that.
- **Cloth folds few and large.** `cloth.robe` and `cloth.sleeve` weigh a robe's three fold counts 0.62, 0.30 and 0.08 (`cloth.FOLD_WEIGHTS`), so the finest barely shows, and cut the hem between tears into a few broad tongues, with each slit two columns wide.
- **Planes instead of clay.** `core.planes` cuts a voxel sculpt into flat facets with crisp creases, at a triangle count; `creature.facet` is `creature.form` cut that way. The humanoid boots use it (1,400 triangles each, down from 6,172), and the hidden void heads keep 30% of their faces.

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
- Hands are larger than life: the driver grows every hand 1.4x on top of the class's own scale (see [What the library does to every model](#what-the-library-does-to-every-model)).

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
- `art/build/portraits/<collection>_<n>.png`: the eight NFTs' portraits, each a specimen plate in Second Ink (art bible 3.6 and 12.5): the husk in its ghost wash, with its key line and paper knockout, in a window washed with the ghost of its backdrop, on paper, with a registration mark at each corner (`husks.specimen_plates`). Every plate of a collection is seen from the same place and framed on the creature wearing all it can, so the eight read as one set. These are the pictures the game shows, so they have to read at thumbnail size.

The build fails, with a list of what to fix, if a creature breaks any rule below. `creature_kinds/penguin.py` is the reference: start by reading it.

### The spec

`build(spec)` is given the numbers the rules play by. Shape the model to them, never the other way round.

- `spec["gait"]`: `"ground"` for a creature that walks, `"air"` for one that flies.
- `spec["halfWidth"]`, `spec["halfHeight"]`: half the body's width and height in metres. The middle of the body is the origin. A walker's feet stand on the floor at `z = -halfHeight` (`creature.floor(spec)`), and the top of its head is near `z = +halfHeight`. Width and length should stay within about one and a half half-widths of the middle.
- `spec["seals"]`: where strikes land. `creature.studs(root, spec)` puts a small glowing stud at each. Model the body so every stud sits on its surface or just proud of it, never buried and never floating: the build measures each seal's centre against the outermost surface near it, prints `SEAL seal_<n> stands … mm off the body`, and fails one buried more than half a stud's height or standing off more than three quarters of it. Never move a seal; shape the body to it.

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

Make organic parts with `creature.form(name, blobs, material, parent, triangles=...)`, which fuses spheres and cubes into one shape, trims it to a triangle count and paints its shading. `creature.facet` takes the same arguments and cuts the shape into flat planes with crisp creases instead, which prints as a carving in hard bands rather than a speckle of small facets: prefer it for heads, shoulders and anything that should catch a straight band edge, and raise its `flat` for fewer, larger planes. Use `core.revolve`, `core.loft`, `core.tube` and `core.plate` for hard or regular shapes, and pass what they return through `creature.finish`.

### What makes a creature read

- It has to be the animal at a glance, as a black shape, from the side as well as the front. Check the silhouette panel and the side view first. An ape is heavy in the shoulders and walks on its knuckles; a llama is mostly neck.
- It is a husk: what is left of something. The eyes are dark sockets with a small light far back in them. Nothing else about it needs to look dead.
- Stylised and chunky, not realistic. Big clear forms, no fine detail that vanishes at three metres on a phone.
- About 14,000 triangles at most for the whole creature. The build enforces it.
- The portrait matters as much as the model. If the eight portraits do not look like a collection of the same creature in different colours, it is not done.

### The reliquary stack

The body of a rift is not a creature: `husks.reliquary_stack` builds it, a leaning tower of four picture-frame crates crowned by a smaller casket with a gabled roof, held apart by seams of crystal at the rules' seal heights. The casket's height is whatever the rules' half-height leaves above the top seam once its roof is on, and the build fails a tower whose top stands over `halfHeight` or that reaches more than one and a half half-widths from its axis, naming the part that does. The game finds its parts by these names:

| Node | What it is |
| --- | --- |
| `cube_<storey>` | One crate, 0 at the bottom. Its own node, slid and turned a little off the one below, so the game can let it fall when the seam under it breaks. |
| `block_<storey>` | The crate's frame (`frame`): posts, rails, liners, lid and skirt; the plinth on the bottom one and the roof on the top one. |
| `panel_<storey>_<face>` | A portrait face (`canvas`): a square sheet in the frame's opening carrying the whole of one portrait, UV 0 to 1, upright as seen from outside. Face 0 looks to the front (-Y), 1 to the tower's left (+X), 2 to the back, 3 to its right. Twenty faces for a collection's eight portraits: the game deals them round. |
| `seam_<index>` | A seam of crystal between two crates (`glow`), 0 the lowest. The seams are the stack's seals. |
| `seal_<index>` | A seal's boss on the flat front of its seam (`glow`), at the rules' seal offset, as on a creature. |

The seams and seals hang from the root and stay where they are when a crate falls.
