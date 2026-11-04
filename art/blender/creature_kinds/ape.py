"""The ape: Degenerate Ape Academy.

A heavy knuckle-walker. Everything about it is carried at the front: a hump
of shoulder above a deep chest, long thick arms planted on the knuckles, and
a wide head hung forward and low between them with no neck to speak of. The
back falls away to a small rump on short bowed legs.

The arms are its front legs. They swing from the shoulders in diagonal pairs
with the back legs, so it lopes, and charges when it is startled.

Its face, chest, hands and feet are bare, in the second colour. The mouth
hangs open and loose, showing a row of big teeth.

It wears a baseball cap or a crown.
"""

import math

from lib import core, creature

# The joints, in the creature's own space. Every part is drawn where it sits
# on the creature and then hung from one of these.
HEAD = (0, -0.08, 0.3)
JAW = (0, -0.33, 0.145)
SHOULDER = (0.3, -0.05, 0.16)
HIP = (0.19, 0.4, -0.14)

# The voxel sizes for the big plain forms. At these sizes a form comes out of
# the remesh already near its triangle budget, as an even grid. Fused finer
# and then cut down hard, the same form has ragged triangles, and the wear
# that is painted from them mottles a broad back like a walnut.
BODY_VOXEL = 0.045
SKULL_VOXEL = 0.032


def build(spec):
    root = core.empty("husk")
    body = core.material("body")
    trim = core.material("trim")
    floor = creature.floor(spec)

    # Ribs and shoulders make one mass at the front, held high. Behind them
    # the back is a single long slope, leaning down from the hump to a rump
    # much narrower than the shoulders.
    creature.form(
        "torso",
        [
            ("sphere", (0, -0.05, 0.08), (0.33, 0.29, 0.3)),
            ("sphere", (0, 0.03, 0.33), (0.32, 0.26, 0.22)),
            ("sphere", (0, 0.2, 0.1), (0.28, 0.225, 0.42), (math.radians(35), 0, 0)),
            ("sphere", (0, 0.41, -0.11), (0.22, 0.19, 0.18)),
        ],
        body,
        root,
        triangles=3400,
        voxel=BODY_VOXEL,
    )

    # The bare chest, set into the front of the ribs and sagging to a belly.
    creature.form("chest", [("sphere", (0, -0.1, 0.04), (0.26, 0.27, 0.24))], trim, root, triangles=600)

    _head(root, body, trim)

    for side, name in ((1, "l"), (-1, "r")):
        _arm(root, side, name, body, trim, floor)
        _leg(root, side, name, body, trim, floor)

    creature.studs(root, spec)
    creature.shadow(root, spec)

    return root


def _hang(blobs, joint):
    """Blobs drawn where they sit on the creature, moved into the space of
    the pivot at `joint`, which is where a part's meshes have to be."""

    return [(kind, _from(joint, at), *rest) for kind, at, *rest in blobs]


def _from(joint, at):
    """A place on the creature, as seen from the pivot at `joint`."""

    return tuple(value - origin for value, origin in zip(at, joint))


def _mirror(joint, side):
    return (joint[0] * side, joint[1], joint[2])


def _head(root, body, trim):
    detail = core.material("detail")
    dark = core.material("dark")
    accent = core.material("accent")
    head = creature.pivot("head", root, HEAD)

    # A wide skull with a crest, cheeks that run down to the jaw, and one
    # heavy ridge of brow pulled forward over the eyes, thickest above each.
    creature.form(
        "skull",
        _hang(
            [
                ("sphere", (0, -0.13, 0.37), (0.21, 0.165, 0.155)),
                ("sphere", (0, -0.09, 0.455), (0.13, 0.15, 0.09)),
                ("sphere", (0, -0.15, 0.26), (0.185, 0.13, 0.1)),
                ("sphere", (0, -0.245, 0.388), (0.175, 0.062, 0.038)),
                ("sphere", (0.095, -0.248, 0.394), (0.09, 0.056, 0.044)),
                ("sphere", (-0.095, -0.248, 0.394), (0.09, 0.056, 0.044)),
            ],
            HEAD,
        ),
        body,
        head,
        triangles=1600,
        voxel=SKULL_VOXEL,
    )

    # The bare face: a mask around the eyes, a broad muzzle that pushes out
    # past the chest, a thick upper lip, and jowls that hang at the corners
    # of the mouth so the dark inside does not bulge out at the sides.
    creature.form(
        "face",
        _hang(
            [
                ("sphere", (0, -0.22, 0.33), (0.165, 0.075, 0.065)),
                ("sphere", (0, -0.3, 0.275), (0.165, 0.13, 0.055)),
                ("sphere", (0, -0.405, 0.245), (0.15, 0.035, 0.027)),
                ("sphere", (0.122, -0.29, 0.235), (0.04, 0.08, 0.07)),
                ("sphere", (-0.122, -0.29, 0.235), (0.04, 0.08, 0.07)),
            ],
            HEAD,
        ),
        trim,
        head,
        triangles=900,
    )

    for side in (1, -1):
        # What is left of its eyes: dark sockets with a light far back in them.
        creature.form(f"socket_{side}", _hang([("sphere", (side * 0.08, -0.286, 0.342), (0.052, 0.022, 0.036))], HEAD), dark, head, triangles=80)
        creature.eye(side, head, _from(HEAD, (side * 0.08, -0.304, 0.342)), (0.02, 0.01, 0.02), accent, triangles=48)
        # A pad of fur over each eye, in front of the ridge of the brow, so
        # the face can knit and lift.
        creature.brow(side, head, _from(HEAD, (side * 0.083, -0.3, 0.392)), (0.058, 0.02, 0.017), body, triangles=90)
        creature.form(f"nostril_{side}", _hang([("sphere", (side * 0.034, -0.414, 0.296), (0.019, 0.014, 0.013))], HEAD), dark, head, triangles=40)
        creature.form(f"ear_{side}", _hang([("sphere", (side * 0.21, -0.12, 0.35), (0.035, 0.035, 0.055))], HEAD), trim, head, triangles=64)

    # The roof of the mouth: a dark ball behind the lips, which is what
    # shows where the jaw hangs.
    creature.form("mouth", _hang([("sphere", (0, -0.3, 0.19), (0.125, 0.115, 0.055))], HEAD), dark, head, triangles=110)

    # A row of slab teeth behind the upper lip, following its curve. Four
    # big ones read at a distance; a full set would be a pale smear.
    creature.form(
        "teeth_upper",
        _hang(
            [("cube", (side * 0.02, -0.402, 0.21), (0.019, 0.016, 0.025)) for side in (1, -1)]
            + [("cube", (side * 0.06, -0.392, 0.21), (0.019, 0.016, 0.025)) for side in (1, -1)],
            HEAD,
        ),
        detail,
        head,
        triangles=240,
    )

    # The lower jaw hangs a little open at rest, with a lip that rolls out
    # and a peg of a tooth at each corner. The game swings it a long way when
    # it shrieks, so it is short and hinged well forward, level with the lip:
    # the lip then drops straight down, and stops short of the seal on the
    # chest rather than swinging back into it.
    jaw = creature.pivot("jaw", head, _from(HEAD, JAW))

    creature.form(
        "jaw_lower",
        _hang(
            [
                ("sphere", (0, -0.33, 0.115), (0.16, 0.1, 0.042)),
                ("sphere", (0, -0.405, 0.133), (0.15, 0.04, 0.027)),
            ],
            JAW,
        ),
        trim,
        jaw,
        triangles=400,
    )

    # The floor of the mouth. At rest it is lost inside the roof; when the
    # jaw drops it comes down with it, so the open mouth is dark all the way
    # and the chest does not show through the gap.
    creature.form("mouth_floor", _hang([("sphere", (0, -0.34, 0.165), (0.125, 0.08, 0.06))], JAW), dark, jaw, triangles=90)
    creature.form(
        "teeth_lower",
        _hang([("cube", (side * 0.097, -0.385, 0.172), (0.016, 0.015, 0.023)) for side in (1, -1)], JAW),
        detail,
        jaw,
        triangles=120,
    )

    _cap(creature.pivot("trait_1", head, (0, 0, 0)), detail, trim)
    _crown(creature.pivot("trait_2", head, (0, 0, 0)), detail)


def _arm(root, side, name, body, trim, floor):
    joint = _mirror(SHOULDER, side)
    arm = creature.pivot(f"leg_f{name}", root, joint)

    # The shoulder is a ball about the joint, so it stays where it is, under
    # its seal, when the arm swings. The forearm is as thick as the upper arm
    # and leans forward to put the fist ahead of the chest.
    #
    # The mesh cannot be called arm_<side>: that is the name of a pivot the
    # game swings against the legs, and these arms are legs.
    creature.form(
        f"forelimb_{name}",
        _hang(
            [
                ("sphere", joint, (0.155, 0.165, 0.155)),
                ("sphere", (side * 0.355, -0.08, -0.04), (0.115, 0.125, 0.2)),
                ("sphere", (side * 0.37, -0.2, -0.3), (0.115, 0.13, 0.22), (math.radians(-18), 0, 0)),
            ],
            joint,
        ),
        body,
        arm,
        triangles=1200,
        voxel=BODY_VOXEL,
    )

    # A big bare fist, standing on the knuckles: the back of the hand, three
    # curled fingers in front of it and the thumb tucked inside.
    creature.form(
        f"fist_{name}",
        _hang(
            [
                ("sphere", (side * 0.36, -0.29, floor + 0.075), (0.095, 0.11, 0.08)),
                ("sphere", (side * 0.28, -0.32, floor + 0.07), (0.04, 0.06, 0.04)),
            ]
            + [("sphere", (side * 0.36 + across, -0.365, floor + 0.056), (0.038, 0.065, 0.058)) for across in (-0.058, 0, 0.058)],
            joint,
        ),
        trim,
        arm,
        triangles=400,
    )


def _leg(root, side, name, body, trim, floor):
    joint = _mirror(HIP, side)
    leg = creature.pivot(f"leg_b{name}", root, joint)

    # Short and bowed: the knee is pushed out and the shin comes back in.
    creature.form(
        f"haunch_{name}",
        _hang(
            [
                ("sphere", (side * 0.25, 0.38, -0.25), (0.13, 0.16, 0.18)),
                ("sphere", (side * 0.31, 0.38, -0.39), (0.085, 0.09, 0.12), (0, math.radians(side * -14), 0)),
            ],
            joint,
        ),
        body,
        leg,
        triangles=640,
        voxel=BODY_VOXEL,
    )

    # A long flat foot, as bare as the hands, with a heel that takes the
    # whole end of the shin so the leg does not finish in a stump.
    creature.form(
        f"foot_{name}",
        _hang(
            [
                ("sphere", (side * 0.29, 0.29, floor + 0.04), (0.09, 0.16, 0.04)),
                ("sphere", (side * 0.3, 0.39, floor + 0.062), (0.088, 0.09, 0.062)),
            ],
            joint,
        ),
        trim,
        leg,
        triangles=240,
    )


def _cap(parent, cloth, button):
    # The dome swallows the crest, so the cap sits down on the brow. The
    # brim tips up a little, which keeps it clear of the eyes and of the
    # seal on the brow.
    creature.form("cap",_hang([("sphere", (0, -0.12, 0.47), (0.215, 0.2, 0.11))], HEAD), cloth, parent, triangles=440)
    creature.form(
        "cap_brim",
        _hang([("sphere", (0, -0.36, 0.44), (0.16, 0.15, 0.016), (math.radians(-8), 0, 0))], HEAD),
        cloth,
        parent,
        triangles=200,
    )
    creature.form("cap_button", _hang([("sphere", (0, -0.12, 0.58), (0.032, 0.032, 0.018))], HEAD), button, parent, triangles=40)


def _crown(parent, gold):
    points = 7

    def ring(across, deep, height, spike=0.0):
        """A loop about the top of the skull, every other point lifted by
        `spike`."""

        loop = []

        for step in range(points * 2):
            angle = step / (points * 2) * math.tau
            at = (math.cos(angle) * across, -0.1 + math.sin(angle) * deep, height + spike * (step % 2))

            loop.append(_from(HEAD, at))

        return loop

    # The foot of the crown is sunk into the skull, the crest stands up
    # inside it, and it flares a little toward the points.
    creature.finish(core.loft("crown", [ring(0.15, 0.14, 0.44), ring(0.168, 0.158, 0.54, spike=0.09)], gold, parent, closed=True))
    creature.finish(
        core.loft(
            "crown_band",
            [ring(0.153, 0.143, 0.47), ring(0.164, 0.154, 0.48), ring(0.166, 0.156, 0.5), ring(0.158, 0.148, 0.51)],
            gold,
            parent,
            closed=True,
        )
    )
