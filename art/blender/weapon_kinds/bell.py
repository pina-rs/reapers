"""The bell: the mourner's weapon.

A heavy handbell carried mouth down, below the hand that holds it. The light
inside its mouth is the one light on it.
"""

import math

from mathutils import Vector, noise

from lib import core
from lib.armoury import finish

LIP = -0.36


def build():
    """The mourner's weapon: a handbell that hangs below the hand."""

    root = core.empty("weapon_bell")
    metal = core.material("metal")
    wood = core.material("wood")

    # A turned handle: a pommel, a waist for the fingers and a foot.
    finish(
        core.revolve(
            "bell_handle",
            [
                (0.0, 0.122),
                (0.01, 0.121),
                (0.019, 0.118),
                (0.027, 0.112),
                (0.032, 0.104),
                (0.034, 0.095),
                (0.033, 0.087),
                (0.029, 0.08),
                (0.024, 0.075),
                (0.02, 0.071),
                (0.019, 0.064),
                (0.025, 0.054),
                (0.029, 0.03),
                (0.03, 0.0),
                (0.028, -0.03),
                (0.023, -0.05),
                (0.02, -0.058),
                (0.0, -0.058),
            ],
            wood,
            root,
            segments=28,
        ),
        wear_boost=1.4,
    )

    # The yoke that fixes the handle to the crown of the bell.
    finish(
        core.revolve(
            "bell_yoke",
            [
                (0.019, -0.05),
                (0.027, -0.052),
                (0.031, -0.059),
                (0.027, -0.066),
                (0.021, -0.07),
                (0.02, -0.08),
                (0.028, -0.086),
                (0.036, -0.094),
                (0.036, -0.1),
            ],
            metal,
            root,
            segments=20,
        ),
        wear_boost=1.8,
    )

    def worn(angle, radius, height):
        """Bites a few chips out of the lip and pulls the mouth slightly out
        of round, the way a bell goes after it has been set down hard for
        long enough."""

        rim = max(0.0, min(1.0, (LIP + 0.07 - height) / 0.07))
        chip = min(1.0, max(0.0, noise.noise(Vector((math.cos(angle) * 2.4, math.sin(angle) * 2.4, 3.0))) - 0.3) * 5)
        oval = 1 + 0.03 * rim * math.cos(angle * 2 + 0.7)

        return radius * oval, height + 0.016 * rim**3 * chip

    # Out from the crown, over the shoulder, down the waist and around a
    # thick lip, then back up the inside.
    finish(
        core.shade_smooth(
            core.revolve(
                "bell_body",
                [
                    (0.0, -0.092),
                    (0.034, -0.094),
                    (0.052, -0.1),
                    (0.062, -0.112),
                    (0.066, -0.126),
                    (0.071, -0.13),
                    (0.071, -0.138),
                    (0.066, -0.142),
                    (0.067, -0.18),
                    (0.071, -0.225),
                    (0.079, -0.268),
                    (0.091, -0.304),
                    (0.103, -0.326),
                    (0.112, -0.334),
                    (0.117, -0.346),
                    (0.114, LIP + 0.003),
                    (0.106, LIP),
                    (0.097, LIP + 0.005),
                    (0.09, -0.335),
                    (0.078, -0.3),
                    (0.066, -0.262),
                    (0.058, -0.2),
                    (0.055, -0.13),
                    (0.0, -0.108),
                ],
                metal,
                root,
                segments=48,
                shape=worn,
            ),
            angle=50,
        ),
        wear_boost=2.0,
    )

    # The clapper hangs to one side, where it came to rest.
    rest = Vector((0.03, 0.008, LIP - 0.028))
    finish(
        core.tube(
            "bell_clapper",
            core.bezier([(0, 0, -0.11), (0.008, 0.002, -0.2), (0.021, 0.006, -0.3), rest], samples=6),
            # A thin rod that ends in a ball.
            lambda t: 0.0055 + 0.019 * math.sqrt(max(0.0, 1 - ((t - 0.88) / 0.115) ** 2)),
            wood,
            root,
            sides=10,
        ),
        wear_boost=1.5,
    )

    # The mourner's one light fills the mouth and sags just below the lip,
    # so it shows as a sliver from the side, with the clapper dark against
    # it, and through every chip.
    core.revolve(
        "bell_light",
        [(0.0, LIP - 0.007), (0.04, LIP - 0.006), (0.075, LIP - 0.002), (0.093, LIP + 0.006), (0.09, LIP + 0.03)],
        core.material("accent"),
        root,
        segments=32,
        shape=lambda angle, radius, height: (radius * (1 + 0.03 * math.cos(angle * 2 + 0.7)), height),
    )

    return root
