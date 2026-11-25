"""The scythe: the harvester's weapon.

A crescent of bone longer than a forearm on a snath of wood taller than its
bearer, so the blade rides above the head and out past the body's line
(art bible 6.4). The crescent is broad and chunky, cut in clean planes, with
a hooked spur behind the collar. A seam of light runs along the blade's back,
the one light on it, and a band of the owner's light rings the snath below
the upper grip.
"""

from lib import core
from lib.armoury import band, binding, finish, haft

# The crescent's back and its edge, (x, z), from the collar to the point.
SPINE = [(-0.07, 1.50), (0.08, 1.62), (0.34, 1.69), (0.66, 1.60), (0.93, 1.35), (1.10, 0.98)]
EDGE = [(-0.05, 1.35), (0.12, 1.39), (0.36, 1.40), (0.64, 1.34), (0.87, 1.19), (1.10, 0.98)]


def build():
    """The harvester's weapon: a bone crescent on a wooden snath."""

    root = core.empty("weapon_scythe")
    metal = core.material("metal")
    bone = core.material("bone")

    haft("scythe_haft", root, bottom=-1.05, top=1.5)
    binding("scythe_grip", root, -0.09, 0.12)
    band("scythe_band", root, -0.135, -0.1, radius=0.026)
    binding("scythe_lashing", root, 1.29, 1.44, radius=0.028, turns=6)

    spine = core.bezier(SPINE, samples=10)
    edge = core.bezier(EDGE, samples=10)
    count = min(len(spine), len(edge))
    spine = [(point.x, point.y) for point in spine[:count]]
    edge = [(point.x, point.y) for point in edge[:count]]

    finish(core.blade("scythe_blade", spine, edge, 0.034, bone, root), wear_boost=2.0)

    # The seam of light along the crescent's back, riding its ridge, and
    # thinning toward the point.
    seam = []

    for (sx, sz), (ex, ez) in zip(spine[1:-2], edge[1:-2]):
        across = ((sx - ex) ** 2 + (sz - ez) ** 2) ** 0.5
        seam.append((sx + (sx - ex) / across * 0.002, 0, sz + (sz - ez) / across * 0.002))

    core.tube("scythe_seam", seam, lambda t: 0.0068 * (1 - 0.6 * t), core.material("accent"), root, sides=6)

    # The forged collar the tang is socketed in.
    finish(
        core.shade_smooth(
            core.revolve(
                "scythe_collar",
                [
                    (0.0, 1.565),
                    (0.014, 1.545),
                    (0.03, 1.515),
                    (0.041, 1.502),
                    (0.043, 1.488),
                    (0.037, 1.474),
                    (0.043, 1.442),
                    (0.037, 1.350),
                    (0.042, 1.312),
                    (0.032, 1.292),
                    (0.0, 1.285),
                ],
                metal,
                root,
                segments=12,
            ),
            angle=45,
        ),
        wear_boost=1.8,
    )

    # The spur behind the collar: the crescent's root, hooked back.
    spur_spine = core.bezier([(-0.025, 1.50), (-0.11, 1.575), (-0.205, 1.525), (-0.225, 1.395)], samples=6)
    spur_edge = core.bezier([(-0.025, 1.44), (-0.10, 1.505), (-0.17, 1.475), (-0.225, 1.395)], samples=6)
    count = min(len(spur_spine), len(spur_edge))
    finish(
        core.blade(
            "scythe_hook",
            [(p.x, p.y) for p in spur_spine[:count]],
            [(p.x, p.y) for p in spur_edge[:count]],
            0.024,
            bone,
            root,
        ),
        wear_boost=1.8,
    )

    return root
