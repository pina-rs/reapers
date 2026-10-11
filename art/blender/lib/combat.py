"""Combat key poses with loaded anticipation, decisive contact and recovery.

All impact keys agree with the shared presentation catalogue. A planted lower
body and independently leading pelvis turn these into whole-body actions.
"""

REST = {}
GUARD = dict(arm_r=(-25, 16, -8), arm_l=(-32, -20, 8), elbow_r=(-62, 0, 0), elbow_l=(-85, 0, 0), head=(5, 0, 0))
KEYS = {
    "slash": [
        (0, REST),
        (0.24, dict(motion=(-8, -5, -42), arm_r=(-45, 40, -32), elbow_r=(-90, 0, 0), wrist_r=(0, 30, -20), arm_l=(-30, -24, 0), elbow_l=(-65, 0, 0))),
        (0.34, dict(motion=(-3, -3, -30), arm_r=(-58, 48, -38), elbow_r=(-78, 0, 0), wrist_r=(0, 42, -24), head=(0, 0, 18))),
        (0.45, dict(move=(0, -0.18, -0.04), motion=(12, 5, 34), arm_r=(-75, -8, 42), elbow_r=(-18, 0, 0), wrist_r=(0, -30, 14), arm_l=(18, -35, 0))),
        (0.61, dict(motion=(14, 7, 55), arm_r=(-30, -20, 60), elbow_r=(-28, 0, 0), wrist_r=(10, -48, 28), arm_l=(22, -22, 0))),
        (0.82, GUARD),
        (1, REST),
    ],
    "thrust": [
        (0, REST),
        (
            0.28,
            dict(
                move=(0, 0.09, -0.07),
                motion=(-6, 0, -27),
                arm_r=(-24, 8, -12),
                elbow_r=(-100, 0, 0),
                wrist_r=(112, 0, 0),
                arm_l=(-42, -20, 0),
                elbow_l=(-65, 0, 0),
            ),
        ),
        (0.36, dict(move=(0, 0.02, -0.06), motion=(4, 0, -12), arm_r=(-32, 6, -6), elbow_r=(-88, 0, 0), wrist_r=(115, 0, 0))),
        (0.45, dict(move=(0, -0.32, -0.02), motion=(18, 0, 22), arm_r=(-78, 0, 8), elbow_r=(-10, 0, 0), wrist_r=(172, 0, 0), arm_l=(10, -25, 0))),
        (0.57, dict(move=(0, -0.34, -0.03), motion=(20, 0, 25), arm_r=(-80, 0, 12), elbow_r=(-8, 0, 0), wrist_r=(178, 0, 0))),
        (0.77, GUARD),
        (1, REST),
    ],
    "parry": [
        (0, REST),
        (0.18, GUARD),
        (0.32, dict(motion=(4, 0, -18), arm_r=(-58, 32, -22), elbow_r=(-65, 0, 0), wrist_r=(12, 15, 52), arm_l=(-35, -20, 0), elbow_l=(-80, 0, 0))),
        (0.43, dict(move=(0, 0.10, -0.05), motion=(-8, -4, -28), arm_r=(-52, 24, -30), elbow_r=(-80, 0, 0), wrist_r=(5, 5, 42))),
        (0.58, dict(motion=(6, 0, 8), arm_r=(-62, 40, 4), elbow_r=(-32, 0, 0), wrist_r=(0, -15, 62))),
        (0.78, GUARD),
        (1, REST),
    ],
    "finisherReap": [
        (0, REST),
        (0.27, dict(move=(0, 0.08, -0.07), motion=(-12, -6, -48), arm_r=(-48, 45, -45), elbow_r=(-78, 0, 0), wrist_r=(8, 55, -20), head=(5, 0, 24))),
        (0.41, dict(motion=(-5, -8, -34), arm_r=(-68, 55, -40), elbow_r=(-62, 0, 0), wrist_r=(10, 60, -28))),
        (0.52, dict(move=(0, -0.28, -0.05), motion=(17, 8, 48), arm_r=(-66, -15, 55), elbow_r=(-12, 0, 0), wrist_r=(0, -40, 25), arm_l=(20, -42, 0))),
        (0.66, dict(motion=(18, 12, 76), arm_r=(-25, -25, 80), elbow_r=(-25, 0, 0), wrist_r=(15, -55, 36), head=(-4, 0, -25))),
        (0.84, GUARD),
        (1, REST),
    ],
    "finisherImpale": [
        (0, REST),
        (0.29, dict(move=(0, 0.10, -0.08), motion=(-10, 0, -34), arm_r=(-15, 12, -20), elbow_r=(-105, 0, 0), wrist_r=(108, 0, 0))),
        (0.39, dict(motion=(3, 0, -18), arm_r=(-28, 8, -12), elbow_r=(-90, 0, 0), wrist_r=(115, 0, 0))),
        (0.46, dict(move=(0, -0.40, -0.04), motion=(22, 0, 26), arm_r=(-80, 0, 15), elbow_r=(-12, 0, 0), wrist_r=(175, 0, 0))),
        (0.59, dict(move=(0, -0.42, -0.03), motion=(18, 0, 28), arm_r=(-86, 0, 18), elbow_r=(-16, 0, 0), wrist_r=(180, 0, 0))),
        (0.73, dict(move=(0, -0.25, 0), motion=(-6, 0, 15), arm_r=(-105, 15, 12), elbow_r=(-55, 0, 0), wrist_r=(165, 0, 0))),
        (0.87, GUARD),
        (1, REST),
    ],
    "finisherPalm": [
        (0, REST),
        (
            0.30,
            dict(
                move=(0, 0.08, -0.08),
                motion=(8, 0, -35),
                arm_l=(-12, -18, -24),
                elbow_l=(-112, 0, 0),
                wrist_l=(-18, 0, 0),
                arm_r=(-28, 20, 0),
                elbow_r=(-80, 0, 0),
            ),
        ),
        (0.41, dict(motion=(5, 0, -20), arm_l=(-22, -16, -18), elbow_l=(-98, 0, 0), wrist_l=(-20, 0, 0))),
        (0.50, dict(move=(0, -0.32, -0.02), motion=(16, 0, 28), arm_l=(-85, -8, 15), elbow_l=(-12, 0, 0), wrist_l=(-12, 0, 0), arm_r=(12, 22, 0))),
        (0.62, dict(move=(0, -0.34, -0.02), motion=(18, 0, 33), arm_l=(-88, -6, 18), elbow_l=(-10, 0, 0), wrist_l=(-12, 0, 0))),
        (0.82, GUARD),
        (1, REST),
    ],
    "finisherUppercut": [
        (0, REST),
        (0.29, dict(move=(0, 0, -0.14), motion=(22, 0, -32), arm_r=(10, 20, -15), elbow_r=(-100, 0, 0), arm_l=(-35, -20, 0), elbow_l=(-75, 0, 0))),
        (0.39, dict(move=(0, -0.06, -0.05), motion=(12, 0, -8), arm_r=(-28, 16, -6), elbow_r=(-105, 0, 0))),
        (0.48, dict(move=(0, -0.18, 0.34), motion=(-12, 0, 32), arm_r=(-118, 18, 10), elbow_r=(-65, 0, 0), arm_l=(16, -28, 0))),
        (0.60, dict(move=(0, -0.18, 0.30), motion=(-16, 0, 42), arm_r=(-140, 18, 12), elbow_r=(-38, 0, 0))),
        (0.79, dict(move=(0, 0, -0.06), motion=(14, 0, 12), arm_r=(-70, 14, 0), elbow_r=(-70, 0, 0))),
        (1, REST),
    ],
    "finisherSoulCrush": [
        (0, REST),
        (0.24, dict(motion=(6, 0, -15), arm_l=(-55, -18, -12), elbow_l=(-65, 0, 0), head=(5, 0, 10))),
        (0.43, dict(motion=(-5, 0, 15), arm_l=(-90, -15, 8), elbow_l=(-32, 0, 0), head=(-12, 0, 10))),
        (0.55, dict(motion=(-9, 0, 18), arm_l=(-98, -14, 8), elbow_l=(-38, 0, 0), wrist_l=(0, 0, 20))),
        (0.60, dict(move=(0, 0, -0.06), motion=(15, 0, -8), arm_l=(-62, -12, -10), elbow_l=(-105, 0, 0), wrist_l=(0, 0, 72), head=(16, 0, 8))),
        (0.73, dict(motion=(8, 0, -18), arm_l=(-45, -28, -18), elbow_l=(-88, 0, 0), wrist_l=(0, 0, 60))),
        (1, REST),
    ],
}
