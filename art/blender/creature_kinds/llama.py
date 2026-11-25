"""The llama: Sollamas.

A stylized fluffy llama: a tall neck of a few broad planes rising from a
plush cloud of a body, a big head with a long muzzle, a heavy underbite and
curved banana ears, legs that taper from a woolly thigh to a narrow cannon
over cloven hooves, and a woven blanket draped over its back that hangs
below its belly with a hem.

It wears a creased cowboy hat or a folded neckerchief.
"""

import math

from mathutils import Matrix, Vector
from mathutils.bvhtree import BVHTree

from lib import core, creature

# How thick the blanket is, and how far each seal's socket stands out of
# whatever it is set in.
BLANKET = 0.016
SEAT_RISE = 0.008
# The head is drawn at the size below and then grown by this much about the
# brow seal, which stays where the rules put it.
HEAD_GROWTH = 1.3
# How far below the flank seals the blanket's hem hangs, past the belly.
DRAPE = 0.27


def build(spec):
    root = core.empty("husk")
    body = core.material("body", color=(1.0, 1.0, 1.0), roughness=0.94, pbr="wool")
    trim = core.material("trim", color=(1.0, 1.0, 1.0), roughness=0.88, pbr="textile")
    detail = core.material("detail", color=(1.0, 1.0, 1.0), roughness=0.72, pbr="textile")
    dark = core.material("dark", color=(1.0, 1.0, 1.0), roughness=0.26)
    accent = core.material("accent", color=(1.0, 1.0, 1.0), roughness=0.22)
    floor = creature.floor(spec)

    brow, throat, flank, _, rump = (creature.to_blender(seal["offset"]) for seal in spec["seals"])

    spine = flank.z
    # The barrel is as wide as leaves room for the blanket and the flank
    # seals' sockets on it.
    wide = flank.x - BLANKET - SEAT_RISE
    shoulder = throat.y + 0.14
    hip = rump.y + 0.03

    barrel = ((0, flank.y + 0.01, spine - 0.015), (wide, 0.42, 0.21))
    haunch = ((0, rump.y - 0.01, spine), (wide - 0.008, 0.19, rump.z - spine + 0.005))
    chest = ((0, shoulder, spine - 0.01), (0.175, 0.15, 0.2))

    torso = creature.facet(
        "torso",
        [
            ("sphere", *barrel),
            ("sphere", *haunch),
            ("sphere", *chest),
            *_fleece(*chest, 28, 0.064, lambda at: at.y < shoulder + 0.02),
            *_fleece(*haunch, 22, 0.066, lambda at: at.y > rump.y - 0.02 and (at - rump).length > 0.11),
        ],
        body,
        root,
        triangles=2200,
        voxel=0.018,
        flat=11.0,
    )

    # The blanket lies between the shoulders and the hips, clear of the
    # legs, which would otherwise poke through where it hangs.
    _blanket(root, _surface(torso), spine, flank.y + 0.015, spine - DRAPE, trim, detail)

    # Low enough that the tuft stands clear of the rump seal's socket.
    tail_at = Vector((0, rump.y + 0.16, rump.z - 0.17))
    tail = creature.pivot("tail", root, tail_at)
    creature.form(
        "tail_tuft",
        [
            ("sphere", (0, 0.042, 0.022), (0.056, 0.062, 0.058)),
            ("sphere", (0, 0.088, 0.062), (0.046, 0.046, 0.052)),
            ("sphere", (0, 0.105, 0.032), (0.036, 0.038, 0.042)),
        ],
        body,
        tail,
        triangles=340,
        voxel=0.011,
    )

    for side, name in ((1, "l"), (-1, "r")):
        for end, at in (("f", shoulder), ("b", hip)):
            _leg(root, f"{end}{name}", Vector((side * 0.115, at, spine - 0.14)), floor, body, detail)

    _neck(root, brow, throat, spine, body, detail, dark, accent)
    part = {node.name: node for node in root.children_recursive}

    # The seals' sockets: the brow's on the forehead, the throat's on the
    # neck's flat front, the flanks' on the blanket and the rump's on the
    # wool behind it.
    creature.seat("seat_0", spec, 0, detail, part["head"], [part["skull"]], radius=0.044, rise=SEAT_RISE)
    creature.seat("seat_1", spec, 1, detail, part["neck"], [part["neck_wool"]], radius=0.05, rise=SEAT_RISE)

    for index in (2, 3):
        creature.seat(f"seat_{index}", spec, index, detail, root, [part["blanket"]], radius=0.05, rise=SEAT_RISE)

    creature.seat("seat_4", spec, 4, detail, root, [torso], radius=0.048, rise=SEAT_RISE)

    creature.studs(root, spec)
    creature.shadow(root, spec)

    return root


def _about(origin, blobs):
    return [(kind, tuple(Vector(at) - Vector(origin)), *rest) for kind, at, *rest in blobs]


def _fleece(centre, radii, count, size, keep):
    turn = math.pi * (3 - math.sqrt(5))
    puffs = []

    for index in range(count):
        height = 1 - 2 * (index + 0.5) / count
        ring = math.sqrt(1 - height * height)
        out = (math.cos(index * turn) * ring, math.sin(index * turn) * ring, height)
        at = Vector(centre) + Vector([way * (radius - size * 0.86) for way, radius in zip(out, radii)])

        if out[0] < 0 or not keep(at):
            continue

        for side in (1, -1):
            puffs.append(("sphere", (side * at.x, at.y, at.z), (size, size, size)))

    return puffs


def _surface(part):
    mesh = part.data
    return BVHTree.FromPolygons(
        [vertex.co + part.location for vertex in mesh.vertices],
        [polygon.vertices for polygon in mesh.polygons],
    )


def _reach(surface, origin, direction, missing=0.0):
    distance = surface.ray_cast(origin, direction)[3]
    return missing if distance is None else distance


def _blanket(root, wool, spine, middle, hem, cloth, border):
    """A woven blanket thrown over the back: a thick plate that follows the
    wool over the spine, then drops straight down each side and hangs free
    below the belly, where a rolled hem of the border colour weighs it
    down. It is a cloth laid on, not a band round the barrel."""

    half = 0.16
    _saddle("blanket", wool, spine, middle - half, middle + half, hem, BLANKET, cloth, root, rows=8)

    for side in (1, -1):
        edge = Vector((side * (_reach(wool, Vector((0, middle, spine)), Vector((side, 0, 0))) + BLANKET + 0.012), 0, hem + 0.012))
        rings = []

        for y in (middle - half - 0.012, middle + half + 0.012):
            centre = edge + Vector((0, y, 0))
            rings.append([centre + Vector((side * dx, 0, dz)) for dx, dz in ((-0.024, -0.016), (0.012, -0.022), (0.024, 0.0), (0.012, 0.024), (-0.024, 0.02))])

        roll = core.loft(f"blanket_hem_{'l' if side > 0 else 'r'}", rings, border, root, cap_start=True, cap_end=True)
        creature.finish(core.shade_smooth(roll, angle=50))


def _saddle(name, wool, spine, front, back, hem, lift, mat, parent, rows=8, across=8):
    sunk = 0.022
    rings = []

    for row in range(rows + 1):
        origin = Vector((0, front + (back - front) * row / rows, spine))
        outer = []
        inner = []

        for step in range(-across, across + 1):
            angle = step / across * math.pi / 2
            direction = Vector((math.sin(angle), 0, math.cos(angle)))
            reach = _reach(wool, origin, direction) + lift

            outer.append(origin + direction * reach)
            inner.append(origin + direction * (reach - sunk))

        for points in (outer, inner):
            right = points[0]
            left = points[-1]

            # Straight down from the widest point, flaring a little as it
            # falls, to the hem below the belly.
            for drop in (0.35, 0.7, 1.0):
                level = spine + (hem - spine) * drop
                flare = 0.012 * drop

                points.insert(0, Vector((right.x - flare, origin.y, level)))
                points.append(Vector((left.x + flare, origin.y, level)))

        rings.append(outer + inner[::-1])

    cloth = core.loft(name, rings, mat, parent, closed=True, cap_start=True, cap_end=True)
    core.shade_smooth(cloth, angle=40)

    return creature.finish(cloth)


def _leg(root, name, hip, floor, body, detail):
    """A leg that tapers the way a llama's does: a woolly thigh, a knee,
    and a narrow cannon down to a fetlock over a cloven hoof."""

    leg = creature.pivot(f"leg_{name}", root, hip)
    ground = floor - hip.z

    creature.shell(
        f"leg_wool_{name}",
        [
            ((0, 0.0, 0.1), 0.05, 0.05, 0.05, 2.4),
            ((0, 0.0, 0.02), 0.088, 0.098, 0.098, 2.8),
            ((0, 0.004, -0.1), 0.076, 0.084, 0.084, 3.0),
            ((0, 0.004, -0.19), 0.058, 0.062, 0.064, 3.0),
            ((0, 0.0, -0.26), 0.046, 0.05, 0.05, 2.8),
        ],
        body,
        leg,
        sides=12,
        start=0.0,
        end=0.0,
        crease=40,
    )
    creature.shell(
        f"shin_{name}",
        [
            ((0, 0.0, -0.24), 0.045, 0.048, 0.048, 2.6),
            ((0, 0.0, -0.3), 0.04, 0.042, 0.042, 2.6),
            ((0, -0.002, -0.38), 0.031, 0.033, 0.033, 2.6),
            ((0, -0.005, ground + 0.1), 0.029, 0.031, 0.031, 2.6),
            ((0, -0.009, ground + 0.07), 0.037, 0.04, 0.038, 2.6),
            ((0, -0.012, ground + 0.045), 0.034, 0.036, 0.034, 2.4),
        ],
        body,
        leg,
        sides=10,
        start=0.0,
        end=0.0,
        crease=40,
    )

    # Sculpted cloven llama hoof standing flat on the ground.
    creature.facet(
        f"hoof_{name}",
        [
            ("sphere", (0.023, -0.022, ground + 0.036), (0.03, 0.064, 0.036)),
            ("sphere", (-0.023, -0.022, ground + 0.036), (0.03, 0.064, 0.036)),
            ("cube", (0.0, -0.02, ground + 0.017), (0.042, 0.056, 0.017)),
        ],
        detail,
        leg,
        triangles=180,
        voxel=0.009,
        flat=12.0,
    )


def _neck(root, brow, throat, spine, body, detail, dark, accent):
    """Graceful S-curved woolly llama neck with organic fleece ripples instead of stacked balls."""

    girth = 0.118
    axis = throat.y + girth
    neck_at = Vector((0, axis, spine + 0.08))
    # The skull sits low enough that the forehead, with the brow seal on
    # it, shows above the bridge of the muzzle from the front.
    skull_at = Vector((0, brow.y + 0.103, brow.z - 0.045))
    head_at = Vector((0, skull_at.y + 0.01, skull_at.z - 0.085))
    neck = creature.pivot("neck", root, neck_at)

    # The neck: a column of a few broad planes, boxy enough that each side
    # prints as one tone, leaning forward at the chest and back under the
    # head, its front flat where the throat seal sits.
    top = head_at.z - neck_at.z + 0.06
    wool = creature.shell(
        "neck_wool",
        [
            ((0, 0.02, -0.06), 0.145, 0.14, 0.14, 2.6),
            ((0, 0.01, 0.06), 0.13, 0.124, 0.13, 3.0),
            ((0, 0.0, 0.2), 0.112, 0.112, 0.116, 3.0),
            ((0, 0.0, 0.33), 0.1, 0.098, 0.1, 3.0),
            ((0, 0.01, top - 0.08), 0.092, 0.088, 0.092, 2.8),
            ((0, 0.02, top), 0.078, 0.072, 0.078, 2.4),
        ],
        body,
        neck,
        sides=10,
        start=0.0,
        end=0.03,
        crease=40,
    )

    _neckerchief(creature.pivot("trait_2", neck, (0, 0, 0)), _surface(wool), throat.z + 0.17 - neck_at.z, detail)
    head = _head(neck, neck_at, head_at, skull_at, body, detail, dark, accent)
    _grow(head, brow, HEAD_GROWTH)


def _grow(head, about, factor):
    """Grows the head and everything on it by `factor` about the point
    `about` (the brow seal, which stays put). Pivots are moved, never
    scaled: each mesh's own vertices grow, and each pivot's place."""

    head.location = Vector(about) - creature.origin(head.parent) + (creature.origin(head) - Vector(about)) * factor

    for node in head.children_recursive:
        node.location = Vector(node.location) * factor

        if node.type == "MESH":
            node.data.transform(Matrix.Scale(factor, 4))


def _head(neck, neck_at, head_at, skull_at, body, detail, dark, accent):
    head = creature.pivot("head", neck, head_at - neck_at)
    _, y, z = skull_at

    def feature(name, at, size, mat, triangles, rotation=(0, 0, 0)):
        creature.form(name, _about(head_at, [("sphere", at, size, rotation)]), mat, head, triangles=triangles)

    # Sculpted expressive llama skull: soft woolly cheeks, fluffy brow topknot,
    # clean tapered bridge, and gentle cleft upper muzzle.
    creature.facet(
        "skull",
        _about(
            head_at,
            [
                ("sphere", (0, y, z), (0.108, 0.106, 0.096)),
                ("sphere", (0.074, y + 0.038, z - 0.044), (0.052, 0.054, 0.050)),
                ("sphere", (-0.074, y + 0.038, z - 0.044), (0.052, 0.054, 0.050)),
                # Fluffy woolly crown tuft.
                ("sphere", (0, y - 0.008, z + 0.094), (0.054, 0.056, 0.042)),
                ("sphere", (0.034, y - 0.032, z + 0.084), (0.038, 0.038, 0.036)),
                ("sphere", (-0.034, y - 0.032, z + 0.084), (0.038, 0.038, 0.036)),
                # Tapered llama nose bridge & soft cleft upper lip pads.
                ("sphere", (0, y - 0.115, z - 0.032), (0.056, 0.086, 0.050)),
                ("sphere", (0.022, y - 0.178, z - 0.048), (0.034, 0.036, 0.034)),
                ("sphere", (-0.022, y - 0.178, z - 0.048), (0.034, 0.036, 0.034)),
            ],
        ),
        body,
        head,
        triangles=1100,
        voxel=0.011,
        flat=10.0,
    )

    for side in (1, -1):
        # Graceful curved banana-shaped llama ears lofted with clean taper.
        yaw = math.radians(side * 30)
        ear_rings = []
        ear_stops = [
            (0.0, 0.068, 0.066, 0.048, 0.034, 0.020),
            (0.35, 0.090, 0.068, 0.100, 0.038, 0.018),
            (0.70, 0.092, 0.068, 0.152, 0.030, 0.014),
            (0.92, 0.076, 0.064, 0.184, 0.016, 0.010),
            (1.0, 0.066, 0.060, 0.194, 0.006, 0.005),
        ]
        cy_w, sy_w = math.cos(yaw), math.sin(yaw)
        for _, ex, ey, ez, rx, ry in ear_stops:
            ring = []
            for s in range(14):
                a = math.tau * s / 14
                lx = math.cos(a) * rx
                # Concave front ear groove!
                ly = math.sin(a) * ry + (0.007 * max(0.0, -math.sin(a)) if rx > 0.012 else 0.0)
                rx_rot = lx * cy_w - ly * sy_w
                ry_rot = lx * sy_w + ly * cy_w
                pt = Vector((side * ex + rx_rot, y + ey + ry_rot, z + ez)) - head_at
                ring.append(pt)
            ear_rings.append(ring)
        ear = core.loft(f"ear_{side}", ear_rings, body, head, cap_start=True, cap_end=True)
        creature.finish(core.shade_smooth(ear))

        lean = (0, math.radians(side * 4), yaw)
        feature(f"ear_hollow_{side}", (side * 0.090, y + 0.058, z + 0.118), (0.019, 0.010, 0.046), dark, 60, lean)

        turn = (0, 0, math.radians(side * 41))
        feature(f"socket_{side}", (side * 0.072, y - 0.078, z + 0.012), (0.030, 0.016, 0.035), dark, 120, turn)
        creature.eye(side, head, Vector((side * 0.079, y - 0.087, z + 0.012)) - head_at, (0.012, 0.009, 0.012), accent, turn)
        creature.brow(side, head, Vector((side * 0.074, y - 0.086, z + 0.052)) - head_at, (0.030, 0.012, 0.010), dark, turn)
        feature(f"nostril_{side}", (side * 0.020, y - 0.208, z - 0.030), (0.010, 0.008, 0.008), dark, 40)

    chin = (0, y - 0.115, z - 0.091)
    jaw_at = Vector((0, y - 0.07, z - 0.07))
    jaw = creature.pivot("jaw", head, jaw_at - head_at)

    feature("mouth", chin, (0.034, 0.052, 0.016), dark, 80)
    # The lower jaw: a heavy chin with a jowl to either side, jutting past
    # the upper lip in an underbite.
    jut = (0, chin[1] - 0.058, chin[2] + 0.004)
    creature.facet(
        "jaw_lower",
        _about(
            jaw_at,
            [
                ("sphere", chin, (0.052, 0.074, 0.036)),
                ("sphere", (0.034, chin[1] + 0.03, chin[2] + 0.004), (0.03, 0.05, 0.032)),
                ("sphere", (-0.034, chin[1] + 0.03, chin[2] + 0.004), (0.03, 0.05, 0.032)),
                ("sphere", jut, (0.042, 0.044, 0.032)),
            ],
        ),
        body,
        jaw,
        triangles=320,
        voxel=0.009,
        flat=10.0,
    )

    # The underbite: two flat front teeth standing up out of the lower jaw
    # past the lip, a llama's sass.
    teeth = []

    for side in (1, -1):
        root_at = Vector((side * 0.012, jut[1] - 0.03, jut[2] + 0.01)) - jaw_at
        teeth.append(core.tube(f"tooth_{side}", [root_at, root_at + Vector((0, -0.004, 0.036))], lambda t: 0.012 - 0.002 * t, detail, sides=6, flatten=0.55))

    incisors = core.join(teeth, "teeth")
    incisors.parent = jaw
    creature.finish(core.shade_smooth(incisors, angle=40))

    _cowboy_hat(creature.pivot("trait_1", head, (0, 0, 0)), Vector((0, y - 0.02, z + 0.09)) - head_at, detail, core.material("trim"))

    return head


def _cowboy_hat(parent, at, felt, band):
    """Crisp creased Stetson cowboy hat with pinched front crown and rolled brim."""

    crown = 0.07
    brim = 0.145

    def shape(angle, radius, height):
        side = math.cos(angle) ** 2
        front = max(0.0, -math.sin(angle))
        out = max(0.0, radius - crown) / (brim - crown)
        up = max(0.0, height - 0.03) / 0.09

        if height > 0.09:
            height = 0.092 + 0.026 * min(1.0, abs(math.cos(angle)) * radius / 0.058) ** 2

        return (
            radius * (1 + 0.11 * (1 - side)) * (1 - 0.32 * front * front * up),
            height + out * out * (0.045 * side - 0.014 * (1 - side)),
        )

    hat = core.revolve(
        "hat",
        [
            (0.012, 0.100),
            (0.040, 0.104),
            (0.058, 0.119),
            (0.066, 0.104),
            (0.070, 0.050),
            (0.072, 0.010),
            (0.088, 0.005),
            (0.118, 0.004),
            (0.145, 0.006),
            (0.148, 0.000),
            (0.145, -0.006),
            (0.088, -0.006),
            (0.055, -0.004),
        ],
        felt,
        parent,
        segments=36,
        shape=shape,
        cap_top=True,
    )
    ribbon = core.revolve(
        "hat_band",
        [(0.071, 0.010), (0.078, 0.014), (0.078, 0.035), (0.071, 0.039)],
        band,
        parent,
        segments=36,
        shape=shape,
    )

    for part in (hat, ribbon):
        part.location = at
        creature.finish(core.shade_smooth(part, angle=52))


def _neckerchief(parent, wool, height, cloth):
    steps = 30
    rings = []

    def lay(direction, z, lift):
        origin = Vector((0, 0, z))
        return origin + direction * (_reach(wool, origin, direction, missing=0.09) + lift)

    for step in range(steps + 1):
        angle = math.pi / 2 + step / steps * math.tau
        direction = Vector((math.cos(angle), math.sin(angle), 0))
        facing = math.acos(max(-1.0, min(1.0, -direction.y)))
        point = 0.125 * max(0.0, 1 - facing / math.radians(70))
        top = height + 0.028
        bottom = height - 0.028 - point

        rings.append(
            [
                lay(direction, top - 0.004, -0.02),
                *(lay(direction, top + (bottom - top) * fold / 3, 0.013) for fold in range(4)),
                lay(direction, bottom + 0.004, -0.02),
            ]
        )

    creature.finish(core.shade_smooth(core.loft("neckerchief", rings, cloth, parent, closed=True), angle=50))

    back = _reach(wool, Vector((0, 0, height)), Vector((0, 1, 0)), missing=0.09)

    creature.form(
        "neckerchief_knot",
        [
            ("sphere", (0, back + 0.018, height), (0.030, 0.022, 0.026)),
            ("sphere", (0.032, back + 0.022, height - 0.048), (0.017, 0.011, 0.044), (0, math.radians(-26), 0)),
            ("sphere", (-0.032, back + 0.022, height - 0.048), (0.017, 0.011, 0.044), (0, math.radians(26), 0)),
        ],
        cloth,
        parent,
        triangles=240,
        voxel=0.008,
    )
