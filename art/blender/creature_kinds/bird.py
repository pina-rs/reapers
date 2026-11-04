"""The bird: Thugbirdz.

A tough little street bird caught mid-beat: a round head sunk into heavy
shoulders, a thick neck, a barrel of a body with the chest pushed out and a
short fanned tail trailing low behind it. It never lands, so its feet are
balled up under it and the wings are the parts that matter most: two chunky
blades of feathers, held out to the sides in a shallow M so that it reads as
a bird in flight from straight ahead, which is where the player meets it.

Its face is a wrap-around visor. The dark band is where its eyes were, and
the one thin strip of light in it is all that is left of them.

It wears a cap turned backwards or a chain.
"""

import math

import bpy
from mathutils import Matrix, Vector

from lib import core, creature

# The skull, measured from the head pivot. The pivot is straight under the
# crown, so the seal there stays on the head however far the head turns.
SKULL = Vector((0, -0.01, 0.085))
SKULL_SIZE = Vector((0.135, 0.14, 0.135))


def build(spec):
    root = core.empty("husk")
    body = core.material("body")
    trim = core.material("trim")
    detail = core.material("detail")
    dark = core.material("dark")
    accent = core.material("accent")

    # A barrel, leaning back so the chest leads and the rump hangs low, with
    # a hunch that runs from behind the head to the root of the tail and a
    # shoulder each side of the neck. The shoulders are the widest part of
    # the bird and sit level with its chin: that, and the high back, are
    # what make it look hunched.
    torso = creature.form(
        "torso",
        [
            ("sphere", (0, 0.045, -0.07), (0.2, 0.225, 0.185), (math.radians(-12), 0, 0)),
            ("sphere", (0, 0.11, 0.0), (0.165, 0.155, 0.13)),
            ("sphere", (0.12, 0.02, 0.03), (0.085, 0.09, 0.075)),
            ("sphere", (-0.12, 0.02, 0.03), (0.085, 0.09, 0.075)),
        ],
        body,
        root,
        triangles=1700,
    )

    # The second colour: the belly, and above it the thick ruff of a neck
    # that the head sits in. The ruff goes all the way round, so the neck
    # shows from the side and from behind as well. It is remeshed coarsely
    # because a fine mesh cut down this far comes out lumpy.
    front = creature.form(
        "front",
        [
            ("sphere", (0, -0.05, -0.1), (0.155, 0.145, 0.15)),
            *(("sphere", _collar(math.tau * step / 30), (0.045, 0.045, 0.045)) for step in range(30)),
        ],
        trim,
        root,
        triangles=1500,
        voxel=0.016,
    )

    _tail(root, body)
    _feet(root, detail)
    _head(root, body, detail, dark, accent)

    for side, name in ((1, "l"), (-1, "r")):
        _wing(root, side, name, body)

    _chain(creature.pivot("trait_2", root, (0, 0, 0)), detail, (torso, front))

    creature.studs(root, spec)

    return root


def _collar(angle):
    """The line the neck follows around the foot of the skull: lowest on the
    chest, straight ahead at angle 0, and highest behind the head. It stays
    low until it is well round to the back, so the beak clears it when the
    head turns."""

    ahead = ((1 + math.cos(angle)) / 2) ** 0.6

    return Vector((math.sin(angle) * 0.138, -0.045 - math.cos(angle) * 0.111, 0.1 - 0.14 * ahead))


def _along(direction, up=(0, 0, 1)):
    """The rotation that lays a blob's long (Y) axis along `direction` with
    its flat (Z) side facing `up` as nearly as it can."""

    forward = Vector(direction).normalized()
    across = forward.cross(Vector(up)).normalized()
    normal = across.cross(forward)

    return Matrix((across, forward, normal)).transposed().to_euler()


def _feather(base, direction, length, width, thickness=0.015, up=(0, 0, 1)):
    """One flat feather growing from `base` along `direction`."""

    heading = Vector(direction).normalized()

    return ("sphere", Vector(base) + heading * (length / 2 - 0.02), (width, length / 2, thickness), _along(heading, up))


def _plumage(name, blobs, mat, parent, triangles):
    """A part made of feathers. They are thin, and cutting a thin part down
    to its triangle count can fold a pair of faces flat onto each other,
    which the exporter then warns about. Those pairs are dropped here."""

    part = creature.form(name, blobs, mat, parent, triangles=triangles)
    part.data.validate()

    return part


def _wing(root, side, name, feathers):
    # The pivot is just inside the shoulder, behind and below the seal, so
    # the seal sits on the front of the wing root and stays there as the
    # wing beats.
    wing = creature.pivot(f"wing_{name}", root, (side * 0.17, 0.052, 0.05))

    def at(x, y, z):
        return Vector((side * x, y, z))

    # The arm runs up and out to the wrist, then dips to the tip: half of
    # the M a bird in flight draws against the sky. It is close to level
    # overall, because the game beats it both ways from here.
    shoulder, wrist, tip = at(0, 0, 0), at(0.15, -0.03, 0.1), at(0.25, 0.01, 0.08)
    blobs = []

    for start, end, girth in ((shoulder, wrist, 0.042), (wrist, tip, 0.03)):
        blobs.append(("sphere", (start + end) / 2, (girth * 1.25, (end - start).length / 2 + girth, girth), _along(end - start)))

    # The flight feathers hang back and down from the arm, so the wing shows
    # its face to someone in front of it and is still a blade the game can
    # beat. Toward the tip they fan outward into fingers.
    lengths = (0.16, 0.19, 0.21, 0.22, 0.22, 0.21, 0.19, 0.16)

    for index, length in enumerate(lengths):
        along = index / (len(lengths) - 1)
        reach = 0.12 + 0.88 * along
        base = shoulder.lerp(wrist, reach / 0.55) if reach < 0.55 else wrist.lerp(tip, (reach - 0.55) / 0.45)
        fan = math.radians(-6 + 68 * along**1.6)
        droop = math.radians(44 - 18 * along)
        direction = (side * math.sin(fan) * math.cos(droop), math.cos(fan) * math.cos(droop), -math.sin(droop))

        # Each feather is tipped a little, like a slat in a blind, so its
        # edge stands clear of the next one.
        blobs.append(_feather(base, direction, length, 0.044, up=(side * 0.3, 0, 1)))

    _plumage(f"feathers_{name}", blobs, feathers, wing, triangles=1800)


def _tail(root, feathers):
    # The fan starts under the seal on the back and falls away behind it.
    # It is folded like a very shallow roof, the outer feathers a little
    # lower than the middle one, so it is more than a line from the side.
    tail = creature.pivot("tail", root, (0, 0.24, 0.02))
    blobs = []

    for spread in (-40, -20, 0, 20, 40):
        swing = math.radians(spread)
        fall = math.radians(22 + abs(spread) * 0.15)
        direction = (math.sin(swing) * math.cos(fall), math.cos(swing) * math.cos(fall), -math.sin(fall))

        blobs.append(_feather((0, 0, 0), direction, 0.24 - abs(spread) * 0.001, 0.05, thickness=0.02, up=(math.sin(swing) * 0.5, 0, 1)))

    _plumage("tail_fan", blobs, feathers, tail, triangles=600)


def _feet(root, horn):
    blobs = []

    for side in (1, -1):
        # Balled up against the belly: a knuckle and three toes curled under.
        blobs.append(("sphere", (side * 0.075, -0.04, -0.235), (0.032, 0.034, 0.034)))

        for toe in (-1, 0, 1):
            blobs.append(("sphere", (side * 0.075 + toe * 0.026, -0.08, -0.257), (0.015, 0.04, 0.016), (math.radians(28), 0, math.radians(toe * -14))))

    creature.form("feet", blobs, horn, root, triangles=640)


def _head(root, body, detail, dark, accent):
    head = creature.pivot("head", root, (0, -0.08, 0.0))

    creature.form("skull", [("sphere", SKULL, SKULL_SIZE)], body, head, triangles=1100)

    _visor(head, dark, accent)

    # A short heavy beak: deep where it leaves the face, its ridge curving
    # over into a hook that hangs past the lower half.
    _horn(
        "beak_upper",
        Vector((0, -0.125, 0)),
        0.12,
        lambda t: 0.07 * (1 - t**2.4) + 0.007,
        lambda t: 0.102 - 0.09 * t**2.4,
        lambda t: 0.03 - 0.05 * (max(0.0, t - 0.55) / 0.45) ** 2,
        detail,
        head,
    )

    # The lower beak drops when it shrieks.
    jaw = creature.pivot("jaw", head, (0, -0.125, 0.03))

    _horn(
        "beak_lower",
        Vector((0, 0, 0)),
        0.082,
        lambda t: 0.056 * (1 - t**2.5) + 0.005,
        lambda t: 0.0,
        lambda t: -0.005 - 0.036 * (1 - t) ** 0.8,
        detail,
        jaw,
        keel=True,
    )

    _cap(creature.pivot("trait_1", head, (0, 0, 0)), detail)


def _horn(name, origin, length, width, top, bottom, mat, parent, keel=False, steps=7):
    """Half a beak: a wedge running forward from `origin` to a point.

    `width(t)` is half its width and `top(t)` and `bottom(t)` its upper and
    lower edge, at `t` from the face (0) to the tip (1). The upper half has
    its ridge on top and a flat underside; with `keel` it is the other way
    up, for the lower half.
    """

    rings = []

    for step in range(steps + 1):
        t = step / steps
        half, high, low = width(t), top(t), bottom(t)
        flat, ridge = (high, low) if keel else (low, high)

        def point(across, toward):
            return origin + Vector((half * across, -length * t, flat + (ridge - flat) * toward))

        rings.append([point(-1, 0), point(-0.92, 0.5), point(-0.55, 0.9), point(0, 1), point(0.55, 0.9), point(0.92, 0.5), point(1, 0)])

    horn = core.loft(name, rings, mat, parent, cap_start=True, cap_end=True)

    return creature.finish(core.shade_smooth(horn, angle=60))


def _band(name, radii, sweep, edges, depth, mat, parent, steps=22, origin=Vector()):
    """A strip wrapped around the skull at the height of the eyes.

    It reaches `sweep` radians either side of straight ahead. `radii` is its
    outer surface and `depth` how far it is sunk in from there.
    `edges(fraction)` gives its lower and upper height above the middle of
    the skull, and how far it has drawn in toward the skull, at `fraction`
    of the way from straight ahead (0) to either end (1).
    """

    chamfer = min(depth, 0.008)
    rings = []

    for step in range(steps + 1):
        fraction = step / steps * 2 - 1
        angle = fraction * sweep
        low, high, drawn = edges(abs(fraction))

        def point(inset, height):
            return SKULL - origin + Vector((math.sin(angle) * (radii[0] - drawn - inset), -math.cos(angle) * (radii[1] - drawn - inset), height))

        rings.append(
            [
                point(depth, low),
                point(chamfer, low),
                point(0, low + chamfer),
                point(0, high - chamfer),
                point(chamfer, high),
                point(depth, high),
            ]
        )

    band = core.loft(name, rings, mat, parent, cap_start=True, cap_end=True)

    return creature.finish(core.shade_smooth(band, angle=40))


def _visor(head, dark, accent):
    # Chunky and wrapped right round to the sides, where it narrows into
    # arms that lie against the skull. The top edge dips over the beak,
    # which is all the scowl it needs.
    def lens(fraction):
        narrow = max(0.0, fraction - 0.5) / 0.5
        scowl = max(0.0, 1 - fraction / 0.3)

        return -0.005 + 0.02 * narrow, 0.062 - 0.014 * narrow - 0.012 * scowl, 0.018 * narrow

    _band("visor", (0.153, 0.158), math.radians(104), lens, 0.05, dark, head)

    # What is left of its eyes: one thin strip of light across the lens. It
    # hangs from a pivot at its own height, so the game can open it wide or
    # squeeze it to a slit.
    middle = SKULL + Vector((0, 0, 0.0265))
    eyes = creature.pivot("eyes", head, middle)

    _band("eyes", (0.156, 0.161), math.radians(50), lambda fraction: (0.02, 0.033, 0), 0.012, accent, eyes, steps=12, origin=middle)

    # Two hard ridges over the top of the visor, following the skull round:
    # they are what knits into a scowl or lifts in fright.
    for side in (1, -1):
        angle = side * math.radians(21)
        at = SKULL + Vector((math.sin(angle) * 0.162, -math.cos(angle) * 0.166, 0.079))

        creature.brow(side, head, at, (0.042, 0.013, 0.011), dark, rotation=(0, 0, angle))


def _cap(parent, cloth):
    """A plain cap worn backwards. The seal on the crown shows through the
    top of it, where a cap's button would be."""

    size = SKULL_SIZE + Vector((0.011, 0.011, 0.008))
    brim = 0.064
    segments = 22
    lifts = [brim + (size.z - brim) * math.sin(step / 6 * math.pi / 2) for step in range(6)]
    rings = []

    # The dome starts on top of the visor. Its first ring tucks the edge in
    # against the skull.
    for inset, lift in ((0.03, brim), *((0.0, lift) for lift in lifts)):
        level = math.sqrt(max(0.0, 1 - (lift / size.z) ** 2))
        ring = []

        for segment in range(segments):
            angle = math.tau * segment / segments

            ring.append(SKULL + Vector((math.cos(angle) * (size.x * level - inset), math.sin(angle) * (size.y * level - inset), lift)))

        rings.append(ring)

    creature.finish(core.loft("cap", rings, cloth, parent, cap_end=True))

    # The bill points back over the hunch.
    creature.form(
        "cap_bill",
        [("sphere", SKULL + Vector((0, size.y + 0.02, brim + 0.012)), (0.09, 0.08, 0.013), (math.radians(8), 0, 0))],
        cloth,
        parent,
        triangles=260,
    )


def _link(centre, along, across, mat):
    """One oval link of a chain, lying in the plane of `along` and `across`."""

    normal = along.cross(across).normalized()
    rings = []
    steps = 8

    for step in range(steps + 1):
        angle = math.tau * step / steps
        out = (along * math.cos(angle) + across * math.sin(angle)).normalized()
        middle = centre + along * math.cos(angle) * 0.025 + across * math.sin(angle) * 0.016

        rings.append([middle + (out * math.cos(turn) + normal * math.sin(turn)) * 0.007 for turn in (0, math.pi / 2, math.pi, math.pi * 1.5)])

    return core.loft("link", rings, mat)


def _rest(point, bodies, lift):
    """`point` moved to sit `lift` above the nearest surface of `bodies`,
    and the direction that surface faces."""

    best = None

    for obj in bodies:
        _, surface, normal, _ = obj.closest_point_on_mesh(obj.matrix_world.inverted() @ point)
        surface = obj.matrix_world @ surface

        if best is None or (surface - point).length < (best[0] - point).length:
            best = (surface, normal)

    return best[0] + best[1] * lift, best[1]


def _chain(parent, metal, bodies):
    """A plain chunky chain: round the back of the neck, down either side of
    it, and hanging in a loop on the chest that frames the seal there."""

    half = [
        (0, -0.2, -0.11),
        (0.06, -0.2, -0.085),
        (0.105, -0.175, -0.03),
        (0.14, -0.12, 0.03),
        (0.155, -0.045, 0.08),
        (0.1, 0.04, 0.13),
        (0, 0.075, 0.15),
    ]
    loop = half + [(-x, y, z) for x, y, z in reversed(half[1:-1])]
    path = core.bezier(loop + loop[:1], samples=12)
    lengths = [0.0]

    for start, end in zip(path, path[1:]):
        lengths.append(lengths[-1] + (end - start).length)

    def walk(distance):
        distance %= lengths[-1]

        for index in range(1, len(path)):
            if lengths[index] >= distance:
                return path[index - 1].lerp(path[index], (distance - lengths[index - 1]) / (lengths[index] - lengths[index - 1]))

        return path[-1]

    # The links are laid on the body itself, so the chain still sits right
    # if the body under it is reshaped.
    bpy.context.view_layer.update()

    count = round(lengths[-1] / 0.037 / 2) * 2
    pitch = lengths[-1] / count
    links = []

    for index in range(count):
        centre, outward = _rest(walk(index * pitch), bodies, 0.011)
        along = (walk(index * pitch + 0.01) - walk(index * pitch - 0.01)).normalized()
        flat = along.cross(outward).normalized()

        # Every other link lies flat against the body; the ones between
        # stand on edge through them.
        links.append(_link(centre, along, flat if index % 2 else flat.cross(along), metal))

    chain = core.join(links, "chain")
    chain.parent = parent

    creature.finish(core.shade_smooth(chain, angle=50))
