"""Combat key poses: anticipation, a contact crossed at the top of its
speed, a follow-through that carries past its pose and settles, and one
unhurried recovery home.

Every pose names what it means to move and holds the rest (`curves`): the
off hand keeps its guard through a wind-up instead of dropping to its bind
pose between keys. The head counters the chest's twist, so the eyes stay on
the target while the body winds and unwinds around them.

Each attack's contact is a `strike` key. Its time is the clip's impact in
`ReaperMotion` (`art/build/motions.json`); `motion.build` refuses to bake a
clip whose contact key is anywhere else, or whose weapon (or striking hand)
crosses it at less than `motion.CONTACT_SHARE` of its top speed. A swing
speeds up all the way into its contact (`curves.STRIKE`); a stab drives on
past it and settles back (`curves.stab`). The pelvis, chest and head lead
the blow and are already slowing as it lands; the arm starts its swing
ahead of the hand but meets the contact with it (`motion.ON_THE_BLOW`).

After the contact the blade only slows: it carries on past its
follow-through pose and settles (`over`), then comes home in one
unhurried move (a `PASS` key halfway, at the top of its speed), so its
speed has one peak at the contact and one gentle hump on the way home.
"""

from .curves import AUTO, PASS, REST, STOP, STRIKE, over, pose, stab, stance


class Strike(tuple):
    """A contact key: `(time, pose, manner)`, at the clip's impact."""


def strike(time, shown, manner=STRIKE):
    return Strike((time, shown, manner))


def contacts(keys):
    """The times of a clip's contact keys."""

    return [key[0] for key in keys if isinstance(key, Strike)]


GUARD = stance(arm_r=(-25, 16, -8), arm_l=(-32, -20, 8), elbow_r=(-62, 0, 0), elbow_l=(-85, 0, 0), head=(5, 0, 0))
# A landing gives in the body: it falls faster and faster onto its feet,
# sinks on past the pose as far as that speed carries it, slowing all the
# way, and springs back.
LANDING = {"*": AUTO, "move": stab(0.07, 0.14), "motion": stab(0.07, 0.14)}


def trunk(manner, limbs=AUTO):
    """A key manner for the body alone (`move` and `motion`), the limbs
    passing the key as `limbs` says: a landing gives in the hips, not in the
    arms, and a turn runs out in the body while the blade sweeps on."""

    return {"*": limbs, "move": manner, "motion": manner}


# While the arm loads, the chest is already unwinding, the head turning
# against it: only the limbs stop.
LOADED = {"*": STOP, "motion": AUTO, "move": AUTO, "head": AUTO}

# A follow-through: the limbs carry past the pose and settle back, the body
# a little less.
FOLLOW = {"*": over(0.12, 0.07, 0.14), "move": over(0.08, 0.07, 0.12), "motion": over(0.08, 0.07, 0.12)}

KEYS = {
    "slash": [
        (0, REST),
        (
            0.24,
            pose(
                move=(0, 0.04, -0.03),
                motion=(-8, -5, -42),
                head=(4, 0, 20),
                arm_r=(-45, 40, -32),
                elbow_r=(-90, 0, 0),
                wrist_r=(0, 30, -20),
                arm_l=(-30, -24, 0),
                elbow_l=(-65, 0, 0),
            ),
            STOP,
        ),
        (0.34, pose(motion=(-3, -3, -30), head=(2, 0, 16), arm_r=(-58, 48, -38), elbow_r=(-78, 0, 0), wrist_r=(0, 42, -24)), LOADED),
        # The blade crosses at chest height, its shaft level and clear of
        # the hat brim, the elbow still straightening and the wrist still
        # snapping through...
        strike(
            0.45,
            pose(
                move=(0, -0.18, -0.04),
                motion=(12, 5, 34),
                head=(-2, 0, -16),
                arm_r=(-45, -6, 40),
                elbow_r=(-26, 0, 0),
                wrist_r=(-40, -8, 20),
                arm_l=(18, -35, 0),
                elbow_l=(-30, 0, 0),
            ),
        ),
        # ...sweeps on across the body, past the follow-through and back...
        (
            0.57,
            pose(move=(0, -0.21, -0.05), motion=(16, 7, 54), head=(0, 0, -24), arm_r=(-56, -28, 84), elbow_r=(-12, 0, 0), wrist_r=(-44, -40, 32), arm_l=(30, -38, 0), elbow_l=(-26, 0, 0)),
            FOLLOW,
        ),
        # ...and comes home in one move.
        (
            0.80,
            pose(move=(0, -0.07, -0.02), motion=(6, 2, 20), head=(2, 0, -8), arm_r=(-30, -10, 40), elbow_r=(-34, 0, 0), wrist_r=(-16, -20, 16), arm_l=(4, -16, 0), elbow_l=(-30, 0, 0)),
            PASS,
        ),
        (1, REST, STOP),
    ],
    "thrust": [
        (0, REST),
        (
            0.28,
            pose(
                move=(0, 0.09, -0.07),
                motion=(-6, 0, -27),
                head=(6, 0, 14),
                arm_r=(-24, 8, -12),
                elbow_r=(-100, 0, 0),
                wrist_r=(112, 0, 0),
                arm_l=(-42, -20, 0),
                elbow_l=(-65, 0, 0),
            ),
            STOP,
        ),
        (0.36, pose(move=(0, 0.02, -0.06), motion=(4, 0, -12), head=(3, 0, 8), arm_r=(-32, 6, -6), elbow_r=(-88, 0, 0), wrist_r=(115, 0, 0)), LOADED),
        # The point lands at chest height and drives on in, the elbow
        # locking short of straight, and settles back.
        strike(
            0.45,
            pose(
                move=(0, -0.32, -0.02),
                motion=(18, 0, 22),
                head=(-6, 0, -12),
                arm_r=(-62, 0, 8),
                elbow_r=(-30, 0, 0),
                wrist_r=(150, 0, 0),
                arm_l=(10, -25, 0),
                elbow_l=(-35, 0, 0),
            ),
            {"*": stab(0.05, 0.1), "elbow_r": stab(0.05, 0.1, 0.4), "move": stab(0.05, 0.1, 0.15), "motion": stab(0.05, 0.1, 0.15)},
        ),
        # It draws the blade back out round a loop rather than straight down
        # the line it went in on, and comes home.
        (
            0.70,
            pose(move=(0, -0.12, -0.02), motion=(6, 0, 10), head=(0, 0, -4), arm_r=(-52, 8, 34), elbow_r=(-48, 0, 0), wrist_r=(120, 0, 0), arm_l=(-6, -20, 0), elbow_l=(-40, 0, 0)),
            PASS,
        ),
        (1, REST, STOP),
    ],
    "parry": [
        (0, REST),
        (0.18, GUARD),
        # The blade snaps across the line and stops dead there: a block.
        (
            0.32,
            pose(motion=(4, 0, -18), head=(2, 0, 10), arm_r=(-58, 32, -22), elbow_r=(-65, 0, 0), wrist_r=(12, 15, 52), arm_l=(-35, -20, 0), elbow_l=(-80, 0, 0)),
            STOP,
        ),
        # The blow lands on it and drives the body back, which gives and
        # springs back.
        (
            0.43,
            pose(move=(0, 0.10, -0.05), motion=(-8, -4, -28), head=(-2, 0, 14), arm_r=(-52, 24, -30), elbow_r=(-80, 0, 0), wrist_r=(5, 5, 42), arm_l=(-40, -24, 0), elbow_l=(-92, 0, 0)),
            over(0.2, 0.05, 0.1),
        ),
        # And throws it off, the blade sweeping on outward and round, down
        # to the guard.
        (0.56, pose(move=(0, -0.04, -0.02), motion=(6, 0, 8), head=(4, 0, -4), arm_r=(-66, 40, 10), elbow_r=(-32, 0, 0), wrist_r=(0, -15, 62), arm_l=(-28, -18, 0), elbow_l=(-70, 0, 0))),
        (0.68, pose(move=(0, -0.03, -0.01), motion=(4, 0, 12), head=(4, 0, -2), arm_r=(-50, 34, 44), elbow_r=(-26, 0, 0), wrist_r=(0, -18, 70), arm_l=(-22, -16, 0), elbow_l=(-58, 0, 0))),
        (0.84, stance(motion=(2, 0, 3), head=(3, 0, 0), arm_r=(-28, 18, 16), elbow_r=(-40, 0, 0), wrist_r=(0, -6, 24), arm_l=(-16, -12, 0), elbow_l=(-44, 0, 0))),
        (1, REST, STOP),
    ],
    "finisherReap": [
        (0, REST),
        # A small counter-move before the great wind-up: the body anticipates.
        (0.10, pose(move=(0, -0.02, 0), motion=(3, 2, 10), head=(2, 0, -5), arm_r=(-18, 14, 6), elbow_r=(-40, 0, 0)), STOP),
        (
            0.27,
            pose(
                move=(0, 0.08, -0.07),
                motion=(-12, -6, -48),
                head=(5, 0, 24),
                arm_r=(-48, 45, -45),
                elbow_r=(-78, 0, 0),
                wrist_r=(8, 55, -20),
                arm_l=(-28, -20, 0),
                elbow_l=(-60, 0, 0),
            ),
        ),
        (0.41, pose(motion=(-5, -8, -34), head=(3, 0, 20), arm_r=(-68, 55, -40), elbow_r=(-62, 0, 0), wrist_r=(0, 36, -20)), LOADED),
        # The slash, wider: level through the contact under the brim...
        strike(
            0.52,
            pose(
                move=(0, -0.28, -0.05),
                motion=(17, 8, 48),
                head=(-2, 0, -20),
                arm_r=(-46, -12, 52),
                elbow_r=(-20, 0, 0),
                wrist_r=(-40, -30, 26),
                arm_l=(20, -42, 0),
                elbow_l=(-30, 0, 0),
            ),
        ),
        # ...on round the body, past the follow-through and back...
        (
            0.59,
            pose(move=(0, -0.31, -0.06), motion=(20, 12, 70), head=(-4, 0, -26), arm_r=(-58, -36, 98), elbow_r=(-10, 0, 0), wrist_r=(-46, -56, 36), arm_l=(28, -38, 0), elbow_l=(-38, 0, 0)),
            FOLLOW,
        ),
        # ...held there a beat, the blade still drifting on...
        (0.72, pose(move=(0, -0.28, -0.05), motion=(18, 10, 66), head=(-3, 0, -24), arm_r=(-60, -32, 92), elbow_r=(-14, 0, 0), wrist_r=(-42, -50, 36), arm_l=(24, -34, 0), elbow_l=(-40, 0, 0))),
        # ...and home in one move.
        (
            0.86,
            pose(move=(0, -0.1, -0.02), motion=(8, 4, 26), head=(2, 0, -8), arm_r=(-30, -12, 44), elbow_r=(-36, 0, 0), wrist_r=(-16, -24, 16), arm_l=(2, -20, 0), elbow_l=(-40, 0, 0)),
            PASS,
        ),
        (1, REST, STOP),
    ],
    "finisherImpale": [
        (0, REST),
        (0.12, pose(move=(0, -0.02, 0.01), motion=(4, 0, 8), head=(-2, 0, -4), arm_r=(-30, 10, 0), elbow_r=(-70, 0, 0), wrist_r=(60, 0, 0)), STOP),
        (
            0.29,
            pose(
                move=(0, 0.10, -0.08),
                motion=(-10, 0, -34),
                head=(6, 0, 18),
                arm_r=(-15, 12, -20),
                elbow_r=(-105, 0, 0),
                wrist_r=(108, 0, 0),
                arm_l=(-36, -22, 0),
                elbow_l=(-70, 0, 0),
            ),
        ),
        (0.39, pose(motion=(3, 0, -18), head=(2, 0, 10), arm_r=(-28, 8, -12), elbow_r=(-90, 0, 0), wrist_r=(115, 0, 0)), LOADED),
        # The point goes in, drives on and settles...
        strike(
            0.46,
            pose(
                move=(0, -0.40, -0.04),
                motion=(22, 0, 26),
                head=(-6, 0, -14),
                arm_r=(-66, 0, 15),
                elbow_r=(-30, 0, 0),
                wrist_r=(160, 0, 0),
                arm_l=(12, -28, 0),
                elbow_l=(-40, 0, 0),
            ),
            {"*": stab(0.06, 0.14), "elbow_r": stab(0.06, 0.14, 0.4), "move": stab(0.06, 0.14, 0.15), "motion": stab(0.06, 0.14, 0.15)},
        ),
        # ...and is leant on, still pushing.
        (0.59, pose(move=(0, -0.42, -0.03), motion=(19, 0, 28), head=(-5, 0, -15), arm_r=(-70, 0, 18), elbow_r=(-18, 0, 0), wrist_r=(164, 0, 0))),
        # Lift what is on the point, the off hand bracing, and fling it off
        # outward, the blade sweeping round and down rather than dropping
        # back the way it rose.
        (
            0.70,
            pose(move=(0, -0.25, 0), motion=(-6, 0, 15), head=(-14, 0, -8), arm_r=(-96, 6, 26), elbow_r=(-52, 0, 0), wrist_r=(150, 0, 0), arm_l=(-40, -20, 0), elbow_l=(-72, 0, 0)),
            trunk(over(0.1, 0.08, 0.14)),
        ),
        (0.80, pose(move=(0, -0.16, 0), motion=(-2, 0, 18), head=(-8, 0, -4), arm_r=(-88, 12, 50), elbow_r=(-46, 0, 0), wrist_r=(100, 0, 0), arm_l=(-28, -18, 0), elbow_l=(-58, 0, 0))),
        (0.90, pose(move=(0, -0.06, 0), motion=(2, 0, 8), head=(-2, 0, -2), arm_r=(-48, 10, 30), elbow_r=(-50, 0, 0), wrist_r=(40, 0, 0), arm_l=(-18, -14, 0), elbow_l=(-44, 0, 0)), PASS),
        (1, REST, STOP),
    ],
    "finisherPalm": [
        (0, REST),
        # The palm draws back, past its wind-up and back: the body coils.
        (
            0.30,
            pose(
                move=(0, 0.08, -0.08),
                motion=(8, 0, -35),
                head=(6, 0, 18),
                arm_l=(-12, -26, -34),
                elbow_l=(-112, 0, 0),
                wrist_l=(-18, 0, 0),
                arm_r=(-28, 20, 0),
                elbow_r=(-80, 0, 0),
            ),
            over(0.15, 0.08, 0.14),
        ),
        (0.41, pose(motion=(5, 0, -20), head=(4, 0, 10), arm_l=(-26, -22, -30), elbow_l=(-98, 0, 0), wrist_l=(-20, 0, 0)), LOADED),
        # It strikes out round an arc and drives on in, the elbow locking
        # short of straight, and settles.
        strike(
            0.50,
            pose(
                move=(0, -0.32, -0.02),
                motion=(16, 0, 28),
                head=(-4, 0, -14),
                arm_l=(-85, -8, 15),
                elbow_l=(-30, 0, 0),
                wrist_l=(-12, 0, 0),
                arm_r=(12, 22, 0),
                elbow_r=(-40, 0, 0),
            ),
            {"*": stab(0.06, 0.14), "elbow_l": stab(0.06, 0.14, 0.4), "move": stab(0.06, 0.14, 0.15), "motion": stab(0.06, 0.14, 0.15)},
        ),
        (0.64, pose(move=(0, -0.34, -0.02), motion=(18, 0, 33), head=(-5, 0, -16), arm_l=(-88, -4, 22), elbow_l=(-12, 0, 0), wrist_l=(-14, 0, 0), arm_r=(16, 20, 0), elbow_r=(-46, 0, 0))),
        (0.82, pose(move=(0, -0.08, -0.01), motion=(5, 0, 10), head=(1, 0, -4), arm_l=(-36, -14, 16), elbow_l=(-44, 0, 0), wrist_l=(-4, 0, 0), arm_r=(-6, 16, 0), elbow_r=(-38, 0, 0)), PASS),
        (1, REST, STOP),
    ],
    "finisherUppercut": [
        (0, REST),
        (
            0.29,
            pose(
                move=(0, 0, -0.14),
                motion=(22, 0, -32),
                head=(10, 0, 16),
                arm_r=(10, 20, -15),
                elbow_r=(-100, 0, 0),
                arm_l=(-35, -20, 0),
                elbow_l=(-75, 0, 0),
            ),
            STOP,
        ),
        (0.39, pose(move=(0, -0.06, -0.05), motion=(12, 0, -8), head=(6, 0, 6), arm_r=(-14, 16, -6), elbow_r=(-110, 0, 0)), LOADED),
        # The fist drives up through the jaw at the top of its speed, the
        # body still rising and turning under it...
        strike(
            0.48,
            pose(
                move=(0, -0.17, 0.26),
                motion=(-10, 0, 20),
                head=(-14, 0, -14),
                arm_r=(-112, 18, 10),
                elbow_r=(-50, 0, 0),
                arm_l=(16, -28, 0),
                elbow_l=(-40, 0, 0),
            ),
        ),
        # ...carries on up, past the reach and back...
        (
            0.58,
            pose(move=(0, -0.2, 0.40), motion=(-16, 0, 42), head=(-18, 0, -18), arm_r=(-166, 18, 14), elbow_r=(-28, 0, 0), arm_l=(22, -24, 0), elbow_l=(-50, 0, 0)),
            over(0.1, 0.07, 0.14),
        ),
        # ...hangs at the top...
        (0.70, pose(move=(0, -0.17, 0.26), motion=(-12, 0, 34), head=(-14, 0, -14), arm_r=(-140, 16, 12), elbow_r=(-38, 0, 0), arm_l=(18, -22, 0), elbow_l=(-48, 0, 0))),
        # ...and comes down onto both feet, the knees taking it.
        (
            0.88,
            pose(move=(0, 0, -0.05), motion=(12, 0, 10), head=(6, 0, -4), arm_r=(-36, 12, 0), elbow_r=(-50, 0, 0), arm_l=(-16, -18, 0), elbow_l=(-46, 0, 0)),
            LANDING,
        ),
        (1, REST, STOP),
    ],
    "finisherSoulCrush": [
        (0, REST),
        # The hand reaches out for the soul and draws it up round one arc,
        # out across the body and up, never stopping on the way.
        (0.24, pose(motion=(6, 0, -15), arm_l=(-50, -24, -12), elbow_l=(-60, 0, 0), head=(5, 0, 10), arm_r=(-6, 14, 0), elbow_r=(-24, 0, 0))),
        (0.43, pose(motion=(-5, 0, 15), arm_l=(-88, -10, 36), elbow_l=(-30, 0, 0), head=(-12, 0, 10), arm_r=(-12, 22, 0), elbow_r=(-34, 0, 0))),
        # The soul held up, trembling, lifted past and back: the tension
        # before the crush.
        (0.53, pose(motion=(-9, 0, 18), arm_l=(-104, -16, 0), elbow_l=(-44, 0, 0), wrist_l=(0, 0, 20), head=(-15, 0, 10)), over(0.1, 0.06, 0.12)),
        # It is dragged down and crushed, the fist driving on in.
        strike(
            0.60,
            pose(
                move=(0, 0, -0.06),
                motion=(15, 0, -8),
                arm_l=(-62, -12, -10),
                elbow_l=(-105, 0, 0),
                wrist_l=(0, 0, 72),
                head=(16, 0, 8),
                arm_r=(10, 26, 0),
                elbow_r=(-30, 0, 0),
            ),
            stab(0.05, 0.12),
        ),
        (0.73, pose(motion=(8, 0, -18), arm_l=(-45, -28, -18), elbow_l=(-88, 0, 0), wrist_l=(0, 0, 60), head=(10, 0, 4), arm_r=(4, 18, 0), elbow_r=(-26, 0, 0))),
        (0.87, pose(motion=(3, 0, -6), arm_l=(-18, -16, -6), elbow_l=(-46, 0, 0), wrist_l=(0, 0, 20), head=(4, 0, 2), arm_r=(0, 10, 0), elbow_r=(-16, 0, 0)), PASS),
        (1, REST, STOP),
    ],
}


# What a class's weapon changes in the shared keys, by class and clip: for
# each key time, the joints it poses differently (over the shared pose), and
# optionally how they pass the key. The Hollow's chain whip hangs from its
# fist and trails out, where a blade stands up from it: at the turns that
# set a blade's edge level it would stand over the Hollow's head, pointing
# back. Its keys lay the lash out toward the target at each contact.
CLASS_KEYS = {
    # The Harvester's hat is a brim as wide as its shoulders. A blade held
    # at the head would run its shaft through it, so it thrusts and blocks
    # from lower: the shaft crosses in front of the chest on the thrust and
    # in front of the face, under the brim, on the block.
    "harvester": {
        "thrust": {
            0.45: (dict(arm_r=(-30, 0, 10), elbow_r=(-28, 0, 0), wrist_r=(130, 0, 25)), {"elbow_r": stab(0.05, 0.1, 0.2)}),
        },
        "parry": {
            0.32: dict(arm_r=(-24, 30, -22), elbow_r=(-75, 0, 0), wrist_r=(55, 15, 52)),
            0.43: dict(arm_r=(-20, 22, -30), elbow_r=(-88, 0, 0), wrist_r=(50, 5, 42)),
        },
    },
    "hollow": {
        # The lash sweeps out level and forward across the body.
        "slash": {
            0.45: dict(wrist_r=(-40, -8, -70)),
            0.57: dict(wrist_r=(-44, -24, -88)),
            0.80: dict(wrist_r=(-16, -16, -30)),
        },
        # A crack: the lash is cocked up over the shoulder, thrown forward
        # through the level at the contact, and carries on down, where it
        # stays until the Hollow coils it home.
        "thrust": {
            0.28: dict(wrist_r=(-90, 0, 0)),
            0.36: dict(wrist_r=(-100, 0, 0)),
            0.45: (dict(wrist_r=(10, 0, 0)), {"wrist_r": stab(0.05, 0.1), "elbow_r": stab(0.05, 0.1, 0.25)}),
            0.70: dict(wrist_r=(60, 0, 0)),
        },
    },
}


def for_class(keys, calling, name):
    """`keys`, a clip's, as the class `calling` performs them
    (`CLASS_KEYS`)."""

    changes = CLASS_KEYS.get(calling, {}).get(name)

    if not changes:
        return keys

    out = []

    for key in keys:
        change = changes.get(key[0])

        if change is None:
            out.append(key)
            continue

        poses, manners = change if isinstance(change, tuple) else (change, {})
        shown = type(key[1])({**key[1], **poses})
        manner = key[2] if len(key) > 2 else AUTO

        if manners:
            manner = {**(manner if isinstance(manner, dict) else {"*": manner}), **manners}

        out.append(Strike((key[0], shown, manner)) if isinstance(key, Strike) else (key[0], shown, manner))

    return out
