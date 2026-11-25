"""The hollow: the rider with no head.

Silhouette: two great pauldrons whose cops rise into points above the
shoulders, with nothing between them but an open collar; a riding coat
split at the front; and the missing head carried low at the off hand as a
lantern. The pauldrons' inner edges climb from the collar to their points,
so the outline's top is a deep V with a hole at the bottom of it: the void
where a head should be is the read (docs/art-bible.md 6.2).

Three families show: the coat and its collar (`cloth`), the iron (`metal`)
and the skull and hands (`bone`). The skull is the one bone mass, larger
than life and cut into planes, and the light is in its eyes. The lantern is
the figure's one asymmetry: the shoulders, collars and mantles are square
and alike.
"""

import math

import bmesh
import bpy
from mathutils import Matrix, Quaternion, Vector, noise

from lib import anatomy, cloth, core, motion
from lib.figure import NECK_HEIGHT, SHOULDER_HEIGHT, finish

# The coat's (depth, half-width, half-depth) stops: broad and square at the
# shoulder, drawn in at the waist without pinching it, then falling straight
# to skirts that part at the front rather than flare.
COAT = ((0.0, 0.125, 0.1), (0.04, 0.29, 0.15), (0.15, 0.26, 0.16), (0.37, 0.18, 0.13), (0.56, 0.225, 0.165), (1.0, 0.34, 0.27))
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

# Where the eyes burn in the skull's own frame. lib/face.py centres each
# eye 0.041 either side of the middle at this height and samples how deep to
# seat it at 0.036, so the sockets are cut round both.
EYE_X = 0.041
EYE_Z = -0.010

# The neck, cut: the collar stands round a hole this far across.
NECK = (0.165, 0.138)


def build():
    """The rider with no head: great pauldrons, a riding coat, and a
    lantern where the head went."""

    root = core.empty("reaper")

    # The skirts trail behind the rider, so every hem leans back.
    hems = [
        dict(tatter=0.26, tears=5, slits=6, fold_depth=0.12, seed=31, lean=0.14),
        dict(tatter=0.4, tears=8, slits=10, fold_depth=0.14, seed=32, lean=0.2),
        dict(tatter=0.18, tears=4, slits=3, fold_depth=0.1, seed=33, lean=0.1),
        dict(tatter=0.34, tears=5, slits=8, fold_depth=0.15, seed=34, lean=0.3),
    ]

    for index, hem in enumerate(hems):
        coat = cloth.robe(
            f"robe_{index}",
            core.material("cloth"),
            root,
            top=COAT_TOP,
            hem=COAT_HEM,
            profile=COAT,
            folds=(3, 5, 11),
            rings=40,
            segments=104,
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
    edges well apart, as a riding coat is cut to sit astride: the skirts
    part rather than flare.

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
        near = 1 - abs(vertex.co.x) / 0.2
        middle = lean * ((COAT_TOP - vertex.co.z) / (COAT_TOP - COAT_HEM)) ** 2

        if below <= 0 or near <= 0 or vertex.co.y > middle:
            continue

        if abs(vertex.co.x) < 1e-6:
            seam.append(vertex)
            continue

        # The edges part fast below the waist and hang straight below the
        # knee, each turned back on itself like a lapel.
        part = min(1.0, below / 0.45) ** 0.8
        shift = Vector((math.copysign(0.15, vertex.co.x), 0.07, 0)) * part * near**0.7
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


# Planes ------------------------------------------------------------------


def _outward(obj):
    """Turns every face of a closed mesh to face out."""

    mesh = bmesh.new()
    mesh.from_mesh(obj.data)
    bmesh.ops.recalc_face_normals(mesh, faces=mesh.faces)
    mesh.to_mesh(obj.data)
    mesh.free()

    return obj


def _solid(name, rings, mat, parent=None):
    """A closed solid skinned over `rings`, equal-length loops from top to
    bottom, capped at both ends: a carved block with flat faces."""

    mesh = bmesh.new()
    rows = [[mesh.verts.new(point) for point in ring] for ring in rings]
    count = len(rows[0])

    for upper, lower in zip(rows, rows[1:]):
        for index in range(count):
            follow = (index + 1) % count
            mesh.faces.new((upper[index], lower[index], lower[follow], upper[follow]))

    mesh.faces.new(rows[0])
    mesh.faces.new(list(reversed(rows[-1])))

    return core.object_from_bmesh(name, mesh, mat, parent, smooth=False)


def _prism(name, outline, near, far):
    """A cutter: `outline`, (x, z) pairs, pushed straight back from y `near`
    to y `far`."""

    return _solid(name, [[(x, near, z) for x, z in outline], [(x, far, z) for x, z in outline]], core.material("bone"))


def _boolean(target, tools, operation):
    """Unions `tools` into `target`, or cuts them out of it, and removes
    them. The parts are small closed blocks, so the exact solver is sure."""

    for tool in tools:
        modifier = target.modifiers.new("Boolean", "BOOLEAN")
        modifier.operation = operation
        modifier.object = tool
        modifier.solver = "EXACT"
        modifier.use_hole_tolerant = True
        core.activate(target)
        bpy.ops.object.modifier_apply(modifier=modifier.name)
        mesh = tool.data
        bpy.data.objects.remove(tool, do_unlink=True)
        bpy.data.meshes.remove(mesh)

    return target


def _plate(name, sections, thickness, mat):
    """Forged sheet over `sections`, open polylines of equal length laid
    side by side, given thickness and turned to face out. Each section's
    middle point is the plate's keel: a hard ridge the light breaks on."""

    rows = [[Vector(point) for point in section] for section in sections]
    sheet = core.loft(name, rows, mat, closed=False)
    core.apply_modifier(sheet, "SOLIDIFY", thickness=thickness, offset=-1.0, use_even_offset=True)

    return _outward(sheet)


def _strap(name, points, centre, width, thickness, mat):
    """A flat iron strap laid along `points`, its broad side turned away
    from `centre`."""

    points = [Vector(point) for point in points]
    rings = []

    for index, point in enumerate(points):
        ahead = points[min(index + 1, len(points) - 1)]
        behind = points[max(index - 1, 0)]
        tangent = (ahead - behind).normalized()
        out = point - centre
        out = (out - tangent * out.dot(tangent)).normalized()
        side = tangent.cross(out).normalized()
        rings.append([point + side * width + out * thickness, point - side * width + out * thickness, point - side * width, point + side * width])

    return _solid(name, rings, mat)


# Arms --------------------------------------------------------------------


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
                sides=12,
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
    for the left): a cop and two lames stepping down the arm.

    The cop is keeled along its top, and the keel climbs in one straight
    line from the collar out over the shoulder to a point a hand's breadth
    above the collar, then falls away again down the outside of the arm, so
    that the two cops frame a wide V with the open neck at the bottom of it
    and each is a peak on the line of the shoulders rather than an ear at
    the end of it. Inboard of the shoulder the keel stays under the head's
    height, and the point is thin from front to back, so from the side the
    shoulders stay a line with a blade on it rather than a hood. Every plate
    is a few flat facets with one hard keel."""

    metal = core.material("metal")

    # The cop, as sections across the keel from the collar outward:
    # (out, keel height, flank height, half-depth). The two middle points of
    # each section turn the flanks once, so each side is two flat facets.
    cop = []

    for out, keel, flank, half in (
        (-0.15, 0.045, 0.02, 0.09),
        (-0.06, 0.09, -0.01, 0.15),
        (0.03, 0.135, -0.045, 0.16),
        (0.1, 0.17, -0.07, 0.09),
        (0.14, 0.205, -0.09, 0.025),
        (0.2, 0.12, -0.12, 0.07),
        (0.245, 0.035, -0.14, 0.1),
    ):
        x = side * out
        shoulder = keel - 0.38 * (keel - flank)
        cop.append(
            [
                (x, -half, flank),
                (x, -half * 0.58, shoulder),
                (x, 0.0, keel),
                (x, half * 0.58, shoulder),
                (x, half, flank),
            ]
        )

    parts = [_plate(f"{name}_cop", cop, 0.009, metal)]

    # Two lames round the upper arm, below the cop, each narrower than the
    # one above and keeled on its outer side.
    along = Vector((side * 0.33, 0.0, -0.94)).normalized()
    out = Vector((side * 0.94, 0.0, 0.33)).normalized()
    across = Vector((0.0, 1.0, 0.0))

    for index, (start, end, radius) in enumerate(((0.1, 0.19, 0.118), (0.17, 0.26, 0.104))):
        sections = []

        for share, flare in ((0.0, 1.0), (1.0, 1.1)):
            centre = along * (start + (end - start) * share)
            section = []

            for turn in (-105, -52, 0, 52, 105):
                angle = math.radians(turn)
                reach = radius * flare * (1 + 0.16 * (1 - abs(turn) / 105))
                section.append(centre + (out * math.cos(angle) + across * math.sin(angle)) * reach)

            sections.append(section)

        parts.append(_plate(f"{name}_lame_{index}", sections, 0.008, metal))

    plates = core.join(parts, name)

    return core.crisp(plates, 0.003, angle=20, segments=1)


def _idle_turn(chain, calling="hollow"):
    """How far the idle pose's first key turns the end of `chain`, the
    joints from the waist out to a hand, read from lib/motion.py itself so
    that nothing here repeats its numbers."""

    _, rotations = motion.performance("idle", 0.0, False, calling)
    turn = Quaternion()

    for joint in chain:
        turn = turn @ rotations[joint]

    return turn


def _lantern(wrist):
    """The hollow's own head, carried as a lantern: a skull nearly twice the
    size of life on an iron bail, hung on two big links from a stirrup."""

    metal = core.material("metal")
    lantern = core.empty("lantern", wrist)

    # A lantern hangs plumb. It is turned back by however far the idle pose
    # carries the off hand, so that it hangs straight down in the stance
    # every screen shows, and swings with the hand from there.
    lantern.rotation_mode = "QUATERNION"
    lantern.rotation_quaternion = _idle_turn(("motion", "spine", "arm_l", "elbow_l", "wrist_l")).inverted()

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

    chain = _chain("lantern_chain", [Vector((0, 0, -0.07 - 0.006 * step)) for step in range(16)], 0.075, 0.011)
    chain.parent = lantern
    finish(chain, hollow=_whole, wear_boost=1.6)

    # Hung by the eye on its crown from the lower link, and turned a little
    # out from the body, so that both sockets face the viewer from the front.
    carried = core.empty("lantern_head", lantern, location=(0, 0, -0.17 - 0.16 * SKULL_SCALE))
    carried.rotation_euler = (0, 0, math.radians(10))
    carried.scale = (SKULL_SCALE, SKULL_SCALE, SKULL_SCALE)

    skull = _skull("lantern_skull")

    # The light burns at the back of both sockets. lib/face.py replaces it
    # with the eyes, so it is here for the authoring scene alone.
    flame = core.sculpt(
        "lantern_flame",
        [("sphere", (side * EYE_X, -0.05, EYE_Z), (0.016, 0.01, 0.012)) for side in (-1, 1)],
        core.material("accent"),
        voxel=0.005,
    )

    # Iron round the skull: a bail of two thin straps, one up each temple,
    # meeting at the eye on its crown that the chain hangs from. No band
    # rings the skull, so it never reads as a banded ball.
    centre = Vector((0, CRANIUM[0][1], CRANIUM[0][2]))
    straps = []

    for side in (-1, 1):
        path = []

        for step in range(9):
            # From behind the orbit, at the temple, up over the crown.
            rise = math.radians(-12 + 102 * step / 8)
            path.append(
                (
                    side * (CRANIUM[1][0] + 0.007) * math.cos(rise),
                    centre.y + 0.004,
                    centre.z + (CRANIUM[1][2] + 0.007) * math.sin(rise),
                )
            )

        straps.append(_strap(f"lantern_strap_{side}", path, centre, 0.009, 0.006, metal))

    eye = core.revolve(
        "lantern_eye",
        [(0.0, 0.17), (0.013, 0.165), (0.017, 0.152), (0.024, 0.138), (0.0, 0.132)],
        metal,
        segments=10,
    )
    eye.location = (0, centre.y + 0.004, 0)
    core.activate(eye)
    bpy.ops.object.transform_apply(location=True, rotation=False, scale=False)
    straps.append(eye)
    harness = core.crisp(core.join(straps, "lantern_harness"), 0.002, angle=40, segments=1)

    for part in (skull, flame, harness):
        part.parent = carried

    finish(skull, wear_boost=1.4)
    finish(harness, hollow=_whole, wear_boost=1.6)


# The cranium: (centre, half-axes) of the faceted dome a skull is built on.
CRANIUM = ((0.0, 0.03, 0.038), (0.1, 0.118, 0.104))


def _skull(name):
    """A skull carved in planes rather than modelled in clay: a faceted
    cranium, a face block with a flat front and square cheekbones, and a jaw
    block, fused, with two great orbits, a nose and a row of four teeth cut
    into them. About 1,000 triangles, every one of them on a plane a band
    edge can run along."""

    bone = core.material("bone")
    (cx, cy, cz), (rx, ry, rz) = CRANIUM

    # The cranium: twelve facets round, six courses from the crown down to
    # the occiput.
    rings = []

    for latitude in (78, 56, 32, 8, -18, -42):
        lift = math.radians(latitude)
        ring = []

        for step in range(12):
            turn = math.tau * (step + 0.5) / 12
            ring.append((cx + math.cos(turn) * rx * math.cos(lift), cy + math.sin(turn) * ry * math.cos(lift), cz + rz * math.sin(lift)))

        rings.append(ring)

    skull = _solid(name, rings, bone)

    def block(courses):
        """A block from (height, front, half-width, back) courses, its front
        a flat plane turned at the corners."""

        rings = []

        for height, front, half, back in courses:
            rings.append(
                [
                    (0.0, front, height),
                    (half * 0.6, front + 0.004, height),
                    (half, front + 0.034, height),
                    (half * 0.92, back, height),
                    (0.0, back + 0.01, height),
                    (-half * 0.92, back, height),
                    (-half, front + 0.034, height),
                    (-half * 0.6, front + 0.004, height),
                ]
            )

        return _solid(f"{name}_block", rings, bone)

    # The face: a heavy brow over the orbits, the cheekbones the widest
    # part of the face, and the upper jaw narrowing under them.
    face = block(
        (
            (0.055, -0.07, 0.07, 0.0),
            (0.016, -0.106, 0.09, 0.0),
            (-0.034, -0.096, 0.1, 0.0),
            (-0.07, -0.09, 0.06, 0.0),
        )
    )

    # The jaw: a square block, narrower at the chin.
    jaw = block(
        (
            (-0.06, -0.088, 0.054, -0.005),
            (-0.122, -0.08, 0.036, -0.03),
        )
    )

    _boolean(skull, [face, jaw], "UNION")

    cutters = []

    for side in (-1, 1):
        # A great orbit, higher at its outer corner than its inner, so the
        # brow comes down over the nose in a scowl.
        outline = [(-0.026, -0.006), (-0.017, 0.019), (0.017, 0.027), (0.031, 0.008), (0.023, -0.024), (-0.012, -0.027)]
        cutters.append(_prism(f"{name}_orbit_{side}", [(side * (EYE_X + u), EYE_Z + z) for u, z in outline], -0.2, -0.044))

    # The nose: a pear turned upside down.
    cutters.append(_prism(f"{name}_nose", [(0.0, -0.02), (0.015, -0.042), (0.009, -0.052), (-0.009, -0.052), (-0.015, -0.042)], -0.2, -0.066))

    # The line of the mouth, with the gaps that make four big teeth of the
    # bone above it and four below, cut as one comb.
    lower, upper = [(-0.052, -0.072)], [(0.048, -0.065)]

    for gap in (-0.026, 0.0, 0.026):
        lower += [(gap - 0.003, -0.072), (gap - 0.003, -0.094), (gap + 0.003, -0.094), (gap + 0.003, -0.072)]

    for gap in (0.026, 0.0, -0.026):
        upper += [(gap + 0.003, -0.065), (gap + 0.003, -0.055), (gap - 0.003, -0.055), (gap - 0.003, -0.065)]

    cutters.append(_prism(f"{name}_mouth", lower + [(0.052, -0.072)] + upper + [(-0.048, -0.065)], -0.2, -0.074))

    _boolean(skull, cutters, "DIFFERENCE")
    skull.data.validate()

    # A solver that loses its way leaves a few stray faces rather than
    # failing; a skull with its face cut in has well over a hundred.
    if len(skull.data.polygons) < 100:
        raise RuntimeError(f"{name}: carving the skull left {len(skull.data.polygons)} faces")

    return core.crisp(skull, 0.0025, angle=26, segments=1)


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


# Collars ---------------------------------------------------------------


def _collars(head):
    """Three collars and no heads.

    Each is a ragged ring round the cut neck, low at the throat and high at
    the nape, below the points of the pauldrons, so that the V between them
    ends in a hole. The hole is lined in void, which prints flat key with no
    line: from the front you look over the collar's lip into the dark. One
    small ember burns at the bottom of it, out of sight from the front. The
    collar is what the game turns in place of a head.
    """

    base = SHOULDER_HEIGHT - NECK_HEIGHT
    fabric = core.material("cloth")

    def ragged(turn, seed):
        """Torn points along a collar's lip, deepest where the cloth is
        thinnest, never cut lower than half its height."""

        return 0.022 * abs(math.sin(turn * 5.5 + seed)) ** 3 + 0.01 * abs(math.sin(turn * 11 + seed * 2.0))

    def cut(turn):
        """A standing collar, cut down in a V at the throat and at the nape
        and highest under the pauldrons, so that from any side the notch
        between the pauldrons runs down into it."""

        return 0.045 + 0.05 * abs(math.sin(turn)) ** 1.2

    def turned(turn):
        """A riding coat's collar turned down: open at the throat, and
        standing a little at either side."""

        return 0.04 + 0.04 * abs(math.sin(turn)) ** 1.2

    variants = [
        dict(rise=cut, gap=0.0, flare=0.4, seed=0.6),
        dict(rise=turned, gap=1.3, flare=1.0, seed=2.1),
    ]

    for index in range(3):
        group = core.empty(f"head_{index}", head)

        if index < 2:
            variant = variants[index]

            def point(u, v, variant=variant, inset=0.0, torn=True):
                # `turn` runs from one side of the throat, round the nape, to
                # the other; 0 is the back of the neck.
                turn = (u * 2 - 1) * (math.pi - variant["gap"] / 2)
                gain = 1 + variant["flare"] * v**1.6
                rise = variant["rise"](turn) - (ragged(turn, variant["seed"]) if torn else 0.012)

                return (
                    math.sin(turn) * (NECK[0] * gain - inset),
                    math.cos(turn) * (NECK[1] * gain - inset) + 0.012,
                    base + rise * v,
                )

            collar = _shell(f"collar_{index}", fabric, point, 72, 4, 0.012, group, closed=variant["gap"] == 0)

            def lining(turn, variant=variant):
                # The void follows the collar's inside face up to just under
                # its lip, and stands low where a turned collar is open.
                u = (turn / (math.pi - variant["gap"] / 2) + 1) / 2
                inside = abs(turn) <= math.pi - variant["gap"] / 2

                if inside:
                    return Vector(point(u, 1.0, inset=0.02, torn=False)), Vector(point(u, 0.0, inset=0.02, torn=False))

                return Vector((math.sin(turn) * NECK[0] * 0.86, math.cos(turn) * NECK[1] * 0.86 + 0.012, base + 0.012)), None
        else:
            collar = _ruff("collar_2", fabric, group)

            def lining(turn):
                return Vector((math.sin(turn) * 0.112, math.cos(turn) * 0.1 + 0.012, base + 0.02)), None

        finish(core.crisp(collar, 0.002, angle=50, segments=1), hollow=_whole, seed=index)
        _hole(f"void_{index}", lining, base, group)

        # The ember sits at the bottom of the hole, small: the hollow's
        # light is in its skull's eyes. It is built around its own origin
        # so the game can scale it by the genome's gaze without moving it.
        ember = core.sculpt(
            f"eyes_{index}",
            [
                ("sphere", (0, 0, 0), (0.02, 0.016, 0.008)),
                ("sphere", (0.016, 0.008, 0.002), (0.011, 0.009, 0.006)),
            ],
            core.material("accent"),
            group,
            voxel=0.004,
        )
        ember.location = (0, 0.02, base - 0.05)

        if index == 1:
            _neck_ring(group)


def _hole(name, lining, base, group):
    """The dark inside a collar: a funnel of void from just under the lip
    down into the coat, where the neck was. `lining(turn)` gives the lip and
    the foot of its wall at `turn` round the neck."""

    rings = [[], [], [], []]

    for step in range(32):
        turn = -math.pi + math.tau * (step + 0.5) / 32
        lip, foot = lining(turn)
        foot = foot if foot is not None else Vector((lip.x, lip.y, base))
        rings[0].append(lip)
        rings[1].append(foot)
        rings[2].append(Vector((foot.x * 0.62, foot.y * 0.62, base - 0.06)))
        rings[3].append(Vector((foot.x * 0.2, foot.y * 0.2, base - 0.11)))

    hole = core.loft(name, [[tuple(point) for point in ring] for ring in rings], core.material("void"), group, cap_end=True)

    # Given thickness, so it shows a face to the eye looking down into it
    # as well as to one outside it.
    core.apply_modifier(hole, "SOLIDIFY", thickness=0.004, offset=0.0)

    return _outward(hole)


def _shell(name, mat, point, columns, rows, thickness, parent=None, closed=False):
    """A sheet with real thickness: a collar, or a plate of armour.

    `point(u, v)` maps the unit square onto the sheet's outer face. A
    `closed` sheet wraps round on itself at u = 1, with no seam.
    """

    span = columns if closed else columns + 1
    rings = [[point(column / columns, row / rows) for column in range(span)] for row in range(rows + 1)]
    sheet = core.loft(name, rings, mat, parent, closed=closed)

    return core.apply_modifier(sheet, "SOLIDIFY", thickness=thickness, offset=-1.0)


def _ruff(name, mat, group):
    """A pleated ruff worn on the neck that is no longer there: a flat wheel
    of figure-of-eight folds round the hole, with nothing above it. Twelve
    pleats, each big enough to take a band of its own."""

    pleats = 12
    columns = pleats * 6
    rows = 6
    rings = []

    for row in range(rows + 1):
        # Round the wheel's outer face, from under it to over it.
        phi = math.radians(-118 + 236 * row / rows)
        ring = []

        for column in range(columns):
            turn = math.tau * column / columns
            lobe = abs(math.sin(turn * pleats / 2)) ** 0.6
            reach = 0.16 + (0.052 + 0.03 * lobe) * math.cos(phi)
            rise = (0.04 + 0.01 * lobe) * math.sin(phi)

            ring.append((math.sin(turn) * reach, math.cos(turn) * reach * 0.88 + 0.01, -0.058 + rise + 0.012 * math.cos(turn)))

        rings.append(ring)

    ruff = core.loft(name, rings, mat, group)

    return core.apply_modifier(ruff, "SOLIDIFY", thickness=0.008, offset=-1.0)


def _neck_ring(group):
    """An iron ring that was locked around the neck, broken open at one side
    of the throat and left lying crooked in the collar."""

    path = []

    for step in range(41):
        # The break is the part of the circle the ring never covers.
        turn = -2.2 + (math.tau - 0.6) * step / 40

        path.append((math.cos(turn) * 0.142, math.sin(turn) * 0.122 + 0.004, -0.03 - 0.012 * math.sin(turn + 0.5)))

    ring = core.tube(
        "neck_ring",
        path,
        # Each broken end is drawn out thin where the iron tore.
        lambda t: 0.018 * (1 - 0.7 * max(0.0, abs(2 * t - 1) - 0.88) / 0.12),
        core.material("metal"),
        group,
        sides=8,
        flatten=0.75,
    )
    finish(core.shade_smooth(ring), hollow=_whole, wear_boost=1.8)


# Mantles ---------------------------------------------------------------


def _mantles(root):
    """What sits on a hollow's shoulders, under its pauldrons and below the
    V between them: a gorget, a cape or a chain. None rises past the collar,
    and none breaks the symmetry the lantern alone breaks."""

    metal = core.material("metal")

    # An iron gorget: two lames round the cut neck, the lower one drawn down
    # to a point over the breastbone.
    group = core.empty("mantle_0", root)
    lames = []

    for index, (top, bottom, inner, outer, point) in enumerate(
        (
            (1.7, 1.635, (0.15, 0.125), (0.272, 0.172), 0.0),
            (1.655, 1.575, (0.24, 0.16), (0.3, 0.19), 0.085),
        )
    ):

        def at(u, v, top=top, bottom=bottom, inner=inner, outer=outer, point=point):
            # 0 is the front of the throat; the gorget is hinged open at the
            # nape.
            turn = (u * 2 - 1) * (math.pi - 0.14)
            front = math.cos(turn / 2) ** 8
            width = inner[0] + (outer[0] - inner[0]) * v
            depth = inner[1] + (outer[1] - inner[1]) * v

            return (math.sin(turn) * width, -math.cos(turn) * depth + 0.01, top + (bottom - top) * v - point * front * v**1.4)

        lames.append(_shell(f"gorget_{index}", metal, at, 36, 2, 0.007))

    gorget = core.crisp(_outward(core.join(lames, "gorget")), 0.002, angle=30, segments=1)
    gorget.parent = group
    finish(gorget, wear_boost=1.8)

    # A cape hung from the shoulders down the back, under the pauldrons, and
    # held across the chest by three big links.
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
        fold = 1 + 0.08 * reach * (0.6 * math.sin(turn * 3 + 0.7) + 0.4 * math.sin(turn * 5 + 2.1))

        return (math.sin(turn) * width * fold, math.cos(turn) * thickness * fold + 0.01, height)

    finish(core.loft("cape", [[cape(column / 60, row / 18) for column in range(61)] for row in range(19)], core.material("cloth"), group, closed=False), seed=3)

    path = []

    for step in range(61):
        share = step / 30 - 1
        sag = 1 - share * share
        path.append(Vector((0.16 * share, -0.15 - 0.035 * sag, 1.665 - 0.07 * sag)))

    clasp = _chain("cape_clasp", path, 0.1, 0.014)
    clasp.parent = group
    finish(clasp, wear_boost=1.6)

    # One great chain slung from shoulder to shoulder across the chest and
    # one across the back, under the pauldrons, in four links each: big
    # enough to read as a chain when the reaper is a thumb tall.
    group = core.empty("mantle_2", root)
    swags = []

    for middle, spread, sag in ((270, 64, 0.22), (90, 60, 0.16)):
        path = []

        for step in range(121):
            share = step / 60 - 1
            path.append(_coat_point(math.radians(middle + spread * share), 1.64 - sag * (1 - share * share), 0.035))

        swags.append(_chain(f"swag_{len(swags)}", path, 0.125, 0.017))

    chains = core.join(swags, "chains")
    chains.parent = group
    finish(chains, wear_boost=1.6)
