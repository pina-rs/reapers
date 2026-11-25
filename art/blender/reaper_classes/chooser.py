"""The chooser: the one who picks who falls, then strikes once.

Silhouette: a pair of carrion wings on bone arms behind square shoulders, a
great faceted bone helm, and a spear stood at the weapon side like a
verdict already given. The outline reads as a tall "Y" with one hard
vertical beside it: the wings are the head and shoulder line, the spear is
the weapon side, and the spear is the figure's one asymmetry.

Three families show: the surcoat and the cloth at the throat (`cloth`),
the feathers (`feather`), and the helm, the wings' arms and the hands
(`bone`). Cobalt is the darkest light, so the bone carries the figure's
value (docs/art-bible.md 6.2). Each wing has seven large primaries, pointed
and longest at the tip, and every mantle is a pair of wings, so the class
reads as winged whatever it wears.
"""

import math

import bmesh
import bpy
from mathutils import Matrix, Vector

from lib import cloth, core
from lib.figure import HEAD_HEIGHT, NECK_HEIGHT, SHOULDER_HEIGHT, arms, eyes, finish, void_head

ROBE_TOP = SHOULDER_HEIGHT + 0.07
ROBE_HEM = 0.06

# A surcoat cut square at the shoulder, with a hard corner where the flat
# of the shoulder turns down the arm, close at the waist and falling
# straight to a wide hem: (depth, half-width, half-depth) from the collar.
SURCOAT = ((0.0, 0.12, 0.1), (0.028, 0.268, 0.132), (0.07, 0.272, 0.14), (0.2, 0.235, 0.135), (0.36, 0.162, 0.112), (0.52, 0.18, 0.135), (1.0, 0.29, 0.25))

# A broad belt round the waist, standing proud of the surcoat: the one
# break in the column, and a ledge for the off hand.
BELT = (1.06, 1.15, 0.024)

# The shoulders, square with the surcoat's corners.
SHOULDER_X = 0.25

# Every wing grows from behind its shoulder blade.
WING_ROOT = Vector((0.1, 0.11, SHOULDER_HEIGHT - 0.07))

# The middle of every helm. A helm is about 1.3 times a head, carved in
# planes, and the eye line lib/face.py draws the lights on runs through its
# visor (`EYE`, above the middle, for each helm in turn).
HELM_CENTRE = Vector((0, -0.006, HEAD_HEIGHT + 0.004))
EYE = (0.008, 0.0, 0.037)


def build():
    """The one who picks who falls: wings, a bone helm, an upright spear."""

    root = core.empty("reaper")
    cloth_mat = core.material("cloth")

    # Four hems. None sweeps or leans: the chooser stands upright, and its
    # one asymmetry is the spear.
    hems = [
        dict(tatter=0.2, tears=4, slits=4, fold_depth=0.1, seed=41),
        dict(tatter=0.34, tears=7, slits=8, fold_depth=0.12, seed=42),
        dict(tatter=0.16, tears=3, slits=2, fold_depth=0.09, seed=43),
        dict(tatter=0.28, tears=5, slits=10, fold_depth=0.13, seed=44),
    ]

    for index, hem in enumerate(hems):
        robe = cloth.robe(
            f"robe_{index}",
            cloth_mat,
            root,
            top=ROBE_TOP,
            hem=ROBE_HEM,
            profile=SURCOAT,
            folds=(3, 5, 11),
            rings=40,
            segments=96,
            **hem,
        )
        finish(_belt(robe, *BELT), seed=index)

    # The weapon hand closes on the haft with no turn of its own, as every
    # class's does, so the spear meets the thrust level with the ground
    # where lib/motion.py aims it.
    arms(
        root,
        # The upper arm hangs and the forearm points ahead, so the fist
        # stands out from the hip with the haft upright in it.
        weapon_side_path=[(-SHOULDER_X, 0.0, SHOULDER_HEIGHT - 0.03), (-0.335, 0.06, 1.31), (-0.38, -0.05, 1.25)],
        # The off hand closes into a fist at the belt.
        off_side_path=[(SHOULDER_X, 0.0, SHOULDER_HEIGHT - 0.03), (0.32, 0.07, 1.33), (0.27, 0.0, 1.16)],
        sleeve_shoulder=0.074,
        sleeve_cuff=0.082,
        droop=0.05,
        off_curl=1.0,
        tatter=0.12,
    )

    # A fist at the belt, the back of the hand turned out and the palm in
    # toward the thigh.
    fist = next(child for child in root.children_recursive if child.name == "arm_l_hand")
    fist.rotation_euler = (math.radians(92), math.radians(12), math.radians(-10))

    head = core.empty("head", root, location=(0, 0, NECK_HEIGHT))
    _heads(head)
    _mantles(root)

    return root


def _smooth(value):
    """Eases 0..1 in and out, and holds outside it."""

    value = max(0.0, min(1.0, value))

    return value * value * (3 - 2 * value)


def _belt(robe, low, high, proud):
    """Girds `robe` with a belt from height `low` to `high`, `proud` outside
    the surcoat: the cloth is cut along both edges, twice, so the belt
    steps out of it square, and the band is drawn true round the waist
    rather than following the folds.

    Every shape key moves with the cloth it belongs to, so the belt sways
    with the robe; a humanoid fit draws it back into the robe's surface."""

    mesh = bmesh.new()
    mesh.from_mesh(robe.data)

    for height in (low - 0.005, low, high, high + 0.005):
        geometry = mesh.verts[:] + mesh.edges[:] + mesh.faces[:]
        bmesh.ops.bisect_plane(mesh, geom=geometry, plane_co=(0, 0, height), plane_no=(0, 0, 1))

    shapes = mesh.verts.layers.shape.values()
    band = [vertex for vertex in mesh.verts if low - 1e-5 <= vertex.co.z <= high + 1e-5]

    for vertex in band:
        x, y = vertex.co.x, vertex.co.y
        turn = math.atan2(y, x)
        # The band's own ellipse: the widest the surcoat comes at this
        # height, so every fold is covered.
        width, depth = _radii(vertex.co.z)
        target = Vector((math.cos(turn) * (width * 1.035 + proud), math.sin(turn) * (depth * 1.035 + proud), vertex.co.z))
        shift = target - vertex.co
        vertex.co = target

        for shape in shapes:
            vertex[shape] += shift

    mesh.to_mesh(robe.data)
    mesh.free()

    return robe


def _radii(height):
    """The surcoat's half-width and half-depth at `height`, before folds."""

    depth = (ROBE_TOP - height) / (ROBE_TOP - ROBE_HEM)

    for (d0, w0, t0), (d1, w1, t1) in zip(SURCOAT, SURCOAT[1:]):
        if depth <= d1:
            blend = _smooth((depth - d0) / (d1 - d0))

            return w0 + (w1 - w0) * blend, t0 + (t1 - t0) * blend

    return SURCOAT[-1][1:]


# Planes ------------------------------------------------------------------


def _solid(name, rings, mat, cap_top=True, cap_bottom=True):
    """A block skinned over `rings`, equal-length loops from one end to the
    other, with flat faces."""

    mesh = bmesh.new()
    rows = [[mesh.verts.new(point) for point in ring] for ring in rings]
    count = len(rows[0])

    for upper, lower in zip(rows, rows[1:]):
        for index in range(count):
            follow = (index + 1) % count
            mesh.faces.new((upper[index], lower[index], lower[follow], upper[follow]))

    if cap_top:
        mesh.faces.new(rows[0])

    if cap_bottom:
        mesh.faces.new(list(reversed(rows[-1])))

    return core.object_from_bmesh(name, mesh, mat, smooth=False)


def _box(name, low, high):
    """A cutter: the box between corners `low` and `high`."""

    (x0, y0, z0), (x1, y1, z1) = low, high
    ring = lambda y: [(x0, y, z0), (x1, y, z0), (x1, y, z1), (x0, y, z1)]  # noqa: E731

    return _solid(name, [ring(y0), ring(y1)], core.material("bone"))


def _cut(target, cutters):
    """Cuts `cutters` out of `target` and removes them."""

    for cutter in cutters:
        modifier = target.modifiers.new("Cut", "BOOLEAN")
        modifier.operation = "DIFFERENCE"
        modifier.object = cutter
        modifier.solver = "EXACT"
        modifier.use_hole_tolerant = True
        core.activate(target)
        bpy.ops.object.modifier_apply(modifier=modifier.name)
        mesh = cutter.data
        bpy.data.objects.remove(cutter, do_unlink=True)
        bpy.data.meshes.remove(mesh)

    return target


# Feathers ---------------------------------------------------------------


def _skin(mesh, rings):
    """Adds a closed, capped solid through `rings` to `mesh`."""

    rows = [[mesh.verts.new(point) for point in ring] for ring in rings]
    count = len(rows[0])

    for upper, lower in zip(rows, rows[1:]):
        for index in range(count):
            follow = (index + 1) % count
            mesh.faces.new((upper[index], lower[index], lower[follow], upper[follow]))

    mesh.faces.new(rows[0])
    mesh.faces.new(list(reversed(rows[-1])))


def _feather(mesh, spine, facing, width, cup=0.22, shaft=0.007, lead=0.45, hand=1, notch=None, point=True):
    """Adds one feather to `mesh` along `spine`, its quill first: a broad
    blade either side of a raised shaft. A `point`ed feather is widest a
    quarter of the way out and tapers straight to a hard point, as a
    primary does; otherwise it holds its width and rounds off, as the
    feathers that lie over others do, so only the wingtip is sharp.

    `facing` is the way its upper side looks. `lead` is the leading vane's
    share of the trailing one, as on a flight feather, and `notch` (a share
    of the length) bites one nick out of the trailing edge.
    """

    facing = Vector(facing)
    count = len(spine) - 1
    rings = []

    for step, centre in enumerate(spine):
        t = step / count
        forward = (spine[min(step + 1, count)] - spine[max(step - 1, 0)]).normalized()
        normal = (facing - forward * facing.dot(forward)).normalized()
        # A right wing's feathers are its left's mirrored (`hand` -1).
        side = forward.cross(normal) * hand
        radius = shaft * (1 - 0.7 * t)
        if point:
            leaf = min(1.0, t / 0.14) ** 0.55 * max(0.0, 1 - max(0.0, t - 0.28) / 0.72) ** 0.8
        else:
            leaf = min(1.0, t / 0.16) ** 0.6 * max(0.0, 1 - t**2.6) ** 0.55

        trailing = max(width * leaf, radius * 0.5)

        if notch is not None and abs(t - notch) < 0.5 / count:
            trailing *= 0.6

        leading = max(width * leaf * lead, radius * 0.5)

        rings.append(
            [
                centre - side * (radius + trailing) - normal * cup * trailing,
                centre - side * (radius + trailing * 0.5) - normal * cup * trailing * 0.2,
                centre - side * radius,
                centre + normal * radius * 1.2,
                centre + side * radius,
                centre + side * (radius + leading * 0.5) - normal * cup * leading * 0.2,
                centre + side * (radius + leading) - normal * cup * leading,
                centre - normal * radius,
            ]
        )

    _skin(mesh, rings)


class _Plane:
    """A wing's own plane: `out` along the wing, `up` across it, and depth
    toward the figure's back. Points are given in it as (along, up)."""

    def __init__(self, side, sweep, root=WING_ROOT, tilt=0.0):
        turn = math.radians(sweep)
        self.side = side
        self.root = Vector((root.x * side, root.y, root.z))
        self.out = Vector((side * math.cos(turn), math.sin(turn), 0))
        self.up = Vector((0, 0, 1))
        self.back = self.up.cross(self.out).normalized() * side

        if tilt:
            # Leans the whole wing forward over the shoulder.
            self.up = (self.up + Vector((0, -math.tan(math.radians(tilt)), 0))).normalized()

    def at(self, along, up, depth=0.0):
        return self.root + self.out * along + self.up * up + self.back * depth

    def toward(self, degrees):
        """A direction in the plane, `degrees` up from straight out."""

        turn = math.radians(degrees)

        return self.out * math.cos(turn) + self.up * math.sin(turn)


def _wing(mesh, bones, side, sweep, edge, primaries, secondaries, coverts, tilt=0.0, droop=0.0, width=0.08):
    """Adds one wing to `mesh`, and its arm to `bones`.

    `edge` is the leading edge as (along, up) stops in the wing's plane,
    from the shoulder to the wrist and on to the hand. `primaries` and
    `secondaries` are (angle, length) pairs, the first rooted along the
    hand from the wrist outward, the second along the forearm from the
    shoulder to the wrist; `coverts` are laid over the secondaries' roots.
    Angles are in degrees up from straight out along the wing. Each primary
    turns `droop` degrees further down on its way to its tip.
    """

    plane = _Plane(side, sweep, tilt=tilt)
    line = [Vector(point) for point in core.bezier(edge, samples=6)]
    wrist = len(line) * 2 // 3

    def stop(share, start, end):
        position = start + (end - start) * share
        index = min(int(position), len(line) - 2)

        return line[index].lerp(line[index + 1], position - index)

    def spine(points, angle, length, depth, bow, turn):
        """A feather's spine from `points` in the plane, turning `turn`
        degrees down along its length and bowed `bow` toward the viewer."""

        along, up = points
        position = plane.at(along, up, depth)
        result = [position.copy()]
        segments = 9

        for step in range(segments):
            share = (step + 0.5) / segments
            heading = plane.toward(angle - turn * share**1.4)
            position = position + heading * (length / segments)
            result.append(position - plane.back * bow * length * 4 * share * (1 - share))

        return result

    # The arm of the wing, in bone: shoulder, elbow and wrist along the
    # leading edge, a knuckle at the wrist, and the alula's claw standing
    # forward from it, so the wing reads as a limb with feathers hung on
    # it rather than as a fan.
    arm = [plane.at(*point, -0.018) for point in line]
    bone = core.tube("_wing_arm", arm, lambda t: 0.03 - 0.014 * t + 0.012 * math.exp(-(((t - 0.66) / 0.06) ** 2)), core.material("bone"), sides=8)
    bones.from_mesh(bone.data)
    _remove(bone)
    knuckle = plane.at(*line[wrist], -0.018)
    lead = (line[min(wrist + 2, len(line) - 1)] - line[max(wrist - 2, 0)]).normalized()
    claw_axis = (plane.out * lead.x + plane.up * lead.y).normalized()
    claw = core.tube(
        "_wing_claw",
        [knuckle, knuckle + claw_axis * 0.06 - plane.up * 0.01, knuckle + claw_axis * 0.11 - plane.up * 0.03],
        lambda t: 0.017 * (1 - t) + 0.002,
        core.material("bone"),
        sides=8,
    )
    bones.from_mesh(claw.data)
    _remove(claw)

    for index, (angle, length) in enumerate(primaries):
        share = index / max(len(primaries) - 1, 1)
        points = spine(stop(share, wrist, len(line) - 1), angle, length, 0.012 + 0.005 * index, 0.03, droop)
        _feather(mesh, points, -plane.back, width * (1.0 + 0.12 * share), hand=side, notch=0.72 if index % 3 == 1 else None)

    for index, (angle, length) in enumerate(secondaries):
        share = (index + 0.5) / len(secondaries)
        points = spine(stop(share, 0, wrist), angle, length, 0.0, 0.025, droop * 0.4)
        _feather(mesh, points, -plane.back, width * 0.95, lead=0.6, hand=side, point=False)

    for index, (angle, length) in enumerate(coverts):
        share = (index + 0.35) / len(coverts)
        points = spine(stop(share, 0, wrist + 2), angle, length, -0.026, 0.08, 0.0)
        _feather(mesh, points, -plane.back, width * 0.8, lead=0.8, hand=side, point=False)


def _remove(obj):
    mesh = obj.data
    bpy.data.objects.remove(obj, do_unlink=True)
    bpy.data.meshes.remove(mesh)


def _scapulars(mesh, side):
    """Broad feathers laid over each shoulder like a pauldron's lames, in
    two courses, the upper over the lower: they carry the shoulder line out
    flat and square past the sleeve, and end in points."""

    for course, (height, reach, droop, width) in enumerate(((0.085, 0.24, -12, 0.095), (0.045, 0.27, -26, 0.105))):
        for index in range(3):
            turn = math.radians(-48 + 48 * index)
            outward = Vector((side * 0.8, math.sin(turn) * 0.6, 0)).normalized()
            base = Vector((side * 0.13, 0.02 + math.sin(turn) * 0.06, SHOULDER_HEIGHT + height))
            tilt = math.radians(droop - 8 * abs(math.sin(turn)))
            direction = outward * math.cos(tilt) + Vector((0, 0, math.sin(tilt)))
            facing = Vector((0, -0.15, 1)) + outward * 0.3
            spine = [base + direction * reach * step / 6 + facing.normalized() * 0.018 * math.sin(math.pi * step / 6) for step in range(7)]
            _feather(mesh, spine, facing, width, cup=0.3, lead=1.0, hand=side, point=False)


def _mantle(index, root, build_wing):
    """A pair of wings as mantle `index`, mirrored across the back, with
    their arms in bone."""

    group = core.empty(f"mantle_{index}", root)
    mesh = bmesh.new()
    bones = bmesh.new()

    for side in (-1, 1):
        build_wing(mesh, bones, side)
        _scapulars(mesh, side)

    wings = core.object_from_bmesh(f"wing_{index}", mesh, core.material("feather"), group)
    finish(core.shade_smooth(wings, angle=40), wear_boost=1.4, seed=index)
    arms = core.object_from_bmesh(f"wing_{index}_bone", bones, core.material("bone"), group)
    finish(core.shade_smooth(arms), wear_boost=1.6, seed=index)


def _mantles(root):
    """Three pairs of wings."""

    # Raised: each wing climbs from the shoulder to a wrist level with the
    # crown, and its seven primaries stand close and overlapping, longest
    # at the outer edge, so the wing ends in one hard point up and out.
    def raised(mesh, bones, side):
        _wing(
            mesh,
            bones,
            side,
            sweep=24,
            edge=[(0.0, 0.0), (0.2, 0.14), (0.34, 0.32), (0.4, 0.42)],
            primaries=[(98, 0.52), (90, 0.57), (82, 0.62), (74, 0.67), (66, 0.72), (58, 0.8), (51, 0.84)],
            secondaries=[(-56, 0.44), (-38, 0.48), (-20, 0.5), (-2, 0.52), (16, 0.52)],
            coverts=[(-44, 0.26), (-26, 0.28), (-8, 0.29), (10, 0.28), (28, 0.27)],
            droop=10,
            width=0.085,
        )

    _mantle(0, root, raised)

    # Folded: the wrist stands over each shoulder like a horn, and the
    # primaries fall from it outside the arms to the knee, the outer two
    # longest, crossing at their points.
    def folded(mesh, bones, side):
        _wing(
            mesh,
            bones,
            side,
            sweep=14,
            edge=[(0.0, 0.0), (0.11, 0.28), (0.2, 0.5), (0.23, 0.56)],
            primaries=[(-96, 0.74), (-92, 0.8), (-88, 0.86), (-85, 0.92), (-82, 0.98), (-79, 1.06), (-76, 1.1)],
            secondaries=[(-100, 0.56), (-96, 0.64), (-92, 0.7), (-88, 0.74), (-84, 0.76)],
            coverts=[(-100, 0.26), (-94, 0.28), (-88, 0.3), (-82, 0.3), (-76, 0.28)],
            droop=6,
        )

    _mantle(1, root, folded)

    # Mantled, after the fire: spread wide and low over the fallen, as a
    # bird of prey shields its kill. The primaries lie close over one
    # another and droop at their points, so the wing is one mass with a
    # scalloped edge.
    def mantled(mesh, bones, side):
        _wing(
            mesh,
            bones,
            side,
            sweep=28,
            edge=[(0.0, 0.0), (0.2, 0.17), (0.36, 0.24), (0.43, 0.22)],
            primaries=[(-4, 0.46), (-12, 0.52), (-20, 0.57), (-28, 0.6), (-36, 0.62), (-44, 0.6), (-52, 0.56)],
            secondaries=[(-92, 0.4), (-84, 0.44), (-76, 0.47), (-68, 0.48), (-60, 0.48)],
            coverts=[(-88, 0.2), (-78, 0.21), (-68, 0.22), (-58, 0.21), (-48, 0.2)],
            droop=30,
            width=0.09,
        )

    _mantle(2, root, mantled)


# Helms ------------------------------------------------------------------


def _course(height, front, recede, width, back, rake=0.0, cheek=0.62):
    """One course of a helm round the head at `height` above its middle:
    ten points, flat between them, from the front round to the back and
    mirrored. `recede` draws the front back from its middle, so the face is
    two planes meeting on a keel; `rake` lifts the course behind."""

    half = [
        (0.0, -front),
        (width * cheek, -front + recede),
        (width, -front * 0.42),
        (width * 0.97, back * 0.42),
        (width * 0.6, back - 0.02),
        (0.0, back),
    ]
    loop = half + [(-x, y) for x, y in reversed(half[1:-1])]

    return [HELM_CENTRE + Vector((x, y, height + rake * y)) for x, y in loop]


def _helm(name, courses, eye, stem=True):
    """A helm of bone plates from `courses`, crown first, open below, with
    a visor cut through it at `eye` above its middle: a slot across the
    eyes and, with `stem`, a slot down from it over the nose and mouth, so
    the face is a dark T in the bone."""

    rings = [_course(*course) for course in courses]
    shell = _solid(f"{name}_shell", rings, core.material("bone"), cap_bottom=False)
    core.apply_modifier(shell, "SOLIDIFY", thickness=0.012, offset=-1.0, use_even_offset=True)
    _outward(shell)

    centre = HELM_CENTRE
    cutters = [_box(f"{name}_slot", (centre.x - 0.118, centre.y - 0.3, centre.z + eye - 0.014), (centre.x + 0.118, centre.y - 0.05, centre.z + eye + 0.015))]

    if stem:
        cutters.append(_box(f"{name}_stem", (centre.x - 0.019, centre.y - 0.3, centre.z - 0.118), (centre.x + 0.019, centre.y - 0.05, centre.z + eye)))

    _cut(shell, cutters)

    return core.crisp(shell, 0.003, angle=24, segments=1)


def _outward(obj):
    """Turns every face of a closed mesh to face out."""

    mesh = bmesh.new()
    mesh.from_mesh(obj.data)
    bmesh.ops.recalc_face_normals(mesh, faces=mesh.faces)
    mesh.to_mesh(obj.data)
    mesh.free()

    return obj


def _beak(name, root_height, length=0.4, drop=0.17, width=0.12, height=0.09):
    """A raven's upper bill in bone, grown from the helm's face below the
    visor and hooked down at its point: a wedge of four flat faces, ridged
    on top and flat beneath."""

    rings = []

    for step in range(7):
        t = step / 6
        thin = max(1 - t, 0.04) ** 0.85
        centre = Vector((0, HELM_CENTRE.y - 0.16 - length * t, root_height - drop * t**1.7))
        half = width * 0.5 * thin
        rise = height * 0.6 * thin
        rings.append([centre + Vector((0, 0, rise)), centre + Vector((half, 0, -rise * 0.15)), centre + Vector((half * 0.5, 0, -rise * 0.5)), centre + Vector((-half * 0.5, 0, -rise * 0.5)), centre + Vector((-half, 0, -rise * 0.15))])

    bill = _solid(name, rings, core.material("bone"))

    return core.crisp(bill, 0.002, angle=30, segments=1)


def _crest(name):
    """A crest of feathers standing along a helm's crown from the brow over
    to the nape, rising behind and sweeping back past the nape into a
    point: from the front a dark fin on the dome, from the side a plume.

    Skinned through sections across it rather than cut from one outline,
    so that every face is a flat quad however the edge curves."""

    sections = []

    # (back, foot, top, half-thickness) from the helm's middle, front to
    # back; the foot is sunk into the dome until the blade leaves it at the
    # nape.
    for back, foot, top, half in (
        (-0.17, 0.09, 0.2, 0.016),
        (-0.1, 0.18, 0.33, 0.017),
        (0.0, 0.21, 0.4, 0.016),
        (0.1, 0.19, 0.42, 0.014),
        (0.2, 0.12, 0.39, 0.012),
        (0.3, 0.17, 0.3, 0.009),
        (0.38, 0.21, 0.225, 0.005),
    ):
        y = HELM_CENTRE.y + back
        low, high = HELM_CENTRE.z + foot, HELM_CENTRE.z + top
        sections.append([(-half, y, low), (-half, y, high), (half, y, high), (half, y, low)])

    return _solid(name, sections, core.material("feather"))


def _halo(name, radius=0.235, band=0.05, thickness=0.022, sides=14):
    """A ring of bone standing behind the head: flat-faced, and broad
    enough to print at a thumb's height. The chooser is crowned by nothing
    but the light it judges by."""

    centre = HELM_CENTRE + Vector((0, 0.235, 0.085))
    rings = []

    for step in range(sides):
        turn = math.tau * (step + 0.5) / sides
        out = Vector((math.cos(turn), 0, math.sin(turn)))
        inner, outer = centre + out * (radius - band), centre + out * radius
        lift = Vector((0, thickness / 2, 0))
        rings.append([outer - lift, outer + lift, inner + lift, inner - lift])

    # Skinned round the ring rather than along it.
    mesh = bmesh.new()
    rows = [[mesh.verts.new(point) for point in ring] for ring in rings]

    for current, following in zip(rows, rows[1:] + rows[:1]):
        for index in range(4):
            follow = (index + 1) % 4
            mesh.faces.new((current[index], following[index], following[follow], current[follow]))

    ring = core.object_from_bmesh(name, mesh, core.material("bone"), smooth=False)

    return core.crisp(ring, 0.003, angle=24, segments=1)


def _aventail(name, seed):
    """Cloth hung from inside a helm's rim to the shoulders and drawn up
    over the throat. It is the surcoat's own cloth, so that the figure
    shows three families."""

    curtain = cloth.robe(
        name,
        core.material("cloth"),
        None,
        top=HELM_CENTRE.z - 0.1,
        hem=SHOULDER_HEIGHT - 0.04,
        profile=((0.0, 0.13, 0.13), (0.5, 0.145, 0.14), (1.0, 0.21, 0.17)),
        folds=(4, 6, 9),
        fold_depth=0.07,
        tatter=0.3,
        tears=4,
        slits=0,
        seed=seed,
        rings=8,
        segments=40,
    )
    curtain.shape_key_clear()
    curtain.location = (0, HELM_CENTRE.y, 0)

    return finish(curtain, seed=seed)


def _heads(head):
    """Three bone helms, each carved in planes and half again the size of
    a head, closed but for a dark visor where two narrow lights show
    (lib/face.py sets them by the helm's index)."""

    offset = Vector(head.location)

    def place(obj, group):
        obj.parent = group
        obj.location = Vector(obj.location) - offset

        return obj

    # The sentinel: a domed helm with a flat brow plate standing out over
    # the visor, a face of two planes meeting on a keel with a T cut down
    # it, cheek plates that flare out at the jaw, and a crest of black
    # feathers along the crown sweeping back into a point behind. The crest
    # is a part of its own, in the wings' feather, so that it reads against
    # the bone in every view.
    group = core.empty("head_0", head)
    shell = _helm(
        "helm_0_shell",
        (
            (0.245, 0.06, 0.0, 0.07, 0.12, 0.1),
            (0.19, 0.135, 0.016, 0.128, 0.172, 0.0),
            (0.11, 0.18, 0.026, 0.156, 0.196, 0.0),
            (0.042, 0.2, 0.034, 0.162, 0.2, 0.0),
            (-0.012, 0.188, 0.034, 0.158, 0.198, 0.0),
            (-0.085, 0.186, 0.036, 0.17, 0.19, 0.0),
            (-0.17, 0.176, 0.032, 0.205, 0.18, 0.0),
        ),
        EYE[0],
    )
    shell.name = shell.data.name = "helm_0"
    place(finish(shell, wear_boost=1.6, seed=0), group)
    place(finish(_crest("crest_0"), seed=0), group)

    # The raven: a lower, rounder helm with a narrow slot for a visor and
    # the bill of a carrion bird hung from below it.
    group = core.empty("head_1", head)
    shell = _helm(
        "helm_1_shell",
        (
            (0.225, 0.06, 0.0, 0.06, 0.12, 0.12),
            (0.17, 0.13, 0.012, 0.118, 0.172, 0.0),
            (0.08, 0.172, 0.016, 0.152, 0.192, 0.0),
            (0.03, 0.19, 0.02, 0.158, 0.198, 0.0),
            (-0.03, 0.18, 0.02, 0.158, 0.195, 0.0),
            (-0.1, 0.176, 0.02, 0.165, 0.188, 0.0),
            (-0.165, 0.166, 0.018, 0.186, 0.176, 0.0),
        ),
        EYE[1],
        stem=False,
    )
    helm = core.join([shell, _beak("helm_1_bill", HELM_CENTRE.z + EYE[1] - 0.04)], "helm_1")
    place(finish(helm, wear_boost=1.6, seed=1), group)

    # The haloed: a close helm, smooth-crowned and plain, with a T for a
    # face and a ring of bone standing behind it.
    group = core.empty("head_2", head)
    shell = _helm(
        "helm_2_shell",
        (
            (0.25, 0.06, 0.0, 0.06, 0.12, 0.1),
            (0.2, 0.13, 0.018, 0.118, 0.17, 0.0),
            (0.12, 0.176, 0.026, 0.152, 0.19, 0.0),
            (0.068, 0.198, 0.032, 0.16, 0.198, 0.0),
            (0.0, 0.186, 0.032, 0.16, 0.196, 0.0),
            (-0.09, 0.182, 0.032, 0.166, 0.19, 0.0),
            (-0.165, 0.172, 0.03, 0.19, 0.176, 0.0),
        ),
        EYE[2],
    )
    helm = core.join([shell, _halo("helm_2_halo")], "helm_2")
    place(finish(helm, wear_boost=1.6, seed=2), group)

    for index in range(3):
        group = next(child for child in head.children if child.name == f"head_{index}")
        # The dark inside the helm, grown to fill it.
        void = void_head(None, 0.0, f"void_{index}")
        void.data.transform(Matrix.Diagonal((1.3, 1.3, 1.25, 1)))
        place(void, group)
        place(_aventail(f"wrap_{index}", seed=50 + index), group)
        place(eyes(None, 0.0, spacing=0.035, height=HELM_CENTRE.z + EYE[index], size=0.011, name=f"eyes_{index}"), group)
