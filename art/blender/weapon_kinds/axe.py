"""The axe: a heavy head no class starts with.

A bearded head on a short haft, with a spike behind it. It is meant for the
body, not the seals: the whole curve of its edge is the one light on it.
"""

from lib import core
from lib.armoury import binding, finish, haft

HEAD_LOW = 0.34
HEAD_HIGH = 0.6
TOP = 0.66
BUTT = -0.52


def build():
    """A bearded axe head and a back spike on a short haft."""

    root = core.empty("weapon_axe")
    metal = core.material("metal")

    haft("axe_haft", root, bottom=BUTT, top=TOP, radius=0.022, bow=0.018)
    binding("axe_grip", root, -0.12, 0.13, radius=0.027)
    binding("axe_lashing", root, HEAD_LOW - 0.07, HEAD_LOW - 0.01, radius=0.027, turns=4)

    # The head: a cheek around the haft that flares out to a long, curved
    # edge, with a beard hooking down below it.
    edge = core.bezier([(0.27, HEAD_HIGH + 0.05), (0.31, 0.5), (0.3, 0.38), (0.21, 0.24)], samples=12)
    edge = [(point.x, point.y) for point in edge]
    outline = (
        [(0.03, HEAD_HIGH), (0.12, HEAD_HIGH - 0.005), (0.22, HEAD_HIGH + 0.035)]
        + edge
        + [(0.15, 0.3), (0.09, HEAD_LOW + 0.02), (0.03, HEAD_LOW + 0.03)]
    )

    finish(core.plate("axe_head", outline, 0.022, metal, root, bevel=0.004), wear_boost=2.0)

    lit = core.tube(
        "axe_edge",
        [(x + 0.004, 0, z) for x, z in edge],
        lambda t: 0.004 + 0.002 * (1 - abs(2 * t - 1)),
        core.material("accent"),
        root,
        sides=6,
    )
    lit.location = (0, 0, 0)

    # The eye the haft passes through.
    finish(
        core.revolve(
            "axe_eye",
            [(0.0, HEAD_LOW + 0.02), (0.034, HEAD_LOW + 0.025), (0.038, HEAD_HIGH - 0.02), (0.034, HEAD_HIGH + 0.005), (0.0, HEAD_HIGH + 0.008)],
            metal,
            root,
            segments=14,
        ),
        wear_boost=1.8,
    )

    # A spike behind, for the back swing.
    finish(
        core.tube(
            "axe_spike",
            core.bezier([(-0.03, 0, 0.49), (-0.09, 0, 0.5), (-0.15, 0, 0.47), (-0.19, 0, 0.42)], samples=6),
            lambda t: 0.017 * (1 - 0.92 * t),
            metal,
            root,
            sides=8,
        ),
        wear_boost=1.8,
    )

    # A cap on the butt of the haft.
    cap = core.revolve(
        "axe_butt",
        [(0.0, 0.03), (0.026, 0.028), (0.03, 0.0), (0.024, -0.03), (0.0, -0.04)],
        metal,
        root,
        segments=12,
    )

    cap.location = (0.005, 0, BUTT)
    finish(cap, wear_boost=1.6)

    return root
