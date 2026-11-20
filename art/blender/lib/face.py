"""Spectral facial glyphs with baked expression morphs on every head variant."""

import math

import bpy
from mathutils import Vector

from . import core

EXPRESSIONS = ("joy", "sorrow", "anger", "smirk", "awe", "fear")
CLIPS = {
    "laugh": "joy",
    "cry": "sorrow",
    "rage": "anger",
    "fear": "fear",
    "bow": "sorrow",
    "victory": "joy",
    "gigachad": "smirk",
    "sigma": "smirk",
    "shrug": "smirk",
    "absoluteCinema": "awe",
    "tPose": "smirk",
    "animeCharge": "anger",
    "finisherSoulCrush": "anger",
}


def glyph(expression, narrow=1.0, curve=1.8):
    """A fixed topology: two curved spectral eyes, an expressive mouth glyph, and tear trails.

    At rest (`neutral`), the mouth collapses invisibly so every reaper reads
    with dignified dark-fantasy glowing eyes alone; during emotes (`joy`,
    `sorrow`, `anger`, `smirk`, `awe`, `fear`), the mouth and tears blossom.
    """
    vertices, faces = [], []
    # Eye height, tilt, mouth opening, smile curvature, asymmetry, tears, mouth_width, mouth_band.
    settings = {
        "neutral": (0.0085, 0.14, 0.0001, 0.0, 0.0, 0.0001, 0.0006, 0.00008),
        "joy": (0.0045, 0.24, 0.019, 0.016, 0.0, 0.0001, 0.042, 0.0022),
        "sorrow": (0.0065, -0.34, 0.005, -0.015, 0.0, 0.052, 0.036, 0.0020),
        "anger": (0.0045, 0.42, 0.006, -0.008, 0.0, 0.0001, 0.038, 0.0022),
        "smirk": (0.0055, 0.16, 0.0035, 0.008, 0.016, 0.0001, 0.038, 0.0020),
        "awe": (0.0135, 0.0, 0.014, 0.0, 0.0, 0.0001, 0.020, 0.0022),
        "fear": (0.0165, -0.14, 0.020, -0.004, 0.0, 0.0001, 0.022, 0.0022),
    }
    eye, tilt, opening, smile, skew, tears, mouth_width, mouth_band = settings[expression]

    def hug(x):
        # Curve the spectral glyph gently around the mask/hood/skull plane.
        return curve * x * x

    for side in (-1, 1):
        start = len(vertices)
        cx = side * 0.041
        vertices.append((cx, hug(cx), 0.0))
        for step in range(24):
            angle = math.tau * step / 24
            ca = math.cos(angle)
            sa = math.sin(angle)
            # Tapered almond/ember eye silhouette instead of a blunt oval.
            x_off = 0.026 * ca
            pinch = (1.0 - 0.55 * ca * ca) ** 0.7
            z = eye * narrow * sa * pinch + x_off * side * tilt
            if expression == "smirk" and side == -1:
                z += 0.006
            px = cx + x_off
            vertices.append((px, hug(px), z))
        for step in range(24):
            faces.append((start, start + 1 + step, start + 1 + (step + 1) % 24))

    start = len(vertices)
    for edge in (1, -1):
        for step in range(40):
            angle = math.tau * step / 40
            x = (mouth_width + edge * mouth_band) * math.cos(angle)
            z = -0.064 + (opening + edge * mouth_band) * math.sin(angle)
            z += smile * (x / max(mouth_width, 1e-4)) ** 2 + skew * x / max(mouth_width, 1e-4)
            vertices.append((x, hug(x), z))
    for step in range(40):
        next_step = (step + 1) % 40
        faces.append((start + step, start + next_step, start + 40 + next_step, start + 40 + step))

    for side in (-1, 1):
        start = len(vertices)
        x = side * 0.043
        width = 0.0026 if expression == "sorrow" else 0.00001
        y_top = hug(x)
        y_bot = hug(x + side * 0.008)
        vertices.extend(((x - width, y_top, -0.015), (x + width, y_top, -0.015), (x + side * 0.008, y_bot, -0.015 - tears)))
        faces.append((start, start + 2, start + 1))
    return vertices, faces


def build(root, calling):
    """Project light onto each identity's mask, veil, hood or carried skull."""
    nodes = {node.name: node for node in root.children_recursive}
    if calling == "hollow":
        parents = [nodes["lantern_head"]]
        bpy.data.objects.remove(nodes["lantern_flame"], do_unlink=True)
    else:
        parents = [nodes[f"head_{i}"] for i in range(3)]
        for node in list(root.children_recursive):
            if node.name.startswith("eyes_"):
                bpy.data.objects.remove(node, do_unlink=True)
    bpy.context.view_layer.update()
    result = []
    for index, parent in enumerate(parents):
        # Local coordinates of the eye line.
        origin = Vector((0, -0.15, 0.152))
        if calling == "ferryman":
            origin = Vector((0, -0.22, 0.095))
        elif calling == "weigher":
            origin = Vector((0, -0.16, 0.188))
        elif calling == "chooser":
            origin.z = (0.162, 0.154, 0.191)[index]
        elif calling == "hollow":
            origin = Vector((0, -0.11, -0.010))
        narrow = 0.62 if calling in ("chooser", "weigher") else 0.92
        curve = 2.4 if calling in ("weigher", "hollow") else 1.6

        # On Chooser helm_0, ignore the iron nasal bar when sampling eye depth so
        # the glowing eyes gleam hauntingly from behind the nasal guard rather
        # than floating outside the helmet.
        prefixes = ("void_", "mask_", "veil_", "cowl_", "lantern_skull")
        if calling == "chooser" and index == 2:
            prefixes = ("void_", "helm_")
        surfaces = [
            node
            for node in parent.children_recursive
            if node.type == "MESH" and node.name.startswith(prefixes)
        ]

        def depth(point):
            local = origin + Vector(point)
            world = parent.matrix_world @ Vector((local.x, -2, local.z))
            direction = parent.matrix_world.to_3x3() @ Vector((0, 1, 0))
            closest = None
            for surface in surfaces:
                inverse = surface.matrix_world.inverted()
                hit, at, _, _ = surface.ray_cast(inverse @ world, inverse.to_3x3() @ direction)
                if hit:
                    at = parent.matrix_world.inverted() @ (surface.matrix_world @ at)
                    if closest is None or at.y < closest:
                        closest = at.y
            return closest if closest is not None else origin.y

        # Sample across the ocular zone so the glyph sits snugly right on the
        # mask/void/socket plane without being pushed far out by lower wraps.
        clearance = 0.015 if calling == "harvester" else (0.005 if calling in ("weigher", "mourner") else 0.007)
        if calling == "hollow":
            # Seat embers inside the recessed skull sockets cleanly.
            origin.y = min(depth((-0.036, 0, 0)), depth((0.036, 0, 0))) - 0.005
        else:
            origin.y = min(depth(point) for point in ((0, 0, 0), (-0.045, 0, 0), (0.045, 0, 0))) - clearance

        vertices, faces = glyph("neutral", narrow, curve)
        mesh = bpy.data.meshes.new(f"face_{index}")
        mesh.from_pydata(vertices, [], faces)
        mesh.update()
        obj = bpy.data.objects.new(f"face_{index}", mesh)
        bpy.context.collection.objects.link(obj)
        obj.parent = parent
        obj.location = origin
        mesh.materials.append(core.material("accent"))
        obj.shape_key_add(name="Basis")
        for expression in EXPRESSIONS:
            shape = obj.shape_key_add(name=expression)
            points, _ = glyph(expression, narrow, curve)
            for vertex, point in zip(shape.data, points):
                vertex.co = point
            shape.value = 0
        result.append(obj)
    return result


def weight(clip, expression, phase):
    """Anticipate, hold the expression, then return to neutral on recovery."""
    if CLIPS.get(clip) != expression:
        return 0.0
    ramp = min(1, phase / 0.2, (1 - phase) / 0.18)
    strength = ramp * ramp * (3 - 2 * ramp)
    if clip == "laugh":
        strength *= 0.82 + 0.18 * math.sin(phase * math.tau * 5)
    if clip == "bow":
        strength *= 0.35
    return strength


def bake(faces, specs):
    """NLA track names merge face weights into the same GLB body clips."""
    for obj in faces:
        data = obj.data.shape_keys
        data.animation_data_create()
        offset = 1
        for spec in specs:
            frames = round(spec["seconds"] * 30)
            for frame in range(frames + 1):
                for expression in EXPRESSIONS:
                    shape = data.key_blocks[expression]
                    shape.value = weight(spec["motion"], expression, frame / frames)
                    shape.keyframe_insert(data_path="value", frame=frame)
            action = data.animation_data.action
            action.name = f'{spec["name"]}_{obj.name}'
            slot = data.animation_data.action_slot
            track = data.animation_data.nla_tracks.new()
            track.name = spec["name"]
            strip = track.strips.new(spec["name"], offset, action)
            strip.action_slot = slot
            strip.extrapolation = "NOTHING"
            data.animation_data.action = None
            for shape in data.key_blocks:
                shape.value = 0
            offset += frames + 5
