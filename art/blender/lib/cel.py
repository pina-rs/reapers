"""The cel look a review sheet can be rendered in, so a model can be judged
in the game's target style before the game's renderer draws it.

Two things make the look, both read from `art/blender/cel.toml`:

- Flat colour bands. Every surface is drawn unlit, in its flat colour, and
  stepped into a few bands by how much it faces one key light. The darkest
  band is the colour multiplied by a shade tint, so shadows lean cool rather
  than grey. The key light is fixed to the camera, so every view of a sheet
  is lit the same way.
- An ink outline, as an inverted hull: a copy of each mesh pushed out along
  its normals, turned inside out and drawn in ink with its front faces
  culled, so only the rim that peeks out behind the model shows. The copy's
  vertices are first welded by position, so the normals it is pushed along
  are smooth across the model's hard edges and the hull never tears open
  where the shading normals split.

Only the review sheets use this. It changes no model: the ink is a modifier
added for the render and removed after it.
"""

import os
import tomllib

import bpy
from mathutils import Vector

from .core import COLOR_ATTRIBUTE

# The palette, beside the drivers.
PALETTE = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "cel.toml")

# Materials that show an NFT's image in the game. A review shows Blender's
# numbered colour grid on them instead, which makes stretched, flipped or
# misplaced texture coordinates obvious.
IMAGE_MATERIALS = ("canvas", "paper", "shard", "ribbon")

# Materials that are pure light. They are drawn flat in the accent colour,
# with no bands and no ink.
GLOWING = ("accent", "glow")

# Materials drawn as one flat colour with no bands: they are darkness, not
# surfaces.
FLAT = ("void", "dark", "shadow")

# The first-person effects' unlit materials (`viewmodel_parts/effects.py`):
# a soft spot the game tints and adds to what is behind, a flat light it
# tints, and a soft dark disc it blends behind a glow.
EFFECT_GLOW = "fx_glow"
EFFECT_LIGHT = "fx_light"
EFFECT_SHADE = "fx_shade"
EFFECTS = (EFFECT_GLOW, EFFECT_LIGHT, EFFECT_SHADE)

# Materials that never get an outline: light, a floor shadow, and pictures.
UNINKED = (*GLOWING, *EFFECTS, "shadow", *IMAGE_MATERIALS)

# Objects and materials the review itself adds, which are never restyled.
REVIEW_PREFIX = "review_"

INK_MODIFIER = "review_ink"
INK_GROUP = "review_ink_hull"
INK_MATERIAL = "review_ink"
KEY_NODE = "cel_key"


def linear(hex_colour):
    """An sRGB `#rrggbb` colour as the linear RGB Blender works in."""

    value = hex_colour.lstrip("#")

    if len(value) != 6:
        raise ValueError(f"{hex_colour!r} is not a #rrggbb colour")

    def channel(byte):
        byte /= 255

        return byte / 12.92 if byte <= 0.04045 else ((byte + 0.055) / 1.055) ** 2.4

    return tuple(channel(int(value[at : at + 2], 16)) for at in (0, 2, 4))


def load(path=PALETTE):
    """The palette, with its colours made linear and its numbers checked."""

    with open(path, "rb") as source:
        raw = tomllib.load(source)

    bands = int(raw["bands"]["count"])

    if bands < 2:
        raise ValueError(f"{path}: [bands] count must be at least 2, not {bands}")

    width = float(raw["ink"]["width"])

    if width < 0:
        raise ValueError(f"{path}: [ink] width cannot be negative")

    key = Vector(raw["bands"]["key"])

    if key.length == 0:
        raise ValueError(f"{path}: [bands] key must point somewhere")

    return {
        "ink": linear(raw["ink"]["colour"]),
        "width": width,
        "wobble": float(raw["ink"]["wobble"]),
        "wobble_scale": float(raw["ink"]["wobble_scale"]),
        "bands": bands,
        "shade": linear(raw["bands"]["shade_tint"]),
        "key": key.normalized(),
        "paper": linear(raw["paper"]["colour"]),
        "surfaces": {name: linear(colour) for name, colour in raw["surfaces"].items()},
    }


def base_name(mat):
    """A material's name without the `.001` Blender adds to a duplicate."""

    return mat.name.split(".")[0]


def sources(mat):
    """What a material was authored with, recorded on it the first time a
    review restyles it, since restyling throws its nodes away: its metallic
    and roughness, and the images feeding its base and emission colours."""

    if "_review_captured" in mat:
        return mat

    tree = mat.node_tree if mat.use_nodes else None
    shader = None if tree is None else next((node for node in tree.nodes if node.type == "BSDF_PRINCIPLED"), None)

    def image(socket):
        if shader is None or not shader.inputs[socket].is_linked:
            return ""

        node = shader.inputs[socket].links[0].from_node

        # Through any mixing on the way, to the image itself.
        while node.type != "TEX_IMAGE" and node.inputs and any(i.is_linked for i in node.inputs):
            node = next(i for i in node.inputs if i.is_linked).links[0].from_node

        return node.image.name if node.type == "TEX_IMAGE" and node.image else ""

    if "_pina_authored_metallic" not in mat:
        mat["_pina_authored_metallic"] = float(shader.inputs["Metallic"].default_value) if shader else 0.0
        mat["_pina_authored_roughness"] = float(shader.inputs["Roughness"].default_value) if shader else 0.86

    mat["_review_base_image"] = image("Base Color")
    mat["_review_glow_image"] = image("Emission Color")
    mat["_review_captured"] = True

    return mat


def palette_image(mat):
    """The palette texture a material takes its colours from, or `None`.

    The first-person parts are painted from one palette texture
    (`viewmodel_parts/common.py`) and carry no masks, so a review shows that
    texture rather than a colour by material name.
    """

    from viewmodel_parts.common import PALETTE_MATERIAL

    if base_name(mat) != PALETTE_MATERIAL:
        return None

    return bpy.data.images.get(sources(mat)["_review_base_image"])


def glow_image(mat):
    return bpy.data.images.get(sources(mat)["_review_glow_image"])


def masked_materials():
    """The materials every mesh drawn with carries the mask attribute, which
    the normal review multiplies into the colour."""

    masked = {}

    for obj in bpy.data.objects:
        if obj.type != "MESH":
            continue

        has = COLOR_ATTRIBUTE in obj.data.color_attributes

        for slot in obj.material_slots:
            if slot.material is not None:
                masked[slot.material.name] = masked.get(slot.material.name, True) and has

    return masked


def grid_image():
    grid = bpy.data.images.get("PreviewGrid")

    if grid is None:
        grid = bpy.data.images.new("PreviewGrid", 512, 512)
        grid.generated_type = "COLOR_GRID"

    return grid


# Materials -----------------------------------------------------------------


def style(palette, look_colours=None, accent=None):
    """Redraws every material flat, in bands of its colour.

    `look_colours` maps `body`, `trim` and `detail` to the colours an NFT's
    look dresses a creature in, and `accent` is its eyes' colour.
    """

    look_colours = look_colours or {}
    accent = accent or palette["surfaces"]["accent"]

    for mat in bpy.data.materials:
        if mat.name.startswith(REVIEW_PREFIX):
            continue

        sources(mat)
        palette_texture = palette_image(mat)
        glow_texture = glow_image(mat)
        mat.use_nodes = True
        tree = mat.node_tree
        tree.nodes.clear()
        output = tree.nodes.new("ShaderNodeOutputMaterial")
        emission = tree.nodes.new("ShaderNodeEmission")
        tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])
        name = base_name(mat)

        if effect(tree, output, mat, palette["surfaces"]["glow"]):
            continue

        if name.startswith(GLOWING):
            emission.inputs["Color"].default_value = (*accent, 1)
            continue

        if name in FLAT:
            emission.inputs["Color"].default_value = (*palette["surfaces"].get(name, palette["ink"]), 1)
            continue

        if palette_texture is not None or name.startswith(IMAGE_MATERIALS):
            texture = tree.nodes.new("ShaderNodeTexImage")
            texture.image = palette_texture or grid_image()
            texture.interpolation = "Closest" if palette_texture else "Linear"
            base = texture.outputs["Color"]
        else:
            colour = look_colours.get(name) or palette["surfaces"].get(name) or palette["surfaces"]["fallback"]
            rgb = tree.nodes.new("ShaderNodeRGB")
            rgb.outputs["Color"].default_value = (*colour, 1)
            base = rgb.outputs["Color"]

        banded = _bands(tree, base, palette)

        if glow_texture is not None:
            # A palette's emissive half: soul light shows through the bands.
            glow = tree.nodes.new("ShaderNodeTexImage")
            glow.image = glow_texture
            glow.interpolation = "Closest"
            add = tree.nodes.new("ShaderNodeMix")
            add.data_type = "RGBA"
            add.blend_type = "ADD"
            add.inputs["Factor"].default_value = 1.0
            tree.links.new(banded, add.inputs["A"])
            tree.links.new(glow.outputs["Color"], add.inputs["B"])
            banded = add.outputs["Result"]

        tree.links.new(banded, emission.inputs["Color"])


def effect(tree, output, mat, glow, strength=1.0):
    """Draws an effect's material as the game does, tinted `glow`, into a
    cleared `tree`, and says whether `mat` is one.

    The glow is the soft spot of its texture coordinates, added to what is
    behind; the shade is the same spot as darkness, blended over it; the
    light is flat. Unlit, so the same in both looks."""

    name = base_name(mat)

    if name not in EFFECTS:
        return False

    nodes, links = tree.nodes, tree.links
    emission = nodes.new("ShaderNodeEmission")
    emission.inputs["Strength"].default_value = strength

    if name == EFFECT_LIGHT:
        emission.inputs["Color"].default_value = (*glow, 1)
        links.new(emission.outputs["Emission"], output.inputs["Surface"])

        return True

    spot = _spot(tree)
    clear = nodes.new("ShaderNodeBsdfTransparent")
    mat.surface_render_method = "BLENDED"

    if name == EFFECT_GLOW:
        tint = nodes.new("ShaderNodeMix")
        tint.data_type = "RGBA"
        tint.inputs["A"].default_value = (0, 0, 0, 1)
        tint.inputs["B"].default_value = (*glow, 1)
        links.new(spot, tint.inputs["Factor"])
        links.new(tint.outputs["Result"], emission.inputs["Color"])
        add = nodes.new("ShaderNodeAddShader")
        links.new(clear.outputs["BSDF"], add.inputs[0])
        links.new(emission.outputs["Emission"], add.inputs[1])
        links.new(add.outputs["Shader"], output.inputs["Surface"])
    else:
        emission.inputs["Color"].default_value = (0, 0, 0, 1)
        mix = nodes.new("ShaderNodeMixShader")
        links.new(spot, mix.inputs["Fac"])
        links.new(clear.outputs["BSDF"], mix.inputs[1])
        links.new(emission.outputs["Emission"], mix.inputs[2])
        links.new(mix.outputs["Shader"], output.inputs["Surface"])

    return True


def _spot(tree):
    """The effects' soft spot from the texture coordinates: 1 in the middle,
    0 at the edge, with the falloff `viewmodel_parts/effects.py` bakes."""

    nodes, links = tree.nodes, tree.links
    coordinates = nodes.new("ShaderNodeTexCoord")
    centred = nodes.new("ShaderNodeVectorMath")
    centred.operation = "MULTIPLY_ADD"
    centred.inputs[1].default_value = (2, 2, 0)
    centred.inputs[2].default_value = (-1, -1, 0)
    links.new(coordinates.outputs["UV"], centred.inputs[0])
    reach = nodes.new("ShaderNodeVectorMath")
    reach.operation = "LENGTH"
    links.new(centred.outputs["Vector"], reach.inputs[0])

    def math(operation, first, second, clamp=False):
        node = nodes.new("ShaderNodeMath")
        node.operation = operation
        node.use_clamp = clamp

        for socket, value in zip(node.inputs, (first, second)):
            if isinstance(value, float):
                socket.default_value = value
            else:
                links.new(value, socket)

        return node.outputs["Value"]

    falloff = math("SUBTRACT", 1.0, reach.outputs["Value"], clamp=True)
    soft = math("MULTIPLY", math("POWER", falloff, 2.2), 0.75)
    hot = math("MULTIPLY", math("POWER", falloff, 8.0), 0.25)

    return math("ADD", soft, hot)


def _bands(tree, base, palette):
    """`base` stepped into the palette's bands by how much the surface faces
    the key light. Returns the output socket."""

    def math(operation, first, second=None, clamp=False):
        node = tree.nodes.new("ShaderNodeMath")
        node.operation = operation
        node.use_clamp = clamp

        for index, value in enumerate((first, second)):
            if value is None:
                continue
            if isinstance(value, (int, float)):
                node.inputs[index].default_value = value
            else:
                tree.links.new(value, node.inputs[index])

        return node.outputs["Value"]

    geometry = tree.nodes.new("ShaderNodeNewGeometry")
    key = tree.nodes.new("ShaderNodeCombineXYZ")
    key.name = KEY_NODE
    key.label = KEY_NODE
    facing = tree.nodes.new("ShaderNodeVectorMath")
    facing.operation = "DOT_PRODUCT"
    tree.links.new(geometry.outputs["Normal"], facing.inputs[0])
    tree.links.new(key.outputs["Vector"], facing.inputs[1])

    count = palette["bands"]
    # Half-Lambert, so the side turned from the light still reads as form.
    light = math("MULTIPLY_ADD", facing.outputs["Value"], 0.5)
    light.node.inputs[2].default_value = 0.5
    step = math("DIVIDE", math("FLOOR", math("MULTIPLY", light, count)), count - 1, clamp=True)

    shade = tree.nodes.new("ShaderNodeMix")
    shade.data_type = "RGBA"
    shade.blend_type = "MULTIPLY"
    shade.inputs["Factor"].default_value = 1.0
    tree.links.new(base, shade.inputs["A"])
    shade.inputs["B"].default_value = (*palette["shade"], 1)

    mix = tree.nodes.new("ShaderNodeMix")
    mix.data_type = "RGBA"
    tree.links.new(step, mix.inputs["Factor"])
    tree.links.new(shade.outputs["Result"], mix.inputs["A"])
    tree.links.new(base, mix.inputs["B"])

    return mix.outputs["Result"]


def aim_key(palette, camera):
    """Turns the key light with `camera`, so every view is lit alike."""

    direction = (camera.matrix_world.to_3x3() @ palette["key"]).normalized()

    for mat in bpy.data.materials:
        if not mat.use_nodes:
            continue

        node = mat.node_tree.nodes.get(KEY_NODE)

        if node is not None:
            for axis, value in zip("XYZ", direction):
                node.inputs[axis].default_value = value


# Ink -----------------------------------------------------------------------


def _ink_material(palette):
    mat = bpy.data.materials.get(INK_MATERIAL) or bpy.data.materials.new(INK_MATERIAL)
    mat.use_nodes = True
    # The hull is inside out: culling the faces that point at the camera
    # leaves only the rim behind the model.
    mat.use_backface_culling = True
    tree = mat.node_tree
    tree.nodes.clear()
    output = tree.nodes.new("ShaderNodeOutputMaterial")
    emission = tree.nodes.new("ShaderNodeEmission")
    emission.inputs["Color"].default_value = (*palette["ink"], 1)
    tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])

    return mat


def _ink_group(palette):
    """The geometry nodes that add the hull: weld by position, push out
    along the welded (smooth) normals by a width that wobbles along the
    model, turn inside out, paint with ink, and join to the model."""

    group = bpy.data.node_groups.get(INK_GROUP)

    if group is not None:
        bpy.data.node_groups.remove(group)

    group = bpy.data.node_groups.new(INK_GROUP, "GeometryNodeTree")
    interface = group.interface
    interface.new_socket("Geometry", in_out="INPUT", socket_type="NodeSocketGeometry")
    interface.new_socket("Width", in_out="INPUT", socket_type="NodeSocketFloat")
    interface.new_socket("Ink", in_out="INPUT", socket_type="NodeSocketMaterial")
    interface.new_socket("Geometry", in_out="OUTPUT", socket_type="NodeSocketGeometry")

    nodes, links = group.nodes, group.links
    source = nodes.new("NodeGroupInput")
    result = nodes.new("NodeGroupOutput")

    weld = nodes.new("GeometryNodeMergeByDistance")
    weld.inputs["Distance"].default_value = 0.0005
    links.new(source.outputs["Geometry"], weld.inputs["Geometry"])

    # The width swings along the model, as a hand-drawn line does.
    position = nodes.new("GeometryNodeInputPosition")
    noise = nodes.new("ShaderNodeTexNoise")
    noise.inputs["Scale"].default_value = palette["wobble_scale"]
    links.new(position.outputs["Position"], noise.inputs["Vector"])
    swing = nodes.new("ShaderNodeMath")
    swing.operation = "MULTIPLY_ADD"
    links.new(noise.outputs["Fac"], swing.inputs[0])
    swing.inputs[1].default_value = 4 * palette["wobble"]
    swing.inputs[2].default_value = 1 - 2 * palette["wobble"]
    floor = nodes.new("ShaderNodeMath")
    floor.operation = "MAXIMUM"
    links.new(swing.outputs["Value"], floor.inputs[0])
    floor.inputs[1].default_value = 0.0
    width = nodes.new("ShaderNodeMath")
    width.operation = "MULTIPLY"
    links.new(floor.outputs["Value"], width.inputs[0])
    links.new(source.outputs["Width"], width.inputs[1])

    normal = nodes.new("GeometryNodeInputNormal")
    offset = nodes.new("ShaderNodeVectorMath")
    offset.operation = "SCALE"
    links.new(normal.outputs["Normal"], offset.inputs["Vector"])
    links.new(width.outputs["Value"], offset.inputs["Scale"])

    push = nodes.new("GeometryNodeSetPosition")
    links.new(weld.outputs["Geometry"], push.inputs["Geometry"])
    links.new(offset.outputs["Vector"], push.inputs["Offset"])

    flip = nodes.new("GeometryNodeFlipFaces")
    links.new(push.outputs["Geometry"], flip.inputs["Mesh"])

    paint = nodes.new("GeometryNodeSetMaterial")
    links.new(flip.outputs["Mesh"], paint.inputs["Geometry"])
    links.new(source.outputs["Ink"], paint.inputs["Material"])

    join = nodes.new("GeometryNodeJoinGeometry")
    links.new(paint.outputs["Geometry"], join.inputs["Geometry"])
    links.new(source.outputs["Geometry"], join.inputs["Geometry"])
    links.new(join.outputs["Geometry"], result.inputs["Geometry"])

    return group


def _socket(group, name):
    """The identifier a modifier keys an input of `group` by."""

    return next(item.identifier for item in group.interface.items_tree if item.in_out == "INPUT" and item.name == name)


def _inked(obj):
    if obj.type != "MESH" or obj.name.startswith(REVIEW_PREFIX):
        return False

    names = [base_name(slot.material) for slot in obj.material_slots if slot.material is not None]

    return bool(names) and not all(name.startswith(UNINKED) for name in names)


def add_ink(palette):
    """Gives every mesh that should be outlined its hull. Returns them."""

    group = _ink_group(palette)
    ink = _ink_material(palette)
    inked = []

    for obj in bpy.data.objects:
        if not _inked(obj):
            continue

        modifier = obj.modifiers.new(INK_MODIFIER, "NODES")
        modifier.node_group = group
        modifier[_socket(group, "Ink")] = ink
        inked.append(obj)

    return inked


def set_ink(world_width, shown=True):
    """Sets every hull's width, in metres, and whether it is drawn."""

    for obj in bpy.data.objects:
        modifier = obj.modifiers.get(INK_MODIFIER) if obj.type == "MESH" else None

        if modifier is None:
            continue

        scale = sum(obj.matrix_world.to_scale()) / 3
        modifier[_socket(modifier.node_group, "Width")] = world_width / max(scale, 1e-6)
        modifier.show_render = shown
        modifier.show_viewport = shown


def remove_ink():
    for obj in bpy.data.objects:
        modifier = obj.modifiers.get(INK_MODIFIER) if obj.type == "MESH" else None

        if modifier is not None:
            obj.modifiers.remove(modifier)

    group = bpy.data.node_groups.get(INK_GROUP)

    if group is not None:
        bpy.data.node_groups.remove(group)
