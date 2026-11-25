"""What the first-person parts share: the palette every one of them is
painted from, merging a node's pieces into one mesh, the shapes they are cut
from (straight sections, bones), the four-vertebra grip the pistol and the
knife share, and the six-chamber drum the pistol and the spare cylinder are.

Everything here is drawn every frame, close to the eye, so every moving node
costs one draw call: its pieces are merged into one mesh with one material,
`viewmodel`, whose colours come from a small palette texture. Each piece is
modelled with the usual material names (`bone`, `metal`, `cloth`,
`cloth_alt`, `accent`, `gold`, `void`); `paint` turns those names into
palette cells by pointing every corner of the piece at its cell's middle.
The palette has a second, emissive half, black everywhere but `accent`, so
the souls glow without a material of their own. In Second Ink each cell
prints as the family it is named for (art/blender/cel.toml).

Nothing is painted with vertex colours: the game's build strips them from
every model, because the reapers' masks would draw as colours.

Whatever the game counts or lights on a part (the pistol's six chambers, the
knife's three notches) is in the part's one mesh, with a morph target that
empties each (`spent_<n>`) and an empty at each (`<part>_<n>`) where the game
hangs its halo: a count costs no draw.
"""

import math

import bmesh
import bpy
from mathutils import Matrix, Vector

from lib import core

# Each named surface's cell in the palette, and its colour (sRGB, as the
# texture stores it). The palette is CELLS cells wide and one tall.
CELLS = 10
CELL_PIXELS = 8
SURFACES = {
    # Old bone, warm and a little green in the creases.
    "bone": (0, (0.80, 0.76, 0.66)),
    # Black iron with a cold sheen; the light finds its edges.
    "metal": (1, (0.20, 0.21, 0.24)),
    # The sleeve: a robe gone to rags, nearly black.
    "cloth": (2, (0.085, 0.075, 0.095)),
    # Soul light: the one colour that glows.
    "accent": (3, (0.62, 1.0, 0.90)),
    # The band where the reaper's light runs into the tool.
    "gold": (4, (0.86, 0.64, 0.26)),
    "wood": (5, (0.30, 0.20, 0.13)),
    "rope": (6, (0.42, 0.35, 0.25)),
    # Bone in shadow: the inside of a knuckle, a socket.
    "bone_dark": (7, (0.46, 0.42, 0.36)),
    # The torn cuff, lined in the robe's second cloth.
    "cloth_alt": (8, (0.16, 0.10, 0.09)),
    # An empty chamber, a bore, a spent notch: no light at all.
    "void": (9, (0.02, 0.018, 0.024)),
}

# Per-cell (roughness, metallic) PBR response so iron and gold gleam while
# cloth stays matte and bone reads satin — all inside the single draw call.
SURFACE_PBR = {
    "bone": (0.68, 0.0),
    "metal": (0.34, 0.95),
    "cloth": (0.95, 0.0),
    "accent": (0.28, 0.0),
    "gold": (0.26, 0.98),
    "wood": (0.78, 0.0),
    "rope": (0.92, 0.0),
    "bone_dark": (0.78, 0.0),
    "cloth_alt": (0.95, 0.0),
    "void": (1.0, 0.0),
}

# The light each cell gives off of its own, as the emissive texture stores
# it. Only soul light glows.
GLOWS = {"accent": (0.62, 1.0, 0.90)}

PALETTE_MATERIAL = "viewmodel"


def _image(name, colours, colorspace="sRGB"):
    """A CELLS-wide strip of flat cells, as a packed PNG image."""

    existing = bpy.data.images.get(name)

    if existing is not None:
        return existing

    width, height = CELLS * CELL_PIXELS, CELL_PIXELS
    image = bpy.data.images.new(name, width, height, alpha=False)
    image.colorspace_settings.name = colorspace
    pixels = []

    for _ in range(height):
        for x in range(width):
            pixels.extend((*colours[x // CELL_PIXELS], 1.0))

    image.pixels = pixels
    image.file_format = "PNG"
    image.pack()

    return image


def palette_material():
    """The one material every first-person mesh is drawn with."""

    existing = bpy.data.materials.get(PALETTE_MATERIAL)

    if existing is not None:
        return existing

    base = [(0.0, 0.0, 0.0)] * CELLS
    glow = [(0.0, 0.0, 0.0)] * CELLS
    orm = [(1.0, 0.62, 0.0)] * CELLS

    for surface, (cell, colour) in SURFACES.items():
        base[cell] = colour
        glow[cell] = GLOWS.get(surface, (0.0, 0.0, 0.0))
        rough, metal = SURFACE_PBR.get(surface, (0.62, 0.0))
        orm[cell] = (1.0, rough, metal)

    result = bpy.data.materials.new(PALETTE_MATERIAL)
    result.use_nodes = True
    nodes = result.node_tree.nodes
    links = result.node_tree.links
    shader = nodes.get("Principled BSDF")
    shader.inputs["Roughness"].default_value = 1.0
    shader.inputs["Metallic"].default_value = 1.0

    for socket, name, colours in (
        ("Base Color", "viewmodel_palette", base),
        ("Emission Color", "viewmodel_glow", glow),
    ):
        texture = nodes.new("ShaderNodeTexImage")
        texture.image = _image(name, colours, colorspace="sRGB")
        texture.interpolation = "Closest"
        links.new(texture.outputs["Color"], shader.inputs[socket])

    pbr_tex = nodes.new("ShaderNodeTexImage")
    pbr_tex.image = _image("viewmodel_pbr", orm, colorspace="Non-Color")
    pbr_tex.interpolation = "Closest"
    sep = nodes.new("ShaderNodeSeparateColor")
    links.new(pbr_tex.outputs["Color"], sep.inputs["Color"])
    links.new(sep.outputs["Green"], shader.inputs["Roughness"])
    links.new(sep.outputs["Blue"], shader.inputs["Metallic"])

    shader.inputs["Emission Strength"].default_value = 1.0

    return result


def paint(obj, surface=None):
    """Points every corner of `obj` at its surface's palette cell and draws
    it with the palette. The surface is `obj`'s own material's name unless
    given."""

    mesh = obj.data
    surface = surface or (mesh.materials[0].name if mesh.materials else "bone")
    cell = SURFACES[surface][0]
    u = (cell + 0.5) / CELLS

    for attribute in list(mesh.color_attributes):
        mesh.color_attributes.remove(attribute)

    while mesh.uv_layers:
        mesh.uv_layers.remove(mesh.uv_layers[0])

    layer = mesh.uv_layers.new(name="UVMap")

    for loop in layer.data:
        loop.uv = (u, 0.5)

    mesh.materials.clear()
    mesh.materials.append(palette_material())

    return obj


def merge(node, label):
    """Paints every mesh directly under `node` and joins them into one mesh,
    `<label>`, so the node costs one draw call. Vertex groups (`tag`) come
    through the join by name."""

    parts = [child for child in node.children if child.type == "MESH"]

    if not parts:
        return None

    for part in parts:
        paint(part)

    joined = core.join(parts, label) if len(parts) > 1 else parts[0]
    joined.name = label
    joined.data.name = label
    joined.parent = node

    return joined


# Shapes --------------------------------------------------------------------


def section(name, stations, profile, mat, parent=None, across=(1, 0, 0), up=(0, 0, 1), caps=True, smooth=False):
    """A straight-sided solid: `profile`, a loop of (u, v) points, placed at
    each of `stations`, `(centre, size_u, size_v)`, with u along `across` and
    v along `up`. Flat-shaded by default, so each side prints as one plane.
    `caps` closes both ends, or is a pair (first, last) to leave an end that
    is buried in something else open."""

    across, up = Vector(across), Vector(up)
    rings = [[Vector(centre) + across * u * su + up * v * sv for u, v in profile] for centre, su, sv in stations]
    first, last = caps if isinstance(caps, tuple) else (caps, caps)
    result = core.loft(name, rings, mat, parent, cap_start=first, cap_end=last)

    for polygon in result.data.polygons:
        polygon.use_smooth = smooth

    return result


def skin(name, rings, mat, parent=None, tip=None, smooth=False):
    """Skins `rings` (equal loops of points) into a solid, capped at the
    first ring, and closed at the last either by a cap or, given `tip`, by
    running every side in to that one point: a blade's or a claw's end."""

    mesh = bmesh.new()
    rows = [[mesh.verts.new(point) for point in ring] for ring in rings]
    count = len(rows[0])

    for lower, upper in zip(rows, rows[1:]):
        for index in range(count):
            follow = (index + 1) % count
            mesh.faces.new((lower[index], lower[follow], upper[follow], upper[index]))

    if tip is None:
        mesh.faces.new(rows[-1])
    else:
        point = mesh.verts.new(tip)

        for index in range(count):
            mesh.faces.new((rows[-1][index], rows[-1][(index + 1) % count], point))

    mesh.faces.new(list(reversed(rows[0])))

    return core.object_from_bmesh(name, mesh, mat, parent, smooth=smooth)


def polygon(sides, start=0.0, squash=1.0):
    """A regular polygon's corners on the unit circle, from angle `start`,
    its v squashed by `squash`: a profile for `section`."""

    return [(math.cos(start + math.tau * k / sides), math.sin(start + math.tau * k / sides) * squash) for k in range(sides)]


def frame(direction, hint=None):
    """Two unit vectors square to `direction` and to each other: (across,
    up), `up` as near `hint` (default +Z) as it can be."""

    direction = Vector(direction).normalized()
    hint = Vector(hint or (0, 0, 1))

    if abs(direction.dot(hint.normalized())) > 0.95:
        hint = Vector((1, 0, 0)) if abs(direction.x) < 0.9 else Vector((0, 1, 0))

    across = direction.cross(hint).normalized()

    return across, across.cross(direction).normalized()


def bone(name, start, end, head, tail, mat, parent=None, sides=6, waist=0.74, claw=False, squash=0.9, hint=None, rounded=True):
    """One bone from `start` to `end`: a joint `head` thick at the start
    and `tail` at the end, each rounded off past its joint, the shaft pinched
    to `waist` between them, so the knuckles step out of it and the ink line
    breaks at every joint. A `claw` runs out to a point instead of a joint.
    `squash` flattens it across `hint`'s side, as a finger is flatter on its
    back than it is deep. A bone whose head sits inside the joint before it
    need not round it off: `rounded=False` saves those faces."""

    start, end = Vector(start), Vector(end)
    along = end - start
    length = along.length
    unit = along / length
    across, up = frame(unit, hint)
    stations = [(-head * 0.62, head * 0.5)] if rounded else []
    stations += [(-head * 0.12, head), (length * 0.5, waist * (head + tail) / 2)]

    if claw:
        stations += [(length * 0.86, tail * 0.55), (length + tail * 0.2, tail * 0.12)]
    else:
        stations += [(length + tail * 0.12, tail), (length + tail * 0.62, tail * 0.5)]

    rings = []

    for distance, radius in stations:
        centre = start + unit * distance
        rings.append([centre + (across * math.cos(a) + up * math.sin(a) * squash) * radius for a in (math.tau * k / sides + math.pi / sides for k in range(sides))])

    return core.loft(name, rings, mat, parent, cap_start=True, cap_end=True)


def tag(obj, group):
    """Puts every vertex of `obj` in the vertex group `group`, so it can be
    found again once `obj` is merged."""

    vertices = obj.vertex_groups.new(name=group)
    vertices.add(list(range(len(obj.data.vertices))), 1.0, "REPLACE")

    return obj


def empties(obj, groups, move):
    """For each vertex group in `groups` (made by `tag`), a morph target on
    `obj` named `spent_<n>` that carries those vertices by `move(co)`, and
    then drops the groups: what is left is the mesh and its keys."""

    for index, group in enumerate(groups):
        found = obj.vertex_groups[group].index
        key = core.add_shape_key(obj, f"spent_{index}", lambda co: co)

        for vertex in obj.data.vertices:
            if any(g.group == found and g.weight > 0.5 for g in vertex.groups):
                key.data[vertex.index].co = move(vertex.co.copy())

    for group in groups:
        obj.vertex_groups.remove(obj.vertex_groups[group])

    return obj


# The grip ------------------------------------------------------------------

# A vertebra seen from above: deeper than it is wide, flat at the back where
# the palm sits, with a keel at the front for the fingers to close on.
VERTEBRA = [(-0.86, 0.82), (0.86, 0.82), (1.0, -0.1), (0.5, -1.0), (-0.5, -1.0), (-1.0, -0.1)]


def vertebrae(prefix, parent, top, bottom, half_width=0.0145, half_depth=0.0165, count=4, rake=0.0, mat=None):
    """`count` iron vertebrae stacked down a grip from `top` to `bottom`
    (heights), each a short body flared at its ends with a gap between, the
    whole stack raked back toward +Y by `rake` per metre of drop. The pistol
    and the knife share it, so the two read as one maker's."""

    mat = mat or core.material("metal")
    step = (top - bottom) / count
    parts = []
    # Across is X; "up" in the profile is the back of the grip, +Y.
    for index in range(count):
        high = top - index * step - step * 0.08
        low = high - step * 0.84
        stations = []

        for z, flare in ((high, 0.86), (high - step * 0.16, 1.0), (low + step * 0.16, 1.0), (low, 0.86)):
            stations.append((Vector((0.0, (top - z) * rake, z)), half_width * flare, half_depth * flare))

        # Smooth across its six sides: small enough that the bands round it
        # well, and a fraction of the vertices flat facets would cost.
        parts.append(section(f"{prefix}_vertebra_{index}", stations, VERTEBRA, mat, parent, across=(1, 0, 0), up=(0, 1, 0), smooth=True))

    return parts


# The drum ------------------------------------------------------------------

DRUM_RADIUS = 0.034
DRUM_LENGTH = 0.044
# How far each chamber is from the drum's axis: the top one lines up with
# the bore (grave_pistol.BORE_Z), which the game's tests hold where it is,
# so the ring is a little tighter than the bible's .02. Six chambers on a
# ring of radius r touch at radius r / 2: the sockets stay just inside that.
CHAMBER_RING = 0.0172
CHAMBERS = 6
SOCKET = 0.0080
SOUL = 0.0062


def chamber_angle(index):
    """Where chamber `index` sits round the drum, in its (x, z) plane: 0 at
    the top, under the bore, and on round clockwise as the eye sees the
    drum's back."""

    return math.pi / 2 + math.tau * index / CHAMBERS


def drum(label, parent, chambers=None):
    """A revolver's drum, on its axis along Y and centred on `parent`: a
    six-sided iron prism, chamfered front and back, whose six chambers face
    the eye from its back face, each a socket of void holding a plug of soul
    light. Merged into `<label>`.

    Given `chambers`, the drum counts: each chamber has a morph target,
    `spent_<n>`, that sinks its light into the drum and leaves the void
    socket, and an empty, `<chambers>_<n>`, at its mouth, where the game
    hangs the chamber's halo. Chamber 0 is under the bore (`chamber_angle`).
    Without it every chamber is loaded for good: the spare drum."""

    metal = core.material("metal")
    void = core.material("void")
    accent = core.material("accent")
    back = DRUM_LENGTH / 2
    hexagon = polygon(6, start=0.0)
    # The prism, a flat toward the bore, so each chamber sits behind a flat.
    section(
        f"{label}_body",
        [((0, -back, 0), DRUM_RADIUS * 0.86, DRUM_RADIUS * 0.86), ((0, -back + 0.004, 0), DRUM_RADIUS, DRUM_RADIUS), ((0, back - 0.004, 0), DRUM_RADIUS, DRUM_RADIUS), ((0, back, 0), DRUM_RADIUS * 0.88, DRUM_RADIUS * 0.88)],
        hexagon,
        metal,
        parent,
        across=(1, 0, 0),
        up=(0, 0, 1),
    )
    # The pin it turns on, standing out of its front.
    section(f"{label}_pin", [((0, -back + 0.001, 0), 0.0075, 0.0075), ((0, -back - 0.004, 0), 0.0068, 0.0068), ((0, -back - 0.006, 0), 0.004, 0.004)], polygon(6, math.pi / 6), metal, parent)
    groups = []

    for index in range(CHAMBERS):
        angle = chamber_angle(index)
        centre = Vector((math.cos(angle) * CHAMBER_RING, back, math.sin(angle) * CHAMBER_RING))
        section(
            f"{label}_socket_{index}",
            [(centre + Vector((0, -0.002, 0)), SOCKET, SOCKET), (centre + Vector((0, 0.0007, 0)), SOCKET, SOCKET)],
            polygon(8, math.pi / 8),
            void,
            parent,
            up=(0, 0, 1),
            caps=(False, True),
        )
        soul = section(
            f"{label}_soul_{index}",
            [(centre + Vector((0, -0.004, 0)), SOUL, SOUL), (centre + Vector((0, 0.0013, 0)), SOUL, SOUL), (centre + Vector((0, 0.0022, 0)), SOUL * 0.7, SOUL * 0.7)],
            polygon(8, math.pi / 8),
            accent,
            parent,
            smooth=True,
            caps=(False, True),
        )

        if chambers:
            groups.append(f"chamber_{index}")
            tag(soul, groups[-1])
            core.empty(f"{chambers}_{index}", parent, location=centre + Vector((0, 0.0022, 0)))

    joined = merge(parent, label)

    if chambers:
        # Sunk 7 mm, a spent soul is inside the drum, behind its void socket.
        empties(joined, groups, lambda co: co + Vector((0, -0.007, 0)))

    return joined


# Kept for the parts that cut a frame's flat side from an outline.
def side_plate(name, outline, thickness, mat, parent=None, bevel=0.0015):
    """A plate whose outline is drawn in the YZ plane, as (y, z) pairs, and
    which is `thickness` wide across X: the flat side of a frame."""

    result = core.plate(name, outline, thickness, mat, parent, bevel=bevel)

    # core.plate draws in XZ with its depth along Y. A quarter turn about Z
    # carries its x onto y and its depth onto x.
    result.data.transform(Matrix.Rotation(math.radians(90), 4, "Z"))

    return result
