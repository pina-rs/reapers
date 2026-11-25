"""The chain whip: the hollow's weapon.

A skull dragged on a chain (art bible 6.4). The links, iron vertebrae cast
twice the size a lash's would be, hang plumb from the fist, trail out across
the ground and end in a skull of bone, so nothing the hollow carries ever
rises into the space where its own head should be: it drags one instead.
The skull's eyes are the one light on it. A band of the owner's gold rings
the handle under the grip.
"""

import math

from mathutils import Matrix, Vector

from lib import core
from lib.armoury import band, binding, finish, haft

# Where the lash runs, as (x, z) from the foot of the handle to the barb.
LASH = [(0.0, -0.15), (-0.014, -0.4), (0.0, -0.66), (0.06, -0.87), (0.2, -0.975), (0.42, -1.0), (0.6, -0.975), (0.72, -0.91)]
FIRST_LINK = 0.17
LAST_LINK = 0.085
# The skull at the lash's end: how big, and which way its face turns, out
# toward the bearer's front and up off the ground.
SKULL = 0.085
FACE = Vector((0.45, -0.8, 0.4))


def build():
    """The hollow's weapon: a tapering spine of iron links on a short grip."""

    root = core.empty("weapon_chainWhip")
    metal = core.material("metal")

    haft("whip_handle", root, bottom=-0.12, top=0.15, radius=0.021, bow=0.0)
    binding("whip_grip", root, -0.068, 0.11, radius=0.0255, turns=6)
    band("whip_band", root, -0.086, -0.066, radius=0.0255)

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
    start = 0.0

    # The skull takes the lash's last stretch.
    reach = length - SKULL * 1.6

    while True:
        size = _link_size(start / reach)

        if start + size > reach:
            break

        links.append(_vertebra(len(links), at, start, size, metal))
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

    _skull(root, at(start), _tangent(at, start))

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
                middle + inward * radius * 1.05 + tangent * radius * 0.25,
                middle + inward * radius * 1.4 + tangent * radius * 0.8,
            ],
            lambda t: radius * 0.42 * (1 - 0.9 * t),
            metal,
            sides=5,
            flatten=0.5,
        ),
    ]

    for side in (-1, 1):
        parts.append(
            core.tube(
                f"whip_wing_{index}_{side}",
                [middle + across * side * radius * 0.5, middle + across * side * radius * 1.35 + tangent * radius * 0.4],
                lambda t: radius * 0.32 * (1 - 0.85 * t),
                metal,
                sides=4,
            )
        )

    return core.join(parts, f"whip_vertebra_{index}")


def _skull(root, neck, tangent):
    """The skull the lash ends in: a cranium and a muzzle of bone with a
    jaw under it, the last link through the back of its head at `neck`, its
    face turned out toward `FACE`. Its eyes, in deep sockets, are the
    hollow's one light."""

    bone = core.material("bone")
    face = FACE.normalized()
    up = (Vector((0, 0, 1)) - face * face.z).normalized()
    side = face.cross(up)
    # The back of the head sits on the end of the lash.
    centre = neck + tangent * SKULL * 0.7 + face * SKULL * 0.25 + up * SKULL * 0.35

    def at(forward, rise, across=0.0):
        return centre + face * forward * SKULL + up * rise * SKULL + side * across * SKULL

    parts = []

    # The cranium: a box of planes, rounded only by its facets.
    cranium = core.revolve(
        "whip_cranium",
        [(0.0, -0.95), (0.62, -0.82), (0.95, -0.35), (1.0, 0.15), (0.86, 0.62), (0.5, 0.92), (0.0, 1.0)],
        bone,
        segments=8,
    )
    cranium.data.transform(_frame(centre - face * SKULL * 0.12 + up * SKULL * 0.12, face, up, side, SKULL * 0.92, SKULL * 1.06, SKULL * 0.88))
    parts.append(cranium)

    # The face, from the brow down to the teeth: broad across the cheeks,
    # narrowing to the upper teeth.
    parts.append(
        core.loft(
            "whip_muzzle",
            [
                [at(0.42, rise, across) for across, rise in ((-0.82, 0.28), (0.82, 0.28), (0.78, -0.30), (0.42, -0.80), (-0.42, -0.80), (-0.78, -0.30))],
                [at(0.96, rise, across) for across, rise in ((-0.74, 0.24), (0.74, 0.24), (0.66, -0.26), (0.30, -0.70), (-0.30, -0.70), (-0.66, -0.26))],
            ],
            bone,
            cap_start=True,
            cap_end=True,
        )
    )

    # The jaw: hung a little open under the teeth.
    parts.append(
        core.loft(
            "whip_jaw",
            [
                [at(0.10, rise, across) for across, rise in ((-0.72, -0.50), (0.72, -0.50), (0.62, -0.78), (-0.62, -0.78))],
                [at(0.86, rise, across) for across, rise in ((-0.34, -0.92), (0.34, -0.92), (0.28, -1.14), (-0.28, -1.14))],
            ],
            bone,
            cap_start=True,
            cap_end=True,
        )
    )

    skull = core.join(parts, "whip_skull")
    skull.parent = root

    for polygon in skull.data.polygons:
        polygon.use_smooth = False

    finish(skull, wear_boost=1.6)

    # Two sockets of void in the brow, and in each a coal of light.
    void = core.material("void")
    accent = core.material("accent")
    sockets = []
    eyes = []

    for index, across in enumerate((-0.38, 0.38)):
        socket = at(0.96, -0.02, across)
        sockets.append(core.tube(f"whip_socket_{index}", [socket - face * SKULL * 0.06, socket + face * SKULL * 0.02], SKULL * 0.29, void, sides=6))
        eyes.append(core.tube(f"whip_eye_{index}", [socket - face * SKULL * 0.04, socket + face * SKULL * 0.034], SKULL * 0.15, accent, sides=6))

    # The nose: a notch of void under the eyes.
    nose = at(0.96, -0.40, 0.0)
    sockets.append(core.tube("whip_nose", [nose - face * SKULL * 0.06, nose + face * SKULL * 0.018], SKULL * 0.13, void, sides=3))

    core.join(sockets, "whip_eye_sockets").parent = root
    core.join(eyes, "whip_eyes").parent = root


def _frame(origin, face, up, side, forward_size, up_size, side_size):
    """The matrix that carries a shape made about Z (as `core.revolve`
    turns it) onto a skull: its Z along `face`, scaled to the given sizes."""

    rotation = Matrix((side, up, face)).transposed().to_4x4()

    return Matrix.Translation(origin) @ rotation @ Matrix.Diagonal((side_size, up_size, forward_size, 1.0))
