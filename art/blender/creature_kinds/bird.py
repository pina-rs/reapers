"""The bird: Thugbirdz.

A tough little street bird caught mid-beat: a sleek sculpted skull sunk into
heavy feathered shoulders, a crisp layered collar ruff, sculpted hooked beak,
glossy wrap-around cyberpunk visor, individually lofted airfoil flight
feathers, and a fanned feathered tail.

It wears a crisp backwards snapback cap or a chunky Cuban-link chain.
"""

import math

import bmesh
import bpy
from mathutils import Matrix, Vector

from lib import core, creature

SKULL = Vector((0, -0.01, 0.085))
SKULL_SIZE = Vector((0.135, 0.14, 0.135))


def build(spec):
    root = core.empty("husk")
    body = core.material("body", color=(1.0, 1.0, 1.0), roughness=0.70, pbr="plumage")
    trim = core.material("trim", color=(1.0, 1.0, 1.0), roughness=0.78, pbr="plumage")
    detail = core.material("detail", color=(1.0, 1.0, 1.0), roughness=0.38, metallic=0.18, pbr="skin")
    dark = core.material("dark", color=(1.0, 1.0, 1.0), roughness=0.10, metallic=0.22)
    accent = core.material("accent", color=(1.0, 1.0, 1.0), roughness=0.20)

    # Compact aerodynamic hunched street-bird body.
    torso = creature.form(
        "torso",
        [
            ("sphere", (0, 0.045, -0.07), (0.198, 0.222, 0.184), (math.radians(-12), 0, 0)),
            ("sphere", (0, 0.11, 0.005), (0.162, 0.155, 0.130)),
            ("sphere", (0.12, 0.02, 0.032), (0.084, 0.090, 0.075)),
            ("sphere", (-0.12, 0.02, 0.032), (0.084, 0.090, 0.075)),
        ],
        body,
        root,
        triangles=1600,
        voxel=0.014,
    )

    # Smooth sculpted breast plumage + clean continuous lofted feathered collar ruff
    # (replaces the bumpy remeshed sphere ring).
    breast = creature.form(
        "front_breast",
        [
            ("sphere", (0, -0.052, -0.098), (0.154, 0.145, 0.148)),
            ("sphere", (0, -0.082, -0.035), (0.128, 0.100, 0.096)),
        ],
        trim,
        root,
        triangles=650,
        voxel=0.013,
    )
    ruff_steps = 36
    ruff_path = [_collar(math.tau * step / ruff_steps) for step in range(ruff_steps + 1)]
    ruff = core.tube(
        "front_ruff",
        ruff_path,
        lambda t: 0.042 * (1.0 + 0.18 * math.sin(t * math.tau * 10)),
        trim,
        sides=10,
        cap=False,
        flatten=0.82,
    )
    front = core.join([breast, ruff], "front")
    front.parent = root
    creature.finish(core.shade_smooth(front))

    _tail(root, body)
    _feet(root, detail)
    _head(root, body, detail, dark, accent)

    for side, name in ((1, "l"), (-1, "r")):
        _wing(root, side, name, body)

    _chain(creature.pivot("trait_2", root, (0, 0, 0)), detail, (torso, front))

    creature.studs(root, spec)

    return root


def _collar(angle):
    ahead = ((1 + math.cos(angle)) / 2) ** 0.6
    return Vector((math.sin(angle) * 0.136, -0.045 - math.cos(angle) * 0.110, 0.098 - 0.136 * ahead))


def _along(direction, up=(0, 0, 1)):
    forward = Vector(direction).normalized()
    across = forward.cross(Vector(up)).normalized()
    normal = across.cross(forward)
    return Matrix((across, forward, normal)).transposed().to_euler()


def _add_feather(mesh, base, direction, length, width, up=(0, 0, 1), thickness=0.010, curl=0.045, segments=6):
    """Adds one sculpted airfoil feather with a raised central quill ridge to `mesh`."""

    base = Vector(base)
    forward = Vector(direction).normalized()
    side = forward.cross(Vector(up)).normalized()
    normal = side.cross(forward).normalized()

    rings = []
    for step in range(segments + 1):
        t = step / segments
        centre = base + forward * (length * t) + normal * (curl * length * 4.0 * t * (1.0 - t))
        vane = width * math.sin(max(0.05, t) ** 0.68 * math.pi) ** 0.75 * (1.0 - 0.12 * t)
        vane = max(vane, 0.0025)
        thick = thickness * (1.0 - 0.72 * t)
        cup = vane * 0.16

        rings.append(
            [
                centre - side * vane - normal * cup,
                centre - side * (vane * 0.45) + normal * (thick * 0.45),
                centre + normal * thick,
                centre + side * (vane * 0.45) + normal * (thick * 0.45),
                centre + side * vane - normal * cup,
                centre - normal * (thick * 0.55),
            ]
        )

    rows = [[mesh.verts.new(pt) for pt in ring] for ring in rings]
    n = len(rows[0])
    for upper, lower in zip(rows, rows[1:]):
        for i in range(n):
            j = (i + 1) % n
            mesh.faces.new((upper[i], lower[i], lower[j], upper[j]))
    mesh.faces.new(rows[0])
    mesh.faces.new(list(reversed(rows[-1])))


def _wing(root, side, name, feathers):
    """Crisp sculpted bird wing with leading arm bone and layered airfoil flight feathers."""

    wing = creature.pivot(f"wing_{name}", root, (side * 0.17, 0.052, 0.05))

    def at(x, y, z):
        return Vector((side * x, y, z))

    shoulder, wrist, tip = at(0, 0, 0), at(0.15, -0.03, 0.1), at(0.25, 0.01, 0.08)

    arm_bar = core.tube(
        f"wing_arm_{name}",
        core.bezier([shoulder, wrist, tip], samples=5),
        lambda t: 0.040 * (1.0 - 0.55 * t),
        feathers,
        sides=10,
        flatten=0.72,
    )

    mesh = bmesh.new()
    # Primary & secondary flight feathers fanning cleanly back and outward.
    lengths = (0.17, 0.20, 0.22, 0.23, 0.23, 0.22, 0.20, 0.17)
    for index, length in enumerate(lengths):
        along = index / (len(lengths) - 1)
        reach = 0.10 + 0.90 * along
        base = shoulder.lerp(wrist, reach / 0.55) if reach < 0.55 else wrist.lerp(tip, (reach - 0.55) / 0.45)
        fan = math.radians(-6 + 70 * along**1.55)
        droop = math.radians(42 - 18 * along)
        direction = (side * math.sin(fan) * math.cos(droop), math.cos(fan) * math.cos(droop), -math.sin(droop))
        _add_feather(mesh, base, direction, length, 0.044, up=(side * 0.28, 0, 1), thickness=0.011, segments=6)

    # Shorter overlapping covert feathers along the upper wing ridge for depth.
    for index in range(6):
        along = index / 5
        reach = 0.10 + 0.78 * along
        base = (shoulder.lerp(wrist, reach / 0.55) if reach < 0.55 else wrist.lerp(tip, (reach - 0.55) / 0.45)) + Vector((0, -0.008, 0.010))
        fan = math.radians(-4 + 54 * along**1.5)
        droop = math.radians(34 - 14 * along)
        direction = (side * math.sin(fan) * math.cos(droop), math.cos(fan) * math.cos(droop), -math.sin(droop))
        _add_feather(mesh, base, direction, 0.115, 0.036, up=(side * 0.24, 0, 1), thickness=0.009, segments=4)

    blade = core.object_from_bmesh(f"wing_blade_{name}", mesh, feathers)
    joined = core.join([arm_bar, blade], f"feathers_{name}")
    joined.parent = wing
    creature.finish(core.shade_smooth(joined, angle=46))


def _tail(root, feathers):
    tail = creature.pivot("tail", root, (0, 0.24, 0.02))
    mesh = bmesh.new()
    for spread in (-40, -20, 0, 20, 40):
        swing = math.radians(spread)
        fall = math.radians(22 + abs(spread) * 0.15)
        direction = (math.sin(swing) * math.cos(fall), math.cos(swing) * math.cos(fall), -math.sin(fall))
        _add_feather(
            mesh,
            (0, -0.01, 0),
            direction,
            0.25 - abs(spread) * 0.001,
            0.048,
            up=(math.sin(swing) * 0.45, 0, 1),
            thickness=0.012,
            segments=6,
        )
    part = core.object_from_bmesh("tail_fan", mesh, feathers, tail)
    creature.finish(core.shade_smooth(part, angle=48))


def _feet(root, horn):
    blobs = []
    for side in (1, -1):
        blobs.append(("sphere", (side * 0.075, -0.04, -0.235), (0.032, 0.034, 0.034)))
        for toe in (-1, 0, 1):
            blobs.append(
                ("sphere", (side * 0.075 + toe * 0.026, -0.08, -0.257), (0.015, 0.042, 0.015), (math.radians(28), 0, math.radians(toe * -14)))
            )
    creature.form("feet", blobs, horn, root, triangles=560, voxel=0.008)


def _head(root, body, detail, dark, accent):
    head = creature.pivot("head", root, (0, -0.08, 0.0))

    # Sleek stylized street-bird cranium with subtle cheek & crown tuft contour.
    creature.form(
        "skull",
        [
            ("sphere", SKULL, SKULL_SIZE),
            ("sphere", SKULL + Vector((0, 0.025, 0.095)), (0.085, 0.095, 0.052)),
            ("sphere", SKULL + Vector((0.085, -0.02, -0.035)), (0.065, 0.070, 0.060)),
            ("sphere", SKULL + Vector((-0.085, -0.02, -0.035)), (0.065, 0.070, 0.060)),
        ],
        body,
        head,
        triangles=1100,
        voxel=0.012,
    )

    _visor(head, dark, accent)

    _horn(
        "beak_upper",
        Vector((0, -0.125, 0)),
        0.128,
        lambda t: 0.070 * (1 - t**2.3) + 0.006,
        lambda t: 0.104 - 0.094 * t**2.3,
        lambda t: 0.030 - 0.056 * (max(0.0, t - 0.50) / 0.50) ** 2,
        detail,
        head,
        steps=10,
    )

    jaw = creature.pivot("jaw", head, (0, -0.125, 0.03))
    _horn(
        "beak_lower",
        Vector((0, 0, 0)),
        0.086,
        lambda t: 0.056 * (1 - t**2.4) + 0.005,
        lambda t: 0.0,
        lambda t: -0.005 - 0.036 * (1 - t) ** 0.8,
        detail,
        jaw,
        keel=True,
        steps=9,
    )

    _cap(creature.pivot("trait_1", head, (0, 0, 0)), detail)


def _horn(name, origin, length, width, top, bottom, mat, parent, keel=False, steps=9):
    rings = []
    for step in range(steps + 1):
        t = step / steps
        half, high, low = width(t), top(t), bottom(t)
        flat, ridge = (high, low) if keel else (low, high)

        def point(across, toward):
            return origin + Vector((half * across, -length * t, flat + (ridge - flat) * toward))

        rings.append([point(-1, 0), point(-0.92, 0.5), point(-0.52, 0.9), point(0, 1), point(0.52, 0.9), point(0.92, 0.5), point(1, 0)])

    horn = core.loft(name, rings, mat, parent, cap_start=True, cap_end=True)
    return creature.finish(core.shade_smooth(horn, angle=52))


def _band(name, radii, sweep, edges, depth, mat, parent, steps=26, origin=Vector()):
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
    return creature.finish(core.shade_smooth(band, angle=38))


def _visor(head, dark, accent):
    def lens(fraction):
        narrow = max(0.0, fraction - 0.5) / 0.5
        scowl = max(0.0, 1 - fraction / 0.3)
        return -0.006 + 0.022 * narrow, 0.064 - 0.014 * narrow - 0.013 * scowl, 0.018 * narrow

    _band("visor", (0.154, 0.159), math.radians(106), lens, 0.05, dark, head)

    middle = SKULL + Vector((0, 0, 0.0265))
    eyes = creature.pivot("eyes", head, middle)
    _band("eyes", (0.157, 0.162), math.radians(52), lambda fraction: (0.020, 0.033, 0), 0.012, accent, eyes, steps=16, origin=middle)

    for side in (1, -1):
        angle = side * math.radians(21)
        at = SKULL + Vector((math.sin(angle) * 0.160, -math.cos(angle) * 0.164, 0.068))
        creature.brow(side, head, at, (0.044, 0.014, 0.011), dark, rotation=(0, 0, angle))


def _cap(parent, cloth):
    """Crisp backwards snapback cap with flat sculpted bill."""

    size = SKULL_SIZE + Vector((0.012, 0.012, 0.008))
    brim = 0.064
    segments = 28
    lifts = [brim + (size.z - brim) * math.sin(step / 7 * math.pi / 2) for step in range(7)]
    rings = []

    for inset, lift in ((0.03, brim), *((0.0, lift) for lift in lifts)):
        level = math.sqrt(max(0.0, 1 - (lift / size.z) ** 2))
        ring = []
        for segment in range(segments):
            angle = math.tau * segment / segments
            ring.append(SKULL + Vector((math.cos(angle) * (size.x * level - inset), math.sin(angle) * (size.y * level - inset), lift)))
        rings.append(ring)

    creature.finish(core.shade_smooth(core.loft("cap", rings, cloth, parent, cap_end=True), angle=55))

    # Crisp flat street-style backwards snapback bill instead of a squashed sphere.
    bill_rings = []
    for step in range(6):
        t = step / 5
        span = 0.095 * (1.0 - 0.36 * (t**1.8))
        y = size.y - 0.02 + 0.125 * t
        z = brim + 0.008 + 0.014 * t
        ring = []
        for s in range(10):
            u = (s / 9) * 2.0 - 1.0
            ring.append(SKULL + Vector((u * span, y - 0.022 * (u * u), z + 0.006)))
        for s in range(9, -1, -1):
            u = (s / 9) * 2.0 - 1.0
            ring.append(SKULL + Vector((u * span, y - 0.022 * (u * u), z - 0.006)))
        bill_rings.append(ring)

    bill = core.loft("cap_bill", bill_rings, cloth, parent, closed=True, cap_start=True, cap_end=True)
    creature.finish(core.shade_smooth(bill, angle=48))


def _link(centre, along, across, mat):
    normal = along.cross(across).normalized()
    rings = []
    steps = 10

    for step in range(steps + 1):
        angle = math.tau * step / steps
        out = (along * math.cos(angle) + across * math.sin(angle)).normalized()
        middle = centre + along * math.cos(angle) * 0.025 + across * math.sin(angle) * 0.016
        rings.append([middle + (out * math.cos(turn) + normal * math.sin(turn)) * 0.0075 for turn in (0, math.pi / 2, math.pi, math.pi * 1.5)])

    return core.loft("link", rings, mat)


def _rest(point, bodies, lift):
    best = None
    for obj in bodies:
        _, surface, normal, _ = obj.closest_point_on_mesh(obj.matrix_world.inverted() @ point)
        surface = obj.matrix_world @ surface
        if best is None or (surface - point).length < (best[0] - point).length:
            best = (surface, normal)
    return best[0] + best[1] * lift, best[1]


def _chain(parent, metal, bodies):
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

    bpy.context.view_layer.update()

    count = round(lengths[-1] / 0.037 / 2) * 2
    pitch = lengths[-1] / count
    links = []

    for index in range(count):
        centre, outward = _rest(walk(index * pitch), bodies, 0.011)
        along = (walk(index * pitch + 0.01) - walk(index * pitch - 0.01)).normalized()
        flat = along.cross(outward).normalized()
        links.append(_link(centre, along, flat if index % 2 else flat.cross(along), metal))

    chain = core.join(links, "chain")
    chain.parent = parent
    creature.finish(core.shade_smooth(chain, angle=46))
