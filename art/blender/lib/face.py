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


def glyph(expression, narrow=1.0):
    """A fixed topology: two eyes, a mouth outline and two tear trails."""
    vertices, faces = [], []
    # Eye height, tilt, mouth opening, smile curvature, asymmetry, tears.
    settings = {
        "neutral": (0.010, 0, 0.003, 0, 0, 0.0001),
        "joy": (0.004, 0.2, 0.022, 0.017, 0, 0.0001),
        "sorrow": (0.006, -0.32, 0.005, -0.016, 0, 0.052),
        "anger": (0.004, 0.38, 0.006, -0.007, 0, 0.0001),
        "smirk": (0.005, 0.12, 0.003, 0.008, 0.018, 0.0001),
        "awe": (0.014, 0, 0.014, 0, 0, 0.0001),
        "fear": (0.018, -0.12, 0.022, -0.004, 0, 0.0001),
    }
    eye, tilt, opening, smile, skew, tears = settings[expression]
    for side in (-1, 1):
        start = len(vertices)
        vertices.append((side * 0.043, 0, 0))
        for step in range(24):
            angle = math.tau * step / 24
            x = 0.028 * math.cos(angle)
            z = eye * narrow * math.sin(angle) + x * side * tilt
            if expression == "smirk" and side == -1:
                z += 0.008
            vertices.append((side * 0.043 + x, 0, z))
        for step in range(24):
            faces.append((start, start + 1 + step, start + 1 + (step + 1) % 24))

    start = len(vertices)
    mouth_width = 0.022 if expression in ("fear", "awe") else 0.047
    for edge in (1, -1):
        for step in range(40):
            angle = math.tau * step / 40
            x = (mouth_width + edge * 0.0018) * math.cos(angle)
            z = -0.068 + (opening + edge * 0.0015) * math.sin(angle)
            z += smile * (x / mouth_width) ** 2 + skew * x / mouth_width
            vertices.append((x, 0, z))
    for step in range(40):
        next_step = (step + 1) % 40
        faces.append((start + step, start + next_step, start + 40 + next_step, start + 40 + step))

    for side in (-1, 1):
        start = len(vertices)
        x = side * 0.046
        width = 0.003 if expression == "sorrow" else 0.00001
        vertices.extend(((x - width, 0, -0.016), (x + width, 0, -0.016), (x + side * 0.009, 0, -0.016 - tears)))
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
        # Local coordinates of the eye line. The ferryman's deeply recessed
        # hood carries its face further forward; the hollow uses skull space.
        origin = Vector((0, -0.15, 0.15))
        if calling == "ferryman":
            origin = Vector((0, -0.22, 0.09))
        elif calling == "weigher":
            origin = Vector((0, -0.16, 0.19))
        elif calling == "chooser":
            origin.z = (0.162, 0.154, 0.191)[index]
        elif calling == "hollow":
            origin = Vector((0, -0.11, -0.012))
        narrow = 0.65 if calling in ("chooser", "weigher") else 1.0
        surfaces = [
            node
            for node in parent.children_recursive
            if node.type == "MESH" and node.name.startswith(("void_", "wrap_", "mask_", "veil_", "cowl_", "helm_", "lantern_skull"))
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

        # Keep the glyph on one plane just in front of the face covering.
        # Projecting each animated vertex onto folds made the mouth buckle
        # and sent eyes onto the brim as their shape changed.
        origin.y = min(depth(point) for point in ((0, 0, 0), (-0.06, 0, 0), (0.06, 0, 0), (0, 0, -0.07), (0, 0, -0.10))) - 0.009

        vertices, faces = glyph("neutral", narrow)
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
            points, _ = glyph(expression, narrow)
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
                    shape.value = weight(spec["name"], expression, frame / frames)
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
