"""Parts every weapon is assembled from."""

import math

from . import core
from .core import finish


def haft(name, parent, bottom, top, radius=0.019, bow=0.03, mat="wood"):
    """A long pole through the grip, bowed slightly so it never looks turned
    on a lathe."""

    path = core.bezier(
        [
            (bow * 0.4, 0, bottom),
            (-bow * 0.3, 0, bottom * 0.5),
            (0, 0, 0),
            (bow, 0, top * 0.55),
            (0, 0, top),
        ],
        samples=10,
    )

    return finish(
        core.tube(name, path, lambda t: radius * (0.82 + 0.3 * t), core.material(mat), parent, sides=10),
        wear_boost=1.3,
    )


def binding(name, parent, low, high, radius=0.024, turns=7, mat="rope"):
    """Cord wound around a haft."""

    path = []
    steps = turns * 14

    for step in range(steps + 1):
        share = step / steps
        angle = math.tau * turns * share

        path.append((math.cos(angle) * radius, math.sin(angle) * radius, low + (high - low) * share))

    return finish(core.tube(name, path, 0.0065, core.material(mat), parent, sides=6))




def band(name, parent, low, high, radius=0.027, mat="gold", segments=12):
    """The spot band (art bible 6.4): one ring of the owner's light, chamfered
    at both edges, round a haft where the hand meets it, from `low` to
    `high`. Faceted on purpose: each face prints as one plane."""

    chamfer = min(0.004, (high - low) / 4)
    ring = core.revolve(
        name,
        [(0.0, low), (radius * 0.9, low), (radius, low + chamfer), (radius, high - chamfer), (radius * 0.9, high), (0.0, high)],
        core.material(mat),
        parent,
        segments=segments,
    )

    for polygon in ring.data.polygons:
        polygon.use_smooth = False

    return finish(ring, wear_boost=1.6)
