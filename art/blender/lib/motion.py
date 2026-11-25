"""Shared spirit and humanoid performances, baked to named glTF clips.

Pose angles are Euler degrees, local XYZ in Blender's Z-up frame, and key
poses are interpolated as turns, not axis by axis (`curves`). Each clip keys
every joint on every frame, so switching never inherits a stale pose. The
rig is articulated rigid cloth, matching the existing node-based renderer.

What makes it move like one body rather than a set of rods:

- **Hold, do not snap.** A key pose that leaves a joint out holds it where
  it was; only a stance (`REST`, `GUARD`) sends a joint home (`curves`).
- **Manners.** A key says how it is passed: crossed at speed (a contact),
  stopped on (an apex, the wind-up a body anticipates from) or carried past
  and settled back onto (a landing, a body's turn running out) (`curves`).
  A follow-through loops round into its recovery rather than reversing, so
  the weapon keeps moving (`combat`).
- **Overlap.** Each link moves a little ahead of the next (`LEAD`): the
  pelvis first, then the chest, the head, the shoulder and the elbow, and
  the wrist, with the weapon in it, last. The keys are the weapon hand's
  times, so a contact key lands on the clip's impact.
- **Secondary motion, baked.** At bake time a damped spring, driven by the
  body's own motion, swings the robe's `sway_x`, `sway_y` and `billow`
  targets and the Hollow's carried lantern (`build`).
- **Sixty keys a second** (`RATE`), so a swing that turns a joint 30
  degrees in a thirtieth of a second draws an arc, not a polyline.
"""

import math

import bpy
from bpy_extras import anim_utils
from mathutils import Matrix, Quaternion, Vector

from . import body, combat, core, curves, locomotion
from .combat import trunk
from .curves import AUTO, PASS, REST, STOP, over, pose, stab, stance

JOINTS = ("motion", "spine", "head", "arm_r", "arm_l", "elbow_r", "elbow_l", "wrist_r", "wrist_l") + body.LEG_JOINTS

# Keys a second every clip is baked at: the scene rate while baking.
RATE = 60

# Subtle timing/size accents preserve silhouettes without losing shared clips.
AMPLITUDE = {"harvester": 1.0, "ferryman": 0.85, "weigher": 0.8, "chooser": 1.05, "hollow": 1.12, "mourner": 0.75}

# How far ahead of the weapon hand each link moves, in seconds. In a
# one-shot the lead swells in from the first frame and out by the last, so
# both stay exactly on their keys; in a loop each link simply runs behind
# the pelvis by the difference.
LEAD = {"pelvis": 0.10, "spine": 0.075, "head": 0.05, "arm": 0.033, "elbow": 0.017, "wrist": 0.0}
# The most a lead may be of a one-shot, so its time never runs backwards.
MOST_LEAD = 0.25
# The links that meet a contact on the impact rather than ahead of it: the
# hips, the chest and the arm start the blow ahead of the hand, each a
# little later than the one before (`LEAD`), but drive through it with the
# hand, so the blade crosses its contact at the top of its speed. Only the
# head, which turns to watch, still leads it there.
ON_THE_BLOW = ("pelvis", "spine", "arm", "elbow")
LINKS = {"head": "head", "arm_r": "arm", "arm_l": "arm", "elbow_r": "elbow", "elbow_l": "elbow", "wrist_r": "wrist", "wrist_l": "wrist"}

LOOPS = ("idle", "creep", "walk", "run", "sprint")

# Each clip's length in seconds, by name, from the specs `variants` read.
TIMINGS = {}

GUARD = stance(arm_r=(-28, 12, -12), arm_l=(-48, -18, 18), elbow_r=(-28, 0, 0), elbow_l=(-42, 0, 0), head=(10, 0, 0))



# Each move has anticipation, a readable apex and a recovery that settles.
# Positions are offsets, so authored class proportions and hunched necks
# remain intact.
KEYS = {
    "jump": [
        (0, REST),
        # Crouch and draw the arms back...
        (
            0.18,
            pose(move=(0, 0, -0.05), motion=(10, 0, 0), head=(6, 0, 0), arm_r=(14, 6, 0), arm_l=(22, -6, 0), elbow_r=(-30, 0, 0), elbow_l=(-30, 0, 0)),
            STOP,
        ),
        # ...and throw them up to leave the floor.
        (
            0.30,
            pose(move=(0, 0, 0.24), motion=(-6, 0, 0), head=(-6, 0, 0), arm_r=(-30, 8, 0), arm_l=(-55, -14, 0), elbow_r=(-20, 0, 0), elbow_l=(-24, 0, 0)),
            PASS,
        ),
        (
            0.48,
            pose(move=(0, 0, 0.65), motion=(-10, 0, 0), head=(-10, 0, 0), arm_r=(-38, 10, 0), arm_l=(-65, -20, 0), elbow_r=(-28, 0, 0), elbow_l=(-30, 0, 0)),
            STOP,
        ),
        (0.72, pose(move=(0, 0, 0.14), motion=(5, 0, 0), head=(3, 0, 0), arm_r=(-12, 12, 0), arm_l=(-22, -14, 0), elbow_r=(-34, 0, 0), elbow_l=(-38, 0, 0))),
        # Land and give: the knees and the arms take it, and spring back.
        (
            0.82,
            pose(move=(0, 0, -0.04), motion=(12, 0, 0), head=(8, 0, 0), arm_r=(6, 10, 0), arm_l=(10, -10, 0), elbow_r=(-30, 0, 0), elbow_l=(-32, 0, 0)),
            trunk(stab(0.07, 0.14)),
        ),
        (1, REST, STOP),
    ],
    "land": [
        # It starts falling: already moving down into the floor.
        (0, stance(move=(0, 0, 0.2), motion=(-4, 0, 0), arm_r=(-14, 10, 0), arm_l=(-20, -12, 0)), PASS),
        (
            0.26,
            pose(move=(0, 0, -0.05), motion=(12, 0, 0), head=(10, 0, 0), arm_r=(-10, 14, 0), arm_l=(-25, -22, 0), elbow_r=(-30, 0, 0), elbow_l=(-30, 0, 0)),
            # The hips give on past the floor at the speed they fell, not
            # driven faster, and the chest bows through the key without a
            # stop: either would turn the weapon's tip a corner there.
            {"*": AUTO, "move": over(0.08, 0.07, 0.16)},
        ),
        (0.62, pose(move=(0, 0, 0.04), motion=(-4, 0, 0), head=(-3, 0, 0), arm_r=(0, 6, 0), arm_l=(-6, -8, 0), elbow_r=(-16, 0, 0), elbow_l=(-18, 0, 0))),
        (1, REST, STOP),
    ],
    "dash": [
        (0, REST),
        # Rock back to load the burst.
        (0.2, pose(move=(0, 0.1, -0.02), motion=(-7, 0, 0), head=(-4, 0, 0), arm_r=(-14, 10, 0), arm_l=(-20, -10, 0), elbow_r=(-30, 0, 0), elbow_l=(-30, 0, 0)), STOP),
        (0.48, pose(move=(0, -0.8, 0.12), motion=(32, 0, 0), head=(-18, 0, 0), arm_r=(20, 6, 0), arm_l=(38, -12, 0), elbow_r=(-16, 0, 0), elbow_l=(-14, 0, 0)), trunk(PASS)),
        # Brake: the body pitches up and the arms swing through.
        (0.7, pose(move=(0, -0.9, 0.06), motion=(8, 0, 0), head=(-2, 0, 0), arm_r=(-12, 10, 0), arm_l=(-18, -10, 0), elbow_r=(-34, 0, 0), elbow_l=(-38, 0, 0)), trunk(over(0.15, 0.05, 0.1))),
        # And drift back on a low arc.
        (0.86, pose(move=(0, -0.32, 0.07), motion=(-5, 0, 0), arm_r=(-4, 8, 0), arm_l=(-8, -8, 0), elbow_r=(-24, 0, 0), elbow_l=(-24, 0, 0))),
        (1, REST, STOP),
    ],
    "laugh": [
        (0, REST),
        # A snort first: the body tips in before it throws back.
        (0.08, pose(motion=(6, 0, 0), head=(10, 0, 0), arm_l=(-15, -8, 0), elbow_l=(-25, 0, 0)), STOP),
        (
            0.22,
            pose(motion=(-12, 0, 4), head=(-26, 0, 6), arm_l=(-48, -18, 0), elbow_l=(-50, 0, 0), arm_r=(-6, 10, 0), elbow_r=(-20, 0, 0)),
            over(0.15, 0.08, 0.15),
        ),
        (0.50, pose(motion=(-8, 0, -3), head=(-18, 0, -8), arm_l=(-44, -16, 0), elbow_l=(-46, 0, 0))),
        # A second peal.
        (0.64, pose(motion=(-13, 0, 2), head=(-28, 0, 4)), over(0.1, 0.06, 0.12)),
        (0.84, pose(motion=(-5, 0, 0), head=(-12, 0, 0), arm_l=(-36, -14, 0), elbow_l=(-40, 0, 0), arm_r=(0, 8, 0), elbow_r=(-14, 0, 0))),
        (1, REST, STOP),
    ],
    "cry": [
        (0, REST),
        # A breath in before it breaks.
        (0.10, pose(motion=(-3, 0, 0), head=(-6, 0, 0)), STOP),
        (
            0.25,
            pose(head=(20, 0, 0), motion=(10, 0, 0), arm_l=(-35, -10, -15), elbow_l=(-100, 0, 0), wrist_l=(-10, 0, 0), arm_r=(4, 6, 0), elbow_r=(-18, 0, 0)),
            over(0.1, 0.08, 0.15),
        ),
        (0.55, pose(head=(26, 0, -6), motion=(13, 0, -2), elbow_l=(-104, 0, 0))),
        (0.80, pose(head=(24, 0, -8), motion=(12, 0, 0), arm_l=(-40, -10, -15), elbow_l=(-105, 0, 0))),
        (1, REST, STOP),
    ],
    "rage": [
        (0, REST),
        # Coil: head down, fists drawn in...
        (
            0.28,
            pose(head=(20, 0, 0), motion=(10, 0, 0), arm_r=(-15, 30, 0), arm_l=(-15, -30, 0), elbow_r=(-65, 0, 0), elbow_l=(-65, 0, 0), wrist_r=(10, 0, 0), wrist_l=(10, 0, 0)),
            STOP,
        ),
        # ...and roar, arms flung wide.
        (
            0.55,
            pose(head=(-25, 0, 0), motion=(-8, 0, 0), move=(0, 0, 0.1), arm_r=(0, 50, 0), arm_l=(0, -50, 0), elbow_r=(-20, 0, 0), elbow_l=(-20, 0, 0), wrist_r=(-8, 0, 0), wrist_l=(-8, 0, 0)),
            over(0.12, 0.08, 0.16),
        ),
        (0.8, GUARD),
        (1, REST, STOP),
    ],
    "fear": [
        (0, REST),
        # A startle: thrown back, arms up to shield.
        (
            0.20,
            pose(
                move=(0, 0.28, 0.04),
                motion=(-12, 0, 0),
                head=(-12, 0, 15),
                arm_r=(-80, 15, 0),
                arm_l=(-100, -25, 0),
                elbow_r=(-40, 0, 0),
                elbow_l=(-30, 0, 0),
                wrist_l=(-20, 0, 0),
            ),
            over(0.18, 0.06, 0.14),
        ),
        # Cowering, the head ducked behind the arms...
        (0.48, pose(move=(0, 0.22, 0.0), motion=(-9, 0, 3), head=(6, 0, 10), elbow_r=(-48, 0, 0))),
        # ...a peek out the other side...
        (0.64, pose(head=(-6, 0, -12)), STOP),
        # ...and it comes down slowly.
        (0.80, pose(move=(0, 0.14, 0), motion=(-6, 0, 0), head=(0, 0, -4), arm_r=(-58, 12, 0), arm_l=(-68, -20, 0), elbow_r=(-50, 0, 0), elbow_l=(-40, 0, 0), wrist_l=(-8, 0, 0))),
        (1, REST, STOP),
    ],
    "bow": [
        (0, REST),
        # Rise a little, hand to the heart, then bow.
        (0.12, pose(motion=(-3, 0, 0), head=(-4, 0, 0), arm_l=(-20, -10, -15), elbow_l=(-40, 0, 0)), STOP),
        # The chest is already leaning into the bow as the hand settles.
        (0.26, pose(motion=(6, 0, 0), head=(3, 0, 0), arm_r=(-6, 8, 0), arm_l=(-35, -15, -25), elbow_l=(-68, 0, 0), wrist_l=(10, 0, 0))),
        (0.40, pose(motion=(26, 0, 0), head=(18, 0, 0)), over(0.08, 0.1, 0.2)),
        # Held, sinking a little further.
        (0.64, pose(motion=(30, 0, 0), head=(24, 0, 0)), STOP),
        (0.86, pose(motion=(2, 0, 0), head=(6, 0, 0), arm_l=(-12, -6, -8), elbow_l=(-30, 0, 0), wrist_l=(0, 0, 0))),
        (1, REST, STOP),
    ],
    "victory": [
        (0, REST),
        (0.10, pose(move=(0, 0, -0.04), motion=(8, 0, 0), head=(6, 0, 0), arm_r=(12, 12, 0), elbow_r=(-50, 0, 0)), STOP),
        (
            0.25,
            pose(move=(0, 0, 0.13), motion=(-6, 0, 0), head=(-16, 0, 0), arm_r=(-105, 20, 0), elbow_r=(-20, 0, 0), arm_l=(-8, -18, 0), elbow_l=(-20, 0, 0)),
            over(0.12, 0.08, 0.16),
        ),
        # Weapon aloft: it looks round, and the blade lifts and dips.
        (0.50, pose(head=(-12, 0, 14), arm_r=(-112, 24, 0), elbow_r=(-12, 0, 0), wrist_r=(-12, 0, 0))),
        (0.75, pose(move=(0, 0, 0.12), head=(-15, 0, -15), arm_r=(-110, 25, 0), elbow_r=(-15, 0, 0), wrist_r=(6, 0, 0), arm_l=(0, -25, 0))),
        (1, REST, STOP),
    ],
    "gigachad": [
        (0, REST),
        (0.12, pose(motion=(4, 0, 4), head=(6, 0, -4)), STOP),
        (
            0.30,
            pose(motion=(-8, 0, -12), head=(-18, 0, 24), arm_r=(0, 28, 0), arm_l=(0, -28, 0), elbow_r=(-50, 0, 0), elbow_l=(-50, 0, 0)),
            over(0.1, 0.1, 0.2),
        ),
        # Held with life, not frozen: the chest swells and the arms flex.
        (0.55, pose(motion=(-10, 0, -14), head=(-22, 0, 28), elbow_r=(-58, 0, 0), elbow_l=(-58, 0, 0)), STOP),
        (0.78, pose(motion=(-7, 0, -10), head=(-16, 0, 22), elbow_r=(-48, 0, 0), elbow_l=(-48, 0, 0))),
        (1, REST, STOP),
    ],
    "sigma": [
        (0, REST),
        (0.28, pose(head=(12, 0, -30), motion=(0, 0, 10)), STOP),
        (0.45, pose(head=(-12, 0, 12), motion=(-4, 0, 10)), over(0.15, 0.08, 0.18)),
        (0.80, pose(head=(-14, 0, 9), motion=(-5, 0, 12))),
        (1, REST, STOP),
    ],
    "shrug": [
        (0, REST),
        (0.12, pose(motion=(3, 0, 0), head=(4, 0, 0)), STOP),
        (
            0.30,
            pose(move=(0, 0, 0.03), motion=(-3, 0, 0), head=(0, -12, 0), arm_r=(0, 55, 0), arm_l=(0, -55, 0), elbow_r=(-15, 0, 0), elbow_l=(-15, 0, 0), wrist_l=(0, 0, 70)),
            over(0.12, 0.07, 0.14),
        ),
        (0.70, pose(move=(0, 0, 0.02), head=(0, 12, 0), arm_r=(0, 50, 0), arm_l=(0, -50, 0), elbow_r=(-18, 0, 0), elbow_l=(-18, 0, 0), wrist_l=(0, 0, 60))),
        (1, REST, STOP),
    ],
    "absoluteCinema": [
        (0, REST),
        (0.12, pose(motion=(5, 0, 0), head=(6, 0, 0), arm_r=(6, 30, 0), arm_l=(6, -30, 0), elbow_r=(-30, 0, 0), elbow_l=(-30, 0, 0)), STOP),
        (
            0.28,
            pose(
                move=(0, 0, 0.04),
                motion=(-4, 0, 0),
                arm_r=(-10, 125, 0),
                arm_l=(-10, -125, 0),
                elbow_r=(-25, 0, 0),
                elbow_l=(-25, 0, 0),
                wrist_r=(0, -40, 0),
                wrist_l=(0, 40, 0),
                head=(-8, 0, 0),
            ),
            over(0.08, 0.1, 0.2),
        ),
        # Held in awe, still rising: the arms keep opening and the head
        # tips back to take it in.
        (0.80, pose(move=(0, 0, 0.10), motion=(-9, 0, 0), arm_r=(-16, 138, 0), arm_l=(-16, -138, 0), elbow_r=(-16, 0, 0), elbow_l=(-16, 0, 0), head=(-22, 0, 0))),
        (1, REST, STOP),
    ],
    "tPose": [
        (0, REST),
        (0.10, pose(arm_r=(0, 20, 0), arm_l=(0, -20, 0), elbow_r=(-20, 0, 0), elbow_l=(-20, 0, 0)), STOP),
        (0.30, pose(arm_r=(0, 90, 0), arm_l=(0, -90, 0), elbow_r=(0, 0, 0), elbow_l=(0, 0, 0)), over(0.1, 0.06, 0.14)),
        (0.80, pose(arm_r=(-2, 92, 0), arm_l=(-2, -92, 0), head=(-3, 0, 0), move=(0, 0, 0.02))),
        (1, REST, STOP),
    ],
    "animeCharge": [
        (0, REST),
        (0.20, GUARD),
        # Power builds...
        (0.45, pose(move=(0, 0, 0.06), motion=(8, 0, 0), head=(10, 0, 0), arm_r=(0, 30, 0), arm_l=(0, -30, 0), elbow_r=(-75, 0, 0), elbow_l=(-75, 0, 0))),
        (
            0.72,
            pose(move=(0, 0, 0.16), motion=(12, 0, 0), head=(15, 0, 0), arm_r=(0, 40, 0), arm_l=(0, -40, 0), elbow_r=(-80, 0, 0), elbow_l=(-80, 0, 0)),
            STOP,
        ),
        # ...and bursts.
        (
            0.86,
            pose(move=(0, 0, 0.3), motion=(-8, 0, 0), head=(-25, 0, 0), arm_r=(0, 75, 0), arm_l=(0, -75, 0), elbow_r=(-15, 0, 0), elbow_l=(-15, 0, 0)),
            over(0.12, 0.05, 0.08),
        ),
        (1, REST, STOP),
    ],
}
KEYS.update(combat.KEYS)

# Laughing, sobbing, charging and fear shake the chest and head: cycles a
# clip and degrees, under an envelope that is still at both ends.
TREMBLE = {"laugh": (5, 2.0, 1.2), "cry": (5, 1.6, 1.0), "animeCharge": (7, 1.5, 0.6), "fear": (8, 0.8, 0.8)}

# Spirit travel: lean, glide trail and hover lift, by gait.
GLIDE = {"idle": (0, 0, 0.012), "creep": (4, 2, 0.010), "walk": (7, 5, 0.016), "run": (16, 12, 0.022), "sprint": (27, 22, 0.028)}
# Humanoid gaits: arm swing and forward lean.
STRIDE = {"creep": (9, 9), "walk": (22, 3), "run": (38, 12), "sprint": (48, 19)}


def keys(name, calling=None):
    """A key-posed clip's keys, as the class `calling` performs it
    (`combat.CLASS_KEYS`)."""

    if name in KEYS:
        return combat.for_class(KEYS[name], calling, name)

    if name in ("dodgeLeft", "dodgeRight"):
        side = 1 if name == "dodgeLeft" else -1

        return [
            (0, REST),
            (0.18, GUARD, STOP),
            (
                0.48,
                pose(move=(side * 0.6, 0, 0.08), motion=(10, side * -22, 0), head=(-4, side * 14, 0), arm_r=(-25, 20, 0), arm_l=(-40, -20, 0), elbow_r=(-24, 0, 0), elbow_l=(-36, 0, 0)),
                trunk(PASS),
            ),
            # Land on the far side, giving into it.
            (0.70, pose(move=(side * 0.62, 0, -0.02), motion=(6, side * 8, 0), head=(4, side * 4, 0), arm_r=(-30, 14, 0), arm_l=(-44, -16, 0), elbow_r=(-34, 0, 0), elbow_l=(-46, 0, 0)), trunk(stab(0.06, 0.12))),
            (1, REST, STOP),
        ]

    if name in ("dashLeft", "dashRight"):
        # +X is the reaper's left. Bank into a fast lateral burst, with
        # trailing arms and a longer excursion than the guarded dodge.
        side = 1 if name == "dashLeft" else -1

        return [
            (0, REST),
            (0.16, pose(move=(-side * 0.1, 0, -0.025), motion=(0, -side * 8, 0), head=(0, side * 6, 0)), STOP),
            (
                0.4,
                pose(move=(side * 0.95, 0, 0.1), motion=(12, side * 30, 0), head=(-6, -side * 12, 0), arm_r=(22, side * 18, 0), arm_l=(35, side * 18, 0), elbow_r=(-12, 0, 0), elbow_l=(-10, 0, 0)),
                trunk(PASS),
            ),
            (
                0.65,
                pose(move=(side * 1.05, 0, 0.05), motion=(5, side * 12, 0), head=(-2, -side * 4, 0), arm_r=(10, side * 8, 0), arm_l=(18, side * 8, 0), elbow_r=(-24, 0, 0), elbow_l=(-22, 0, 0)),
                trunk(over(0.12, 0.05, 0.1)),
            ),
            (1, REST, STOP),
        ]

    raise KeyError(f"no key-posed clip called {name!r}")


_TRACKS = {}


def _tracks(name, seconds, link=None, calling=None):
    """A clip's compiled keys, as the class `calling` performs it and the
    link `link` runs through it: a link on the blow (`ON_THE_BLOW`) has each
    contact key moved as late as its lead, so it reaches the contact pose on
    the impact itself."""

    synced = link in ON_THE_BLOW and name not in LOOPS and any(isinstance(key, combat.Strike) for key in keys(name))
    cache = (name, seconds, link if synced else None, calling if name in combat.CLASS_KEYS.get(calling, {}) else None)

    if cache not in _TRACKS:
        clip = keys(name, calling)

        if synced:
            ahead = min(LEAD[link] / seconds, MOST_LEAD)
            clip = [combat.Strike((key[0] + ahead * math.sin(math.pi * key[0]) ** 2, *key[1:])) if isinstance(key, combat.Strike) else key for key in clip]

        _TRACKS[cache] = curves.compile(clip, seconds)

    return _TRACKS[cache]


def _smooth(t):
    t = max(0.0, min(1.0, t))

    return t * t * (3 - 2 * t)


def _glance(p, start=0.30, turned=0.42, back=0.60, gone=0.74):
    """0 to 1 and back: a look aside, drifting a little while it is held."""

    if p < turned:
        return _smooth((p - start) / (turned - start))

    if p < back:
        return 1 - 0.18 * _smooth((p - turned) / (back - turned))

    return 0.82 * (1 - _smooth((p - back) / (gone - back)))


def _ring(p, start, width=0.06, cycles=9.0):
    """A small turn that starts still, peaks `width` later and rings down."""

    x = p - start

    if x <= 0:
        return 0.0

    u = x / width

    return u * u * math.exp(2 * (1 - u)) * math.cos(math.tau * cycles * x)


def _loop(name, p, humanoid):
    """A looping performance at `p`, closed form: Euler degrees by joint, and
    metres for `move`. Every term repeats exactly once a clip or more."""

    t = math.tau * p

    if name == "idle":
        # One breath a loop, a glance aside, and the weapon hand regripping
        # after it; a spirit hovers in a slow figure of eight, a humanoid
        # shifts its weight from foot to foot (`locomotion.grounded`).
        breath = math.sin(t)
        glance = _glance(p)
        regrip = _ring(p, 0.47)
        shift = locomotion.weight_shift(p)
        roll = -2.2 * shift if humanoid else 1.6 * math.sin(t + 0.9)

        return dict(
            move=(0.011 * math.sin(t), 0.004 * math.sin(2 * t + 0.5), 0.009 * math.sin(2 * t) + 0.004 * breath),
            motion=(-1.4 * breath, roll, 2.5 * math.sin(t + 2.0) + 4 * glance),
            head=(1.2 * math.sin(t - 0.5) + 2 * glance, -0.4 * roll, 14 * glance + 1.5 * math.sin(t + 0.4)),
            arm_r=(2.2 * math.sin(t - 0.6) + 1.0 * math.sin(2 * t + 0.4), 8 + 1.2 * breath, 0),
            arm_l=(1.6 * math.sin(t - 0.9), -8 - 1.0 * breath, 0),
            elbow_r=(-24 - 2.5 * breath + 4 * regrip, 0, 0),
            elbow_l=(-16 - 2.0 * breath, 0, 0),
            wrist_r=(1.5 * math.sin(2 * t + 0.4) - 3 * regrip, 0, 6 * regrip),
            wrist_l=(2.0 * math.sin(t - 1.2), 0, 0),
        )

    if humanoid:
        # The free arm drives the run; the object-carrying arm stays close.
        # Elbows bend as their arm swings forward, and the head turns
        # against the chest so the eyes stay level and ahead.
        swing, lean = STRIDE[name]
        wave = math.cos(t)
        base = -24 if name in ("creep", "walk") else -80

        return dict(
            motion=(lean, -2 * wave, -7 * wave),
            head=(-lean * 0.6 + 1.5 * math.cos(2 * t + 0.5), 0, 4 * wave),
            arm_r=(-swing * 0.35 * wave, 10, 0),
            arm_l=(swing * wave, -10, 0),
            elbow_r=(-32 - 4 * wave, 0, 0),
            elbow_l=(base - 12 * (0.5 - 0.5 * wave), 0, 0),
            wrist_r=(-3 * wave, 0, 0),
            wrist_l=(6 * wave, 0, 0),
        )

    # A spirit glides: speed is lean and hover, the arms trail quietly and
    # stir with the hover, and the robe streams behind (`build`).
    lean, trail, lift = GLIDE[name]
    wave = math.sin(t)

    return dict(
        move=(0.012 * wave, 0, lift * (1 - math.cos(t))),
        motion=(lean + 1.5 * math.sin(t - 0.6), 0.6 * wave, 1.2 * math.sin(t + 0.3)),
        head=(-lean * 0.5 + 1.6 * math.sin(t + 0.4), 0, 0.7 * wave),
        arm_r=(trail * 0.3 + 2.5 * wave, 8, 0),
        arm_l=(trail + 3.0 * wave, -8, 0),
        elbow_r=(-24 - 3 * wave, 0, 0),
        elbow_l=(-16 - 4 * wave, 0, 0),
        wrist_r=(2 * wave, 0, 0),
        wrist_l=(3 * wave, 0, 0),
    )


def sample(name, phase, seconds, humanoid=False, link=None, calling=None):
    """Every joint of the performance `name` at `phase`, as the class
    `calling` performs it and the link `link` runs through it (`_tracks`),
    before overlap: a rotation vector in degrees for a turn (`curves`),
    metres for `move`."""

    if name in LOOPS:
        raw = _loop(name, phase, humanoid)
        values = {joint: Vector(value) if joint == curves.TRANSLATION else curves.turn(value) for joint, value in raw.items()}
    else:
        values = curves.sample(_tracks(name, seconds, link, calling), phase)

    if name in TREMBLE:
        cycles, chest, head = TREMBLE[name]
        tremble = math.sin(phase * math.tau * cycles) * math.sin(phase * math.pi) ** 2
        values["motion"] = values.get("motion", Vector()) + Vector((chest * tremble, 0, 0))
        values["head"] = values.get("head", Vector()) - Vector((head * tremble, 0, 0))

    return values


def lead(name, phase, seconds, link):
    """When in the performance `name` the link `link` is at `phase` of the
    clip: ahead of the weapon hand in a one-shot, behind the pelvis in a
    loop (`LEAD`)."""

    if name in LOOPS:
        return (phase - (LEAD["pelvis"] - LEAD[link]) / seconds) % 1.0

    ahead = min(LEAD[link] / seconds, MOST_LEAD)

    return phase + ahead * math.sin(math.pi * phase) ** 2


def variants(specs):
    """Keep 32 studio states while exporting distinct performances per form.
    Remembers each clip's length for `hold`."""

    for spec in specs:
        TIMINGS[spec["name"]] = spec["seconds"]

    return [dict(spec, motion=spec["name"], humanoid=human, name=("humanoid_" if human else "") + spec["name"]) for human in (False, True) for spec in specs]


# The clips a class performs with its other hand: the Hollow strikes with
# its right, the lantern in its left.
MIRRORED = {"hollow": ("finisherPalm", "finisherSoulCrush")}


def _mirrored(values):
    """A sampled pose as the other hand performs it: left and right swap,
    and the whole body turns, leans and steps the other way with them."""

    out = {}

    for joint, value in values.items():
        if joint.endswith(("_l", "_r")):
            joint = joint[:-1] + ("l" if joint.endswith("_r") else "r")

        out[joint] = Vector((-value.x, value.y, value.z)) if joint == curves.TRANSLATION else Vector((value.x, -value.y, -value.z))

    return out


def performance(name, phase, humanoid, calling, seconds):
    """Sample the whole body once, each link at its own time (`lead`)."""

    poses = {}
    mirror = name in MIRRORED.get(calling, ())

    def at(link):
        if link not in poses:
            values = sample(name, lead(name, phase, seconds, link), seconds, humanoid, link, calling)
            poses[link] = _mirrored(values) if mirror else values

        return poses[link]

    zero = Vector()
    pelvis = at("pelvis")
    hips = pelvis.get("motion", zero)
    chest = at("spine").get("motion", zero)
    positions = {joint: Vector() for joint in JOINTS}
    rotations = {joint: Quaternion() for joint in JOINTS}

    if humanoid:
        # A loop's pelvis is on the clip's own phase, in step with its feet.
        move, yaw, feet = locomotion.grounded(name, lead(name, phase, seconds, "pelvis"), pelvis)
        # Lower the pelvis when a long stride needs more reach. Preserve bone
        # lengths and planted ankles instead of stretching or clipping a leg.
        yaw_rotation = curves.quaternion(Vector((0, 0, yaw)))

        for side, sign in (("l", 1), ("r", -1)):
            hip_offset = yaw_rotation @ Vector((sign * locomotion.HIP_X, 0, 0))
            ankle = feet[side][0]
            horizontal = Vector((move.x + hip_offset.x - ankle.x, move.y + hip_offset.y - ankle.y))
            vertical = math.sqrt((2 * locomotion.LENGTH - 0.012) ** 2 - horizontal.length_squared)
            move.z = min(move.z, ankle.z + vertical - locomotion.HIP_Z)

        root = Vector((0, 0, yaw))
        spine = Vector((chest.x, chest.y, chest.z - yaw))
    else:
        move = Vector(pelvis.get("move", zero))
        root = Vector((hips.x * 0.18, hips.y * 0.4, hips.z * 0.45))
        spine = Vector((chest.x * 0.82, chest.y * 0.6, chest.z * 0.55))

    rotations["motion"] = curves.quaternion(root)
    rotations["spine"] = curves.quaternion(spine)
    positions["motion"] = move

    for joint, link in LINKS.items():
        value = at(link).get(joint, zero)
        gain = AMPLITUDE[calling]

        if calling == "mourner" and joint == "head":
            gain *= 0.25

        rotations[joint] = curves.quaternion(value * gain)

    if humanoid:
        root_rotation = rotations["motion"]
        root_position = Vector((0, 0, 1.05)) + move

        for side, sign in (("l", 1), ("r", -1)):
            hip = root_position + root_rotation @ Vector((sign * locomotion.HIP_X, 0, locomotion.HIP_Z - 1.05))
            ankle, foot_rotation = feet[side]

            try:
                thigh, knee, shin = locomotion.solve(hip, ankle, root_rotation @ Vector((0, -1, 0)))
            except ValueError as error:
                raise ValueError(f"{name} phase={phase:.3f} {side}: {error}") from error

            rotations[f"leg_{side}"] = root_rotation.inverted() @ thigh
            rotations[f"knee_{side}"] = knee
            rotations[f"foot_{side}"] = shin.inverted() @ foot_rotation

    return positions, rotations


def rig(root):
    """Inserts the waist (`motion`) and chest (`spine`) pivots, keeping every
    part where it is. Returns the joints by name and their rest locations."""
    children = list(root.children)
    pivot = core.empty("motion", root, location=(0, 0, 1.05))
    for child in children:
        child.parent = pivot
        child.location.z -= 1.05
    spine = core.empty("spine", pivot, location=(0, 0, 0.13))
    for child in list(pivot.children):
        if child.name in ("head", "arm_l", "arm_r", "spirit_body") or child.name.startswith("mantle_"):
            child.parent = spine
            child.location.z -= 0.13
    nodes = {node.name: node for node in root.children_recursive}
    joints = {name: nodes[name] for name in JOINTS}
    rest = {name: node.location.copy() for name, node in joints.items()}
    return joints, rest


def hold(joints, rest, calling, name, phase, humanoid):
    """Poses the joints at `phase` of the performance `name` without baking
    a clip: what the clip's key there would show. For the fast review. The
    clip's length comes from the specs `variants` last read."""

    if name not in TIMINGS:
        raise KeyError(f"no timing for {name!r}: read the specs with motion.variants first")

    positions, rotations = performance(name, phase, humanoid, calling, TIMINGS[name])
    for joint, node in joints.items():
        node.rotation_mode = "QUATERNION"
        node.rotation_quaternion = rotations[joint]
        node.location = rest[joint] + positions[joint]
    bpy.context.view_layer.update()


# Baking ----------------------------------------------------------------------


def timeline(specs):
    """Where each clip lies on the bake's timeline, at `RATE`: `(spec, first
    frame, frames)`. Every baked track (joints, faces, robes) lays its clip
    there, which is how the exporter merges them into one named clip."""

    first = 1

    for spec in specs:
        frames = round(spec["seconds"] * RATE)
        yield spec, first, frames
        first += frames + 5


def check_contact(spec):
    """Refuses a clip whose contact key is not at its impact in
    `ReaperMotion`: the game lands the blow there."""

    name, impact = spec["motion"], spec.get("impact")

    if name in LOOPS:
        found = []
    else:
        found = combat.contacts(keys(name))

    if impact is None and found:
        raise ValueError(f"{name}: has a contact key at {found}, but ReaperMotion gives it no impact")

    if impact is not None and (len(found) != 1 or abs(found[0] - impact) > 1e-9):
        raise ValueError(f"{name}: its contact key is at {found or 'none'}, but ReaperMotion lands its impact at {impact}")


# A strike crosses its contact near the top of its speed: the striking
# hand, at the impact, at least this share of the fastest it moves in the
# clip. Below it the blow reads as a hand slowing onto its mark.
CONTACT_SHARE = 0.6


def striking_hand(name, calling):
    """The wrist that lands the blow in the clip `name`."""

    if name in ("finisherPalm", "finisherSoulCrush"):
        return "wrist_r" if name in MIRRORED.get(calling, ()) else "wrist_l"

    return "wrist_r"


def contact_share(spec, calling, joints, locals_):
    """The striking hand's speed across the impact as a share of its top
    speed in the clip (`CONTACT_SHARE`), from the clip's posed frames
    `locals_`; None for a clip that lands no blow."""

    impact = spec.get("impact")

    if impact is None:
        return None

    hand = joints[striking_hand(spec["motion"], calling)]
    points = [_world(hand, local).translation for local in locals_]
    speeds = [(after - before).length for before, after in zip(points, points[1:])]
    at = min(round(impact * (len(points) - 1)), len(speeds) - 1)

    return speeds[at] / max(speeds)


def check_strike(spec, calling, joints, locals_):
    """Refuses a strike whose hand is slowing onto its mark at the impact
    (`contact_share`)."""

    share = contact_share(spec, calling, joints, locals_)

    if share is not None and share < CONTACT_SHARE:
        raise ValueError(f"{spec['name']} ({calling}): the striking hand crosses the impact at {share:.0%} of its top speed; a strike needs {CONTACT_SHARE:.0%}")


def key(owner, id_type, action_name, track, first, channels):
    """Keys `channels`, `{(data path, index): value per frame}`, on `owner`
    (an object, or a mesh's shape keys) as one action, laid on an NLA track
    called `track` from frame `first`."""

    data = owner.animation_data or owner.animation_data_create()
    action = bpy.data.actions.new(action_name)
    slot = action.slots.new(id_type=id_type, name=owner.name)
    bag = anim_utils.action_ensure_channelbag_for_slot(action, slot)

    for (path, index), values in channels.items():
        curve = bag.fcurves.new(path, index=index)
        curve.keyframe_points.add(len(values))
        curve.keyframe_points.foreach_set("co", [number for frame, value in enumerate(values) for number in (frame, value)])
        # Linear, as the game plays them: one key a frame.
        curve.keyframe_points.foreach_set("interpolation", [1] * len(values))
        curve.update()

    lane = data.nla_tracks.new()
    lane.name = track
    strip = lane.strips.new(track, first, action)
    strip.action_slot = slot
    strip.extrapolation = "NOTHING"


def _world(node, local):
    """`node`'s world matrix, with `local` (by name) for the nodes a frame
    poses and each other node's own transform."""

    chain = []

    while node is not None:
        chain.append(node)
        node = node.parent

    matrix = Matrix.Identity(4)

    for item in reversed(chain):
        basis = local.get(item.name, item.matrix_basis)
        matrix = basis.copy() if item.parent is None else matrix @ item.matrix_parent_inverse @ basis

    return matrix


def spring(targets, frequency, damping, loop):
    """Where a point a damped spring pulls toward each frame's target is, at
    every frame. It lags only while its target speeds up or slows down. A
    loop is run until it repeats, and closed exactly."""

    steps = 8
    h = 1 / (RATE * steps)
    stiffness = (math.tau * frequency) ** 2
    drag = 2 * damping * math.tau * frequency
    point, velocity = targets[0].copy(), Vector()

    for _ in range(4 if loop else 1):
        out = [point.copy()]

        for before, after in zip(targets, targets[1:]):
            moving = (after - before) * RATE

            for step in range(steps):
                goal = before.lerp(after, (step + 1) / steps)
                velocity += (stiffness * (goal - point) + drag * (moving - velocity)) * h
                point += velocity * h

            out.append(point.copy())

    if loop:
        # Spread what the last pass misses closing by over the loop.
        miss = out[-1] - out[0]
        out = [point - miss * (index / (len(out) - 1)) for index, point in enumerate(out)]

    return out


def _settled(frames, loop):
    """1 through a clip, falling to 0 over a one-shot's last 15%, which
    ends at rest and holds there."""

    if loop:
        return [1.0] * (frames + 1)

    return [1 - _smooth((frame / frames - 0.85) / 0.15) for frame in range(frames + 1)]


# Where a robe's hem hangs below the waist pivot, and how its sway morphs
# move it: `cloth._add_motion_keys` moves the hem 0.2 m at weight 1.
HEM = Vector((0, 0, -0.9))
SWAY_METRES = 0.2
BILLOW_METRES = 0.06
ROBE = (1.5, 0.5)  # frequency (Hz), damping ratio
# Spirit gaits are in place: their speed shows as the robe streaming back.
TRAIL = {"creep": 0.12, "walk": 0.22, "run": 0.45, "sprint": 0.7}
# A grounded garment cannot drift through the legs like an empty spirit robe.
HUMANOID_DRIFT = 0.35


def robe_weights(name, humanoid, waists):
    """The robe's `sway_x`, `sway_y` and `billow` at each frame, from where
    its waist (the `motion` pivot) is in the world at each frame."""

    loop = name in LOOPS
    frames = len(waists) - 1
    targets = [waist @ HEM for waist in waists]
    hem = spring(targets, *ROBE, loop)
    fade = _settled(frames, loop)
    drift = HUMANOID_DRIFT if humanoid else 1.0
    trail = TRAIL.get(name, 0.0)
    weights = {"sway_x": [], "sway_y": [], "billow": []}

    for frame, (waist, point, target) in enumerate(zip(waists, hem, targets)):
        lag = waist.to_quaternion().inverted() @ (point - target) * fade[frame]
        rhythm = math.sin(math.tau * 2 * frame / frames)
        streaming = trail * (1 + 0.12 * rhythm)
        weights["sway_x"].append(max(-1.0, min(1.0, drift * lag.x / SWAY_METRES)))
        weights["sway_y"].append(max(-1.0, min(1.0, drift * (lag.y / SWAY_METRES + streaming))))
        weights["billow"].append(max(0.0, min(1.0, max(0.0, lag.z) / BILLOW_METRES + 0.4 * trail * (1 + 0.15 * rhythm))))

    return weights


# The Hollow's lantern swings from its handle: where its weight hangs below
# the handle, how freely it swings, and how far.
BOB = Vector((0, 0, -0.30))
LANTERN = (1.6, 0.22)
SWING_DEGREES = 35.0


def swing(name, handles):
    """The lantern's turn about its handle at each frame, in its own rest
    frame, from where its handle (rigidly carried) is at each frame."""

    loop = name in LOOPS
    frames = len(handles) - 1
    targets = [handle @ BOB for handle in handles]
    weight = spring(targets, *LANTERN, loop)
    fade = _settled(frames, loop)
    down = BOB.normalized()
    turns = []

    for frame, (handle, point) in enumerate(zip(handles, weight)):
        hanging = handle.to_quaternion().inverted() @ (point - handle.translation)
        turn = down.rotation_difference(hanging.normalized())
        axis, angle = turn.to_axis_angle()
        # A soft limit: a small swing is left as it is, a wild one eases
        # toward `SWING_DEGREES` and never meets a wall there.
        limit = math.radians(SWING_DEGREES)
        angle = limit * math.tanh(angle / limit) * fade[frame]
        turns.append(Quaternion(axis, angle))

    return turns


def build(root, calling, specs):
    """Bake form-specific clips onto one hierarchy with a chest and pelvis,
    with the robe's sway and anything that hangs swinging from the body's
    own motion."""

    joints, rest = rig(root)
    bpy.context.scene.render.fps = RATE
    bpy.context.scene.render.fps_base = 1.0
    nodes = {node.name: node for node in root.children_recursive}
    robes = [node for node in nodes.values() if node.name.startswith("robe_") and node.type == "MESH" and node.data.shape_keys is not None]
    lantern = nodes.get("lantern")
    lantern_basis = lantern.matrix_basis.copy() if lantern is not None else None

    for spec, first, frames in timeline(specs):
        check_contact(spec)
        name, clip = spec["motion"], spec["name"]
        samples = [performance(name, frame / frames, spec["humanoid"], calling, spec["seconds"]) for frame in range(frames + 1)]

        for joint, node in joints.items():
            node.animation_data_create()
            node.animation_data.action = None
            node.rotation_mode = "QUATERNION"
            channels = {("rotation_quaternion", axis): [] for axis in range(4)}
            channels.update({("location", axis): [] for axis in range(3)})
            previous = None

            for positions, rotations in samples:
                rotation = rotations[joint].copy()

                if previous is not None and previous.dot(rotation) < 0:
                    rotation.negate()

                previous = rotation
                location = rest[joint] + positions[joint]

                for axis in range(4):
                    channels["rotation_quaternion", axis].append(rotation[axis])

                for axis in range(3):
                    channels["location", axis].append(location[axis])

            key(node, "OBJECT", f"{clip}_{joint}", clip, first, channels)
            node.location = rest[joint]
            node.rotation_quaternion = Quaternion()

        locals_ = [
            {joint: Matrix.Translation(rest[joint] + positions[joint]) @ rotations[joint].to_matrix().to_4x4() for joint in joints}
            for positions, rotations in samples
        ]
        check_strike(spec, calling, joints, locals_)

        if not robes and lantern is None:
            continue

        if robes:
            weights = robe_weights(name, spec["humanoid"], [_world(joints["motion"], local) for local in locals_])

            for robe in robes:
                data = robe.data.shape_keys
                # A sway runs both ways; Blender clamps a key's value at
                # its slider, 0 by default, and exports what it evaluates.
                data.key_blocks["sway_x"].slider_min = data.key_blocks["sway_y"].slider_min = -1.0
                data.animation_data_create()
                data.animation_data.action = None
                channels = {(f'key_blocks["{target}"].value', 0): values for target, values in weights.items()}
                key(data, "KEY", f"{clip}_{robe.name}", clip, first, channels)

        if lantern is not None:
            handles = [_world(lantern.parent, local) @ lantern.matrix_parent_inverse @ lantern_basis for local in locals_]
            rest_turn = lantern_basis.to_quaternion()
            lantern.animation_data_create()
            lantern.animation_data.action = None
            lantern.rotation_mode = "QUATERNION"
            channels = {("rotation_quaternion", axis): [] for axis in range(4)}
            previous = None

            for turn in swing(name, handles):
                rotation = rest_turn @ turn

                if previous is not None and previous.dot(rotation) < 0:
                    rotation.negate()

                previous = rotation

                for axis in range(4):
                    channels["rotation_quaternion", axis].append(rotation[axis])

            key(lantern, "OBJECT", f"{clip}_lantern", clip, first, channels)
            lantern.rotation_quaternion = rest_turn

    bpy.context.scene.frame_set(0)
    return joints
