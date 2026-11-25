"""The axe: a heavy head no class starts with.

Its one idea is the death's-head moth (art bible 6.4 asks each Bought weapon
for one): a labrys whose two bits are a moth's wings, spread from a body
forged round the haft, with the moth's skull worn on its back. The bit
across the body (+X) is the main one: a forewing swept up to a pointed apex,
whose outer margin is the cutting edge and runs down to a hooked beard, over
a smaller, rounder hindwing, with a notch between them. Only that cutting
edge carries the seam of light, the one light on the axe. The back bit
(`axe_spike`) is the same pair of wings, smaller. The eye is the moth's
head, thorax and abdomen in three masses, and on the thorax, front and back,
is a plate of bone in the shape of a skull (`axe_skull`), its sockets void:
the death's-head. The haft is long enough to lift the head clear of its
bearer's hood. Head iron, haft wood, a band of the owner's gold at the grip.
"""

import math

from mathutils import Matrix

from lib import core
from lib.armoury import band, binding, finish, haft

# How far the head stands above where a short axe's would, so it clears
# the hood of every bearer, and the top of the haft under it.
LIFT = 0.45
TOP = 0.7 + LIFT
BUTT = -0.58

# The main bit, as a moth's wings are seen from above: its root along the
# eye, top to bottom, and its margin from the forewing's apex down its
# cutting edge to the beard, in at the notch, round the hindwing and back
# to the eye, (x, z). Both have the same number of points: each root point
# is skinned to the margin point beside it.
MAIN_ROOT = [(0.04, z + LIFT) for z in (0.66, 0.645, 0.625, 0.60, 0.57, 0.54, 0.525, 0.49, 0.44, 0.39, 0.355)]
MAIN_MARGIN = [
    (0.27, 0.87),
    (0.355, 0.78),
    (0.395, 0.66),
    (0.38, 0.55),
    (0.335, 0.47),
    (0.29, 0.395),
    (0.17, 0.46),
    (0.205, 0.40),
    (0.19, 0.345),
    (0.13, 0.315),
    (0.07, 0.33),
]
MAIN_MARGIN = [(x, z + LIFT) for x, z in MAIN_MARGIN]
# Where the cutting edge ends: the beard's tip, in MAIN_MARGIN.
BEARD = 5
# The back bit: the same pair of wings, smaller.
BACK_SCALE = 0.68
SAMPLES = 4


def build():
    """A moth-winged labrys on a long haft."""

    root = core.empty("weapon_axe")
    metal = core.material("metal")

    haft("axe_haft", root, bottom=BUTT, top=TOP, radius=0.023, bow=0.018)
    binding("axe_grip", root, -0.12, 0.13, radius=0.028)
    band("axe_band", root, -0.17, -0.125, radius=0.037)
    binding("axe_lashing", root, 0.22 + LIFT, 0.28 + LIFT, radius=0.028, turns=4)

    margin = core.bezier(MAIN_MARGIN, samples=SAMPLES)
    finish(_wing("axe_head", core.bezier(MAIN_ROOT, samples=SAMPLES), margin, 0.04, metal, root), wear_boost=2.0)

    # The cutting edge, from the forewing's apex down to the beard, is the
    # one light.
    edge = margin[1 : BEARD * SAMPLES]
    core.tube(
        "axe_edge",
        [(point.x + 0.003, 0, point.y) for point in edge],
        lambda t: 0.0075 * (1 - abs(2 * t - 1) ** 3) + 0.003,
        core.material("accent"),
        root,
        sides=6,
        flatten=0.6,
    )

    # The back bit: the main one's wings mirrored behind the haft and made
    # smaller about the thorax.
    middle = 0.53 + LIFT

    def back(points):
        return [(-x * BACK_SCALE, middle + (z - middle) * (0.4 + 0.6 * BACK_SCALE)) for x, z in points]

    finish(_wing("axe_spike", core.bezier(back(MAIN_ROOT), samples=SAMPLES), core.bezier(back(MAIN_MARGIN), samples=SAMPLES), 0.034, metal, root), wear_boost=1.8)

    # The moth's body round the haft, in three masses: a small head, a broad
    # thorax between the wings, and an abdomen tapering down the haft. Cut
    # in eight planes, a flat toward the front for the skull to lie on.
    body = core.revolve(
        "axe_eye",
        [
            (0.0, TOP + 0.04),
            (0.026, TOP + 0.03),
            (0.032, TOP + 0.008),
            (0.026, TOP - 0.012),
            (0.058, TOP - 0.04),
            (0.064, 0.60 + LIFT),
            (0.06, 0.50 + LIFT),
            (0.044, 0.46 + LIFT),
            (0.05, 0.43 + LIFT),
            (0.042, 0.37 + LIFT),
            (0.026, 0.32 + LIFT),
            (0.0, 0.29 + LIFT),
        ],
        metal,
        segments=8,
    )
    body.data.transform(Matrix.Rotation(math.pi / 8, 4, "Z"))
    body.parent = root

    for polygon in body.data.polygons:
        polygon.use_smooth = False

    finish(body, wear_boost=1.8)
    _skull(root)

    # Faceted cap on the butt of the haft.
    cap = core.revolve(
        "axe_butt",
        [(0.0, 0.034), (0.03, 0.030), (0.035, 0.008), (0.03, -0.018), (0.016, -0.04), (0.0, -0.06)],
        metal,
        root,
        segments=8,
    )
    cap.location = (0.005, 0, BUTT)

    for polygon in cap.data.polygons:
        polygon.use_smooth = False

    finish(cap, wear_boost=1.6)

    return root


def _skull(root):
    """The death's-head: a plate of bone shaped as a skull on the flat of
    the thorax, front and back, with two sockets of void in each."""

    # A cranium, the cheekbones' step and a narrow jaw, (x, z) about the
    # middle of the thorax.
    outline = [(-0.034, 0.012), (-0.028, 0.036), (-0.013, 0.048), (0.013, 0.048), (0.028, 0.036), (0.034, 0.012), (0.031, -0.012), (0.02, -0.022), (0.016, -0.04), (-0.016, -0.04), (-0.02, -0.022), (-0.031, -0.012)]
    centre = 0.585 + LIFT
    flat = 0.064 * math.cos(math.pi / 8) * 0.985
    plates = []
    sockets = []

    for face in (-1, 1):
        plate = core.plate(f"axe_skull_{face}", [(x, centre + z) for x, z in outline], 0.008, core.material("bone"), bevel=0.0)
        plate.location = (0.0, face * (flat + 0.003), 0.0)
        plates.append(plate)

        for across in (-1, 1):
            eye = core.revolve(f"axe_socket_{face}_{across}", [(0.0, 0.0), (0.0105, 0.0), (0.0105, 0.004), (0.0, 0.004)], core.material("void"), segments=6)
            eye.data.transform(Matrix.Translation((across * 0.0135, face * (flat + 0.0068), centre + 0.012)) @ Matrix.Rotation(-face * math.pi / 2, 4, "X"))
            sockets.append(eye)

    skull = core.join(plates, "axe_skull")
    skull.parent = root

    for polygon in skull.data.polygons:
        polygon.use_smooth = False

    finish(skull, wear_boost=1.6)
    core.join(sockets, "axe_skull_eyes").parent = root


def _wing(name, root, margin, thickness, mat, parent):
    """A wing of iron from `root` (along the eye) out to `margin`, both
    running top to bottom: thick at the root and ground in two planes on
    each face to a keen margin. Flat-shaded, so each plane prints whole."""

    rings = []

    for (rx, rz), (mx, mz) in zip(root, margin):
        half = thickness / 2

        def at(share):
            return rx + (mx - rx) * share, rz + (mz - rz) * share

        (ax, az), (bx, bz) = at(0.35), at(0.8)
        rings.append([(rx, -half, rz), (ax, -half * 0.8, az), (bx, -half * 0.3, bz), (mx, 0, mz), (bx, half * 0.3, bz), (ax, half * 0.8, az), (rx, half, rz)])

    wing = core.loft(name, rings, mat, parent, closed=True, cap_start=True, cap_end=True)

    for polygon in wing.data.polygons:
        polygon.use_smooth = False

    return wing
