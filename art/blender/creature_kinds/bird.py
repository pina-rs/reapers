"""The bird: Thugbirdz.

A street pigeon built like a bouncer: a barrel of a body with a puffed chest
shield stuck out under its chin, a big flat-topped head with a wraparound
visor and two narrow glints behind it, and a heavy hooked beak cut in
planes. It hangs in the air on two wings held up and out like a crest, each
a fan of a few big feathers turned to whoever it faces, and steers with a
short square tail. A ducktail sticks out of the back of its head.

Every part is lofted through boxy sections (`creature.shell`) or cut as a
thick blade (`creature.blade`), so it prints in a few big planes.

It wears a snapback turned backwards or a heavy chain.
"""

import math

from mathutils import Vector

from lib import core, creature

# The middle of the head, which it turns about.
SKULL = Vector((0.0, 0.02, 0.17))
# The left wing's root, on the shoulder; the right mirrors it.
WING_ROOT = Vector((0.16, 0.05, 0.13))
TAIL_ROOT = Vector((0.0, 0.17, -0.13))
JAW = Vector((0.0, -0.095, 0.112))


def build(spec):
    root = core.empty("husk")
    body = core.material("body", color=(1.0, 1.0, 1.0), roughness=0.70, pbr="plumage")
    trim = core.material("trim", color=(1.0, 1.0, 1.0), roughness=0.78, pbr="plumage")
    detail = core.material("detail", color=(1.0, 1.0, 1.0), roughness=0.38, metallic=0.18, pbr="skin")
    dark = core.material("dark", color=(1.0, 1.0, 1.0), roughness=0.10, metallic=0.22)
    accent = core.material("accent", color=(1.0, 1.0, 1.0), roughness=0.20)

    _torso(root, body, trim)
    _feet(root, detail)
    _tail(root, body)

    for side in (1, -1):
        _wing(root, side, body, trim)

    head = _head(root, body, trim, detail, dark, accent)

    _snapback(creature.pivot("trait_1", head, (0, 0, 0)), detail)
    _chain(creature.pivot("trait_2", root, (0, 0, 0)), detail, root)

    creature.studs(root, spec)

    return root


def _torso(root, body, trim):
    """The barrel, and the chest shield puffed out in front of it."""

    creature.shell(
        "torso",
        [
            ((0, 0.05, -0.255), 0.06, 0.05, 0.06, 2.0),
            ((0, 0.035, -0.225), 0.135, 0.125, 0.15, 2.2),
            ((0, 0.02, -0.14), 0.19, 0.17, 0.19, 2.5),
            ((0, 0.01, -0.04), 0.21, 0.18, 0.197, 2.6),
            ((0, 0.0, 0.06), 0.201, 0.17, 0.200, 2.6),
            ((0, 0.02, 0.125), 0.155, 0.12, 0.18, 2.6),
            ((0, 0.04, 0.165), 0.095, 0.06, 0.11, 2.4),
        ],
        body,
        root,
        sides=24,
        start=0.012,
        end=0.02,
        axis=(0, 0, 1),
    )

    creature.shell(
        "front",
        [
            ((0, -0.04, -0.2), 0.03, 0.05, 0.03, 2.0),
            ((0, -0.055, -0.15), 0.09, 0.112, 0.05, 2.0),
            ((0, -0.06, -0.04), 0.155, 0.141, 0.05, 2.4),
            ((0, -0.06, 0.04), 0.168, 0.152, 0.05, 2.8),
            ((0, -0.05, 0.1), 0.15, 0.118, 0.04, 3.0),
        ],
        trim,
        root,
        sides=24,
        start=0.01,
        end=0.02,
        axis=(0, 0, 1),
    )


def _head(root, body, trim, detail, dark, accent):
    head = creature.pivot("head", root, SKULL)

    # A big flat-topped head, square at the brow, set back so the face
    # stands over the chest.
    skull = creature.shell(
        "skull",
        [
            (Vector((0, 0.0, 0.06)) - SKULL, 0.11, 0.1, 0.1, 2.2),
            (Vector((0, 0.01, 0.11)) - SKULL, 0.15, 0.125, 0.125, 2.6),
            (Vector((0, 0.02, 0.17)) - SKULL, 0.158, 0.13, 0.13, 2.9),
            (Vector((0, 0.025, 0.225)) - SKULL, 0.142, 0.103, 0.115, 2.9),
            (Vector((0, 0.03, 0.27)) - SKULL, 0.1, 0.062, 0.08, 2.6),
        ],
        body,
        head,
        sides=24,
        start=0.01,
        end=0.016,
        axis=(0, 0, 1),
    )

    tree = creature.surface([skull])
    _visor(head, tree, dark, accent)
    _brows(head, tree, trim)
    _beak(head, detail, dark)
    _ducktail(head, body)

    return head


def _around(tree, angle, height, lift):
    """A point on the skull `angle` degrees round from the front, at
    `height`, `lift` off its surface, in the head's space (the tree is the
    creature's)."""

    turn = math.radians(angle)
    direction = Vector((math.sin(turn), -math.cos(turn), 0.0))
    centre = Vector((0.0, 0.02, height))

    return creature.onto(tree, centre + direction * 0.5, -direction, missing=centre) + direction * lift - SKULL


def _visor(head, tree, dark, accent):
    """Wraparound shades: a band round the face, deepest over the beak and
    tapering to points at the temples, with a narrow glint for each eye."""

    reach = 74.0
    steps = 28
    rings = []

    for step in range(steps + 1):
        angle = -reach + 2 * reach * step / steps
        u = abs(angle) / reach
        top = 0.205 + 0.006 * math.sin(min(u / 0.35, 1.0) * math.pi / 2) - 0.03 * u**2.4
        bottom = 0.118 + 0.045 * u**1.7
        inner, outer = -0.008, 0.012 - 0.008 * u**2
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

    visor = core.loft("visor", rings, dark, head, cap_start=True, cap_end=True)
    creature.finish(core.shade_smooth(visor, angle=40))

    # The glints: two slits on the lens, their inner ends low, a scowl.
    middle = _around(tree, 0.0, 0.164, 0.015)
    eyes = creature.pivot("eyes", head, middle)
    strips = []

    for side in (1, -1):
        rings = []

        for step in range(7):
            t = step / 6
            angle = side * (10 + 36 * t)
            height = 0.154 + 0.02 * t
            half = 0.005 + 0.007 * math.sin(t * math.pi) ** 0.7
            rings.append([_around(tree, angle, height + way * half, lift) - middle for way, lift in ((-1, 0.011), (1, 0.011), (1, 0.016), (-1, 0.016))])

        strips.append(core.loft(f"glint_{side}", rings, accent, cap_start=True, cap_end=True))

    band = core.join(strips, "eye_band")
    band.parent = eyes
    core.shade_smooth(band, angle=40)


def _brows(head, tree, trim):
    """A thick feathered brow over each lens, level, so the game tilts it."""

    for side in (1, -1):
        path = [_around(tree, side * angle, 0.212, 0.01) for angle in (5, 19, 34, 50)]
        middle = path[1].lerp(path[2], 0.5)
        brow = creature.pivot(f"brow_{creature.SIDES[side]}", head, middle)
        creature.blade(
            f"brow_{side}",
            [point - middle for point in path],
            lambda t: 0.02 + 0.006 * math.sin(t * math.pi) - 0.008 * t,
            0.026,
            trim,
            brow,
            up=(0, -1, 0.3),
            tip=0.014,
            crease=45,
        )


def _beak(head, detail, dark):
    """A heavy hooked beak cut in planes: a ridge along the top, flat sides,
    and a hook that drops over the lower half. The lower half hangs from the
    jaw, and a dark mouth shows when it drops."""

    def at(x, y, z):
        return Vector((x, y, z)) - SKULL

    creature.shell(
        "beak_upper",
        [
            (at(0, -0.088, 0.128), 0.066, 0.044, 0.018, 1.8),
            (at(0, -0.15, 0.128), 0.056, 0.042, 0.012, 1.7),
            (at(0, -0.205, 0.116), 0.042, 0.034, 0.009, 1.7),
            (at(0, -0.248, 0.096), 0.026, 0.026, 0.008, 1.7),
            (at(0, -0.268, 0.072), 0.015, 0.017, 0.007, 1.8),
            (at(0, -0.265, 0.05), 0.007, 0.008, 0.005, 2.0),
        ],
        detail,
        head,
        sides=12,
        start=0.0,
        end=0.006,
        forward=(0, 0, 1),
        crease=38,
    )

    # The roof of the mouth, tucked inside the upper beak.
    creature.shell(
        "mouth",
        [
            (at(0, -0.086, 0.108), 0.054, 0.014, 0.01, 2.2),
            (at(0, -0.165, 0.11), 0.042, 0.012, 0.008, 2.2),
            (at(0, -0.225, 0.096), 0.022, 0.008, 0.006, 2.2),
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
            (low(0, -0.09, 0.11), 0.058, 0.004, 0.032, 2.0),
            (low(0, -0.15, 0.106), 0.048, 0.004, 0.028, 1.9),
            (low(0, -0.2, 0.097), 0.033, 0.004, 0.021, 1.9),
            (low(0, -0.234, 0.088), 0.016, 0.004, 0.012, 2.0),
        ],
        detail,
        jaw,
        sides=12,
        start=0.0,
        end=0.008,
        forward=(0, 0, 1),
        crease=38,
    )
    creature.shell(
        "mouth_floor",
        [
            (low(0, -0.088, 0.112), 0.047, 0.004, 0.006, 2.2),
            (low(0, -0.19, 0.102), 0.028, 0.004, 0.006, 2.2),
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

    for index, (across, lift, length) in enumerate(((0.0, 0.0, 0.14), (0.04, -0.016, 0.11), (-0.04, -0.016, 0.11))):
        start = Vector((across, 0.1, 0.2 + lift)) - SKULL
        sweep = Vector((across * 1.4, 1.0, 0.36)).normalized()
        path = [start + sweep * length * t + Vector((0, 0, 0.035 * t * t)) for t in (0.0, 0.4, 0.75, 1.0)]
        creature.blade(f"tuft_{index}", path, lambda t: 0.034 * (1 - t**1.4) + 0.004, 0.018, body, head, up=(0, -0.3, 1), tip=0.022, crease=45)


def _wing(root, side, body, trim):
    """A wing held up and out like a crest: a thick covert along its arm,
    secondaries hanging from it, and four primaries fanned from the wrist
    like the fingers of a hand, the whole fan turned to the front so it
    reads from there as well as from the side."""

    name = creature.SIDES[side]
    root_at = creature.mirrored(WING_ROOT, side)
    wing = creature.pivot(f"wing_{name}", root, root_at)
    span = creature.mirrored((1.0, 0.5, 0.55), side).normalized()
    hang = Vector((0.0, 0.5, -1.0))
    chord = (hang - span * hang.dot(span)).normalized()
    face = span.cross(chord).normalized() * side
    wrist = span * 0.3
    parts = [
        creature.shell(
            f"covert_{name}",
            [
                (span * 0.01 - chord * 0.01, 0.06, 0.032, 0.03, 2.4),
                (span * 0.12 - chord * 0.012, 0.075, 0.036, 0.03, 2.6),
                (span * 0.24 - chord * 0.01, 0.065, 0.03, 0.026, 2.6),
                (wrist + span * 0.04, 0.042, 0.022, 0.02, 2.4),
            ],
            body,
            sides=16,
            start=0.01,
            end=0.03,
            forward=face,
        )
    ]

    # Secondaries, square-tipped, hanging from the forearm.
    for index, (along, length) in enumerate(((0.05, 0.16), (0.12, 0.175), (0.19, 0.185), (0.255, 0.19))):
        base = span * along + chord * 0.03
        direction = (chord + span * (0.05 + 0.1 * index)).normalized()
        path = [base + direction * length * t for t in (0.0, 0.35, 0.7, 1.0)]
        parts.append(creature.blade(f"secondary_{name}_{index}", path, lambda t: 0.04 + 0.012 * math.sin(t * math.pi * 0.8), 0.018, body, up=face, tip=0.008))

    # Primaries from the wrist, longest outermost, splayed apart at their
    # tips like fingers.
    for index, (turn, length) in enumerate(((8, 0.24), (30, 0.23), (52, 0.215), (74, 0.2))):
        angle = math.radians(turn)
        base = wrist - span * 0.02 * index + chord * 0.02
        direction = span * math.cos(angle) + chord * math.sin(angle)
        path = [base + direction * length * t for t in (0.0, 0.3, 0.62, 0.86, 1.0)]
        parts.append(creature.blade(f"primary_{name}_{index}", path, lambda t: 0.04 * (1.0 - 0.4 * t**2.2) + 0.006, 0.018, body, up=face, tip=0.024))

    feathers = core.join(parts, f"feathers_{name}")
    feathers.parent = wing
    core.shade_smooth(feathers, angle=48)


def _tail(root, body):
    """A short square tail: three broad feathers fanned back and down, and
    a short one either side."""

    tail = creature.pivot("tail", root, TAIL_ROOT)
    parts = []

    for index, (spread, length, wide) in enumerate(((-34, 0.15, 0.05), (-14, 0.2, 0.06), (0, 0.22, 0.064), (14, 0.2, 0.06), (34, 0.15, 0.05))):
        swing = math.radians(spread)
        direction = Vector((math.sin(swing), math.cos(swing), -0.6)).normalized()
        path = [Vector((math.sin(swing) * 0.03, -0.05, 0.0)) + direction * length * t for t in (0.0, 0.35, 0.7, 1.0)]
        parts.append(creature.blade(f"tail_{index}", path, lambda t, w=wide: w * (0.75 + 0.3 * t), 0.02, body, up=(math.sin(swing) * 0.3, 0.6, 1.0), tip=0.004))

    fan = core.join(parts, "tail_fan")
    fan.parent = tail
    core.shade_smooth(fan, angle=48)


def _feet(root, detail):
    """Feet tucked up under the belly, as a flying bird carries them: a
    short shank and three curled talons each."""

    parts = []

    for side in (1, -1):
        ankle = Vector((side * 0.075, 0.0, -0.215))
        parts.append(creature.shell(f"shank_{side}", [(ankle, 0.026, 0.026, 0.026, 2.0), (ankle + Vector((0, -0.02, -0.045)), 0.022, 0.022, 0.022, 2.0)], detail, sides=10, start=0.0, end=0.01))

        for toe in (-1, 0, 1):
            base = ankle + Vector((toe * 0.02, -0.03, -0.052))
            path = [base, base + Vector((toe * 0.012, -0.035, -0.006)), base + Vector((toe * 0.016, -0.06, 0.006)), base + Vector((toe * 0.016, -0.07, 0.024))]
            parts.append(creature.blade(f"talon_{side}_{toe}", path, lambda t: 0.013 * (1 - 0.6 * t), 0.017, detail, up=(0, 0, 1), tip=0.008))

    feet = core.join(parts, "feet")
    feet.parent = root
    core.shade_smooth(feet, angle=45)


def _snapback(parent, cloth):
    """A snapback turned backwards: a crown pushed back on the head, its
    front edge clear of the brow, and the flat bill sticking out behind."""

    def at(x, y, z):
        return Vector((x, y, z)) - SKULL

    creature.shell(
        "cap",
        [
            (at(0, 0.035, 0.2), 0.165, 0.128, 0.142, 2.6),
            (at(0, 0.035, 0.24), 0.156, 0.118, 0.132, 2.6),
            (at(0, 0.037, 0.282), 0.125, 0.09, 0.105, 2.4),
            (at(0, 0.04, 0.31), 0.075, 0.05, 0.062, 2.2),
        ],
        cloth,
        parent,
        sides=24,
        start=None,
        end=0.012,
        axis=(0, 0.42, 1),
    )

    rings = []

    for step in range(6):
        t = step / 5
        span = 0.11 * (1.0 - 0.3 * t**1.6)
        y = 0.15 + 0.12 * t
        z = 0.205 + 0.01 * t
        rings.append([at(u * span, y - 0.02 * u * u, z + 0.007 - 0.012 * u * u) for u in (-1, -0.5, 0, 0.5, 1)] + [at(u * span, y - 0.02 * u * u, z - 0.007 - 0.012 * u * u) for u in (1, 0.5, 0, -0.5, -1)])

    bill = core.loft("cap_bill", rings, cloth, parent, cap_start=True, cap_end=True)
    creature.finish(core.shade_smooth(bill, angle=40))


def _chain(parent, metal, root):
    """A heavy chain: fourteen fat links lying round the neck and over the
    top of the chest shield, every other one turned flat."""

    body = [node for node in root.children_recursive if node.type == "MESH" and node.name in ("torso", "front", "skull")]
    tree = creature.surface(body)
    count = 14
    links = []
    points = []

    for index in range(count):
        angle = math.tau * index / count
        front = (1 + math.cos(angle)) / 2
        height = 0.1 - 0.07 * front**2
        direction = Vector((math.sin(angle), -math.cos(angle), 0.0))
        axis = Vector((0, 0.0, height))
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
        middle = centre + along * math.cos(angle) * 0.036 + across * math.sin(angle) * 0.022
        rings.append([middle + (out.normalized() * math.cos(turn) + normal * math.sin(turn)) * 0.01 for turn in (0, math.pi / 2, math.pi, math.pi * 1.5)])

    rings.append(rings[0])

    return core.loft("link", rings, mat)
