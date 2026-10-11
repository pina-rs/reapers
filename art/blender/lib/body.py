"""A shared spectral body, articulated legs and removable outer clothing."""

import math

import bpy
from mathutils import Vector

from . import core

LEG_JOINTS = ("leg_l", "leg_r", "knee_l", "knee_r", "foot_l", "foot_r")


def build(root, calling):
    """Keep the class's head, shoulders and hands around a complete body."""
    nodes = {node.name: node for node in root.children_recursive}
    shoulder = nodes["arm_l"].location
    width = {"ferryman": 0.24, "hollow": 0.19, "mourner": 0.13}.get(calling, 0.16)
    material = core.material("spirit", color=(0.08, 0.12, 0.15))
    torso = core.sculpt(
        "spirit_body",
        [
            ("sphere", (0, shoulder.y * 0.7, 1.42), (width, 0.10, 0.24)),
            ("sphere", (0, shoulder.y * 0.3, 1.18), (width * 0.66, 0.085, 0.20)),
            ("sphere", (0, 0, 1.01), (width * 0.85, 0.10, 0.12)),
        ],
        material,
        root,
        voxel=0.018,
    )
    core.finish(torso)
    # Sleeves can be removed without leaving hands suspended in space.
    for side in ("l", "r"):
        arm, elbow, wrist = (nodes[f"{joint}_{side}"] for joint in ("arm", "elbow", "wrist"))
        for parent, end, name in ((arm, elbow.location, "upper"), (elbow, wrist.location, "lower")):
            core.finish(core.tube(f"spirit_arm_{side}_{name}", [Vector(), end], lambda t: 0.046 - 0.013 * t, material, parent, sides=10))

    legs = core.empty("humanoid_form", root)
    for side, sign in (("l", 1), ("r", -1)):
        hip = core.empty(f"leg_{side}", legs, location=(sign * 0.10, 0, 1.01))
        knee = core.empty(f"knee_{side}", hip, location=(0, -0.015, -0.44))
        ankle = core.empty(f"foot_{side}", knee, location=(0, 0.015, -0.44))
        core.finish(core.tube(f"thigh_{side}", [(0, 0, 0), (0, -0.015, -0.46)], lambda t: 0.068 - 0.026 * t, material, hip, sides=12))
        core.finish(core.tube(f"shin_{side}", [(0, 0, 0.025), (0, 0.015, -0.46)], lambda t: 0.048 - 0.021 * t, material, knee, sides=12))
        core.finish(core.sculpt(f"boot_{side}", [("sphere", (0, -0.055, -0.075), (0.053, 0.11, 0.052))], core.material("cloth_alt"), ankle, voxel=0.012))

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
        # A knee-length tunic retains each class's authored upper tailoring.
        for vertex in short.data:
            if vertex.co.z < 1.18:
                vertex.co.z = 1.18 + (vertex.co.z - 1.18) * 0.36
        short.value = 0


def sample(name, phase):
    """Leg performances share clip timing with the floating upper body."""
    result = {}
    for side, offset in (("l", 0), ("r", math.pi)):
        cycle = phase * math.tau + offset
        if name in ("creep", "walk", "run", "sprint"):
            stride = {"creep": 10, "walk": 22, "run": 38, "sprint": 50}[name]
            result[f"leg_{side}"] = (stride * math.sin(cycle), 0, 0)
            result[f"knee_{side}"] = (-max(0, math.sin(cycle)) * stride * 1.4, 0, 0)
            result[f"foot_{side}"] = (max(0, math.sin(cycle)) * stride * 0.45, 0, 0)
        else:
            tuck = math.sin(phase * math.pi) ** 2
            bend = 35 if name in ("jump", "land", "finisherUppercut") else 8
            result[f"leg_{side}"] = (bend * tuck, 0, 0)
            result[f"knee_{side}"] = (-bend * 1.5 * tuck, 0, 0)
    return result
