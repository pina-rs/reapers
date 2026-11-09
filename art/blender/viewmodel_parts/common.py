"""What the first-person parts share: the palette every one of them is
painted from, merging a node's pieces into one mesh, side plates, and the
soul drum that both the pistol and the spare cylinder are made of.

Everything here is drawn every frame, close to the eye, so every moving node
costs one draw call: its pieces are merged into one mesh with one material,
`viewmodel`, whose colours come from a small palette texture. Each piece is
modelled with the usual material names (`bone`, `metal`, `cloth`, `accent`,
`gold`, `wood`, `rope`); `paint` turns those names into palette cells by
pointing every corner of the piece at its cell's middle. The palette has a
second, emissive half, black everywhere but `accent`, so the souls glow
without a material of their own.

Nothing is painted with vertex colours: the game's build strips them from
every model, because the reapers' masks would draw as colours.
"""

import math

import bpy
from mathutils import Matrix

from lib import core

# Each named surface's cell in the palette, and its colour (sRGB, as the
# texture stores it). The palette is CELLS cells wide and one tall.
CELLS = 8
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
    "gold": (4, (0.86, 0.64, 0.26)),
    "wood": (5, (0.30, 0.20, 0.13)),
    "rope": (6, (0.42, 0.35, 0.25)),
    # Bone in shadow: the inside of a knuckle, a socket.
    "bone_dark": (7, (0.46, 0.42, 0.36)),
}

# The light each cell gives off of its own, as the emissive texture stores
# it. Only soul light glows.
GLOWS = {"accent": (0.62, 1.0, 0.90)}

PALETTE_MATERIAL = "viewmodel"


def _image(name, colours):
    """A CELLS-wide strip of flat cells, as a packed PNG image."""

    existing = bpy.data.images.get(name)

    if existing is not None:
        return existing

    width, height = CELLS * CELL_PIXELS, CELL_PIXELS
    image = bpy.data.images.new(name, width, height, alpha=False)
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

    for surface, (cell, colour) in SURFACES.items():
        base[cell] = colour
        glow[cell] = GLOWS.get(surface, (0.0, 0.0, 0.0))

    result = bpy.data.materials.new(PALETTE_MATERIAL)
    result.use_nodes = True
    nodes = result.node_tree.nodes
    links = result.node_tree.links
    shader = nodes.get("Principled BSDF")
    shader.inputs["Roughness"].default_value = 0.62
    shader.inputs["Metallic"].default_value = 0.0

    for socket, name, colours in (
        ("Base Color", "viewmodel_palette", base),
        ("Emission Color", "viewmodel_glow", glow),
    ):
        texture = nodes.new("ShaderNodeTexImage")
        texture.image = _image(name, colours)
        texture.interpolation = "Closest"
        links.new(texture.outputs["Color"], shader.inputs[socket])

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
    `<label>`, so the node costs one draw call."""

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


def side_plate(name, outline, thickness, mat, parent=None, bevel=0.0015):
    """A plate whose outline is drawn in the YZ plane, as (y, z) pairs, and
    which is `thickness` wide across X: the flat side of a frame."""

    result = core.plate(name, outline, thickness, mat, parent, bevel=bevel)

    # core.plate draws in XZ with its depth along Y. A quarter turn about Z
    # carries its x onto y and its depth onto x.
    result.data.transform(Matrix.Rotation(math.radians(90), 4, "Z"))

    return result


def soul_drum(label, parent, length=0.042, radius=0.02, souls=6):
    """A revolver's cylinder: an iron drum with a glowing soul set into each
    chamber, on its axis along -Y and centred on the origin of `parent`, and
    merged into one mesh, `<label>`."""

    metal = core.material("metal")
    accent = core.material("accent")
    half = length / 2

    drum = core.revolve(
        f"{label}_drum",
        [
            (0.0, -half),
            (radius * 0.7, -half),
            (radius * 0.94, -half + 0.002),
            (radius, -half + 0.006),
            (radius, half - 0.006),
            (radius * 0.94, half - 0.002),
            (radius * 0.72, half),
            (0.0, half),
        ],
        metal,
        parent,
        segments=18,
    )

    pin = core.revolve(
        f"{label}_pin",
        [(0.0, half + 0.006), (0.004, half + 0.005), (0.005, half), (0.005, -half), (0.0, -half)],
        metal,
        parent,
        segments=8,
    )

    vials = []

    for chamber in range(souls):
        angle = math.tau * chamber / souls + math.pi / 2
        x = math.cos(angle) * radius * 0.86
        y = math.sin(angle) * radius * 0.86
        vials.append(
            core.tube(
                f"{label}_soul_{chamber}",
                [(x, y, -half - 0.0015), (x, y, 0.0), (x, y, half + 0.0015)],
                lambda t: 0.0056 * (0.55 + 0.45 * math.sin(t * math.pi) ** 0.5),
                accent,
                parent,
                sides=6,
            )
        )

    for part in (drum, pin, *vials):
        # Built along +Z; the drum's axis runs along the barrel, toward -Y.
        part.data.transform(Matrix.Rotation(math.radians(90), 4, "X"))

    return merge(parent, label)
