"""The hollow: the rider with no head.

Silhouette: two great pauldrons with nothing between them, a riding coat
pinched at the waist and flaring into split skirts, and the missing head
carried low at the off hand as a lantern. Each pauldron's crown sweeps up
beside the cut neck, so the outline's top is a notch: the void between the
pauldrons, where a head should be, is the read (docs/art-bible.md 6.2).

Three families show: the coat and its collar (`cloth`), the iron (`metal`)
and the skull and hands (`bone`). The skull is the one bone mass, larger
than life, and the light is in its eyes. The lantern is the figure's one
asymmetry: the shoulders, collars and mantles are square and alike.
"""

import math

import bmesh
from mathutils import Matrix, Vector, noise

from lib import anatomy, cloth, core
from lib.figure import NECK_HEIGHT, SHOULDER_HEIGHT, finish

# The coat's (depth, half-width, half-depth) stops: broad and square at the
# shoulder, pinched hard at the waist, then flaring into wide skirts.
COAT = ((0.0, 0.125, 0.1), (0.04, 0.29, 0.15), (0.15, 0.255, 0.16), (0.37, 0.14, 0.115), (0.56, 0.21, 0.165), (1.0, 0.45, 0.34))
COAT_TOP = SHOULDER_HEIGHT + 0.06
COAT_HEM = 0.16
WAIST_HEIGHT = 1.12
CHEST_HEIGHT = 1.38
SHOULDER_WIDTH = 0.26
HAND_SCALE = 1.45

# The skull lantern, as much larger than life as the bible's one bone mass
# wants. Its geometry is a skull's at 1.0, and lib/face.py seats the eyes in
# its sockets in that frame, so it grows by its node's scale alone.
SKULL_SCALE = 1.75

# The off arm in the idle pose's first key (lib/motion.py, at the hollow's
# amplitude of 1.12 there): swung out from the body, the forearm raised.
# The lantern is turned back by both, so that at rest it hangs plumb.
IDLE_SHOULDER_L = math.radians(-8 * 1.12)
IDLE_ELBOW_L = math.radians(-16 * 1.12)


def build():
    """The rider with no head: great pauldrons, a riding coat, and a
    lantern where the head went."""

    root = core.empty("reaper")

    # The skirts trail behind the rider, so every hem leans back.
    hems = [
        dict(tatter=0.26, tears=5, slits=6, fold_depth=0.13, seed=31, lean=0.14),
        dict(tatter=0.4, tears=8, slits=10, fold_depth=0.16, seed=32, lean=0.2),
        dict(tatter=0.18, tears=4, slits=3, fold_depth=0.11, seed=33, lean=0.1),
        dict(tatter=0.34, tears=5, slits=8, fold_depth=0.17, seed=34, lean=0.3),
    ]

    for index, hem in enumerate(hems):
        coat = cloth.robe(
            f"robe_{index}",
            core.material("cloth"),
            root,
            top=COAT_TOP,
            hem=COAT_HEM,
            profile=COAT,
            folds=(4, 6, 11),
            rings=44,
            segments=112,
            **hem,
        )
        finish(_open_front(coat, hem["lean"]), hollow=_chest_first, seed=index)

    _arms(root)

    head = core.empty("head", root, location=(0, 0, NECK_HEIGHT))
    _collars(head)
    _mantles(root)

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
    """A point on the coat's surface, `lift` proud of it, so that chains can
    be laid on the chest rather than through it."""

    depth = (COAT_TOP - height) / (COAT_TOP - COAT_HEM)
    width, thickness = COAT[-1][1:]

    for (d0, w0, t0), (d1, w1, t1) in zip(COAT, COAT[1:]):
        if depth <= d1:
            blend = (depth - d0) / (d1 - d0)
            blend = blend * blend * (3 - 2 * blend)
            width, thickness = w0 + (w1 - w0) * blend, t0 + (t1 - t0) * blend

            break

    return Vector((math.cos(angle) * (width + lift), math.sin(angle) * (thickness + lift), height))


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


def _shell(name, mat, point, columns, rows, thickness, parent=None, closed=False):
    """A sheet with real thickness: a collar, or a plate of armour.

    `point(u, v)` maps the unit square onto the sheet's outer face. A
    `closed` sheet wraps round on itself at u = 1, with no seam.
    """

    span = columns if closed else columns + 1
    rings = [[point(column / columns, row / rows) for column in range(span)] for row in range(rows + 1)]
    sheet = core.loft(name, rings, mat, parent, closed=closed)

    return core.apply_modifier(sheet, "SOLIDIFY", thickness=thickness, offset=-1.0)


def _lames(name, axis, plates, sweep, squeeze=0.0, rake=0.0):
    """Overlapping plates wrapped over a shoulder.

    `axis` is the line the shoulder and upper arm follow, in the XZ plane.
    Each plate is a (start, end, radius) span along it. A plate is keeled
    along its crown, runs a little further there, and lifts into a lip at
    its trailing edge, so the layers read as separate pieces of forged iron
    rather than as a bent pipe. `squeeze` draws the plates in from front to
    back toward the start of the axis, and `rake` sweeps whatever stands
    higher than the shoulder back, so that seen from the side the inner end
    stands as a swept fin rather than a dome the size of a head.
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
            share = start + (end - start) * v * (0.84 + 0.16 * crown**3)
            centre, up = station(share)
            reach = radius * (1 - 0.06 * v + 0.06 * crown**8) + 0.012 * v**4 * crown
            depth = 1 - squeeze * (1 - share) ** 2
            position = centre + up * reach * math.cos(turn) + Vector((0, reach * math.sin(turn) * depth, 0))
            position.y += rake * max(0.0, position.z - 0.05)

            return position

        parts.append(_shell(f"{name}_{index}", metal, point, 24, 5, 0.008))

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

    return core.tube(name, path, wire, core.material("metal"), sides=6, cap=False)


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

        links.append(_ring(f"{name}_{len(links)}", far, tangent * pitch * 0.8, across * pitch * 0.38, wire))

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
        "arm_r": [(-SHOULDER_WIDTH, 0, SHOULDER_HEIGHT - 0.03), (-0.37, 0.08, 1.35), (-0.4, -0.15, 1.2)],
        "arm_l": [(SHOULDER_WIDTH, 0, SHOULDER_HEIGHT - 0.03), (0.39, 0.08, 1.36), (0.48, -0.08, 1.21)],
    }

    for label, path in paths.items():
        side = 1 if label == "arm_l" else -1
        shoulder = Vector(path[0])
        pivot = core.empty(label, root, location=shoulder)
        local = [Vector(point) - shoulder for point in core.bezier(path, samples=8)]
        direction = (local[-1] - local[-4]).normalized()
        grip = local[-1] + direction * 0.1 * HAND_SCALE
        middle = len(local) // 2
        elbow_at = local[middle]
        elbow = core.empty(label.replace("arm", "elbow"), pivot, location=elbow_at)
        wrist = core.empty(label.replace("arm", "wrist"), elbow, location=grip - elbow_at)

        # A coat's sleeve: close to the arm, ending in a deep turned-back
        # cuff instead of a robe's hanging one.
        finish(
            cloth.sleeve(
                f"{label}_upper_sleeve",
                core.material("cloth"),
                pivot,
                local[: middle + 3],
                0.09,
                0.082,
                droop=0,
                tatter=0,
                seed=4 + side,
                cap_shoulder=True,
            ),
            hollow=_whole,
        )
        finish(
            cloth.sleeve(
                f"{label}_lower_sleeve",
                core.material("cloth"),
                elbow,
                [point - elbow_at for point in local[middle - 1 :]],
                0.082,
                0.068,
                droop=0.03,
                tatter=0.2,
                seed=4 + side,
            ),
            hollow=_whole,
        )
        finish(
            core.tube(
                f"{label}_cuff",
                [point - elbow_at for point in local[-5:]],
                lambda t: 0.1 - 0.014 * t,
                core.material("cloth"),
                elbow,
                sides=20,
                cap=False,
            ),
            hollow=_whole,
        )

        spaulder = _pauldron(f"{label}_spaulder", side)
        spaulder.parent = pivot
        finish(spaulder, hollow=_whole, wear_boost=1.6)

        if label == "arm_r":
            socket = core.empty("socket_weapon", wrist)
            hand = anatomy.hand(f"{label}_hand", socket, curl=1.0, side=1, scale=HAND_SCALE)
        else:
            # Knuckles forward and the wrist tipped back toward the forearm,
            # with the fingers hooked under the lantern's handle.
            hand = anatomy.hand(f"{label}_hand", wrist, grip_radius=0.014, curl=1.0, side=1, scale=HAND_SCALE)
            hand.rotation_euler = (Matrix.Rotation(math.radians(-28), 3, "X") @ Matrix.Rotation(math.radians(90), 3, "Z") @ Matrix.Rotation(math.radians(90), 3, "X")).to_euler()
            _lantern(wrist)

        finish(hand, wear_boost=1.6)


def _pauldron(name, side):
    """A great pauldron, built in its shoulder pivot's frame (`side` is +1
    for the left): a high cop over two lames stepping down the arm, each
    narrower than the one above. Its inner end is swept up beside the cut
    neck, so that the two pauldrons stand higher than the collar between
    them and frame the void where a head should be.

    Three plates, not more: each lip is a step in the outline, and at hunt
    scale more steps read as a fringe."""

    return _lames(
        name,
        [(side * -0.12, 0, 0.11), (side * -0.03, 0, 0.09), (side * 0.07, 0, 0.04), (side * 0.14, 0, -0.07), (side * 0.16, 0, -0.2)],
        [(0.0, 0.5, 0.152), (0.38, 0.77, 0.128), (0.64, 1.0, 0.106)],
        sweep=math.radians(200),
        squeeze=0.42,
        rake=0.45,
    )


def _lantern(wrist):
    """The hollow's own head, carried as a lantern: a skull twice the size of
    life in an iron harness, hung on two links from a stirrup."""

    metal = core.material("metal")
    lantern = core.empty("lantern", wrist)
    lantern.rotation_mode = "XYZ"
    lantern.rotation_euler = (Matrix.Rotation(-IDLE_ELBOW_L, 3, "X") @ Matrix.Rotation(-IDLE_SHOULDER_L, 3, "Y")).to_euler()

    # A stirrup for a handle: the hollow was a rider.
    finish(
        core.shade_smooth(
            core.tube(
                "lantern_stirrup",
                core.bezier(
                    [(-0.08, 0, 0), (0.08, 0, 0), (0.075, 0, -0.05), (0.0, 0, -0.095), (-0.075, 0, -0.05), (-0.08, 0, 0)],
                    samples=6,
                ),
                0.014,
                metal,
                lantern,
                sides=8,
                cap=False,
            )
        ),
        hollow=_whole,
        wear_boost=1.6,
    )

    chain = _chain("lantern_chain", [Vector((0, 0, -0.07 - 0.004 * step)) for step in range(16)], 0.05, 0.009)
    chain.parent = lantern
    finish(chain, hollow=_whole, wear_boost=1.6)

    # Hung by the eye on its crown from the lower link, and turned a little
    # out from the body, so that both sockets face the viewer from the front.
    carried = core.empty("lantern_head", lantern, location=(0, 0, -0.16 - 0.162 * SKULL_SCALE))
    carried.rotation_euler = (0, 0, math.radians(10))
    carried.scale = (SKULL_SCALE, SKULL_SCALE, SKULL_SCALE)

    # A gothic cranium: the dome, a heavy brow, the cheekbones, the arch of
    # the teeth and a narrow jaw.
    blobs = [
        ("sphere", (0, 0.022, 0.034), (0.086, 0.104, 0.088)),
        ("sphere", (0, -0.048, 0.028), (0.074, 0.046, 0.042)),
        ("sphere", (-0.034, -0.078, 0.016), (0.036, 0.024, 0.017), (0, math.radians(10), 0)),
        ("sphere", (0.034, -0.078, 0.016), (0.036, 0.024, 0.017), (0, math.radians(-10), 0)),
        ("sphere", (0, -0.088, -0.012), (0.014, 0.018, 0.024)),
        ("sphere", (-0.056, -0.058, -0.03), (0.028, 0.038, 0.022), (0, 0, math.radians(-16))),
        ("sphere", (0.056, -0.058, -0.03), (0.028, 0.038, 0.022), (0, 0, math.radians(16))),
        ("sphere", (0, -0.068, -0.058), (0.048, 0.044, 0.034)),
        ("sphere", (0, -0.078, -0.082), (0.038, 0.034, 0.02)),
        ("sphere", (0, -0.068, -0.104), (0.032, 0.03, 0.022)),
    ]

    # Five big teeth along the arch: at hunt scale more would be a grey band.
    for tooth in (-2, -1, 0, 1, 2):
        angle = tooth * 0.26
        blobs.append(("cube", (math.sin(angle) * 0.036, -0.072 - math.cos(angle) * 0.028, -0.088), (0.0062, 0.006, 0.012), (0, 0, angle)))

    skull = core.sculpt("lantern_skull", blobs, core.material("bone"), voxel=0.0055, smooth=2)

    # Deep orbits where the light burns, a pear-shaped nose and hollow
    # cheeks. lib/face.py seats the eyes at x ±0.036, z -0.010.
    for x, z, width, height, depth in (
        (-0.036, -0.01, 0.03, 0.026, 0.046),
        (0.036, -0.01, 0.03, 0.026, 0.046),
        (0.0, -0.048, 0.015, 0.022, 0.034),
        (-0.056, -0.066, 0.022, 0.024, 0.022),
        (0.056, -0.066, 0.022, 0.024, 0.022),
    ):
        _dent(skull, x, z, width, height, depth)

    # Cut into planes, so that it prints as a carved mass in a few hard
    # bands rather than a speckle of small ones.
    core.planes(skull, 1500, flat=7.0, crease=36.0)

    # The light burns at the back of both sockets. lib/face.py replaces it
    # with the eyes, so it is here for the authoring scene alone.
    flame = core.sculpt(
        "lantern_flame",
        [("sphere", (side * 0.036, -0.056, -0.012), (0.018, 0.018, 0.016)) for side in (-1, 1)],
        core.material("accent"),
        voxel=0.005,
    )

    # Iron round the skull: a band at the brow and two straps over the
    # crown, meeting at the eye the chain hangs from.
    bands = [
        core.tube(
            "lantern_brow",
            [(math.cos(math.tau * step / 32) * 0.094, 0.018 + math.sin(math.tau * step / 32) * 0.112, 0.04) for step in range(33)],
            0.011,
            metal,
            sides=8,
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

        bands.append(core.tube(f"lantern_crown_{band}", path, 0.0095, metal, sides=8, cap=False, flatten=0.45))

    bands.append(
        core.revolve(
            "lantern_eye",
            [(0.0, 0.162), (0.014, 0.156), (0.018, 0.14), (0.026, 0.128), (0.0, 0.124)],
            metal,
            segments=12,
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


def _collars(head):
    """Three collars and no heads.

    Each stands low around a dark stump where the neck was cut, below the
    crowns of the pauldrons, so that the notch between them stays empty.
    One small ember burns on the stump, out of sight from the front. The
    collar is what the game turns in place of a head.
    """

    base = SHOULDER_HEIGHT - NECK_HEIGHT
    fabric = core.material("cloth")

    def cut(turn):
        """A plain standing collar cut straight round, a little higher at
        the nape."""

        return 0.1 + 0.025 * (1 + math.cos(turn)) / 2

    def turned(turn):
        """A riding coat's collar turned down: low, and open at the throat."""

        return 0.07 + 0.02 * (1 + math.cos(turn)) / 2

    variants = [
        dict(rise=cut, gap=0.0, flare=0.32),
        dict(rise=turned, gap=1.3, flare=1.05),
    ]

    for index in range(3):
        group = core.empty(f"head_{index}", head)

        if index < 2:
            variant = variants[index]

            def point(u, v, variant=variant):
                # `turn` runs from one side of the throat, round the nape, to
                # the other; 0 is the back of the neck.
                turn = (u * 2 - 1) * (math.pi - variant["gap"] / 2)
                gain = 1 + variant["flare"] * v**1.6

                return (
                    math.sin(turn) * 0.165 * gain,
                    math.cos(turn) * 0.138 * gain + 0.012,
                    base + variant["rise"](turn) * v,
                )

            collar = _shell(f"collar_{index}", fabric, point, 80, 6, 0.012, group, closed=variant["gap"] == 0)
        else:
            collar = _ruff("collar_2", fabric, group)

        finish(_crisp(collar, 50), hollow=_whole, seed=index)

        # The stump is cut flat, so that nothing inside the collar can be
        # taken for a head. It closes the neck of the coat.
        stump = core.revolve(
            f"void_{index}",
            [(0.0, -0.035), (0.11, -0.035), (0.128, -0.045), (0.132, -0.09)],
            core.material("void"),
            group,
            segments=32,
        )
        stump.data.transform(Matrix.Diagonal((1, 0.86, 1, 1)))

        # The ember sits on the stump, small: the hollow's light is in its
        # skull's eyes. It is built around its own origin so the game can
        # scale it by the genome's gaze without moving it.
        ember = core.sculpt(
            f"eyes_{index}",
            [
                ("sphere", (0, 0, 0), (0.02, 0.016, 0.008)),
                ("sphere", (0.016, 0.008, 0.002), (0.011, 0.009, 0.006)),
            ],
            core.material("accent"),
            group,
            voxel=0.004,
            smooth=1,
        )
        ember.location = (0, 0.02, -0.03)

        if index == 1:
            _neck_ring(group)


def _ruff(name, mat, group):
    """A pleated ruff worn on the neck that is no longer there: a flat wheel
    of figure-of-eight folds round the stump, with nothing above it."""

    pleats = 18
    columns = pleats * 6
    rows = 8
    rings = []

    for row in range(rows + 1):
        # Round the wheel's outer face, from under it to over it.
        phi = math.radians(-118 + 236 * row / rows)
        ring = []

        for column in range(columns):
            turn = math.tau * column / columns
            lobe = abs(math.sin(turn * pleats / 2)) ** 0.6
            reach = 0.155 + (0.048 + 0.026 * lobe) * math.cos(phi)
            rise = (0.036 + 0.008 * lobe) * math.sin(phi)

            ring.append((math.sin(turn) * reach, math.cos(turn) * reach * 0.88 + 0.01, -0.04 + rise + 0.012 * math.cos(turn)))

        rings.append(ring)

    ruff = core.loft(name, rings, mat, group)

    return core.apply_modifier(ruff, "SOLIDIFY", thickness=0.008, offset=-1.0)


def _neck_ring(group):
    """An iron ring that was locked around the neck, broken open at one side
    of the throat and left lying crooked in the collar."""

    path = []

    for step in range(57):
        # The break is the part of the circle the ring never covers.
        turn = -2.2 + (math.tau - 0.6) * step / 56

        path.append((math.cos(turn) * 0.142, math.sin(turn) * 0.122 + 0.004, -0.03 - 0.012 * math.sin(turn + 0.5)))

    ring = core.tube(
        "neck_ring",
        path,
        # Each broken end is drawn out thin where the iron tore.
        lambda t: 0.018 * (1 - 0.7 * max(0.0, abs(2 * t - 1) - 0.88) / 0.12),
        core.material("metal"),
        group,
        sides=10,
        flatten=0.75,
    )
    finish(core.shade_smooth(ring), hollow=_whole, wear_boost=1.8)


def _mantles(root):
    """What sits on a hollow's shoulders, under its pauldrons and below the
    notch between them: a gorget, a cape or chains. None rises past the
    collar, and none breaks the symmetry the lantern alone breaks."""

    metal = core.material("metal")

    # An iron gorget: two lames round the cut neck, the lower one drawn down
    # to a point over the breastbone.
    group = core.empty("mantle_0", root)
    lames = []

    for index, (top, bottom, inner, outer, point) in enumerate(
        (
            (1.73, 1.655, (0.145, 0.12), (0.272, 0.172), 0.0),
            (1.672, 1.585, (0.24, 0.16), (0.3, 0.19), 0.085),
        )
    ):

        def at(u, v, top=top, bottom=bottom, inner=inner, outer=outer, point=point):
            # 0 is the front of the throat; the gorget is hinged open at the
            # nape.
            turn = (u * 2 - 1) * (math.pi - 0.14)
            front = math.cos(turn / 2) ** 8
            bulge = 0.014 * math.sin(math.pi * v)
            width = inner[0] + (outer[0] - inner[0]) * v + bulge
            depth = inner[1] + (outer[1] - inner[1]) * v + bulge

            return (math.sin(turn) * width, -math.cos(turn) * depth + 0.01, top + (bottom - top) * v - point * front * v**1.4)

        lames.append(_shell(f"gorget_{index}", metal, at, 72, 4, 0.007))

    gorget = _crisp(core.join(lames, "gorget"), 50)
    gorget.parent = group
    finish(gorget, wear_boost=1.8)

    # A cape hung from the shoulders down the back, under the pauldrons, and
    # held across the chest by a chain.
    group = core.empty("mantle_1", root)
    stops = core.bezier([(0.2, 0.13, 1.69), (0.3, 0.165, 1.58), (0.36, 0.22, 1.3), (0.4, 0.28, 0.9), (0.44, 0.32, 0.55)], samples=6)

    def cape(u, v):
        # From one side of the chest, round the back, to the other; 0 is
        # the middle of the back.
        turn = (u * 2 - 1) * math.radians(130)
        torn = 0.16 * abs(math.sin(turn * 5 + 1)) * abs(math.sin(turn * 2.3 + 2))
        reach = v * (1 - torn * v)
        position = reach * (len(stops) - 1)
        index = min(int(position), len(stops) - 2)
        width, thickness, height = stops[index].lerp(stops[index + 1], position - index)
        fold = 1 + 0.08 * reach * (0.6 * math.sin(turn * 5 + 0.7) + 0.4 * math.sin(turn * 9 + 2.1))

        return (math.sin(turn) * width * fold, math.cos(turn) * thickness * fold + 0.01, height)

    finish(core.loft("cape", [[cape(column / 72, row / 20) for column in range(73)] for row in range(21)], core.material("cloth"), group, closed=False), seed=3)

    path = []

    for step in range(61):
        share = step / 30 - 1
        sag = 1 - share * share
        path.append(Vector((0.15 * share, -0.14 - 0.035 * sag, 1.665 - 0.06 * sag)))

    clasp = _chain("cape_clasp", path, 0.05, 0.009)
    clasp.parent = group
    finish(clasp, wear_boost=1.6)

    # Chains from shoulder to shoulder, under the pauldrons: two across the
    # chest and one across the back, in links big enough to read.
    group = core.empty("mantle_2", root)
    swags = []

    for middle, spread, sag in ((270, 62, 0.12), (270, 70, 0.26), (90, 66, 0.18)):
        path = []

        for step in range(121):
            share = step / 60 - 1
            path.append(_coat_point(math.radians(middle + spread * share), 1.64 - sag * (1 - share * share), 0.03))

        swags.append(_chain(f"swag_{len(swags)}", path, 0.07, 0.011))

    chains = core.join(swags, "chains")
    chains.parent = group
    finish(chains, wear_boost=1.6)
