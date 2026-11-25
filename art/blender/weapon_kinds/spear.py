"""The spear: the chooser's weapon.

A long leaf of bone on a haft of wood taller than its bearer, made for one
strike (art bible 6.4). The leaf is big, half again a spearhead's length,
swelling early and running a long way to its point, its faces ground in
clean planes to a rib, with a barb hooked back either side of its foot so
the tip has a second note. A broad stripe of light runs down the rib on both
faces, the fuller and the one light on it, still found as `spear_edge`. A
proud band of the owner's gold rings the haft where the hand meets it, and
two broad streamers and a feather hang under the leaf.
"""

import math

from mathutils import Vector

from lib import core
from lib.armoury import band, binding, finish, haft

BLADE_FOOT = 0.9
BLADE_TIP = 1.62
BUTT = -0.97

# Which way loose things hang. The chooser carries the spear leaning, so they
# trail a little off the haft rather than straight down it.
HANG = Vector((-0.3, 0.0, -0.954))


def build():
    """The chooser's weapon: a leaf blade, a bound collar and a butt spike."""

    root = core.empty("weapon_spear")
    metal = core.material("metal")
    bone = core.material("bone")

    haft("spear_haft", root, bottom=BUTT, top=0.86, radius=0.021, bow=0.012)
    binding("spear_grip", root, -0.1, 0.14, radius=0.0275)
    band("spear_band", root, 0.145, 0.195, radius=0.036)

    blade = core.join([_blade("spear_leaf", bone, None), *_barbs(bone)], "spear_blade")
    blade.parent = root

    for polygon in blade.data.polygons:
        polygon.use_smooth = False

    finish(blade, wear_boost=2.0)

    # The fuller: a broad stripe of light lying over the rib of each face,
    # from the foot of the leaf most of the way to its point.
    seams = []

    for face in (-1, 1):
        rings = []

        for t in _steps(0.05, 0.8, 12):
            width = _blade_width(t)
            rib = _rib(t)
            height = _blade_height(t)
            # Widest a third of the way up, and never wider than a third of
            # the leaf.
            half = min(0.016 * math.sin(math.pi * min(1.0, (t - 0.02) / 0.82)) ** 0.5 + 0.003, width * 0.36)
            # The face falls from the rib to the edge in a straight line.
            fall = rib * (1 - half / width)
            proud = 0.0012
            rings.append(
                [
                    (-half, face * (fall + proud), height),
                    (0.0, face * (rib + proud), height),
                    (half, face * (fall + proud), height),
                    (half * 0.8, face * (fall - 0.0015), height),
                    (0.0, face * (rib - 0.002), height),
                    (-half * 0.8, face * (fall - 0.0015), height),
                ]
            )

        seams.append(core.loft(f"spear_edge_{face}", rings, core.material("accent"), cap_start=True, cap_end=True))

    edge = core.join(seams, "spear_edge")
    edge.parent = root

    for polygon in edge.data.polygons:
        polygon.use_smooth = False

    # The socket the blade grows out of, flared where it grips the haft.
    finish(
        core.revolve(
            "spear_socket",
            [(0.0, 0.78), (0.031, 0.782), (0.034, 0.795), (0.03, 0.81), (0.027, 0.87), (0.019, 0.93), (0.011, 0.99), (0.0, 1.04)],
            metal,
            root,
            segments=16,
        ),
        wear_boost=1.8,
    )
    binding("spear_collar", root, 0.67, 0.77, radius=0.0295, turns=5)
    _tokens(root)

    # A spike at the other end, to plant the spear or to strike backward.
    spike = core.revolve(
        "spear_butt",
        [(0.0, 0.115), (0.026, 0.11), (0.029, 0.095), (0.025, 0.08), (0.023, 0.01), (0.014, -0.03), (0.0, -0.08)],
        metal,
        root,
        segments=14,
    )

    # The haft is bowed, so its foot is not quite under the grip.
    spike.location = (0.005, 0, BUTT)
    finish(spike, wear_boost=1.8)

    return root


def _steps(start, end, count):
    """`count` evenly spaced values from `start` to `end`."""

    return [start + (end - start) * step / (count - 1) for step in range(count)]


def _blade_height(t):
    """How high the blade is, `t` of the way from its foot to its point."""

    return BLADE_FOOT + (BLADE_TIP - BLADE_FOOT) * t


def _blade_width(t):
    """Half the blade's width, `t` of the way from its foot to its point: a
    leaf that swells early and then runs a long way to nothing."""

    return 0.018 * (1 - t) + 0.076 * math.sin(math.pi * t**0.72) ** 1.1


def _rib(t):
    """How far the rib stands out of the blade's middle, `t` of the way up."""

    return 0.0145 * (1 - t) ** 0.7 + 0.0014


def _blade(name, mat, parent):
    """A leaf blade lying in the XZ plane, each face two planes meeting at
    a rib down its middle: a flattened diamond."""

    rings = []

    for t in _steps(0.0, 1.0, 16):
        width = max(_blade_width(t), 0.0008)
        rib = _rib(t)
        height = _blade_height(t)

        rings.append([(-width, 0, height), (0, -rib, height), (width, 0, height), (0, rib, height)])

    return core.loft(name, rings, mat, parent, cap_start=True, cap_end=True)


def _barbs(mat):
    """A barb either side of the leaf's foot, swept out and hooked back
    down toward the hand: a winged spear's lugs, cut in planes."""

    barbs = []

    for side in (-1, 1):
        spine = [(side * 0.016, BLADE_FOOT + 0.07), (side * 0.07, BLADE_FOOT + 0.045), (side * 0.112, BLADE_FOOT - 0.005), (side * 0.118, BLADE_FOOT - 0.06)]
        edge = [(side * 0.016, BLADE_FOOT + 0.012), (side * 0.056, BLADE_FOOT + 0.002), (side * 0.088, BLADE_FOOT - 0.02), (side * 0.118, BLADE_FOOT - 0.06)]
        barbs.append(core.blade(f"spear_barb_{side}", spine, edge, 0.02, mat))

    return barbs


def _tokens(root):
    """What the chooser ties below the blade: two broad streamers of red
    cloth, swallow-tailed, and one big feather."""

    streamers = []

    for index, (turn, length, width, twist) in enumerate(((0.6, 0.36, 0.036, 1.4), (2.9, 0.27, 0.03, -1.1))):
        start = Vector((math.cos(turn) * 0.03, math.sin(turn) * 0.03, 0.74 - 0.02 * index))
        out = Vector((math.cos(turn), math.sin(turn), 0))
        path = core.bezier([start, start + out * 0.05 + HANG * length * 0.35, start + out * 0.07 + HANG * length * 0.7, start + out * 0.06 + HANG * length], samples=3)
        streamers.append(_streamer(f"spear_thread_{index}", path, out, width, twist))

    tied = core.join(streamers, "spear_threads")
    tied.parent = root

    for polygon in tied.data.polygons:
        polygon.use_smooth = False

    finish(tied)

    start = Vector((math.cos(4.2) * 0.033, math.sin(4.2) * 0.033, 0.7))
    out = Vector((math.cos(4.2), math.sin(4.2), 0))
    plume = _feather("spear_feathers", start, start + out * 0.08 + HANG * 0.36, out, 0.05)
    plume.parent = root
    finish(core.shade_smooth(plume, angle=50), wear_boost=1.4)


def _streamer(name, path, facing, width, twist):
    """A strip of cloth `width` wide hanging down `path`, turning `twist`
    radians about itself as it falls, its end cut into a swallow's tail."""

    points = [Vector(point) for point in path]
    rings = []
    count = len(points)

    for index, centre in enumerate(points):
        share = index / (count - 1)
        ahead = points[min(index + 1, count - 1)]
        behind = points[max(index - 1, 0)]
        tangent = (ahead - behind).normalized()
        side = tangent.cross(facing).normalized()
        normal = side.cross(tangent)
        angle = twist * share
        side, normal = side * math.cos(angle) + normal * math.sin(angle), normal * math.cos(angle) - side * math.sin(angle)
        half = width / 2 * (0.7 + 0.3 * share)
        thick = 0.0022
        # The last ring's middle is drawn back up the strip: the tail's cut.
        middle = centre - tangent * (0.06 * width / 0.036 if index == count - 1 else 0.0)
        rings.append(
            [
                centre - side * half - normal * thick,
                middle - normal * thick,
                centre + side * half - normal * thick,
                centre + side * half + normal * thick,
                middle + normal * thick,
                centre - side * half + normal * thick,
            ]
        )

    return core.loft(name, rings, core.material("cloth_alt"), cap_start=True, cap_end=True)


def _feather(name, base, tip, facing, width):
    """A small hanging feather: a leaf either side of a spine, on a bare quill."""

    axis = tip - base
    forward = axis.normalized()
    side = forward.cross(facing).normalized()
    normal = side.cross(forward)
    rings = []

    for t in _steps(0.0, 1.0, 10):
        centre = base + axis * t + normal * (0.1 * axis.length * 4 * t * (1 - t))
        shaft = 0.0028 * (1 - 0.75 * t)
        along = max(0.0, (t - 0.16) / 0.84)
        vane = width * min(1.0, along / 0.25) ** 0.7 * (1 - along**2.4) ** 0.6

        rings.append(
            [
                centre - side * (shaft + vane) - normal * vane * 0.3,
                centre + normal * shaft * 1.3,
                centre + side * (shaft + vane * 0.7) - normal * vane * 0.2,
                centre - normal * shaft,
            ]
        )

    return core.loft(name, rings, core.material("feather"), cap_start=True, cap_end=True)
