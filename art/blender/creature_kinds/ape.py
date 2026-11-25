"""The ape: Degenerate Ape Academy.

A heavy knuckle-walker cut in big planes: a wedge of a back that climbs from
the rump to a peak over the shoulders, a flat yoke of trapezius from there
to the crest of the head, a boxy barrel of a chest, and forearms that flare
from the elbow to the knuckles they walk on. The head hangs low and forward
of the shoulders with a heavy brow and a muzzle that juts past it, so the
face shows in profile. The lips meet at rest; the jaw drops wide when it
shrieks, clear of the chest, and shows the fangs.

It wears the academy's mortarboard or a royal crown.
"""

import math

from mathutils import Matrix, Vector

from lib import core, creature

# The joints, in the creature's own space.
HEAD = (0, -0.08, 0.3)
JAW = (0, -0.37, 0.12)
SHOULDER = (0.3, -0.05, 0.16)
HIP = (0.19, 0.4, -0.14)

BODY_VOXEL = 0.025
SKULL_VOXEL = 0.017
# The face and everything on it sit this far forward and down of where the
# brow is: the muzzle juts past the brow and below it.
MUZZLE = Vector((0, -0.045, -0.03))


def build(spec):
    root = core.empty("husk")
    body = core.material("body", color=(1.0, 1.0, 1.0), roughness=0.88, pbr="fur")
    trim = core.material("trim", color=(1.0, 1.0, 1.0), roughness=0.62, pbr="skin")
    detail = core.material("detail", color=(1.0, 1.0, 1.0), roughness=0.40, metallic=0.10, pbr="bone")
    floor = creature.floor(spec)

    # The barrel, rump to chest: each section level, boxy enough that its
    # top, flanks and belly are planes. The top climbs from the rump to a
    # peak over the shoulders (the back seal sits on the slope) and stays
    # broad and flat across the shoulders, where the shoulder seals sit.
    torso = creature.shell(
        "torso",
        [
            ((0, 0.6, 0.0), 0.07, 0.05, 0.05, 2.2),
            ((0, 0.55, 0.02), 0.18, 0.12, 0.14, 2.6),
            ((0, 0.45, 0.05), 0.24, 0.19, 0.21, 3.0),
            ((0, 0.28, 0.1), 0.28, 0.255, 0.29, 3.2),
            ((0, 0.1, 0.12), 0.35, 0.35, 0.32, 3.2),
            ((0, -0.04, 0.1), 0.39, 0.36, 0.29, 3.0),
            ((0, -0.18, 0.1), 0.33, 0.25, 0.28, 3.0),
            ((0, -0.28, 0.1), 0.22, 0.12, 0.22, 2.8),
        ],
        body,
        root,
        sides=20,
        start=0.03,
        end=0.04,
        crease=40,
        forward=(0, 0, 1),
        axis=(0, -1, 0),
    )

    # The bare chest: a plate on the barrel's front, flat across, two
    # planes of pectoral over a belly that falls back between the arms.
    chest = creature.shell(
        "chest",
        [
            ((0, -0.2, -0.24), 0.07, 0.05, 0.04, 2.2),
            ((0, -0.26, -0.15), 0.19, 0.08, 0.06, 2.8),
            ((0, -0.3, -0.02), 0.24, 0.05, 0.08, 3.4),
            ((0, -0.29, 0.1), 0.25, 0.045, 0.08, 3.4),
            ((0, -0.25, 0.19), 0.18, 0.04, 0.07, 3.0),
        ],
        trim,
        root,
        sides=20,
        start=0.02,
        end=0.03,
        crease=40,
    )

    head, skull = _head(root, body, trim, detail)

    for side, name in ((1, "l"), (-1, "r")):
        _arm(root, side, name, body, trim, floor)
        _leg(root, side, name, body, trim, floor)

    # The seals' sockets: the brow's on the brow ridge, the chest's an
    # academy medallion on the plate, the shoulders' and back's on the yoke.
    creature.seat("seat_0", spec, 0, detail, head, [skull], radius=0.05, rise=0.012)
    creature.seat("seat_1", spec, 1, detail, root, [chest], radius=0.066, rise=0.016, lobes=10, scallop=0.1)

    for index in (2, 3, 4):
        creature.seat(f"seat_{index}", spec, index, detail, root, [torso], radius=0.06, rise=0.012)

    creature.studs(root, spec)
    creature.shadow(root, spec)

    return root


def _hang(blobs, joint):
    """Blobs drawn where they sit on the creature, moved into `joint`'s space."""

    return [(kind, _from(joint, at), *rest) for kind, at, *rest in blobs]


def _from(joint, at):
    return tuple(value - origin for value, origin in zip(at, joint))


def _mirror(joint, side):
    return (joint[0] * side, joint[1], joint[2])


def _ahead(at):
    """A point on the face, moved forward and down with the muzzle."""

    return tuple(Vector(at) + MUZZLE)


def _head(root, body, trim, detail):
    dark = core.material("dark", color=(1.0, 1.0, 1.0), roughness=0.28)
    accent = core.material("accent", color=(1.0, 1.0, 1.0), roughness=0.22)
    head = creature.pivot("head", root, HEAD)

    # The cranium: a sagittal crest along the crown that peaks the head,
    # flared cheeks, and the heavy brow bar the brow seal sits on.
    skull = creature.facet(
        "skull",
        _hang(
            [
                ("sphere", (0, -0.13, 0.37), (0.205, 0.165, 0.152)),
                ("sphere", (0, -0.09, 0.46), (0.125, 0.148, 0.088)),
                ("sphere", (0, -0.16, 0.25), (0.185, 0.14, 0.098)),
                ("sphere", (0.155, -0.2, 0.27), (0.07, 0.08, 0.064)),
                ("sphere", (-0.155, -0.2, 0.27), (0.07, 0.08, 0.064)),
                ("sphere", (0, -0.236, 0.388), (0.178, 0.062, 0.038)),
                ("sphere", (0.095, -0.244, 0.394), (0.092, 0.056, 0.042)),
                ("sphere", (-0.095, -0.244, 0.394), (0.092, 0.056, 0.042)),
                ("cube", (0, -0.07, 0.455), (0.058, 0.13, 0.058), (0, math.radians(45), 0)),
            ],
            HEAD,
        ),
        body,
        head,
        triangles=950,
        voxel=SKULL_VOXEL,
        flat=11.0,
    )

    # The bare face: an orbital mask under the brow, and a muzzle that juts
    # forward and down past it: a broad nose and a heavy upper lip over
    # jowls.
    creature.facet(
        "face",
        _hang(
            [
                ("sphere", (0, -0.22, 0.33), (0.162, 0.074, 0.066)),
                ("sphere", (0, -0.26, 0.28), (0.15, 0.1, 0.06)),
                ("sphere", _ahead((0, -0.3, 0.278)), (0.162, 0.128, 0.056)),
                ("sphere", _ahead((0, -0.365, 0.308)), (0.06, 0.054, 0.032)),
                ("sphere", _ahead((0.05, -0.398, 0.296)), (0.03, 0.03, 0.025)),
                ("sphere", _ahead((-0.05, -0.398, 0.296)), (0.03, 0.03, 0.025)),
                ("sphere", _ahead((0, -0.4, 0.244)), (0.122, 0.05, 0.032)),
                ("sphere", _ahead((0.1, -0.32, 0.245)), (0.05, 0.07, 0.058)),
                ("sphere", _ahead((-0.1, -0.32, 0.245)), (0.05, 0.07, 0.058)),
            ],
            HEAD,
        ),
        trim,
        head,
        triangles=640,
        voxel=0.013,
        flat=10.0,
    )

    for side in (1, -1):
        creature.facet(f"socket_{side}", _hang([("sphere", (side * 0.08, -0.282, 0.342), (0.054, 0.024, 0.038))], HEAD), dark, head, triangles=80)
        creature.eye(side, head, _from(HEAD, (side * 0.08, -0.3, 0.342)), (0.019, 0.010, 0.019), accent, triangles=48)
        # Bare skin over the sockets, out to either side of the brow seal,
        # so the brows print light on the dark of the brow ridge.
        creature.brow(side, head, _from(HEAD, (side * 0.112, -0.296, 0.402)), (0.056, 0.022, 0.019), trim, triangles=90)
        creature.facet(f"nostril_{side}", _hang([("sphere", _ahead((side * 0.036, -0.42, 0.293)), (0.019, 0.014, 0.013))], HEAD), dark, head, triangles=40)
        creature.facet(
            f"ear_{side}",
            _hang(
                [
                    ("sphere", (side * 0.208, -0.12, 0.352), (0.032, 0.034, 0.054)),
                    ("sphere", (side * 0.218, -0.105, 0.365), (0.018, 0.022, 0.032)),
                ],
                HEAD,
            ),
            trim,
            head,
            triangles=90,
            voxel=0.008,
        )

    # The inside of the mouth, behind the lips: it only shows when the jaw
    # drops.
    creature.facet("mouth", _hang([("sphere", _ahead((0, -0.31, 0.205)), (0.11, 0.1, 0.05))], HEAD), dark, head, triangles=110)

    # The upper teeth: four incisors between two long canines, all behind
    # the closed lips at rest.
    upper_teeth = []

    for index, (angle, length, width) in enumerate(((-34, 0.046, 0.016), (-15, 0.026, 0.016), (0, 0.027, 0.017), (15, 0.026, 0.016), (34, 0.046, 0.016))):
        turn = math.radians(angle)
        x, y = math.sin(turn) * 0.108, -0.292 - math.cos(turn) * 0.108
        drop = 0.007 if abs(angle) > 25 else 0.0
        upper_teeth.append(
            core.tube(
                f"tooth_u_{index}",
                [_from(HEAD, _ahead(point)) for point in ((x, y, 0.236), (x, y - 0.003, 0.236 - length * 0.55), (x * 0.98, y + 0.002, 0.236 - length - drop))],
                lambda t, w=width, fang=abs(angle) > 25: w * ((1.0 - 0.82 * t) if fang else (0.95 - 0.35 * t)),
                detail,
                sides=8,
                flatten=0.72,
            )
        )

    teeth = core.join(upper_teeth, "teeth_upper")
    teeth.parent = head
    creature.finish(core.shade_smooth(teeth, angle=45))

    jaw = creature.pivot("jaw", head, _from(HEAD, JAW))

    # The lower jaw: a heavy chin whose lip meets the upper one at rest.
    # It hinges low under the chin, so it drops forward and down like a
    # drawer and, open all the way, swings clear in front of the chest and
    # its medallion rather than back into them.
    creature.facet(
        "jaw_lower",
        _hang(
            [
                ("sphere", _ahead((0, -0.35, 0.162)), (0.118, 0.075, 0.044)),
                ("sphere", _ahead((0, -0.4, 0.19)), (0.112, 0.05, 0.03)),
                ("sphere", _ahead((0, -0.378, 0.148)), (0.092, 0.058, 0.028)),
            ],
            JAW,
        ),
        trim,
        jaw,
        triangles=320,
        voxel=0.012,
        flat=10.0,
    )

    creature.facet("mouth_floor", _hang([("sphere", _ahead((0, -0.31, 0.198)), (0.095, 0.075, 0.03))], JAW), dark, jaw, triangles=90)

    lower_teeth = []

    for index, (angle, length, width) in enumerate(((-30, 0.034, 0.014), (-11, 0.018, 0.012), (11, 0.018, 0.012), (30, 0.034, 0.014))):
        turn = math.radians(angle)
        x, y = math.sin(turn) * 0.1, -0.3 - math.cos(turn) * 0.095
        lower_teeth.append(
            core.tube(
                f"tooth_l_{index}",
                [_from(JAW, _ahead(point)) for point in ((x, y, 0.2), (x, y - 0.002, 0.2 + length * 0.55), (x * 0.96, y + 0.003, 0.2 + length))],
                lambda t, w=width, fang=abs(angle) > 20: w * ((1.0 - 0.80 * t) if fang else (0.95 - 0.35 * t)),
                detail,
                sides=8,
                flatten=0.72,
            )
        )

    teeth = core.join(lower_teeth, "teeth_lower")
    teeth.parent = jaw
    creature.finish(core.shade_smooth(teeth, angle=45))

    _mortarboard(creature.pivot("trait_1", head, (0, 0, 0)), detail, trim)
    _crown(creature.pivot("trait_2", head, (0, 0, 0)), detail)

    return head, skull


def _arm(root, side, name, body, trim, floor):
    joint = _mirror(SHOULDER, side)
    arm = creature.pivot(f"leg_f{name}", root, joint)

    def at(x, y, z):
        return Vector((side * x, y, z)) - Vector(joint)

    # Shoulder to wrist: a deltoid cap, an upper arm that narrows to the
    # elbow, and a forearm that flares from there to the wrist, each a box
    # of four planes turned to face forward.
    creature.shell(
        f"forelimb_{name}",
        [
            (at(0.33, -0.05, 0.25), 0.09, 0.09, 0.09, 2.4),
            (at(0.35, -0.055, 0.17), 0.15, 0.14, 0.14, 3.0),
            (at(0.37, -0.08, 0.03), 0.13, 0.12, 0.12, 3.2),
            (at(0.38, -0.12, -0.12), 0.1, 0.095, 0.1, 3.2),
            (at(0.385, -0.17, -0.26), 0.12, 0.115, 0.11, 3.4),
            (at(0.38, -0.22, -0.38), 0.15, 0.135, 0.125, 3.6),
            (at(0.375, -0.245, -0.43), 0.15, 0.13, 0.12, 3.4),
        ],
        body,
        arm,
        sides=16,
        start=0.03,
        end=0.0,
        crease=40,
    )

    # The fist it walks on: a block of a hand folded onto its knuckles,
    # with one heavy bar of knuckles across its front and a thumb.
    creature.shell(
        f"fist_{name}",
        [
            (at(0.375, -0.255, -0.41), 0.13, 0.1, 0.1, 3.0),
            (at(0.375, -0.28, -0.47), 0.15, 0.13, 0.11, 4.0),
            (at(0.375, -0.29, floor + 0.04), 0.15, 0.13, 0.1, 4.0),
            (at(0.375, -0.29, floor + 0.005), 0.13, 0.11, 0.09, 3.0),
        ],
        trim,
        arm,
        sides=16,
        start=0.0,
        end=0.0,
        crease=40,
    )
    creature.facet(
        f"thumb_{name}",
        _hang([("sphere", (side * 0.255, -0.31, floor + 0.05), (0.04, 0.06, 0.04), (0, 0, math.radians(side * 18)))], joint),
        trim,
        arm,
        triangles=60,
        voxel=0.012,
    )


def _leg(root, side, name, body, trim, floor):
    joint = _mirror(HIP, side)
    leg = creature.pivot(f"leg_b{name}", root, joint)

    creature.facet(
        f"haunch_{name}",
        _hang(
            [
                ("sphere", (side * 0.25, 0.38, -0.25), (0.132, 0.160, 0.182)),
                ("sphere", (side * 0.308, 0.38, -0.39), (0.088, 0.094, 0.122), (0, math.radians(side * -14), 0)),
            ],
            joint,
        ),
        body,
        leg,
        triangles=440,
        voxel=BODY_VOXEL,
        flat=13.0,
    )

    # Sculpted bare ape foot with distinct hallux (big toe) and toe pads flat on floor.
    creature.facet(
        f"foot_{name}",
        _hang(
            [
                ("sphere", (side * 0.29, 0.30, floor + 0.040), (0.086, 0.152, 0.040)),
                ("sphere", (side * 0.30, 0.39, floor + 0.062), (0.086, 0.088, 0.062)),
                # Opposable big toe angled inward.
                ("sphere", (side * 0.215, 0.23, floor + 0.032), (0.034, 0.062, 0.032), (0, 0, math.radians(side * -30))),
                ("sphere", (side * 0.29, 0.165, floor + 0.030), (0.064, 0.042, 0.030)),
            ],
            joint,
        ),
        trim,
        leg,
        triangles=220,
        voxel=0.011,
        flat=12.0,
    )


def _mortarboard(parent, cloth, tassel):
    """The academy's mortarboard, worn cocked: a skullcap over the crest
    and a thick square board on it, tipped down to the left and a little
    to the front, so from the front it is a slab on a slant and not a bar,
    and a heavy tassel hanging from its low front corner beside the brow."""

    skullcap = core.revolve(
        "cap",
        [(0.0, 0.565), (0.13, 0.552), (0.195, 0.51), (0.214, 0.462), (0.206, 0.436)],
        cloth,
        parent,
        segments=24,
        cap_top=True,
        cap_bottom=True,
    )
    skullcap.location = _from(HEAD, (0, -0.1, 0))
    creature.finish(core.shade_smooth(skullcap, angle=40))

    turn = Matrix.Rotation(math.radians(7), 3, "X") @ Matrix.Rotation(math.radians(15), 3, "Y") @ Matrix.Rotation(math.radians(6), 3, "Z")
    centre = Vector((0.01, -0.1, 0.594))
    half = 0.26

    def on_board(x, y, z):
        return centre + turn @ Vector((x, y, z))

    rings = [[tuple(Vector(_from(HEAD, tuple(on_board(x * half, y * half, z))))) for x, y in ((-1, -1), (1, -1), (1, 1), (-1, 1))] for z in (0.026, -0.026)]
    board = core.loft("cap_board", rings, cloth, parent, cap_start=True, cap_end=True)
    creature.finish(core.crisp(board, 0.007, segments=1))

    button = core.revolve("cap_button", [(0.0, 0.034), (0.03, 0.026), (0.035, 0.0)], tassel, parent, segments=10, cap_bottom=True)
    button.data.transform(turn.to_4x4())
    button.data.transform(Matrix.Translation(_from(HEAD, tuple(on_board(0, 0, 0.018)))))
    creature.finish(core.shade_smooth(button))

    # The cord runs from the button to the board's low front corner and
    # drops past it; the tassel dangles from it out past the ear, so the
    # board's slant ends in a bob in the outline.
    top = on_board(0, 0, 0.04)
    corner = on_board(half * 0.94, -half * 0.94, 0.028)
    drop = corner + Vector((0.006, -0.004, -0.026))
    cord = core.tube("cap_cord", [_from(HEAD, tuple(point)) for point in (top, top.lerp(corner, 0.5) + Vector((0, 0, 0.014)), corner, drop)], 0.011, tassel, parent, sides=6)
    creature.finish(cord)
    tuft = core.revolve(
        "cap_tassel",
        [(0.0, 0.0), (0.022, -0.01), (0.036, -0.04), (0.042, -0.088), (0.034, -0.104), (0.0, -0.108)],
        tassel,
        parent,
        segments=8,
        cap_bottom=True,
    )
    tuft.location = _from(HEAD, tuple(drop))
    creature.finish(core.shade_smooth(tuft, angle=40))


def _crown(parent, gold):
    """Thickened flared royal crown with sculpted spires and embossed band."""

    points = 7
    steps = points * 4

    def ring(across, deep, height, spike=0.0):
        loop = []
        for step in range(steps):
            angle = step / steps * math.tau
            tooth = abs(math.sin(angle * points / 2)) ** 1.4
            at = (math.cos(angle) * across, -0.10 + math.sin(angle) * deep, height + spike * tooth)
            loop.append(_from(HEAD, at))
        return loop

    crown = core.loft(
        "crown",
        [
            ring(0.150, 0.140, 0.440),
            ring(0.172, 0.162, 0.540, spike=0.092),
            ring(0.156, 0.146, 0.536, spike=0.088),
            ring(0.136, 0.126, 0.440),
            ring(0.150, 0.140, 0.440),
        ],
        gold,
        parent,
        closed=True,
    )
    creature.finish(core.shade_smooth(crown, angle=42))

    band = core.loft(
        "crown_band",
        [
            ring(0.152, 0.142, 0.466),
            ring(0.168, 0.158, 0.478),
            ring(0.170, 0.160, 0.502),
            ring(0.158, 0.148, 0.514),
            ring(0.152, 0.142, 0.466),
        ],
        gold,
        parent,
        closed=True,
    )
    creature.finish(core.shade_smooth(band, angle=45))
