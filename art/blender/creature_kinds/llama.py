"""The llama: Sollamas.

A cartoon llama, and a llama is mostly neck: a tall column of wool rising
from the front of a small round cloud of a body, a little head with a blunt
muzzle carried high on top of it, and two upright ears above that. The coat
is drawn as rolls and puffs of wool rather than as hair, the legs are slim
below a woolly top, and a striped blanket lies over the back.

It wears a cowboy hat or a neckerchief.
"""

import math

from mathutils import Vector
from mathutils.bvhtree import BVHTree

from lib import core, creature

# How far the blanket, and the stripes on it, stand off the wool under them.
BLANKET = 0.016
STRIPE = 0.024


def build(spec):
    root = core.empty("husk")
    body = core.material("body")
    trim = core.material("trim")
    detail = core.material("detail")
    dark = core.material("dark")
    accent = core.material("accent")
    floor = creature.floor(spec)

    # The body is laid out from the seals, so that every stud ends up on a
    # surface: the brow, the front of the neck, the two flanks and the rump.
    brow, throat, flank, _, rump = (creature.to_blender(seal["offset"]) for seal in spec["seals"])

    # The flank studs sit on a stripe of the blanket, so the wool under it
    # stops that much short of them.
    spine = flank.z
    wide = flank.x - STRIPE

    # Where the front and the back legs stand. The chest is over the front
    # pair, under the neck.
    shoulder = throat.y + 0.14
    hip = rump.y + 0.03

    barrel = ((0, flank.y + 0.01, spine - 0.015), (wide, 0.42, 0.21))
    haunch = ((0, rump.y - 0.01, spine), (wide - 0.008, 0.19, rump.z - spine + 0.005))
    chest = ((0, shoulder, spine - 0.01), (0.175, 0.15, 0.2))
    hem = spine - 0.12

    torso = creature.form(
        "torso",
        [
            # The barrel, a rump that tops out at its seal, and a deep chest.
            ("sphere", *barrel),
            ("sphere", *haunch),
            ("sphere", *chest),
            # Puffs of wool where the blanket does not lie: down the chest and
            # round the back of the rump. The top of the rump is left smooth
            # for its seal to sit on.
            *_fleece(*chest, 14, 0.085, lambda at: at.y < shoulder + 0.02),
            *_fleece(*haunch, 14, 0.085, lambda at: at.y > rump.y - 0.02 and (at - rump).length > 0.11),
        ],
        body,
        root,
        triangles=2800,
    )

    _blanket(root, _surface(torso), spine, flank.y, hem, trim, detail)

    # A short tail, held up, with a tuft at the end of it.
    tail_at = Vector((0, rump.y + 0.16, rump.z - 0.1))
    tail = creature.pivot("tail", root, tail_at)

    creature.form(
        "tail_tuft",
        [
            ("sphere", (0, 0.045, 0.025), (0.055, 0.06, 0.06)),
            ("sphere", (0, 0.085, 0.075), (0.045, 0.04, 0.05)),
        ],
        body,
        tail,
        triangles=320,
    )

    # The legs hang from inside the body, so that a swing never opens a gap
    # at the top of one.
    for side, name in ((1, "l"), (-1, "r")):
        for end, at in (("f", shoulder), ("b", hip)):
            _leg(root, f"{end}{name}", Vector((side * 0.115, at, spine - 0.14)), floor, body, detail)

    _neck(root, brow, throat, spine, body, detail, dark, accent)

    creature.studs(root, spec)
    creature.shadow(root, spec)

    return root


def _about(origin, blobs):
    """`blobs` placed in the creature's own space, moved into the space of a
    pivot at `origin`, so that every part can be laid out against the seals."""

    return [(kind, tuple(Vector(at) - Vector(origin)), *rest) for kind, at, *rest in blobs]


def _fleece(centre, radii, count, size, keep):
    """Puffs of wool scattered evenly over an ellipsoid: spheres sunk most of
    the way into its surface, which fuse with it into a fleece. `keep(at)`
    says whether a puff belongs at a place.

    Only the creature's left side is scattered. The right is its mirror
    image, because a fleece that is lopsided looks like a mistake.
    """

    turn = math.pi * (3 - math.sqrt(5))
    puffs = []

    for index in range(count):
        height = 1 - 2 * (index + 0.5) / count
        ring = math.sqrt(1 - height * height)
        out = (math.cos(index * turn) * ring, math.sin(index * turn) * ring, height)
        at = Vector(centre) + Vector([way * (radius - size * 0.68) for way, radius in zip(out, radii)])

        if out[0] < 0 or not keep(at):
            continue

        for side in (1, -1):
            puffs.append(("sphere", (side * at.x, at.y, at.z), (size, size, size)))

    return puffs


def _surface(part):
    """A part's surface as something a ray can be cast at, in the space of
    the part's parent. Cloth is laid on a body by finding the wool under it."""

    mesh = part.data

    return BVHTree.FromPolygons(
        [vertex.co + part.location for vertex in mesh.vertices],
        [polygon.vertices for polygon in mesh.polygons],
    )


def _reach(surface, origin, direction, missing=0.0):
    """How far it is from `origin`, inside a part, out to its surface."""

    distance = surface.ray_cast(origin, direction)[3]

    return missing if distance is None else distance


def _blanket(root, wool, spine, middle, hem, cloth, stripe):
    """The saddle-cloth, and three broad stripes across it. The middle stripe
    runs under the flank seals."""

    half = 0.19

    _saddle("blanket", wool, spine, middle - half, middle + half, hem, BLANKET, cloth, root, rows=8)

    # A stripe is the same cloth again, narrower and standing a little
    # further off, and its hem hangs a hair lower so the two never share a
    # face.
    for index, offset in enumerate((-0.125, 0.0, 0.125)):
        at = middle + offset

        _saddle(f"blanket_stripe_{index}", wool, spine, at - 0.027, at + 0.027, hem - 0.004, STRIPE, stripe, root, rows=2)


def _saddle(name, wool, spine, front, back, hem, lift, mat, parent, rows=8, across=6):
    """A cloth lying over the back from `front` to `back`, and hanging down
    each side to the height `hem`.

    It follows the wool under it, which is found by casting rays out from
    the body's axis at the height `spine`, and stands `lift` off it. Below
    the widest part of the body it hangs free rather than tucking under, as
    cloth does. Its underside is sunk into the body, so that no edge shows a
    gap.
    """

    sunk = 0.03
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

        # The two ends hang free, the right one first and then the left.
        # Hanging cloth swings in toward the body under it, but not all the
        # way. The body is the same on both sides, so one ray measures both.
        for points in (outer, inner):
            right = points[0]
            left = points[-1]

            for drop in (0.5, 1.0):
                level = Vector((0, origin.y, spine + (hem - spine) * drop))
                tuck = 0.6 * (_reach(wool, origin, Vector((1, 0, 0))) - _reach(wool, level, Vector((1, 0, 0))))

                points.insert(0, Vector((right.x + tuck, origin.y, level.z)))
                points.append(Vector((left.x - tuck, origin.y, level.z)))

        rings.append(outer + inner[::-1])

    cloth = core.loft(name, rings, mat, parent, closed=True, cap_start=True, cap_end=True)
    core.shade_smooth(cloth, angle=50)

    return creature.finish(cloth)


def _leg(root, name, hip, floor, body, detail):
    """One leg, hanging from a pivot at `hip`: a thick woolly top that ends
    in a cuff, a narrow bare shin under it, and a hoof."""

    leg = creature.pivot(f"leg_{name}", root, hip)
    ground = floor - hip.z

    creature.form(
        f"leg_wool_{name}",
        [
            ("sphere", (0, 0, -0.1), (0.076, 0.092, 0.14)),
            ("sphere", (0, 0, -0.23), (0.07, 0.08, 0.06)),
        ],
        body,
        leg,
        triangles=420,
    )

    # The shin is a plain tapering post, so it is swept rather than sculpted:
    # sculpted this thin, it comes out ridged.
    shin = [(0, 0, -0.22), (0, -0.004, -0.36), (0, -0.01, ground + 0.05)]

    creature.finish(core.tube(f"shin_{name}", shin, lambda along: 0.044 - 0.009 * along, body, leg, sides=10))

    # The hoof is flat underneath, so the creature stands on the floor
    # rather than balancing on four points.
    creature.form(
        f"hoof_{name}",
        [
            ("sphere", (0, -0.015, ground + 0.04), (0.045, 0.06, 0.04)),
            ("cube", (0, -0.015, ground + 0.016), (0.034, 0.048, 0.016)),
        ],
        detail,
        leg,
        triangles=160,
    )


def _neck(root, brow, throat, spine, body, detail, dark, accent):
    """The neck, and the head it carries. The neck is a stack of rolls of
    wool, each one a pair of puffs, set side by side and then one behind the
    other in turn. That gives its outline the scalloped edge of a fleece from
    the front and from the side, without it turning into a stack of rings."""

    girth = 0.118
    lean = 0.024
    rise = 0.088
    axis = throat.y + girth
    neck_at = Vector((0, axis, spine + 0.08))
    skull_at = Vector((0, brow.y + 0.103, brow.z - 0.02))
    head_at = Vector((0, skull_at.y + 0.01, skull_at.z - 0.085))
    neck = creature.pivot("neck", root, neck_at)

    # Thick where it leaves the chest, slimmer under the head.
    column = [
        ("sphere", (0, axis + 0.01, neck_at.z + 0.01), (0.135, 0.13, 0.1)),
        ("sphere", (0, axis, (neck_at.z + head_at.z) / 2), (0.08, 0.08, 0.3)),
    ]

    for index in range(-2, 4):
        # Roll 0 is the one the throat seal sits on, so it is level with the
        # seal and its puffs are the ones set fore and aft.
        height = throat.z + index * rise
        size = girth + (throat.z - height) * 0.09 - lean
        across, along = (lean, 0) if index % 2 else (0, lean)

        for way in (1, -1):
            column.append(("sphere", (way * across, axis + way * along, height), (size, size, 0.072)))

    wool = creature.form("neck_wool", _about(neck_at, column), body, neck, triangles=1700)

    # The neckerchief is tied high, clear of the throat seal.
    _neckerchief(creature.pivot("trait_2", neck, (0, 0, 0)), _surface(wool), throat.z + 0.17 - neck_at.z, detail)
    _head(neck, neck_at, head_at, skull_at, body, detail, dark, accent)


def _head(neck, neck_at, head_at, skull_at, body, detail, dark, accent):
    """The head, on a pivot at the top of the neck, and everything on it:
    ears, eyes, the jaw and the hat."""

    head = creature.pivot("head", neck, head_at - neck_at)
    _, y, z = skull_at

    def feature(name, at, size, mat, triangles, rotation=(0, 0, 0)):
        """One small part of the face, made of a single blob."""

        creature.form(name, _about(head_at, [("sphere", at, size, rotation)]), mat, head, triangles=triangles)

    # A round skull with woolly cheeks and a topknot, and a blunt muzzle that
    # droops a little toward the nose, which is what a llama's face does.
    creature.form(
        "skull",
        _about(
            head_at,
            [
                ("sphere", (0, y, z), (0.11, 0.108, 0.098)),
                ("sphere", (0.076, y + 0.04, z - 0.045), (0.05, 0.054, 0.05)),
                ("sphere", (-0.076, y + 0.04, z - 0.045), (0.05, 0.054, 0.05)),
                ("sphere", (0, y - 0.005, z + 0.095), (0.052, 0.056, 0.04)),
                ("sphere", (0.032, y - 0.03, z + 0.086), (0.036, 0.036, 0.034)),
                ("sphere", (-0.032, y - 0.03, z + 0.086), (0.036, 0.036, 0.034)),
                ("sphere", (0, y - 0.12, z - 0.035), (0.058, 0.088, 0.054)),
                ("sphere", (0, y - 0.17, z - 0.044), (0.047, 0.043, 0.043)),
            ],
        ),
        body,
        head,
        triangles=1500,
    )

    for side in (1, -1):
        # Upright ears shaped like bananas: out from the skull, then curving
        # back in toward each other at the tips. They are flat, and turned a
        # little outward so that they show from the side as well as the front.
        yaw = math.radians(side * 32)

        creature.form(
            f"ear_{side}",
            _about(
                head_at,
                [
                    ("sphere", (side * 0.072, y + 0.07, z + 0.07), (0.036, 0.022, 0.05), (0, math.radians(side * 14), yaw)),
                    ("sphere", (side * 0.09, y + 0.07, z + 0.12), (0.036, 0.02, 0.045), (0, 0, yaw)),
                    ("sphere", (side * 0.082, y + 0.07, z + 0.165), (0.02, 0.015, 0.032), (0, math.radians(side * -22), yaw)),
                ],
            ),
            body,
            head,
            triangles=260,
        )

        # The hollow of the ear, which is what makes it an ear and not a horn.
        lean = (0, math.radians(side * 4), yaw)

        feature(f"ear_hollow_{side}", (side * 0.093, y + 0.059, z + 0.112), (0.019, 0.011, 0.044), dark, 60, lean)

        # What is left of its eyes: dark sockets with a light far back in
        # them, set where the skull turns from the front to the side so they
        # show from both.
        turn = (0, 0, math.radians(side * 41))

        feature(f"socket_{side}", (side * 0.072, y - 0.078, z + 0.012), (0.03, 0.016, 0.035), dark, 120, turn)
        creature.eye(side, head, Vector((side * 0.079, y - 0.087, z + 0.012)) - head_at, (0.012, 0.009, 0.012), accent, turn)
        # A tuft of dark wool over each eye, which is what lets the face
        # knit and lift.
        creature.brow(side, head, Vector((side * 0.074, y - 0.086, z + 0.052)) - head_at, (0.03, 0.012, 0.01), dark, turn)
        feature(f"nostril_{side}", (side * 0.019, y - 0.207, z - 0.03), (0.01, 0.008, 0.008), dark, 40)

    # The lower jaw is a chin under the muzzle, hinged at the back so that it
    # drops when the creature shrieks. The inside of the mouth is hidden in
    # it until it does.
    chin = (0, y - 0.115, z - 0.091)
    jaw_at = Vector((0, y - 0.07, z - 0.07))
    jaw = creature.pivot("jaw", head, jaw_at - head_at)

    feature("mouth", chin, (0.032, 0.05, 0.014), dark, 80)
    creature.form("jaw_lower", _about(jaw_at, [("sphere", chin, (0.047, 0.066, 0.03))]), body, jaw, triangles=220)

    # The hat covers the topknot. The ears stand up through the back of its
    # brim.
    _cowboy_hat(creature.pivot("trait_1", head, (0, 0, 0)), Vector((0, y - 0.02, z + 0.09)) - head_at, detail, core.material("trim"))


def _cowboy_hat(parent, at, felt, band):
    """A plain cowboy hat, with its base at `at`: a creased crown pinched in
    at the front, and a wide brim rolled up at the sides. It sits just in
    front of the ears."""

    crown = 0.07
    brim = 0.14

    def shape(angle, radius, height):
        side = math.cos(angle) ** 2
        front = max(0.0, -math.sin(angle))
        out = max(0.0, radius - crown) / (brim - crown)
        up = max(0.0, height - 0.03) / 0.09

        # The top of the crown is creased from front to back: low along the
        # middle, high at the two sides.
        if height > 0.09:
            height = 0.094 + 0.024 * min(1.0, abs(math.cos(angle)) * radius / 0.058) ** 2

        # The hat is longer than it is wide, the crown narrows toward the
        # front as it rises, and the brim rolls up at the sides and dips a
        # little fore and aft.
        return (
            radius * (1 + 0.1 * (1 - side)) * (1 - 0.3 * front * front * up),
            height + out * out * (0.04 * side - 0.012 * (1 - side)),
        )

    hat = core.revolve(
        "hat",
        [
            (0.012, 0.1),
            (0.04, 0.104),
            (0.058, 0.118),
            (0.066, 0.104),
            (0.07, 0.05),
            (0.072, 0.01),
            (0.085, 0.005),
            (0.115, 0.004),
            (0.14, 0.006),
            (0.143, 0.0),
            (0.14, -0.006),
            (0.085, -0.006),
            (0.055, -0.004),
        ],
        felt,
        parent,
        segments=20,
        shape=shape,
        cap_top=True,
    )
    ribbon = core.revolve(
        "hat_band",
        [(0.071, 0.01), (0.077, 0.014), (0.077, 0.034), (0.071, 0.038)],
        band,
        parent,
        segments=20,
        shape=shape,
    )

    for part in (hat, ribbon):
        part.location = at
        creature.finish(part)


def _neckerchief(parent, wool, height, cloth):
    """A square of cloth folded to a triangle and tied round the neck at
    `height`: a band with its point hanging down the front, and a knot behind.

    Like the blanket it is laid on the wool by casting rays, here out from
    the neck's axis, which the neck's pivot stands on.
    """

    steps = 24
    rings = []

    def lay(direction, z, lift):
        origin = Vector((0, 0, z))

        return origin + direction * (_reach(wool, origin, direction, missing=0.09) + lift)

    for step in range(steps + 1):
        # Starting from the back, so that the seam is under the knot.
        angle = math.pi / 2 + step / steps * math.tau
        direction = Vector((math.cos(angle), math.sin(angle), 0))
        facing = math.acos(max(-1.0, min(1.0, -direction.y)))
        point = 0.12 * max(0.0, 1 - facing / math.radians(70))
        top = height + 0.028
        bottom = height - 0.028 - point

        # Each ring is a slice through the cloth: its two edges are sunk into
        # the wool, and four points between them lie on it.
        rings.append(
            [
                lay(direction, top - 0.004, -0.02),
                *(lay(direction, top + (bottom - top) * fold / 3, 0.012) for fold in range(4)),
                lay(direction, bottom + 0.004, -0.02),
            ]
        )

    creature.finish(core.loft("neckerchief", rings, cloth, parent, closed=True))

    back = _reach(wool, Vector((0, 0, height)), Vector((0, 1, 0)), missing=0.09)

    creature.form(
        "neckerchief_knot",
        [
            ("sphere", (0, back + 0.018, height), (0.03, 0.022, 0.026)),
            ("sphere", (0.03, back + 0.02, height - 0.045), (0.018, 0.012, 0.04), (0, math.radians(-28), 0)),
            ("sphere", (-0.03, back + 0.02, height - 0.045), (0.018, 0.012, 0.04), (0, math.radians(28), 0)),
        ],
        cloth,
        parent,
        triangles=240,
    )
