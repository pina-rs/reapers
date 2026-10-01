"""The lantern pole: the ferryman's weapon.

A boat pole taller than its bearer, shod in iron, with a crook at its top and
a caged lantern hanging from the crook. The flame is the one light on it.
"""

import math

from lib import core
from lib.armoury import binding, finish, haft

# How far the haft bows. Whatever is fitted to it away from the grip follows.
BOW = 0.022

# How far out from the pole the lantern hangs, and the height of the crook's
# tip that it hangs from.
HANG = 0.3
TIP = 1.425


def build():
    """The ferryman's weapon: a shod pole with a lantern hung from its crook."""

    root = core.empty("weapon_lanternPole")
    metal = core.material("metal")

    haft("lantern_pole_haft", root, bottom=-1.03, top=1.46, radius=0.027, bow=BOW)
    binding("lantern_pole_grip", root, -0.1, 0.13, radius=0.031)

    # The haft bows, so the lashing sits a little off the grip's axis.
    binding("lantern_pole_lashing", root, 1.19, 1.31, radius=0.036, turns=6).location = (0.4 * BOW, 0, 0)

    # An iron shoe, so the pole bites the river bed instead of splitting.
    shoe = core.revolve(
        "lantern_pole_shoe",
        [(0.0, -1.09), (0.012, -1.075), (0.026, -1.0), (0.029, -0.9), (0.034, -0.895), (0.034, -0.865), (0.026, -0.86)],
        metal,
        root,
        segments=14,
    )
    shoe.location = (BOW * 0.4, 0, 0)
    finish(shoe, wear_boost=1.8)

    finish(
        core.revolve(
            "lantern_pole_collar",
            [(0.0, 1.5), (0.022, 1.495), (0.038, 1.47), (0.04, 1.43), (0.036, 1.34), (0.041, 1.335), (0.041, 1.315), (0.03, 1.31)],
            metal,
            root,
            segments=14,
        ),
        wear_boost=1.8,
    )

    # The crook rises out of the collar, arches toward the bearer and turns
    # up at its tip, which is where the lantern's ring sits.
    finish(
        core.tube(
            "lantern_pole_crook",
            core.bezier(
                [
                    (0.0, 0, 1.46),
                    (0.03, 0, 1.535),
                    (0.13, 0, 1.565),
                    (0.26, 0, 1.555),
                    (0.335, 0, 1.5),
                    (HANG + 0.02, 0, TIP + 0.005),
                    (HANG - 0.02, 0, TIP),
                    (HANG - 0.04, 0, TIP + 0.04),
                ],
                samples=5,
            ),
            lambda t: 0.017 * (1 - 0.62 * t),
            metal,
            root,
            sides=8,
        ),
        wear_boost=1.8,
    )

    # A boat hook behind the crook, for pulling things in.
    finish(
        core.tube(
            "lantern_pole_hook",
            core.bezier([(-0.02, 0, 1.44), (-0.1, 0, 1.5), (-0.17, 0, 1.45), (-0.16, 0, 1.35)], samples=6),
            lambda t: 0.015 * (1 - 0.88 * t),
            metal,
            root,
            sides=8,
        ),
        wear_boost=1.8,
    )

    _lantern(root, metal)

    return root


def _lantern(root, metal):
    """A storm lantern: a roof, a floor and a cage of bars around one flame."""

    roof = TIP - 0.05
    eave = roof - 0.1
    floor = eave - 0.225
    parts = []

    # Two links between the crook and the roof.
    for link, height in enumerate((TIP - 0.012, TIP - 0.04)):
        turn = link * math.pi / 2
        path = []

        for step in range(15):
            angle = math.tau * step / 14
            reach = math.cos(angle) * 0.016

            path.append((math.cos(turn) * reach, math.sin(turn) * reach, height + math.sin(angle) * 0.02))

        parts.append(core.tube(f"lantern_link_{link}", path, 0.0048, metal, sides=5, cap=False))

    parts.append(
        core.revolve(
            "lantern_roof",
            [
                (0.0, roof + 0.004),
                (0.014, roof),
                (0.02, roof - 0.022),
                (0.044, roof - 0.034),
                (0.05, roof - 0.052),
                (0.112, eave),
                (0.116, eave - 0.008),
                (0.098, eave - 0.012),
                (0.04, eave + 0.03),
            ],
            metal,
            segments=20,
        )
    )
    parts.append(
        core.revolve(
            "lantern_floor",
            [(0.088, floor + 0.016), (0.1, floor + 0.012), (0.102, floor), (0.07, floor - 0.02), (0.03, floor - 0.026), (0.0, floor - 0.04)],
            metal,
            segments=20,
        )
    )

    # The cage bows outward, like a barrel.
    def bulge(share):
        return 0.09 + 0.014 * math.sin(share * math.pi)

    for bar in range(8):
        angle = math.tau * (bar + 0.5) / 8
        path = []

        for step in range(7):
            share = step / 6
            path.append((math.cos(angle) * bulge(share), math.sin(angle) * bulge(share), floor + 0.012 + (eave - floor - 0.016) * share))

        parts.append(core.tube(f"lantern_bar_{bar}", path, 0.0055, metal, sides=5))

    for hoop, share in enumerate((0.3, 0.7)):
        height = floor + 0.012 + (eave - floor - 0.016) * share
        path = [(math.cos(math.tau * step / 24) * bulge(share), math.sin(math.tau * step / 24) * bulge(share), height) for step in range(25)]

        parts.append(core.tube(f"lantern_hoop_{hoop}", path, 0.0045, metal, sides=5, cap=False))

    # The cup the flame stands in.
    parts.append(
        core.revolve(
            "lantern_cup",
            [(0.0, floor + 0.05), (0.022, floor + 0.052), (0.03, floor + 0.04), (0.024, floor + 0.012)],
            metal,
            segments=12,
        )
    )

    lantern = core.join(parts, "lantern_pole_lantern")
    lantern.location = (HANG, 0, 0)
    lantern.parent = root
    finish(core.shade_smooth(lantern, angle=50), wear_boost=1.8)

    # The flame is the ferryman's one light: a teardrop, leaning as it would
    # in a draught.
    base = floor + 0.05
    flame = core.tube(
        "lantern_pole_flame",
        [(0.012 * (step / 9) ** 2, 0, base + 0.125 * step / 9) for step in range(10)],
        lambda t: 0.003 + 0.031 * math.sin(min(t * 1.6, 1.0) * math.pi * 0.5) * (1 - t) ** 0.8,
        core.material("accent"),
        root,
        sides=10,
    )
    flame.location = (HANG, 0, 0)
