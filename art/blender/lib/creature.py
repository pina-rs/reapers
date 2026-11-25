"""What every creature shares: its moving parts, its seals and its shadow.

A creature is not skinned. It is built from rigid parts, each hanging from a
pivot, and the game moves the pivots: legs swing, wings beat, a head turns
to watch the player. That keeps the models simple to build and lets the game
drive the motion from what the creature is actually doing.

See README.md for the full convention.
"""

import math

import bmesh
from mathutils import Vector

from . import core

# The names the game looks for, and what it does with each. A creature uses
# whichever of these its body has.
MOVING_PARTS = (
    "head",
    "neck",
    "jaw",
    "leg_l",
    "leg_r",
    "leg_fl",
    "leg_fr",
    "leg_bl",
    "leg_br",
    "arm_l",
    "arm_r",
    "wing_l",
    "wing_r",
    "tail",
    # The face. Each eye hangs from its own pivot, or both from one when
    # they are a single band, and a brow over each, so the game can widen
    # the eyes in terror, squeeze them shut at a blow and raise or knit the
    # brows. See README.md.
    "eye_l",
    "eye_r",
    "eyes",
    "brow_l",
    "brow_r",
)

# Which way a side is, in a pivot's name: +x is the creature's left.
SIDES = {1: "l", -1: "r"}

# The only materials a creature may use. The game throws Blender's material
# away and keeps the name, which picks the surface.
MATERIALS = ("body", "trim", "detail", "accent", "dark", "glow", "shadow")

# How many things a creature can wear: nodes trait_1 .. trait_<TRAITS>.
TRAITS = 2


def to_blender(offset):
    """A position in the rules' frame (x to the creature's left, y up, z
    forward) as a Blender position (the creature faces -Y)."""

    return Vector((offset[0], -offset[2], offset[1]))


def pivot(name, parent, at):
    """An unrotated empty at `at`, in the parent's space: the joint a moving
    part turns about. Parent the part's meshes to it.

    The game turns pivots about their own axes, which only works if a pivot
    is never rotated or scaled itself. Model the part in place instead.
    """

    return core.empty(name, parent, location=at)


def eye(side, head, at, size, mat, rotation=(0, 0, 0), triangles=60):
    """One eye on its own pivot at `at`, in the head's space: a single blob
    of `size`, made about the pivot so the game scales it in place.

    `side` is 1 for the creature's left and -1 for its right.
    """

    joint = pivot(f"eye_{SIDES[side]}", head, at)

    form(f"eye_{side}", [("sphere", (0, 0, 0), size, rotation)], mat, joint, triangles=triangles)

    return joint


def brow(side, head, at, size, mat, rotation=(0, 0, 0), triangles=90):
    """A brow over one eye, on its own pivot at `at`, in the head's space.

    The game lifts the inner end of each brow when the creature is afraid
    and drives it down when it is furious, turning the brow about its
    middle, so model it level and centred on the pivot.
    """

    joint = pivot(f"brow_{SIDES[side]}", head, at)

    form(f"brow_{side}", [("sphere", (0, 0, 0), size, rotation)], mat, joint, triangles=triangles)

    return joint


def finish(obj, **masks):
    """Paints the occlusion the game shades a creature with."""

    return core.paint_masks(obj, **masks)


def form(name, blobs, mat, parent, triangles=900, voxel=None):
    """One organic part, fused from overlapping spheres and cubes, cut down
    to about `triangles` triangles and finished.

    `blobs` is what `core.sculpt` takes: (kind, location, scale[, rotation])
    with kind "sphere" or "cube". Fusing is done fine and then simplified, so
    a part keeps its shape without spending triangles it does not need. A
    whole creature has about 14,000 to spend.
    """

    smallest = min(min(scale) for _, _, scale, *_ in blobs)
    part = core.sculpt(name, blobs, mat, parent, voxel=voxel or max(0.006, min(0.02, smallest / 3.5)))
    count = sum(len(polygon.vertices) - 2 for polygon in part.data.polygons)

    if count > triangles:
        core.decimate(part, triangles / count)
        # Simplifying a thin shape can leave faces folded onto each other,
        # which the exporter warns about and may write wrongly.
        part.data.validate()

    core.shade_smooth(part)

    return finish(part)


def facet(name, blobs, mat, parent, triangles=900, voxel=None, flat=9.0, crease=34.0):
    """`form`, cut into planes: the same fused blobs, but the near-flat
    stretches of the surface merged into flat facets, with a crisp edge
    wherever two facets meet at more than `crease` degrees.

    A voxel form simplified to a triangle count shades in hard bands as a
    speckle of small triangles, each a little off its neighbours, with no
    plane for a band edge to run along. A faceted one prints as a few broad
    planes, like a carving, so its band edges are long and straight and its
    creases take a line. `flat` is how far apart, in degrees, two faces may
    turn and still be merged into one facet: raise it for fewer, larger
    planes. It stays at about `triangles` or under.
    """

    smallest = min(min(scale) for _, _, scale, *_ in blobs)
    part = core.sculpt(name, blobs, mat, parent, voxel=voxel or max(0.006, min(0.02, smallest / 3.5)))

    return finish(core.planes(part, triangles, flat, crease))


def floor(spec):
    """The height of the floor under a creature that walks, in Blender's Z.
    The middle of the body is the origin, so the floor is half a body down."""

    return -spec["halfHeight"]


# Planar construction ----------------------------------------------------
#
# A creature drawn in hard bands wants big planes with deliberate edges
# between them, not the speckle a voxel blob decimates to. `shell` lofts a
# form through cross-sections that run from round (power 2) to boxy (power
# 3 to 5): a boxy section is four broad planes joined by tight rounds, so a
# band edge runs straight down the round and each plane prints as one tone.
# `blade` is the same for flat things (a feather, an ear, a crest). Both are
# modelled where they sit, in their parent's space.


def _frames(centres, forward, axis=None):
    """A (tangent, front, side) frame at each centre of a path: `front` is
    `forward` turned square to the path, `side` the path's left. With
    `axis`, every frame is square to it instead of to the path, so the
    sections stay level however the centres wander."""

    count = len(centres)
    frames = []
    forward = Vector(forward).normalized()

    for index in range(count):
        ahead = centres[min(index + 1, count - 1)]
        behind = centres[max(index - 1, 0)]
        tangent = Vector(axis).normalized() if axis is not None else (ahead - behind).normalized()
        front = forward - tangent * forward.dot(tangent)

        if front.length < 1e-4:
            helper = Vector((0, 0, 1)) if abs(tangent.z) < 0.9 else Vector((0, -1, 0))
            front = helper - tangent * helper.dot(tangent)

        front.normalize()
        frames.append((tangent, front, tangent.cross(front).normalized()))

    return frames


def section(across, front, back, power=2.0, sides=16, below=None):
    """The points of one cross-section, as (side, front) offsets: a
    superellipse `across` wide either side, reaching `front` ahead and `back`
    behind. Power 2 is an ellipse; 3 to 5 square it into four planes."""

    exponent = 2.0 / power
    points = []

    for step in range(sides):
        angle = math.tau * step / sides
        c, s = math.cos(angle), math.sin(angle)
        x = math.copysign(abs(c) ** exponent, c) * across
        y = math.copysign(abs(s) ** exponent, s) * (front if s > 0 else back)
        points.append((x, y))

    return points


def shell(name, stations, mat, parent=None, sides=16, start=0.0, end=0.0, crease=None, forward=(0, -1, 0), axis=None):
    """A closed form lofted through `stations`, each `(at, across, front,
    back, power)`: a cross-section centred on `at`, square to the path the
    centres make, `across` wide to either side, reaching `front` toward
    `forward` and `back` away from it (`section`).

    `start` and `end` close each end with a pole that far beyond its last
    ring along the path (0 for a flat cap), or None to leave it open. With
    `crease`, edges sharper than that many degrees shade hard. `axis` keeps
    every section square to one direction (`_frames`): a body's sections
    level as its middle leans. Returns the finished mesh."""

    centres = [Vector(station[0]) for station in stations]
    frames = _frames(centres, forward, axis)
    mesh = bmesh.new()
    rings = []

    for (centre, (tangent, front, side)), (_, across, ahead, behind, power) in zip(zip(centres, frames), stations):
        rings.append([mesh.verts.new(centre + side * x + front * y) for x, y in section(across, ahead, behind, power, sides)])

    for upper, lower in zip(rings, rings[1:]):
        for index in range(sides):
            follow = (index + 1) % sides
            mesh.faces.new((upper[index], lower[index], lower[follow], upper[follow]))

    for ring, centre, (tangent, _, _), reach, way in ((rings[0], centres[0], frames[0], start, -1), (rings[-1], centres[-1], frames[-1], end, 1)):
        if reach is None:
            continue

        pole = mesh.verts.new(centre + tangent * reach * way)

        for index in range(sides):
            mesh.faces.new((ring[index], ring[(index + 1) % sides], pole))

    obj = core.object_from_bmesh(name, mesh, mat, parent)
    core.shade_smooth(obj, angle=crease)

    return finish(obj)


def blade(name, path, width, thickness, mat, parent=None, up=(0, 0, 1), tip=0.0, crease=None, belly=0.55):
    """A flat, thick-edged form along `path`: a feather, an ear, a crest, a
    claw. `width` and `thickness` are numbers or functions of the fraction
    along the path; the blade lies across `up` (its broad faces look along
    it). Its section is a lens, fuller on the `up` side by `belly`, so it
    turns its edge to a band like a carved plank. `tip` closes the far end
    with a pole that far on."""

    points = [Vector(point) for point in path]
    count = len(points)
    frames = _frames(points, up)
    shape = ((-1.0, 0.0), (-0.62, 0.78), (0.0, 1.0), (0.62, 0.78), (1.0, 0.0), (0.62, -belly), (0.0, -belly * 1.15), (-0.62, -belly))
    mesh = bmesh.new()
    rings = []

    for index, (point, (_, normal, side)) in enumerate(zip(points, frames)):
        along = index / max(count - 1, 1)
        wide = width(along) if callable(width) else width
        thick = thickness(along) if callable(thickness) else thickness
        rings.append([mesh.verts.new(point + side * x * wide + normal * y * thick / 2) for x, y in shape])

    sides = len(shape)

    for upper, lower in zip(rings, rings[1:]):
        for index in range(sides):
            follow = (index + 1) % sides
            mesh.faces.new((upper[index], lower[index], lower[follow], upper[follow]))

    mesh.faces.new(list(reversed(rings[0])))
    pole = mesh.verts.new(points[-1] + frames[-1][0] * tip)

    for index in range(sides):
        mesh.faces.new((rings[-1][index], rings[-1][(index + 1) % sides], pole))

    obj = core.object_from_bmesh(name, mesh, mat, parent)
    core.shade_smooth(obj, angle=crease)

    return finish(obj)


def mirrored(point, side):
    """`point` on the creature's `side` (1 left, -1 right)."""

    return Vector((point[0] * side, point[1], point[2]))


def origin(node):
    """Where a node sits on the creature: pivots are never turned or
    scaled, so it is the sum of the offsets up the chain."""

    at = Vector()

    while node is not None:
        at += Vector(node.location)
        node = node.parent

    return at


def surface(objects):
    """A tree to measure `objects`' surfaces against, in the creature's
    space (their pivots are never turned, so a translation takes each
    there)."""

    from mathutils.bvhtree import BVHTree

    positions = []
    faces = []

    for obj in objects:
        at = origin(obj)
        base = len(positions)
        positions.extend(at + vertex.co for vertex in obj.data.vertices)
        faces.extend([base + index for index in polygon.vertices] for polygon in obj.data.polygons)

    return BVHTree.FromPolygons(positions, faces)


def onto(tree, start, direction, lift=0.0, missing=None):
    """Where a ray from `start` along `direction` meets the surface `tree`,
    plus `lift` along the ray, or `missing` when it meets nothing."""

    direction = Vector(direction).normalized()
    hit, _, _, _ = tree.ray_cast(Vector(start), direction)

    return missing if hit is None else hit + direction * lift


# Seals -----------------------------------------------------------------

# A seal is a domed boss pressed into the body: a wax seal. Its face looks
# out along the body's surface where it sits, so it reads as set into the
# creature from wherever it is seen, and it prints as one clean disc.
SEAL_RADIUS = 0.032
SEAL_HEIGHT = 0.016


def studs(root, spec, radius=SEAL_RADIUS, height=SEAL_HEIGHT, body=None):
    """A seal at each of the spec's seals, so there is something to strike
    at, named seal_0, seal_1 and so on: a domed boss centred on the seal,
    its face turned along the surface it sits on. Model the body so that
    each seal sits on its surface (`seal_gaps` measures it).

    `body` is the meshes the seals sit on, every mesh but the traits and
    the shadow when None; they must hang from unturned pivots."""

    if body is None:
        body = [node for node in root.children_recursive if node.type == "MESH" and not _under(node, "trait_") and node.name != "shadow"]

    tree = surface(body) if body else None
    made = []
    profile = [(0.0, height), (radius * 0.55, height * 0.86), (radius * 0.86, height * 0.5), (radius, 0.0), (radius * 0.9, -height * 0.9), (0.0, -height)]

    for index, seal in enumerate(spec["seals"]):
        at = to_blender(seal["offset"])
        facing = _facing(tree, at) if tree is not None else Vector((0, -1, 0))
        stud = core.revolve(f"seal_{index}", profile, core.material("glow", color=(1.0, 1.0, 1.0), roughness=0.20), root, segments=12, cap_top=True, cap_bottom=True)
        turn = Vector((0, 0, 1)).rotation_difference(facing).to_matrix().to_4x4()
        stud.data.transform(turn)
        stud.location = at
        core.shade_smooth(stud, angle=50)
        made.append(stud)

    return made


def _facing(tree, at):
    """The surface's outward normal nearest `at`."""

    point, normal, _, _ = tree.find_nearest(at)

    if point is None:
        return Vector((0, -1, 0))

    return Vector(normal).normalized()


def seal_gaps(root, spec, body=None):
    """How far each seal stands off the body: its centre's distance from
    the outermost surface near it, positive outside, negative buried.
    `body` is as `studs` takes it.

    Parts overlap (a belly laid over a torso), so each part is measured on
    its own: a seal just under the surface of any part (`SEAL_DEPTH` or
    less) is buried by as little as the shallowest of them; otherwise the
    nearest part decides, as a far part's nearest normal says little about
    which side of it the seal is on."""

    if body is None:
        body = [
            node
            for node in root.children_recursive
            if node.type == "MESH" and not node.name.startswith("seal_") and node.name != "shadow" and not _under(node, "trait_")
        ]

    trees = [surface([part]) for part in body]
    gaps = []

    for seal in spec["seals"]:
        at = to_blender(seal["offset"])
        signed = []

        for tree in trees:
            point, normal, _, distance = tree.find_nearest(at, SEAL_REACH)

            if point is not None:
                signed.append(math.copysign(distance, (at - point).dot(normal)))

        inside = [gap for gap in signed if -SEAL_DEPTH <= gap < 0]
        gaps.append(max(inside) if inside else min(signed, key=abs, default=math.inf))

    return gaps


# How far from a seal a part is looked for when measuring its gap, and how
# deep under a part's surface a seal is still taken to be inside it.
SEAL_REACH = 0.25
SEAL_DEPTH = 0.03


def shadow(root, spec, scale=1.15):
    """The soft dark patch under a creature that stands on the floor.

    It is a flat disc whose texture coordinates run from its middle, which is
    all the game's shadow surface needs to fade it out toward the rim.
    """

    radius = max(spec["halfWidth"], 0.2) * scale
    mesh = bmesh.new()
    layer = mesh.loops.layers.uv.new("shadow")
    centre = mesh.verts.new((0, 0, floor(spec) + 0.004))
    steps = 28
    rim = [
        mesh.verts.new(
            (
                math.cos(step / steps * math.tau) * radius,
                math.sin(step / steps * math.tau) * radius,
                floor(spec) + 0.004,
            )
        )
        for step in range(steps)
    ]

    for step in range(steps):
        face = mesh.faces.new((centre, rim[step], rim[(step + 1) % steps]))

        for loop in face.loops:
            at = loop.vert.co

            loop[layer].uv = (0.5 + at.x / radius / 2, 0.5 + at.y / radius / 2)

    return core.object_from_bmesh("shadow", mesh, core.material("shadow", color=(0.0, 0.0, 0.0), roughness=1.0), root, smooth=False)


def check(root, spec):
    """Fails the build when a creature breaks the convention, with a message
    that says what to fix."""

    problems = []
    names = {}

    def visit(node):
        names[node.name] = node

        for child in node.children:
            visit(child)

    visit(root)

    if root.name != "husk":
        problems.append(f'the root must be an empty named "husk", not "{root.name}"')

    for index in range(len(spec["seals"])):
        if f"seal_{index}" not in names:
            problems.append(f"seal_{index} is missing: call creature.studs(root, spec)")

    for trait in range(1, TRAITS + 1):
        if f"trait_{trait}" not in names:
            problems.append(f"trait_{trait} is missing: every creature wears {TRAITS} different things")

    if "head" not in names:
        problems.append('there is no pivot named "head"')

    if spec["gait"] == "ground" and "shadow" not in names:
        problems.append("a creature that walks needs creature.shadow(root, spec)")

    for name in MOVING_PARTS:
        node = names.get(name)

        if node is None:
            continue

        if node.type != "EMPTY":
            problems.append(f"{name} must be a pivot (an empty), with its meshes as children")
        elif any(abs(angle) > 1e-6 for angle in node.rotation_euler) or any(abs(scale - 1) > 1e-6 for scale in node.scale):
            problems.append(f"the pivot {name} is rotated or scaled; pivots must be left as they are made")

    low = math.inf
    high = -math.inf
    wide = 0.0
    used = set()

    for node in names.values():
        if node.type != "MESH" or node.name == "shadow":
            continue

        for slot in node.material_slots:
            if slot.material is not None:
                used.add(slot.material.name.split(".")[0])

        # What a creature wears may stand proud of the body it was sized to.
        if _under(node, "trait_"):
            continue

        for corner in node.bound_box:
            world = node.matrix_world @ Vector(corner)
            low = min(low, world.z)
            high = max(high, world.z)
            wide = max(wide, abs(world.x), abs(world.y))

    for name in sorted(used - set(MATERIALS)):
        problems.append(f'the material "{name}" is not one the game knows; use one of {", ".join(MATERIALS)}')

    half_height = spec["halfHeight"]

    if spec["gait"] == "ground" and abs(low + half_height) > 0.03:
        problems.append(f"its feet must stand on the floor at z = {-half_height:.2f}; the lowest point is at z = {low:.2f}")

    if high > half_height * 1.25 + 0.05:
        problems.append(f"it is too tall: the top is at z = {high:.2f}, and the body should end near z = {half_height:.2f}")

    if high < half_height * 0.7:
        problems.append(f"it is too short: the top is at z = {high:.2f}, and the body should reach near z = {half_height:.2f}")

    if wide > spec["halfWidth"] * 1.6 + 0.1:
        problems.append(f"it is too wide or long: it reaches {wide:.2f} from the middle, against a half width of {spec['halfWidth']:.2f}")

    triangles = core.triangle_count(root)

    if triangles > 14000:
        problems.append(f"it has {triangles} triangles; keep a creature under 14,000")

    # A seal is a boss SEAL_HEIGHT either side of its centre: buried deeper
    # than half that it is a dimple, and lifted further it floats.
    for index, gap in enumerate(seal_gaps(root, spec)):
        print(f"SEAL seal_{index} stands {gap * 1000:+.0f} mm off the body")

        if gap < -SEAL_HEIGHT / 2:
            problems.append(f"seal_{index} is buried {-gap * 1000:.0f} mm deep: shape the body so its surface meets the seal")
        elif gap > SEAL_HEIGHT * 0.75:
            problems.append(f"seal_{index} floats {gap * 1000:.0f} mm off the body: shape the body out to meet it")

    if problems:
        raise SystemExit("This creature does not follow art/blender/README.md:\n  - " + "\n  - ".join(problems))


def _under(node, prefix):
    while node is not None:
        if node.name.startswith(prefix):
            return True

        node = node.parent

    return False
