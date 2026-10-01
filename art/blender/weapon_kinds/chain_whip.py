"""The chain whip: the hollow's weapon.

A spine on a handle. Its links hang plumb from the fist, trail out across the
ground and end in a hook, so nothing the hollow carries ever rises into the
space where its head should be.
"""

import math

from mathutils import Vector

from lib import core
from lib.armoury import binding, finish, haft

# Where the lash runs, as (x, z) from the foot of the handle to the barb.
LASH = [(0.0, -0.15), (-0.014, -0.4), (0.0, -0.66), (0.06, -0.87), (0.2, -0.975), (0.42, -1.0), (0.6, -0.975), (0.72, -0.91)]
FIRST_LINK = 0.096
LAST_LINK = 0.036
LIT_JOINTS = 6


def build():
    """The hollow's weapon: a tapering spine of iron links on a short grip."""

    root = core.empty("weapon_chainWhip")
    metal = core.material("metal")

    haft("whip_handle", root, bottom=-0.12, top=0.15, radius=0.021, bow=0.0)
    binding("whip_grip", root, -0.085, 0.11, radius=0.0255, turns=6)

    # A spiked cap over the fist, and a socket under it that the lash hangs
    # from.
    finish(
        core.revolve(
            "whip_pommel",
            [(0.0, 0.25), (0.008, 0.2), (0.03, 0.175), (0.036, 0.155), (0.03, 0.125), (0.022, 0.115)],
            metal,
            root,
            segments=12,
        ),
        wear_boost=1.8,
    )
    finish(
        core.revolve(
            "whip_socket",
            [(0.024, -0.085), (0.034, -0.1), (0.037, -0.125), (0.028, -0.15), (0.0, -0.158)],
            metal,
            root,
            segments=12,
        ),
        wear_boost=1.8,
    )

    at, length = _measured(core.bezier([(x, 0, z) for x, z in LASH], samples=14))
    links = []
    joints = []
    start = 0.0

    while True:
        size = _link_size(start / length)

        if start + size > length:
            break

        links.append(_vertebra(len(links), at, start, size, metal))
        joints.append(start + size * 0.91)
        start += size

    # A cord through every link, so the gaps between them are joints rather
    # than air.
    cord = [at(start * step / 30) for step in range(31)]
    links.append(core.tube("whip_cord", cord, lambda t: 0.0095 - 0.0055 * t, metal, sides=5))

    spine = core.join(links, "whip_spine")
    spine.parent = root

    # Forged, not turned: every link keeps its facets.
    for polygon in spine.data.polygons:
        polygon.use_smooth = False

    finish(spine, wear_boost=1.6)

    # The last few joints are lit from inside.
    lit = []

    for joint in joints[-LIT_JOINTS:]:
        size = _link_size(joint / length)
        tangent = _tangent(at, joint)

        lit.append(
            core.tube(
                "whip_joint",
                [at(joint) - tangent * size * 0.07, at(joint) + tangent * size * 0.07],
                size * 0.2,
                core.material("accent"),
                sides=8,
            )
        )

    core.join(lit, "whip_embers").parent = root

    _barb(root, at(start), _tangent(at, start), metal)

    return root


def _link_size(share):
    """How long a link is, `share` of the way down the lash: they taper."""

    return FIRST_LINK + (LAST_LINK - FIRST_LINK) * share


def _measured(curve):
    """Lets a curve be walked by distance, so links can be laid end to end
    however unevenly the curve was sampled."""

    lengths = [0.0]

    for near, far in zip(curve, curve[1:]):
        lengths.append(lengths[-1] + (far - near).length)

    def at(distance):
        distance = max(0.0, min(lengths[-1], distance))

        for index in range(1, len(curve)):
            if distance <= lengths[index]:
                share = (distance - lengths[index - 1]) / (lengths[index] - lengths[index - 1])

                return curve[index - 1].lerp(curve[index], share)

        return curve[-1].copy()

    return at, lengths[-1]


def _tangent(at, distance):
    """The direction the lash is running in, `distance` along it."""

    return (at(distance + 0.01) - at(distance - 0.01)).normalized()


def _vertebra(index, at, start, size, metal):
    """One link of the lash, shaped like a vertebra: a waisted body, a spur
    raked toward the barb and a short wing on either side."""

    middle = at(start + size * 0.41)
    tangent = _tangent(at, start + size * 0.41)
    across = Vector((0, 1, 0))
    inward = tangent.cross(across)
    radius = size * 0.33

    parts = [
        core.tube(
            f"whip_link_{index}",
            [at(start + size * 0.82 * step / 4) for step in range(5)],
            # Flared end plates around a narrow waist.
            lambda t: radius * (0.64 + 0.36 * abs(2 * t - 1) ** 2),
            metal,
            sides=8,
            flatten=0.82,
        ),
        core.tube(
            f"whip_spur_{index}",
            [
                middle + inward * radius * 0.5,
                middle + inward * radius * 1.4 + tangent * radius * 0.4,
                middle + inward * radius * 2.0 + tangent * radius * 1.3,
            ],
            lambda t: radius * 0.55 * (1 - 0.9 * t),
            metal,
            sides=5,
            flatten=0.5,
        ),
    ]

    for side in (-1, 1):
        parts.append(
            core.tube(
                f"whip_wing_{index}_{side}",
                [middle + across * side * radius * 0.5, middle + across * side * radius * 1.8 + tangent * radius * 0.6],
                lambda t: radius * 0.4 * (1 - 0.85 * t),
                metal,
                sides=4,
            )
        )

    return core.join(parts, f"whip_vertebra_{index}")


def _barb(root, tip, tangent, metal):
    """The hook the lash ends in, curled up and back like a sting so that it
    catches on the way out."""

    inward = tangent.cross(Vector((0, 1, 0)))
    centre = tip + inward * 0.075
    spine = []
    edge = []

    for step in range(13):
        share = step / 12
        angle = share * 2.8
        radial = tangent * math.sin(angle) - inward * math.cos(angle)
        width = 0.026 * (1 - share) ** 0.7
        outer = centre + radial * (0.075 + width)
        inner = centre + radial * (0.075 - width * 0.6)

        spine.append((outer.x, outer.z))
        edge.append((inner.x, inner.z))

    finish(core.blade("whip_barb", spine, edge, 0.016, metal, root), wear_boost=2.0)
