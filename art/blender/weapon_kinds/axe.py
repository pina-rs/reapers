"""The axe: a heavy head no class starts with.

A forged bearded head with tapered wedge cheeks on a short haft, backed by a
curved forged armor-piercing beak. The whole curve of its edge is the one
light on it.
"""

from lib import core
from lib.armoury import binding, finish, haft

HEAD_LOW = 0.34
HEAD_HIGH = 0.6
TOP = 0.66
BUTT = -0.52


def build():
    """A forged bearded axe head and back beak on a short haft."""

    root = core.empty("weapon_axe")
    metal = core.material("metal")

    haft("axe_haft", root, bottom=BUTT, top=TOP, radius=0.022, bow=0.018)
    binding("axe_grip", root, -0.12, 0.13, radius=0.027)
    binding("axe_lashing", root, HEAD_LOW - 0.07, HEAD_LOW - 0.01, radius=0.027, turns=4)

    # Forged tapered bearded axe blade: thick socket neck flaring smoothly to a
    # keen crescent edge with sweeping lower beard.
    edge_pts = core.bezier([(0.26, HEAD_HIGH + 0.065), (0.315, 0.51), (0.305, 0.375), (0.205, 0.225)], samples=12)
    edge = [(p.x, p.y) for p in edge_pts]
    n = len(edge)
    # Spine runs along the eye/neck face so core.blade() tapers cleanly from thick eye to razor edge!
    spine = [
        (
            0.032 + 0.022 * (1.0 - abs(2.0 * (i / max(n - 1, 1)) - 1.0)),
            (HEAD_HIGH + 0.005) + ((HEAD_LOW + 0.025) - (HEAD_HIGH + 0.005)) * (i / max(n - 1, 1)),
        )
        for i in range(n)
    ]

    finish(core.blade("axe_head", spine, edge, 0.030, metal, root), wear_boost=2.0)

    lit = core.tube(
        "axe_edge",
        [(x + 0.003, 0, z) for x, z in edge],
        lambda t: 0.0038 + 0.0022 * (1 - abs(2 * t - 1)),
        core.material("accent"),
        root,
        sides=6,
    )
    lit.location = (0, 0, 0)

    # Molded octagonal iron eye collar the haft passes through.
    finish(
        core.shade_smooth(
            core.revolve(
                "axe_eye",
                [
                    (0.0, HEAD_LOW + 0.012),
                    (0.033, HEAD_LOW + 0.018),
                    (0.041, HEAD_LOW + 0.035),
                    (0.037, (HEAD_LOW + HEAD_HIGH) * 0.5),
                    (0.042, HEAD_HIGH - 0.012),
                    (0.034, HEAD_HIGH + 0.012),
                    (0.018, HEAD_HIGH + 0.026),
                    (0.0, HEAD_HIGH + 0.032),
                ],
                metal,
                root,
                segments=16,
            ),
            angle=42,
        ),
        wear_boost=1.8,
    )

    # Forged curved gothic war-beak behind the eye.
    beak_spine = core.bezier([(-0.032, 0.525), (-0.095, 0.530), (-0.165, 0.485), (-0.215, 0.415)], samples=6)
    beak_edge = core.bezier([(-0.032, 0.465), (-0.090, 0.480), (-0.150, 0.455), (-0.215, 0.415)], samples=6)
    b_count = min(len(beak_spine), len(beak_edge))
    finish(
        core.blade(
            "axe_spike",
            [(p.x, p.y) for p in beak_spine[:b_count]],
            [(p.x, p.y) for p in beak_edge[:b_count]],
            0.022,
            metal,
            root,
        ),
        wear_boost=1.8,
    )

    # Faceted spiked pommel cap on the butt of the haft.
    cap = core.revolve(
        "axe_butt",
        [(0.0, 0.034), (0.028, 0.030), (0.033, 0.008), (0.028, -0.018), (0.016, -0.038), (0.0, -0.056)],
        metal,
        root,
        segments=14,
    )
    cap.location = (0.005, 0, BUTT)
    finish(core.shade_smooth(cap, angle=45), wear_boost=1.6)

    return root
