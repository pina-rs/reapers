"""The chooser: the one who picks who falls, then strikes once.

Silhouette: a pair of carrion wings behind square, feathered shoulders, a
bone helm, and a spear stood upright at the weapon side like a verdict
already given. The outline reads as a tall "Y" with one hard vertical
beside it: the wings are the head and shoulder line, the spear is the
weapon side, and the spear is the figure's one asymmetry.

Three families show: the robe (`cloth`, with `cloth_alt` at the throat),
the wings (`feather`) and the helm and hands (`bone`). Cobalt is the
darkest light, so the bone helm carries the figure's value (docs/art-
bible.md 6.2). Each wing has seven large primaries, and every mantle is a
pair of wings, so the class reads as winged whatever it wears.
"""

import math

import bmesh
from mathutils import Matrix, Vector

from lib import cloth, core
from lib.figure import HEAD_HEIGHT, NECK_HEIGHT, SHOULDER_HEIGHT, arms, eyes, finish, void_head

ROBE_TOP = SHOULDER_HEIGHT + 0.07
ROBE_HEM = 0.06

# A surcoat cut square at the shoulder and close at the waist, falling
# straight to a wide hem: (depth, half-width, half-depth) from the collar.
SURCOAT = ((0.0, 0.115, 0.1), (0.045, 0.225, 0.13), (0.15, 0.215, 0.135), (0.37, 0.14, 0.11), (0.52, 0.165, 0.13), (1.0, 0.29, 0.25))

# Where the weapon hand closes. The idle performance carries the forearm
# forward about the elbow and the arm out about the shoulder
# (lib/motion.py `sample("idle")`); the socket is turned back by as much, so
# the spear stands plumb in the stance every screen shows.
IDLE_ELBOW = math.radians(-24 * 1.05)
IDLE_SHOULDER = math.radians(8 * 1.05)
SPEAR_SPLAY = math.radians(4)

# Every wing grows from behind its shoulder blade.
WING_ROOT = Vector((0.1, 0.11, SHOULDER_HEIGHT - 0.07))

HELM_CENTRE = Vector((0, -0.006, HEAD_HEIGHT + 0.004))


def build():
    """The one who picks who falls: wings, a bone helm, an upright spear."""

    root = core.empty("reaper")
    cloth_mat = core.material("cloth")

    # Four hems. None sweeps or leans: the chooser stands upright, and its
    # one asymmetry is the spear.
    hems = [
        dict(tatter=0.2, tears=4, slits=4, fold_depth=0.11, seed=41),
        dict(tatter=0.34, tears=7, slits=8, fold_depth=0.13, seed=42),
        dict(tatter=0.16, tears=3, slits=2, fold_depth=0.1, seed=43),
        dict(tatter=0.28, tears=5, slits=10, fold_depth=0.14, seed=44),
    ]

    for index, hem in enumerate(hems):
        robe = cloth.robe(
            f"robe_{index}",
            cloth_mat,
            root,
            top=ROBE_TOP,
            hem=ROBE_HEM,
            profile=SURCOAT,
            folds=(4, 6, 11),
            rings=40,
            segments=96,
            **hem,
        )
        finish(robe, seed=index)

    weapon_arm, _ = arms(
        root,
        # The upper arm hangs and the forearm points ahead, so the fist
        # stands out from the hip with the haft upright in it.
        weapon_side_path=[(-0.215, 0.0, SHOULDER_HEIGHT - 0.03), (-0.31, 0.06, 1.31), (-0.36, -0.05, 1.25)],
        off_side_path=[(0.215, 0.0, SHOULDER_HEIGHT - 0.03), (0.28, 0.04, 1.32), (0.29, -0.02, 1.05)],
        sleeve_shoulder=0.072,
        sleeve_cuff=0.082,
        droop=0.05,
        tatter=0.12,
    )

    socket = next(child for child in weapon_arm.children_recursive if child.name == "socket_weapon")
    socket.rotation_mode = "XYZ"
    socket.rotation_euler = (Matrix.Rotation(-IDLE_ELBOW, 3, "X") @ Matrix.Rotation(-IDLE_SHOULDER - SPEAR_SPLAY, 3, "Y")).to_euler()

    head = core.empty("head", root, location=(0, 0, NECK_HEIGHT))
    _heads(head)
    _mantles(root)

    return root


def _smooth(value):
    """Eases 0..1 in and out, and holds outside it."""

    value = max(0.0, min(1.0, value))

    return value * value * (3 - 2 * value)


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


def _feather(mesh, base, tip, facing, width, curl=0.04, bend=0.0, cup=0.22, shaft=0.007, segments=8, blunt=0.0, notch=None, lead=0.45, hand=1):
    """Adds one feather to `mesh`: a broad blade either side of a raised
    spine, from its quill at `base` to `tip`.

    The vane is wide almost at once and holds its width, then comes to a
    point, so a feather prints as one clean leaf rather than a quill.
    `facing` is the way its upper side looks; `curl` bows the spine toward
    it and `bend` bows it sideways. `lead` is the leading vane's share of
    the trailing one, as on a flight feather. `blunt` leaves the tip torn
    square at that share of the full width, and `notch` (a share of the
    length) bites one nick out of the trailing edge.
    """

    base = Vector(base)
    axis = Vector(tip) - base
    forward = axis.normalized()
    normal = Vector(facing)
    normal = (normal - forward * normal.dot(forward)).normalized()
    # A right wing's feathers are its left's mirrored (`hand` -1).
    side = forward.cross(normal) * hand
    rings = []

    for step in range(segments + 1):
        t = 1 - (1 - step / segments) ** 1.35
        centre = base + axis * t + (normal * curl + side * bend) * (axis.length * 4 * t * (1 - t))
        radius = shaft * (1 - 0.7 * t)
        leaf = min(1.0, t / 0.16) ** 0.6 * max(blunt, (1 - t**2.6)) ** 0.55
        trailing = max(width * leaf, radius * 0.5)

        if notch is not None and abs(t - notch) < 0.5 / segments:
            trailing *= 0.55

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


def _wing(mesh, side, sweep, edge, primaries, secondaries, coverts, tilt=0.0, **shape):
    """Adds one wing to `mesh`.

    `edge` is the leading edge as (along, up) stops in the wing's plane,
    from the shoulder to the wrist and on to the hand. `primaries` and
    `secondaries` are (angle, length) pairs, the first rooted along the
    hand from the wrist outward, the second along the forearm from the
    shoulder to the wrist; `coverts` are laid over the secondaries' roots.
    Angles are in degrees up from straight out along the wing.
    """

    plane = _Plane(side, sweep, tilt=tilt)
    line = [Vector(point) for point in core.bezier(edge, samples=6)]
    wrist = len(line) * 2 // 3

    def stop(share, start, end):
        position = start + (end - start) * share
        index = min(int(position), len(line) - 2)

        return line[index].lerp(line[index + 1], position - index)

    def place(points, angle, length, depth, width, **extra):
        along, up = points
        base = plane.at(along, up, depth)
        tip = base + plane.toward(angle) * length
        _feather(mesh, base, tip, -plane.back, width, hand=side, **{**shape, **extra})

    # The bony leading edge, feathered over: one smooth arm the feathers
    # hang from, so the wing's top line is a single stroke.
    arm = [plane.at(*point, -0.012) for point in line]
    leading = core.tube("_wing_arm", arm, lambda t: 0.034 - 0.016 * t, core.material("feather"), sides=8)
    mesh.from_mesh(leading.data)
    _remove(leading)

    for index, (angle, length) in enumerate(primaries):
        share = index / max(len(primaries) - 1, 1)
        place(stop(share, wrist, len(line) - 1), angle, length, 0.012 + 0.004 * index, 0.05 + 0.012 * math.sin(math.pi * share), notch=0.7 if index % 3 == 1 else None)

    for index, (angle, length) in enumerate(secondaries):
        share = (index + 0.5) / len(secondaries)
        place(stop(share, 0, wrist), angle, length, 0.0, 0.058, lead=0.6)

    for index, (angle, length) in enumerate(coverts):
        share = (index + 0.35) / len(coverts)
        place(stop(share, 0, wrist + 2), angle, length, -0.024, 0.052, lead=0.8, segments=5, curl=0.08)


def _remove(obj):
    import bpy

    mesh = obj.data
    bpy.data.objects.remove(obj, do_unlink=True)
    bpy.data.meshes.remove(mesh)


def _scapulars(mesh, side):
    """Broad feathers laid over each shoulder like a pauldron's lames, in
    two courses, the upper overlapping the lower: they carry the shoulder
    line out square past the sleeve."""

    # Two courses of three broad feathers, front, top and back of the
    # shoulder, laid down over it rather than standing off it.
    for course, (height, reach, droop, width) in enumerate(((0.075, 0.25, -24, 0.09), (0.035, 0.29, -40, 0.1))):
        for index in range(3):
            turn = math.radians(-48 + 48 * index)
            outward = Vector((side * 0.8, math.sin(turn) * 0.6, 0)).normalized()
            base = Vector((side * 0.12, 0.02 + math.sin(turn) * 0.06, SHOULDER_HEIGHT + height))
            tilt = math.radians(droop - 10 * abs(math.sin(turn)))
            direction = outward * math.cos(tilt) + Vector((0, 0, math.sin(tilt)))
            _feather(
                mesh,
                base,
                base + direction * reach,
                Vector((0, -0.15, 1)) + outward * 0.4,
                width,
                curl=0.16,
                cup=0.34,
                segments=5,
                lead=1.0,
                hand=side,
            )


def _mantle(index, root, build_wing):
    """A pair of wings as mantle `index`, mirrored across the back."""

    group = core.empty(f"mantle_{index}", root)
    mesh = bmesh.new()

    for side in (-1, 1):
        build_wing(mesh, side)
        _scapulars(mesh, side)

    wings = core.object_from_bmesh(f"wing_{index}", mesh, core.material("feather"), group)
    finish(core.shade_smooth(wings, angle=40), wear_boost=1.4, seed=index)


def _mantles(root):
    """Three pairs of wings."""

    # Raised: each wing climbs from the shoulder to a wrist level with the
    # crown, and its seven primaries fan from straight up to well out, with
    # gaps between their tips like a carrion bird's.
    def raised(mesh, side):
        _wing(
            mesh,
            side,
            sweep=24,
            edge=[(0.0, 0.0), (0.2, 0.14), (0.34, 0.32), (0.4, 0.42)],
            # The trailing edge runs on unbroken from the last secondary to
            # the first primary, so the wing is one shape and not two fans.
            primaries=[(102, 0.5), (90, 0.6), (78, 0.66), (66, 0.68), (54, 0.65), (42, 0.6), (30, 0.54)],
            secondaries=[(-62, 0.44), (-44, 0.48), (-26, 0.5), (-8, 0.51), (10, 0.52)],
            coverts=[(-50, 0.26), (-32, 0.28), (-14, 0.29), (4, 0.28), (22, 0.27)],
        )

    _mantle(0, root, raised)

    # Folded: the wrist stands over each shoulder like a horn, and the
    # primaries fall from it outside the arms to the knee.
    def folded(mesh, side):
        _wing(
            mesh,
            side,
            sweep=14,
            edge=[(0.0, 0.0), (0.11, 0.28), (0.2, 0.5), (0.23, 0.56)],
            primaries=[(-64, 0.98), (-70, 1.02), (-76, 0.98), (-82, 0.92), (-88, 0.84), (-94, 0.76), (-100, 0.66)],
            secondaries=[(-100, 0.56), (-94, 0.64), (-88, 0.7), (-82, 0.74), (-77, 0.76)],
            coverts=[(-100, 0.26), (-92, 0.28), (-85, 0.3), (-78, 0.3), (-72, 0.28)],
        )

    _mantle(1, root, folded)

    # Mantled, after the fire: spread wide and low over the fallen, as a
    # bird of prey shields its kill, each primary torn square.
    def mantled(mesh, side):
        _wing(
            mesh,
            side,
            sweep=28,
            edge=[(0.0, 0.0), (0.2, 0.17), (0.36, 0.24), (0.43, 0.22)],
            primaries=[(-14, 0.44), (-25, 0.5), (-36, 0.54), (-47, 0.55), (-58, 0.53), (-69, 0.49), (-80, 0.44)],
            secondaries=[(-100, 0.4), (-94, 0.44), (-88, 0.46), (-82, 0.46), (-76, 0.44)],
            coverts=[(-96, 0.2), (-88, 0.21), (-80, 0.22), (-72, 0.21), (-64, 0.2)],
            blunt=0.5,
        )

    _mantle(2, root, mantled)


# Helms ------------------------------------------------------------------


def _dome(width, depth, crown, taper=1.0, brow=-0.1, jaw=-0.12, nape=-0.13, flare=0.1, point=0.0):
    """Returns `at(angle, share)`, a point on a helm's outer surface.

    `angle` runs around the head (-pi/2 is the face) and `share` from the
    crown (0) to the rim (1). The rim is at `brow` over the face, `jaw` at
    the cheeks and `nape` behind, as heights from the middle of the head.
    `point` draws the front of the helm forward into a keel.
    """

    def rim(angle):
        away = math.degrees(abs((angle + math.pi * 1.5) % math.tau - math.pi))
        face = 1 - _smooth((away - 30) / 40)
        low = jaw + (nape - jaw) * _smooth((away - 80) / 100)

        return brow * face + low * (1 - face)

    def at(angle, share, scale=1.0):
        edge = rim(angle)
        reach = math.acos(max(-1.0, min(1.0, edge / crown))) / (math.pi / 2) if edge > 0 else 1 - edge / crown
        drop = max(reach * share, 0.02)

        if drop <= 1:
            spread = math.sin(drop * math.pi / 2) ** taper
            height = crown * math.cos(drop * math.pi / 2)
        else:
            spread = 1 + flare * (drop - 1) ** 2
            height = -(drop - 1) * crown

        front = max(0.0, -math.sin(angle)) ** 6
        keel = 1 + point * front * spread

        return HELM_CENTRE + Vector((math.cos(angle) * width * spread, math.sin(angle) * depth * spread * keel, height)) * scale

    return at


def _shell(name, at, slot, columns=48, rows=16, thickness=0.009):
    """A helm's shell from `at` (see `_dome`), cut away wherever
    `slot(angle, height)` says, and given a plate's thickness, with its cut
    edges chamfered so they catch a highlight."""

    mesh = bmesh.new()
    angles = [math.tau * column / columns for column in range(columns)]
    apex = mesh.verts.new(at(0.0, 0.0))
    grid = [[mesh.verts.new(at(angle, (row + 1) / (rows + 1))) for angle in angles] for row in range(rows + 1)]

    for column in range(columns):
        follow = (column + 1) % columns
        mesh.faces.new((apex, grid[0][follow], grid[0][column]))

    for row in range(rows):
        for column in range(columns):
            follow = (column + 1) % columns
            middle = (grid[row][column].co + grid[row + 1][follow].co) / 2

            if slot(math.atan2(middle.y - HELM_CENTRE.y, middle.x), middle.z):
                continue

            mesh.faces.new((grid[row][column], grid[row][follow], grid[row + 1][follow], grid[row + 1][column]))

    loose = [vertex for vertex in mesh.verts if not vertex.link_faces]
    bmesh.ops.delete(mesh, geom=loose, context="VERTS")
    shell = core.object_from_bmesh(name, mesh, core.material("bone"))
    core.apply_modifier(shell, "SOLIDIFY", thickness=thickness, offset=-1.0, use_even_offset=True)

    return core.crisp(shell, 0.0025, angle=50, segments=1)


def _comb(name, at, rise=0.075, start=0.5, end=0.62):
    """A blade of bone standing along a helm's crown from the brow to the
    nape, highest behind the crown, so the helm's outline comes to a point."""

    line = [at(-math.pi / 2, start * (1 - step / 8)) for step in range(8)] + [at(math.pi / 2, end * step / 10) for step in range(11)]
    lower = [point - (point - HELM_CENTRE).normalized() * 0.012 for point in line]
    upper = []

    for index, point in enumerate(line):
        share = index / (len(line) - 1)
        lift = rise * math.sin(math.pi * share) ** 0.8 * (0.7 + 0.5 * share)
        upper.append(point + (point - HELM_CENTRE).normalized() * lift)

    # A plate is cut in the XZ plane; this one is cut in YZ and turned.
    outline = [(-point.y, point.z) for point in upper] + [(-point.y, point.z) for point in reversed(lower)]
    blade = core.plate(name, outline, 0.014, core.material("bone"))
    blade.data.transform(Matrix.Rotation(-math.pi / 2, 4, "Z"))

    return blade


def _aside(angle):
    """How far around from the face `angle` is, in radians."""

    return abs((angle + math.pi / 2 + math.pi) % math.tau - math.pi)


def _beak(name, root_height, length=0.3, drop=0.13, width=0.105, height=0.075):
    """A raven's upper bill in bone, grown from the helm's face below the
    eye slot and hooked down at its point."""

    rings = []

    for step in range(11):
        t = step / 10
        thin = max(1 - t, 0.03) ** 0.9
        centre = Vector((0, HELM_CENTRE.y - 0.118 - length * t, root_height - drop * t**1.6))
        ring = []

        for point in range(10):
            angle = math.tau * point / 10
            across = math.sin(angle) * width * 0.5 * thin**1.3
            lift = math.cos(angle) * height * 0.5 * thin

            # A ridge along the top of the bill, flat cheeks, a flat palate.
            if lift > 0:
                lift *= 1 + 0.5 * math.exp(-((across / 0.012) ** 2))
            else:
                lift *= 0.45

            ring.append(centre + Vector((across, 0.04 * (1 - t) ** 3 * abs(across) / max(width, 1e-6), lift)))

        rings.append(ring)

    bill = core.loft(name, rings, core.material("bone"), cap_start=True, cap_end=True)

    return core.crisp(bill, 0.002, angle=40, segments=1)


def _aventail(name, parent, seed, top=HEAD_HEIGHT - 0.035):
    """Cloth hung from a helm's rim to the shoulders and drawn up over the
    throat: the light band between the bone and the robe."""

    curtain = cloth.robe(
        name,
        core.material("cloth_alt"),
        None,
        top=top,
        hem=SHOULDER_HEIGHT - 0.05,
        profile=((0.0, 0.112, 0.12), (0.4, 0.12, 0.128), (0.7, 0.16, 0.14), (1.0, 0.235, 0.175)),
        folds=(4, 6, 9),
        fold_depth=0.08,
        tatter=0.3,
        tears=5,
        slits=0,
        seed=seed,
        rings=12,
        segments=48,
    )
    curtain.shape_key_clear()
    curtain.location = (0, HELM_CENTRE.y, 0)
    curtain.parent = parent

    return finish(curtain, seed=seed)


def _diadem(name, at, share=0.34, points=(0.06, 0.085, 0.12, 0.085, 0.06), spread=100):
    """Five blades of bone standing round the front of a helm, tallest over
    the brow: a crown cut like the primaries of a wing. Each is a flat leaf
    following the helm's curve, `spread` degrees round from one end to the
    other."""

    blades = []

    for index, height in enumerate(points):
        turn = -math.pi / 2 + math.radians(spread) * (index / (len(points) - 1) - 0.5)
        foot = at(turn, share)
        outward = Vector((foot.x - HELM_CENTRE.x, foot.y - HELM_CENTRE.y, 0)).normalized()
        up = (Vector((0, 0, 1)) + outward * 0.35).normalized()
        across = up.cross(outward).normalized()
        half = 0.022 + 0.006 * (height / 0.12)
        outline = [(-half * 0.8, -0.02), (-half, 0.25 * height), (-half * 0.55, 0.7 * height), (0.0, height), (half * 0.55, 0.7 * height), (half, 0.25 * height), (half * 0.8, -0.02)]
        blade = core.plate(f"{name}_{index}", outline, 0.01, core.material("bone"))
        frame = Matrix((across, -outward, up)).transposed().to_4x4()
        blade.data.transform(Matrix.Translation(foot - outward * 0.004) @ frame)
        blades.append(blade)

    return core.join(blades, name)


def _heads(head):
    """Three bone helms, each closed but for a dark slot where two narrow
    lights show (lib/face.py sets them by the helm's index)."""

    offset = Vector(head.location)

    def place(obj, group):
        obj.parent = group
        obj.location = Vector(obj.location) - offset

        return obj

    # The sentinel: a tall sallet drawn to a keel over the face and swept
    # into a tail at the nape, with one narrow slot across the eyes and a
    # comb along its crown.
    group = core.empty("head_0", head)
    at = _dome(0.122, 0.142, 0.215, taper=1.5, brow=-0.12, jaw=-0.11, nape=-0.17, flare=0.55, point=0.22)
    eye = HEAD_HEIGHT + 0.012

    def sentinel_slot(angle, height):
        return _aside(angle) < math.radians(58) and abs(height - eye) < 0.017

    helm = core.join([_shell("helm_0_shell", at, sentinel_slot), _comb("helm_0_comb", at)], "helm_0")
    place(finish(helm, wear_boost=1.6, seed=0), group)

    # The raven: a round skull of a helm with the bill of a carrion bird
    # hung from below its eye slot.
    group = core.empty("head_1", head)
    at = _dome(0.124, 0.136, 0.17, taper=1.1, brow=-0.09, jaw=-0.1, nape=-0.13, flare=0.25, point=0.1)
    eye = HEAD_HEIGHT + 0.004

    def raven_slot(angle, height):
        return _aside(angle) < math.radians(62) and abs(height - eye) < 0.018

    helm = core.join([_shell("helm_1_shell", at, raven_slot), _beak("helm_1_bill", eye - 0.032)], "helm_1")
    place(finish(helm, wear_boost=1.6, seed=1), group)

    # The crowned: a close helm with a plain mask of bone for a face, and a
    # fan of seven feathers standing behind it like a halo.
    group = core.empty("head_2", head)
    at = _dome(0.12, 0.134, 0.17, taper=1.15, brow=-0.13, jaw=-0.12, nape=-0.12, flare=0.15, point=0.06)
    eye = HEAD_HEIGHT + 0.041

    def crowned_slot(angle, height):
        # The face is closed; the lights show on the mask itself, in a
        # shallow groove pressed across it.
        return False

    shell = _shell("helm_2_shell", at, crowned_slot)

    for vertex in shell.data.vertices:
        co = vertex.co
        aside = _aside(math.atan2(co.y - HELM_CENTRE.y, co.x))

        if co.y < HELM_CENTRE.y - 0.06:
            vertex.co.y += 0.008 * math.exp(-(((co.z - eye) / 0.012) ** 2)) * max(0.0, math.cos(aside * 1.4))

    helm = core.join([shell, _diadem("helm_2_diadem", at)], "helm_2")
    place(finish(helm, wear_boost=1.6, seed=2), group)

    for index in range(3):
        group = next(child for child in head.children if child.name == f"head_{index}")
        place(void_head(None, 0.0, f"void_{index}"), group)
        place(_aventail(f"wrap_{index}", None, seed=50 + index), group)
        place(eyes(None, 0.0, spacing=0.035, height=(HEAD_HEIGHT + 0.012, HEAD_HEIGHT + 0.004, HEAD_HEIGHT + 0.041)[index], size=0.011, name=f"eyes_{index}"), group)
