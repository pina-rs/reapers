"""The reaper's own arms, as it sees them: a skeletal hand and forearm
coming out of a ragged sleeve.

Each arm is modelled about the middle of its grip, with the forearm running
back toward +Y from the wrist into a sleeve, and the hand closed toward -Y.

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

from mathutils import Vector

from lib import anatomy, cloth, core

from . import common

HAND_SCALE = 1.08
GRIP_RADIUS = 0.019

# How far the right hand's wrist sits behind the haft it holds, along +Y.
WRIST = GRIP_RADIUS + 0.112 * HAND_SCALE

# Where the right elbow is, from the grip.
ELBOW = (0.012, 0.44, -0.035)

# The pistol's trigger, from the grip: what an active reload spins it about.
TRIGGER = (0.0, -0.02, 0.028)


def build_right(parent):
    """The right arm under `parent`. Returns the trigger pivot, which the
    weapons go under."""

    arm = core.empty("vm_right", parent)
    wrist = core.empty("vm_wrist_r", arm, location=(0.0, WRIST, 0.0))
    # The hand alone hangs from the wrist, so it is painted on its own.
    common.paint(_hand(wrist, "arm_r", curl=1.0, side=1, grip_radius=GRIP_RADIUS, keep=0.3, origin=(0.0, -WRIST, 0.0)))
    _forearm(arm, "arm_r_forearm", side=1, grip_radius=GRIP_RADIUS)
    common.merge(arm, "arm_r_forearm")

    socket = core.empty("socket_weapon", wrist, location=(0.0, -WRIST, 0.0))

    return core.empty("vm_spin", socket, location=TRIGGER)


def build_left(parent):
    """The left arm under `parent`. Returns its palm socket."""

    arm = core.empty("vm_left", parent)
    _hand(arm, "arm_l_hand", curl=0.52, side=-1, grip_radius=0.028, keep=0.24, origin=(0.0, 0.0, 0.0))
    _forearm(arm, "arm_l_forearm", side=-1, grip_radius=0.028)
    common.merge(arm, "arm_l")

    return core.empty("socket_palm", arm, location=(-0.004, -0.004, 0.0))


def _hand(parent, label, curl, side, grip_radius, keep, origin):
    """A skeletal hand closed around a grip at `origin` in `parent`'s frame.
    Seen from a hand's width away, the knot of wrist bones needs far fewer
    faces than voxel remeshing gives it, so it is decimated to `keep`."""

    hand = anatomy.hand(label, parent, grip_radius=grip_radius, curl=curl, side=side, scale=HAND_SCALE)
    hand.location = origin
    core.decimate(hand, keep)

    return hand


def _forearm(parent, label, side, grip_radius):
    wrist = Vector((0.0, grip_radius + 0.112 * HAND_SCALE, 0.0))
    elbow = Vector((ELBOW[0] * side, ELBOW[1], ELBOW[2]))
    anatomy.forearm(f"{label}_bones", parent, wrist, elbow)

    # The sleeve the forearm comes out of, its torn mouth halfway up.
    cloth.sleeve(
        f"{label}_sleeve",
        core.material("cloth"),
        parent,
        [elbow + Vector((0.0, 0.22, -0.03)), elbow + Vector((0.0, 0.08, -0.008)), wrist + Vector((0.0, 0.16, -0.012))],
        shoulder=0.052,
        cuff=0.07,
        droop=0.05,
        tatter=0.55,
        seed=7 if side > 0 else 11,
        rings=8,
        segments=14,
    )
