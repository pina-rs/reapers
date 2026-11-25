"""Scene, mesh and export helpers shared by every Reapers art script.

These scripts only ever run inside Blender (`blender --background --python`),
driven by `cargo xtask build-art`. Blender's authoring API is Python, so
this is the one place in the repository where Python is the right language.

Conventions used throughout:

- Metres, Z up, the character facing -Y (Blender's front view). The glTF
  exporter converts to Y up on the way out.
- Every mesh carries one colour attribute named `Color`. The game's material
  reads its channels as masks, not as a colour:
    R  ambient occlusion and grime (1 is clean and open)
    G  char: how soon this spot burns as an outfit darkens (1 burns first)
    B  wear: raised edges that catch the light
    A  hollow: how soon this spot vanishes as an outfit hollows (0 goes first)
- Every finished mesh carries one UV channel, `UVMap` (`TEXCOORD_0`): a box
  projection of each vertex's object-space position at 3.5 per metre
  (`box_uvs`). No model carries a surface texture: Second Ink draws every
  material in flat bands (docs/art-bible.md, 5.6). Each vertex's ink weight
  rides in a second UV channel (`TEXCOORD_1`, lib/ink.py).
"""

import math
import random

import bmesh
import bpy
from mathutils import Vector, noise

COLOR_ATTRIBUTE = "Color"

# The surface a material stands for, which the normal review sheets shade by
# (`material(..., pbr=)`).
SURFACE_PROPERTY = "_pina_pbr_kind"


def reset_scene():
    """Starts from a completely empty file so builds never depend on state."""

    bpy.ops.wm.read_factory_settings(use_empty=True)

    return bpy.context.scene


def empty(name, parent=None, location=(0, 0, 0)):
    """Adds a transform-only node, used for groups, pivots and sockets."""

    node = bpy.data.objects.new(name, None)
    node.location = location
    bpy.context.scene.collection.objects.link(node)

    if parent is not None:
        node.parent = parent

    return node


def material(name, color=(0.5, 0.5, 0.5), roughness=0.85, metallic=0.0, emission=None, strength=0.0, pbr=None):
    """Returns the shared material called `name`, creating it on first use.

    `pbr` names the surface a husk's material stands for (`fur`, `machined`,
    `stone`...); the normal review sheets shade by it. It makes no texture:
    the models carry none, because Second Ink draws every material in flat
    bands (docs/art-bible.md, 5.6).
    """

    existing = bpy.data.materials.get(name)

    if existing is not None:
        return existing

    result = bpy.data.materials.new(name)
    result.use_nodes = True

    shader = result.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = (*color, 1)
    shader.inputs["Roughness"].default_value = roughness
    shader.inputs["Metallic"].default_value = metallic

    if emission is not None:
        shader.inputs["Emission Color"].default_value = (*emission, 1)
        shader.inputs["Emission Strength"].default_value = strength

    if pbr is not None:
        result[SURFACE_PROPERTY] = pbr

    return result


def object_from_bmesh(name, mesh, mat, parent=None, smooth=True):
    """Turns a bmesh into a linked, shaded object and frees the bmesh."""

    bmesh.ops.recalc_face_normals(mesh, faces=mesh.faces)

    data = bpy.data.meshes.new(name)
    mesh.to_mesh(data)
    mesh.free()

    result = bpy.data.objects.new(name, data)
    bpy.context.scene.collection.objects.link(result)
    data.materials.append(mat)

    for polygon in data.polygons:
        polygon.use_smooth = smooth

    if parent is not None:
        result.parent = parent

    return result


def activate(obj):
    """Makes `obj` the only selected, active object, as operators expect."""

    for other in bpy.context.scene.objects:
        other.select_set(False)

    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj


def apply_modifier(obj, kind, **settings):
    """Adds a modifier, sets it up and applies it straight away."""

    modifier = obj.modifiers.new(kind.title(), kind)

    for key, value in settings.items():
        setattr(modifier, key, value)

    activate(obj)
    bpy.ops.object.modifier_apply(modifier=modifier.name)

    return obj


def join(objects, name):
    """Joins `objects` into one mesh named `name`."""

    activate(objects[0])

    for obj in objects:
        obj.select_set(True)

    bpy.ops.object.join()

    result = bpy.context.view_layer.objects.active
    result.name = name
    result.data.name = name

    return result


def shade_smooth(obj, angle=None):
    """Smooth shading, optionally keeping edges sharper than `angle` crisp."""

    for polygon in obj.data.polygons:
        polygon.use_smooth = True

    if angle is None:
        return obj

    # This marks the sharp edges on the mesh itself. The "auto smooth"
    # operator would add a modifier instead, and modifiers are not applied
    # on export (applying them would destroy shape keys), so the hard edges
    # would show in a preview and vanish in the game.
    activate(obj)
    bpy.ops.object.shade_smooth_by_angle(angle=math.radians(angle))

    return obj


# Building blocks -------------------------------------------------------


def loft(name, rings, mat, parent=None, closed=True, cap_start=False, cap_end=False):
    """Skins a surface over `rings`, a list of equal-length point loops.

    With `closed`, each ring wraps around on itself (a tube). Otherwise each
    ring is an open strip (a sheet).
    """

    mesh = bmesh.new()
    rows = [[mesh.verts.new(point) for point in ring] for ring in rings]
    count = len(rows[0])
    span = count if closed else count - 1

    for upper, lower in zip(rows, rows[1:]):
        for index in range(span):
            follow = (index + 1) % count
            mesh.faces.new((upper[index], lower[index], lower[follow], upper[follow]))

    if cap_start:
        mesh.faces.new(rows[0])

    if cap_end:
        mesh.faces.new(list(reversed(rows[-1])))

    return object_from_bmesh(name, mesh, mat, parent)


def revolve(name, profile, mat, parent=None, segments=48, shape=None, cap_top=False, cap_bottom=False):
    """Spins `profile`, a list of (radius, height) pairs, around the Z axis.

    `shape(angle, radius, height)` may return an adjusted (radius, height) to
    break the symmetry: a drooping brim, a dented bell, a ragged edge.
    """

    rings = []

    for radius, height in profile:
        ring = []

        for segment in range(segments):
            angle = 2 * math.pi * segment / segments
            r, z = (radius, height) if shape is None else shape(angle, radius, height)
            ring.append((math.cos(angle) * r, math.sin(angle) * r, z))

        rings.append(ring)

    return loft(name, rings, mat, parent, cap_start=cap_top, cap_end=cap_bottom)


def tube(name, path, radius, mat, parent=None, sides=12, cap=True, twist=0.0, flatten=1.0):
    """Sweeps a circle along `path`.

    `radius` is a number or a function of the fraction along the path.
    `flatten` below 1 squashes the circle into a band: for a level loop, the
    band lies flat against whatever it is wound around.
    """

    points = [Vector(point) for point in path]
    rings = []
    normal = None

    for index, point in enumerate(points):
        ahead = points[min(index + 1, len(points) - 1)]
        behind = points[max(index - 1, 0)]
        tangent = (ahead - behind).normalized()

        # Carry the frame forward from the last ring so the tube never twists
        # on itself where the path bends.
        if normal is None:
            helper = Vector((0, 0, 1)) if abs(tangent.z) < 0.9 else Vector((1, 0, 0))
            normal = tangent.cross(helper).normalized()
        else:
            normal = (normal - tangent * normal.dot(tangent)).normalized()

        binormal = tangent.cross(normal)
        fraction = index / max(len(points) - 1, 1)
        size = radius(fraction) if callable(radius) else radius
        ring = []

        for side in range(sides):
            angle = 2 * math.pi * side / sides + twist * fraction
            ring.append(point + (normal * math.cos(angle) * flatten + binormal * math.sin(angle)) * size)

        rings.append(ring)

    return loft(name, rings, mat, parent, cap_start=cap, cap_end=cap)


def bezier(points, samples=24):
    """Samples a smooth curve through `points` (Catmull-Rom)."""

    knots = [Vector(point) for point in points]
    knots = [knots[0]] + knots + [knots[-1]]
    result = []

    for index in range(1, len(knots) - 2):
        p0, p1, p2, p3 = knots[index - 1 : index + 3]

        for sample in range(samples):
            t = sample / samples
            result.append(
                0.5
                * (
                    2 * p1
                    + (p2 - p0) * t
                    + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t * t
                    + (3 * p1 - p0 - 3 * p2 + p3) * t * t * t
                )
            )

    result.append(knots[-2])

    return result


def plate(name, outline, thickness, mat, parent=None, bevel=0.0):
    """Extrudes a flat outline in the XZ plane into a plate `thickness` deep.

    Used for blades, pans and anything cut from sheet. `outline` is a list of
    (x, z) pairs running once around the shape.
    """

    mesh = bmesh.new()
    front = [mesh.verts.new((x, -thickness / 2, z)) for x, z in outline]
    back = [mesh.verts.new((x, thickness / 2, z)) for x, z in outline]
    count = len(outline)

    mesh.faces.new(front)
    mesh.faces.new(list(reversed(back)))

    for index in range(count):
        follow = (index + 1) % count
        mesh.faces.new((front[index], back[index], back[follow], front[follow]))

    result = object_from_bmesh(name, mesh, mat, parent, smooth=False)

    if bevel > 0:
        apply_modifier(result, "BEVEL", width=bevel, segments=2, limit_method="ANGLE")

    return result


def sculpt(name, blobs, mat, parent=None, voxel=0.012, smooth=2):
    """Fuses overlapping primitives into one organic form.

    `blobs` is a list of (kind, location, scale, rotation) tuples where kind
    is "sphere" or "cube". The parts are joined and voxel-remeshed, which is
    how skulls, masks and knuckles get their soft unions.
    """

    parts = []

    for kind, location, scale, *rest in blobs:
        rotation = rest[0] if rest else (0, 0, 0)

        if kind == "sphere":
            bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=3, radius=1, location=location)
        else:
            bpy.ops.mesh.primitive_cube_add(size=2, location=location)

        part = bpy.context.object
        part.scale = scale
        part.rotation_euler = rotation
        parts.append(part)

    result = join(parts, name)
    result.data.materials.append(mat)

    activate(result)
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)

    result.data.remesh_voxel_size = voxel
    bpy.ops.object.voxel_remesh()

    if smooth > 0:
        apply_modifier(result, "SMOOTH", factor=0.6, iterations=smooth)

    if parent is not None:
        result.parent = parent

    return shade_smooth(result)


def carve(target, cutter):
    """Cuts `cutter` out of `target` and removes the cutter."""

    modifier = target.modifiers.new("Carve", "BOOLEAN")
    modifier.operation = "DIFFERENCE"
    modifier.object = cutter
    modifier.solver = "EXACT"

    activate(target)
    bpy.ops.object.modifier_apply(modifier=modifier.name)
    bpy.data.objects.remove(cutter, do_unlink=True)

    return target


def roughen(obj, strength, scale, seed=0.0):
    """Pushes every vertex along its normal by smooth noise."""

    offset = Vector((seed * 13.1, seed * 7.7, seed * 3.3))

    for vertex in obj.data.vertices:
        amount = noise.noise((vertex.co + offset) * scale)
        vertex.co += vertex.normal * amount * strength

    return obj


def decimate(obj, ratio):
    """Reduces the triangle count to `ratio` of what it was."""

    return apply_modifier(obj, "DECIMATE", ratio=ratio)


# Colour masks ----------------------------------------------------------


def paint_masks(obj, char=None, hollow=None, wear_boost=1.0, ao_samples=24, seed=0.0, rest_position=True):
    """Writes the four mask channels the game's material reads.

    `char(co)` and `hollow(co)` map a vertex position to 0..1. Without them,
    both default to a height gradient broken up by noise, so that outfits
    burn and hollow from the hem upward.

    A mesh that carries an image keeps its own texture coordinates: pass
    `rest_position=False` so the UV channels are left alone.
    """

    mesh = obj.data
    occlusion = _vertex_occlusion(obj, ao_samples)
    wear = _vertex_convexity(obj)
    attribute = mesh.color_attributes.get(COLOR_ATTRIBUTE)

    if attribute is None:
        attribute = mesh.color_attributes.new(COLOR_ATTRIBUTE, "FLOAT_COLOR", "POINT")

    mesh.color_attributes.active_color = attribute
    mesh.color_attributes.render_color_index = mesh.color_attributes.find(COLOR_ATTRIBUTE)

    heights = [(obj.matrix_world @ vertex.co).z for vertex in mesh.vertices]
    low, high = min(heights), max(heights)
    span = max(high - low, 1e-6)
    offset = Vector((seed * 5.3, seed * 9.1, seed * 2.7))

    for index, vertex in enumerate(mesh.vertices):
        world = obj.matrix_world @ vertex.co
        height = (world.z - low) / span
        grain = 0.5 + 0.5 * noise.noise((world + offset) * 6.0)
        coarse = 0.5 + 0.5 * noise.noise((world + offset) * 1.7)

        burn = char(world) if char else _clamp(1.0 - height * 1.15 + (grain - 0.5) * 0.45)
        keep = hollow(world) if hollow else _clamp(height * 0.75 + coarse * 0.45 - 0.08)
        grime = _clamp(occlusion[index] * (0.82 + 0.18 * grain))

        attribute.data[index].color = (
            grime,
            burn,
            _clamp(wear[index] * wear_boost),
            keep,
        )

    if rest_position:
        _write_rest_position(obj)

    return obj


def _write_rest_position(obj, scale=3.5):
    box_uvs(obj, scale)


def box_uvs(obj, scale=3.5):
    """Writes `obj`'s only UV channel, `UVMap` (`TEXCOORD_0`): each face's
    vertices projected along the axis it faces most, at `scale` per metre,
    in object space. Replaces any UV channel the mesh had."""

    mesh = obj.data

    for layer in list(mesh.uv_layers):
        mesh.uv_layers.remove(layer)

    corners = mesh.uv_layers.new(name="UVMap").data
    vertices = mesh.vertices

    for polygon in mesh.polygons:
        x, y, z = (abs(component) for component in polygon.normal)
        # The two axes the face is projected onto.
        first, second = (0, 1) if z >= x and z >= y else (0, 2) if y >= x else (1, 2)

        for loop in polygon.loop_indices:
            position = vertices[mesh.loops[loop].vertex_index].co
            corners[loop].uv = (position[first] * scale, position[second] * scale)


# What a class or weapon module calls on each finished part.
finish = paint_masks


def _vertex_occlusion(obj, samples):
    """Ambient occlusion per vertex, by casting rays against the mesh itself."""

    from mathutils.bvhtree import BVHTree

    mesh = obj.data
    tree = BVHTree.FromPolygons(
        [vertex.co for vertex in mesh.vertices],
        [polygon.vertices for polygon in mesh.polygons],
    )
    generator = random.Random(7)
    directions = []

    while len(directions) < samples:
        direction = Vector((generator.uniform(-1, 1), generator.uniform(-1, 1), generator.uniform(-1, 1)))

        if 0.05 < direction.length <= 1:
            directions.append(direction.normalized())

    result = []

    for vertex in mesh.vertices:
        origin = vertex.co + vertex.normal * 0.004
        blocked = 0
        counted = 0

        for direction in directions:
            if direction.dot(vertex.normal) <= 0.05:
                continue

            counted += 1

            if tree.ray_cast(origin, direction, 0.45)[0] is not None:
                blocked += 1

        result.append(1.0 - blocked / max(counted, 1))

    return result


def _vertex_convexity(obj):
    """How much each vertex stands proud of its neighbours, from 0 to 1."""

    mesh = bmesh.new()
    mesh.from_mesh(obj.data)
    mesh.verts.ensure_lookup_table()
    result = []

    for vertex in mesh.verts:
        if not vertex.link_edges:
            result.append(0.0)
            continue

        centre = Vector()

        for edge in vertex.link_edges:
            centre += edge.other_vert(vertex).co

        centre /= len(vertex.link_edges)

        length = sum(edge.calc_length() for edge in vertex.link_edges) / len(vertex.link_edges)
        proud = (vertex.co - centre).dot(vertex.normal) / max(length, 1e-6)

        result.append(_clamp(proud * 2.2))

    mesh.free()

    return result


def _clamp(value):
    return max(0.0, min(1.0, value))


# Shape keys ------------------------------------------------------------


def add_shape_key(obj, name, move):
    """Adds a shape key where `move(co)` returns each vertex's new position."""

    if obj.data.shape_keys is None:
        obj.shape_key_add(name="Basis")

    key = obj.shape_key_add(name=name)

    for index, vertex in enumerate(obj.data.vertices):
        key.data[index].co = move(vertex.co.copy())

    # Blender starts a new key fully on. A model is exported, and reviewed,
    # at rest; the game turns the keys up itself.
    key.value = 0.0

    return key


# Export ----------------------------------------------------------------


def export_glb(path, root, animations=False):
    """Exports `root` and everything under it as one binary glTF file, with
    each vertex's ink weight in `ink.CARRIER` (lib/ink.py: the one place
    that decides how the weight reaches the game)."""

    from . import ink

    carried = ink.carry(root)

    for obj in bpy.context.scene.objects:
        obj.select_set(False)

    def select(node):
        node.select_set(True)

        for child in node.children:
            select(child)

    select(root)
    bpy.context.view_layer.objects.active = root

    bpy.ops.export_scene.gltf(
        filepath=path,
        export_format="GLB",
        use_selection=True,
        export_yup=True,
        export_apply=False,
        export_morph=True,
        export_morph_normal=True,
        export_vertex_color="ACTIVE",
        export_all_vertex_colors=False,
        export_active_vertex_color_when_no_material=True,
        export_animations=animations,
        export_animation_mode="NLA_TRACKS",
        export_nla_strips=True,
        export_anim_slide_to_zero=True,
        export_force_sampling=True,
        export_optimize_animation_size=False,
        export_optimize_animation_keep_anim_object=True,
        export_materials="EXPORT",
        **carried,
    )


def triangle_count(root):
    """Triangles under `root`, for keeping models inside their budget."""

    total = 0

    def visit(node):
        nonlocal total

        if node.type == "MESH":
            total += sum(len(polygon.vertices) - 2 for polygon in node.data.polygons)

        for child in node.children:
            visit(child)

    visit(root)

    return total


def blade(name, spine, edge, thickness, mat, parent=None):
    """A forged 3D blade between two curves in the XZ plane.

    `spine` and `edge` are equal-length lists of (x, z) points running from
    the tang to the tip. Each station has a raised forged spine rib, hollow-
    ground cheeks, and a sharp bevelled edge, tapering cleanly toward the tip.
    """

    rings = []
    count = len(spine)

    for index, ((sx, sz), (ex, ez)) in enumerate(zip(spine, edge)):
        t = index / max(count - 1, 1)
        # Distal taper: strong at the heel/tang, keen toward the point.
        taper = 1.0 - 0.72 * (t**1.4)
        half = max(thickness * 0.5 * taper, 0.0008)

        def lerp(frac):
            return sx + (ex - sx) * frac, sz + (ez - sz) * frac

        r_x, r_z = lerp(0.14)
        h_x, h_z = lerp(0.46)
        b_x, b_z = lerp(0.84)

        rings.append(
            [
                (sx, 0.0, sz),
                (r_x, -half * 1.12, r_z),
                (h_x, -half * 0.62, h_z),
                (b_x, -half * 0.28, b_z),
                (ex, 0.0, ez),
                (b_x, half * 0.28, b_z),
                (h_x, half * 0.62, h_z),
                (r_x, half * 1.12, r_z),
            ]
        )

    result = loft(name, rings, mat, parent, closed=True, cap_start=True, cap_end=True)
    return shade_smooth(result, angle=32)
