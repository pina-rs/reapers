"""The greatsword: a heavy blade no class starts with.

Its one idea is the coffin (art bible 6.4 asks each Bought weapon for one):
the blade is a coffin's lid, in plan and in section. In plan it is six
straight sides: a narrow head at the guard, shoulders nearly twice as wide a
seventh of the way up at a hard angle, a long straight taper, and a foot
clipped off at a slant instead of a point. In section it is a lid: a flat
raised panel down the middle and a broad chamfer to each edge, thick enough
at the panel to survive a turn. A broad seam of light runs down the panel of
each face, the fuller and the one light on it. The crossguard is a square
bar whose ends turn down and are nailed, a lid's cross; a collar of the
owner's gold closes the head of the blade, and the grip is wrapped in rope.
"""

from lib import core
from lib.armoury import binding, finish

GUARD = 0.2
BLADE_TIP = 1.44
POMMEL = -0.29

# The coffin in plan, as (share of the way up the blade, half its width):
# its head at the guard, the shoulders, and the foot's corners. The foot is
# clipped at a slant: its corner across the body (+X) is the higher one.
OUTLINE = ((0.0, 0.05), (0.035, 0.05), (0.14, 0.098), (0.955, 0.044), (1.0, 0.044))
CLIP = 0.955

# The seam of light down each face's panel, from and to these shares.
FULLER = (0.19, 0.86)


def build():
    """A coffin-shaped blade on a two-handed grip."""

    root = core.empty("weapon_greatsword")
    metal = core.material("metal")

    finish(_blade("greatsword_blade", metal, root), wear_boost=2.0)

    # The fuller: a broad seam of light lying on the panel of each face.
    for face in (-1, 1):
        rings = []

        for t in _steps(FULLER[0], FULLER[1], 8):
            share = (t - FULLER[0]) / (FULLER[1] - FULLER[0])
            half = 0.013 * (1 - 0.35 * share) * min(1.0, share / 0.06 + 0.3, (1 - share) / 0.06 + 0.3)
            panel = _thickness(t)
            height = _height(t)
            rings.append(
                [
                    (-half, face * (panel + 0.0012), height),
                    (half, face * (panel + 0.0012), height),
                    (half * 0.8, face * (panel - 0.002), height),
                    (-half * 0.8, face * (panel - 0.002), height),
                ]
            )

        fuller = core.loft(f"greatsword_fuller_{'f' if face < 0 else 'b'}", rings, core.material("accent"), root, cap_start=True, cap_end=True)

        for polygon in fuller.data.polygons:
            polygon.use_smooth = False

    # The gold collar that closes the head of the blade, above the guard.
    collar = core.loft(
        "greatsword_band",
        [
            [(x * 0.06 * inset, y * 0.024 * inset, GUARD + z) for x, y in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
            for z, inset in ((0.004, 0.86), (0.009, 1.0), (0.036, 1.0), (0.041, 0.86))
        ],
        core.material("gold"),
        root,
        cap_start=True,
        cap_end=True,
    )

    for polygon in collar.data.polygons:
        polygon.use_smooth = False

    finish(collar, wear_boost=1.6)

    # A lid's cross for a guard: a square bar whose ends turn down toward
    # the hand, each end nailed with a pyramid head.
    guard = core.tube(
        "greatsword_guard_bar",
        core.bezier(
            [(-0.25, 0, GUARD - 0.075), (-0.215, 0, GUARD - 0.012), (-0.1, 0, GUARD + 0.002), (0.0, 0, GUARD + 0.002), (0.1, 0, GUARD + 0.002), (0.215, 0, GUARD - 0.012), (0.25, 0, GUARD - 0.075)],
            samples=3,
        ),
        lambda t: 0.021,
        metal,
        sides=4,
        flatten=0.8,
    )
    parts = [guard]

    for side in (-1, 1):
        nail = core.revolve("greatsword_nail", [(0.0, 0.0), (0.034, -0.018), (0.034, -0.03), (0.0, -0.034)], metal, segments=4)
        nail.location = (side * 0.252, 0, GUARD - 0.07)
        parts.append(nail)

    guard = core.join(parts, "greatsword_guard")
    guard.parent = root

    for polygon in guard.data.polygons:
        polygon.use_smooth = False

    finish(guard, wear_boost=1.8)

    # A long grip for two hands, wrapped, and a faceted pommel below it.
    finish(
        core.revolve(
            "greatsword_grip",
            [(0.0, GUARD - 0.02), (0.02, GUARD - 0.025), (0.022, 0.0), (0.02, POMMEL + 0.07), (0.0, POMMEL + 0.065)],
            core.material("wood"),
            root,
            segments=8,
        ),
        wear_boost=1.2,
    )
    binding("greatsword_wrap", root, POMMEL + 0.08, GUARD - 0.04, radius=0.022, turns=10)
    pommel = core.revolve(
        "greatsword_pommel",
        [
            (0.0, POMMEL + 0.076),
            (0.024, POMMEL + 0.07),
            (0.04, POMMEL + 0.054),
            (0.056, POMMEL + 0.026),
            (0.056, POMMEL + 0.006),
            (0.034, POMMEL - 0.02),
            (0.0, POMMEL - 0.04),
        ],
        metal,
        root,
        segments=6,
    )

    for polygon in pommel.data.polygons:
        polygon.use_smooth = False

    finish(pommel, wear_boost=1.8)

    return root


def _steps(start, end, count):
    """`count` evenly spaced values from `start` to `end`."""

    return [start + (end - start) * step / (count - 1) for step in range(count)]


def _height(t):
    """How high the blade is, `t` of the way from the guard to its foot."""

    return GUARD + (BLADE_TIP - GUARD) * t


def _half_width(t):
    """Half the blade's width, `t` of the way up: the coffin, straight
    between its corners."""

    for (t0, w0), (t1, w1) in zip(OUTLINE, OUTLINE[1:]):
        if t <= t1:
            return w0 + (w1 - w0) * (t - t0) / (t1 - t0)

    return OUTLINE[-1][1]


def _thickness(t):
    """Half the blade's thickness at its panel, `t` of the way up."""

    return 0.0165 * (1 - 0.45 * t) + 0.0012


def _blade(name, mat, parent):
    """The coffin blade: at every station a raised panel down the middle and
    a chamfer out to each edge, so each face prints as three planes. A
    station at each of the coffin's corners keeps them sharp, and the last
    station is cut on the slant of the foot."""

    rings = []

    for t in sorted(set(_steps(0.0, CLIP - 0.025, 7) + [share for share, _ in OUTLINE if share < CLIP])):
        width = _half_width(t)
        panel = _thickness(t)
        middle = width * 0.46
        height = _height(t)
        rings.append([(-width, 0, height), (-middle, -panel, height), (middle, -panel, height), (width, 0, height), (middle, panel, height), (-middle, panel, height)])

    # The foot, clipped on a slant from its left corner up to its right.
    width = _half_width(1.0)
    panel = _thickness(1.0)
    middle = width * 0.46

    def slant(x):
        return _height(CLIP + (1.0 - CLIP) * (x + width) / (2 * width))

    rings.append([(x, y, slant(x)) for x, y in ((-width, 0), (-middle, -panel), (middle, -panel), (width, 0), (middle, panel), (-middle, panel))])

    blade = core.loft(name, rings, mat, parent, cap_start=True, cap_end=True)

    for polygon in blade.data.polygons:
        polygon.use_smooth = False

    return blade
