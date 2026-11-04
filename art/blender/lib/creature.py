"""What every creature shares: its moving parts, its seals and its shadow.

A creature is not skinned. It is built from rigid parts, each hanging from a
pivot, and the game moves the pivots: legs swing, wings beat, a head turns
to watch the player. That keeps the models simple to build and lets the game
drive the motion from what the creature is actually doing.

See README.md for the full convention.
"""

import math

import bmesh
import bpy
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


def floor(spec):
    """The height of the floor under a creature that walks, in Blender's Z.
    The middle of the body is the origin, so the floor is half a body down."""

    return -spec["halfHeight"]


def studs(root, spec, radius=0.028):
    """A small bright stud at each seal, so there is something to strike at.

    They are named seal_0, seal_1 and so on. Model the body so that each
    stud sits on or just proud of its surface.
    """

    made = []

    for index, seal in enumerate(spec["seals"]):
        bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2, radius=radius, location=to_blender(seal["offset"]))

        stud = bpy.context.object
        stud.name = f"seal_{index}"
        stud.data.materials.append(core.material("glow"))
        stud.parent = root
        core.shade_smooth(stud)
        made.append(stud)

    return made


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

    return core.object_from_bmesh("shadow", mesh, core.material("shadow"), root, smooth=False)


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

    if problems:
        raise SystemExit("This creature does not follow art/blender/README.md:\n  - " + "\n  - ".join(problems))


def _under(node, prefix):
    while node is not None:
        if node.name.startswith(prefix):
            return True

        node = node.parent

    return False
