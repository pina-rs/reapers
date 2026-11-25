"""The review sheets of every kind of model, shared by the drivers (which
render them as they build, with `--preview`) and by `review.py` (which
renders them again from models already built, without rebuilding).

Every sheet is rendered by `preview.render_grid`, which frames each row on
what it shows. See art/blender/README.md for what each sheet holds.
"""

import importlib
import json
import os

import bpy

from . import cel, core, preview

# The weapon each reaper class is shown holding, and the order the eight
# weapons are shown in. The keys are the modules in `weapon_kinds/`.
CLASS_WEAPONS = {
    "harvester": "scythe",
    "ferryman": "lantern_pole",
    "weigher": "balance_flail",
    "chooser": "spear",
    "hollow": "chain_whip",
    "mourner": "bell",
}
WEAPON_KINDS = ("scythe", "lantern_pole", "balance_flail", "spear", "chain_whip", "bell", "greatsword", "axe")

# A class file holds four robes and three heads and mantles. A row of a
# class's sheet shows robe `n` with head and mantle `n`, the last head and
# mantle again with the fourth robe.
VARIANTS = 4
FORMS = ("spirit", "humanoid")


def looks(with_cel):
    """The looks a review renders: always the normal one, and the cel look
    from `art/blender/cel.toml` as well when asked for."""

    return [None, cel.load()] if with_cel else [None]


def import_glb(path):
    """Imports a built model into the scene. Returns the objects it added."""

    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=path)

    return [obj for obj in bpy.data.objects if obj not in before]


def rest_pose():
    """Stops every clip and goes to frame 0, so every node sits at the rest
    transform it was exported with, rather than wherever a clip left it."""

    for obj in bpy.data.objects:
        animations = [obj.animation_data]

        if obj.type == "MESH" and obj.data.shape_keys is not None:
            animations.append(obj.data.shape_keys.animation_data)

        for animation in animations:
            if animation is None:
                continue

            animation.action = None

            for track in animation.nla_tracks:
                track.mute = True

    bpy.context.scene.frame_set(0)


def base(node):
    return node.name.split(".")[0]


def find(node, name):
    """The node under `node` (or `node` itself) called `name`."""

    if base(node) == name:
        return node

    return next((child for child in node.children_recursive if base(child) == name), None)


def hide_tree(node, hidden):
    node.hide_render = hidden

    for child in node.children_recursive:
        child.hide_render = hidden


# Reapers -------------------------------------------------------------------


def hold_weapons(root, kinds=WEAPON_KINDS):
    """Builds each weapon kind from its script into the reaper's
    `socket_weapon`, hidden. Returns them by kind."""

    socket = find(root, "socket_weapon")
    held = {}

    for kind in kinds:
        weapon = importlib.import_module(f"weapon_kinds.{kind}").build()
        weapon.parent = socket
        weapon.matrix_parent_inverse.identity()
        hide_tree(weapon, True)
        held[kind] = weapon

    return held


def dress(root, variant, form, weapons=None, weapon=None, clothing="robe"):
    """Shows what one genome would: robe `variant` with the matching head
    and mantle, in `form`, holding `weapon` (or nothing), as the game does.

    `clothing` is the humanoid's garment: `robe` or `tunic`, the shape keys
    the game blends for it; a spirit in a tunic wears the short robe.
    """

    counts = {prefix: 0 for prefix in ("head_", "mantle_", "robe_")}

    for node in root.children_recursive:
        name = base(node)

        for prefix in counts:
            if name.startswith(prefix) and name[len(prefix) :].isdigit():
                counts[prefix] = max(counts[prefix], int(name[len(prefix) :]) + 1)

    for node in root.children_recursive:
        name = base(node)

        if name in ("spirit_form", "humanoid_form"):
            hide_tree(node, name != f"{form}_form")

    for node in root.children_recursive:
        name = base(node)

        for prefix, count in counts.items():
            if name.startswith(prefix) and name[len(prefix) :].isdigit():
                hide_tree(node, int(name[len(prefix) :]) != min(variant, count - 1))

        if name.startswith("robe_") and node.type == "MESH" and node.data.shape_keys is not None:
            keys = node.data.shape_keys.key_blocks

            for key in keys[1:]:
                key.value = 0.0

            garment = f"humanoid_{clothing}" if form == "humanoid" else "short_robe" if clothing == "tunic" else None

            if garment is not None:
                keys[garment].value = 1.0

    for kind, held in (weapons or {}).items():
        hide_tree(held, kind != weapon)


def reaper(glb, calling, out, cel_look=None):
    """The sheets of one reaper class, from its built model at rest:

    - `reaper_<class>_spirit.png` and `reaper_<class>_humanoid.png`: a row
      for each variant (robe, head and mantle), holding the class's weapon,
      from the front, three-quarter, side and back, and in silhouette.
    - `reaper_<class>_armoury.png`: the class holding each of the eight
      weapons, then each in silhouette.
    """

    core.reset_scene()
    import_glb(glb)
    rest_pose()
    root = bpy.data.objects["reaper"]
    weapons = hold_weapons(root)
    own = CLASS_WEAPONS[calling]

    for form in FORMS:
        rows = [
            preview.row(f"{calling} {form}, robe {variant}, head and mantle {min(variant, 2)}:", lambda variant=variant, form=form: dress(root, variant, form, weapons, own))
            for variant in range(VARIANTS)
        ]
        preview.render_grid(preview.sheet_path(out, f"reaper_{calling}_{form}", cel_look), rows, tile=(400, 600), cel_look=cel_look)

    armoury = []

    for view in ("threequarter", "silhouette"):
        armoury.append(
            [
                {"label": f"{kind} {view}", "setup": lambda kind=kind: dress(root, 0, "spirit", weapons, kind), "view": view, "look": None}
                for kind in WEAPON_KINDS
            ]
        )

    preview.render_grid(preview.sheet_path(out, f"reaper_{calling}_armoury", cel_look), armoury, tile=(360, 540), cel_look=cel_look)


# Weapons -------------------------------------------------------------------


def weapons(roots, out, cel_look=None, lineup=True):
    """The sheets of the weapons under `roots`, each modelled with its grip
    at the origin:

    - `weapon_<kind>.png`: one weapon alone, framed on itself, from the
      front, three-quarter, side and back, and in silhouette.
    - `weapons.png`: all of them side by side at one scale, from the front
      and in silhouette, so their sizes can be compared.
    """

    def alone(chosen):
        for weapon in roots:
            hide_tree(weapon, weapon is not chosen)

    for weapon in roots:
        kind = snake(base(weapon).removeprefix("weapon_"))
        preview.render_grid(
            preview.sheet_path(out, f"weapon_{kind}", cel_look),
            [preview.row(kind, lambda weapon=weapon: alone(weapon))],
            tile=(320, 640),
            cel_look=cel_look,
        )

    if not lineup or len(roots) < 2:
        return

    # Side by side, each as wide as it is, for the lineup only.
    placed = [weapon.location.copy() for weapon in roots]
    widths = []

    for weapon in roots:
        alone(weapon)
        bpy.context.view_layer.update()
        corners = preview.visible_corners(())
        widths.append(max(corner.x for corner in corners) - min(corner.x for corner in corners))

    gap = max(widths) * 0.25
    x = 0.0

    for weapon, width in zip(roots, widths):
        weapon.location.x = x + width / 2
        x += width + gap

    for weapon in roots:
        hide_tree(weapon, False)

    try:
        preview.render_grid(
            preview.sheet_path(out, "weapons", cel_look),
            [preview.row("all eight", None, ("front",)), preview.row("all eight", None, ("silhouette",))],
            tile=(1800, 700),
            cel_look=cel_look,
        )
    finally:
        for weapon, location in zip(roots, placed):
            weapon.location = location


def snake(name):
    """A `WeaponKind` name as its script's name: `lanternPole` is
    `lantern_pole`."""

    return "".join(f"_{letter.lower()}" if letter.isupper() else letter for letter in name)


# The first-person view -----------------------------------------------------


def first_person(moments, out, name="viewmodel", cel_look=None):
    """`viewmodel.png`: a row for each moment of the first-person view,
    showing the meshes that moment shows, from four sides and in
    silhouette. `moments` maps a moment's name to its meshes' names."""

    meshes = [obj for obj in bpy.data.objects if obj.type == "MESH"]
    known = {base(obj) for obj in meshes}

    def show(names):
        missing = set(names) - known

        if missing:
            raise ValueError(f"{name}: no mesh called {', '.join(sorted(missing))}")

        for obj in meshes:
            obj.hide_render = base(obj) not in names

    rows = [preview.row(moment, lambda names=names: show(names)) for moment, names in moments.items()]
    # An arm seen from its end is a fraction of its length from the side, so
    # each view is framed on its own.
    preview.render_grid(preview.sheet_path(out, name, cel_look), rows, tile=(420, 420), elevation=12.0, cel_look=cel_look, same_scale=False)


def whole(out, name, cel_look=None, tile=(420, 420)):
    """A one-row sheet of everything in the scene."""

    preview.render_grid(preview.sheet_path(out, name, cel_look), [preview.row(name, None)], tile=tile, elevation=12.0, cel_look=cel_look)


# Creatures -----------------------------------------------------------------


def husk(name, looks, out, traits, cel_look=None):
    """`husk_<vessel>.png`: a row for each thing the creature can wear,
    dressed in one NFT's look, from four sides and in silhouette."""

    if looks:
        rows = [preview.row(f"{name} trait {trait}", None, look={**looks[trait % len(looks)], "trait": trait}) for trait in range(1, traits + 1)]
    else:
        rows = [preview.row(name, None)]

    preview.render_grid(preview.sheet_path(out, f"husk_{name}", cel_look), rows, tile=(460, 460), cel_look=cel_look, skip=("shadow",))


# Clips ---------------------------------------------------------------------


def _animations():
    for obj in bpy.data.objects:
        if obj.animation_data is not None:
            yield obj.animation_data

        if obj.type == "MESH" and obj.data.shape_keys is not None and obj.data.shape_keys.animation_data is not None:
            yield obj.data.shape_keys.animation_data


def clip_frames(clip):
    """The first and last frame of an imported clip, from its tracks."""

    strips = [strip for animation in _animations() for track in animation.nla_tracks if track.name == clip for strip in track.strips]

    if not strips:
        raise ValueError(f"the model has no clip called {clip!r}")

    return min(strip.frame_start for strip in strips), max(strip.frame_end for strip in strips)


def play(clip, phase):
    """Plays only `clip` (faces included) and goes to `phase` of the way
    through it, 0 at its start and 1 at its end. Returns the frame."""

    first, last = clip_frames(clip)

    for animation in _animations():
        animation.action = None

        for track in animation.nla_tracks:
            track.mute = track.name != clip

    frame = first + phase * (last - first)
    bpy.context.scene.frame_set(int(frame), subframe=frame % 1)

    return frame


def reaper_for_motion(models, calling):
    """Imports a built reaper for a motion review, at rest, with every
    weapon in its hand, hidden. Returns its root and the weapons; `dress`
    then picks what it wears and holds."""

    core.reset_scene()
    import_glb(os.path.join(models, f"reaper_{calling}.glb"))
    rest_pose()
    root = bpy.data.objects["reaper"]

    return root, hold_weapons(root)


def held_kind(spec, calling, wanted=None):
    """The weapon a clip is reviewed with: none for an unarmed clip, the
    one a finisher needs, or the class's own."""

    if spec.get("unarmed", False):
        return None

    return wanted or {"finisherImpale": "spear", "finisherReap": "scythe"}.get(spec["name"], CLASS_WEAPONS[calling])


def motion_spec(motions, clip):
    with open(motions, encoding="utf-8") as source:
        specs = json.load(source)

    found = next((spec for spec in specs if spec["name"] == clip), None)

    if found is None:
        raise SystemExit(f"No clip called {clip!r} in {motions}; the clips are {', '.join(spec['name'] for spec in specs)}")

    return found
