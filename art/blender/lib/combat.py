"""Combat key poses: anticipation, a contact crossed at speed, and a
follow-through that carries on round a loop into the recovery, so the
weapon never stops and swings back the way it came.

Every pose names what it means to move and holds the rest (`curves`): the
off hand keeps its guard through a wind-up instead of dropping to its bind
pose between keys. The head counters the chest's twist, so the eyes stay on
the target while the body winds and unwinds around them.

Each attack's contact is a `strike` key. Its time is the clip's impact in
`ReaperMotion` (`art/build/motions.json`); `motion.build` refuses to bake a
clip whose contact key is anywhere else. The pelvis, chest, head, shoulder
and elbow lead the weapon hand into it (`motion.LEAD`), so the hand, and the
weapon in it, crosses its contact key on the impact.
"""

from .curves import AUTO, PASS, REST, STOP, over, pose, stance


class Strike(tuple):
    """A contact key: `(time, pose, manner)`, at the clip's impact."""


def strike(time, shown, manner=PASS):
    return Strike((time, shown, manner))


def contacts(keys):
    """The times of a clip's contact keys."""

    return [key[0] for key in keys if isinstance(key, Strike)]


GUARD = stance(arm_r=(-25, 16, -8), arm_l=(-32, -20, 8), elbow_r=(-62, 0, 0), elbow_l=(-85, 0, 0), head=(5, 0, 0))
# A landing gives in the body, which sinks past the pose and springs back.
LANDING = {"*": AUTO, "move": over(0.3, 0.05, 0.12), "motion": over(0.2, 0.06, 0.12)}


def trunk(manner):
    """A key manner for the body alone (`move` and `motion`), the limbs
    passing the key as they will: a landing gives in the hips, not in the
    arms, and a turn runs out in the body while the blade sweeps on."""

    return {"*": AUTO, "move": manner, "motion": manner}


# While the arm loads, the chest is already unwinding, the head turning
# against it: only the limbs stop.
LOADED = {"*": STOP, "motion": AUTO, "move": AUTO, "head": AUTO}

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
        strike(
            0.45,
            pose(
                move=(0, -0.18, -0.04),
                motion=(12, 5, 34),
                head=(-2, 0, -16),
                arm_r=(-75, -8, 42),
                elbow_r=(-18, 0, 0),
                wrist_r=(0, -30, 14),
                arm_l=(18, -35, 0),
                elbow_l=(-30, 0, 0),
            ),
        ),
        # Past the contact the blade sweeps on round a loop, never back the
        # way it came: the body's turn carries it on across while the arm
        # lifts it...
        (
            0.55,
            pose(move=(0, -0.21, -0.05), motion=(16, 7, 56), head=(0, 0, -26), arm_r=(-120, -10, 40), elbow_r=(-34, 0, 0), wrist_r=(2, -36, 18), arm_l=(32, -38, 0), elbow_l=(-26, 0, 0)),
            trunk(over(0.08, 0.07, 0.12)),
        ),
        # ...up over the shoulder as the body unwinds under it...
        (0.65, pose(move=(0, -0.14, -0.04), motion=(10, 5, 36), head=(1, 0, -16), arm_r=(-130, -20, 10), elbow_r=(-66, 0, 0), wrist_r=(10, -48, 26), arm_l=(20, -30, 0), elbow_l=(-38, 0, 0))),
        # ...and down to the guard.
        (0.77, pose(move=(0, -0.07, -0.02), motion=(5, 2, 12), head=(2, 0, -6), arm_r=(-100, -20, -20), elbow_r=(-56, 0, 0), wrist_r=(10, -44, 24), arm_l=(2, -20, 0), elbow_l=(-44, 0, 0))),
        (0.89, stance(move=(0, -0.02, -0.01), motion=(2, 1, 4), head=(2, 0, -2), arm_r=(-30, 8, 4), elbow_r=(-40, 0, 0), wrist_r=(2, -6, 4), arm_l=(-10, -10, 2), elbow_l=(-30, 0, 0))),
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
        # A thrust lands at full reach and crosses it at speed: the point
        # drives on and up...
        strike(
            0.45,
            pose(
                move=(0, -0.32, -0.02),
                motion=(18, 0, 22),
                head=(-6, 0, -12),
                arm_r=(-78, 0, 8),
                elbow_r=(-10, 0, 0),
                wrist_r=(172, 0, 0),
                arm_l=(10, -25, 0),
                elbow_l=(-35, 0, 0),
            ),
        ),
        (
            0.55,
            pose(move=(0, -0.35, -0.03), motion=(20, 0, 26), head=(-7, 0, -14), arm_r=(-100, 0, 20), elbow_r=(-12, 0, 0), wrist_r=(176, 0, 0), arm_l=(14, -24, 0), elbow_l=(-40, 0, 0)),
            trunk(over(0.08, 0.05, 0.1)),
        ),
        # ...and draws back round a loop rather than straight down the line
        # it went out on, the blade unwinding over the way home.
        (0.65, pose(move=(0, -0.22, -0.03), motion=(10, 0, 16), head=(-3, 0, -8), arm_r=(-96, 4, 40), elbow_r=(-40, 0, 0), wrist_r=(120, 0, 0), arm_l=(2, -22, 0), elbow_l=(-46, 0, 0))),
        (0.76, pose(move=(0, -0.1, -0.02), motion=(6, 0, 8), head=(1, 0, -3), arm_r=(-66, 8, 34), elbow_r=(-52, 0, 0), wrist_r=(50, 0, 0), arm_l=(-12, -18, 0), elbow_l=(-44, 0, 0))),
        (0.86, stance(move=(0, -0.04, -0.02), motion=(3, 0, 4), head=(3, 0, 0), arm_r=(-30, 8, 10), elbow_r=(-34, 0, 0), wrist_r=(20, 0, 0), arm_l=(-14, -14, 0), elbow_l=(-30, 0, 0))),
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
        (0.41, pose(motion=(-5, -8, -34), head=(3, 0, 20), arm_r=(-68, 55, -40), elbow_r=(-62, 0, 0), wrist_r=(10, 60, -28)), LOADED),
        strike(
            0.52,
            pose(
                move=(0, -0.28, -0.05),
                motion=(17, 8, 48),
                head=(-2, 0, -20),
                arm_r=(-66, -15, 55),
                elbow_r=(-12, 0, 0),
                wrist_r=(0, -40, 25),
                arm_l=(20, -42, 0),
                elbow_l=(-30, 0, 0),
            ),
        ),
        # Like the slash, but wider: the body's turn carries the blade on
        # across, the arm lifts it up over the shoulder and down to guard,
        # one loop with no stop in it.
        (
            0.63,
            pose(move=(0, -0.3, -0.06), motion=(18, 12, 72), head=(-4, 0, -25), arm_r=(-115, -15, 55), elbow_r=(-30, 0, 0), wrist_r=(4, -48, 28), arm_l=(26, -36, 0), elbow_l=(-40, 0, 0)),
            trunk(over(0.1, 0.08, 0.14)),
        ),
        (0.73, pose(move=(0, -0.2, -0.04), motion=(12, 6, 44), head=(0, 0, -16), arm_r=(-130, -22, 22), elbow_r=(-64, 0, 0), wrist_r=(8, -52, 30), arm_l=(18, -28, 0), elbow_l=(-40, 0, 0))),
        (0.84, pose(move=(0, -0.1, -0.02), motion=(8, 4, 26), head=(2, 0, -8), arm_r=(-100, -20, -8), elbow_r=(-56, 0, 0), wrist_r=(8, -34, 20), arm_l=(0, -20, 0), elbow_l=(-44, 0, 0))),
        (0.92, stance(move=(0, -0.05, -0.02), motion=(4, 2, 10), head=(2, 0, -5), arm_r=(-24, 2, 22), elbow_r=(-40, 0, 0), wrist_r=(6, -16, 10), arm_l=(-10, -16, 0), elbow_l=(-44, 0, 0))),
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
        strike(
            0.46,
            pose(
                move=(0, -0.40, -0.04),
                motion=(22, 0, 26),
                head=(-6, 0, -14),
                arm_r=(-80, 0, 15),
                elbow_r=(-12, 0, 0),
                wrist_r=(175, 0, 0),
                arm_l=(12, -28, 0),
                elbow_l=(-40, 0, 0),
            ),
            over(0.07, 0.04, 0.1),
        ),
        (0.59, pose(move=(0, -0.42, -0.03), motion=(18, 0, 28), head=(-5, 0, -15), arm_r=(-86, 0, 18), elbow_r=(-16, 0, 0), wrist_r=(180, 0, 0))),
        # Lift what is on the point, the off hand bracing, and fling it off
        # outward, the blade sweeping round and down to the guard rather
        # than dropping back the way it rose.
        (
            0.68,
            pose(move=(0, -0.25, 0), motion=(-6, 0, 15), head=(-14, 0, -8), arm_r=(-108, 6, 26), elbow_r=(-55, 0, 0), wrist_r=(150, 0, 0), arm_l=(-40, -20, 0), elbow_l=(-72, 0, 0)),
            trunk(over(0.1, 0.08, 0.14)),
        ),
        (0.77, pose(move=(0, -0.18, 0), motion=(-2, 0, 20), head=(-8, 0, -4), arm_r=(-100, 12, 54), elbow_r=(-44, 0, 0), wrist_r=(100, 0, 0), arm_l=(-30, -18, 0), elbow_l=(-60, 0, 0))),
        (0.85, pose(move=(0, -0.1, 0), motion=(2, 0, 10), head=(-2, 0, -2), arm_r=(-64, 10, 40), elbow_r=(-52, 0, 0), wrist_r=(40, 0, 0), arm_l=(-22, -16, 0), elbow_l=(-50, 0, 0))),
        (0.92, stance(move=(0, -0.05, 0), motion=(2, 0, 4), head=(0, 0, -2), arm_r=(-34, 8, 16), elbow_r=(-46, 0, 0), wrist_r=(10, 0, 0), arm_l=(-20, -16, 0), elbow_l=(-44, 0, 0))),
        (1, REST, STOP),
    ],
    "finisherPalm": [
        (0, REST),
        (
            0.30,
            pose(
                move=(0, 0.08, -0.08),
                motion=(8, 0, -35),
                head=(6, 0, 18),
                arm_l=(-12, -18, -24),
                elbow_l=(-112, 0, 0),
                wrist_l=(-18, 0, 0),
                arm_r=(-28, 20, 0),
                elbow_r=(-80, 0, 0),
            ),
            STOP,
        ),
        (0.41, pose(motion=(5, 0, -20), head=(4, 0, 10), arm_l=(-22, -16, -18), elbow_l=(-98, 0, 0), wrist_l=(-20, 0, 0)), LOADED),
        strike(
            0.50,
            pose(
                move=(0, -0.32, -0.02),
                motion=(16, 0, 28),
                head=(-4, 0, -14),
                arm_l=(-85, -8, 15),
                elbow_l=(-12, 0, 0),
                wrist_l=(-12, 0, 0),
                arm_r=(12, 22, 0),
                elbow_r=(-40, 0, 0),
            ),
            over(0.08, 0.04, 0.1),
        ),
        (0.62, pose(move=(0, -0.34, -0.02), motion=(18, 0, 33), head=(-5, 0, -16), arm_l=(-88, -6, 18), elbow_l=(-10, 0, 0), wrist_l=(-12, 0, 0), arm_r=(16, 20, 0), elbow_r=(-46, 0, 0))),
        (0.82, stance(move=(0, -0.06, -0.01), motion=(4, 0, 8), head=(1, 0, -4), arm_l=(-30, -12, 6), elbow_l=(-46, 0, 0), wrist_l=(-4, 0, 0), arm_r=(-8, 16, 0), elbow_r=(-40, 0, 0))),
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
        (0.39, pose(move=(0, -0.06, -0.05), motion=(12, 0, -8), head=(6, 0, 6), arm_r=(-28, 16, -6), elbow_r=(-105, 0, 0)), LOADED),
        strike(
            0.48,
            pose(
                move=(0, -0.18, 0.34),
                motion=(-12, 0, 32),
                head=(-14, 0, -14),
                arm_r=(-118, 18, 10),
                elbow_r=(-65, 0, 0),
                arm_l=(16, -28, 0),
                elbow_l=(-40, 0, 0),
            ),
        ),
        (0.60, pose(move=(0, -0.18, 0.30), motion=(-16, 0, 42), head=(-18, 0, -18), arm_r=(-140, 18, 12), elbow_r=(-38, 0, 0), arm_l=(22, -24, 0), elbow_l=(-50, 0, 0)), over(0.1, 0.07, 0.12)),
        # Down onto both feet, the knees taking it.
        (
            0.79,
            pose(move=(0, 0, -0.06), motion=(14, 0, 12), head=(8, 0, -4), arm_r=(-62, 14, 0), elbow_r=(-62, 0, 0), arm_l=(-18, -18, 0), elbow_l=(-50, 0, 0)),
            LANDING,
        ),
        (1, REST, STOP),
    ],
    "finisherSoulCrush": [
        (0, REST),
        (0.24, pose(motion=(6, 0, -15), arm_l=(-55, -18, -12), elbow_l=(-65, 0, 0), head=(5, 0, 10), arm_r=(-6, 14, 0), elbow_r=(-24, 0, 0))),
        (0.43, pose(motion=(-5, 0, 15), arm_l=(-90, -15, 8), elbow_l=(-32, 0, 0), head=(-12, 0, 10), arm_r=(-12, 22, 0), elbow_r=(-34, 0, 0))),
        # The soul held up, trembling: the tension before the crush.
        (0.55, pose(motion=(-9, 0, 18), arm_l=(-98, -14, 8), elbow_l=(-38, 0, 0), wrist_l=(0, 0, 20), head=(-15, 0, 10)), STOP),
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
            over(0.15, 0.04, 0.1),
        ),
        (0.73, pose(motion=(8, 0, -18), arm_l=(-45, -28, -18), elbow_l=(-88, 0, 0), wrist_l=(0, 0, 60), head=(10, 0, 4), arm_r=(4, 18, 0), elbow_r=(-26, 0, 0))),
        (1, REST, STOP),
    ],
}
