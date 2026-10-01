"""The penguin: Pesky Penguins.

A squat, upright bird with a heavy brow. The originals are pixel art, so
this one is kept chunky and simple rather than soft and round: a barrel of a
body, a block of a head, flat flippers. It waddles, so the two legs are the
parts that matter most.

It wears a beanie or a crown.
"""

import math

from lib import core, creature


def build(spec):
    root = core.empty("husk")
    body = core.material("body")
    trim = core.material("trim")
    detail = core.material("detail")
    dark = core.material("dark")
    accent = core.material("accent")
    floor = creature.floor(spec)

    # The barrel, widest just below the middle, and the pale belly set into
    # the front of it.
    creature.form(
        "torso",
        [
            ("sphere", (0, 0, -0.1), (0.27, 0.25, 0.31)),
            ("sphere", (0, 0.01, 0.1), (0.21, 0.2, 0.2)),
            ("sphere", (0, 0.06, -0.26), (0.2, 0.2, 0.12)),
        ],
        body,
        root,
        triangles=1600,
    )
    creature.form(
        "belly",
        [
            ("sphere", (0, -0.085, -0.1), (0.2, 0.19, 0.27)),
            ("sphere", (0, -0.07, 0.08), (0.14, 0.15, 0.15)),
        ],
        trim,
        root,
        triangles=900,
    )

    # A stub of a tail, which wags as it walks.
    tail = creature.pivot("tail", root, (0, 0.2, -0.3))

    creature.form("tail_fan", [("sphere", (0, 0.07, -0.02), (0.08, 0.1, 0.035), (math.radians(-25), 0, 0))], body, tail, triangles=220)

    _head(root, body, trim, detail, dark, accent)

    for side, name in ((1, "l"), (-1, "r")):
        # Flippers hang from the shoulder and are thin from front to back.
        arm = creature.pivot(f"arm_{name}", root, (side * 0.24, 0, 0.13))

        creature.form(
            f"flipper_{name}",
            [
                ("sphere", (side * 0.035, 0.0, -0.13), (0.045, 0.085, 0.17), (0, math.radians(side * -9), 0)),
                ("sphere", (side * 0.06, 0.0, -0.27), (0.03, 0.06, 0.06), (0, math.radians(side * -14), 0)),
            ],
            body,
            arm,
            triangles=420,
        )

        # Short legs, almost all foot. The pivot is at the hip, inside the
        # body, so a swing reads as a waddle.
        leg = creature.pivot(f"leg_{name}", root, (side * 0.11, 0.0, -0.3))

        creature.form(
            f"foot_{name}",
            [
                ("sphere", (0, -0.01, floor + 0.3 + 0.055), (0.05, 0.05, 0.06)),
                ("sphere", (0, -0.07, floor + 0.3 + 0.022), (0.075, 0.12, 0.022)),
            ],
            detail,
            leg,
            triangles=360,
        )

    creature.studs(root, spec)
    creature.shadow(root, spec)

    return root


def _head(root, body, trim, detail, dark, accent):
    head = creature.pivot("head", root, (0, 0, 0.2))

    # A block of a head with the brow pulled forward over the eyes.
    creature.form(
        "skull",
        [
            ("cube", (0, 0.0, 0.1), (0.17, 0.16, 0.13)),
            ("sphere", (0, -0.02, 0.12), (0.19, 0.18, 0.15)),
            ("cube", (0, -0.13, 0.17), (0.15, 0.05, 0.035)),
        ],
        body,
        head,
        triangles=1500,
    )
    creature.form(
        "face",
        [
            ("sphere", (0.075, -0.15, 0.09), (0.07, 0.04, 0.065)),
            ("sphere", (-0.075, -0.15, 0.09), (0.07, 0.04, 0.065)),
        ],
        trim,
        head,
        triangles=500,
    )

    # What is left of its eyes: dark sockets with a light far back in them.
    for side in (1, -1):
        creature.form(f"socket_{side}", [("sphere", (side * 0.075, -0.178, 0.1), (0.036, 0.02, 0.036))], dark, head, triangles=120)
        creature.form(f"eye_{side}", [("sphere", (side * 0.075, -0.192, 0.1), (0.014, 0.01, 0.014))], accent, head, triangles=60)

    creature.form(
        "beak_upper",
        [("sphere", (0, -0.21, 0.045), (0.05, 0.085, 0.026)), ("sphere", (0, -0.27, 0.036), (0.028, 0.04, 0.016))],
        detail,
        head,
        triangles=260,
    )

    # The lower beak drops when it shrieks.
    jaw = creature.pivot("jaw", head, (0, -0.16, 0.02))

    creature.form("beak_lower", [("sphere", (0, -0.055, -0.004), (0.04, 0.065, 0.014))], detail, jaw, triangles=160)

    _beanie(creature.pivot("trait_1", head, (0, 0, 0)), detail, core.material("trim"))
    _crown(creature.pivot("trait_2", head, (0, 0, 0)), detail)


def _beanie(parent, wool, band):
    creature.form("beanie", [("sphere", (0, 0.0, 0.21), (0.185, 0.175, 0.1))], wool, parent, triangles=500)
    creature.finish(core.revolve("beanie_band", [(0.186, 0.15), (0.198, 0.17), (0.198, 0.2), (0.186, 0.215)], band, parent, segments=28))
    creature.form("beanie_bobble", [("sphere", (0, 0.0, 0.32), (0.045, 0.045, 0.045))], band, parent, triangles=120)


def _crown(parent, gold):
    points = 7
    profile = []

    for step in range(points * 2 + 1):
        angle = step / (points * 2) * math.tau
        height = 0.34 if step % 2 else 0.27

        profile.append((math.cos(angle) * 0.14, math.sin(angle) * 0.13, height))

    rim = [(x, y, 0.22) for x, y, _ in profile]

    creature.finish(core.loft("crown", [rim, profile], gold, parent, closed=True))
    creature.finish(core.revolve("crown_band", [(0.142, 0.215), (0.152, 0.225), (0.152, 0.245), (0.142, 0.255)], gold, parent, segments=28))
