"""Light the reaper's weapons and the capture orb throw: muzzle flashes, a
tracer, soul-shard sparks, halos, a ring wave, the soul streaming in and a
heavy swing's trail.

None of these is lit. They are drawn with `fx_glow`, an unlit material whose
colour is a soft white spot, brightest in the middle of its texture and
black at the edges: the game tints it with the light's colour and adds it
to what is behind (leptoskit's additive blend), so a mesh's texture
coordinates decide where it is bright. `fx_shade` is the one darkening
effect: a soft disc, blended normally and tinted black by the game, that
goes behind a glow so it still reads against a bright wall.

Every effect is built at a unit size about its own origin, and the game
scales, turns and fades it: they hold no animation.

Each effect is one mesh, and one draw while it shows.
"""

import math
import random

import bmesh
import bpy

from lib import core

GLOW = "fx_glow"
SHADE = "fx_shade"
LIGHT = "fx_light"
SPOT_PIXELS = 64


def _spot(name, alpha):
    """A soft round spot: white in the middle, falling smoothly to nothing
    at the edge, in the colour or, with `alpha`, in the alpha channel."""

    existing = bpy.data.images.get(name)

    if existing is not None:
        return existing

    image = bpy.data.images.new(name, SPOT_PIXELS, SPOT_PIXELS, alpha=alpha)
    pixels = []

    for y in range(SPOT_PIXELS):
        for x in range(SPOT_PIXELS):
            dx = (x + 0.5) / SPOT_PIXELS * 2 - 1
            dy = (y + 0.5) / SPOT_PIXELS * 2 - 1
            falloff = max(0.0, 1.0 - math.sqrt(dx * dx + dy * dy))
            # A hot middle and a long soft skirt.
            value = round(falloff**2.2 * 0.75 + falloff**8 * 0.25, 4)

            if alpha:
                pixels.extend((1.0, 1.0, 1.0, value))
            else:
                pixels.extend((value, value, value, 1.0))

    image.pixels = pixels
    image.file_format = "PNG"
    image.pack()

    return image


def _unlit(name, image=None):
    existing = bpy.data.materials.get(name)

    if existing is not None:
        return existing

    result = bpy.data.materials.new(name)
    result.use_nodes = True
    result.use_backface_culling = False
    nodes = result.node_tree.nodes
    nodes.clear()
    output = nodes.new("ShaderNodeOutputMaterial")
    background = nodes.new("ShaderNodeBackground")
    background.inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
    result.node_tree.links.new(background.outputs["Background"], output.inputs["Surface"])

    if image is not None:
        texture = nodes.new("ShaderNodeTexImage")
        texture.image = image
        result.node_tree.links.new(texture.outputs["Color"], background.inputs["Color"])

    return result


def glow_material():
    """The unlit soft white spot that every light here is drawn with."""

    return _unlit(GLOW, _spot("fx_spot", alpha=False))


def light_material():
    """Unlit flat white, for the orb's core: the game tints it."""

    return _unlit(LIGHT)


def shade_material():
    """A soft disc that fades out toward its edge, blended normally."""

    existing = bpy.data.materials.get(SHADE)

    if existing is not None:
        return existing

    result = bpy.data.materials.new(SHADE)
    result.use_nodes = True
    result.use_backface_culling = False
    nodes = result.node_tree.nodes
    shader = nodes.get("Principled BSDF")
    texture = nodes.new("ShaderNodeTexImage")
    texture.image = _spot("fx_spot_alpha", alpha=True)
    result.node_tree.links.new(texture.outputs["Color"], shader.inputs["Base Color"])
    result.node_tree.links.new(texture.outputs["Alpha"], shader.inputs["Alpha"])
    shader.inputs["Roughness"].default_value = 1.0

    return result


def _mesh(name, parent, mat, quads):
    """One mesh from `quads`, each a polygon of corners as ((x, y, z),
    (u, v))."""

    mesh = bmesh.new()
    layer = mesh.loops.layers.uv.new("UVMap")

    for quad in quads:
        verts = [mesh.verts.new(corner) for corner, _ in quad]
        face = mesh.faces.new(verts)

        for loop, (_, uv) in zip(face.loops, quad):
            loop[layer].uv = uv

    data = bpy.data.meshes.new(name)
    mesh.to_mesh(data)
    mesh.free()
    result = bpy.data.objects.new(name, data)
    bpy.context.scene.collection.objects.link(result)
    data.materials.append(mat)

    if parent is not None:
        result.parent = parent

    return result


def _disc_quads(radius, segments, plane, spin=0.0, star=None):
    """A flat disc (or star, with `star` its inner radius and point count)
    as a fan of triangles, its texture coordinates laid flat so the spot
    sits in its middle. `plane` maps (a, b) in the disc to a point."""

    triangles = []
    count = segments if star is None else star[1] * 2

    def corner(index):
        angle = spin + math.tau * index / count
        reach = radius if star is None or index % 2 == 0 else star[0]
        a, b = math.cos(angle) * reach, math.sin(angle) * reach

        return plane(a, b), (0.5 + a / (2 * radius), 0.5 + b / (2 * radius))

    middle = (plane(0.0, 0.0), (0.5, 0.5))

    for index in range(count):
        triangles.append([middle, corner(index), corner(index + 1)])

    return triangles


def _blade_quads(start, end, width, glow_along=False):
    """Two crossed quads from `start` to `end`, `width` across, so a line of
    light reads from any side. Bright down its middle; with `glow_along`,
    also brightest at `start` and fading to `end`."""

    sx, sy, sz = start
    ex, ey, ez = end
    dx, dy, dz = ex - sx, ey - sy, ez - sz
    length = math.sqrt(dx * dx + dy * dy + dz * dz)
    dx, dy, dz = dx / length, dy / length, dz / length
    # Two directions across the line.
    helper = (0.0, 0.0, 1.0) if abs(dz) < 0.9 else (1.0, 0.0, 0.0)
    ax = (dy * helper[2] - dz * helper[1], dz * helper[0] - dx * helper[2], dx * helper[1] - dy * helper[0])
    norm = math.sqrt(sum(value * value for value in ax))
    ax = tuple(value / norm for value in ax)
    bx = (dy * ax[2] - dz * ax[1], dz * ax[0] - dx * ax[2], dx * ax[1] - dy * ax[0])
    head, tail = (0.5, 0.5) if not glow_along else (0.5, 1.0)
    quads = []

    for across in (ax, bx):
        half = [value * width / 2 for value in across]
        quads.append(
            [
                ((sx - half[0], sy - half[1], sz - half[2]), (head, 0.0)),
                ((ex - half[0], ey - half[1], ez - half[2]), (tail, 0.0)),
                ((ex + half[0], ey + half[1], ez + half[2]), (tail, 1.0)),
                ((sx + half[0], sy + half[1], sz + half[2]), (head, 1.0)),
            ]
        )

    return quads


# The muzzle flash ------------------------------------------------------

# Three frames, shown one after another for a frame each: a wide hot star,
# a bigger ragged one, and a small one going out. Each is a star facing
# back along the barrel, toward the eye, and a spike thrown forward.
FLASH_FRAMES = (
    # (outer radius, inner radius, points, spin, spike length)
    (0.042, 0.016, 8, 0.0, 0.07),
    (0.06, 0.018, 6, 0.3, 0.12),
    (0.032, 0.014, 5, 0.9, 0.05),
)


def muzzle_flashes(muzzle):
    for frame, (outer, inner, points, spin, spike) in enumerate(FLASH_FRAMES):
        quads = _disc_quads(outer, None, lambda a, b: (a, -0.004, b), spin, star=(inner, points))
        quads += _blade_quads((0.0, 0.0, 0.0), (0.0, -spike, 0.0), outer * 0.7, glow_along=True)
        flash = _mesh(f"vm_flash_{frame}", muzzle, glow_material(), quads)
        flash.hide_render = frame > 0


# The heavy swing's trail -----------------------------------------------

TRAIL_DEGREES = 120
TRAIL_SEGMENTS = 20
TRAIL_HEIGHT = 0.16
# Where along the arc the trail is brightest. Past it the light runs on out
# to the spot's far edge and the arc rounds off, so the head is soft: it
# once ended on its brightest line.
TRAIL_PEAK = 0.85


def trail(parent):
    """An arc of light, a unit's radius about the origin in the horizontal
    plane, from the tail at angle 0 round to the head at 120 degrees: dim at
    its tail, brightest just behind its head, and soft at both ends."""

    quads = []

    for segment in range(TRAIL_SEGMENTS):
        corners = []

        for step, v, z in ((segment, 0.0, -TRAIL_HEIGHT / 2), (segment + 1, 0.0, -TRAIL_HEIGHT / 2), (segment + 1, 1.0, TRAIL_HEIGHT / 2), (segment, 1.0, TRAIL_HEIGHT / 2)):
            share = step / TRAIL_SEGMENTS
            angle = math.radians(TRAIL_DEGREES) * share
            # Thinner toward the tail, and rounded off at the head.
            height = z * (0.35 + 0.65 * share)

            if share <= TRAIL_PEAK:
                u = 0.5 * share / TRAIL_PEAK
            else:
                past = (share - TRAIL_PEAK) / (1.0 - TRAIL_PEAK)
                u = 0.5 + 0.5 * past
                height *= math.sqrt(max(0.0, 1.0 - past * past))

            corners.append(((math.cos(angle), math.sin(angle), height), (u, v)))

        quads.append(corners)

    return _mesh("vm_trail", parent, glow_material(), quads)


# Halos, flares and the ring wave ---------------------------------------


def _facing(a, b):
    # Facing -Y in Blender, which is +Z once exported: toward a camera that
    # looks down -Z at it.
    return (a, 0.0, b)


def halo(name, parent, mat):
    """A unit disc facing the eye, for the game to turn toward the camera."""

    return _mesh(name, parent, mat, _disc_quads(1.0, 24, _facing))


def ring(parent):
    """A thin flat ring of light a unit's radius across, lying on the
    ground about the origin: the wave each pulse of the orb sends out."""

    quads = []
    segments = 40
    inner, outer = 0.9, 1.0

    for segment in range(segments):
        a0 = math.tau * segment / segments
        a1 = math.tau * (segment + 1) / segments
        quads.append(
            [
                ((math.cos(a0) * inner, math.sin(a0) * inner, 0.0), (0.5, 0.0)),
                ((math.cos(a1) * inner, math.sin(a1) * inner, 0.0), (0.5, 0.0)),
                ((math.cos(a1) * outer, math.sin(a1) * outer, 0.0), (0.5, 1.0)),
                ((math.cos(a0) * outer, math.sin(a0) * outer, 0.0), (0.5, 1.0)),
            ]
        )

    return _mesh("fx_ring", parent, glow_material(), quads)


def tracer(parent):
    """A line of light a unit long from the origin up +Z, a unit wide: the
    game stretches it from the muzzle to what the shot hit."""

    return _mesh("fx_tracer", parent, glow_material(), _blade_quads((0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 1.0))


def sparks(name, parent, seed, count=11):
    """A burst of soul shards flying out from the origin, a unit across at
    most. The game scales it out and fades it."""

    generator = random.Random(seed)
    quads = []

    for shard in range(count):
        # Spread evenly over a sphere, then jittered.
        height = 1 - 2 * (shard + 0.5) / count
        around = shard * math.pi * (3 - math.sqrt(5)) + generator.uniform(-0.3, 0.3)
        flat = math.sqrt(max(0.0, 1 - height * height))
        direction = (math.cos(around) * flat, math.sin(around) * flat, height)
        near = generator.uniform(0.12, 0.3)
        far = generator.uniform(0.65, 1.0)
        quads += _blade_quads(
            tuple(value * near for value in direction),
            tuple(value * far for value in direction),
            generator.uniform(0.05, 0.08),
            glow_along=True,
        )

    return _mesh(name, parent, glow_material(), quads)


# The soul streaming in -------------------------------------------------

STREAM_MOTES = 36
STREAM_TURNS = 2.5


def _stream_point(share):
    """Where a mote sits along the stream, from the husk at 0 to the orb at
    1, up +Z: a helix that tightens as it nears the orb."""

    angle = math.tau * STREAM_TURNS * share
    radius = 0.13 * (1 - share) ** 1.4 + 0.01

    return (math.cos(angle) * radius, math.sin(angle) * radius, share)


def stream(parent):
    """A helix of soul motes a unit long, from the husk at the origin to the
    orb up +Z. Two morph targets move it: `flow` carries every mote to where
    the next one was, so turning it up over and over draws the souls in;
    `gather` pulls every mote into the orb."""

    shares = [index / STREAM_MOTES for index in range(STREAM_MOTES + 1)]
    centres = [_stream_point(share) for share in shares]
    triangles = []
    corners_per_mote = 24

    for index in range(STREAM_MOTES):
        cx, cy, cz = centres[index]
        size = 0.028 * (1 - 0.6 * shares[index])
        tips = [(cx + size, cy, cz), (cx, cy + size, cz), (cx - size, cy, cz), (cx, cy - size, cz)]
        top = (cx, cy, cz + size * 1.6)
        bottom = (cx, cy, cz - size * 1.6)

        for side in range(4):
            a, b = tips[side], tips[(side + 1) % 4]
            # Bright all over: the mote is small, the spot's middle is enough.
            triangles.append([(top, (0.5, 0.5)), (a, (0.5, 0.5)), (b, (0.5, 0.5))])
            triangles.append([(bottom, (0.5, 0.5)), (b, (0.5, 0.5)), (a, (0.5, 0.5))])

    result = _mesh("fx_stream", parent, glow_material(), triangles)
    result.shape_key_add(name="Basis")
    end = centres[-1]

    for name in ("flow", "gather"):
        key = result.shape_key_add(name=name)

        for index, vertex in enumerate(result.data.vertices):
            mote = index // corners_per_mote
            centre = centres[mote]

            if name == "flow":
                to = centres[mote + 1]
                key.data[index].co = tuple(vertex.co[axis] + to[axis] - centre[axis] for axis in range(3))
            else:
                key.data[index].co = tuple(end[axis] + (vertex.co[axis] - centre[axis]) * 0.3 for axis in range(3))

        key.value = 0.0

    return result


def build(root):
    """Every effect the world shows, under `root`, each at rest and hidden
    until the game shows it."""

    parts = [
        halo("fx_shade", root, shade_material()),
        halo("fx_halo", root, glow_material()),
        halo("fx_flare", root, glow_material()),
        sparks("fx_sparks_0", root, seed=3),
        sparks("fx_sparks_1", root, seed=8),
        ring(root),
        stream(root),
        tracer(root),
    ]

    return parts
