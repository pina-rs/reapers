"""The bird: Thugbirdz.

A street pigeon built like a bouncer: a big round barrel of a body with a
puffed chest, a small hard head thrust out ahead of it on a short neck, and
a pair of black wraparound shades across the face, the one shape you see
first. A heavy hooked beak juts under the shades and drops open on its
hinge at the cheek. It hangs in the air on short wings held out, drooped
and swept back, so the body reads from the side, and cocks a fan of tail
up behind it.

Every part is lofted through boxy sections (`creature.shell`) or cut as a
thick blade (`creature.blade`), so it prints in a few big planes.

It wears a snapback turned to one side or a heavy chain.
"""

import math

from mathutils import Vector

from lib import core, creature

# The middle of the head, which it turns about: forward of the body, so the
# head and the beak stand clear of the chest in profile.
SKULL = Vector((0.0, -0.04, 0.2))
# The left wing's root, high on the shoulder; the right mirrors it.
WING_ROOT = Vector((0.12, 0.05, 0.135))
TAIL_ROOT = Vector((0.0, 0.15, -0.2))
# The beak's hinge, back at the cheek under the shades.
JAW = Vector((0.0, -0.11, 0.112))


def build(spec):
    root = core.empty("husk")
    body = core.material("body", color=(1.0, 1.0, 1.0), roughness=0.70, pbr="plumage")
    trim = core.material("trim", color=(1.0, 1.0, 1.0), roughness=0.78, pbr="plumage")
    detail = core.material("detail", color=(1.0, 1.0, 1.0), roughness=0.38, metallic=0.18, pbr="skin")
    dark = core.material("dark", color=(1.0, 1.0, 1.0), roughness=0.10, metallic=0.22)
    accent = core.material("accent", color=(1.0, 1.0, 1.0), roughness=0.20)

    torso, chest = _torso(root, body, trim)
    _feet(root, detail)
    _tail(root, body)

    for side in (1, -1):
        _wing(root, side, body)

    head, skull = _head(root, body, trim, detail, dark, accent)

    # The seals' sockets: the brow's on the forehead over the shades, the
    # chest's on the shield, the flanks' and the back's on the barrel.
    creature.seat("seat_0", spec, 0, detail, head, [skull], radius=0.048, rise=0.01)
    creature.seat("seat_1", spec, 1, detail, root, [chest], radius=0.054, rise=0.01)

    for index in (2, 3, 4):
        creature.seat(f"seat_{index}", spec, index, detail, root, [torso], radius=0.052, rise=0.01)

    _snapback(creature.pivot("trait_1", head, (0, 0, 0)), detail, skull)
    _chain(creature.pivot("trait_2", root, (0, 0, 0)), detail, root)

    creature.studs(root, spec)

    return root


def _torso(root, body, trim):
    """The barrel, round and deep, and the chest shield puffed out low in
    front of it. Its sides meet the flank seals and its hunched back the
    back seal; above them it narrows hard into the neck."""

    torso = creature.shell(
        "torso",
        [
            ((0, 0.03, -0.272), 0.06, 0.05, 0.06, 2.0),
            ((0, 0.025, -0.235), 0.15, 0.14, 0.16, 2.4),
            ((0, 0.015, -0.15), 0.218, 0.185, 0.205, 3.0),
            ((0, 0.005, -0.05), 0.232, 0.19, 0.205, 3.4),
            ((0, 0.0, 0.05), 0.2, 0.17, 0.2, 3.4),
            ((0, 0.005, 0.1), 0.162, 0.13, 0.184, 3.0),
            ((0, 0.0, 0.13), 0.1, 0.08, 0.13, 2.4),
            ((0, -0.01, 0.15), 0.055, 0.05, 0.065, 2.2),
        ],
        body,
        root,
        sides=20,
        start=0.012,
        end=0.012,
        axis=(0, 0, 1),
    )

    # The shield: proudest low on the chest, where its seal is, and falling
    # back into the throat so the beak swings clear of it.
    chest = creature.shell(
        "front",
        [
            ((0, -0.035, -0.215), 0.04, 0.05, 0.03, 2.0),
            ((0, -0.05, -0.17), 0.11, 0.12, 0.05, 2.4),
            ((0, -0.055, -0.07), 0.165, 0.138, 0.05, 3.0),
            ((0, -0.055, 0.0), 0.168, 0.136, 0.05, 3.2),
            ((0, -0.05, 0.06), 0.142, 0.115, 0.05, 3.0),
            ((0, -0.04, 0.105), 0.09, 0.075, 0.04, 2.6),
        ],
        trim,
        root,
        sides=24,
        start=0.01,
        end=0.012,
        axis=(0, 0, 1),
    )

    return torso, chest


def _head(root, body, trim, detail, dark, accent):
    head = creature.pivot("head", root, SKULL)

    # A small hard head: a flat face for the shades, a brow that slopes back
    # over them to the crown, a pinch of neck under the jaw.
    skull = creature.shell(
        "skull",
        [
            (Vector((0, -0.01, 0.09)) - SKULL, 0.07, 0.065, 0.065, 2.2),
            (Vector((0, -0.03, 0.12)) - SKULL, 0.11, 0.12, 0.09, 2.6),
            (Vector((0, -0.04, 0.155)) - SKULL, 0.166, 0.13, 0.125, 3.2),
            (Vector((0, -0.035, 0.19)) - SKULL, 0.168, 0.112, 0.135, 3.2),
            (Vector((0, -0.025, 0.215)) - SKULL, 0.155, 0.05, 0.135, 3.0),
            (Vector((0, -0.012, 0.25)) - SKULL, 0.138, 0.03, 0.125, 2.8),
            (Vector((0, 0.0, 0.282)) - SKULL, 0.098, 0.026, 0.09, 2.5),
            (Vector((0, 0.005, 0.3)) - SKULL, 0.045, 0.015, 0.045, 2.2),
        ],
        body,
        head,
        sides=24,
        start=0.01,
        end=0.012,
        axis=(0, 0, 1),
    )

    tree = creature.surface([skull])
    _shades(head, tree, dark, accent)
    _brows(head, tree, detail)
    _beak(head, detail, dark)
    _ducktail(head, body)

    return head, skull


def _around(tree, angle, height, lift):
    """A point on the skull `angle` degrees round from the front, at
    `height`, `lift` off its surface, in the head's space (the tree is the
    creature's)."""

    turn = math.radians(angle)
    direction = Vector((math.sin(turn), -math.cos(turn), 0.0))
    centre = Vector((0.0, SKULL.y, height))

    return creature.onto(tree, centre + direction * 0.5, -direction, missing=centre) + direction * lift - SKULL


def _shades(head, tree, dark, accent):
    """Wraparound shades: one broad black band across the face and round to
    the cheeks, deepest over the beak, with a short thin glint on each lens
    and nothing else. It is the bird's read, so it is the biggest shape on
    the face."""

    reach = 84.0
    steps = 30
    rings = []

    for step in range(steps + 1):
        angle = -reach + 2 * reach * step / steps
        u = abs(angle) / reach
        top = 0.198 - 0.014 * u**2
        bottom = 0.138 + 0.022 * u**1.8
        inner, outer = -0.006, 0.013 - 0.006 * u**2
        rings.append(
            [
                _around(tree, angle, bottom, inner),
                _around(tree, angle, bottom, outer - 0.004),
                _around(tree, angle, bottom + 0.004, outer),
                _around(tree, angle, top - 0.004, outer),
                _around(tree, angle, top, outer - 0.004),
                _around(tree, angle, top, inner),
            ]
        )

    shades = core.loft("visor", rings, dark, head, cap_start=True, cap_end=True)
    creature.finish(core.shade_smooth(shades, angle=40))

    # The glints: a short slit on each lens, its inner end low, a scowl.
    middle = _around(tree, 0.0, 0.172, 0.016)
    eyes = creature.pivot("eyes", head, middle)
    strips = []

    for side in (1, -1):
        rings = []

        for step in range(5):
            t = step / 4
            angle = side * (22 + 22 * t)
            height = 0.17 + 0.012 * t
            half = 0.0028 + 0.0024 * math.sin(t * math.pi)
            rings.append([_around(tree, angle, height + way * half, lift) - middle for way, lift in ((-1, 0.014), (1, 0.014), (1, 0.018), (-1, 0.018))])

        strips.append(core.loft(f"glint_{side}", rings, accent, cap_start=True, cap_end=True))

    band = core.join(strips, "eye_band")
    band.parent = eyes
    core.shade_smooth(band, angle=40)


def _brows(head, tree, detail):
    """A heavy brow over each lens, just above the shades' top edge, level,
    so the game tilts it. Each starts well out from the brow seal, so the
    seal's rosette stands clear between them instead of joining them in one
    band."""

    for side in (1, -1):
        path = [_around(tree, side * angle, 0.214, 0.008) for angle in (37, 50, 63, 76)]
        middle = path[1].lerp(path[2], 0.5)
        brow = creature.pivot(f"brow_{creature.SIDES[side]}", head, middle)
        creature.blade(
            f"brow_{side}",
            [point - middle for point in path],
            lambda t: 0.021 + 0.006 * math.sin(t * math.pi) - 0.008 * t,
            0.026,
            detail,
            brow,
            up=(0, -1, 0.5),
            tip=0.012,
            crease=45,
        )


def _beak(head, detail, dark):
    """A heavy hooked beak cut in planes: a ridge along the top, flat sides,
    and a hook over the tip of the lower half. The lower half reaches the
    hook and hangs from the hinge at the cheek, so it drops well clear; the
    inside of both halves is dark, so an open beak is a black wedge."""

    def at(x, y, z):
        return Vector((x, y, z)) - SKULL

    creature.shell(
        "beak_upper",
        [
            (at(0, -0.12, 0.122), 0.06, 0.026, 0.01, 1.9),
            (at(0, -0.19, 0.122), 0.052, 0.028, 0.009, 1.8),
            (at(0, -0.245, 0.11), 0.04, 0.025, 0.008, 1.8),
            (at(0, -0.29, 0.088), 0.026, 0.02, 0.007, 1.8),
            (at(0, -0.31, 0.06), 0.015, 0.014, 0.006, 1.9),
            (at(0, -0.303, 0.04), 0.007, 0.007, 0.005, 2.0),
        ],
        detail,
        head,
        sides=12,
        start=0.0,
        end=0.006,
        forward=(0, 0, 1),
        crease=38,
    )

    # The roof of the mouth, the whole width of the upper beak's underside.
    creature.shell(
        "mouth",
        [
            (at(0, -0.11, 0.115), 0.054, 0.005, 0.005, 2.4),
            (at(0, -0.19, 0.115), 0.046, 0.005, 0.005, 2.4),
            (at(0, -0.25, 0.103), 0.032, 0.004, 0.004, 2.4),
            (at(0, -0.285, 0.083), 0.017, 0.004, 0.004, 2.2),
        ],
        dark,
        head,
        sides=12,
        start=0.0,
        end=0.006,
        forward=(0, 0, 1),
    )

    jaw = creature.pivot("jaw", head, JAW - SKULL)

    def low(x, y, z):
        return Vector((x, y, z)) - JAW

    creature.shell(
        "beak_lower",
        [
            (low(0, -0.11, 0.106), 0.056, 0.005, 0.028, 2.4),
            (low(0, -0.175, 0.106), 0.048, 0.005, 0.026, 2.2),
            (low(0, -0.235, 0.1), 0.035, 0.004, 0.02, 2.0),
            (low(0, -0.28, 0.088), 0.02, 0.004, 0.012, 2.0),
        ],
        detail,
        jaw,
        sides=12,
        start=0.0,
        end=0.01,
        forward=(0, 0, 1),
        crease=38,
    )
    creature.shell(
        "mouth_floor",
        [
            (low(0, -0.11, 0.11), 0.048, 0.003, 0.004, 2.6),
            (low(0, -0.175, 0.11), 0.041, 0.003, 0.004, 2.6),
            (low(0, -0.238, 0.103), 0.026, 0.003, 0.004, 2.4),
        ],
        dark,
        jaw,
        sides=12,
        start=0.0,
        end=0.01,
        forward=(0, 0, 1),
    )


def _ducktail(head, body):
    """Three swept feathers out of the back of the head, under where a cap
    would sit: the street in the silhouette from the side."""

    for index, (across, lift, length) in enumerate(((0.0, 0.0, 0.11), (0.042, -0.02, 0.085), (-0.042, -0.02, 0.085))):
        start = Vector((across, 0.06, 0.24 + lift)) - SKULL
        sweep = Vector((across * 1.6, 1.0, 0.3)).normalized()
        path = [start + sweep * length * t + Vector((0, 0, 0.035 * t * t)) for t in (0.0, 0.4, 0.75, 1.0)]
        creature.blade(f"tuft_{index}", path, lambda t: 0.042 * (1 - t**1.3) + 0.005, 0.02, body, head, up=(0, -0.3, 1), tip=0.016, crease=45)


def _wing(root, side, body):
    """A short wing held out from the shoulder, drooped and swept back: a
    thick covert along its arm, three broad secondaries trailing from it
    and three primaries from the wrist, longest outermost. Its feathers
    trail back more than down, so from the side it is a wedge behind the
    shoulder, not a sail over the body."""

    name = creature.SIDES[side]
    root_at = creature.mirrored(WING_ROOT, side)
    wing = creature.pivot(f"wing_{name}", root, root_at)
    span = creature.mirrored((1.0, 0.5, -0.36), side).normalized()
    trail = Vector((0.0, 0.4, -1.0))
    chord = (trail - span * trail.dot(span)).normalized()
    face = span.cross(chord).normalized() * side
    wrist = span * 0.24
    parts = [
        creature.shell(
            f"covert_{name}",
            [
                (span * 0.0 + chord * 0.005, 0.055, 0.032, 0.03, 2.6),
                (span * 0.1, 0.07, 0.036, 0.03, 2.8),
                (span * 0.19 + chord * 0.004, 0.06, 0.03, 0.026, 2.8),
                (wrist + span * 0.035, 0.04, 0.022, 0.02, 2.4),
            ],
            body,
            sides=16,
            start=0.01,
            end=0.03,
            forward=face,
        )
    ]

    # Secondaries, square-tipped, trailing from the forearm.
    for index, (along, length) in enumerate(((0.05, 0.13), (0.125, 0.15), (0.2, 0.16))):
        base = span * along + chord * 0.028
        direction = (chord + span * (0.06 + 0.12 * index)).normalized()
        path = [base + direction * length * t for t in (0.0, 0.35, 0.7, 1.0)]
        parts.append(creature.blade(f"secondary_{name}_{index}", path, lambda t: 0.046 + 0.012 * math.sin(t * math.pi * 0.8), 0.02, body, up=face, tip=0.008))

    # Primaries from the wrist, splayed like fingers and turned back up to
    # the level of the shoulder, so the drooped arm and the lifted hand make
    # the gull's kink that says bird in a silhouette.
    lift = Vector((0.0, 0.0, 1.0))
    for index, (turn, length) in enumerate(((2, 0.21), (30, 0.19), (58, 0.17))):
        angle = math.radians(turn)
        base = wrist - span * 0.025 * index + chord * 0.02
        direction = (span * math.cos(angle) + chord * math.sin(angle) + lift * (0.3 - 0.3 * index / 2)).normalized()
        path = [base + direction * length * t for t in (0.0, 0.3, 0.62, 0.86, 1.0)]
        parts.append(creature.blade(f"primary_{name}_{index}", path, lambda t: 0.04 * (1.0 - 0.45 * t**2.2) + 0.006, 0.02, body, up=face, tip=0.026))

    feathers = core.join(parts, f"feathers_{name}")
    feathers.parent = wing
    core.shade_smooth(feathers, angle=48)


def _tail(root, body):
    """A broad square tail spread down and back from the rump, as a bird
    brakes in the air: five feathers in a shallow bowl, so it shows as a
    fan under the body from behind and as a wedge from the side."""

    tail = creature.pivot("tail", root, TAIL_ROOT)
    parts = []

    for index, (spread, length, wide) in enumerate(((-36, 0.17, 0.058), (-18, 0.2, 0.064), (0, 0.215, 0.068), (18, 0.2, 0.064), (36, 0.17, 0.058))):
        swing = math.radians(spread)
        rise = 0.5 * abs(math.sin(swing)) / math.sin(math.radians(36))
        direction = Vector((math.sin(swing) * 0.95, math.cos(swing), -0.55 + rise)).normalized()
        path = [Vector((math.sin(swing) * 0.03, -0.02, 0.0)) + direction * length * t for t in (0.0, 0.35, 0.7, 1.0)]
        roll = Vector((math.sin(swing) * 0.5, 0.45, 1.0))
        parts.append(creature.blade(f"tail_{index}", path, lambda t, w=wide: w * (0.7 + 0.35 * t), 0.022, body, up=roll, tip=0.004))

    fan = core.join(parts, "tail_fan")
    fan.parent = tail
    core.shade_smooth(fan, angle=48)


def _feet(root, detail):
    """Feet tucked up under the belly, as a flying bird carries them: a
    short shank and three curled talons each."""

    parts = []

    for side in (1, -1):
        ankle = Vector((side * 0.08, -0.005, -0.235))
        parts.append(creature.shell(f"shank_{side}", [(ankle, 0.027, 0.027, 0.027, 2.0), (ankle + Vector((0, -0.02, -0.045)), 0.023, 0.023, 0.023, 2.0)], detail, sides=10, start=0.0, end=0.01))

        for toe in (-1, 0, 1):
            base = ankle + Vector((toe * 0.021, -0.03, -0.052))
            path = [base, base + Vector((toe * 0.012, -0.036, -0.006)), base + Vector((toe * 0.016, -0.062, 0.006)), base + Vector((toe * 0.016, -0.072, 0.024))]
            parts.append(creature.blade(f"talon_{side}_{toe}", path, lambda t: 0.014 * (1 - 0.6 * t), 0.018, detail, up=(0, 0, 1), tip=0.008))

    feet = core.join(parts, "feet")
    feet.parent = root
    core.shade_smooth(feet, angle=45)


def _snapback(parent, cloth, skull):
    """A snapback pushed back on the head and turned to the side: its crown
    sits on the skull behind the forehead seal, and its flat peak sticks
    out over the left ear, so the hat reads from the front and the side."""

    tree = creature.surface([skull])
    axis = Vector((0, 0.75, 1)).normalized()
    centre = Vector((0, 0.0, 0.232))
    forward = Vector((0, -1, 0))
    front = (forward - axis * forward.dot(axis)).normalized()
    left = axis.cross(front).normalized()
    sides = 24
    rim = []

    for step in range(sides):
        turn = math.tau * step / sides
        direction = front * math.cos(turn) - left * math.sin(turn)
        rim.append(creature.onto(tree, centre + direction * 0.4, -direction, lift=-0.002, missing=centre + direction * 0.12) + direction * 0.01)

    rings = [rim]

    for rise, scale in ((0.03, 0.97), (0.058, 0.85), (0.078, 0.6), (0.088, 0.3)):
        rings.append([centre + axis * rise + (point - centre) * scale for point in rim])

    crown = core.loft("cap", [[point - SKULL for point in ring] for ring in rings], cloth, parent, cap_end=True)
    creature.finish(core.shade_smooth(crown, angle=40))

    # The peak, a flat curved slab out over the left ear and a little back.
    turn = math.radians(82)
    out = Vector((math.sin(turn), math.cos(turn), 0.0)).normalized()
    across = Vector((0, 0, 1)).cross(out).normalized()
    base = creature.onto(tree, centre + out * 0.4, -out, missing=centre + out * 0.13) + Vector((0, 0, 0.004))
    rings = []

    for step in range(6):
        t = step / 5
        half = 0.08 * (1.0 - 0.3 * t**1.6)
        middle = base + out * (0.11 * t - 0.03) - Vector((0, 0, 0.018 * t))
        rings.append(
            [middle + across * u * half - Vector((0, 0, 0.012 * u * u - 0.007)) for u in (-1, -0.5, 0, 0.5, 1)]
            + [middle + across * u * half - Vector((0, 0, 0.012 * u * u + 0.007)) for u in (1, 0.5, 0, -0.5, -1)]
        )

    bill = core.loft("cap_bill", [[point - SKULL for point in ring] for ring in rings], cloth, parent, cap_start=True, cap_end=True)
    creature.finish(core.shade_smooth(bill, angle=40))


def _chain(parent, metal, root):
    """A heavy chain: twelve fat links lying round the neck and over the
    top of the chest shield, every other one turned flat."""

    body = [node for node in root.children_recursive if node.type == "MESH" and node.name in ("torso", "front", "skull")]
    tree = creature.surface(body)
    count = 12
    links = []
    points = []

    for index in range(count):
        angle = math.tau * index / count
        front = (1 + math.cos(angle)) / 2
        height = 0.11 - 0.08 * front**2
        direction = Vector((math.sin(angle), -math.cos(angle), 0.0))
        axis = Vector((0, -0.01, height))
        points.append((creature.onto(tree, axis + direction * 0.5, -direction, missing=axis + direction * 0.2), axis))

    for index, (point, axis) in enumerate(points):
        ahead = points[(index + 1) % count][0]
        behind = points[index - 1][0]
        tangent = (ahead - behind).normalized()
        outward = (point - axis).normalized()
        across = outward.cross(tangent).normalized() if index % 2 == 0 else outward
        links.append(_link(point + outward * 0.02, tangent, across, metal))

    chain = core.join(links, "chain")
    chain.parent = parent
    core.shade_smooth(chain, angle=50)
    creature.finish(chain)


def _link(centre, along, across, mat):
    """One fat oval link, `along` its length, `across` its width."""

    normal = along.cross(across).normalized()
    rings = []

    for step in range(10):
        angle = math.tau * step / 10
        out = along * math.cos(angle) + across * math.sin(angle)
        middle = centre + along * math.cos(angle) * 0.038 + across * math.sin(angle) * 0.023
        rings.append([middle + (out.normalized() * math.cos(turn) + normal * math.sin(turn)) * 0.011 for turn in (0, math.pi / 2, math.pi, math.pi * 1.5)])

    rings.append(rings[0])

    return core.loft("link", rings, mat)
