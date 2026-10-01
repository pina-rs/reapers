"""The spear: the chooser's weapon.

A long leaf of a blade on a haft taller than its bearer, made for one strike.
The blade's edges, up to the point, are the one light on it.
"""

import math

from mathutils import Vector

from lib import core
from lib.armoury import binding, finish, haft

BLADE_FOOT = 0.9
BLADE_TIP = 1.45
BUTT = -0.97

# Which way loose things hang. The chooser carries the spear leaning, so they
# trail a little off the haft rather than straight down it.
HANG = Vector((-0.3, 0.0, -0.954))


def build():
    """The chooser's weapon: a leaf blade, a bound collar and a butt spike."""

    root = core.empty("weapon_spear")
    metal = core.material("metal")

    haft("spear_haft", root, bottom=BUTT, top=0.86, radius=0.021, bow=0.012)
    binding("spear_grip", root, -0.1, 0.14, radius=0.0275)

    finish(core.shade_smooth(_blade("spear_blade", metal, root), angle=28), wear_boost=2.0)

    # The edges are the chooser's one light, brightest where they meet.
    core.tube(
        "spear_edge",
        [(-_blade_width(t), 0, _blade_height(t)) for t in _steps(0.42, 1.0, 14)]
        + [(_blade_width(t), 0, _blade_height(t)) for t in _steps(0.96, 0.42, 13)],
        lambda t: 0.0022 + 0.0034 * (1 - abs(2 * t - 1)) ** 1.5,
        core.material("accent"),
        root,
        sides=6,
    )

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

    return 0.012 * (1 - t) + 0.047 * math.sin(math.pi * t**0.72) ** 1.1


def _blade(name, mat, parent):
    """A leaf blade lying in the XZ plane, with a rib down its middle."""

    rings = []

    for t in _steps(0.0, 1.0, 23):
        width = max(_blade_width(t), 0.0008)
        rib = 0.0105 * (1 - t) ** 0.7 + 0.0012
        height = _blade_height(t)

        rings.append(
            [
                (-width, 0, height),
                (-width * 0.2, -rib * 0.4, height),
                (0, -rib, height),
                (width * 0.2, -rib * 0.4, height),
                (width, 0, height),
                (width * 0.2, rib * 0.4, height),
                (0, rib, height),
                (-width * 0.2, rib * 0.4, height),
            ]
        )

    return core.loft(name, rings, mat, parent, cap_start=True, cap_end=True)


def _tokens(root):
    """What the chooser ties below the blade: red thread, and a feather for
    each side of the choice."""

    threads = []

    for index, (turn, drop) in enumerate(((0.4, 0.34), (2.2, 0.25), (3.5, 0.42), (4.6, 0.2), (5.5, 0.3))):
        start = Vector((math.cos(turn) * 0.031, math.sin(turn) * 0.031, 0.71 + 0.01 * index))
        out = Vector((math.cos(turn), math.sin(turn), 0)) * 0.035
        fall = HANG * drop

        threads.append(
            core.tube(
                f"spear_thread_{index}",
                core.bezier([start, start + out + fall * 0.3, start + out * 1.3 + fall * 0.68, start + out + fall], samples=5),
                # Each thread ends in a knot.
                lambda t: 0.0032 + 0.0045 * max(0.0, 1 - abs(t - 0.95) / 0.05) ** 2,
                core.material("cloth_alt"),
                sides=5,
            )
        )

    tied = core.join(threads, "spear_threads")
    tied.parent = root
    finish(tied)

    feathers = []

    for index, (turn, length, width) in enumerate(((3.3, 0.34, 0.034), (4.5, 0.26, 0.028))):
        start = Vector((math.cos(turn) * 0.033, math.sin(turn) * 0.033, 0.69))
        out = Vector((math.cos(turn), math.sin(turn), 0))

        feathers.append(_feather(f"spear_feather_{index}", start, start + out * 0.07 + HANG * length, out, width))

    plume = core.join(feathers, "spear_feathers")
    plume.parent = root
    finish(core.shade_smooth(plume, angle=50), wear_boost=1.4)


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
