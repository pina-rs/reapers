"""The greatsword: a heavy blade no class starts with.

A straight blade as long as a leg, broad at the guard and running to a
point, with a wide crossguard and a heavy pommel to swing it by. The edges
are the one light on it, and the fuller down its middle glows faintly.
"""

import math

from lib import core
from lib.armoury import binding, finish

GUARD = 0.2
BLADE_TIP = 1.42
POMMEL = -0.27


def build():
    """A long, broad blade on a two-handed grip."""

    root = core.empty("weapon_greatsword")
    metal = core.material("metal")

    finish(core.shade_smooth(_blade("greatsword_blade", metal, root), angle=30), wear_boost=2.0)

    # Both edges are lit, brightest toward the point.
    for side in (-1, 1):
        core.tube(
            f"greatsword_edge_{'l' if side < 0 else 'r'}",
            [(side * _half_width(t), 0, _height(t)) for t in _steps(0.0, 0.985, 18)],
            lambda t: 0.0022 + 0.003 * t,
            core.material("accent"),
            root,
            sides=6,
        )

    # The fuller: a groove down the middle of each face, faintly lit.
    for face in (-1, 1):
        fuller = core.tube(
            f"greatsword_fuller_{'f' if face < 0 else 'b'}",
            [(0, face * 0.006, _height(t)) for t in _steps(0.02, 0.6, 8)],
            lambda t: 0.0035 * (1 - 0.6 * t),
            core.material("accent"),
            root,
            sides=5,
            flatten=0.35,
        )
        fuller.scale = (1.0, 0.6, 1.0)

    # A crossguard that bends a little toward the point at its ends.
    finish(
        core.tube(
            "greatsword_guard",
            core.bezier([(-0.21, 0, GUARD + 0.03), (-0.1, 0, GUARD - 0.008), (0.1, 0, GUARD - 0.008), (0.21, 0, GUARD + 0.03)], samples=10),
            lambda t: 0.017 + 0.009 * abs(2 * t - 1) ** 3,
            metal,
            root,
            sides=8,
            flatten=0.7,
        ),
        wear_boost=1.8,
    )

    # A long grip for two hands, wrapped, and a heavy pommel below it.
    finish(
        core.revolve(
            "greatsword_grip",
            [(0.0, GUARD - 0.02), (0.019, GUARD - 0.025), (0.021, 0.0), (0.019, POMMEL + 0.07), (0.0, POMMEL + 0.065)],
            core.material("wood"),
            root,
            segments=12,
        ),
        wear_boost=1.2,
    )
    binding("greatsword_wrap", root, POMMEL + 0.08, GUARD - 0.04, radius=0.021, turns=11)
    finish(
        core.revolve(
            "greatsword_pommel",
            [(0.0, POMMEL + 0.07), (0.022, POMMEL + 0.065), (0.04, POMMEL + 0.04), (0.042, POMMEL + 0.015), (0.03, POMMEL - 0.012), (0.0, POMMEL - 0.02)],
            metal,
            root,
            segments=14,
        ),
        wear_boost=1.8,
    )

    return root


def _steps(start, end, count):
    """`count` evenly spaced values from `start` to `end`."""

    return [start + (end - start) * step / (count - 1) for step in range(count)]


def _height(t):
    """How high the blade is, `t` of the way from the guard to the point."""

    return GUARD + (BLADE_TIP - GUARD) * t


def _half_width(t):
    """Half the blade's width: broad at the guard, narrowing slowly, then
    running quickly to the point over the last fifth."""

    taper = 0.052 - 0.014 * t

    return taper * (1.0 if t < 0.8 else math.cos((t - 0.8) / 0.2 * math.pi / 2) ** 0.8)


def _blade(name, mat, parent):
    """A blade in the XZ plane, thick down its middle and thin at its edges."""

    rings = []

    for t in _steps(0.0, 1.0, 20):
        width = max(_half_width(t), 0.0008)
        middle = 0.0085 * (1 - 0.7 * t) + 0.0008
        height = _height(t)

        rings.append(
            [
                (-width, 0, height),
                (-width * 0.45, -middle * 0.85, height),
                (0, -middle, height),
                (width * 0.45, -middle * 0.85, height),
                (width, 0, height),
                (width * 0.45, middle * 0.85, height),
                (0, middle, height),
                (-width * 0.45, middle * 0.85, height),
            ]
        )

    return core.loft(name, rings, mat, parent, cap_start=True, cap_end=True)
