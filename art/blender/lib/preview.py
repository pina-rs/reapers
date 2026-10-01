"""Review renders, so a model can be judged before it reaches the game.

The game draws these models with its own shader, so a preview is a check on
shape, silhouette and the painted masks rather than on final colour.
"""

import math

import bpy
from mathutils import Vector

from .core import COLOR_ATTRIBUTE

# Roughly what the game's palette does to each material slot, so that a
# preview reads the way the model will.
PREVIEW_COLORS = {
    "cloth": (0.085, 0.07, 0.06),
    "cloth_alt": (0.16, 0.11, 0.06),
    "bone": (0.62, 0.57, 0.47),
    "metal": (0.16, 0.16, 0.17),
    "wood": (0.12, 0.085, 0.06),
    "rope": (0.3, 0.24, 0.15),
    "feather": (0.03, 0.035, 0.05),
    "gold": (0.55, 0.38, 0.12),
    "paper": (0.7, 0.62, 0.48),
    "void": (0.0, 0.0, 0.0),
    "frame": (0.2, 0.14, 0.08),
    "core": (0.01, 0.01, 0.015),
    "shroud": (0.2, 0.26, 0.3),
    "body": (0.2, 0.14, 0.1),
    "trim": (0.55, 0.42, 0.3),
    "detail": (0.7, 0.66, 0.58),
    "dark": (0.004, 0.004, 0.006),
    "shadow": (0.0, 0.0, 0.0),
}
ACCENT = (0.75, 0.95, 1.0)

# Materials that show an NFT's image in the game. A preview shows Blender's
# numbered colour grid on them instead, which makes stretched, flipped or
# misplaced texture coordinates obvious.
IMAGE_MATERIALS = ("canvas", "paper", "shard", "ribbon")


def _linear(colour):
    """An 0xRRGGBB colour as the linear RGB Blender works in."""

    def channel(value):
        value /= 255

        return value / 12.92 if value <= 0.04045 else ((value + 0.055) / 1.055) ** 2.4

    return (channel(colour >> 16 & 0xFF), channel(colour >> 8 & 0xFF), channel(colour & 0xFF))


def _dress(look):
    """Shows what an NFT's look wears and hides the rest. Returns the colours
    its materials take."""

    if look is None:
        return {}, ACCENT

    for obj in bpy.data.objects:
        node = obj

        while node is not None:
            if node.name.startswith("trait_"):
                obj.hide_render = node.name != f"trait_{look['trait']}"
                break

            node = node.parent

    colours = {name: _linear(look[name]) for name in ("body", "trim", "detail")}

    return colours, _linear(look["accent"])


def _style_materials(silhouette, look=None):
    colours, accent = _dress(look)

    for mat in bpy.data.materials:
        mat.use_nodes = True
        tree = mat.node_tree
        tree.nodes.clear()

        output = tree.nodes.new("ShaderNodeOutputMaterial")

        if silhouette:
            emission = tree.nodes.new("ShaderNodeEmission")
            emission.inputs["Color"].default_value = (0, 0, 0, 1)
            tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])
            continue

        if mat.name.startswith(("accent", "glow")):
            emission = tree.nodes.new("ShaderNodeEmission")
            emission.inputs["Color"].default_value = (*accent, 1)
            emission.inputs["Strength"].default_value = 14 if look is None else 6
            tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])
            continue

        if mat.name.startswith(IMAGE_MATERIALS):
            grid = bpy.data.images.get("PreviewGrid")

            if grid is None:
                grid = bpy.data.images.new("PreviewGrid", 512, 512)
                grid.generated_type = "COLOR_GRID"

            shader = tree.nodes.new("ShaderNodeBsdfPrincipled")
            texture = tree.nodes.new("ShaderNodeTexImage")
            texture.image = grid
            shader.inputs["Roughness"].default_value = 0.7
            tree.links.new(texture.outputs["Color"], shader.inputs["Base Color"])
            tree.links.new(texture.outputs["Color"], shader.inputs["Emission Color"])
            shader.inputs["Emission Strength"].default_value = 0.35
            tree.links.new(shader.outputs["BSDF"], output.inputs["Surface"])
            continue

        name = mat.name.split(".")[0]
        base = colours.get(name, PREVIEW_COLORS.get(name, (0.3, 0.3, 0.3)))
        shader = tree.nodes.new("ShaderNodeBsdfPrincipled")
        masks = tree.nodes.new("ShaderNodeVertexColor")
        masks.layer_name = COLOR_ATTRIBUTE
        split = tree.nodes.new("ShaderNodeSeparateColor")
        tint = tree.nodes.new("ShaderNodeMix")
        tint.data_type = "RGBA"
        tint.blend_type = "MULTIPLY"
        tint.inputs["Factor"].default_value = 1.0
        tint.inputs["A"].default_value = (*base, 1)
        wear = tree.nodes.new("ShaderNodeMix")
        wear.data_type = "RGBA"
        wear.blend_type = "ADD"
        wear.inputs["B"].default_value = (base[0] * 1.6 + 0.05, base[1] * 1.6 + 0.05, base[2] * 1.6 + 0.05, 1)

        tree.links.new(masks.outputs["Color"], split.inputs["Color"])
        tree.links.new(split.outputs["Red"], tint.inputs["B"])
        tree.links.new(tint.outputs["Result"], wear.inputs["A"])
        tree.links.new(split.outputs["Blue"], wear.inputs["Factor"])
        tree.links.new(wear.outputs["Result"], shader.inputs["Base Color"])
        tree.links.new(shader.outputs["BSDF"], output.inputs["Surface"])

        metal = mat.name.startswith(("metal", "gold"))
        shader.inputs["Metallic"].default_value = 1.0 if metal else 0.0
        shader.inputs["Roughness"].default_value = 0.42 if metal else 0.86


def _light(name, kind, location, energy, color, size=1.0, target=(0, 0, 1.1)):
    data = bpy.data.lights.new(name, kind)
    data.energy = energy
    data.color = color

    if kind == "AREA":
        data.size = size

    light = bpy.data.objects.new(name, data)
    light.location = location
    bpy.context.scene.collection.objects.link(light)

    direction = Vector(target) - Vector(location)
    light.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()

    return light


def _camera(location, target, lens=70):
    data = bpy.data.cameras.new("PreviewCamera")
    data.lens = lens

    camera = bpy.data.objects.new("PreviewCamera", data)
    camera.location = location
    bpy.context.scene.collection.objects.link(camera)

    direction = Vector(target) - Vector(location)
    camera.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()
    bpy.context.scene.camera = camera

    return camera


def render_sheet(path, centre=(0, 0, 1.4), distance=7.4, width=560, height=840, views=("front", "threequarter", "back", "silhouette"), look=None):
    """Renders the scene from several angles into one side-by-side image.

    `look` dresses a creature in one NFT's colours and trait, so the sheet
    shows it as the game will.
    """

    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = width
    scene.render.resolution_y = height
    scene.render.film_transparent = False
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"

    world = bpy.data.worlds.new("PreviewWorld")
    world.use_nodes = True
    scene.world = world
    background = world.node_tree.nodes["Background"]

    lights = [
        _light("Key", "AREA", (-3.2, -4.2, 3.6), 900, (1.0, 0.94, 0.86), size=3.0, target=centre),
        _light("Rim", "AREA", (2.6, 3.8, 3.2), 1500, (0.55, 0.75, 1.0), size=2.0, target=centre),
        _light("Fill", "AREA", (3.5, -3.0, 0.6), 140, (0.6, 0.7, 0.9), size=4.0, target=centre),
    ]
    angles = {"front": 0.0, "side": 90.0, "back": 180.0, "threequarter": 32.0}
    tiles = []

    for view in views:
        silhouette = view == "silhouette"
        angle = math.radians(angles.get(view, 24.0))

        _style_materials(silhouette, look)
        background.inputs["Color"].default_value = (0.45, 0.45, 0.45, 1) if silhouette else (0.012, 0.014, 0.02, 1)
        background.inputs["Strength"].default_value = 1.0

        for light in lights:
            light.hide_render = silhouette

        eye = (
            centre[0] + math.sin(angle) * distance,
            centre[1] - math.cos(angle) * distance,
            centre[2] - 0.25,
        )
        camera = _camera(eye, centre)
        tile = f"{path}.{view}.png"
        scene.render.filepath = tile
        bpy.ops.render.render(write_still=True)
        tiles.append(tile)
        bpy.data.objects.remove(camera, do_unlink=True)

    _stitch(tiles, path, width, height)


def frame(skip=("shadow",), margin=1.12, lens=70):
    """Where a camera should look, and from how far, to fit what will be
    rendered: the middle of every visible mesh's bounds and the distance at
    which they fill the picture.

    Meshes whose names start with any of `skip` are ignored, as are hidden
    ones, so a floor shadow or an unworn hat does not push the creature into
    a corner.
    """

    low = Vector((math.inf, math.inf, math.inf))
    high = -low

    for obj in bpy.data.objects:
        if obj.type != "MESH" or obj.hide_render or obj.name.startswith(tuple(skip)):
            continue

        for corner in obj.bound_box:
            world = obj.matrix_world @ Vector(corner)
            low = Vector(map(min, low, world))
            high = Vector(map(max, high, world))

    centre = (low + high) / 2
    radius = (high - low).length / 2
    # A 36 mm sensor behind the lens sees this far either side of centre.
    half_view = math.atan(18 / lens)

    return tuple(centre), radius * margin / math.sin(half_view)


def render_portrait(path, look, size=384):
    """Renders one NFT's portrait: its creature, in its colours, on its flat
    backdrop. This is the picture the game shows for the NFT.

    The seals and the shadow are left out; they belong to the hunt.
    """

    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = size
    scene.render.resolution_y = size
    scene.render.film_transparent = False
    scene.render.image_settings.file_format = "PNG"
    scene.view_settings.view_transform = "Standard"
    scene.view_settings.look = "None"

    world = scene.world or bpy.data.worlds.new("PortraitWorld")
    world.use_nodes = True
    scene.world = world
    background = world.node_tree.nodes["Background"]
    background.inputs["Color"].default_value = (*_linear(look["backdrop"]), 1)
    background.inputs["Strength"].default_value = 1.0

    hidden = []

    for obj in bpy.data.objects:
        if obj.type == "LIGHT" or obj.name == "shadow" or obj.name.startswith("seal_"):
            hidden.append((obj, obj.hide_render))
            obj.hide_render = True

    lights = [
        _light("PortraitKey", "AREA", (-2.4, -3.4, 2.8), 210, (1.0, 0.97, 0.92), size=3.5, target=(0, 0, 0)),
        _light("PortraitFill", "AREA", (3.0, -2.6, 0.8), 70, (0.9, 0.94, 1.0), size=4.0, target=(0, 0, 0)),
        _light("PortraitRim", "AREA", (1.6, 3.2, 2.6), 150, (1.0, 1.0, 1.0), size=2.0, target=(0, 0, 0)),
    ]

    _style_materials(False, look)
    bpy.context.view_layer.update()

    centre, distance = frame(skip=("shadow", "seal_"), margin=0.94)
    angle = math.radians(28.0)
    camera = _camera(
        (
            centre[0] + math.sin(angle) * distance,
            centre[1] - math.cos(angle) * distance,
            centre[2] + distance * 0.16,
        ),
        centre,
    )

    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)

    bpy.data.objects.remove(camera, do_unlink=True)

    for light in lights:
        bpy.data.objects.remove(light, do_unlink=True)

    for obj, was in hidden:
        obj.hide_render = was


def _stitch(tiles, path, width, height):
    """Places the rendered tiles next to each other in one image."""

    import os

    import numpy

    sheet = numpy.zeros((height, width * len(tiles), 4), dtype=numpy.float32)

    for index, tile in enumerate(tiles):
        image = bpy.data.images.load(tile)
        pixels = numpy.empty(width * height * 4, dtype=numpy.float32)
        image.pixels.foreach_get(pixels)
        sheet[:, index * width : (index + 1) * width, :] = pixels.reshape(height, width, 4)
        bpy.data.images.remove(image)
        os.remove(tile)

    result = bpy.data.images.new("Sheet", width * len(tiles), height, alpha=False)
    result.pixels.foreach_set(sheet.ravel())
    result.filepath_raw = path
    result.file_format = "PNG"
    result.save()
