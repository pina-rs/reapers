"""The reaper's own arms, as it sees them: a big bone hand and the two bones
of a forearm coming out of a ragged sleeve whose cuff is torn into five
tongues.

Each arm is modelled about the middle of its grip, with the forearm running
back toward +Y from the wrist into a sleeve, and the hand closed round a
haft along Z. The back of the right hand faces right (-X); its fingers wrap
the grip's front and end on its left, and its thumb runs forward along the
left side, high, under the pistol's drum (art bible 6.5).

The hand is bone by bone, at about 1.6 times life: three carpals, four
metacarpals out to the knuckles, three phalanges a finger and three bones to
the thumb. Each bone is its own piece, pinched between its joints, so every
knuckle steps out and the ink line breaks at every joint.

The right arm is two draws: its forearm and sleeve hang from `vm_right`,
and its hand from `vm_wrist_r`, a pivot at the wrist, so the game can bend
the hand without swinging the forearm across the view. The weapon goes in
`socket_weapon`, the middle of the grip, through `vm_spin`, a pivot at the
trigger that an active reload spins the weapon about.

The left arm only ever moves as a whole, so it is one draw: `arm_l`, under
`vm_left`. It is cupped around a small round thing at `socket_palm`: the
spare soul drum or the capture orb.

Every pivot sits at rest: the game places `vm_right` and `vm_left` in the
view and turns the rest itself.
"""

import math

import bmesh
from mathutils import Matrix, Vector

from lib import core

from . import common

# How far the right hand's wrist sits behind the haft it holds, along +Y
# (`viewmodel.rs` WRIST, which a test reads back out of the model).
WRIST = 0.13996

# The haft the right hand closes on.
GRIP_RADIUS = 0.019

# The left hand comes up close to the eye in a reload, palm open toward it,
# so it is smaller than the right. It cups the spare drum, `LEFT_GRIP`
# across, and its wrist sits where the right hand's would at its size.
LEFT_HAND_SCALE = 0.84
LEFT_GRIP = 0.028

# Where the right elbow is, from the grip.
ELBOW = (0.012, 0.44, -0.035)

# The pistol's trigger, from the grip: what an active reload spins it about.
TRIGGER = (0.0, -0.02, 0.028)

# Each finger, index first: the height of its knuckle on the grip, its three
# phalanges' lengths, how thick its knuckle is, how far round the grip its
# knuckle stands (degrees from the grip's back toward its right), how far
# that knuckle stands out from the grip, and how much tighter than the
# index it curls. The index rides highest, alone, its middle bone over the
# trigger, with a gap under it; the middle finger's knuckle stands proudest,
# so the knuckle line steps; the ring and little fingers curl tightest. The
# fist hangs a little under the grip's middle so the pistol's drum clears it.
FINGERS = (
    (0.012, (0.058, 0.036, 0.027), 0.0094, 50, 0.021, 0.0),
    (-0.019, (0.064, 0.041, 0.029), 0.0102, 57, 0.026, 0.0),
    (-0.041, (0.059, 0.037, 0.026), 0.0093, 61, 0.023, 0.0035),
    (-0.061, (0.047, 0.029, 0.022), 0.0080, 58, 0.019, 0.006),
)

# How far out from the grip's axis each joint past the knuckle sits, past
# the grip's own radius: the three joints to the tip.
REACH = (0.021, 0.015, 0.009)

# The right thumb's joints, from the root of the hand forward along the
# grip's left side, high and clear of the pistol's drum.
THUMB = ((0.016, 0.104, -0.008), (0.036, 0.064, 0.013), (0.042, 0.025, 0.023), (0.040, -0.014, 0.026))
# The left thumb's: curled under what the hand holds, so it never crosses
# the face the hand turns toward the eye.
LEFT_THUMB = ((0.016, 0.104, -0.010), (0.034, 0.068, -0.022), (0.040, 0.034, -0.032), (0.036, 0.006, -0.037))

# The torn cuff's five tongues, each (where round the cuff from the outside
# of the arm, in radians; how far it reaches past the cuff; how wide, in
# radians either side): unequal, the longest on the outside.
TONGUES = ((0.0, 0.100, 0.62), (1.3, 0.050, 0.46), (2.45, 0.076, 0.52), (3.7, 0.034, 0.42), (5.0, 0.064, 0.50))


def build_right(parent):
    """The right arm under `parent`. Returns the trigger pivot, which the
    weapons go under."""

    arm = core.empty("vm_right", parent)
    wrist = core.empty("vm_wrist_r", arm, location=(0.0, WRIST, 0.0))
    # The hand alone hangs from the wrist, so it is painted on its own.
    hand = _hand(wrist, "arm_r", grip=GRIP_RADIUS, curl=1.0)
    hand.location = (0.0, -WRIST, 0.0)
    common.paint(hand)

    _forearm(arm, "arm_r_forearm", wrist=Vector((0.0, WRIST, 0.0)), elbow=Vector(ELBOW), outside=0.0, seed=7, size=1.0, rings=7, cloth_sides=20)
    common.merge(arm, "arm_r_forearm")

    socket = core.empty("socket_weapon", wrist, location=(0.0, -WRIST, 0.0))

    return core.empty("vm_spin", socket, location=TRIGGER)


def build_left(parent):
    """The left arm under `parent`. Returns its palm socket."""

    arm = core.empty("vm_left", parent)
    scale = LEFT_HAND_SCALE
    # Built about a wider grip, so that once scaled down it still cups the
    # drum, and opened: a cup, not a fist. Its fingers are fatter, so they
    # read as a hand holding something rather than a bundle of planks, and
    # its thumb curls under what it holds instead of across its face.
    hand = _hand(arm, "arm_l_hand", grip=LEFT_GRIP / scale, curl=0.4, fat=1.2, thumb=LEFT_THUMB)
    # Mirrored across into a left hand, and made smaller. Its back faces +X
    # and the mouth of its cup faces back toward the eye, wherever the rig
    # lifts it, so what it holds shows over its fingers.
    hand.data.transform(_diagonal(-scale, scale, scale))
    hand.data.flip_normals()
    wrist = Vector((0.0, WRIST * scale, 0.0))
    elbow = Vector((-ELBOW[0], ELBOW[1], ELBOW[2]))
    # The outside of the left arm, where the cuff's longest tongue hangs, is
    # +X: half a turn round from the right arm's.
    _forearm(arm, "arm_l_forearm", wrist=wrist, elbow=elbow, outside=math.pi, seed=11, size=scale, sides=5, rings=7, cloth_sides=16)
    common.merge(arm, "arm_l")

    return core.empty("socket_palm", arm, location=(-0.004, -0.004, 0.0))


def _diagonal(x, y, z):
    return Matrix.Diagonal((x, y, z, 1.0))


def _around(angle, reach, z):
    """A point `reach` from the grip's axis, `angle` round from its back
    toward its right, at height `z`."""

    return Vector((-math.sin(angle) * reach, math.cos(angle) * reach, z))


def _hand(parent, label, grip, curl, sides=6, fat=1.0, thumb=THUMB):
    """A right hand of bone closed round a haft of radius `grip` along Z at
    the origin, its wrist toward +Y. `curl` opens the fingers as it falls
    from 1, `fat` thickens them, and `thumb` is its thumb's four joints.
    Every bone is its own piece; they are joined, not welded. The big bones
    (metacarpals, thumb) have `sides` sides, the small ones one fewer, and a
    phalanx past the first starts inside the joint before it. Each finger
    tapers, from a thick first phalanx to a claw half as thick."""

    bone_material = core.material("bone")
    small = sides - 1
    parts = []

    def add(name, start, end, head, tail, **shape):
        shape.setdefault("sides", sides)
        parts.append(common.bone(f"{label}_{name}", start, end, head, tail, bone_material, **shape))

    # Three carpals: the knot of the wrist, the heel of the hand and the
    # root of the thumb.
    add("carpal_0", (-0.026, 0.121, -0.018), (-0.008, 0.125, 0.0), 0.0125, 0.0115, sides=small)
    add("carpal_1", (-0.010, 0.118, -0.034), (0.006, 0.122, -0.016), 0.0115, 0.011, sides=small)
    add("carpal_2", (0.004, 0.114, -0.002), (0.016, 0.108, 0.006), 0.0105, 0.0095, sides=small)

    for index, (z, lengths, thick, degrees, stand, tighter) in enumerate(FINGERS):
        thick *= fat
        angle = math.radians(degrees)
        reach = [grip + stand] + [grip + extra - tighter * (step + 1) for step, extra in enumerate(REACH)]
        knuckle = _around(angle, reach[0], z)
        # The metacarpals fan out from the wrist to the knuckles.
        base = Vector((-0.008 - 0.0045 * index, 0.111 + 0.002 * min(index, 3 - index), z * 0.2 - 0.016))
        add(f"metacarpal_{index}", base, knuckle, thick * 0.74, thick * 1.2, hint=_around(angle, 1.0, 0.0))

        joints = [knuckle]
        height = z

        for step, length in enumerate(lengths):
            inner = reach[step] + (1.0 - curl) * 0.03 * step
            outer = reach[step + 1] + (1.0 - curl) * 0.03 * (step + 1)
            cosine = (inner**2 + outer**2 - length**2) / (2 * inner * outer)
            angle += math.acos(max(-1.0, min(1.0, cosine)))
            height -= 0.0016
            joints.append(_around(angle, outer, height))

        sizes = ((thick * 1.02, thick * 0.84), (thick * 0.8, thick * 0.64), (thick * 0.62, thick * 0.46))

        for step, (head, tail) in enumerate(sizes):
            middle = (joints[step] + joints[step + 1]) / 2
            outward = Vector((middle.x, middle.y, 0.0)).normalized()
            add(f"finger_{index}_{step}", joints[step], joints[step + 1], head, tail, claw=step == 2, hint=outward, sides=small, rounded=step == 0)

    # The thumb, heavy and long, from the root of the hand forward.
    thumb = [Vector(joint) for joint in thumb]
    add("thumb_0", thumb[0], thumb[1], 0.0112 * fat, 0.0126 * fat, hint=(1, 0, 0.3))
    add("thumb_1", thumb[1], thumb[2], 0.0118 * fat, 0.0102 * fat, hint=(1, 0, 0.6))
    add("thumb_2", thumb[2], thumb[3], 0.0098 * fat, 0.0080 * fat, claw=True, hint=(1, 0, 0.8))

    result = core.join(parts, label)
    result.parent = parent
    core.shade_smooth(result)

    return result


def _forearm(parent, label, wrist, elbow, outside, seed, size, sides=6, rings=5, cloth_sides=24):
    """The forearm's two bones (`sides` sided) from `wrist` back to `elbow`,
    in a sleeve of `rings` rings, `cloth_sides` round, that ends in a torn
    cuff a hand's width behind the wrist. `outside` is where round the arm
    its outer side is, from the right (-X); `size` scales the cloth with the
    hand."""

    bone_material = core.material("bone")
    axis = (wrist - elbow).normalized()
    across, up = common.frame(axis)

    # The radius on the thumb's side, wide at the wrist; the ulna on the
    # other, its knob at the elbow.
    for name, side, head, tail in (("radius", -1.0, 0.0085, 0.0125), ("ulna", 1.0, 0.0115, 0.008)):
        offset = across * side * 0.011 * size + up * 0.002 * size
        common.bone(
            f"{label}_{name}",
            elbow + offset * 0.6 + axis * 0.02,
            wrist - axis * 0.016 * size + offset,
            head * size,
            tail * size,
            bone_material,
            parent,
            sides=sides,
            waist=0.7,
        )

    cuff_root = wrist - axis * 0.052 * size
    _sleeve(parent, f"{label}_sleeve", elbow - axis * 0.17, cuff_root, size, seed, rings, cloth_sides)
    _cuff(parent, f"{label}_cuff", cuff_root, axis, outside, size, seed, cloth_sides)
    _lining(parent, f"{label}_lining", cuff_root, axis, size)


# How far a fold of the sleeve stands out, and a pinch between two folds
# draws in, against the sleeve's width.
FOLD_DEPTH = 0.24
PINCH_DEPTH = 0.12


def _folds(angle, ring, seed):
    """How far a sleeve's cloth stands out at `angle` round ring `ring`: a
    few big folds that twist down the arm."""

    return 1.0 + 0.11 * math.sin(3 * angle + 0.9 * ring + seed) + 0.045 * math.sin(7 * angle + 1.3 * ring + seed * 0.5)


def _sleeve(parent, name, start, end, size, seed, count, sides):
    """An open tube of cloth from `start` (toward the shoulder, out of view)
    to `end`, narrowing a little toward the wrist, its underside hanging,
    and bunched into big folds: every other ring between the ends stands out
    and the rings between them pinch in. Each fold is set on a slant, as
    cloth pushed up an arm folds, so the bands catch the folds as steps
    rather than stripes."""

    axis = (end - start).normalized()
    across, up = common.frame(axis)
    rings = []

    for ring in range(count):
        share = ring / (count - 1)
        centre = start.lerp(end, share)
        inside = 0 < ring < count - 1
        bunch = (FOLD_DEPTH if ring % 2 else -PINCH_DEPTH) if inside else 0.0
        radius = (0.064 - 0.01 * share) * size * (1.0 + bunch)
        slant = 0.016 * size * (1 if ring % 2 else -0.5) * inside
        points = []

        for side in range(sides):
            angle = math.tau * side / sides
            out = across * math.cos(angle) + up * math.sin(angle)
            point = centre + out * radius * _folds(angle, ring, seed) + axis * slant * math.sin(angle + seed)
            # The cloth hangs: the lower it sits, the further it falls.
            point.z -= 0.012 * size * max(0.0, -math.sin(angle)) ** 1.5
            points.append(point)

        rings.append(points)

    return core.loft(name, rings, core.material("cloth"), parent)


def _cuff(parent, name, root, axis, outside, size, seed, sides):
    """A cuff of cloth flared wide of the sleeve, whose mouth is torn into
    five tongues of unequal length that splay out over the wrist, the
    longest on the outside, given 7 mm of thickness."""

    across, up = common.frame(axis)

    def tongue(angle):
        reach = 0.0

        for at, length, width in TONGUES:
            distance = abs((angle - outside - at + math.pi) % math.tau - math.pi)
            reach = max(reach, length * max(0.0, 1.0 - distance / width) ** 0.85)

        return reach

    mesh = bmesh.new()
    rows = []

    for ring, (forward, radius, torn) in enumerate(((-0.006, 0.0505, 0.0), (0.014, 0.062, 0.45), (0.030, 0.075, 1.0))):
        row = []

        for side in range(sides):
            angle = math.tau * side / sides
            out = across * math.cos(angle) + up * math.sin(angle)
            reach = (forward + tongue(angle) * torn) * size
            # The longer a tongue, the further it splays out from the wrist.
            splay = radius + 0.3 * tongue(angle) * torn
            point = root + axis * reach + out * splay * size * _folds(angle, 4 + ring * 0.5, seed)
            # The tongues are cloth: they sag as they reach.
            point.z -= 0.35 * tongue(angle) * torn * size * (0.6 + 0.4 * max(0.0, math.sin(angle)))
            row.append(mesh.verts.new(point))

        rows.append(row)

    for upper, lower in zip(rows, rows[1:]):
        for side in range(sides):
            follow = (side + 1) % sides
            mesh.faces.new((upper[side], lower[side], lower[follow], upper[follow]))

    cuff = core.object_from_bmesh(name, mesh, core.material("cloth_alt"), parent)
    core.apply_modifier(cuff, "SOLIDIFY", thickness=0.007 * size, offset=1.0, use_even_offset=True)
    core.shade_smooth(cuff)

    return cuff


def _lining(parent, name, root, axis, size):
    """The dark inside of the cuff's mouth: a short tube of void facing
    inward, seen only by looking into the sleeve, so the torn cuff steps
    down to black where it opens instead of printing as more of the same
    cloth."""

    lining = core.tube(name, [root - axis * 0.012 * size, root + axis * 0.012 * size], 0.047 * size, core.material("void"), parent, sides=8, cap=False)
    lining.data.flip_normals()

    return lining
