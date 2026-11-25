"""Review renders, so a model can be judged before it reaches the game.

The game draws these models with its own shader, so a preview is a check on
shape, silhouette and the painted masks rather than on final colour.
"""

import math
import os

import bpy
from mathutils import Matrix, Vector

from . import cel
from .core import COLOR_ATTRIBUTE, SURFACE_PROPERTY

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
    "spirit": (0.025, 0.055, 0.07),
    "body": (0.2, 0.14, 0.1),
    "trim": (0.55, 0.42, 0.3),
    "detail": (0.7, 0.66, 0.58),
    "dark": (0.004, 0.004, 0.006),
    "shadow": (0.0, 0.0, 0.0),
}
ACCENT = (0.75, 0.95, 1.0)

# Materials that show an NFT's image in the game: see `cel.IMAGE_MATERIALS`.
IMAGE_MATERIALS = cel.IMAGE_MATERIALS


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


def _style_materials(silhouette, look=None, review=False):
    """Restyles every material for a render. Portraits use the default;
    review sheets pass `review`, which leaves the review's own materials
    alone, shows the first-person palette instead of a colour by name, and
    leaves out the masks on meshes that have none, which would otherwise
    multiply them black."""

    colours, accent = _dress(look)
    masked = cel.masked_materials() if review else {}

    for mat in bpy.data.materials:
        if review:
            if mat.name.startswith(cel.REVIEW_PREFIX):
                continue

            cel.sources(mat)

        mat.use_nodes = True
        tree = mat.node_tree
        if "_pina_authored_metallic" not in mat:
            prev_bsdf = tree.nodes.get("Principled BSDF")
            if prev_bsdf is not None:
                mat["_pina_authored_metallic"] = float(prev_bsdf.inputs["Metallic"].default_value)
                mat["_pina_authored_roughness"] = float(prev_bsdf.inputs["Roughness"].default_value)
            else:
                mat["_pina_authored_metallic"] = 0.0
                mat["_pina_authored_roughness"] = 0.86
        authored_metallic = float(mat["_pina_authored_metallic"])
        authored_roughness = float(mat["_pina_authored_roughness"])
        tree.nodes.clear()

        output = tree.nodes.new("ShaderNodeOutputMaterial")

        if silhouette:
            emission = tree.nodes.new("ShaderNodeEmission")
            emission.inputs["Color"].default_value = (0, 0, 0, 1)
            tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])
            continue

        if review and cel.effect(tree, output, mat, ACCENT, strength=2.0):
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

        if review and cel.palette_image(mat) is not None:
            _style_palette(tree, output, mat)
            continue

        name = mat.name.split(".")[0]
        pbr_kind = mat.get(SURFACE_PROPERTY)
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
        wear.inputs["B"].default_value = (base[0] * 1.45 + 0.04, base[1] * 1.45 + 0.04, base[2] * 1.45 + 0.04, 1)

        if not review or masked.get(mat.name, True):
            tree.links.new(masks.outputs["Color"], split.inputs["Color"])
        else:
            # A mesh without masks would read them as black: show its
            # colour clean and unworn instead.
            split.inputs["Color"].default_value = (1, 1, 0, 1)
        tree.links.new(split.outputs["Red"], tint.inputs["B"])
        tree.links.new(tint.outputs["Result"], wear.inputs["A"])
        tree.links.new(split.outputs["Blue"], wear.inputs["Factor"])
        tree.links.new(wear.outputs["Result"], shader.inputs["Base Color"])
        tree.links.new(shader.outputs["BSDF"], output.inputs["Surface"])

        metal = mat.name.startswith(("metal", "gold"))

        if metal or pbr_kind in ("gold", "metal"):
            shader.inputs["Metallic"].default_value = max(authored_metallic, 0.90)
            shader.inputs["Roughness"].default_value = min(authored_roughness, 0.34)
        elif pbr_kind in ("machined", "stone"):
            shader.inputs["Metallic"].default_value = max(authored_metallic, 0.65)
            shader.inputs["Roughness"].default_value = min(authored_roughness, 0.38)
        elif name == "dark":
            shader.inputs["Metallic"].default_value = authored_metallic
            shader.inputs["Roughness"].default_value = min(authored_roughness, 0.25)
        else:
            shader.inputs["Metallic"].default_value = authored_metallic
            shader.inputs["Roughness"].default_value = 0.72 if pbr_kind == "plumage" else min(authored_roughness, 0.84)


def _style_palette(tree, output, mat):
    """The first-person parts: their palette texture, lit, with its emissive
    half glowing (`viewmodel_parts/common.py`)."""

    shader = tree.nodes.new("ShaderNodeBsdfPrincipled")
    base = tree.nodes.new("ShaderNodeTexImage")
    base.image = cel.palette_image(mat)
    base.interpolation = "Closest"
    tree.links.new(base.outputs["Color"], shader.inputs["Base Color"])
    shader.inputs["Roughness"].default_value = 0.6
    shader.inputs["Metallic"].default_value = 0.0

    glow = cel.glow_image(mat)

    if glow is not None:
        emissive = tree.nodes.new("ShaderNodeTexImage")
        emissive.image = glow
        emissive.interpolation = "Closest"
        tree.links.new(emissive.outputs["Color"], shader.inputs["Emission Color"])
        shader.inputs["Emission Strength"].default_value = 6.0

    tree.links.new(shader.outputs["BSDF"], output.inputs["Surface"])


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


# The angle each view looks from, turning about the model from its front.
# `eye` is a camera a tile places itself (see `row`); its lights are a front
# view's.
ANGLES = {"front": 0.0, "threequarter": 32.0, "side": 90.0, "back": 180.0, "silhouette": 0.0, "top": 0.0, "eye": 0.0}

# Every view a model sheet shows.
VIEWS = ("front", "threequarter", "side", "back", "silhouette")

# How tall, in pixels, a silhouette is shown beside its tile, at the size a
# phone shows a figure in a list or a card, so it can be judged where it is
# hardest to read: the art bible's 64 pt test (6.1). A cel sheet takes it
# from cel.toml ([sheets] silhouette_px).
THUMBNAIL = 64

# A portrait phone's screen, width over height: an `eye` tile is this shape,
# so it shows what the game's camera shows on a phone.
PHONE = 9 / 19.5

# The phone the art bible measures in: 390 x 844 points.
PHONE_PT = 844

# The lights of the normal look, about the model's middle, for a camera
# `REFERENCE` metres away at the front. They turn with the camera.
REFERENCE = 7.4
LIGHTS = (
    ("Key", (-3.2, -4.2, 2.3), 900, (1.0, 0.94, 0.86), 3.0),
    ("Rim", (2.6, 3.8, 1.9), 1500, (0.55, 0.75, 1.0), 2.0),
    ("Fill", (3.5, -3.0, -0.7), 140, (0.6, 0.7, 0.9), 4.0),
)
BACKGROUND = (0.012, 0.014, 0.02)
SILHOUETTE_GROUND = (0.45, 0.45, 0.45)


def sheet_path(directory, stem, cel_look=None):
    """Where a sheet goes: cel sheets end in `_cel`, beside the normal ones."""

    return os.path.join(directory, f"{stem}{'_cel' if cel_look else ''}.png")


def row(label, setup, views=VIEWS, look=None, eye=None):
    """One row of a sheet: the same scene, `setup()`, from each view.

    An `eye` view is seen through a camera of its own, which `eye` gives as
    `(location, rotation, fov_y)`: a world point, the quaternion that turns a
    camera looking down -Z with +Y up into it, and its vertical field of view
    in radians. Its tile is a phone's shape (`PHONE`)."""

    if "eye" in views and eye is None:
        raise ValueError(f"{label!r}: an eye view needs the eye's camera")

    return [{"label": f"{view}  {label}".strip(), "setup": setup, "view": view, "look": look, "eye": eye if view == "eye" else None} for view in views]


def render_sheet(path, centre=(0, 0, 1.4), distance=7.4, width=560, height=840, views=("front", "threequarter", "back", "silhouette"), look=None, eye_height=-0.25, view_rotation=None, cel_look=None):
    """Renders the scene from several angles into one side-by-side image,
    from a camera placed by hand. `render_grid` frames the model itself."""

    render_grid(
        path,
        [row("", None, views, look)],
        tile=(width, height),
        cel_look=cel_look,
        framing=(centre, distance, eye_height),
        view_rotation=view_rotation,
        labels=False,
    )


def render_grid(path, rows, tile=(400, 600), cel_look=None, lens=70, elevation=-2.0, margin=1.15, framing=None, view_rotation=None, labels=True, skip=("shadow",), same_scale=True, ground=None, figure_size=None, knockout=True):
    """Renders a sheet: rows of tiles, each a view of the scene as a tile's
    `setup()` leaves it, and stitches them into one image at `path`.

    Each row is framed on its own, from the bounds of every visible mesh in
    each of its views, at one distance for the whole row, so the tiles of a
    row are at one scale and nothing leaves the frame; `same_scale=False`
    frames every tile on its own instead, for a model much longer than it
    is wide. `framing` is
    `(centre, distance, eye_height)` to place the camera by hand instead.

    A tile's `look` dresses a creature in one NFT's colours and trait. A
    `silhouette` tile is the model in black (key on paper in the cel look),
    with the same silhouette at `THUMBNAIL` pixels beside it. `cel_look` is
    a sheet's look from `cel.look()`: the sheet is drawn in Second Ink, on
    the vigil ground, with a paper knockout round each figure unless
    `knockout` is False. `ground` is the colour behind every tile but a
    silhouette, in place of the look's own: the night sky behind light,
    which would not show on paper. `figure_size` is how big, in metres, the
    figure a tile is sized by is (the tallest of a lineup); by default each
    row's bounds say.
    """

    width, height = tile
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = width
    scene.render.resolution_y = height
    scene.render.resolution_percentage = 100
    scene.render.film_transparent = False
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.eevee.taa_render_samples = 16
    if hasattr(scene.eevee, "use_raytracing"):
        scene.eevee.use_raytracing = False
    scene.view_settings.view_transform = "Standard" if cel_look else "AgX"
    scene.view_settings.look = "None" if cel_look else "AgX - Medium High Contrast"

    world = bpy.data.worlds.get("PreviewWorld") or bpy.data.worlds.new("PreviewWorld")
    world.use_nodes = True
    scene.world = world
    background = world.node_tree.nodes["Background"]
    background.inputs["Strength"].default_value = 1.0

    lights = [_light(name, "AREA", (0, 0, 0), energy, colour, size=size) for name, _, energy, colour, size in LIGHTS]
    label = _label() if labels else None
    thumbnail = cel_look["sheets"]["silhouette_px"] if cel_look else THUMBNAIL

    if cel_look:
        cel.add_ink(cel_look)

    half_wide, half_high = _half_view(lens, width, height)
    styled = None
    grid = []

    try:
        for tiles in rows:
            placed = [_place(tile, lens, half_wide, half_high, elevation, margin, framing, skip) for tile in tiles]
            # An eye tile stands where its eye is, so it sets no row's scale.
            framed = [distance for tile, (_, distance, _, _) in zip(tiles, placed) if tile.get("eye") is None]
            widest = max(framed, default=0.0)
            sizes = [size for tile, (_, _, _, size) in zip(tiles, placed) if tile.get("eye") is None and size]
            row_size = figure_size or max(sizes, default=0.0)
            paths = []

            for tile, (centre, own, eye_height, size) in zip(tiles, placed):
                distance = widest if same_scale and tile.get("eye") is None else own
                if tile["setup"] is not None:
                    tile["setup"]()

                silhouette = tile["view"] == "silhouette"
                tile_width = width if tile.get("eye") is None else round(height * PHONE)
                scene.render.resolution_x = tile_width
                tile_wide, tile_high = (half_wide, half_high) if tile.get("eye") is None else _eye_half_view(tile.get("eye"), tile_width, height)
                # A floor shadow is not the creature's shape: a silhouette
                # leaves out what framing leaves out.
                unshaped = [obj for obj in bpy.data.objects if silhouette and skip and obj.type == "MESH" and not obj.hide_render and obj.name.startswith(tuple(skip))]

                for obj in unshaped:
                    obj.hide_render = True
                mode = ("silhouette" if silhouette else "cel" if cel_look else "normal", id(tile["look"]))

                if mode != styled:
                    _style(silhouette, tile["look"], cel_look)
                    styled = mode

                printed = bool(cel_look) and not silhouette
                cut_out = printed and knockout and ground is None
                if cel_look:
                    behind = cel_look["paper"] if silhouette else ground or cel_look["vigil"]
                else:
                    behind = SILHOUETTE_GROUND if silhouette else ground or BACKGROUND
                background.inputs["Color"].default_value = (*behind, 1)
                scene.render.film_transparent = cut_out
                if tile.get("eye") is None:
                    camera = _aim(centre, distance, tile["view"], elevation, eye_height, lens, view_rotation)
                else:
                    camera = _eye(tile.get("eye"))
                    distance = (Vector(centre) - camera.location).length

                for (_, offset, energy, _, size_light), light in zip(LIGHTS, lights):
                    _place_light(light, centre, distance, tile["view"], offset, energy, size_light)
                    light.hide_render = silhouette or bool(cel_look)

                # How many sheet pixels a point of the phone is, here: an eye
                # tile is the phone's own view; any other is the figure at
                # the height its scene stands it (cel.toml `figure_pt`).
                focal = height / (2 * tile_high)
                if tile.get("eye") is not None:
                    px_per_pt = height / PHONE_PT
                else:
                    sized = row_size if same_scale else (size or row_size)
                    px_per_pt = max(sized, 1e-3) * focal / max(distance, 1e-3) / cel_look["scene"]["figure_pt"] if cel_look else 1.0

                if cel_look:
                    cel.aim(cel_look, camera, focal, px_per_pt, cel.hatch_per_metre(cel_look, row_size or size or 1.0))
                    cel.set_ink(cel_look, camera, focal, px_per_pt, shown=not silhouette)

                if label is not None:
                    if cel_look:
                        ink = cel_look["key"] if silhouette or ground is not None else cel_look["ash"]
                    else:
                        ink = (0.1, 0.1, 0.12) if silhouette else (0.6, 0.62, 0.66)
                    _caption(label, camera, tile["label"], ink, tile_wide, tile_high, height)

                tile_path = f"{path}.{len(grid)}.{len(paths)}.png"
                scene.render.filepath = tile_path
                bpy.ops.render.render(write_still=True)
                inset = None

                if silhouette and thumbnail < height:
                    if label is not None:
                        label.hide_render = True
                    inset = f"{tile_path}.thumb.png"
                    scene.render.resolution_x = round(thumbnail * tile_width / height)
                    scene.render.resolution_y = thumbnail
                    scene.render.filepath = inset
                    bpy.ops.render.render(write_still=True)
                    scene.render.resolution_x = tile_width
                    scene.render.resolution_y = height
                    if label is not None:
                        label.hide_render = False

                for obj in unshaped:
                    obj.hide_render = False

                composite = None

                if cut_out:
                    # The caption's rows, descenders and all, get no margin.
                    caption = math.ceil(0.04 * height + 2 * CAPTION_PIXELS) if label is not None else 0
                    composite = {"ground": behind, "knockout": cel_look["line"]["knockout_pt"] * px_per_pt, "paper": cel_look["paper"], "caption": caption}

                paths.append((tile_path, inset, composite))
                bpy.data.objects.remove(camera, do_unlink=True)

            grid.append(paths)
    finally:
        scene.render.film_transparent = False
        if cel_look:
            cel.remove_ink()
            cel.restore()
        for light in lights:
            bpy.data.objects.remove(light, do_unlink=True)
        if label is not None:
            bpy.data.objects.remove(label, do_unlink=True)

    compose(grid, path, width, height)


def _style(silhouette, look, cel_look):
    if cel_look and silhouette:
        cel.silhouette(cel_look)
    elif cel_look:
        _dress(look)
        cel.style(cel_look, {name: look[name] for name in ("body", "trim", "detail")} if look is not None else None)
    else:
        _style_materials(silhouette, look, review=True)


def _half_view(lens, width, height):
    """How far the camera sees either side of its middle, across and up, at
    one metre: the 36 mm sensor spans the longer side."""

    longer = 18 / lens
    shorter = longer * min(width, height) / max(width, height)

    return (longer, shorter) if width >= height else (shorter, longer)


def _eye_half_view(eye, width, height):
    """How far an eye camera sees either side of its middle, across and up,
    at one metre."""

    half_high = math.tan(eye[2] / 2)

    return half_high * width / height, half_high


def _eye(eye):
    """A camera at an eye: `(location, rotation, fov_y)`, as `row` takes it."""

    location, rotation, fov_y = eye
    data = bpy.data.cameras.new("PreviewCamera")
    data.sensor_fit = "VERTICAL"
    data.angle_y = fov_y
    data.clip_start = 0.01
    camera = bpy.data.objects.new("PreviewCamera", data)
    camera.location = location
    camera.rotation_mode = "QUATERNION"
    camera.rotation_quaternion = rotation
    bpy.context.scene.collection.objects.link(camera)
    bpy.context.scene.camera = camera
    bpy.context.view_layer.update()

    return camera


def _direction(view, elevation):
    """From the model's middle toward the camera, for a view."""

    if view == "top":
        # Straight down, the front of the model at the bottom of the frame.
        return Vector((0.0, -1e-3, 1.0)).normalized()

    angle = math.radians(ANGLES.get(view, 24.0))
    rise = math.radians(elevation)

    return Vector((math.sin(angle) * math.cos(rise), -math.cos(angle) * math.cos(rise), math.sin(rise)))


def _place(tile, lens, half_wide, half_high, elevation, margin, framing, skip):
    """Where a tile's camera looks and how far back it must stand for every
    visible mesh to fit, with `margin` to spare."""

    if framing is not None:
        return (*framing, None)

    if tile["setup"] is not None:
        tile["setup"]()
    _dress(tile["look"])
    bpy.context.view_layer.update()

    corners = visible_corners(skip)

    if not corners:
        raise ValueError(f"nothing to render for {tile['label']!r}: every mesh is hidden")

    low = Vector(map(min, *corners)) if len(corners) > 1 else corners[0]
    high = Vector(map(max, *corners)) if len(corners) > 1 else corners[0]
    centre = (low + high) / 2
    back = _direction(tile["view"], elevation)
    right = Vector((0, 0, 1)).cross(back).normalized()
    up = back.cross(right).normalized()
    distance = 0.0

    for corner in corners:
        offset = corner - centre
        toward = offset.dot(back)
        distance = max(
            distance,
            toward + abs(offset.dot(right)) * margin / half_wide,
            toward + abs(offset.dot(up)) * margin / half_high,
        )

    # How big the figure is: its height, or its span across where it is
    # wider than it is tall (a bird's wings, a weapon lying down).
    extent = high - low
    size = max(extent.z, max(extent.x, extent.y) * 0.75)

    return centre, distance, None, size


def visible_corners(skip):
    """The world corners of the bounds of every mesh that will render, but
    those whose names start with any of `skip`."""

    corners = []
    depsgraph = bpy.context.evaluated_depsgraph_get()

    for obj in bpy.data.objects:
        if obj.type != "MESH" or obj.hide_render or (skip and obj.name.startswith(tuple(skip))):
            continue

        evaluated = obj.evaluated_get(depsgraph)
        corners.extend(evaluated.matrix_world @ Vector(corner) for corner in evaluated.bound_box)

    return corners


def _aim(centre, distance, view, elevation, eye_height, lens, view_rotation):
    centre = Vector(centre)

    if eye_height is None:
        eye = centre + _direction(view, elevation) * distance
    else:
        angle = math.radians(ANGLES.get(view, 24.0))
        eye = Vector((centre.x + math.sin(angle) * distance, centre.y - math.cos(angle) * distance, centre.z + eye_height))

    if view_rotation is not None:
        eye = centre + view_rotation @ (eye - centre)

    camera = _camera(tuple(eye), tuple(centre), lens=lens)
    camera.data.clip_start = 0.01

    if view_rotation is not None:
        local_direction = view_rotation.inverted() @ (centre - eye)
        camera.rotation_euler = (view_rotation @ local_direction.to_track_quat("-Z", "Y")).to_euler()

    bpy.context.view_layer.update()

    return camera


def _place_light(light, centre, distance, view, offset, energy, size):
    """Puts a light where it sits for a front view, turned with the camera
    and scaled with its distance, so every view and every size of model is
    lit alike."""

    scale = distance / REFERENCE
    turn = Matrix.Rotation(math.radians(ANGLES.get(view, 24.0)), 3, "Z")
    location = Vector(centre) + turn @ (Vector(offset) * scale)
    light.location = location
    light.rotation_euler = (Vector(centre) - location).to_track_quat("-Z", "Y").to_euler()
    light.data.energy = energy * scale * scale
    light.data.size = size * scale


def _label():
    """A caption the camera carries in its top left corner."""

    data = bpy.data.curves.new("review_label", "FONT")
    data.align_y = "TOP"
    label = bpy.data.objects.new("review_label", data)
    bpy.context.scene.collection.objects.link(label)
    label.visible_shadow = False

    mat = bpy.data.materials.get("review_label") or bpy.data.materials.new("review_label")
    mat.use_nodes = True
    tree = mat.node_tree
    tree.nodes.clear()
    output = tree.nodes.new("ShaderNodeOutputMaterial")
    emission = tree.nodes.new("ShaderNodeEmission")
    emission.inputs["Color"].default_value = (0.55, 0.55, 0.58, 1)
    tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])
    data.materials.append(mat)

    return label


# A caption's height in pixels.
CAPTION_PIXELS = 15


def _caption(label, camera, text, colour, half_wide, half_high, height, depth=0.05, pixels=CAPTION_PIXELS):
    label.data.body = text
    label.data.materials[0].node_tree.nodes["Emission"].inputs["Color"].default_value = (*colour, 1)
    label.data.size = 2 * half_high * depth * pixels / height
    label.parent = camera
    label.matrix_parent_inverse.identity()
    label.location = (-half_wide * depth * 0.95, half_high * depth * 0.96, -depth)
    label.rotation_euler = (0, 0, 0)


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
    scene.eevee.taa_render_samples = 16
    if hasattr(scene.eevee, "use_raytracing"):
        scene.eevee.use_raytracing = False
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


def compose(grid, path, width, height):
    """Places rendered tiles in rows, top row first, and removes them. A
    silhouette's thumbnail goes in a narrow panel of its own beside it, at
    its own size, so it never covers the silhouette. A tile rendered see-
    through for the cel look gets its paper knockout and its ground here
    (`cel.knockout`)."""

    import numpy

    def load(file):
        image = bpy.data.images.load(file)
        w, h = image.size
        pixels = numpy.empty(w * h * 4, dtype=numpy.float32)
        image.pixels.foreach_get(pixels)
        bpy.data.images.remove(image)
        os.remove(file)

        return pixels.reshape(h, w, 4)

    rows = []

    for paths in grid:
        pieces = []

        for tile, inset, composite in paths:
            pixels = load(tile)

            if composite is not None:
                pixels = cel.knockout(pixels, composite["knockout"], composite["ground"], composite["paper"], composite["caption"])

            pieces.append(pixels)

            if inset is not None:
                small = load(inset)
                h, w = small.shape[:2]
                gap = 12
                panel = numpy.empty((height, w + 2 * gap, 4), dtype=numpy.float32)
                # The silhouette's own ground, as it was rendered.
                panel[:] = pixels[-1, -1]
                # A thin frame round the thumbnail, at the bottom of the
                # panel, so it reads as the picture a phone shows.
                y = gap
                panel[y - 2 : y + h + 2, gap - 2 : gap + w + 2] = (0.5, 0.5, 0.5, 1.0)
                panel[y : y + h, gap : gap + w] = small
                pieces.append(panel)

        rows.append(numpy.concatenate(pieces, axis=1))

    sheet_width = max(row.shape[1] for row in rows)
    sheet = numpy.zeros((height * len(rows), sheet_width, 4), dtype=numpy.float32)

    for index, pixels in enumerate(rows):
        # Blender's images start at the bottom row. A row narrower than the
        # widest runs on in the ground of its last tile.
        bottom = (len(rows) - 1 - index) * height
        sheet[bottom : bottom + height] = pixels[-1, -1]
        sheet[bottom : bottom + height, : pixels.shape[1]] = pixels

    sheet[:, :, 3] = 1.0
    result = bpy.data.images.new("Sheet", sheet_width, height * len(rows), alpha=False)
    result.pixels.foreach_set(sheet.ravel())
    result.filepath_raw = path
    result.file_format = "PNG"
    result.save()
    bpy.data.images.remove(result)
    print(f"SHEET {path}")
