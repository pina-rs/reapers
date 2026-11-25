"""The greatsword: a heavy blade no class starts with.

Its one idea is the coffin (art bible 6.4 asks each Bought weapon for one):
the blade is a coffin's lid in outline, narrow at the ricasso, flaring to
broad shoulders a quarter of the way up, then running long and straight to a
clipped head instead of a point. It is ground in clean planes, a flat down
its middle and a bevel to each edge, and a seam of light runs down the flat
of each face: the fuller, the one light on it. With its heavy crossguard it
is a long cross at any distance. Its grip is wrapped in rope, and a collar
of the owner's gold closes the ricasso.
"""

from lib import core
from lib.armoury import binding, finish

GUARD = 0.2
BLADE_TIP = 1.42
POMMEL = -0.27

# The coffin, as (share of the way up the blade, half its width): the
# ricasso, the shoulders, the long run to the head's corners, the head.
OUTLINE = ((0.0, 0.028), (0.05, 0.028), (0.22, 0.084), (0.92, 0.047), (1.0, 0.025))


def build():
    """A coffin-shaped blade on a two-handed grip."""

    root = core.empty("weapon_greatsword")
    metal = core.material("metal")

    finish(_blade("greatsword_blade", metal, root), wear_boost=2.0)

    # The fuller: a seam of light down the flat of each face.
    for face in (-1, 1):
        core.tube(
            f"greatsword_fuller_{'f' if face < 0 else 'b'}",
            [(0, face * (_thickness(t) + 0.0004), _height(t)) for t in _steps(0.09, 0.84, 10)],
            lambda t: 0.0062 * (1 - 0.45 * t),
            core.material("accent"),
            root,
            sides=5,
            flatten=0.35,
        )

    # The gold collar that closes the ricasso, just above the guard.
    collar = core.loft(
        "greatsword_band",
        [
            [(x * 0.036, y * 0.0125, GUARD + z) for x, y in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
            for z in (0.006, 0.010, 0.036, 0.040)
        ],
        core.material("gold"),
        root,
        cap_start=True,
        cap_end=True,
    )
    _chamfer_ends(collar)
    finish(collar, wear_boost=1.6)

    # A heavy crossguard, straight and square, its ends dropped a little.
    guard = core.tube(
        "greatsword_guard",
        core.bezier([(-0.24, 0, GUARD - 0.004), (-0.12, 0, GUARD + 0.002), (0.0, 0, GUARD + 0.002), (0.12, 0, GUARD + 0.002), (0.24, 0, GUARD - 0.004)], samples=4),
        lambda t: 0.019 + 0.007 * abs(2 * t - 1) ** 4,
        metal,
        root,
        sides=4,
        flatten=0.7,
    )

    for polygon in guard.data.polygons:
        polygon.use_smooth = False

    finish(guard, wear_boost=1.8)

    # A long grip for two hands, wrapped, and a faceted pommel below it.
    finish(
        core.revolve(
            "greatsword_grip",
            [(0.0, GUARD - 0.02), (0.019, GUARD - 0.025), (0.021, 0.0), (0.019, POMMEL + 0.07), (0.0, POMMEL + 0.065)],
            core.material("wood"),
            root,
            segments=10,
        ),
        wear_boost=1.2,
    )
    binding("greatsword_wrap", root, POMMEL + 0.08, GUARD - 0.04, radius=0.021, turns=11)
    pommel = core.revolve(
        "greatsword_pommel",
        [
            (0.0, POMMEL + 0.074),
            (0.022, POMMEL + 0.068),
            (0.036, POMMEL + 0.054),
            (0.05, POMMEL + 0.03),
            (0.05, POMMEL + 0.01),
            (0.032, POMMEL - 0.014),
            (0.0, POMMEL - 0.03),
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
    """How high the blade is, `t` of the way from the guard to its head."""

    return GUARD + (BLADE_TIP - GUARD) * t


def _half_width(t):
    """Half the blade's width, `t` of the way up: the coffin, straight
    between its corners."""

    for (t0, w0), (t1, w1) in zip(OUTLINE, OUTLINE[1:]):
        if t <= t1:
            return w0 + (w1 - w0) * (t - t0) / (t1 - t0)

    return OUTLINE[-1][1]


def _thickness(t):
    """Half the blade's thickness at its flat, `t` of the way up."""

    return 0.0095 * (1 - 0.5 * t) + 0.0012


def _blade(name, mat, parent):
    """The coffin blade: at every station a flat down the middle and a bevel
    out to each edge, so each face prints as three planes. A station at each
    of the coffin's corners keeps them sharp."""

    rings = []

    for t in sorted(set(_steps(0.0, 1.0, 9) + [share for share, _ in OUTLINE])):
        width = _half_width(t)
        flat = _thickness(t)
        middle = width * 0.42
        height = _height(t)
        rings.append([(-width, 0, height), (-middle, -flat, height), (middle, -flat, height), (width, 0, height), (middle, flat, height), (-middle, flat, height)])

    blade = core.loft(name, rings, mat, parent, cap_start=True, cap_end=True)

    for polygon in blade.data.polygons:
        polygon.use_smooth = False

    return blade


def _chamfer_ends(obj):
    """Pulls in the first and last rings of a four-sided loft so its ends are
    chamfered."""

    rows = sorted({round(vertex.co.z, 6) for vertex in obj.data.vertices})

    for vertex in obj.data.vertices:
        if round(vertex.co.z, 6) in (rows[0], rows[-1]):
            vertex.co.x *= 0.88
            vertex.co.y *= 0.8

    return obj

