"""The scythe: the harvester's weapon.

A blade longer than a forearm on a haft taller than its bearer. The cutting
edge is the one light on it.
"""

from lib import core
from lib.armoury import binding, finish, haft


def build():
    """The harvester's weapon: a blade longer than a forearm on a haft taller
    than its bearer."""

    root = core.empty("weapon_scythe")
    metal = core.material("metal")

    haft("scythe_haft", root, bottom=-1.05, top=1.5)
    binding("scythe_grip", root, -0.09, 0.12)
    binding("scythe_lashing", root, 1.3, 1.44, radius=0.027, turns=6)

    # The blade sweeps out over the bearer's head and down to a point.
    spine = core.bezier([(-0.11, 1.47), (0.2, 1.63), (0.58, 1.6), (0.9, 1.36), (1.07, 1.0)], samples=9)
    edge = core.bezier([(-0.11, 1.33), (0.16, 1.44), (0.5, 1.43), (0.83, 1.25), (1.07, 1.0)], samples=9)
    count = min(len(spine), len(edge))
    spine = [(point.x, point.y) for point in spine[:count]]
    edge = [(point.x, point.y) for point in edge[:count]]

    finish(core.blade("scythe_blade", spine, edge, 0.02, metal, root), wear_boost=2.0)

    # The cutting edge is the harvester's one light.
    lit = core.tube(
        "scythe_edge",
        [(x, 0, z) for x, z in edge],
        lambda t: 0.0055 * (1 - 0.6 * t),
        core.material("accent"),
        root,
        sides=6,
    )
    lit.location = (0, 0, -0.004)

    # A collar where the tang meets the haft, and a hook behind it.
    finish(
        core.revolve(
            "scythe_collar",
            [(0.0, 1.52), (0.03, 1.51), (0.036, 1.44), (0.03, 1.3), (0.0, 1.29)],
            metal,
            root,
            segments=14,
        ),
        wear_boost=1.8,
    )
    finish(
        core.tube(
            "scythe_hook",
            core.bezier([(-0.02, 0, 1.48), (-0.14, 0, 1.56), (-0.2, 0, 1.46), (-0.16, 0, 1.36)], samples=6),
            lambda t: 0.014 * (1 - 0.85 * t),
            metal,
            root,
            sides=8,
        ),
        wear_boost=1.8,
    )

    return root


