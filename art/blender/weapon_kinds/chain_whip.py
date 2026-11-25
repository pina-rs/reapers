"""The chain whip: the hollow's weapon.

A raven's skull dragged on a chain (art bible 6.4). The links, iron
vertebrae cast twice the size a lash's would be, fall from the fist and
swing out away from the body in a long curve that lies along the ground,
outside the robe's line, so nothing the hollow carries ever rises into the
space where its own head should be. The chain ends in the skull of a raven,
bone, whose long beak curls up into the hook the whip has always ended in
(`whip_barb`): a hook at any distance, and a raven only close to. It is a
bird's skull, not a man's: the hollow's own skull lantern stays the one
human skull it carries. The coals in its eye sockets are the one light on it
(`whip_embers`). A band of the owner's gold rings the handle under the grip.
"""

import math

from mathutils import Matrix, Vector

from lib import core
from lib.armoury import band, binding, finish, haft

# Where the lash runs, as (x, z) from the foot of the handle to the skull:
# down from the fist, out away from the body (-X) and along the ground.
LASH = [(0.0, -0.15), (-0.03, -0.38), (-0.09, -0.62), (-0.19, -0.83), (-0.33, -0.96), (-0.5, -1.0), (-0.64, -0.99)]
FIRST_LINK = 0.17
LAST_LINK = 0.085
# The skull: its cranium's length, and its beak's.
CRANIUM = 0.085
BEAK = 0.27


def build():
    """The hollow's weapon: a tapering spine of iron links on a short grip."""

    root = core.empty("weapon_chainWhip")
    metal = core.material("metal")

    haft("whip_handle", root, bottom=-0.12, top=0.15, radius=0.021, bow=0.0)
    binding("whip_grip", root, -0.058, 0.11, radius=0.0255, turns=6)
    band("whip_band", root, -0.09, -0.06, radius=0.034)

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
    reach = length - CRANIUM * 0.4

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
    """One link of the lash, shaped like a vertebra: a body flared at its
    ends round a narrow waist, a bow tie in outline, and one spur raked
    toward the skull on the outside of the lash's curve, so the chain has
    one clean edge and one spined one."""

    middle = at(start + size * 0.41)
    tangent = _tangent(at, start + size * 0.41)
    # Away from the body while it hangs, up off the ground where it lies.
    outward = Vector((0, 1, 0)).cross(tangent)
    radius = size * 0.33

    parts = [
        core.tube(
            f"whip_link_{index}",
            [at(start + size * 0.82 * step / 4) for step in range(5)],
            # Flared end plates around a narrow waist.
            lambda t: radius * (0.58 + 0.42 * abs(2 * t - 1) ** 2),
            metal,
            sides=6,
            flatten=0.8,
        ),
        core.tube(
            f"whip_spur_{index}",
            [
                middle + outward * radius * 0.5,
                middle + outward * radius * 1.15 + tangent * radius * 0.3,
                middle + outward * radius * 1.55 + tangent * radius * 0.95,
            ],
            lambda t: radius * 0.4 * (1 - 0.9 * t),
            metal,
            sides=4,
            flatten=0.55,
        ),
    ]

    return core.join(parts, f"whip_vertebra_{index}")


def _skull(root, neck, tangent):
    """The raven's skull the lash ends in, the last link through the back of
    its head at `neck`, facing on along the lash (`tangent`): a cranium of
    bone cut in planes, and a beak twice its length that runs on and curls up
    and back into a hook, thick at its root and keen at its point. Its eyes,
    on either side of the cranium as a bird's are, are sockets of void with a
    coal of light in each: the hollow's one light."""

    bone = core.material("bone")
    forward = Vector((tangent.x, 0.0, tangent.z)).normalized()
    up = Vector((-forward.z, 0.0, forward.x))

    if up.z < 0:
        up = -up

    side = forward.cross(up)
    centre = neck + forward * CRANIUM * 0.62 + up * CRANIUM * 0.18

    def at(along, rise):
        return centre + forward * along + up * rise

    # The cranium: a rounded box of eight planes, deepest behind the eyes,
    # with a brow that overhangs the root of the beak.
    cranium = core.revolve(
        "whip_cranium",
        [(0.0, -1.0), (0.62, -0.84), (0.94, -0.36), (1.0, 0.14), (0.86, 0.58), (0.58, 0.86), (0.34, 1.0)],
        bone,
        segments=8,
    )
    rotation = Matrix((side, up, forward)).transposed().to_4x4()
    cranium.data.transform(Matrix.Translation(centre) @ rotation @ Matrix.Diagonal((CRANIUM * 0.36, CRANIUM * 0.42, CRANIUM * 0.56, 1.0)))

    # The beak, in the lash's own plane: on past the brow, then curling up
    # and back over itself, its outer curve the spine and its inner the edge.
    root_at = CRANIUM * 0.4
    spine = [(0.0, -0.36), (0.40, -0.40), (0.78, -0.26), (0.98, 0.10), (0.94, 0.52), (0.72, 0.80), (0.50, 0.84)]
    edge = [(0.0, 0.08), (0.36, -0.06), (0.62, -0.02), (0.76, 0.16), (0.76, 0.46), (0.62, 0.66), (0.50, 0.84)]

    def plane(points):
        return [(lambda p: (p.x, p.z))(at(root_at + along * BEAK, rise * BEAK * 0.6)) for along, rise in points]

    spine_curve = core.bezier(plane(spine), samples=3)
    edge_curve = core.bezier(plane(edge), samples=3)
    count = min(len(spine_curve), len(edge_curve))
    beak = core.blade("whip_beak", [(p.x, p.y) for p in spine_curve[:count]], [(p.x, p.y) for p in edge_curve[:count]], CRANIUM * 0.46, bone)

    skull = core.join([cranium, beak], "whip_barb")
    skull.parent = root

    for polygon in skull.data.polygons:
        polygon.use_smooth = False

    finish(skull, wear_boost=1.8)

    # An eye each side, behind the brow: a socket of void and in it a coal.
    void = core.material("void")
    accent = core.material("accent")
    sockets = []
    embers = []

    for flank in (-1, 1):
        eye = at(CRANIUM * 0.12, CRANIUM * 0.1)
        surface = eye + side * flank * CRANIUM * 0.34
        sockets.append(core.tube(f"whip_socket_{flank}", [surface - side * flank * 0.012, surface + side * flank * 0.006], CRANIUM * 0.17, void, sides=6))
        embers.append(core.tube(f"whip_ember_{flank}", [surface - side * flank * 0.008, surface + side * flank * 0.0085], CRANIUM * 0.085, accent, sides=6))

    core.join(sockets, "whip_eye_sockets").parent = root
    core.join(embers, "whip_embers").parent = root
