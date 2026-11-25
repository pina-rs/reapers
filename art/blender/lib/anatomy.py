"""The few parts of a reaper's body that show: hands and what sits in a hood."""

import math

from mathutils import Matrix, Vector

from .core import bezier, join, material, sculpt, shade_smooth, tube

# How far a knuckle swells over the bone either side of it. Bands and hatch
# flatten a uniform rod into a noodle; a knuckle that steps out is what
# makes a bone finger read as one at a phone's size (docs/art-bible.md 6.1).
KNUCKLE = 0.68


def hand(name, parent, grip_radius=0.022, curl=1.0, side=1, scale=1.0):
    """A skeletal hand closed around a haft of `grip_radius`.

    The hand is built at the origin gripping a haft that runs along Z, with
    the wrist toward +Y, then mirrored for the left side. `curl` opens the
    fingers as it falls toward 0.
    """

    bone = material("bone")
    parts = []

    # The back of the hand: four metacarpals fanning from the wrist.
    for finger in range(4):
        offset = (finger - 1.5) * 0.019
        knuckle = Vector((0.0, grip_radius + 0.012, offset))
        wrist = Vector((0.0, grip_radius + 0.085, offset * 0.45))

        parts.append(
            tube(
                f"{name}_metacarpal_{finger}",
                bezier([wrist, (wrist + knuckle) / 2 + Vector((-0.006, 0, 0)), knuckle], samples=5),
                # Narrow in the shaft, the head at the knuckle the largest.
                lambda t: 0.0066 + 0.0022 * abs(2 * t - 1) ** 2 + 0.0034 * t**6,
                bone,
                sides=8,
            )
        )

        # Three phalanges wrapping the haft.
        reach = grip_radius + 0.012
        length = 0.094 - abs(finger - 1.4) * 0.011
        sweep = math.radians(215) * curl
        path = []

        for step in range(13):
            turn = sweep * step / 12
            radius = reach + (1 - curl) * 0.05 * step / 12

            path.append(
                Vector(
                    (
                        -math.sin(turn) * radius * length / 0.094,
                        math.cos(turn) * radius,
                        offset + step * 0.0004,
                    )
                )
            )

        parts.append(
            tube(
                f"{name}_finger_{finger}",
                path,
                # A knuckle at each joint, the bone narrow between, tapering
                # to a claw.
                lambda t: (0.0068 - 0.0036 * t) * (1 + KNUCKLE * abs(math.sin(t * math.pi * 3)) ** 4),
                bone,
                sides=8,
            )
        )

    # The thumb closes from the other side.
    thumb_base = Vector((0.004, grip_radius + 0.07, -0.036))
    thumb_path = bezier(
        [
            thumb_base,
            Vector((0.022, grip_radius + 0.03, -0.05)),
            Vector((0.03, grip_radius * 0.3, -0.046)),
            Vector((0.012, -grip_radius - 0.006, -0.04)),
        ],
        samples=5,
    )
    parts.append(
        tube(
            f"{name}_thumb",
            thumb_path,
            lambda t: (0.0078 - 0.0040 * t) * (1 + KNUCKLE * abs(math.sin(t * math.pi * 2)) ** 4),
            bone,
            sides=8,
        )
    )

    # Carpals: a knot of small bones at the wrist.
    parts.append(
        sculpt(
            f"{name}_wrist",
            [
                ("sphere", (0.0, grip_radius + 0.092, 0.0), (0.017, 0.014, 0.03)),
                ("sphere", (0.0, grip_radius + 0.112, 0.0), (0.013, 0.02, 0.019)),
            ],
            bone,
            voxel=0.004,
        )
    )

    result = join(parts, name)
    result.parent = parent

    if side < 0 or scale != 1.0:
        result.data.transform(Matrix.Diagonal((scale, scale, scale * side, 1)))

        if side < 0:
            result.data.flip_normals()

    return shade_smooth(result)


def grow(hand, factor):
    """Makes a hand `factor` times larger about the haft it holds, which runs
    along its own Z through its origin, keeping the inside of its grip where
    it was: the fingers grow longer and thicker and the hand wider, but
    still close on the same haft, and the weapon it holds does not move.

    Seen from the haft, a point at distance r from it goes to
    r0 + factor x (r - r0), r0 being the nearest the hand comes to it, at the
    same angle; along the haft it goes to factor x z. The wrist end reaches
    further back into the sleeve."""

    vertices = hand.data.vertices
    inside = min(math.hypot(vertex.co.x, vertex.co.y) for vertex in vertices)

    for vertex in vertices:
        x, y, z = vertex.co
        reach = math.hypot(x, y)

        if reach > 1e-9:
            # Never past the haft's axis, however far `factor` is from 1.
            outward = max(0.0, inside + factor * (reach - inside)) / reach
            x, y = x * outward, y * outward

        vertex.co = (x, y, z * factor)

    hand.data.update()

    return hand


def forearm(name, parent, wrist, elbow):
    """The two bones of a forearm, for where a sleeve has burned away."""

    bone = material("bone")
    wrist = Vector(wrist)
    elbow = Vector(elbow)
    across = (elbow - wrist).cross(Vector((0, 0, 1))).normalized() * 0.011
    parts = []

    for sign in (-1, 1):
        parts.append(
            tube(
                f"{name}_{sign}",
                [wrist + across * sign, (wrist + elbow) / 2 + across * sign * 1.5, elbow + across * sign * 0.6],
                lambda t: 0.0085 + 0.005 * abs(2 * t - 1) ** 2.5,
                bone,
                sides=8,
            )
        )

    result = join(parts, name)
    result.parent = parent

    return shade_smooth(result)
