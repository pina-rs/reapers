"""The few parts of a reaper's body that show: hands and what sits in a hood."""

import math

from mathutils import Matrix, Vector

from .core import bezier, join, material, sculpt, shade_smooth, tube


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
                lambda t: 0.0068 + 0.0026 * abs(2 * t - 1) ** 2,
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
                # Thicker at each joint, thinner between, tapering to a claw.
                lambda t: (0.0064 - 0.0034 * t) * (1 + 0.42 * abs(math.sin(t * math.pi * 3)) ** 3),
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
            lambda t: (0.0075 - 0.0038 * t) * (1 + 0.4 * abs(math.sin(t * math.pi * 2)) ** 3),
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
