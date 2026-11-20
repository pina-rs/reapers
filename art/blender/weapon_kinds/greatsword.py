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

    # Swept gothic crossguard with flared quillon finials.
    finish(
        core.shade_smooth(
            core.tube(
                "greatsword_guard",
                core.bezier([(-0.225, 0, GUARD + 0.042), (-0.11, 0, GUARD - 0.010), (0.0, 0, GUARD - 0.014), (0.11, 0, GUARD - 0.010), (0.225, 0, GUARD + 0.042)], samples=10),
                lambda t: 0.015 + 0.012 * abs(2 * t - 1) ** 3 + 0.009 * (1.0 - abs(2 * t - 1)) ** 4,
                metal,
                root,
                sides=8,
                flatten=0.65,
            ),
            angle=42,
        ),
        wear_boost=1.8,
    )

    # A long grip for two hands, wrapped, and a faceted gothic pommel below it.
    finish(
        core.revolve(
            "greatsword_grip",
            [(0.0, GUARD - 0.02), (0.019, GUARD - 0.025), (0.021, 0.0), (0.019, POMMEL + 0.07), (0.0, POMMEL + 0.065)],
            core.material("wood"),
            root,
            segments=14,
        ),
        wear_boost=1.2,
    )
    binding("greatsword_wrap", root, POMMEL + 0.08, GUARD - 0.04, radius=0.021, turns=11)
    finish(
        core.shade_smooth(
            core.revolve(
                "greatsword_pommel",
                [
                    (0.0, POMMEL + 0.074),
                    (0.020, POMMEL + 0.068),
                    (0.032, POMMEL + 0.056),
                    (0.046, POMMEL + 0.036),
                    (0.046, POMMEL + 0.016),
                    (0.030, POMMEL - 0.008),
                    (0.014, POMMEL - 0.022),
                    (0.0, POMMEL - 0.034),
                ],
                metal,
                root,
                segments=8,
            ),
            angle=40,
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

    taper = 0.054 - 0.015 * t

    return taper * (1.0 if t < 0.8 else math.cos((t - 0.8) / 0.2 * math.pi / 2) ** 0.8)


def _blade(name, mat, parent):
    """A hollow-ground diamond-section blade with recessed fuller groove."""

    rings = []

    for t in _steps(0.0, 1.0, 24):
        width = max(_half_width(t), 0.0008)
        middle = 0.0105 * (1 - 0.68 * t) + 0.0009
        fuller = 0.68 if t < 0.58 else 1.05
        height = _height(t)

        rings.append(
            [
                (-width, 0, height),
                (-width * 0.58, -middle * 0.62, height),
                (-width * 0.22, -middle * 1.05, height),
                (0, -middle * fuller, height),
                (width * 0.22, -middle * 1.05, height),
                (width * 0.58, -middle * 0.62, height),
                (width, 0, height),
                (width * 0.58, middle * 0.62, height),
                (width * 0.22, middle * 1.05, height),
                (0, middle * fuller, height),
                (-width * 0.22, middle * 1.05, height),
                (-width * 0.58, middle * 0.62, height),
            ]
        )

    return core.loft(name, rings, mat, parent, cap_start=True, cap_end=True)
