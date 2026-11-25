"""The lantern pole: the ferryman's weapon.

A boat pole of wood taller than its bearer, shod in iron, with a heavy iron
crook arching off its top and a barbed boat hook behind it: the hooked pole
the ferryman is named by at any distance (art bible 6.2). A six-sided
lantern hangs from the crook's tip, its spired roof, floor and six bars in
iron, so the cage prints in key round the light. The flame is the one light
on it, 2.6 times a candle's, filling the cage bar to bar with a key gap
under the roof, so it reads as the spot seen through the bars. A band of
gold rings the pole under the hand.
"""

import math

from lib import core
from lib.armoury import band, binding, finish, haft

# How far the haft bows. Whatever is fitted to it away from the grip follows.
BOW = 0.022

# How far out from the pole the lantern hangs, and the height of the crook's
# tip that it hangs from.
HANG = 0.32
TIP = 1.62

# How much bigger than a candle's the flame is.
FLAME = 2.6

# The lantern's cage: the radius of its six corners, the height of its
# floor and its eave.
CAGE = 0.11
FLOOR = 1.15
EAVE = 1.42


def build():
    """The ferryman's weapon: a shod pole with a lantern hung from its crook."""

    root = core.empty("weapon_lanternPole")
    metal = core.material("metal")

    haft("lantern_pole_haft", root, bottom=-1.03, top=1.46, radius=0.027, bow=BOW)
    binding("lantern_pole_grip", root, -0.1, 0.13, radius=0.031)
    band("lantern_pole_band", root, -0.16, -0.11, radius=0.041)

    # The haft bows, so the lashing sits a little off the grip's axis.
    binding("lantern_pole_lashing", root, 1.19, 1.31, radius=0.036, turns=6).location = (0.4 * BOW, 0, 0)

    # An iron shoe, so the pole bites the river bed instead of splitting.
    shoe = core.revolve(
        "lantern_pole_shoe",
        [(0.0, -1.12), (0.012, -1.1), (0.028, -1.0), (0.03, -0.9), (0.036, -0.895), (0.036, -0.865), (0.026, -0.86)],
        metal,
        root,
        segments=8,
    )
    shoe.location = (BOW * 0.4, 0, 0)
    finish(shoe, wear_boost=1.8)

    finish(
        core.revolve(
            "lantern_pole_collar",
            [(0.0, 1.5), (0.024, 1.495), (0.04, 1.47), (0.043, 1.43), (0.038, 1.34), (0.044, 1.335), (0.044, 1.315), (0.03, 1.31)],
            metal,
            root,
            segments=10,
        ),
        wear_boost=1.8,
    )

    # The crook rises out of the collar, arches over toward the bearer's
    # front and comes down to its tip, which turns up where the lantern's
    # ring sits: thick at the root and tapering, so it reads as forged.
    finish(
        core.tube(
            "lantern_pole_crook",
            core.bezier(
                [
                    (0.0, 0, 1.46),
                    (0.01, 0, 1.6),
                    (0.08, 0, 1.73),
                    (0.2, 0, 1.775),
                    (0.31, 0, 1.73),
                    (HANG + 0.035, 0, TIP + 0.04),
                    (HANG - 0.01, 0, TIP - 0.012),
                    (HANG - 0.045, 0, TIP + 0.02),
                ],
                samples=4,
            ),
            lambda t: 0.024 * (1 - 0.55 * t),
            metal,
            root,
            sides=8,
        ),
        wear_boost=1.8,
    )

    # A barbed boat hook behind the crook, for pulling things in: a spike
    # up and a hook curling down and back.
    hook = core.tube(
        "lantern_pole_hook",
        core.bezier([(-0.01, 0, 1.43), (-0.1, 0, 1.52), (-0.2, 0, 1.53), (-0.25, 0, 1.44), (-0.21, 0, 1.36)], samples=3),
        lambda t: 0.02 * (1 - 0.9 * t),
        metal,
        root,
        sides=6,
        flatten=0.7,
    )
    spike = core.tube(
        "lantern_pole_spike",
        [(0.0, 0, 1.49), (-0.012, 0, 1.6), (-0.03, 0, 1.7)],
        lambda t: 0.017 * (1 - 0.92 * t),
        metal,
        sides=6,
    )
    hook = core.join([hook, spike], "lantern_pole_hook")
    hook.parent = root

    for polygon in hook.data.polygons:
        polygon.use_smooth = False

    finish(hook, wear_boost=1.8)

    _lantern(root, metal)

    return root


def _lantern(root, metal):
    """A six-sided storm lantern: a spired roof, a floor and six bars of
    iron round one flame."""

    roof = EAVE + 0.1
    parts = []

    # The ring the lantern hangs by, through the crook's tip.
    ring = [(math.cos(math.tau * step / 10) * 0.022, 0.0, TIP - 0.037 + math.sin(math.tau * step / 10) * 0.037) for step in range(11)]
    parts.append(core.tube("lantern_ring", ring, 0.0065, metal, sides=5, cap=False))

    # A spire of a roof, six-sided, flared over the eave, with a knob.
    parts.append(
        core.revolve(
            "lantern_roof",
            [
                (0.0, roof + 0.028),
                (0.012, roof + 0.022),
                (0.014, roof + 0.004),
                (0.022, roof - 0.004),
                (0.03, roof - 0.03),
                (CAGE * 1.18, EAVE + 0.006),
                (CAGE * 1.2, EAVE - 0.008),
                (CAGE * 1.02, EAVE - 0.014),
                (0.0, EAVE - 0.012),
            ],
            metal,
            segments=6,
        )
    )
    # A floor like the roof's foot, with a drip under it.
    parts.append(
        core.revolve(
            "lantern_floor",
            [
                (0.0, FLOOR + 0.012),
                (CAGE * 1.08, FLOOR + 0.012),
                (CAGE * 1.1, FLOOR - 0.004),
                (CAGE * 0.8, FLOOR - 0.026),
                (0.03, FLOOR - 0.036),
                (0.012, FLOOR - 0.06),
                (0.0, FLOOR - 0.07),
            ],
            metal,
            segments=6,
        )
    )

    # Six square bars, one at each corner, straight from floor to eave.
    for bar in range(6):
        angle = math.tau * bar / 6
        x, y = math.cos(angle) * CAGE, math.sin(angle) * CAGE
        parts.append(core.tube(f"lantern_bar_{bar}", [(x, y, FLOOR), (x, y, EAVE)], 0.0105, metal, sides=4, twist=0.0))

    cage = core.join(parts, "lantern_pole_lantern")
    cage.location = (HANG, 0, 0)
    cage.parent = root

    for polygon in cage.data.polygons:
        polygon.use_smooth = False

    finish(cage, wear_boost=1.8)

    # The flame is the ferryman's one light: a teardrop FLAME times as wide
    # as a candle's, leaning as it would in a draught, its body filling the
    # cage bar to bar and its tip a few fingers under the roof, so the
    # roof's key frames it. It is as tall as the cage lets it be.
    base = FLOOR + 0.014
    tall = EAVE - FLOOR - 0.045
    flame = core.tube(
        "lantern_pole_flame",
        [(0.022 * (step / 11) ** 2, 0, base + tall * step / 11) for step in range(12)],
        lambda t: FLAME * (0.0025 + 0.0275 * math.sin(min(t * 1.7, 1.0) * math.pi * 0.5) * (1 - t) ** 0.75),
        core.material("accent"),
        root,
        sides=10,
    )
    flame.location = (HANG, 0, 0)
