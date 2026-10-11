"""Shared anatomy with shaped legs and overlapping anatomical joints."""

import math
import bpy
import bmesh
from mathutils import Vector
from . import core
from .locomotion import HIP_X, HIP_Z, LENGTH

LEG_JOINTS = ("leg_l", "leg_r", "knee_l", "knee_r", "foot_l", "foot_r")


def robe_fit(co):
    """Leave room for the pelvis and a walking stride beneath hanging cloth.

    This is a garment allowance, not skinning: the skirt stays suspended
    around both legs instead of being pulled into either hip joint.
    """
    blend = max(0, min(1, (1.38 - co.z) / 0.27))
    blend = blend * blend * (3 - 2 * blend)
    drop = max(0, min(1, (1.05 - co.z) / 1.0))
    width, depth = 0.30 + 0.10 * drop, 0.27 + 0.32 * drop
    angle = math.atan2(co.y / depth, co.x / width)
    # Retain small hanging folds even where a narrow spirit robe needs room.
    fold = 1 + 0.025 * math.sin(9 * angle + co.z)
    radius = math.hypot(co.x / width, co.y / depth)
    if radius > 1e-6:
        scale = 1 + blend * max(0, fold / radius - 1)
        co.x *= scale
        co.y *= scale
    return co


def limb(name, parent, material, rings):
    """Elliptical cross-sections preserve thigh, patella, calf and ankle shapes."""
    vertices, faces = [], []
    sides = 24
    for z, y, rx, ry in rings:
        for i in range(sides):
            a = math.tau * i / sides
            vertices.append((math.cos(a) * rx, y + math.sin(a) * ry, z))
    for row in range(len(rings) - 1):
        for i in range(sides):
            a = row * sides + i
            b = row * sides + (i + 1) % sides
            faces.append((a, a + sides, b + sides, b))
    faces.append(tuple(reversed(range(sides))))
    faces.append(tuple((len(rings) - 1) * sides + i for i in range(sides)))
    mesh = bmesh.new()
    points = [mesh.verts.new(point) for point in vertices]
    for face in faces:
        mesh.faces.new([points[index] for index in face])
    obj = core.object_from_bmesh(name, mesh, material, parent)
    core.shade_smooth(obj)
    core.finish(obj)
    return obj


def build(root, calling):
    """Fit anatomical mass around the existing class shoulders and garments."""
    nodes = {node.name: node for node in root.children_recursive}
    shoulder = nodes["arm_l"].location
    width = {"ferryman": 0.235, "hollow": 0.19, "mourner": 0.15}.get(calling, 0.185)
    material = core.material("spirit", color=(0.08, 0.12, 0.15))
    torso = core.sculpt(
        "spirit_body",
        [
            ("sphere", (0, shoulder.y * 0.7, 1.44), (width, 0.12, 0.22)),
            ("sphere", (0, shoulder.y * 0.35, 1.24), (width * 0.68, 0.105, 0.22)),
        ],
        material,
        root,
        voxel=0.014,
    )
    core.finish(torso)
    pelvis = core.sculpt(
        "pelvis_body",
        [
            ("sphere", (0, 0.015, 1.04), (0.19, 0.125, 0.15)),
            ("sphere", (-0.09, 0.04, 1.01), (0.10, 0.115, 0.115)),
            ("sphere", (0.09, 0.04, 1.01), (0.10, 0.115, 0.115)),
        ],
        material,
        root,
        voxel=0.015,
    )
    core.finish(pelvis)
    for side in ("l", "r"):
        arm, elbow, wrist = (nodes[f"{joint}_{side}"] for joint in ("arm", "elbow", "wrist"))
        for parent, end, name in ((arm, elbow.location, "upper"), (elbow, wrist.location, "lower")):
            core.finish(core.tube(f"spirit_arm_{side}_{name}", [Vector(), end], lambda t: 0.058 - 0.024 * t, material, parent, sides=16))

    legs = core.empty("humanoid_form", root)
    pelvis.parent = legs
    for side, sign in (("l", 1), ("r", -1)):
        hip = core.empty(f"leg_{side}", legs, location=(sign * HIP_X, 0, HIP_Z))
        knee = core.empty(f"knee_{side}", hip, location=(0, 0, -LENGTH))
        ankle = core.empty(f"foot_{side}", knee, location=(0, 0, -LENGTH))
        limb(
            f"thigh_{side}",
            hip,
            material,
            [
                (0.055, 0.01, 0.048, 0.07),
                (0.0, 0.015, 0.094, 0.115),
                (-0.09, 0.005, 0.102, 0.119),
                (-0.20, -0.005, 0.094, 0.106),
                (-0.31, -0.015, 0.074, 0.080),
                (-0.40, -0.018, 0.060, 0.064),
                (-0.465, 0, 0.048, 0.050),
            ],
        )
        limb(
            f"shin_{side}",
            knee,
            material,
            [
                (0.04, 0, 0.048, 0.052),
                (0.0, -0.014, 0.064, 0.066),
                (-0.055, -0.005, 0.066, 0.067),
                (-0.14, 0.027, 0.075, 0.090),
                (-0.24, 0.02, 0.063, 0.073),
                (-0.34, 0.007, 0.044, 0.047),
                (-0.43, 0, 0.040, 0.041),
                (-0.465, 0, 0.033, 0.037),
            ],
        )
        core.finish(
            core.sculpt(
                f"patella_{side}",
                [
                    ("sphere", (0, -0.056, -0.004), (0.052, 0.022, 0.056)),
                ],
                material,
                knee,
                voxel=0.009,
            )
        )
        core.finish(
            core.sculpt(
                f"boot_{side}",
                [
                    ("sphere", (0, 0, -0.025), (0.055, 0.065, 0.09)),
                    ("sphere", (0, 0.018, -0.087), (0.066, 0.064, 0.043)),
                    ("sphere", (0, -0.10, -0.085), (0.073, 0.12, 0.045)),
                ],
                core.material("cloth"),
                ankle,
                voxel=0.008,
            )
        )

    tail = core.empty("spirit_form", root)
    for strand in range(3):
        points = []
        for step in range(25):
            t = step / 24
            angle = strand * math.tau / 3 + t * 2.8
            radius = 0.095 * (1 - t)
            points.append((math.cos(angle) * radius, math.sin(angle) * radius + 0.12 * t * t, 1.08 - 0.90 * t))
        core.finish(core.tube(f"spirit_wisp_{strand}", points, lambda t: 0.065 * (1 - t) + 0.002, material, tail, sides=10))

    for node in root.children_recursive:
        if not node.name.startswith("robe_") or node.type != "MESH":
            continue
        if node.data.shape_keys is None:
            node.shape_key_add(name="Basis")
        short = node.shape_key_add(name="short_robe")
        for vertex in short.data:
            if vertex.co.z < 1.18:
                vertex.co.z = 1.18 + (vertex.co.z - 1.18) * 0.28
        short.value = 0
        for name, source in (("humanoid_robe", node.data.shape_keys.key_blocks["Basis"]), ("humanoid_tunic", short)):
            fitted = node.shape_key_add(name=name)
            for vertex, original in zip(fitted.data, source.data):
                vertex.co = robe_fit(original.co.copy())
            fitted.value = 0
