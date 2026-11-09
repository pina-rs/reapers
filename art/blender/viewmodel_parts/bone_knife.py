"""The bone knife: a blade carved from one long fang, on a handle of
vertebrae.

It is held like every weapon: the grip at the origin and the blade running
up +Z, its edge toward +X. A thin seam of soul light runs along the edge;
it is the one light on the knife. `knife_tip` is an empty at the point.
The whole knife is one mesh, `knife`: one draw.
"""

import math

from lib import core

from . import common

GUARD_Z = 0.05
TIP_Z = 0.268


def build(parent):
    root = core.empty("weapon_boneKnife", parent)
    metal = core.material("metal")
    bone = core.material("bone")
    accent = core.material("accent")

    # A handle of vertebrae: a turned profile that swells at every joint.
    profile = []
    rings = 20

    for ring in range(rings + 1):
        share = ring / rings
        z = -0.066 + share * (GUARD_Z - 0.004 + 0.066)
        swell = abs(math.sin(share * math.pi * 5)) ** 2.5
        profile.append((0.0128 + 0.0042 * swell, z))

    profile = [(0.0, profile[0][1])] + profile + [(0.0, profile[-1][1])]
    core.revolve("knife_handle", profile, bone, root, segments=10)

    # An iron pommel ring and a short guard that curls toward the edge.
    core.revolve(
        "knife_pommel",
        [(0.0, -0.066), (0.012, -0.067), (0.0165, -0.072), (0.014, -0.08), (0.006, -0.083), (0.0, -0.083)],
        metal,
        root,
        segments=14,
    )
    core.tube(
        "knife_guard",
        core.bezier([(-0.03, 0, GUARD_Z - 0.008), (-0.012, 0, GUARD_Z), (0.012, 0, GUARD_Z), (0.034, 0, GUARD_Z + 0.012)], samples=5),
        lambda t: 0.0058 * (1 - 0.35 * abs(2 * t - 1)),
        metal,
        root,
        sides=8,
        flatten=0.7,
    )
    core.revolve(
        "knife_collar",
        [(0.0, GUARD_Z - 0.006), (0.0158, GUARD_Z - 0.006), (0.0165, GUARD_Z + 0.002), (0.012, GUARD_Z + 0.008), (0.0, GUARD_Z + 0.008)],
        metal,
        root,
        segments=14,
    )

    # The blade: one long fang of bone, flat across and curving back a
    # little toward the point.
    spine = core.bezier(
        [(-0.009, GUARD_Z + 0.006), (-0.008, 0.12), (-0.004, 0.2), (0.004, TIP_Z)],
        samples=6,
    )
    edge = core.bezier(
        [(0.014, GUARD_Z + 0.006), (0.0175, 0.11), (0.014, 0.19), (0.004, TIP_Z)],
        samples=6,
    )
    count = min(len(spine), len(edge))
    middle = [((a.x + b.x) / 2, 0.0, (a.y + b.y) / 2) for a, b in zip(spine[:count], edge[:count])]
    widths = [abs(b.x - a.x) / 2 for a, b in zip(spine[:count], edge[:count])]
    core.tube(
        "knife_blade",
        middle,
        lambda t: max(widths[min(int(t * (count - 1) + 0.5), count - 1)], 0.0012),
        bone,
        root,
        sides=10,
        flatten=0.32,
    )

    # The edge is lit, the knife's one light.
    core.tube(
        "knife_edge",
        [(b.x - 0.0004, 0, b.y) for b in edge[1 : count - 1]],
        lambda t: 0.0017 * (1 - 0.55 * t),
        accent,
        root,
        sides=5,
    )

    core.empty("knife_tip", root, location=(0.004, 0, TIP_Z))
    common.merge(root, "knife")

    return root
