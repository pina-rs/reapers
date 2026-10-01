"""The chooser: the one who picks who falls, then strikes once.

Silhouette: a wing of feathers raised over one shoulder and a spear leaning
the other way, on a slim, upright body. The outline reads as a lopsided "X".
"""

import math
import random

import bmesh
from mathutils import Vector

from lib import cloth, core
from lib.figure import HEAD_HEIGHT, NECK_HEIGHT, SHOULDER_HEIGHT, arms, eyes, finish, void_head

ROBE_TOP = SHOULDER_HEIGHT + 0.06
ROBE_HEM = 0.1

# The spear leans away from the wing, and the wing is rooted behind the
# off-hand shoulder. Those two diagonals are the whole silhouette.
SPEAR_LEAN = math.radians(27)
WING_SHOULDER = Vector((0.2, 0.0, SHOULDER_HEIGHT))
WING_ROOT = Vector((0.15, 0.075, SHOULDER_HEIGHT + 0.015))

HELM_CENTRE = Vector((0, -0.012, HEAD_HEIGHT))
HELM_WIDTH = 0.128
HELM_DEPTH = 0.14


def build():
    """The one who picks who falls: one wing, one spear, a hidden face."""

    root = core.empty("reaper")
    cloth_mat = core.material("cloth")

    # A slim coat, its hem carried sideways as if a wind crossed the figure.
    hems = [
        (-0.26, dict(tatter=0.3, tears=6, slits=8, fold_depth=0.14, seed=41, lean=0.06)),
        (-0.36, dict(tatter=0.38, tears=9, slits=14, fold_depth=0.16, seed=42, lean=0.1)),
        (-0.18, dict(tatter=0.24, tears=4, slits=5, fold_depth=0.12, seed=43)),
        (-0.3, dict(tatter=0.34, tears=7, slits=10, fold_depth=0.18, seed=44, lean=0.17)),
    ]

    for index, (sweep, hem) in enumerate(hems):
        robe = cloth.robe(
            f"robe_{index}",
            cloth_mat,
            root,
            top=ROBE_TOP,
            hem=ROBE_HEM,
            profile=((0.0, 0.11, 0.1), (0.05, 0.19, 0.12), (0.13, 0.205, 0.125), (0.4, 0.15, 0.125), (0.74, 0.185, 0.16), (1.0, 0.25, 0.22)),
            rings=48,
            segments=112,
            **hem,
        )
        finish(_sweep(robe, sweep), seed=index)

    weapon_arm, _ = arms(
        root,
        weapon_side_path=[(-0.2, 0.0, SHOULDER_HEIGHT - 0.03), (-0.29, -0.01, 1.39), (-0.25, -0.2, 1.31)],
        off_side_path=[(0.2, 0.0, SHOULDER_HEIGHT - 0.03), (0.29, 0.02, 1.3), (0.3, -0.04, 1.02)],
        sleeve_shoulder=0.07,
        sleeve_cuff=0.088,
        droop=0.14,
    )

    # Rolling the wrist is what leans the spear: its point away from the
    # wing, its butt across the body.
    socket = next(child for child in weapon_arm.children if child.name == "socket_weapon")
    socket.rotation_euler = (0, -SPEAR_LEAN, 0)

    head = core.empty("head", root, location=(0, 0, NECK_HEIGHT))
    _heads(head)
    _mantles(root)

    return root


def _sweep(robe, amount):
    """Carries the hem `amount` metres sideways, more the lower it hangs.

    Every shape key moves alike, so the cloth still stirs as the game expects.
    """

    offsets = [amount * max(0.0, (ROBE_TOP - vertex.co.z) / (ROBE_TOP - ROBE_HEM)) ** 2 for vertex in robe.data.vertices]

    for vertex, offset in zip(robe.data.vertices, offsets):
        vertex.co.x += offset

    for key in robe.data.shape_keys.key_blocks:
        for point, offset in zip(key.data, offsets):
            point.co.x += offset

    return robe


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


def _feather(mesh, base, tip, facing, width, generator, curl=0.07, bend=0.0, cup=0.35, quill=0.1, vane=1.0, shaft=0.0045, segments=10):
    """Adds one feather to `mesh`: a flat, cupped leaf either side of a raised
    spine, growing from a bare quill at `base` to a point at `tip`.

    `facing` is the way the feather's upper side looks. `curl` bows the spine
    toward it and `bend` bows it sideways, within the leaf. `vane` below 1
    burns the leaf back toward the quill and leaves the rest of the spine
    bare.
    """

    base = Vector(base)
    axis = Vector(tip) - base
    forward = axis.normalized()
    side = forward.cross(Vector(facing)).normalized()
    normal = side.cross(forward)

    # Most feathers have a split somewhere in the vane.
    nick = generator.randrange(segments // 2, segments - 1) if generator.random() < 0.55 else None
    rings = []

    for step in range(segments + 1):
        # Rings bunch toward the tip of a whole feather, so that it comes
        # out rounded. A burned one has no tip left to round.
        t = 1 - (1 - step / segments) ** (1.5 if vane >= 1 else 1.0)
        centre = base + axis * t + (normal * curl + side * bend) * (axis.length * 4 * t * (1 - t))
        radius = shaft * (1 - 0.8 * t)
        along = max(0.0, (t - quill) / (1 - quill))
        leaf = min(1.0, along / 0.2) ** 0.7 * (1 - along**3.2) ** 0.5 * _smooth((vane - along) / 0.3)
        trailing = max(width * leaf * (0.6 if step == nick else 1.0), radius * 0.4)
        leading = max(width * leaf * 0.7, radius * 0.4)

        rings.append(
            [
                centre - side * (radius + trailing) - normal * cup * trailing,
                centre - side * (radius + trailing * 0.5) - normal * cup * trailing * 0.25,
                centre - side * radius,
                centre + normal * radius * 1.3,
                centre + side * radius,
                centre + side * (radius + leading * 0.5) - normal * cup * leading * 0.25,
                centre + side * (radius + leading) - normal * cup * leading,
                centre - normal * radius,
            ]
        )

    _skin(mesh, rings)


def _fan(mesh, generator, count, angles, reach, width, arm, depth=0.0, sweep=0.2, back=False, ragged=0.06, bend=-0.05, vane=(1.0, 1.0), **shape):
    """Adds one layer of a wing: feathers rooted along the wing's `arm` and
    fanned across the plane the figure is seen in, front and back.

    `angles` are the first and last feather's, in degrees up from level, and
    `reach` gives each one's length from how far along the arm it is rooted.
    `depth` sets the layer behind the others, `sweep` trails the tips back,
    and a `back` layer shows its upper side to whoever stands behind. The
    feathers arc as they go, each tip falling away from the arm.
    """

    for index in range(count):
        share = index / (count - 1)
        base = WING_ROOT + arm * share + Vector((0, depth + 0.003 * index, 0))
        turn = math.radians(angles[0] + (angles[1] - angles[0]) * share + generator.uniform(-2.5, 2.5))
        direction = Vector((math.cos(turn), sweep, math.sin(turn))).normalized()
        length = (reach(share) if callable(reach) else reach) * generator.uniform(1 - ragged, 1.05)

        _feather(
            mesh,
            base,
            base + direction * length,
            (0, 1 if back else -1, 0),
            width * generator.uniform(0.92, 1.08),
            generator,
            vane=generator.uniform(*vane),
            bend=-bend if back else bend,
            **shape,
        )


def _row(mesh, generator, count, span, pitch, reach, width, radius, height, vane=(1.0, 1.0), **shape):
    """Adds a row of feathers laid round the off-hand shoulder.

    `span` is the arc in degrees, from 0 pointing out along +X round to 90
    behind the figure. `pitch` is the feathers' tilt in degrees up from level,
    so a negative one lays them down over the arm.
    """

    for index in range(count):
        turn = math.radians(span[0] + (span[1] - span[0]) * index / (count - 1) + generator.uniform(-3, 3))
        outward = Vector((math.cos(turn), math.sin(turn), 0))
        base = WING_SHOULDER + outward * radius + Vector((0, 0, height + generator.uniform(-0.006, 0.006)))
        tilt = math.radians(pitch + generator.uniform(-4, 4))
        direction = outward * math.cos(tilt) + Vector((0, 0, math.sin(tilt)))

        _feather(
            mesh,
            base,
            base + direction * reach * generator.uniform(0.92, 1.08),
            outward,
            width * generator.uniform(0.92, 1.08),
            generator,
            vane=generator.uniform(*vane),
            curl=0.1,
            segments=7,
            **shape,
        )


def _grow(first, last, scale=1.0):
    """How long a feather of a wing is by where it is rooted: `first` at the
    shoulder, growing to `last` at the end of the arm."""

    return lambda share: scale * (first + (last - first) * share)


def _mantle(index, root, mesh):
    """Turns the feathers in `mesh` into mantle `index`, tied on with red
    thread across the chest and under the weapon arm."""

    group = core.empty(f"mantle_{index}", root)
    feathers = core.object_from_bmesh(f"wing_{index}", mesh, core.material("feather"), group)
    finish(core.shade_smooth(feathers, angle=40), wear_boost=1.4, seed=index)

    threads = []

    for strand, shift in enumerate((0.0, 0.024)):
        threads.append(
            core.tube(
                f"wing_{index}_thread_{strand}",
                core.bezier(
                    [
                        (0.2, -0.105, 1.6 - shift),
                        (0.04, -0.14, 1.49 - shift),
                        (-0.145, -0.125, 1.33 - shift),
                        (-0.185, 0.02, 1.27 - shift),
                        (-0.08, 0.135, 1.36 - shift),
                        (0.07, 0.14, 1.5 - shift),
                        (0.17, 0.11, 1.61 - shift),
                    ],
                    samples=6,
                ),
                0.0065,
                core.material("cloth_alt"),
                sides=5,
            )
        )

    tie = core.join(threads, f"wing_{index}_threads")
    tie.parent = group
    finish(tie)


def _mantles(root):
    """Three mantles of feathers, all worn on the off-hand shoulder."""

    lifted = Vector((0.19, 0.03, 0.31))
    folded = Vector((0.21, 0.03, 0.3))

    # A wing raised over the shoulder. Its arm climbs away from the head and
    # its feathers turn from level at the shoulder to follow the arm at its
    # end, with shorter ones laid over their roots, front and back.
    generator = random.Random(3)
    mesh = bmesh.new()

    _fan(mesh, generator, 10, (-8, 64), _grow(0.42, 0.58), 0.072, lifted, depth=0.02)
    _fan(mesh, generator, 9, (-10, 62), _grow(0.42, 0.58, 0.6), 0.064, lifted)
    _fan(mesh, generator, 8, (-14, 60), _grow(0.42, 0.58, 0.33), 0.054, lifted, depth=-0.02, segments=7)
    _fan(mesh, generator, 8, (42, 60), 0.15, 0.046, lifted, depth=-0.03, segments=7)
    _fan(mesh, generator, 8, (-10, 62), _grow(0.42, 0.58, 0.52), 0.064, lifted, depth=0.06, back=True)
    _fan(mesh, generator, 7, (-14, 60), _grow(0.42, 0.58, 0.28), 0.054, lifted, depth=0.075, back=True, segments=7)
    _row(mesh, generator, 7, (-80, 105), -48, 0.14, 0.048, radius=0.03, height=0.065)
    _row(mesh, generator, 8, (-85, 110), -70, 0.2, 0.05, radius=0.075, height=0.02)
    _mantle(0, root, mesh)

    # The same wing folded: its wrist stands over the shoulder, and long
    # feathers droop from it down the arm and the back.
    generator = random.Random(4)
    mesh = bmesh.new()

    _fan(mesh, generator, 9, (-98, -79), _grow(0.76, 1.08), 0.075, folded, depth=0.02, curl=0.05)
    _fan(mesh, generator, 9, (-101, -72), _grow(0.76, 1.08, 0.6), 0.066, folded)
    _fan(mesh, generator, 8, (-105, -62), 0.26, 0.056, folded, depth=-0.02, segments=7)
    _fan(mesh, generator, 8, (56, -52), 0.21, 0.04, folded, depth=-0.03, segments=7)
    _fan(mesh, generator, 8, (-101, -74), _grow(0.76, 1.08, 0.64), 0.066, folded, depth=0.06, back=True)
    _fan(mesh, generator, 7, (-105, -64), 0.27, 0.056, folded, depth=0.075, back=True, segments=7)
    _row(mesh, generator, 7, (-80, 105), -48, 0.14, 0.048, radius=0.03, height=0.065)
    _row(mesh, generator, 8, (-85, 110), -70, 0.2, 0.05, radius=0.075, height=0.02)
    _mantle(1, root, mesh)

    # The raised wing after the fire: bare quills, some snapped short, with
    # a rag of vane left near the root of a few.
    generator = random.Random(5)
    mesh = bmesh.new()

    _fan(mesh, generator, 9, (-8, 64), _grow(0.42, 0.58), 0.05, lifted, depth=0.02, ragged=0.3, bend=-0.1, vane=(0.0, 0.3), shaft=0.0095)
    _fan(mesh, generator, 7, (-10, 62), _grow(0.42, 0.58, 0.62), 0.048, lifted, ragged=0.25, bend=-0.08, vane=(0.15, 0.6), shaft=0.008)
    _fan(mesh, generator, 6, (-14, 60), _grow(0.42, 0.58, 0.36), 0.03, lifted, depth=-0.02, vane=(0.3, 0.8), shaft=0.007)
    _fan(mesh, generator, 6, (40, 60), 0.22, 0.026, lifted, depth=-0.03, vane=(0.35, 0.8), shaft=0.007)
    _row(mesh, generator, 5, (-80, 105), -48, 0.14, 0.044, radius=0.03, height=0.065, vane=(0.3, 0.9))
    _row(mesh, generator, 6, (-85, 110), -70, 0.2, 0.046, radius=0.075, height=0.02, vane=(0.2, 0.7), shaft=0.006)
    _mantle(2, root, mesh)


# Helms ------------------------------------------------------------------


def _helm_surface(crown, taper, brow, gape, jaw=-0.095, nape=-0.115, flare=0.15):
    """Returns `at(angle, share)`, a point on a helm's outer surface.

    `angle` runs around the head and `share` from the crown (0) to the rim
    (1). The rim is at `brow` over the face, an opening `gape` degrees to
    either side, and drops to `jaw` at the cheeks and `nape` behind. All
    three are heights from the middle of the head.
    """

    def rim(angle):
        away = math.degrees(abs((angle + math.pi * 1.5) % math.tau - math.pi))
        face = 1 - _smooth((away - gape[0]) / (gape[1] - gape[0]))
        low = jaw + (nape - jaw) * _smooth((away - 80) / 100)

        return brow * face + low * (1 - face)

    def at(angle, share, scale=1.0):
        edge = rim(angle)
        reach = math.acos(edge / crown) / (math.pi / 2) if edge > 0 else 1 - edge / crown
        drop = max(reach * share, 0.03)

        if drop <= 1:
            spread = math.sin(drop * math.pi / 2) ** taper
            height = crown * math.cos(drop * math.pi / 2)
        else:
            spread = 1 + flare * (drop - 1) ** 2
            height = -(drop - 1) * crown

        return HELM_CENTRE + Vector((math.cos(angle) * HELM_WIDTH * spread, math.sin(angle) * HELM_DEPTH * spread, height)) * scale

    return at


def _helm(name, at, segments=56, rings=14):
    """The parts of a close helm: a shell with an inner wall, so it still
    reads as plate when an outfit empties it, rolled at the rim and banded
    over the crown."""

    metal = core.material("metal")
    angles = [math.tau * segment / segments for segment in range(segments)]
    shell = [[at(angle, ring / rings) for angle in angles] for ring in range(rings + 1)]
    shell += [[at(angle, ring / rings, 0.93) for angle in angles] for ring in range(rings, -1, -3)]
    parts = [core.loft(f"{name}_shell", shell, metal, cap_start=True, cap_end=True)]

    rim = [at(angle, 1.0) for angle in angles]
    parts.append(core.tube(f"{name}_rim", rim + rim[:1], 0.0065, metal, sides=6, cap=False))

    for band, angle in enumerate((-math.pi / 2, 0.0)):
        over = [at(angle, ring / rings) for ring in range(rings, -1, -1)] + [at(angle + math.pi, ring / rings) for ring in range(rings + 1)]
        parts.append(core.tube(f"{name}_band_{band}", over, 0.007, metal, sides=6))

    return parts


def _beak(name):
    """A visor shaped like a raven's bill, drawn down over the face.

    Its ridge runs up to the brow and its shoulders stop short of it, which
    leaves one narrow slit over each eye.
    """

    rings = []

    for step in range(15):
        t = step / 14
        thin = max(1 - t, 0.04)
        centre = Vector((0, -0.15 - 0.19 * t, HEAD_HEIGHT - 0.025 - 0.05 * t - 0.05 * t * t))
        ring = []

        for point in range(24):
            angle = math.tau * point / 24
            across = math.sin(angle) * 0.132 * thin**1.5
            lift = math.copysign(abs(math.cos(angle)) ** 0.6, math.cos(angle)) * 0.058 * thin**0.85

            # The ridge is a narrow crest along the top of the bill.
            if lift > 0:
                lift += (0.03 * thin**1.6 + 0.006 * thin) * math.exp(-((across / 0.02) ** 2))

            # The root of the visor wraps round the front of the helm.
            ring.append(centre + Vector((across, 5.5 * (1 - t) ** 2 * across * across, lift)))

        rings.append(ring)

    return core.loft(name, rings, core.material("metal"), cap_start=True, cap_end=True)


def _aventail(name, parent, seed):
    """Cloth hung from a helm's rim to the shoulders and drawn up over the
    lower face, so that all a helm shows is a dark slot."""

    curtain = cloth.robe(
        name,
        core.material("cloth_alt"),
        None,
        top=HEAD_HEIGHT - 0.03,
        hem=SHOULDER_HEIGHT - 0.06,
        profile=((0.0, 0.118, 0.128), (0.35, 0.124, 0.134), (0.62, 0.15, 0.14), (1.0, 0.225, 0.17)),
        folds=(4, 7, 11),
        fold_depth=0.1,
        tatter=0.4,
        tears=7,
        slits=0,
        seed=seed,
        rings=14,
        segments=64,
    )
    curtain.shape_key_clear()
    curtain.location = (0, HELM_CENTRE.y, 0)
    curtain.parent = parent

    return finish(curtain, seed=seed)


def _narrow(lights, squash=0.5, slant=0.32):
    """Presses a pair of eyes into slits that tilt down toward the nose: a
    chooser's stare is a narrow one.

    The pair's origin is moved from one eye to midway between them, so that
    the game's gaze scale widens both alike inside a narrow helm.
    """

    for vertex in lights.data.vertices:
        vertex.co.x += lights.location.x

    lights.location.x = 0

    count = len(lights.data.vertices)
    level = sum(vertex.co.z for vertex in lights.data.vertices) / count
    middle = sum(abs(vertex.co.x) for vertex in lights.data.vertices) / count

    for vertex in lights.data.vertices:
        vertex.co.z = level + (vertex.co.z - level) * squash + (abs(vertex.co.x) - middle) * slant

    return lights


def _heads(head):
    """Three helms. Each is closed but for a dark gap, and two narrow lights
    in the gap."""

    offset = Vector(head.location)

    def place(obj):
        obj.location = Vector(obj.location) - offset

        return obj

    variants = [
        # A plain, pointed helm with a bar down over the nose.
        dict(crown=0.2, taper=1.6, brow=0.036, gape=(28, 42), gaze=HEAD_HEIGHT + 0.012),
        # A round helm, open only at a slot, with a crest of feathers.
        dict(crown=0.152, taper=1.0, brow=0.024, gape=(23, 33), gaze=HEAD_HEIGHT + 0.004),
        # A helm whose visor is a beak.
        dict(crown=0.162, taper=1.1, brow=0.052, gape=(50, 64), gaze=HEAD_HEIGHT + 0.041),
    ]

    for index, variant in enumerate(variants):
        group = core.empty(f"head_{index}", head)
        at = _helm_surface(variant["crown"], variant["taper"], variant["brow"], variant["gape"])
        parts = _helm(f"helm_{index}", at)

        if index == 0:
            brow = at(-math.pi / 2, 1.0)
            parts.append(
                core.tube(
                    "nasal",
                    [brow + Vector((0, -0.004, 0.014)), brow + Vector((0, -0.013, -0.05)), brow + Vector((0, -0.007, -0.118))],
                    lambda t: 0.016 - 0.008 * t,
                    core.material("metal"),
                    sides=8,
                    flatten=0.4,
                )
            )

        if index == 2:
            parts.append(_beak("visor"))

        helm = core.join(parts, f"helm_{index}")
        helm.parent = group
        place(finish(core.shade_smooth(helm, angle=50), wear_boost=1.6, seed=index))

        if index == 1:
            # Six feathers rooted along the ridge, each longer and lying
            # further back than the one in front of it.
            generator = random.Random(9)
            mesh = bmesh.new()

            for feather in range(6):
                ridge = -0.5 + 0.17 * feather
                base = at(math.copysign(math.pi / 2, ridge), abs(ridge), 0.97)
                rise = math.radians(62 - 4.4 * feather)
                direction = Vector(((feather % 2 - 0.5) * 0.22, math.cos(rise), math.sin(rise)))

                _feather(mesh, base, base + direction * (0.17 + 0.026 * feather), (0, -1, 0.3), 0.05, generator, curl=0.1, quill=0.0)

            crest = core.object_from_bmesh("crest", mesh, core.material("feather"), group)
            place(finish(core.shade_smooth(crest, angle=40), wear_boost=1.4, seed=6))

        place(void_head(group, 0.0, f"void_{index}"))
        place(_aventail(f"wrap_{index}", group, seed=50 + index))
        place(_narrow(eyes(group, 0.0, spacing=0.035, height=variant["gaze"], size=0.011, name=f"eyes_{index}")))
