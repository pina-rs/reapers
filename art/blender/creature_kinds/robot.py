"""The robot: Solarians.

A small scrapyard robot put together from parts that were never meant to
meet: a box of a head on an oil drum, a thin piston arm that ends in a
three-fingered claw and a boxy arm that ends in a clamp, one leg that is a
rod standing on a flat plate and one that is a can standing on a drum. The
mismatch is the collection's signature, so nothing here is mirrored. Every
pair of parts is deliberately from a different family.

It is hard-surface, so it is turned, lofted and bevelled rather than fused
from blobs: flat plates with machined edges, round cans, exposed joints.

It wears an antenna or a commander's crest.
"""

import math

import bpy
from mathutils import Euler, Matrix, Vector

from lib import core, creature

# Edges sharper than this stay crisp; anything gentler (a bevel, the side of
# a can) is shaded round.
SHARP = 50


def build(spec):
    root = core.empty("husk")
    floor = creature.floor(spec)

    # The seals, in the order the rules list them. Every surface a stud sits
    # on is measured from its seal, so the studs land on plate, not in it.
    face, chest, shoulder_l, shoulder_r, back = (creature.to_blender(seal["offset"]) for seal in spec["seals"])

    neck, hips = _torso(root, chest, back, (shoulder_l, shoulder_r))

    _head(root, neck, face, spec["halfHeight"])
    _claw_arm(root, shoulder_l)
    _clamp_arm(root, shoulder_r)
    _piston_leg(root, hips, floor)
    _drum_leg(root, hips, floor)

    creature.studs(root, spec)
    creature.shadow(root, spec)

    return root


def _torso(root, chest, back, shoulders):
    """Builds the drum it is made around. Returns where its neck ends and
    the middle of the axle its legs hang from."""

    body, trim, detail, dark, _ = _palette()

    # An oil drum, exactly as round as the chest and back seals are far
    # apart, with its two hoops kept clear of both studs.
    radius = (back.y - chest.y) / 2
    middle = (back.y + chest.y) / 2
    belt = (chest.z + back.z) / 2
    rim, bottom = belt + 0.086, belt - 0.28
    hoops = (belt + 0.059, belt - 0.14)

    # The lid is a shallow cone. The head pitches a long way to watch the
    # player and the jaw drops further still, and a flat lid would be in the
    # way of both.
    lid = rim + 0.043
    hips = Vector((0, middle, bottom - 0.045))

    def on(degrees, height, proud=0.0):
        """A point on the drum's side, or `proud` of it."""

        return Vector((0, middle, height)) + _outward(degrees) * (radius + proud)

    parts = [
        _lathe(
            [
                (0.05, lid),
                (0.085, lid),
                (radius - 0.012, rim + 0.004),
                (radius, rim - 0.006),
                *_hoop(radius, hoops[0], wide=0.02),
                *_hoop(radius, hoops[1]),
                (radius, bottom + 0.012),
                (radius - 0.012, bottom),
                (radius * 0.6, bottom),
            ],
            (0, middle, 0),
            body,
            sides=32,
        ),
        # The under-frame: a block under the drum with an axle through it.
        # The legs hang from the axle outside the block, so they can swing
        # as far as the game swings them without passing through it.
        _box(hips + Vector((0, 0, 0.01)), (0.045, 0.065, 0.045), trim, bevel=0.012),
        _drum(hips, 0.028, 0.25, trim, toward=(1, 0, 0), sides=12),
        # A bellows of a neck: the gap between drum and head is the first
        # exposed joint anyone sees.
        _lathe(
            [(0.07, lid - 0.004), (0.054, lid + 0.008), (0.07, lid + 0.02), (0.054, lid + 0.032), (0.07, lid + 0.044)],
            (0, middle + 0.035, 0),
            trim,
            sides=16,
            caps=False,
        ),
        # A patch riveted over a hole on its right breast.
        _place(_band(radius, 203, 250, belt - 0.085, belt + 0.035, 0.008, detail), (0, middle, 0)),
        # A pressure gauge on its left breast.
        _bezel(on(306, belt - 0.02), _outward(306), 0.045, 0.032, 0.022, trim),
        _drum(on(306, belt - 0.02), 0.034, 0.012, dark, toward=_outward(306)),
        # A cable looping out of the top of its back and in again by the hip.
        core.tube(
            "part",
            core.bezier(
                [
                    (0.1, middle + 0.12, rim + 0.01),
                    (0.118, middle + 0.275, belt + 0.03),
                    (0.105, middle + 0.255, belt - 0.13),
                    (0.085, middle + 0.16, bottom + 0.03),
                ],
                samples=6,
            ),
            0.015,
            trim,
            sides=8,
        ),
    ]

    # The stubs of the yoke the arms hang from, through the lid.
    for shoulder in shoulders:
        parts.append(_drum((math.copysign(0.155, shoulder.x), shoulder.y, shoulder.z), 0.03, 0.1, trim, toward=(1, 0, 0), sides=12))

    # A plate over the right shoulder only. It is bolted to the yoke, not
    # to the arm, and kept low and narrow: a plate that swung with the arm
    # would tip up into the jaw whenever the head was turned that way.
    plate = shoulders[1] + Vector((0.028, 0, 0.054))

    parts.append(_box(plate, (0.037, 0.082, 0.018), body, bevel=0.01, turn=(0, math.radians(9), 0)))

    for along in (-0.05, 0.05):
        parts.append(_bolt(plate + Vector((0, along, 0.017)), (0, 0, 1), detail, 0.009))

    for degrees in (207, 246):
        for height in (belt - 0.07, belt + 0.02):
            parts.append(_bolt(on(degrees, height, 0.008), _outward(degrees), detail, 0.009))

    # Vent slots low on the front, off to its left.
    for row in range(3):
        height = hoops[1] - 0.05 - row * 0.027

        parts.append(_place(_band(radius, 286, 326, height - 0.007, height + 0.007, 0.003, dark), (0, middle, 0)))

    for height in hoops:
        for degrees in range(22, 360, 45):
            parts.append(_bolt(on(degrees, height, 0.013), _outward(degrees), detail))

    # An exhaust stack up the right of its back that leans out over the
    # shoulder, so even the shoulder line is lopsided. It leans far enough
    # for the head to turn, and the arm to swing, without hitting it.
    stack = core.bezier(
        [
            (-0.15, middle + 0.195, belt - 0.2),
            (-0.152, middle + 0.2, belt),
            (-0.172, middle + 0.205, belt + 0.13),
            (-0.32, middle + 0.185, belt + 0.35),
        ],
        samples=5,
    )
    lean = (stack[-1] - stack[-2]).normalized()

    parts += [
        core.tube("part", stack, 0.028, trim, sides=10),
        _drum(stack[0] + Vector((0, 0, 0.012)), 0.042, 0.036, detail, sides=12),
        _drum(stack[6], 0.036, 0.022, detail, sides=12),
        _drum(stack[-1] - lean * 0.014, 0.036, 0.024, detail, toward=lean, sides=12),
        _drum(stack[-1], 0.021, 0.004, dark, toward=lean, sides=10, chamfer=0.001),
    ]

    _weld("torso", parts, root)

    return Vector((0, middle + 0.035, lid + 0.06)), hips


def _head(root, neck, face, top):
    body, trim, detail, dark, accent = _palette()
    head = creature.pivot("head", root, neck)

    # A box, with its front plate exactly where the face seal is.
    front, rear = face.y + 0.005, 0.15
    crown, chin = top - 0.01, neck.z + 0.085
    skull = Vector((0, (front + rear) / 2, (crown + chin) / 2))
    eye_l = Vector((0.094, front, face.z + 0.03))
    eye_r = Vector((-0.095, front, face.z + 0.038))
    forward = Vector((0, -1, 0))
    socket = Vector((-0.085, 0.03, crown))

    parts = [
        _box(skull, (0.175, (rear - front) / 2, (crown - chin) / 2), body, bevel=0.02),
        # A brow plate bolted on crooked, a little wider than the box.
        _box((0, front - 0.012, crown - 0.02), (0.186, 0.04, 0.024), body, bevel=0.01, turn=(0, math.radians(-3.5), 0)),
        # The roof of the mouth: a dark line over the jaw, and the darkness
        # that shows when the jaw drops.
        _box((0, front + 0.052, chin - 0.012), (0.118, 0.042, 0.02), dark, bevel=0.004),
        # The head sits on a ball, in a collar under the skull, so it can
        # turn and tip without anything opening up. The collar is short, to
        # leave room for the back of the jaw to swing under it.
        _ball(neck, 0.05, trim),
        _drum((neck.x, neck.y, chin - 0.02), 0.062, 0.05, trim),
        # Its left eye is a big round lens.
        _bezel(eye_l, forward, 0.06, 0.045, 0.042, detail),
        _drum(eye_l, 0.047, 0.012, dark, toward=forward),
        _ball(eye_l + forward * 0.01, 0.016, accent),
        # Its left ear is a listening horn, which breaks the outline of the
        # box even when it wears nothing.
        _drum((0.18, 0.0, skull.z), 0.052, 0.036, trim, toward=(1, 0, 0)),
        _lathe(
            [(0.024, 0.0), (0.03, 0.03), (0.07, 0.078), (0.075, 0.086), (0.066, 0.086), (0.024, 0.04)],
            (0.19, 0.0, skull.z),
            detail,
            toward=(1, 0, 0),
            sides=16,
        ),
        _drum((0.231, 0.0, skull.z), 0.023, 0.004, dark, toward=(1, 0, 0), sides=12, chamfer=0.001),
        # Its right ear is a junction box, wired to the collar.
        _box((-0.186, 0.03, skull.z + 0.03), (0.02, 0.042, 0.032), detail, bevel=0.007),
        core.tube(
            "part",
            core.bezier(
                [(-0.192, 0.035, skull.z + 0.005), (-0.216, 0.06, skull.z - 0.05), (-0.17, 0.08, chin - 0.012), (-0.04, 0.06, chin - 0.02)],
                samples=5,
            ),
            0.012,
            trim,
            sides=8,
        ),
        # Where an antenna plugs in.
        _drum(socket, 0.025, 0.018, trim, sides=12, chamfer=0.004),
    ]

    # Its right eye is a small camera under a square hood.
    for across, up in ((0, 1), (0, -1), (1, 0), (-1, 0)):
        bar = (0.011, 0.026, 0.038) if across else (0.052, 0.026, 0.011)

        parts.append(_box(eye_r + Vector((across * 0.041, -0.014, up * 0.029)), bar, trim, bevel=0.004))

    parts += [
        _box(eye_r + forward * 0.002, (0.034, 0.006, 0.022), dark, bevel=0.002),
        _ball(eye_r + forward * 0.008, 0.015, accent),
    ]

    # Vent slots and a bolt in each corner across the back of the skull.
    for row in range(3):
        parts.append(_box((0.05, rear, skull.z + 0.03 - row * 0.03), (0.07, 0.004, 0.008), dark, bevel=0.002))

    for across in (-0.145, 0.145):
        for height in (chin + 0.03, crown - 0.03):
            parts.append(_bolt((across, rear, height), (0, 1, 0), detail))

        parts.append(_bolt((across, front, chin + 0.022), forward, detail))

    _weld("skull", parts, head)

    # The jaw is a tray from a different machine, with a grille across its
    # front. The game drops it a long way open, whichever way the head is
    # turned, so it is short, hinged halfway back and narrower than the
    # skull: a jaw the size of the box above it would land on the chest and
    # the shoulders.
    hinge = Vector((0, front + 0.085, chin - 0.008))
    jaw = creature.pivot("jaw", head, hinge - neck)
    low, lip, back = chin - 0.075, front - 0.018, hinge.y + 0.012
    tray = Vector((0, (lip + back) / 2, (hinge.z + low) / 2))
    parts = [_box(tray, (0.13, (back - lip) / 2, (hinge.z - low) / 2), detail, bevel=0.012)]

    for slot in range(5):
        parts.append(_box(((slot - 2) * 0.045, lip, tray.z - 0.002), (0.013, 0.004, 0.02), dark, bevel=0.002))

    for side in (1, -1):
        parts += [
            _drum((side * 0.136, hinge.y, hinge.z), 0.028, 0.03, trim, toward=(1, 0, 0), sides=12),
            _bolt((side * 0.151, hinge.y, hinge.z), (side, 0, 0), detail),
        ]

    _weld("mouth", parts, jaw)

    _antenna(creature.pivot("trait_1", head, (0, 0, 0)), socket)
    _crest(creature.pivot("trait_2", head, (0, 0, 0)), front, rear, crown)


def _antenna(parent, socket):
    _, trim, detail, _, _ = _palette()

    # A bent mast with a small dish on it, plugged into the socket on the
    # right of the skull. The dish looks up and ahead.
    knuckle = socket + Vector((0, 0, 0.075))
    hub = knuckle + Vector((-0.022, -0.004, 0.06))
    aim = Vector((-0.2, -0.62, 0.76)).normalized()

    _weld(
        "antenna",
        [
            _lathe([(0.032, 0.0), (0.02, 0.014), (0.012, 0.03)], socket, trim, sides=12),
            _rod(socket, knuckle, 0.009, detail),
            _ball(knuckle, 0.018, trim),
            _rod(knuckle, hub, 0.008, detail),
            _lathe(
                [(0.014, -0.008), (0.042, 0.004), (0.066, 0.026), (0.07, 0.033), (0.063, 0.032), (0.04, 0.013), (0.014, 0.004)],
                hub,
                detail,
                toward=aim,
                sides=20,
            ),
            _rod(hub, hub + aim * 0.052, 0.005, trim, cuts=1),
            _ball(hub + aim * 0.052, 0.012, trim),
        ],
        parent,
    )


def _crest(parent, front, rear, crown):
    _, trim, detail, _, _ = _palette()

    # A fin from brow to nape, cut into ridges like the comb on a helmet. It
    # is tallest a third of the way back, and stands in a holder wide enough
    # to show from the front, which is where the head is mostly seen from.
    start, end, ridges = front - 0.03, rear + 0.01, 7
    wide = (end - start) / ridges
    outline = [(end, crown - 0.012), (start, crown - 0.012)]

    for ridge in range(ridges):
        at = start + wide * ridge
        height = crown + 0.07 + 0.075 * math.sin(math.pi * ((ridge + 0.5) / ridges) ** 0.6)

        # One ridge: up out of the notch, along its flat top and down again.
        outline += [(at + wide * along, height - drop) for along, drop in ((0.08, 0.03), (0.22, 0.0), (0.8, 0.0), (0.94, 0.03))]

    # A plate is cut in the XZ plane; a quarter turn stands it along the
    # head. Its many edges are chamfered once rather than rounded, which is
    # all that shows at this size.
    fin = core.apply_modifier(core.plate("part", outline, 0.07, detail), "BEVEL", width=0.008, segments=1)
    holder = _box((0, (start + end) / 2, crown + 0.012), (0.066, (end - start) / 2 + 0.008, 0.02), trim, bevel=0.009, top=(0.8, 1.0))

    _weld("crest", [_place(fin, turn=(0, 0, math.pi / 2)), holder], parent)


def _claw_arm(root, shoulder):
    body, trim, detail, _, _ = _palette()

    # Its left arm: a ball joint, a thin piston, a gauntlet off something
    # bigger and a three-fingered claw. The ball is as wide as the shoulder
    # seal is far out, and a ball looks the same however the arm swings.
    joint = Vector((shoulder.x - 0.05, shoulder.y, shoulder.z))
    elbow = joint + Vector((0.035, 0.0, -0.18))
    wrist = elbow + Vector((0.008, -0.035, -0.17))
    up = (elbow - wrist).normalized()
    arm = creature.pivot("arm_l", root, joint)

    parts = [
        _ball(joint, 0.05, trim),
        _rod(joint, elbow, 0.02, trim),
        _drum(joint.lerp(elbow, 0.48), 0.033, 0.08, trim, toward=elbow - joint, sides=12),
        _ball(elbow, 0.04, trim),
        _lathe(
            [(0.032, -0.004), (0.06, 0.0), (0.064, 0.008), (0.064, 0.036), (0.054, 0.044), (0.045, 0.14), (0.03, 0.162)],
            wrist,
            body,
            toward=up,
            sides=20,
        ),
        _drum(wrist - up * 0.012, 0.042, 0.03, trim, toward=up, sides=14),
        _bolt(wrist + up * 0.022 + Vector((0.064, 0, 0)), (1, 0, 0), detail),
    ]

    # Two fingers in front and one behind, each hooking back in at the tip.
    curl = ((0.02, 0.012), (0.072, 0.05), (0.076, 0.1), (0.03, 0.145))

    for degrees in (215, 325, 90):
        finger = core.bezier([wrist + _outward(degrees) * out - up * down for out, down in curl], samples=5)

        parts.append(core.tube("part", finger, lambda along: 0.02 - 0.013 * along, detail, sides=6))

    _weld("claw_arm", parts, arm)


def _clamp_arm(root, shoulder):
    body, trim, detail, _, _ = _palette()

    # Its right arm: a drum of a shoulder, a box of an upper arm, a hinge,
    # two bare rods and a clamp. The drum lies along the axis the arm swings
    # about, so its end stays under the shoulder seal.
    joint = Vector((shoulder.x + 0.05, shoulder.y, shoulder.z))
    elbow = joint + Vector((-0.025, 0, -0.195))
    wrist = elbow + Vector((0, -0.03, -0.165))
    arm = creature.pivot("arm_r", root, joint)

    parts = [
        _drum(joint, 0.054, 0.1, trim, toward=(1, 0, 0)),
        _box(elbow + Vector((0, 0, 0.1)), (0.043, 0.05, 0.07), body, bevel=0.012, cuts=1),
        _drum(elbow, 0.042, 0.112, trim, toward=(1, 0, 0), sides=16),
        _bolt(elbow + Vector((-0.056, 0, 0)), (-1, 0, 0), detail, 0.016),
        _box(wrist, (0.046, 0.048, 0.028), body, bevel=0.01),
        # The clamp: a spine, a long jaw and a short one, and the screw that
        # closes them, with its knob out to the side.
        _box(wrist + Vector((0, 0, -0.042)), (0.064, 0.032, 0.018), detail, bevel=0.007),
        _box(wrist + Vector((-0.05, 0, -0.105)), (0.014, 0.032, 0.062), detail, bevel=0.006, cuts=2),
        _box(wrist + Vector((-0.03, 0, -0.158)), (0.024, 0.032, 0.011), detail, bevel=0.005),
        _box(wrist + Vector((0.05, 0, -0.09)), (0.014, 0.032, 0.047), detail, bevel=0.006, cuts=2),
        _box(wrist + Vector((0.032, 0, -0.128)), (0.022, 0.032, 0.011), detail, bevel=0.005),
        _rod(wrist + Vector((-0.06, 0, -0.042)), wrist + Vector((-0.115, 0, -0.042)), 0.01, trim, cuts=1),
        _ball(wrist + Vector((-0.118, 0, -0.042)), 0.019, detail),
    ]

    for offset in (-0.024, 0.024):
        parts.append(_rod(elbow + Vector((0, offset, 0)), wrist + Vector((0, offset, 0)), 0.013, trim))

    _weld("clamp_arm", parts, arm)


def _piston_leg(root, hips, floor):
    body, trim, detail, _, _ = _palette()

    # Its left leg is all rod and joint, standing on a flat plate.
    hip = hips + Vector((0.105, 0, 0))
    ankle = Vector((hip.x + 0.004, hip.y + 0.012, floor + 0.045))
    knee = hip.lerp(ankle, 0.5) + Vector((0, -0.034, 0))
    leg = creature.pivot("leg_l", root, hip)

    parts = [
        _drum(hip, 0.044, 0.07, trim, toward=(1, 0, 0)),
        _rod(hip, knee, 0.024, trim),
        _ball(knee, 0.038, trim),
        _box(knee + Vector((0, -0.04, 0.004)), (0.036, 0.011, 0.042), body, bevel=0.008),
        _rod(knee, ankle, 0.019, trim),
        _drum(knee.lerp(ankle, 0.55), 0.03, 0.06, trim, toward=knee - ankle, sides=12),
        _ball(ankle, 0.03, trim),
        # The plate, a little narrower on top so it reads as cut sheet.
        _box((ankle.x + 0.008, ankle.y - 0.05, floor + 0.018), (0.086, 0.14, 0.018), body, bevel=0.007, top=(0.92, 0.95)),
    ]

    for across in (-0.05, 0.05):
        for along in (-0.155, 0.055):
            parts.append(_bolt((ankle.x + 0.008 + across, ankle.y + along, floor + 0.036), (0, 0, 1), detail, 0.01))

    _weld("piston_leg", parts, leg)


def _drum_leg(root, hips, floor):
    body, trim, detail, _, _ = _palette()

    # Its right leg is a tin can on a short piston, standing on a drum. The
    # can starts a little below the hip so it clears the body as it swings.
    hip = hips + Vector((-0.105, 0, 0))
    foot = Vector((hip.x - 0.006, hip.y - 0.015, floor))
    leg = creature.pivot("leg_r", root, hip)

    parts = [
        _ball(hip, 0.05, trim),
        _lathe(
            [
                (0.036, -0.03),
                (0.056, -0.032),
                (0.062, -0.044),
                *_hoop(0.062, -0.072, proud=0.008, wide=0.02),
                (0.062, -0.172),
                (0.054, -0.184),
                (0.034, -0.184),
            ],
            hip + Vector((-0.006, 0, 0)),
            body,
            sides=20,
        ),
        _drum(hip.lerp(foot, 0.62) + Vector((0, 0.008, 0)), 0.03, 0.07, trim, sides=12),
        _lathe(
            [
                (0.052, 0.104),
                (0.078, 0.104),
                (0.084, 0.096),
                *_hoop(0.084, 0.07, proud=0.008, wide=0.018),
                (0.084, 0.01),
                (0.076, 0.0),
                (0.05, 0.0),
            ],
            foot,
            body,
            sides=24,
        ),
    ]

    for degrees in range(30, 360, 60):
        parts.append(_bolt(foot + _outward(degrees) * 0.064 + Vector((0, 0, 0.104)), (0, 0, 1), detail, 0.01))

    _weld("drum_leg", parts, leg)


# The kit: shapes every part above is made from --------------------------


def _palette():
    """The five surfaces a robot is painted in: plating, frame, worn metal,
    darkness and light."""

    return tuple(core.material(name) for name in ("body", "trim", "detail", "dark", "accent"))


def _outward(degrees):
    """The level direction `degrees` around the body. The creature faces -Y,
    so 270 is straight ahead, 0 is its left and 180 its right."""

    return Vector((math.cos(math.radians(degrees)), math.sin(math.radians(degrees)), 0))


def _place(part, at=(0, 0, 0), turn=(0, 0, 0)):
    """Turns and moves a part's mesh, leaving the object itself where it
    was, so it can hang from a pivot without a transform of its own."""

    part.data.transform(Matrix.Translation(at) @ Euler(turn).to_matrix().to_4x4())

    return part


def _aim(part, at, toward):
    """Stands a part that was made along +Z along `toward` instead, at `at`."""

    turn = Vector((0, 0, 1)).rotation_difference(Vector(toward).normalized())
    part.data.transform(Matrix.Translation(at) @ turn.to_matrix().to_4x4())

    return part


def _box(centre, half, mat, bevel=0.012, top=(1.0, 1.0), turn=(0, 0, 0), cuts=0):
    """A machined block. The bevel is what keeps the wear mask on its edges
    and off its faces. `top` narrows the upper face, and `cuts` adds rings up
    a long block so that grime where it meets another part stays near the
    joint instead of smearing down its whole length."""

    x, y, z = half
    rings = []

    for step in range(cuts + 2):
        along = step / (cuts + 1)
        across = x * (1 + (top[0] - 1) * along)
        deep = y * (1 + (top[1] - 1) * along)
        height = z * (2 * along - 1)

        rings.append([(-across, -deep, height), (across, -deep, height), (across, deep, height), (-across, deep, height)])

    part = core.loft("part", rings, mat, cap_start=True, cap_end=True)

    # A bevel too fine to see round is cut as a single chamfer: it does the
    # same job for the wear mask at a third of the triangles.
    if bevel > 0:
        core.apply_modifier(part, "BEVEL", width=bevel, segments=2 if bevel >= 0.008 else 1)

    return _place(part, centre, turn)


def _lathe(profile, at, mat, toward=(0, 0, 1), sides=24, caps=True):
    """A part turned on a lathe: `profile` spun about the axis `toward`."""

    return _aim(core.revolve("part", profile, mat, segments=sides, cap_top=caps, cap_bottom=caps), at, toward)


def _drum(at, radius, length, mat, toward=(0, 0, 1), sides=16, chamfer=0.006):
    """A solid cylinder centred on `at`: a joint, a collar, a can. On a big
    one, a ring inside each end keeps the wear mask on the rim."""

    half = length / 2
    profile = [(radius - chamfer, half), (radius, half - chamfer), (radius, chamfer - half), (radius - chamfer, -half)]

    if radius >= 0.04:
        profile = [(radius * 0.6, half), *profile, (radius * 0.6, -half)]

    return _lathe(profile, at, mat, toward, sides)


def _hoop(radius, height, proud=0.013, wide=0.026):
    """The profile of a raised band around a can, from its top down."""

    return [
        (radius, height + wide / 2 + proud * 0.55),
        (radius + proud, height + wide / 2),
        (radius + proud, height - wide / 2),
        (radius, height - wide / 2 - proud * 0.55),
    ]


def _bezel(at, toward, outer, inner, depth, mat, sides=20):
    """A short open tube standing off a surface: the rim of a lens or a
    gauge. Whatever sits at the bottom of it reads as deep."""

    rim = (outer - inner) * 0.3
    profile = [(outer, -0.012), (outer, depth - rim), (outer - rim, depth), (inner + rim, depth), (inner, depth - rim), (inner, 0.0)]

    return _lathe(profile, at, mat, toward, sides, caps=False)


def _band(radius, start, end, low, high, thick, mat, steps=8):
    """A plate bent around the Z axis to hug a can, from `start` to `end`
    degrees and between two heights."""

    section = ((radius - 0.01, low), (radius + thick, low), (radius + thick, high), (radius - 0.01, high))
    rings = []

    for step in range(steps + 1):
        out = _outward(start + (end - start) * step / steps)

        rings.append([(out.x * reach, out.y * reach, height) for reach, height in section])

    return core.loft("part", rings, mat, cap_start=True, cap_end=True)


def _bolt(at, toward, mat, size=0.012):
    """A hexagonal bolt head standing on a surface at `at`."""

    return _lathe([(size * 0.7, size), (size, size * 0.65), (size, -size * 0.5)], at, mat, toward, sides=6, caps=True)


def _ball(at, radius, mat):
    """A ball: a joint that can turn any way, or the light in an eye."""

    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2, radius=radius)

    part = bpy.context.object
    part.data.materials.append(mat)

    return _place(part, at)


def _rod(start, end, radius, mat, sides=8, cuts=4):
    """A straight rod. It is cut into short lengths because both ends are
    usually sunk in a joint, and the grime that paints there should stay at
    the ends."""

    start, end = Vector(start), Vector(end)

    return core.tube("part", [start.lerp(end, step / (cuts + 1)) for step in range(cuts + 2)], radius, mat, sides=sides)


def _origin(node):
    """Where a pivot sits on the creature. Pivots are never turned or
    scaled, so it is the sum of the offsets up the chain."""

    at = Vector()

    while node is not None:
        at += Vector(node.location)
        node = node.parent

    return at


def _weld(name, parts, parent):
    """Welds loose parts into one mesh for each material, hangs them from
    `parent` and finishes them.

    Parts are modelled where they sit on the creature, which is far easier
    to read than offsets from a joint; the weld moves them into the pivot's
    own space. Welding before finishing lets a part shade its neighbours,
    and keeps the game from drawing a hundred bolts one at a time.
    """

    at = _origin(parent)
    sorted_parts = {}

    for part in parts:
        sorted_parts.setdefault(part.data.materials[0].name, []).append(part)

    for material, group in sorted_parts.items():
        mesh = core.join(group, f"{name}_{material}") if len(group) > 1 else group[0]
        mesh.name = mesh.data.name = f"{name}_{material}"
        mesh.data.transform(Matrix.Translation(-at))
        mesh.parent = parent

        # Flat faces have to stay flat next to a rounded bevel, so the
        # shading is weighted toward the big faces.
        core.shade_smooth(mesh, angle=SHARP)
        core.apply_modifier(mesh, "WEIGHTED_NORMAL", keep_sharp=True)
        creature.finish(mesh)
