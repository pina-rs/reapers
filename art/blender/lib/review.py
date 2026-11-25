"""The review sheets of every kind of model, shared by the drivers (which
render them as they build, with `--preview`) and by `review.py` (which
renders them again from models already built, without rebuilding).

Every sheet is rendered by `preview.render_grid`, which frames each row on
what it shows. See art/blender/README.md for what each sheet holds.
"""

import importlib
import json
import math
import os
import re
import struct

import bpy
from mathutils import Matrix, Quaternion, Vector

from . import cel, core, preview

# The rate every clip is baked at (`motion.py`). A model is imported at it,
# so each key lands on a whole frame, as the game samples it.
CLIP_RATE = 30

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

# A class file holds four robes and fewer heads and mantles. A row of a
# class's sheet shows robe `n` with head and mantle `n`, or the last of them
# where there are fewer (`variant_counts`).
VARIANTS = 4
VARIANT_PARTS = ("head_", "mantle_", "robe_", "face_")

# What a reaper's sheets show it in: the first key of its idle clip, by form.
# A reaper file's own rest transforms put every joint at its parent's
# origin, which is no pose at all, and the game always plays a clip.
STANCES = {"spirit": "idle", "humanoid": "humanoid_idle"}
FORMS = ("spirit", "humanoid")


# The sheets of a reaper class, by name: `--sheets` picks some.
REAPER_SHEETS = ("spirit", "humanoid", "armoury")


def looks(with_cel, only_cel=False, scene=None):
    """The looks a review renders: the normal one, and Second Ink from
    `art/blender/cel.toml` as well when asked for (or alone, `only_cel`),
    every sheet in `scene` when one is named. Each is the palette; a sheet
    narrows it to its kind with `cel.look`."""

    if only_cel:
        return [cel.with_scene(cel.load(), scene)]

    return [None, cel.with_scene(cel.load(), scene)] if with_cel else [None]


def import_glb(path):
    """Imports a built model into the scene at rest (see `rest_pose`), with
    its clips' keys on whole frames at `CLIP_RATE`. Returns the objects it
    added."""

    scene = bpy.context.scene
    scene.render.fps = CLIP_RATE
    scene.render.fps_base = 1.0
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=path)
    rest_pose(path)

    return [obj for obj in bpy.data.objects if obj not in before]


def rest_pose(path):
    """Stops every clip and puts every node and morph back at the transform
    and weights `path` exported it with: its bind pose, which no clip has
    moved.

    Muting the clips is not enough: the importer makes the first clip it
    reads each node's active action and writes that clip's first frame into
    the node, so a node left alone would show that frame instead, and which
    clip that is would hang on the order of the file's clips. A pose to show
    is then played on purpose: see `STANCES` and `play`."""

    for animation in _animations():
        animation.action = None

        for track in animation.nla_tracks:
            track.mute = True

    bpy.context.scene.frame_set(0)
    document = _gltf(path)
    meshes = document.get("meshes", [])
    names = [node.get("name") for node in document["nodes"]]
    # A node no clip moves is where the importer put it already.
    animated = sorted({channel["target"]["node"] for clip in document.get("animations", []) for channel in clip["channels"]})

    for node in (document["nodes"][index] for index in animated):
        if names.count(node.get("name")) != 1:
            raise ValueError(f"{path}: more than one node is called {node.get('name')!r}, so its object cannot be found")

        obj = bpy.data.objects.get(node["name"])

        if obj is None:
            raise ValueError(f"{path}: the importer made no object for node {node['name']!r}")

        if "matrix" in node:
            raise ValueError(f"{path}: node {node['name']!r} is a matrix, not a translation, rotation and scale")

        # glTF is +Y up and Blender +Z up: (x, y, z) is (x, -z, y).
        x, y, z = node.get("translation", (0.0, 0.0, 0.0))
        obj.location = (x, -z, y)
        x, y, z, w = node.get("rotation", (0.0, 0.0, 0.0, 1.0))
        obj.rotation_mode = "QUATERNION"
        obj.rotation_quaternion = Quaternion((w, x, -z, y))
        x, y, z = node.get("scale", (1.0, 1.0, 1.0))
        obj.scale = (x, z, y)

        if "mesh" in node and obj.type == "MESH" and obj.data.shape_keys is not None:
            keys = obj.data.shape_keys.key_blocks[1:]
            weights = meshes[node["mesh"]].get("weights", [0.0] * len(keys))

            for key, weight in zip(keys, weights, strict=True):
                key.value = weight

    bpy.context.view_layer.update()


def _gltf(path):
    """The JSON of a binary glTF file."""

    with open(path, "rb") as source:
        header = source.read(20)
        magic, _, _, length, kind = struct.unpack("<4sIII4s", header)

        if magic != b"glTF" or kind != b"JSON":
            raise ValueError(f"{path} is not a binary glTF file")

        return json.loads(source.read(length))


def base(node):
    return node.name.split(".")[0]


def find(node, name):
    """The node under `node` (or `node` itself) called `name`."""

    if base(node) == name:
        return node

    return next((child for child in node.children_recursive if base(child) == name), None)


def variant_counts(root):
    """How many of each varying part (`VARIANT_PARTS`) a reaper holds: its
    heads, mantles, robes and faces, by prefix."""

    counts = dict.fromkeys(VARIANT_PARTS, 0)

    for node in root.children_recursive:
        name = base(node)

        for prefix in counts:
            if name.startswith(prefix) and name[len(prefix) :].isdigit():
                counts[prefix] = max(counts[prefix], int(name[len(prefix) :]) + 1)

    return counts


def shown_variant(counts, prefix, variant):
    """Which of a part a genome of `variant` shows: the last there is, when
    there are fewer than `variant + 1`."""

    if counts[prefix] == 0:
        raise ValueError(f"the reaper has no {prefix.rstrip('_')}")

    return min(variant, counts[prefix] - 1)


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

    # A sheet drawn again in another look builds its weapons again: hide the
    # ones an earlier look built, or the last it showed stays in every sheet.
    for earlier in socket.children:
        if earlier.name.startswith("weapon_"):
            hide_tree(earlier, True)

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

    counts = {prefix: count for prefix, count in variant_counts(root).items() if count > 0}

    for node in root.children_recursive:
        name = base(node)

        if name in ("spirit_form", "humanoid_form"):
            hide_tree(node, name != f"{form}_form")

    for node in root.children_recursive:
        name = base(node)

        for prefix in counts:
            if name.startswith(prefix) and name[len(prefix) :].isdigit():
                hide_tree(node, int(name[len(prefix) :]) != shown_variant(counts, prefix, variant))

        if name.startswith("robe_") and node.type == "MESH" and node.data.shape_keys is not None:
            keys = node.data.shape_keys.key_blocks

            for key in keys[1:]:
                key.value = 0.0

            garment = f"humanoid_{clothing}" if form == "humanoid" else "short_robe" if clothing == "tunic" else None

            if garment is not None:
                keys[garment].value = 1.0

    for kind, held in (weapons or {}).items():
        hide_tree(held, kind != weapon)


def play_stance(form):
    """Puts an imported reaper in the first key of its idle clip for `form`."""

    play(STANCES[form], 0.0)


def stand(root, variant, form, weapons=None, weapon=None, stance=play_stance):
    """`dress`es a reaper and puts it in its stance for `form` (`STANCES`):
    `stance(form)` poses it, from its clips by default."""

    dress(root, variant, form, weapons, weapon)
    stance(form)


def reaper(glb, calling, out, cel_look=None, sheets=REAPER_SHEETS):
    """The sheets of one reaper class, from its built model: see
    `reaper_sheets`."""

    core.reset_scene()
    import_glb(glb)
    reaper_sheets(bpy.data.objects["reaper"], calling, out, cel_look, sheets=sheets)


def reaper_sheets(root, calling, out, cel_look=None, stance=play_stance, sheets=REAPER_SHEETS):
    """The sheets of one reaper class under `root`, in its idle stance
    (`stance(form)` poses it: from its clips for a built model, or directly
    for one only just built, `motion.hold`):

    - `reaper_<class>_spirit.png` and `reaper_<class>_humanoid.png`: a row
      for each variant (robe, head and mantle), holding the class's weapon,
      from the front, three-quarter, side and back, and in silhouette.
    - `reaper_<class>_armoury.png`: the class holding each of the eight
      weapons, then each in silhouette.

    `sheets` names which (`REAPER_SHEETS`). In the cel look a reaper prints
    in its class's light.
    """

    unknown = set(sheets) - set(REAPER_SHEETS)

    if unknown:
        raise ValueError(f"no reaper sheet called {', '.join(sorted(unknown))}; there are {', '.join(REAPER_SHEETS)}")

    cel_look = cel.look(cel_look, "reaper", cel.class_light(cel_look, calling)) if cel_look else None
    weapons = hold_weapons(root)
    own = CLASS_WEAPONS[calling]
    counts = variant_counts(root)

    def label(form, variant):
        head, mantle = (shown_variant(counts, prefix, variant) for prefix in ("head_", "mantle_"))

        return f"{calling} {form}, robe {variant}, head {head}, mantle {mantle}"

    for form in FORMS:
        if form not in sheets:
            continue

        rows = [preview.row(label(form, variant), lambda variant=variant, form=form: stand(root, variant, form, weapons, own, stance)) for variant in range(VARIANTS)]
        preview.render_grid(preview.sheet_path(out, f"reaper_{calling}_{form}", cel_look), rows, tile=(400, 600), cel_look=cel_look)

    if "armoury" not in sheets:
        return

    armoury = []

    for view in ("threequarter", "silhouette"):
        armoury.append(
            [
                {"label": f"{kind} {view}", "setup": lambda kind=kind: stand(root, 0, "spirit", weapons, kind, stance), "view": view, "look": None}
                for kind in WEAPON_KINDS
            ]
        )

    preview.render_grid(preview.sheet_path(out, f"reaper_{calling}_armoury", cel_look), armoury, tile=(360, 540), cel_look=cel_look)


# Weapons -------------------------------------------------------------------


def weapons(roots, out, cel_look=None, lineup=True, singles=True):
    """The sheets of the weapons under `roots`, each modelled with its grip
    at the origin:

    - `weapon_<kind>.png` (with `singles`): one weapon alone, framed on
      itself, from the front, three-quarter, side and back, and in
      silhouette.
    - `weapons.png` (with `lineup`): all of them side by side at one scale,
      from the front and in silhouette, so their sizes can be compared.
    """

    def alone(chosen):
        for weapon in roots:
            hide_tree(weapon, weapon is not chosen)

    # Each weapon prints in its owner's light (cel.toml [weapons]).
    for weapon in roots:
        kind = snake(base(weapon).removeprefix("weapon_"))

        if cel_look and kind in cel_look["weapons"]:
            weapon[cel.LIGHT_PROPERTY] = cel.weapon_light(cel_look, kind)

    for weapon in roots if singles else ():
        kind = snake(base(weapon).removeprefix("weapon_"))
        preview.render_grid(
            preview.sheet_path(out, f"weapon_{kind}", cel_look),
            [preview.row(kind, lambda weapon=weapon: alone(weapon))],
            tile=(320, 640),
            cel_look=cel.look(cel_look, "weapon", cel.weapon_light(cel_look, kind)) if cel_look else None,
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
    heights = []

    for weapon, width in zip(roots, widths):
        alone(weapon)
        bpy.context.view_layer.update()
        corners = preview.visible_corners(())
        heights.append(max(corner.z for corner in corners) - min(corner.z for corner in corners))
        weapon.location.x = x + width / 2
        x += width + gap

    for weapon in roots:
        hide_tree(weapon, False)

    try:
        preview.render_grid(
            preview.sheet_path(out, "weapons", cel_look),
            [preview.row("all eight", None, ("front",)), preview.row("all eight", None, ("silhouette",))],
            tile=(1800, 700),
            cel_look=cel.look(cel_look, "weapon", cel.weapon_light(cel_look, "scythe")) if cel_look else None,
            figure_size=max(heights),
        )
    finally:
        for weapon, location in zip(roots, placed):
            weapon.location = location


def snake(name):
    """A `WeaponKind` name as its script's name: `lanternPole` is
    `lantern_pole`."""

    return "".join(f"_{letter.lower()}" if letter.isupper() else letter for letter in name)


# The first-person view -----------------------------------------------------


# Where the game's rig puts the first-person parts. The rest pose, the holds
# and every named number are read from the rig's own source, so the sheet
# follows the game; the few numbers a pose works out inside a function are
# copied into `moment_stages`, which says where each comes from.
RIG = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "crates", "reapers_app", "src", "render")
RIG_FILES = ("viewmodel.rs", "arms.rs")
RIG_CONSTANTS = ("REST", "REST_TURN", "REST_WRIST", "ELBOW", "WRIST", "TRIGGER", "SCYTHE_HEAD", "RAISED", "RELOAD_LIFT", "RELOAD_TIP", "RELOAD_ROLL", "RELOAD_IN", "LEFT_RISE")

# The part the right hand holds for each archetype (`weapon_part`).
HELD = {"Gun": "weapon_gravePistol", "Blade": "weapon_boneKnife", "Heavy": "weapon_scythe"}

# The arm model faces away from the eye (`FLIP`): a half turn about +Y.
FLIP = Quaternion((0.0, 0.0, 1.0, 0.0))


def rig():
    """The first-person rig's numbers, from `RIG_FILES`: its named constants,
    its camera's vertical field of view (`fov`, radians) and each
    archetype's hold (`holds`)."""

    source = ""

    for file in RIG_FILES:
        with open(os.path.join(RIG, file), encoding="utf-8") as text:
            source += text.read()

    def value(text):
        repeated = re.fullmatch(r"\[(.+);\s*(\d+)\]", text)

        if repeated:
            return (float(repeated[1]),) * int(repeated[2])

        if text.startswith("["):
            return tuple(float(number) for number in text[1:-1].split(","))

        return float(text)

    def found(pattern, what):
        match = re.search(pattern, source, re.S)

        if match is None:
            raise ValueError(f"no {what} in {', '.join(RIG_FILES)} under {os.path.normpath(RIG)}: the review stages the first-person view from it")

        return match

    numbers = {name: value(found(rf"\bconst {name}: [^=]+= (\[[^\]]*\]|-?[\d._]+);", f"const {name}")[1]) for name in RIG_CONSTANTS}
    numbers["fov"] = math.radians(float(found(r"\bconst VIEWMODEL_FOV: f32 = ([\d._]+) \* std::f32::consts::PI / 180\.0;", "VIEWMODEL_FOV")[1]))
    numbers["holds"] = {
        archetype: {field: value(text) for field, text in re.findall(r"(\w+): (\[[^\]]*\]|-?[\d._]+)", found(rf"Archetype::{archetype}\b[^=]*=>\s*\{{\s*Hold \{{(.*?)\}}", f"hold for {archetype}")[1])}
        for archetype in HELD
    }

    return numbers


def moment_stages(numbers):
    """How each moment of `viewmodel.MOMENTS` is staged: the archetype in
    hand, the right arm's offsets from rest (`ArmPose`) and the left arm's
    place in the view (`LeftPose`), when it shows."""

    # arms.rs `reload` at the top of its dip, the left hand all the way up
    # with the spare drum (`LEFT_IN` at 1), before it carries it across.
    reload_arm = {"pitch": numbers["RELOAD_LIFT"], "wrist_pitch": -numbers["RELOAD_TIP"], "wrist_roll": -numbers["RELOAD_ROLL"], "wrist_yaw": 0.1, "x": -numbers["RELOAD_IN"], "y": 0.02}
    reload_left = {"at": (-0.06 + 0.01, -0.42 + numbers["LEFT_RISE"], 0.52), "pitch": 0.9 - 0.2, "yaw": 0.55, "roll": -0.5}
    # arms.rs `raised`, all the way up: the orb held while a throw is timed.
    raised_arm = {"x": 0.008, "y": -0.015}
    raised_left = {"at": numbers["RAISED"], "pitch": 0.35, "yaw": 0.35, "roll": -0.4}

    return {
        "fire": ("Gun", {}, None),
        "reload": ("Gun", reload_arm, reload_left),
        "throw": ("Gun", raised_arm, raised_left),
        "throw with the scythe": ("Heavy", raised_arm, raised_left),
        "stab": ("Blade", {}, None),
        "swing": ("Heavy", {}, None),
    }


def _view_turn(yaw, pitch, roll):
    """`turn::view_turn`: a turn in the view's sense."""

    return Quaternion((0, 1, 0), -yaw) @ Quaternion((1, 0, 0), pitch) @ Quaternion((0, 0, 1), -roll)


def _view(point):
    """From the arms' frame (z away from the eye) to the view's."""

    return Vector((point[0], point[1], -point[2]))


def _then(frame, local):
    """`Frame::then`: `local`, a frame inside `frame`, seen from outside it."""

    at, turn, size = frame

    return (_point(frame, local[0]), turn @ local[1], size * local[2])


def _point(frame, local):
    at, turn, size = frame

    return at + turn @ (Vector(local) * size)


def _at(point):
    return (Vector(point), Quaternion(), 1.0)


def place(numbers, archetype, arm, left):
    """`viewmodel::place` for a still pose: the offset of every part the rig
    moves, as `(at, turn, size)` in glTF's frame, by node. `arm` holds the
    right arm's offsets from rest and `left` the left arm's place, or None."""

    hold = numbers["holds"][archetype]
    offset = lambda key: arm.get(key, 0.0)  # noqa: E731
    rest_turn = _view_turn(*(rest + extra for rest, extra in zip(numbers["REST_TURN"], hold["turn"]))) @ FLIP
    elbow = rest_turn @ Vector(numbers["ELBOW"])
    turn = _view_turn(offset("yaw"), offset("pitch"), offset("roll"))
    pivot = _view(Vector(numbers["REST"]) + Vector(hold["rest"]) + Vector((offset("x"), offset("y"), offset("z")))) + elbow
    right = (pivot - turn @ elbow, turn @ rest_turn, 1.0)
    wrist_view = _view_turn(*(rest + extra + offset(key) for rest, extra, key in zip(numbers["REST_WRIST"], hold["wrist"], ("wrist_yaw", "wrist_pitch", "wrist_roll"))))
    wrist = right[1].inverted() @ wrist_view @ right[1]
    held = (Vector(), Quaternion((1, 0, 0), hold["lean"]) @ Quaternion((0, 1, 0), hold["twist"]), hold["scale"])
    hand = _then(_then(right, _at(numbers["WRIST"])), (Vector(), wrist, 1.0))
    socket = _then(hand, _at(-Vector(numbers["WRIST"])))
    weapon = _then(_then(_then(socket, _at(numbers["TRIGGER"])), _at(-Vector(numbers["TRIGGER"]))), held)
    # A heavy swing's trail: an arc about the elbow, its bright end at the
    # head (`ViewmodelRig::trail`).
    head = _point(weapon, numbers["SCYTHE_HEAD"])
    reach = head - pivot
    trail = (Vector((pivot.x, head.y, pivot.z)), Quaternion((0, 1, 0), math.atan2(-reach.z, reach.x) - math.radians(120)), max(math.hypot(reach.x, reach.z), 0.05))
    offsets = {"vm_right": right, "vm_wrist_r": (Vector(), wrist, 1.0), HELD[archetype]: held, "vm_trail": trail}

    if left is not None:
        offsets["vm_left"] = (_view(left["at"]), _view_turn(left["yaw"], left["pitch"], left["roll"]) @ FLIP, 1.0)

    return offsets


def blender_offset(frame):
    """An offset in glTF's frame (+Y up) as a matrix in Blender's (+Z up)."""

    at, turn, size = frame
    rotation = Quaternion((turn.w, turn.x, -turn.z, turn.y))

    return Matrix.Translation((at.x, -at.z, at.y)) @ rotation.to_matrix().to_4x4() @ Matrix.Scale(size, 4)


def first_person(moments, out, name="viewmodel", cel_look=None):
    """The sheets of the first-person view:

    - `viewmodel.png`: a row for each moment, its parts placed where the
      game's rig places them (`rig`, `place`), seen first through the game's
      own camera at the eye, at a phone's shape, then from four sides and in
      silhouette. `moments` maps a moment's name to its meshes' names.
    - `viewmodel_parts.png`: every part alone, at rest, framed on itself, so
      what the hand hides (the orb in the palm, the spare drum) is seen too.
    """

    numbers = rig()
    stages = moment_stages(numbers)
    cel_look = cel.look(cel_look, "first_person", cel.class_light(cel_look, cel_look["sheets"]["first_person_class"])) if cel_look else None
    unstaged = sorted(set(moments) - set(stages))

    if unstaged:
        raise ValueError(f"{name}: no staging for {', '.join(unstaged)}; add it to review.moment_stages")

    meshes = [obj for obj in bpy.data.objects if obj.type == "MESH"]
    known = {base(obj) for obj in meshes}
    rest = {obj: obj.matrix_basis.copy() for obj in bpy.data.objects}

    def show(names):
        missing = set(names) - known

        if missing:
            raise ValueError(f"{name}: no mesh called {', '.join(sorted(missing))}")

        for obj in meshes:
            obj.hide_render = base(obj) not in names

    def stage(moment):
        for obj, basis in rest.items():
            obj.matrix_basis = basis

        show(moments[moment])

        for part, frame in place(numbers, *stages[moment]).items():
            obj = bpy.data.objects.get(part)

            if obj is None:
                raise ValueError(f"{name}: the rig moves {part!r}, which is not in the model")

            obj.matrix_basis = rest[obj] @ blender_offset(frame)

    # The game's camera sits at the eye, looking down glTF's -Z with +Y up:
    # Blender's +Y with +Z up.
    eye = (Vector(), Quaternion((1, 0, 0), math.pi / 2), numbers["fov"])
    rows = [preview.row(moment, lambda moment=moment: stage(moment), ("eye", *preview.VIEWS), eye=eye) for moment in moments]
    parts = [preview.row(base(obj), lambda obj=obj: show([base(obj)])) for obj in sorted(meshes, key=base)]

    try:
        # An arm seen from its end is a fraction of its length from the
        # side, so each view is framed on its own.
        preview.render_grid(preview.sheet_path(out, name, cel_look), rows, tile=(380, 560), elevation=12.0, cel_look=cel_look, same_scale=False)

        for obj, basis in rest.items():
            obj.matrix_basis = basis

        preview.render_grid(preview.sheet_path(out, f"{name}_parts", cel_look), parts, tile=(320, 320), elevation=12.0, cel_look=cel_look, same_scale=False)
    finally:
        for obj, basis in rest.items():
            obj.matrix_basis = basis

        for obj in meshes:
            obj.hide_render = False


def effects(out, name="viewmodel_fx", cel_look=None):
    """`viewmodel_fx.png`: each effect on its own row, alone, drawn as the
    game draws it (unlit light added to the dark behind it), from the front,
    three-quarter, side and above. A silhouette of light says nothing, so
    there is none."""

    meshes = sorted((obj for obj in bpy.data.objects if obj.type == "MESH"), key=base)

    def show(only):
        for obj in meshes:
            obj.hide_render = obj is not only

    rows = [preview.row(base(obj), lambda obj=obj: show(obj), ("front", "threequarter", "side", "top")) for obj in meshes]
    # Dusk, not black: light adds to it and the shade darkens it, so both
    # read. The halos and the flare are cards the game turns to face the
    # camera, so they are lines from the side and above.
    dusk = (0.05, 0.055, 0.07)
    cel_look = cel.look(cel_look, "first_person", cel.class_light(cel_look, cel_look["sheets"]["first_person_class"])) if cel_look else None

    try:
        # Each effect is built at a unit size, and some are a flat card or a
        # line: framed on its own, every view fills its tile.
        preview.render_grid(preview.sheet_path(out, name, cel_look), rows, tile=(360, 360), elevation=12.0, cel_look=cel_look, same_scale=False, ground=dusk)
    finally:
        for obj in meshes:
            obj.hide_render = False


def whole(out, name, cel_look=None, tile=(420, 420), kind="first_person"):
    """A one-row sheet of everything in the scene, a model of `kind`."""

    if cel_look:
        calling = cel_look["sheets"]["first_person_class"]
        cel_look = cel.look(cel_look, kind, cel.class_light(cel_look, calling) if kind != "husk" else None)

    preview.render_grid(preview.sheet_path(out, name, cel_look), [preview.row(name, None)], tile=tile, elevation=12.0, cel_look=cel_look)


# Creatures -----------------------------------------------------------------


def husk(name, looks, out, traits, cel_look=None):
    """`husk_<vessel>.png`: a row for each thing the creature can wear,
    dressed in one NFT's look, from four sides and in silhouette."""

    if looks:
        rows = [preview.row(f"{name} trait {trait}", None, look={**looks[trait % len(looks)], "trait": trait}) for trait in range(1, traits + 1)]
    else:
        rows = [preview.row(name, None)]

    cel_look = cel.look(cel_look, "husk") if cel_look else None
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


# Lineups -------------------------------------------------------------------

# What `review.py --lineup` sets side by side, by name.
LINEUPS = {
    "reapers": tuple(f"reaper_{calling}" for calling in CLASS_WEAPONS),
    # Not the reliquary stack, a tower several storeys high: at one scale
    # the creatures would be specks beside it.
    "husks": ("husk_ape", "husk_bird", "husk_penguin", "husk_llama", "husk_robot"),
}

# The suffix a lineup gives every object of a figure already placed, so the
# next import's names are its own (`rest_pose` finds nodes by name).
PLACED = ".lineup"


def lineup(models, names, out, palette, scene=None, specs=None, stem="lineup"):
    """`<stem>.png` from built models: `names` in `models` (without
    `.glb`), all reapers or all husks, imported and set side by side
    (`lay_out`). A reaper stands in its idle stance in robe 0, holding its
    own weapon, in its class's light; a husk wears its collection's first
    look."""

    kinds = {"reaper" if name.startswith("reaper_") else "husk" if name.startswith("husk_") else None for name in names}

    if len(kinds) != 1 or None in kinds:
        raise SystemExit(f"a lineup is all reapers or all husks: {', '.join(names)}")

    kind = kinds.pop()
    core.reset_scene()
    figures = []

    for index, name in enumerate(names):
        step_aside(index)
        before = set(bpy.data.objects)
        import_glb(os.path.join(models, f"{name}.glb"))
        added = [obj for obj in bpy.data.objects if obj not in before]
        root = next(obj for obj in added if obj.parent is None and base(obj) in ("reaper", "husk"))

        if kind == "reaper":
            line_up_reaper(root, name.removeprefix("reaper_"), palette)
        else:
            vessel = name.removeprefix("husk_")
            line_up_husk(root, (specs or {}).get(vessel, {}).get("looks", []))

        figures.append((name, root))

    lay_out(figures, kind, out, palette, scene, stem)


def step_aside(index):
    """Renames every object already in the scene with `PLACED` and `index`,
    so the next figure built or imported has its names to itself: the
    drivers and `rest_pose` find nodes by name."""

    for obj in bpy.data.objects:
        if PLACED not in obj.name:
            obj.name = f"{obj.name}{PLACED}{index}"


def line_up_reaper(root, calling, palette, stance=play_stance):
    """Readies a reaper for a lineup: in robe 0 and its idle stance
    (`stance(form)`), holding its own weapon, printed in its class's light,
    and held still while the others are posed."""

    weapons = hold_weapons(root, (CLASS_WEAPONS[calling],))
    stand(root, 0, "spirit", weapons, CLASS_WEAPONS[calling], stance)
    root[cel.LIGHT_PROPERTY] = cel.class_light(palette, calling)
    _freeze(root)


def line_up_husk(root, looks):
    """Readies a husk for a lineup in the first of its collection's
    `looks`, held still."""

    _wear(root, looks[0] if looks else None)
    _freeze(root)


def lay_out(figures, kind, out, palette, scene=None, stem="lineup"):
    """`<stem>.png`: `figures` ((name, root) pairs, all of one `kind`)
    side by side at one scale, each turned to its three-quarter view and
    standing on one floor, in Second Ink and then in key-only silhouette
    with the 64-pixel strip beside it: the figures compared as a phone shows
    them together. `scene` is cel.toml's scene to light them in (the kind's
    own when None)."""

    sizes = []
    x = 0.0
    gap = None

    for name, root in figures:
        # Each turned to its three-quarter view, seen from the front, so the
        # row does not recede.
        root.rotation_mode = "XYZ"
        root.rotation_euler.z -= math.radians(preview.ANGLES["threequarter"])
        low, high = _bounds(root)
        width = high.x - low.x
        sizes.append(high.z - low.z)
        gap = gap if gap is not None else max(width, 0.4) * 0.35
        root.location.x += x - low.x

        # A reaper's floor is z = 0 already (a spirit floats above it); a
        # husk's is half a body below its middle, so it is stood on z = 0.
        if kind == "husk":
            root.location.z -= low.z

        x += width + gap
        bpy.context.view_layer.update()

    sheet = cel.look(palette, kind, None, scene)
    order = ", ".join(name.split("_", 1)[1] for name, _ in figures)
    rows = [preview.row(f"three-quarter  {order}", None, ("front",)), preview.row(f"{order}", None, ("silhouette",))]
    width = max(1600, min(3200, round(len(figures) * 420)))
    preview.render_grid(os.path.join(out, f"{stem}.png"), rows, tile=(width, 720), cel_look=sheet, figure_size=max(sizes), skip=("shadow", "seal_"))


def _wear(root, look):
    """Dresses a husk in an NFT's `look` for a lineup: its trait shown and
    the rest hidden, and its colours carried for the print."""

    for node in root.children_recursive:
        holder = node

        while holder is not None and not base(holder).startswith("trait_"):
            holder = holder.parent

        if holder is not None:
            node.hide_render = look is None or base(holder) != f"trait_{look['trait']}"

    if look is not None:
        root[cel.LOOK_PROPERTY] = {name: look[name] for name in ("body", "trim", "detail")}


def _freeze(root):
    """Keeps a figure in the pose its clips left it in, and takes its clips
    away, so posing the next figure leaves it still."""

    for node in [root, *root.children_recursive]:
        if node.animation_data is not None:
            node.animation_data_clear()

        if node.type == "MESH" and node.data.shape_keys is not None and node.data.shape_keys.animation_data is not None:
            node.data.shape_keys.animation_data_clear()


def _bounds(root):
    bpy.context.view_layer.update()
    corners = [
        node.matrix_world @ Vector(corner)
        for node in [root, *root.children_recursive]
        if node.type == "MESH" and not node.hide_render and not base(node).startswith(("shadow", "seal_"))
        for corner in node.bound_box
    ]

    return Vector(map(min, *corners)), Vector(map(max, *corners))
