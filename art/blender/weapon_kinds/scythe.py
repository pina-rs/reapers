"""The scythe: the harvester's weapon.

A forged crescent blade longer than a forearm on a haft taller than its bearer,
with a reinforced spine rib, hollow-ground cheeks, ornate forged collar, and
curved back-spur. The cutting edge is the one light on it.
"""

from lib import core
from lib.armoury import binding, finish, haft


def build():
    """The harvester's weapon: a forged crescent blade on a gnarled snath."""

    root = core.empty("weapon_scythe")
    metal = core.material("metal")

    haft("scythe_haft", root, bottom=-1.05, top=1.5)
    binding("scythe_grip", root, -0.09, 0.12)
    binding("scythe_lashing", root, 1.29, 1.44, radius=0.028, turns=6)

    # Forged sweeping crescent blade with tapered heel and hooked gothic point.
    spine = core.bezier([(-0.07, 1.49), (0.08, 1.59), (0.34, 1.65), (0.66, 1.56), (0.92, 1.32), (1.08, 0.99)], samples=10)
    edge = core.bezier([(-0.05, 1.37), (0.12, 1.43), (0.36, 1.45), (0.64, 1.39), (0.86, 1.22), (1.08, 0.99)], samples=10)
    count = min(len(spine), len(edge))
    spine = [(point.x, point.y) for point in spine[:count]]
    edge = [(point.x, point.y) for point in edge[:count]]

    finish(core.blade("scythe_blade", spine, edge, 0.024, metal, root), wear_boost=2.0)

    # The cutting edge is the harvester's one light.
    lit = core.tube(
        "scythe_edge",
        [(x, 0, z) for x, z in edge],
        lambda t: 0.0052 * (1 - 0.58 * t),
        core.material("accent"),
        root,
        sides=6,
    )
    lit.location = (0, 0, -0.003)

    # Molded forged tang socket collar with decorative ridges and finial cap.
    finish(
        core.shade_smooth(
            core.revolve(
                "scythe_collar",
                [
                    (0.0, 1.565),
                    (0.012, 1.545),
                    (0.026, 1.515),
                    (0.036, 1.502),
                    (0.038, 1.488),
                    (0.033, 1.474),
                    (0.039, 1.442),
                    (0.034, 1.350),
                    (0.038, 1.312),
                    (0.030, 1.292),
                    (0.0, 1.285),
                ],
                metal,
                root,
                segments=18,
            ),
            angle=45,
        ),
        wear_boost=1.8,
    )

    # Forged curved gothic back-spur balancing the heavy blade.
    spur_spine = core.bezier([(-0.025, 1.495), (-0.11, 1.565), (-0.195, 1.515), (-0.21, 1.40)], samples=6)
    spur_edge = core.bezier([(-0.025, 1.445), (-0.10, 1.505), (-0.165, 1.475), (-0.21, 1.40)], samples=6)
    s_count = min(len(spur_spine), len(spur_edge))
    finish(
        core.blade(
            "scythe_hook",
            [(p.x, p.y) for p in spur_spine[:s_count]],
            [(p.x, p.y) for p in spur_edge[:s_count]],
            0.016,
            metal,
            root,
        ),
        wear_boost=1.8,
    )

    return root
