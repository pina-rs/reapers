"""The scythe: the harvester's weapon.

A tusk of bone longer than a forearm on a snath of wood taller than its
bearer, so the blade rides above the head and out past the body's line
(art bible 6.4). The blade is heavy at its root, where it is deepest and
thickest, and curls down to its point like a boar's tusk; a hooked spur
twice the size of a scythe's tang rises behind the collar, so the top of
the weapon has two notes, not one. The snath is thick at the foot and
thin under the blade, and kinked under the upper grip.

A broad seam of light runs along the outer stretch of the blade's back,
the one light on it (`scythe_edge`, the name the game has always found it
by), and a proud band of the owner's light rings the snath below the upper
grip.
"""

import math

from mathutils import Vector

from lib import core
from lib.armoury import band, binding, finish

# The crescent's back and its edge, (x, z), from the collar to the point:
# deepest at the root, its inner curve sweeping low, the point curling down.
SPINE = [(-0.075, 1.53), (0.07, 1.70), (0.33, 1.785), (0.64, 1.715), (0.90, 1.50), (1.06, 1.18), (1.10, 0.86)]
EDGE = [(-0.045, 1.25), (0.11, 1.31), (0.35, 1.375), (0.60, 1.355), (0.81, 1.24), (0.98, 1.05), (1.10, 0.86)]
# How thick the blade is at its root.
THICKNESS = 0.042

# The snath, (x, z, radius), from its foot to its head under the collar: a
# run leaning out to the foot, kinked just under the band, straight through
# the grip, and bowed back a little under the blade.
SNATH = ((0.15, -1.05, 0.0245), (0.06, -0.55, 0.0235), (0.0, -0.2, 0.022), (0.0, 0.2, 0.021), (-0.03, 0.85, 0.0185), (0.0, 1.5, 0.0165))


def build():
    """The harvester's weapon: a bone tusk on a kinked wooden snath."""

    root = core.empty("weapon_scythe")
    metal = core.material("metal")
    bone = core.material("bone")

    _snath(root)
    binding("scythe_grip", root, -0.09, 0.12, radius=0.027)
    # The band stands well proud of the snath, square-shouldered, so it
    # prints as a ring of light with a key line at each step.
    band("scythe_band", root, -0.16, -0.105, radius=0.034)
    binding("scythe_lashing", root, 1.10, 1.24, radius=0.024, turns=5)

    spine = core.bezier(SPINE, samples=8)
    edge = core.bezier(EDGE, samples=8)
    count = min(len(spine), len(edge))
    spine = [(point.x, point.y) for point in spine[:count]]
    edge = [(point.x, point.y) for point in edge[:count]]

    finish(core.blade("scythe_blade", spine, edge, THICKNESS, bone, root), wear_boost=2.0)
    _seam(root, spine, edge, THICKNESS, start=0.34, end=0.86, width=0.034)

    # The forged collar the tang is socketed in.
    finish(
        core.shade_smooth(
            core.revolve(
                "scythe_collar",
                [
                    (0.0, 1.575),
                    (0.016, 1.555),
                    (0.034, 1.525),
                    (0.046, 1.505),
                    (0.048, 1.485),
                    (0.040, 1.468),
                    (0.046, 1.43),
                    (0.040, 1.30),
                    (0.044, 1.27),
                    (0.034, 1.25),
                    (0.0, 1.245),
                ],
                metal,
                root,
                segments=12,
            ),
            angle=45,
        ),
        wear_boost=1.8,
    )

    # The spur behind the collar: the tusk's root, curled back and down
    # into a tight hook, a second, shorter note at the top of the weapon.
    spur_spine = core.bezier([(-0.03, 1.55), (-0.15, 1.69), (-0.30, 1.69), (-0.39, 1.58), (-0.36, 1.45), (-0.29, 1.41)], samples=4)
    spur_edge = core.bezier([(-0.03, 1.45), (-0.13, 1.56), (-0.24, 1.585), (-0.305, 1.535), (-0.31, 1.47), (-0.29, 1.41)], samples=4)
    count = min(len(spur_spine), len(spur_edge))
    finish(
        core.blade(
            "scythe_hook",
            [(p.x, p.y) for p in spur_spine[:count]],
            [(p.x, p.y) for p in spur_edge[:count]],
            0.03,
            bone,
            root,
        ),
        wear_boost=1.8,
    )

    return root


def _snath(root):
    """The snath: thick at the foot, kinked under the upper grip so its two
    lengths meet at an angle under the hand, and thin under the blade."""

    wood = core.material("wood")
    points = []

    # Two straight runs, each eased only at its ends, so the kink at the
    # grip stays a kink.
    for (x0, z0, r0), (x1, z1, r1) in zip(SNATH, SNATH[1:]):
        for step in range(4):
            share = step / 4
            points.append((x0 + (x1 - x0) * share, z0 + (z1 - z0) * share, r0 + (r1 - r0) * share))

    points.append(SNATH[-1])
    radii = [radius for _, _, radius in points]
    snath = core.tube(
        "scythe_haft",
        [(x, 0, z) for x, z, _ in points],
        lambda t: radii[min(round(t * (len(radii) - 1)), len(radii) - 1)],
        wood,
        root,
        sides=8,
    )
    finish(core.shade_smooth(snath, angle=40), wear_boost=1.3)


def _seam(root, spine, edge, thickness, start, end, width):
    """The seam of light along the blade's back, from `start` to `end` of
    the way to the point: a cap over the back's ridge, `width` deep down
    each face at its widest, lying on the faces as `core.blade` grinds them,
    so it prints as a broad stripe along the top of the crescent rather than
    a hairline."""

    count = len(spine)
    first, last = round(start * (count - 1)), round(end * (count - 1))
    rings = []

    for index in range(first, last + 1):
        t = index / (count - 1)
        (sx, sz), (ex, ez) = spine[index], edge[index]
        depth = math.hypot(ex - sx, ez - sz)
        down = Vector(((ex - sx) / depth, 0.0, (ez - sz) / depth))
        top = Vector((sx, 0.0, sz))
        # The same taper `core.blade` gives the blade, so the seam lies on it.
        half = max(thickness * 0.5 * (1.0 - 0.72 * t**1.4), 0.0008)
        # Widest in the middle of its run, thinning to nothing at its ends.
        share = (index - first) / max(last - first, 1)
        reach = width * math.sin(math.pi * share) ** 0.6 + 0.004
        # How far the face stands out at that depth: `core.blade` rises to
        # its rib, 1.12 of the half thickness, at 14% of the way down.
        rib = 0.14 * depth
        face = half * 1.12 * min(1.0, reach / rib)
        proud = 0.0012
        lower = top + down * reach
        ring = [
            lower + Vector((0.0, -face - proud, 0.0)),
            top - down * proud,
            lower + Vector((0.0, face + proud, 0.0)),
            lower + Vector((0.0, face - 0.002, 0.0)) - down * 0.002,
            top + down * 0.003,
            lower + Vector((0.0, -face + 0.002, 0.0)) - down * 0.002,
        ]
        rings.append(ring)

    core.loft("scythe_edge", rings, core.material("accent"), root, cap_start=True, cap_end=True)
