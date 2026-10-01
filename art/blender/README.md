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
| `lib/armoury.py` | Parts every weapon is assembled from. |
| `lib/preview.py` | Review sheets: front, three-quarter, back and silhouette. |
| `reaper_classes/<class>.py` | One module per reaper class. `build()` returns the model's root. |
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
    socket_weapon           an empty where the weapon's grip sits, haft along Z
  arm_l                     an empty at the other shoulder (+X)
```

Inside a head, name the parts that should disappear when an outfit hollows the reaper out `void_<n>` (the darkness where a face would be) and `wrap_<n>` (whatever covers the lower face). Name the eyes `eyes_<n>`; the game scales them by the genome's gaze.

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
