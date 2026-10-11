# Reapers art sources

Every 3D model in the game is built by the scripts in this folder. There are no hand-edited model files: change a script, rebuild, and commit the script together with the model it produced.

```sh
dart tool/build_art.dart              # build every model into reapers_flutter/assets/models
dart tool/build_art.dart --preview    # also render review sheets into art/build/previews
```

Blender's authoring API is Python, so these scripts are Python. They only ever run inside Blender; `tool/build_art.dart` is what runs them.

## Layout

| Path | What it is |
| --- | --- |
| `lib/core.py` | Scene, mesh and export helpers: `loft`, `revolve`, `tube`, `plate`, `blade`, `sculpt`, `finish`. |
| `lib/cloth.py` | `robe`, `sleeve` and `drape`: cloth that is drawn, not simulated. |
| `lib/anatomy.py` | Skeletal hands and forearms. |
| `lib/figure.py` | What every reaper shares: landmarks, the hidden face, both arms. |
| `lib/motion.py` | The 30 shared performances, class adaptations and NLA baking. |
| `review_motions.py` | Renders poses from the exported clips for visual review. |
| `lib/armoury.py` | Parts every weapon is assembled from. |
| `lib/preview.py` | Review sheets: front, three-quarter, back and silhouette. |
| `lib/creature.py` | What every husk creature shares: pivots, seals, the shadow, and the checks a creature must pass. |
| `reaper_classes/<class>.py` | One module per reaper class. `build()` returns the model's root. |
| `creature_kinds/<vessel>.py` | One module per husk creature. `build(spec)` returns the model's root. |
| `weapon_kinds/<kind>.py` | One module per weapon kind. `build()` returns the weapon's root. |
| `reapers.py`, `weapons.py`, `husks.py` | The drivers `tool/build_art.dart` runs. |

## Building one thing while you work

The drivers take `--only`, so one class or weapon can be rebuilt and reviewed without touching the game's assets:

```sh
blender --background --python-exit-code 1 --python art/blender/reapers.py -- \
  --out art/build/models --preview art/build/previews --only harvester

blender --background --python-exit-code 1 --python art/blender/weapons.py -- \
  --out art/build/models --preview art/build/previews --only scythe
```

A class's review sheets are `art/build/previews/reaper_<class>_0.png` to `_2.png`, one for each variant, with the class's own weapon in hand when that weapon exists. A weapon's sheet is `art/build/previews/weapon_<kind>.png`.

## Conventions

- **Units and axes.** Metres, Z up. A reaper faces -Y, so its right hand (the weapon hand) is at -X. The exporter converts to glTF's Y up.
- **A reaper floats.** The hem ends a little above the ground, around z = 0.06 to 0.3. Nothing has legs or feet.
- **Landmarks.** `lib/figure.py` holds the shared heights: shoulders 1.64, neck 1.73, head centre 1.88. A class that hunches carries all of them forward (toward -Y) by the same amount.
- **Masks, not colours.** Every mesh that should react to an outfit must be passed through `finish()`. It paints the four mask channels the game's shader reads (grime, char, wear, hollow) and stores each vertex's rest position in the two UV channels. Parts that are pure light (`accent`) or pure dark (`void`) are not finished.
- **Materials are names.** The game throws the Blender material away and keeps its name, which picks a surface in `reapers_flutter/lib/render/reaper_look.dart`. Use only these: `cloth`, `cloth_alt`, `bone`, `metal`, `wood`, `rope`, `feather`, `gold`, `paper`, `accent`, `void`.
- **One light per reaper.** `accent` is the only thing that glows: eyes, a blade edge, a lantern flame, a bell's mouth. Keep it small.
- **Triangles.** About 110,000 for a whole class file, with all its variants. About 6,000 for a weapon.

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

Before export, `lib/motion.py` inserts a `motion` pivot at waist height between `reaper` and these children, preserving their rest positions. All 32 clips key this pivot and the thirteen joints. The exported model uses rigid parts, with overlapping sleeve sections at the elbows. The Hollow's head lantern follows its left wrist. The Mourner's concealed second arm emerges for gestures. See [the animation guide](../../docs/animations.md) for clip names, class differences and review commands.

The reaper driver reads `art/build/motions.json`, written by `fvm dart tool/build_art.dart --specs-only`. Rebuild that spec before invoking Blender directly after changing the shared animation catalogue.

`lib/body.py` adds a spectral torso and arms, mutually exclusive `spirit_form` and `humanoid_form` groups, and hip/knee/foot pivots. The `short_robe` morph makes a tunic while preserving the upper tailoring. Clothing visibility and this morph are presentation controls, independent of the genome.

`lib/face.py` creates one `face_<variant>` mesh per head, or one on the Hollow's carried skull. Its six expression morphs are baked into the same named NLA tracks as the body, including neutral reset tracks. Keep eyes and mouths in the head's local frame, with readable glyphs in front of face coverings. The browser and native renderer play the exported weight tracks without reimplementing expressions.

Inside a head, name the parts that should disappear when an outfit hollows the reaper out `void_<n>` (the darkness where a face would be) and `wrap_<n>` (whatever covers the lower face). Authoring eyes named `eyes_<n>` are replaced by facial glyphs before export. The genome's gaze scales glyph width and the Hollow's remaining collar embers.

## The layout of a weapon

A weapon's root is an empty named `weapon_<kind>`, with `<kind>` spelled as the game's `WeaponKind` enum spells it: `scythe`, `lanternPole`, `balanceFlail`, `spear`, `chainWhip`, `bell`. The grip is at the origin and the haft runs up +Z. +X points across the reaper's body, which is where a blade or a hanging lantern should lean.

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

One model serves every NFT of a collection. What tells one NFT from another is its look: three colours, the colour of its eyes, and at most one thing it wears. The looks are defined in `reapers_shared/lib/src/nft_look.dart` and reach the build through `art/build/vessels.json`.

### Building one while you work

```sh
dart run tool/build_art.dart --specs-only     # write art/build/vessels.json

blender --background --python-exit-code 1 --python art/blender/husks.py -- \
  --out art/build/models --specs art/build/vessels.json \
  --preview art/build/previews --portraits art/build/portraits --only penguin
```

That leaves three things to look at:

- `art/build/previews/husk_<vessel>.png`: front, three-quarter, side, back and silhouette, wearing the first thing it can wear.
- `art/build/previews/husk_<vessel>_trait2.png`: the same creature wearing the second.
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

The numbers live in `reapers_flutter/lib/render/husk_rig.dart`.

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
