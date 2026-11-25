"""The axe: a heavy head no class starts with.

Its one idea is the death's-head moth (art bible 6.4 asks each Bought weapon
for one): a labrys whose two bits are a moth's wings, spread from a body
forged round the haft. Each bit is a pair, a forewing swept up to a pointed
apex over a smaller, rounder hindwing, with a notch between them, so the
head is a moth at rest at any distance. The bit across the body (+X) is the
main one: its whole margin is the cutting edge, with a seam of light along
it, the one light on the axe. The back bit is the same pair, smaller. The
eye is the moth's head, its broad thorax between the wings and its ringed
abdomen tapering down the haft. Head iron, haft wood, a band of the owner's
gold at the grip.
"""

from lib import core
from lib.armoury import band, binding, finish, haft

TOP = 0.70
BUTT = -0.52

# Each bit is a pair of wings, as a moth's are seen from above: a forewing
# swept up to a pointed apex and a smaller, rounder hindwing below it, with a
# notch between. Each is given as its root along the eye, top to bottom, and
# its margin from the apex round to the bottom, (x, z). The main bit, across
# the body (+X), is the cutting one.
MAIN_ROOT = [(0.036, z) for z in (0.650, 0.612, 0.574, 0.540, 0.505, 0.470, 0.430, 0.392, 0.360)]
MAIN_MARGIN = [(0.300, 0.805), (0.334, 0.715), (0.318, 0.625), (0.262, 0.548), (0.170, 0.512), (0.214, 0.470), (0.226, 0.418), (0.186, 0.370), (0.100, 0.340)]
# The back bit: the same pair of wings, smaller.
BACK_SCALE = 0.68

def build():
    """A moth-winged labrys on a short haft."""

    root = core.empty("weapon_axe")
    metal = core.material("metal")

    haft("axe_haft", root, bottom=BUTT, top=TOP, radius=0.022, bow=0.018)
    binding("axe_grip", root, -0.12, 0.13, radius=0.027)
    band("axe_band", root, -0.158, -0.124, radius=0.0275)
    binding("axe_lashing", root, 0.22, 0.275, radius=0.027, turns=4)

    margin = core.bezier(MAIN_MARGIN, samples=2)
    finish(_wing("axe_head", core.bezier(MAIN_ROOT, samples=2), margin, 0.036, metal, root), wear_boost=2.0)

    # The cutting edge, the main bit's whole margin, is the one light.
    core.tube(
        "axe_edge",
        [(point.x + 0.002, 0, point.y) for point in margin[1:-1]],
        lambda t: 0.0048 + 0.002 * (1 - abs(2 * t - 1)),
        core.material("accent"),
        root,
        sides=6,
    )

    # The back bit: the main one's wings mirrored behind the haft and made
    # smaller about the thorax.
    middle = 0.53

    def back(points):
        return [(-x * BACK_SCALE, middle + (z - middle) * (0.4 + 0.6 * BACK_SCALE)) for x, z in points]

    finish(_wing("axe_hindwing", core.bezier(back(MAIN_ROOT), samples=2), core.bezier(back(MAIN_MARGIN), samples=2), 0.03, metal, root), wear_boost=1.8)

    # The moth's body round the haft: a head, a broad thorax between the
    # wings and a ringed abdomen tapering down the haft.
    body = core.revolve(
        "axe_eye",
        [
            (0.0, TOP + 0.034),
            (0.024, TOP + 0.028),
            (0.034, TOP + 0.006),
            (0.028, 0.676),
            (0.054, 0.652),
            (0.060, 0.590),
            (0.054, 0.520),
            (0.058, 0.452),
            (0.046, 0.396),
            (0.038, 0.386),
            (0.046, 0.372),
            (0.036, 0.352),
            (0.042, 0.340),
            (0.032, 0.318),
            (0.036, 0.306),
            (0.026, 0.286),
            (0.0, 0.280),
        ],
        metal,
        root,
        segments=8,
    )

    for polygon in body.data.polygons:
        polygon.use_smooth = False

    finish(body, wear_boost=1.8)

    # Faceted cap on the butt of the haft.
    cap = core.revolve(
        "axe_butt",
        [(0.0, 0.034), (0.028, 0.030), (0.033, 0.008), (0.028, -0.018), (0.016, -0.038), (0.0, -0.056)],
        metal,
        root,
        segments=8,
    )
    cap.location = (0.005, 0, BUTT)

    for polygon in cap.data.polygons:
        polygon.use_smooth = False

    finish(cap, wear_boost=1.6)

    return root


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

