"""The hollow: the rider with no head.

Silhouette: square, armoured shoulders under a high collar with nothing in
it, and the missing head carried low at the hip as a lantern. The outline
reads as a broad coat that simply stops at the neck.
"""

import math

import bmesh
from mathutils import Euler, Matrix, Vector, noise

from lib import anatomy, cloth, core
from lib.figure import NECK_HEIGHT, SHOULDER_HEIGHT, finish

# The coat's (depth, half-width, half-depth) stops: square at the shoulder,
# fitted to the waist, straight over the hip, then flaring into skirts.
COAT = ((0.0, 0.12, 0.1), (0.041, 0.265, 0.14), (0.16, 0.235, 0.16), (0.38, 0.165, 0.125), (0.6, 0.2, 0.16), (1.0, 0.4, 0.3))
COAT_TOP = SHOULDER_HEIGHT + 0.06
COAT_HEM = 0.2
WAIST_HEIGHT = 1.1
CHEST_HEIGHT = 1.38
SHOULDER_WIDTH = 0.225
HAND_SCALE = 1.45


def build():
    """The rider with no head: a riding coat, an empty collar, and a lantern
    where the head went."""

    root = core.empty("reaper")
    alt = core.material("cloth_alt")

    # The skirts trail behind the rider, so every hem leans back.
    hems = [
        dict(tatter=0.3, tears=7, slits=8, fold_depth=0.15, seed=31, lean=0.14),
        dict(tatter=0.46, tears=11, slits=16, fold_depth=0.18, seed=32, lean=0.18),
        dict(tatter=0.22, tears=6, slits=5, fold_depth=0.13, seed=33, lean=0.1),
        dict(tatter=0.38, tears=6, slits=10, fold_depth=0.2, seed=34, lean=0.3),
    ]

    for index, hem in enumerate(hems):
        coat = cloth.robe(
            f"robe_{index}",
            core.material("cloth"),
            root,
            top=COAT_TOP,
            hem=COAT_HEM,
            profile=COAT,
            folds=(4, 7, 12),
            **hem,
        )
        finish(_open_front(coat, hem["lean"]), hollow=_chest_first, seed=index)

    _arms(root)

    head = core.empty("head", root, location=(0, 0, NECK_HEIGHT))
    _collars(head, alt)
    _mantles(root, alt)

    return root


def _open_front(coat, lean):
    """Cuts the coat open down the front from the waist and draws the two
    edges apart, as a riding coat is cut to sit astride.

    `cloth.robe` only slits a hem at random, so the cut is made here, and
    every shape key is moved along with the cloth it belongs to. `lean` is
    the robe's own, which says where the front half of a leaning hem ends.
    """

    mesh = bmesh.new()
    mesh.from_mesh(coat.data)
    shapes = mesh.verts.layers.shape.values()
    seam = []

    for vertex in mesh.verts:
        below = (WAIST_HEIGHT - vertex.co.z) / (WAIST_HEIGHT - COAT_HEM)
        near = 1 - abs(vertex.co.x) / 0.17
        middle = lean * ((COAT_TOP - vertex.co.z) / (COAT_TOP - COAT_HEM)) ** 2

        if below <= 0 or near <= 0 or vertex.co.y > middle:
            continue

        if abs(vertex.co.x) < 1e-6:
            seam.append(vertex)
            continue

        shift = Vector((math.copysign(0.11, vertex.co.x), 0.05, 0)) * below * near
        vertex.co += shift

        for shape in shapes:
            vertex[shape] += shift

    bmesh.ops.delete(mesh, geom=seam, context="VERTS")
    mesh.to_mesh(coat.data)
    mesh.free()

    return coat


def _chest_first(co):
    """How long each spot of the coat lasts as the outfit hollows.

    Every other class is eaten from the hem upward. A hollow loses its chest
    first, between the collar and the belt, and is left as shoulders and
    skirts hanging around nothing.
    """

    ragged = 0.14 * noise.noise(co * 5.0)

    return max(0.0, min(1.0, 0.02 + 1.9 * abs(co.z - CHEST_HEIGHT) + ragged))


def _whole(co):
    """The hollowing mask of everything that outlasts the chest: the collar
    and the arms are what is left to hang around the empty space."""

    return 1.0


def _coat_point(angle, height, lift=0.0):
    """A point on the coat's surface, `lift` proud of it, so that straps and
    chains can be laid on the chest rather than through it."""

    depth = (COAT_TOP - height) / (COAT_TOP - COAT_HEM)
    width, thickness = COAT[-1][1:]

    for (d0, w0, t0), (d1, w1, t1) in zip(COAT, COAT[1:]):
        if depth <= d1:
            blend = (depth - d0) / (d1 - d0)
            blend = blend * blend * (3 - 2 * blend)
            width, thickness = w0 + (w1 - w0) * blend, t0 + (t1 - t0) * blend

            break

    return Vector((math.cos(angle) * (width + lift), math.sin(angle) * (thickness + lift), height))


def _smooth(value):
    """Eases 0..1 in and out, so a collar's edge sweeps instead of kinking."""

    value = max(0.0, min(1.0, value))

    return value * value * (3 - 2 * value)


def _crisp(obj, degrees):
    """Shades `obj` smooth but keeps every edge sharper than `degrees` hard.

    `core.shade_smooth` does this with a modifier, which stays on the object
    and never reaches the exported model, so the hard edges are marked on
    the mesh itself here.
    """

    mesh = bmesh.new()
    mesh.from_mesh(obj.data)

    for edge in mesh.edges:
        edge.smooth = edge.calc_face_angle(math.pi) < math.radians(degrees)

    mesh.to_mesh(obj.data)
    mesh.free()

    return core.shade_smooth(obj)


def _shell(name, mat, point, columns, rows, thickness, parent=None):
    """A sheet with real thickness: a collar, or a plate of armour.

    `point(u, v)` maps the unit square onto the sheet's outer face.
    """

    rings = [[point(column / columns, row / rows) for column in range(columns + 1)] for row in range(rows + 1)]
    sheet = core.loft(name, rings, mat, parent, closed=False)

    return core.apply_modifier(sheet, "SOLIDIFY", thickness=thickness, offset=-1.0)


def _lames(name, axis, plates, sweep):
    """Overlapping plates wrapped over a shoulder.

    `axis` is the line the shoulder and upper arm follow, in the XZ plane.
    Each plate is a (start, end, radius) span along it. A plate is keeled
    along its crown, comes to a point there, and lifts into a lip at its
    trailing edge, so the layers read as separate pieces of forged iron
    rather than as a bent pipe.
    """

    metal = core.material("metal")
    line = core.bezier(axis, samples=12)
    parts = []

    def station(share):
        position = share * (len(line) - 1)
        index = min(int(position), len(line) - 2)
        tangent = (line[index + 1] - line[index]).normalized()
        up = Vector((-tangent.z, 0, tangent.x))

        return line[index].lerp(line[index + 1], position - index), up if up.z > 0 else -up

    for index, (start, end, radius) in enumerate(plates):

        def point(u, v, start=start, end=end, radius=radius):
            turn = (u - 0.5) * sweep
            crown = max(0.0, math.cos(turn))
            centre, up = station(start + (end - start) * v * (0.6 + 0.4 * crown**3))
            reach = radius * (1 - 0.1 * v + 0.07 * crown**8) + 0.024 * v**4 * crown

            return centre + up * reach * math.cos(turn) + Vector((0, reach * math.sin(turn), 0))

        parts.append(_shell(f"{name}_{index}", metal, point, 20, 5, 0.007))

    return _crisp(core.join(parts, name), 50)


def _ring(name, centre, along, across, wire, square=1.0):
    """A loop of iron wire around `centre`. `along` and `across` are its two
    half-axes, and a `square` below 1 pulls the oval toward a rectangle."""

    steps = 10 if square == 1.0 else 20
    path = []

    for step in range(steps + 1):
        turn = math.tau * step / steps
        reach = math.copysign(abs(math.cos(turn)) ** square, math.cos(turn))
        spread = math.copysign(abs(math.sin(turn)) ** square, math.sin(turn))
        path.append(centre + along * reach + across * spread)

    return core.tube(name, path, wire, core.material("metal"), sides=5, cap=False)


def _chain(name, points, pitch, wire):
    """Oval links threaded along `points`, each turned a quarter from the
    last, so a drawn line becomes a chain that could bear weight."""

    links = []
    travelled = pitch

    for near, far in zip(points, points[1:]):
        travelled += (far - near).length

        if travelled < pitch:
            continue

        travelled = 0.0
        tangent = (far - near).normalized()
        outward = Vector((far.x, far.y, 0)) if abs(tangent.z) < 0.99 else Vector((0, -1, 0))
        outward = (outward - tangent * outward.dot(tangent)).normalized()
        across = outward if len(links) % 2 else tangent.cross(outward)

        links.append(_ring(f"{name}_{len(links)}", far, tangent * pitch * 0.8, across * pitch * 0.36, wire))

    return core.shade_smooth(core.join(links, name))


def _dent(obj, x, z, width, height, depth):
    """Presses a pit into the front of a sculpted form.

    Sockets are pushed in rather than cut out: a boolean between two
    remeshed forms fails or succeeds on the smallest change to either.
    """

    origin = Vector(obj.location)

    for vertex in obj.data.vertices:
        co = vertex.co + origin
        distance = ((co.x - x) / width) ** 2 + ((co.z - z) / height) ** 2

        if co.y < 0 and distance < 1:
            vertex.co.y += depth * (1 - distance) ** 0.5


def _arms(root):
    """Both arms, each under its own shoulder pivot so the game can move it.

    Built here rather than with `figure.arms` because a hollow's off hand is
    not open: it is closed around the handle of its own head. Both forearms
    are carried forward, the way a rider holds reins.
    """

    paths = {
        "arm_r": [(-SHOULDER_WIDTH, 0, SHOULDER_HEIGHT - 0.03), (-0.345, 0.09, 1.35), (-0.38, -0.15, 1.2)],
        "arm_l": [(SHOULDER_WIDTH, 0, SHOULDER_HEIGHT - 0.03), (0.37, 0.09, 1.36), (0.47, -0.08, 1.21)],
    }

    for label, path in paths.items():
        side = 1 if label == "arm_l" else -1
        shoulder = Vector(path[0])
        pivot = core.empty(label, root, location=shoulder)
        local = [Vector(point) - shoulder for point in core.bezier(path, samples=8)]
        direction = (local[-1] - local[-4]).normalized()
        grip = local[-1] + direction * 0.1 * HAND_SCALE

        # A coat's sleeve: close to the arm, ending in a deep turned-back
        # cuff instead of a robe's hanging one.
        finish(
            cloth.sleeve(f"{label}_sleeve", core.material("cloth"), pivot, local, 0.088, 0.066, droop=0.03, tatter=0.2, seed=4 + side),
            hollow=_whole,
        )
        finish(
            core.tube(
                f"{label}_cuff",
                local[-5:],
                lambda t: 0.092 - 0.012 * t,
                core.material("cloth_alt"),
                pivot,
                sides=20,
                cap=False,
            ),
            hollow=_whole,
        )

        # Square shoulders: two plates over the head of each sleeve.
        spaulder = _lames(
            f"{label}_spaulder",
            [(side * -0.06, 0, -0.03), (0, 0, -0.024), (side * 0.06, 0, -0.04), (side * 0.085, 0, -0.1)],
            [(0.0, 0.6, 0.094), (0.4, 1.0, 0.082)],
            sweep=math.radians(190),
        )
        spaulder.parent = pivot
        finish(spaulder, hollow=_whole, wear_boost=1.6)

        if label == "arm_r":
            socket = core.empty("socket_weapon", pivot, location=grip)
            hand = anatomy.hand(f"{label}_hand", socket, curl=1.0, side=1, scale=HAND_SCALE)
        else:
            # Knuckles forward and the wrist tipped back toward the forearm,
            # with the fingers hooked under the lantern's handle.
            hand = anatomy.hand(f"{label}_hand", pivot, grip_radius=0.012, curl=1.0, side=1, scale=HAND_SCALE)
            hand.location = grip
            hand.rotation_euler = (
                Matrix.Rotation(math.radians(-28), 3, "X") @ Euler((math.radians(90), 0, math.radians(90))).to_matrix()
            ).to_euler()
            _lantern(pivot, grip)

        finish(hand, wear_boost=1.6)


def _lantern(arm, grip):
    """The hollow's own head, carried as a lantern: a rough skull in an iron
    harness, hung from a stirrup, with the class's light burning inside it."""

    metal = core.material("metal")
    lantern = core.empty("lantern", arm, location=grip)

    # A stirrup for a handle: the hollow was a rider.
    finish(
        core.shade_smooth(
            core.tube(
                "lantern_stirrup",
                core.bezier(
                    [(-0.08, 0, 0), (0.08, 0, 0), (0.075, 0, -0.05), (0.0, 0, -0.095), (-0.075, 0, -0.05), (-0.08, 0, 0)],
                    samples=6,
                ),
                0.012,
                metal,
                lantern,
                sides=8,
                cap=False,
            )
        ),
        hollow=_whole,
        wear_boost=1.6,
    )

    chain = _chain("lantern_chain", [Vector((0, 0, -0.075 - 0.004 * step)) for step in range(19)], 0.036, 0.0065)
    chain.parent = lantern
    finish(chain, hollow=_whole, wear_boost=1.6)

    # The skull is larger than life, and turned a little out from the body so
    # that it is seen in part profile from the front.
    carried = core.empty("lantern_head", lantern, location=(0, 0, -0.305))
    carried.rotation_euler = (0, 0, math.radians(24))
    carried.scale = (1.25, 1.25, 1.25)

    skull = core.sculpt(
        "lantern_skull",
        [
            ("sphere", (0, 0.02, 0.03), (0.088, 0.105, 0.09)),
            ("sphere", (0, -0.068, 0.012), (0.07, 0.03, 0.03)),
            ("sphere", (0, -0.058, -0.045), (0.056, 0.048, 0.06)),
            ("sphere", (-0.05, -0.058, -0.032), (0.024, 0.034, 0.02)),
            ("sphere", (0.05, -0.058, -0.032), (0.024, 0.034, 0.02)),
            ("sphere", (0, -0.064, -0.09), (0.04, 0.036, 0.03)),
        ],
        core.material("bone"),
        voxel=0.008,
    )

    for x, z, width, height, depth in ((-0.036, -0.012, 0.027, 0.024, 0.036), (0.036, -0.012, 0.027, 0.024, 0.036), (0, -0.058, 0.013, 0.02, 0.022)):
        _dent(skull, x, z, width, height, depth)

    # The light burns at the back of both sockets.
    flame = core.sculpt(
        "lantern_flame",
        [("sphere", (side * 0.036, -0.056, -0.012), (0.018, 0.018, 0.016)) for side in (-1, 1)],
        core.material("accent"),
        voxel=0.005,
    )

    # Iron around the skull: a band at the brow and two over the crown,
    # meeting at the eye the chain hangs from.
    bands = [
        core.tube(
            "lantern_brow",
            [(math.cos(math.tau * step / 32) * 0.094, 0.018 + math.sin(math.tau * step / 32) * 0.112, 0.04) for step in range(33)],
            0.01,
            metal,
            sides=6,
            cap=False,
            flatten=0.4,
        )
    ]

    for band, turn in enumerate((0.0, math.pi / 2)):
        path = []

        for step in range(17):
            arc = math.pi * step / 16
            reach = math.cos(arc)
            path.append((math.cos(turn) * reach * 0.094, 0.018 + math.sin(turn) * reach * 0.112, 0.04 + math.sin(arc) * 0.088))

        bands.append(core.tube(f"lantern_crown_{band}", path, 0.008, metal, sides=6, cap=False, flatten=0.45))

    bands.append(
        core.revolve(
            "lantern_eye",
            [(0.0, 0.158), (0.012, 0.152), (0.016, 0.138), (0.024, 0.128), (0.0, 0.124)],
            metal,
            segments=10,
        )
    )
    bands[-1].location = (0, 0.018, 0)
    harness = _crisp(core.join(bands, "lantern_harness"), 50)

    for part in (skull, flame, harness):
        part.parent = carried

    # A sculpted form keeps the texture coordinates of the spheres it was
    # made from, which would push the rest position out of the two channels
    # the game reads it from.
    while skull.data.uv_layers:
        skull.data.uv_layers.remove(skull.data.uv_layers[0])

    finish(skull, wear_boost=1.4)
    finish(harness, hollow=_whole, wear_boost=1.6)


def _collars(head, alt):
    """Three collars and no heads.

    Each stands around a dark stump where the neck was cut, with one ember
    burning low inside it. The collar is what the game turns in place of a
    head.
    """

    base = SHOULDER_HEIGHT - NECK_HEIGHT

    def standing(turn, gap):
        """A collar cut low at the throat, rising to a point beside each
        cheek and dipping at the nape, so the gap between the points reads
        as the missing head."""

        edge = math.pi - gap / 2 - abs(turn)

        return 0.09 + 0.26 * _smooth(edge / 0.75) - 0.18 * _smooth(1 - abs(turn) / 1.3)

    def broad(turn, gap):
        """A cape collar: lower than the standing one and half as wide
        again, an open bowl that dips at the nape."""

        edge = math.pi - gap / 2 - abs(turn)

        return 0.06 + 0.19 * _smooth(edge / 0.9) - 0.1 * math.exp(-((turn / 0.95) ** 2))

    def burned(turn, gap):
        """The standing collar after a fire took one side of it down to the
        shoulder."""

        scorch = math.exp(-(((turn + 1.25) / 0.85) ** 2))
        teeth = abs(math.sin(turn * 7.0 + 1.0)) * abs(math.sin(turn * 3.1 + 2.0))

        return standing(turn, gap) * (1 - 0.78 * scorch) * (1 - (0.12 + 0.5 * scorch) * teeth) + 0.035

    variants = [
        dict(rise=standing, gap=0.95, flare=0.3, splay=0.16),
        dict(rise=broad, gap=1.7, flare=0.75, splay=0.12),
        dict(rise=burned, gap=1.1, flare=0.34, splay=0.22),
    ]

    for index, variant in enumerate(variants):
        group = core.empty(f"head_{index}", head)

        def point(u, v, variant=variant):
            # `turn` runs from one side of the throat, round the nape, to
            # the other; 0 is the back of the neck.
            turn = (u * 2 - 1) * (math.pi - variant["gap"] / 2)
            edge = math.pi - variant["gap"] / 2 - abs(turn)
            gain = 1 + variant["flare"] * v**1.5 + variant["splay"] * v * math.exp(-((edge / 0.5) ** 2))

            return (
                math.sin(turn) * 0.165 * gain,
                math.cos(turn) * 0.138 * gain + 0.012,
                base + variant["rise"](turn, variant["gap"]) * v,
            )

        collar = _shell(f"collar_{index}", alt, point, 96, 9, 0.012, group)
        finish(_crisp(collar, 50), hollow=_whole, seed=index)

        # The stump is cut flat, so that nothing inside the collar can be
        # taken for a head. It closes the neck of the coat.
        stump = core.revolve(
            f"void_{index}",
            [(0.0, -0.02), (0.11, -0.02), (0.128, -0.03), (0.132, -0.075)],
            core.material("void"),
            group,
            segments=32,
        )
        stump.data.transform(Matrix.Diagonal((1, 0.86, 1, 1)))

        # The ember sits on the stump. It is built around its own origin so
        # the game can scale it by the genome's gaze without moving it.
        ember = core.sculpt(
            f"eyes_{index}",
            [
                ("sphere", (0, 0, 0), (0.03, 0.024, 0.012)),
                ("sphere", (0.024, 0.012, 0.003), (0.016, 0.014, 0.009)),
                ("sphere", (-0.02, -0.01, 0.002), (0.014, 0.012, 0.008)),
            ],
            core.material("accent"),
            group,
            voxel=0.004,
            smooth=1,
        )
        ember.location = (0, 0.015, -0.014)

        if index == 1:
            _neck_ring(group)


def _neck_ring(group):
    """An iron ring that was locked around the neck, broken open at one side
    of the throat and left lying crooked in the collar."""

    path = []

    for step in range(57):
        # The break is the part of the circle the ring never covers.
        turn = -2.2 + (math.tau - 0.6) * step / 56

        path.append((math.cos(turn) * 0.142, math.sin(turn) * 0.122 + 0.004, -0.014 - 0.014 * math.sin(turn + 0.5)))

    ring = core.tube(
        "neck_ring",
        path,
        # Each broken end is drawn out thin where the iron tore.
        lambda t: 0.017 * (1 - 0.7 * max(0.0, abs(2 * t - 1) - 0.88) / 0.12),
        core.material("metal"),
        group,
        sides=10,
        flatten=0.75,
    )
    finish(core.shade_smooth(ring), hollow=_whole, wear_boost=1.8)


def _mantles(root, alt):
    """What sits on a hollow's shoulders."""

    # One heavy pauldron over the weapon arm, held on by a strap across the
    # chest.
    group = core.empty("mantle_0", root)
    pauldron = _lames(
        "pauldron",
        [(-0.11, 0, 1.59), (-0.235, 0, 1.612), (-0.335, 0, 1.585), (-0.39, 0, 1.49), (-0.415, 0, 1.38)],
        [(0.0, 0.46, 0.134), (0.3, 0.66, 0.12), (0.52, 0.84, 0.108), (0.72, 1.0, 0.098)],
        sweep=math.radians(196),
    )
    pauldron.parent = group
    finish(pauldron, wear_boost=1.8)

    strap = [
        _coat_point(math.radians(turn), height, 0.012)
        for turn, height in (
            (200, 1.64),
            (235, 1.53),
            (265, 1.41),
            (300, 1.28),
            (340, 1.2),
            (360, 1.17),
            (400, 1.2),
            (440, 1.3),
            (475, 1.43),
            (510, 1.56),
            (535, 1.65),
            (560, 1.64),
        )
    ]
    finish(core.tube("strap", core.bezier(strap, samples=8), 0.026, core.material("rope"), group, sides=8, cap=False, flatten=0.3))

    along = (strap[2] - strap[1]).normalized()
    outward = Vector((strap[1].x, strap[1].y, 0)).normalized()
    buckle = _ring("strap_buckle", strap[1].lerp(strap[2], 0.5) + outward * 0.006, along * 0.024, along.cross(outward) * 0.038, 0.007, square=0.4)
    buckle.parent = group
    finish(core.shade_smooth(buckle), wear_boost=1.8)

    # A short cape lying square over both shoulders, open at the throat and
    # held there by a chain.
    group = core.empty("mantle_1", root)
    stops = core.bezier([(0.165, 0.14, 1.705), (0.3, 0.165, 1.695), (0.395, 0.195, 1.63), (0.425, 0.225, 1.48), (0.43, 0.25, 1.28)], samples=6)

    def cape(u, v):
        turn = (u * 2 - 1) * (math.pi - 0.6)
        torn = 0.34 * abs(math.sin(turn * 7 + 1)) * abs(math.sin(turn * 2.6 + 2))
        reach = v * (1 - torn)
        position = reach * (len(stops) - 1)
        index = min(int(position), len(stops) - 2)
        width, thickness, height = stops[index].lerp(stops[index + 1], position - index)
        fold = 1 + 0.07 * reach**1.5 * (0.6 * math.sin(turn * 5 + 0.7) + 0.4 * math.sin(turn * 9 + 2.1))

        return (math.sin(turn) * width * fold, math.cos(turn) * thickness * fold + 0.005, height)

    finish(core.loft("cape", [[cape(column / 96, row / 22) for column in range(97)] for row in range(23)], alt, group, closed=False), seed=3)

    path = []

    for step in range(61):
        share = step / 30 - 1
        sag = 1 - share * share
        path.append(Vector((0.115 * share, -0.15 - 0.012 * sag, 1.672 - 0.03 * sag)))

    clasp = _chain("cape_clasp", path, 0.034, 0.006)
    clasp.parent = group
    finish(clasp, wear_boost=1.6)

    # Chains from shoulder to shoulder: two across the chest, one across the
    # back, and one that broke and hangs where it fell.
    group = core.empty("mantle_2", root)
    swags = []

    for middle, spread, sag in ((270, 78, 0.13), (270, 84, 0.27), (90, 80, 0.2)):
        path = []

        for step in range(121):
            share = step / 60 - 1
            path.append(_coat_point(math.radians(middle + spread * share), 1.625 - sag * (1 - share * share), 0.03))

        swags.append(_chain(f"swag_{len(swags)}", path, 0.05, 0.0085))

    start = _coat_point(math.radians(318), 1.6, 0.03)
    swags.append(_chain("swag_broken", [start - Vector((0, 0, 0.004 * step)) for step in range(96)], 0.05, 0.0085))

    chains = core.join(swags, "chains")
    chains.parent = group
    finish(chains, wear_boost=1.6)
