"""What every reaper shares: its landmarks, its hidden face and its arms.

A class module builds its own robe, heads and mantles, and calls into here
for everything the six classes have in common, so they stay one family.
"""

import math

import bpy
from mathutils import Vector

from . import anatomy, cloth, core
from .core import finish

# The classes share one skeleton of landmarks, in metres. A class that
# hunches carries all of them forward by the same amount.
SHOULDER_HEIGHT = 1.64
NECK_HEIGHT = 1.73
HEAD_HEIGHT = 1.88
HEAD_RADIUS = 0.105


def eyes(parent, forward=0.0, spacing=0.04, height=HEAD_HEIGHT + 0.012, size=0.012, name="eyes"):
    """Two points of light. A reaper's face is never more than this."""

    accent = core.material("accent")
    parts = []

    for side in (-1, 1):
        parts.append(
            core.sculpt(
                f"{name}_{side}",
                [("sphere", (side * spacing, -forward - 0.118, height), (size * 1.25, size * 0.6, size * 0.62), (0, side * 0.35, 0))],
                accent,
                voxel=0.003,
                smooth=0,
            )
        )

    result = core.join(parts, name)

    # The game scales the eyes by the genome's gaze, so they must grow about
    # their own middle rather than slide away from the head.
    core.activate(result)
    bpy.ops.object.origin_set(type="ORIGIN_GEOMETRY", center="BOUNDS")
    result.parent = parent

    return result


def void_head(parent, forward=0.0, name="void"):
    """The darkness inside a hood or under a brim."""

    return core.sculpt(
        name,
        [("sphere", (0, -forward - 0.012, HEAD_HEIGHT), (HEAD_RADIUS, HEAD_RADIUS * 1.05, HEAD_RADIUS * 1.2))],
        core.material("void"),
        parent,
        voxel=0.012,
    )


def face_wrap(parent, forward=0.0, name="wrap", seed=2, bands=5, top=HEAD_HEIGHT - 0.045):
    """Cloth wound over the neck and lower face, hiding everything below the
    eyes."""

    mat = core.material("cloth_alt")
    bottom = NECK_HEIGHT - 0.07
    parts = []

    for band in range(bands):
        share = band / (bands - 1)
        height = bottom + (top - bottom) * share
        tilt = (band % 2 * 2 - 1) * 0.022
        radius = 0.15 - 0.036 * share
        path = []

        for step in range(41):
            angle = math.tau * step / 40
            path.append(
                (
                    math.cos(angle) * radius,
                    math.sin(angle) * radius * 0.98 - forward * (0.55 + 0.45 * share) - 0.01,
                    height + math.sin(angle + band + seed) * tilt + 0.014 * math.cos(angle * 2 + band),
                )
            )

        parts.append(core.tube(f"{name}_{band}", path, 0.034 - 0.006 * share, mat, sides=10, cap=False, flatten=0.42))

    result = core.join(parts, name)
    result.parent = parent

    return finish(core.shade_smooth(result))


def arms(
    root,
    weapon_side_path,
    off_side_path,
    sleeve_shoulder=0.075,
    sleeve_cuff=0.12,
    droop=0.24,
    off_curl=0.25,
    hand_scale=1.45,
    tatter=0.4,
    seed=1,
):
    """Both arms, each under its own shoulder pivot so the game can move it.

    The weapon-side hand is closed around `socket_weapon`; the other hangs
    open. Hands are larger than life, which is most of what makes a robed
    figure read as dangerous.
    """

    mat = core.material("cloth")
    pivots = []

    for label, path, side in (("arm_r", weapon_side_path, -1), ("arm_l", off_side_path, 1)):
        shoulder = Vector(path[0])
        pivot = core.empty(label, root, location=shoulder)
        local = [Vector(point) - shoulder for point in core.bezier(path, samples=8)]
        wrist = local[-1]
        direction = (local[-1] - local[-4]).normalized()
        grip = wrist + direction * 0.1 * hand_scale
        middle = len(local) // 2
        elbow_at = local[middle]
        elbow = core.empty(label.replace("arm", "elbow"), pivot, location=elbow_at)
        hand_pivot = core.empty(label.replace("arm", "wrist"), elbow, location=grip - elbow_at)

        sleeve = cloth.sleeve(
            f"{label}_upper_sleeve",
            mat,
            pivot,
            local[: middle + 2],
            sleeve_shoulder,
            sleeve_shoulder * 1.12,
            droop=0.015,
            tatter=0,
            seed=seed + side,
        )
        finish(sleeve)
        finish(
            cloth.sleeve(
                f"{label}_lower_sleeve",
                mat,
                elbow,
                [point - elbow_at for point in local[middle - 1 :]],
                sleeve_shoulder * 1.08,
                sleeve_cuff,
                droop=droop,
                tatter=tatter,
                seed=seed + side,
            )
        )

        if label == "arm_r":
            socket = core.empty("socket_weapon", hand_pivot)
            hand = anatomy.hand(f"{label}_hand", socket, curl=1.0, side=1, scale=hand_scale)
        else:
            hand = anatomy.hand(f"{label}_hand", hand_pivot, curl=off_curl, side=-1, scale=hand_scale)
            # Wrist up, fingers hanging.
            hand.rotation_euler = (math.radians(100), 0, math.radians(-18))

        finish(hand, wear_boost=1.6)
        pivots.append(pivot)

    return pivots
