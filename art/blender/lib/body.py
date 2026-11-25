"""Shared anatomy with shaped legs and overlapping anatomical joints."""

import math
import bpy
import bmesh
from mathutils import Vector
from . import core
from .locomotion import HIP_X, HIP_Z, LENGTH

LEG_JOINTS = ("leg_l", "leg_r", "knee_l", "knee_r", "foot_l", "foot_r")
# What each humanoid boot is cut down to (`core.planes`).
BOOT_TRIANGLES = 1400


# How far a humanoid skirt stands off the legs: its (half-width, half-depth)
# at the hips and at the floor. Most classes wear a bell with room for a
# sprint's stride; a class whose read is its column or its taper keeps it,
# and lets a long stride show the legs at the hem.
SKIRT = ((0.30, 0.27), (0.40, 0.59))
SKIRTS = {"harvester": ((0.24, 0.2), (0.4, 0.44)), "weigher": ((0.24, 0.2), (0.26, 0.44))}


def robe_fit(co, skirt=SKIRT):
    """Leave room for the pelvis and a walking stride beneath hanging cloth.

    This is a garment allowance, not skinning: the skirt stays suspended
    around both legs instead of being pulled into either hip joint.
    """
    blend = max(0, min(1, (1.38 - co.z) / 0.27))
    blend = blend * blend * (3 - 2 * blend)
    drop = max(0, min(1, (1.05 - co.z) / 1.0))
    (hip_width, hip_depth), (floor_width, floor_depth) = skirt
    width, depth = hip_width + (floor_width - hip_width) * drop, hip_depth + (floor_depth - hip_depth) * drop
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
    """Fit anatomical mass cleanly inside each class's shoulders and garments."""
    nodes = {node.name: node for node in root.children_recursive}
    shoulder = nodes["arm_l"].location
    # Keep the inner spirit core tucked cleanly inside even the slimmest robes.
    width = {
        "ferryman": 0.195,
        "hollow": 0.145,
        "mourner": 0.115,
        "harvester": 0.135,
        "weigher": 0.145,
        "chooser": 0.135,
    }.get(calling, 0.14)
    depth = 0.088 if calling == "ferryman" else 0.072
    material = core.material("spirit", color=(0.08, 0.12, 0.15))
    torso = core.sculpt(
        "spirit_body",
        [
            ("sphere", (0, shoulder.y * 0.85, 1.46), (width, depth, 0.20)),
            ("sphere", (0, shoulder.y * 0.50, 1.24), (width * 0.66, depth * 0.88, 0.19)),
        ],
        material,
        root,
        voxel=0.014,
    )
    core.finish(torso)
    pelvis = core.sculpt(
        "pelvis_body",
        [
            ("sphere", (0, 0.015, 1.04), (0.16, 0.105, 0.14)),
            ("sphere", (-0.08, 0.035, 1.01), (0.085, 0.095, 0.105)),
            ("sphere", (0.08, 0.035, 1.01), (0.085, 0.095, 0.105)),
        ],
        material,
        root,
        voxel=0.015,
    )
    core.finish(pelvis)
    for side in ("l", "r"):
        arm, elbow, wrist = (nodes[f"{joint}_{side}"] for joint in ("arm", "elbow", "wrist"))
        # Slim tapered spirit core tucked inside the sleeve and stopping cleanly
        # before the skeletal wrist/hand so it never bursts through cuffs or palms.
        if calling == "mourner":
            upper_start = elbow.location * 0.14
            upper_end = elbow.location * 0.94
            lower_end = wrist.location * 0.62
            r_upper = lambda t: 0.018 - 0.005 * t
            r_lower = lambda t: 0.014 * (1.0 - 0.65 * t)
        else:
            upper_start = elbow.location * 0.08
            upper_end = elbow.location * 0.98
            lower_end = wrist.location * 0.68
            r_upper = lambda t: 0.036 - 0.008 * t
            r_lower = lambda t: 0.028 * (1.0 - 0.72 * t)
        core.finish(core.tube(f"spirit_arm_{side}_upper", [upper_start, upper_end], r_upper, material, arm, sides=14))
        core.finish(core.tube(f"spirit_arm_{side}_lower", [Vector(), lower_end], r_lower, material, elbow, sides=14))

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
        # Cut into planes, like a carved clog: a boot is one mass at the
        # floor, and its band edges should run straight along it.
        core.finish(
            core.planes(
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
                ),
                BOOT_TRIANGLES,
            )
        )

    tail = core.empty("spirit_form", root)
    # Graceful tapered spectral wisps tucked inside the lower robe silhouette.
    spread = 0.042 if calling == "mourner" else 0.054
    sweep_x = -0.18 if calling == "chooser" else 0.0
    back_y = 0.08 if calling == "hollow" else 0.01
    for strand in range(3):
        points = []
        for step in range(25):
            t = step / 24
            angle = strand * math.tau / 3 + t * 2.6
            radius = spread * (1.0 - 0.88 * t)
            x = math.cos(angle) * radius + sweep_x * (t**1.8)
            y = math.sin(angle) * radius + back_y + 0.09 * t * t
            z = 0.96 - 0.80 * t
            points.append((x, y, z))
        core.finish(core.tube(f"spirit_wisp_{strand}", points, lambda t: 0.032 * (1.0 - t) ** 0.85 + 0.003, material, tail, sides=10))

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
                vertex.co = robe_fit(original.co.copy(), SKIRTS.get(calling, SKIRT))
            fitted.value = 0
