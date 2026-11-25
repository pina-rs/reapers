"""The bone knife: one long fang on the pistol's grip of four iron
vertebrae, so the two read as one maker's.

It is held like every weapon: the grip at the origin and the blade running
up +Z, its edge toward +X. The spine is straight; the edge is a fang's belly,
widest a third of the way up. Its section is cut in planes (a flat back, two
faces meeting at the edge), so each prints as one tone.

The knife counts its dives (art bible 6.5): three notches cut in the spine,
each holding a wedge of soul light while its dive is ready. Each has a morph
target, `spent_<n>`, that sinks its light into the blade and leaves the
notch void, and an empty, `knife_notch_<n>`, at its mouth for its halo.
Notch 0 is nearest the guard. A seam of light runs along the edge.
`knife_tip` is an empty at the point. The whole knife is one mesh, `knife`:
one draw.

The bible asks for a blade 3.5 times its handle. The handle is the grip
every weapon is held by, four vertebrae a fist long (the one hand closes
on every weapon, so a shorter grip would leave its little finger round
air), so the blade is a little under twice it and the tip stays at .268 m.
What the fang loses in length it makes up in body: its belly is half as
wide again as a stiletto's, its spine a flat back 14 mm thick, and its
notches are cut deep enough that a spent one reads as a black bite at stab
size.
"""

import math

from mathutils import Vector

from lib import core

from . import common

GUARD_Z = 0.05
TIP_Z = 0.268

# The blade's back and edge, (x, height): a straight spine leaning a little
# toward the point, and a belly widest a third of the way up.
SPINE = ((-0.0105, GUARD_Z + 0.002), (0.003, TIP_Z))
BELLY = ((0.016, GUARD_Z + 0.002), (0.030, 0.098), (0.035, 0.123), (0.031, 0.160), (0.020, 0.205), (0.009, 0.245), (0.003, TIP_Z))
THICKNESS = 0.014

# The notches: where each is cut in the spine, how deep and how tall.
NOTCHES = (0.084, 0.112, 0.140)
NOTCH_DEPTH = 0.0093
NOTCH_HALF = 0.0075
# How far a spent notch's light sinks into the blade.
SINK = 0.0095


def build(parent):
    root = core.empty("weapon_boneKnife", parent)
    metal = core.material("metal")
    bone = core.material("bone")

    common.vertebrae("knife_grip", root, top=0.038, bottom=-0.062, half_width=0.0135, half_depth=0.0150)
    common.section(
        "knife_pommel",
        [((0, 0, -0.063), 0.0125, 0.0140), ((0, 0, -0.071), 0.0140, 0.0150), ((0, 0, -0.080), 0.0080, 0.0085)],
        common.polygon(6, math.pi / 6),
        metal,
        root,
    )

    # A collar under the blade, and a short guard that curls toward the edge.
    common.section(
        "knife_collar",
        [((0, 0, 0.037), 0.0150, 0.0150), ((0, 0, 0.041), 0.0172, 0.0168), ((0, 0, GUARD_Z - 0.001), 0.0172, 0.0168), ((0, 0, GUARD_Z + 0.003), 0.0135, 0.0120)],
        common.polygon(6, math.pi / 6),
        metal,
        root,
    )
    core.tube(
        "knife_guard",
        core.bezier([(-0.026, 0, GUARD_Z - 0.010), (-0.014, 0, GUARD_Z - 0.003), (0.014, 0, GUARD_Z - 0.003), (0.030, 0, GUARD_Z + 0.008)], samples=3),
        lambda t: 0.0052 * (1 - 0.4 * abs(2 * t - 1)),
        metal,
        root,
        sides=6,
        flatten=0.7,
    )

    _blade(root, bone)
    _notches(root)
    _edge(root)

    core.empty("knife_tip", root, location=(SPINE[1][0], 0, TIP_Z))
    knife = common.merge(root, "knife")
    common.empties(knife, [f"notch_{index}" for index in range(len(NOTCHES))], lambda co: co + Vector((SINK, 0, 0)))

    return root


def _spine_x(z):
    (x0, z0), (x1, z1) = SPINE

    return x0 + (x1 - x0) * (z - z0) / (z1 - z0)


def _belly_x(z):
    for (x0, z0), (x1, z1) in zip(BELLY, BELLY[1:]):
        if z0 <= z <= z1:
            share = (z - z0) / (z1 - z0)
            # Eased between points, so the belly is a curve, not a polyline.
            share = share * share * (3 - 2 * share)

            return x0 + (x1 - x0) * share

    return BELLY[-1][0]


def _blade(root, bone):
    """The fang: a loft of five-sided sections up the blade, with a V cut in
    the spine at each notch."""

    heights = [GUARD_Z + 0.002, 0.066]

    for notch in NOTCHES:
        heights += [notch - NOTCH_HALF, notch, notch + NOTCH_HALF]

    heights += [0.160, 0.185, 0.210, 0.232, 0.252]
    heights = sorted(set(heights))
    rings = []

    for z in heights:
        spine = _spine_x(z)
        width = _belly_x(z) - spine
        # Thinner toward the point.
        thick = THICKNESS * (0.35 + 0.65 * min(1.0, width / 0.036)) / 2

        if z in NOTCHES:
            spine += NOTCH_DEPTH
            width -= NOTCH_DEPTH

        ridge = spine + width * 0.38
        edge = spine + width
        # A flat back nearly as thick as the blade, so the fang has a spine
        # seen from the side, then two faces ground to the edge.
        rings.append([(spine, thick * 0.86, z), (ridge, thick, z), (edge, 0, z), (ridge, -thick, z), (spine, -thick * 0.86, z)])

    return common.skin("knife_blade", rings, bone, root, tip=(SPINE[1][0], 0, TIP_Z))


def _notches(root):
    """In each notch, a lining of void and a wedge of soul light over it,
    tagged so `spent_<n>` can sink the light into the blade."""

    void = core.material("void")
    accent = core.material("accent")

    for index, z in enumerate(NOTCHES):
        spine = _spine_x(z)
        thick = THICKNESS * 0.86 / 2
        # Both are the notch's own V moved out toward its mouth, so their
        # faces lie just in front of the blade's: the lining 0.4 mm, its
        # mouth sunk 1.5 mm below the spine, and the light 0.8 mm, proud of
        # the spine, covering the lining while it is in.
        _wedge(f"knife_notch_void_{index}", spine + 0.0015, z, NOTCH_DEPTH - 0.0019, NOTCH_HALF * (1 - 0.0019 / NOTCH_DEPTH), thick + 0.0003, void, root)
        light = _wedge(f"knife_notch_light_{index}", spine - 0.0008, z, NOTCH_DEPTH, NOTCH_HALF * (1 + 0.0008 / NOTCH_DEPTH), thick + 0.0006, accent, root)
        common.tag(light, f"notch_{index}")
        core.empty(f"knife_notch_{index}", root, location=(spine, 0, z))


def _wedge(name, mouth, z, depth, half, thick, mat, root):
    """A triangular prism filling a V notch: its mouth on the line x =
    `mouth`, `half` above and below `z`, its point `depth` into the blade."""

    triangle = [(mouth, z - half), (mouth + depth, z), (mouth, z + half)]
    rings = [[Vector((x, y, h)) for x, h in triangle] for y in (thick, -thick)]

    return core.loft(name, rings, mat, root, cap_start=True, cap_end=True)


def _edge(root):
    """The seam of light along the edge, from the guard to near the point."""

    heights = [GUARD_Z + 0.004 + (0.236 - GUARD_Z) * share / 10 for share in range(11)]
    core.tube(
        "knife_edge",
        [(_belly_x(z) - 0.0005, 0, z) for z in heights],
        lambda t: 0.0016 * (1 - 0.5 * t),
        core.material("accent"),
        root,
        sides=4,
    )
