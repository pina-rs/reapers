"""Second Ink, as a review sheet draws it (docs/art-bible.md, sections 3 to 5),
so a model is judged in the game's look before the game's renderer draws it.

Everything is read from art/blender/cel.toml, the one file of art direction:

- The print. Every colour is key ink and one spot (the reaper's light)
  overprinted on paper, as (k, s) coverages per material family (3.3, 3.4).
  A husk prints in key over a ghost of its NFT's own colour, its hue kept and
  its chroma capped (3.6).
- Bands. Three hard bands from N.L against one light fixed to the camera,
  with the thresholds of the sheet's scene (5.1, 5.4): lit, shade and deep,
  and a crosshatch below the deepest on a reaper.
- Hatch. Key lines in the deep band only, in the figure's own space so they
  turn with the form, thickening as the form darkens, antialiased and faded
  to their mean tone as their pitch on screen falls under a few device
  pixels, so they never moire (5.2).
- A paper highlight on turning edges, and on a reaper a rim of its light on
  the side away from the light (5.3).
- The key line, an inverted hull: each mesh welded, its normals smoothed
  over a few rings, pushed out by the weight the build baked (`lib/ink.py`)
  times the game's light-side term and surface-fixed wobble, at a width
  constant on screen (4.2 to 4.4). Light, darkness and pictures have none.
- A paper knockout round each whole figure, outside the line, by dilating
  the render's alpha, on the vigil ground (4.1). See `preview.compose`.

Only the review sheets use this. It changes no model: the line is a
modifier added for the render and every material is put back after it.
"""

import math
import os
import tomllib
import zlib

import bpy
from mathutils import Vector

from . import ink

# The palette, beside the drivers.
PALETTE = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "cel.toml")

# Materials that show an NFT's image in the game. A review shows Blender's
# numbered colour grid on them instead, which makes stretched, flipped or
# misplaced texture coordinates obvious.
IMAGE_MATERIALS = ("canvas", "paper_image", "shard", "ribbon")

# The first-person effects' unlit materials (`viewmodel_parts/effects.py`):
# a soft spot the game tints and adds to what is behind, a flat light it
# tints, and a soft dark disc it blends behind a glow.
EFFECT_GLOW = "fx_glow"
EFFECT_LIGHT = "fx_light"
EFFECT_SHADE = "fx_shade"
EFFECTS = (EFFECT_GLOW, EFFECT_LIGHT, EFFECT_SHADE)

# Materials that never get a line: effects, a floor shadow and pictures.
# Light and darkness have none either; cel.toml says so (`line = 0`).
UNINKED = (*EFFECTS, "shadow", *IMAGE_MATERIALS)

# Objects and materials the review itself adds, which are never restyled.
REVIEW_PREFIX = "review_"

INK_MODIFIER = "review_ink"
INK_GROUP = "review_ink_hull"
INK_MATERIAL = "review_ink"
PRINT_GROUP = "review_print"
SILHOUETTE_MATERIAL = "review_key"

# Attributes the line reads, written for a render: the family's line weight
# by vertex, and the faces that draw no line.
LINE_ATTRIBUTE = "review_line"
UNLINED_ATTRIBUTE = "review_unlined"

# What a figure's root carries to be printed in a light of its own, or a
# husk in its own NFT's colours (a lineup sets them per figure).
LIGHT_PROPERTY = "review_light"
LOOK_PROPERTY = "review_look"


# Colour ---------------------------------------------------------------------


def srgb(hex_colour):
    """An sRGB `#rrggbb` colour (or 0xRRGGBB number) as three 0..1 floats."""

    if isinstance(hex_colour, int):
        value = f"{hex_colour:06x}"
    else:
        value = hex_colour.lstrip("#")

    if len(value) != 6:
        raise ValueError(f"{hex_colour!r} is not a #rrggbb colour")

    return tuple(int(value[at : at + 2], 16) / 255 for at in (0, 2, 4))


def to_linear(colour):
    """sRGB 0..1 to the linear RGB Blender works in."""

    return tuple(channel / 12.92 if channel <= 0.04045 else ((channel + 0.055) / 1.055) ** 2.4 for channel in colour)


def to_srgb(colour):
    """Linear RGB to sRGB 0..1: what a saved sheet's pixels hold."""

    return tuple(channel * 12.92 if channel <= 0.0031308 else 1.055 * channel ** (1 / 2.4) - 0.055 for channel in colour)


def linear(hex_colour):
    """An sRGB `#rrggbb` colour as linear RGB."""

    return to_linear(srgb(hex_colour))


def overprint(paper, key, k, s, spot, base=None):
    """Paper (or `base`) printed with key at coverage `k` and `spot` at `s`,
    per sRGB channel (3.1)."""

    base = base or paper

    return tuple(
        max(0.0, min(1.0, base[i] * (1 - k + k * key[i] / paper[i]) * (1 - s + s * spot[i] / paper[i])))
        for i in range(3)
    )


def _lab(colour):
    r, g, b = to_linear(colour)
    x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047
    y = 0.2126 * r + 0.7152 * g + 0.0722 * b
    z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883

    def f(t):
        return t ** (1 / 3) if t > 216 / 24389 else (24389 / 27 * t + 16) / 116

    fx, fy, fz = f(x), f(y), f(z)

    return 116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)


def _from_lab(lightness, a, b):
    fy = (lightness + 16) / 116
    fx, fz = fy + a / 500, fy - b / 200

    def inverse(t):
        return t**3 if t**3 > 216 / 24389 else (116 * t - 16) / (24389 / 27)

    x, y, z = inverse(fx) * 0.95047, inverse(fy), inverse(fz) * 1.08883
    rgb = (3.2406 * x - 1.5372 * y - 0.4986 * z, -0.9689 * x + 1.8758 * y + 0.0415 * z, 0.0557 * x - 0.2040 * y + 1.0570 * z)

    def encode(channel):
        channel = max(0.0, min(1.0, channel))

        return 12.92 * channel if channel <= 0.0031308 else 1.055 * channel ** (1 / 2.4) - 0.055

    return tuple(encode(channel) for channel in rgb)


def hue(colour):
    """A colour's CIELAB hue in degrees."""

    _, a, b = _lab(colour)

    return math.degrees(math.atan2(b, a)) % 360


# The palette ----------------------------------------------------------------


def load(path=PALETTE):
    """cel.toml, with its colours read and its numbers checked."""

    with open(path, "rb") as source:
        raw = tomllib.load(source)

    inks = {name: srgb(value) for name, value in raw["inks"].items()}
    lights = {name: srgb(value) for name, value in raw["lights"].items()}

    for calling, light in raw["classes"].items():
        if light not in lights:
            raise ValueError(f"{path}: [classes] {calling} names no light: {light!r}")

    for kind, calling in raw["weapons"].items():
        if calling not in raw["classes"]:
            raise ValueError(f"{path}: [weapons] {kind} names no class: {calling!r}")

    families = {}

    for name, family in raw["families"].items():
        if not isinstance(family, dict) or name == "aliases":
            continue

        if not ({"lit", "shade", "deep"} <= set(family) or "flat" in family or family.get("light")):
            raise ValueError(f"{path}: [families.{name}] needs lit, shade and deep, or flat, or light")

        families[name] = family

    fallback = raw["families"]["fallback"]

    if fallback not in families:
        raise ValueError(f"{path}: [families] fallback names no family: {fallback!r}")

    for name, scene in raw["scenes"].items():
        if Vector(scene["light"]).length == 0:
            raise ValueError(f"{path}: [scenes.{name}] light must point somewhere")

        if not scene["t_deep"] <= scene["t_lit"]:
            raise ValueError(f"{path}: [scenes.{name}] t_deep must not be above t_lit")

        if "hatch_per_m" not in scene and "hatch_pt" not in scene:
            raise ValueError(f"{path}: [scenes.{name}] needs hatch_pt or hatch_per_m")

    for kind, scene in raw["sheets"].items():
        if kind in ("reaper", "weapon", "husk", "first_person") and scene not in raw["scenes"]:
            raise ValueError(f"{path}: [sheets] {kind} names no scene: {scene!r}")

    if raw["colourway"]["robe"] not in ("night", "ember", "ash"):
        raise ValueError(f"{path}: [colourway] robe is night, ember or ash")

    return {
        "path": path,
        "raw": raw,
        "inks": inks,
        "lights": lights,
        "classes": raw["classes"],
        "weapons": raw["weapons"],
        "families": families,
        "aliases": raw["families"].get("aliases", {}),
        "fallback": fallback,
        "ghost": raw["ghost"],
        "line": raw["line"],
        "hatch": raw["hatch"],
        "scenes": raw["scenes"],
        "sheets": raw["sheets"],
        "colourway": raw["colourway"],
        "weight": ink.settings(path),
        # What a preview paints its ground and its captions with.
        "paper": to_linear(inks["paper"]),
        "vigil": to_linear(inks["vigil"]),
        "key": to_linear(inks["key"]),
        "ash": to_linear(inks["ash"]),
    }


def look(palette, kind, light=None, scene=None):
    """The look one sheet is drawn in: `palette` for a `kind` of model
    (`reaper`, `weapon`, `husk` or `first_person`), in `light` (a light's
    name, the spot; none for a husk) and `scene` (cel.toml's [sheets] when
    None)."""

    if palette is None:
        return None

    name = scene or palette["sheets"][kind]

    if name not in palette["scenes"]:
        raise ValueError(f"no scene {name!r} in {palette['path']}; there are {', '.join(palette['scenes'])}")

    if light is not None and light not in palette["lights"]:
        raise ValueError(f"no light {light!r} in {palette['path']}")

    return {**palette, "kind": kind, "light": light, "scene_name": name, "scene": palette["scenes"][name]}


def with_scene(palette, scene):
    """`palette` with every kind of sheet drawn in `scene` rather than the
    one [sheets] names: `review.py --scene hunt`."""

    if palette is None or scene is None:
        return palette

    if scene not in palette["scenes"]:
        raise ValueError(f"no scene {scene!r} in {palette['path']}; there are {', '.join(palette['scenes'])}")

    kinds = ("reaper", "weapon", "husk", "first_person")

    return {**palette, "sheets": {**palette["sheets"], **dict.fromkeys(kinds, scene)}}


def class_light(palette, calling):
    return palette["classes"][calling]


def weapon_light(palette, kind):
    return palette["classes"][palette["weapons"][kind]]


def base_name(mat):
    """A material's name without the `.001` Blender adds to a duplicate."""

    return mat.name.split(".")[0]


# What each material prints as -----------------------------------------------


def family_name(cel_look, name):
    """The family a material called `name` prints in."""

    name = cel_look["aliases"].get(name, name)

    if name == "image" or name in IMAGE_MATERIALS:
        return "image"

    if name in EFFECTS or name == "shadow":
        return name

    if cel_look["kind"] == "husk" and name in ("body", "trim", "detail"):
        return name

    return name if name in cel_look["families"] else cel_look["fallback"]


def line_weight(cel_look, name):
    """How heavy the line is on a material called `name`: 0 for none."""

    family = family_name(cel_look, name)

    if family in ("image", "shadow", *EFFECTS):
        return 0.0

    if family in ("body", "trim", "detail"):
        return 1.0

    return float(cel_look["families"][family].get("line", 1.0))


def _coverages(cel_look, family_key):
    """A family's (k, s) bands, after the scene's overrides and the robe's
    colourway."""

    family = dict(cel_look["families"][family_key])
    family.update(cel_look["scene"].get("families", {}).get(family_key, {}))

    if family_key == "cloth" and cel_look["kind"] != "husk":
        robe = cel_look["colourway"]["robe"]

        if robe == "ember":
            family.update({band: cel_look["families"]["cloth_alt"][band] for band in ("lit", "shade", "deep")})
        elif robe == "ash":
            family.update(cel_look["colourway"]["ash"])

    return family


def ghost(cel_look, colour):
    """A husk's colour as the dead print it (3.6): (lit, shade, deep) sRGB,
    and the specular threshold, or 0 for none."""

    settings = cel_look["ghost"]
    paper, key = cel_look["inks"]["paper"], cel_look["inks"]["key"]
    lightness, a, b = _lab(srgb(colour))
    low, high = settings["lightness"]
    lightness = max(low, min(high, lightness))
    chroma = math.hypot(a, b)
    cap = settings["chroma"]
    player = settings.get("player", "")

    if player:
        player_hue = hue(cel_look["lights"][cel_look["classes"][player]])
        distance = abs((math.degrees(math.atan2(b, a)) - player_hue + 180) % 360 - 180)

        if distance < settings["clash_degrees"]:
            cap = min(cap, settings["clash_chroma"])

    if chroma > cap:
        a, b = a * cap / chroma, b * cap / chroma

    lit = _from_lab(lightness, a, b)
    shade = overprint(paper, key, settings["shade_k"], 0.0, paper, base=lit)
    deep = overprint(paper, key, settings["deep_k"], 0.0, paper, base=lit)

    if _lab(deep)[0] < settings["deep_floor"]:
        deep = _from_lab(settings["deep_floor"], 0, 0)

    spec = settings["spec"] if lightness > settings["spec_lightness"] else 0.0

    return (lit, shade, deep), spec


def bands(cel_look, family_key, spot, look_colours=None):
    """What a family prints as, in sRGB: a dict with `flat` (one colour) or
    `lit`, `shade` and `deep`, and `hatch`, `hatch_min`, `cross`, `spec` and
    `rim` for the print group."""

    paper, key = cel_look["inks"]["paper"], cel_look["inks"]["key"]
    husk = cel_look["kind"] == "husk"
    reaper_like = not husk

    if husk and family_key in ("body", "trim", "detail"):
        colour = (look_colours or {}).get(family_key)

        if colour is None:
            colour = cel_look["ghost"]["fallback"][family_key]

        (lit, shade, deep), spec = ghost(cel_look, colour)

        return {"lit": lit, "shade": shade, "deep": deep, "hatch": 1.0, "hatch_min": cel_look["ghost"]["hatch_min"], "cross": 0.0, "spec": spec, "rim": 0.0}

    family = _coverages(cel_look, family_key)

    if family.get("light"):
        if husk:
            # The dead have no light: eyes print paper, seals key.
            return {"flat": paper if family_key == "accent" else key}

        return {"flat": spot}

    # On a husk the spot never prints: its other families are key only.
    spot_share = 0.0 if husk else 1.0

    if "flat" in family:
        k, s = family["flat"]

        return {"flat": overprint(paper, key, k, s * spot_share, spot)}

    printed = {band: overprint(paper, key, family[band][0], family[band][1] * spot_share, spot) for band in ("lit", "shade", "deep")}

    return {
        **printed,
        "hatch": 1.0 if family.get("hatch") else 0.0,
        "hatch_min": cel_look["ghost"]["hatch_min"] if husk else cel_look["hatch"]["duty"]["minimum"],
        "cross": 1.0 if reaper_like else 0.0,
        "spec": float(family.get("spec", 0.0)),
        "rim": 1.0 if reaper_like else 0.0,
    }


def flat_colour(palette, name, light):
    """A material's lit band in `light`, as linear RGB: one flat colour by
    name, for a quick look that has no bands (the motion catalogue)."""

    sheet = look(palette, "reaper", light)
    family_key = family_name(sheet, name)

    if family_key in ("image", "shadow", *EFFECTS):
        return sheet["paper"]

    printed = bands(sheet, family_key, palette["lights"][light])

    return to_linear(printed.get("flat") or printed["lit"])


# Materials ------------------------------------------------------------------


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

    from .core import COLOR_ATTRIBUTE

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


class _Nodes:
    """Builds a node tree a node at a time: numbers or sockets go in."""

    def __init__(self, tree):
        self.tree = tree

    def new(self, kind, **settings):
        node = self.tree.nodes.new(kind)

        for key, value in settings.items():
            setattr(node, key, value)

        return node

    def wire(self, value, socket):
        if isinstance(value, (int, float)):
            socket.default_value = value
        elif isinstance(value, tuple):
            socket.default_value = value
        else:
            self.tree.links.new(value, socket)

    def math(self, operation, *values, clamp=False):
        node = self.new("ShaderNodeMath", operation=operation, use_clamp=clamp)

        for socket, value in zip(node.inputs, values):
            self.wire(value, socket)

        return node.outputs[0]

    def vector(self, operation, *values):
        node = self.new("ShaderNodeVectorMath", operation=operation)

        for socket, value in zip(node.inputs, values):
            self.wire(value, socket)

        return node.outputs["Value"] if operation in ("DOT_PRODUCT", "LENGTH", "DISTANCE") else node.outputs["Vector"]

    def mix(self, a, b, factor):
        node = self.new("ShaderNodeMix", data_type="RGBA")
        self.wire(factor, node.inputs["Factor"])
        self.wire(a if not isinstance(a, tuple) else (*a[:3], 1.0), node.inputs["A"])
        self.wire(b if not isinstance(b, tuple) else (*b[:3], 1.0), node.inputs["B"])

        return node.outputs["Result"]

    def value(self, name, number):
        node = self.new("ShaderNodeValue", name=name, label=name)
        node.outputs[0].default_value = number

        return node.outputs[0]

    def smoothstep(self, low, high, value):
        node = self.new("ShaderNodeMapRange", interpolation_type="SMOOTHSTEP", clamp=True)
        self.wire(value, node.inputs["Value"])
        self.wire(low, node.inputs["From Min"])
        self.wire(high, node.inputs["From Max"])

        return node.outputs["Result"]


def _print_group(cel_look):
    """The node group every printed material runs: bands, hatch, crosshatch,
    paper specular and spot rim, from the family's colours (`bands`)."""

    group = bpy.data.node_groups.get(PRINT_GROUP)

    if group is not None:
        bpy.data.node_groups.remove(group)

    group = bpy.data.node_groups.new(PRINT_GROUP, "ShaderNodeTree")
    interface = group.interface

    for name in ("Lit", "Shade", "Deep", "Spot"):
        interface.new_socket(name, in_out="INPUT", socket_type="NodeSocketColor")

    for name in ("Hatch", "HatchMin", "Cross", "Spec", "Rim"):
        interface.new_socket(name, in_out="INPUT", socket_type="NodeSocketFloat")

    interface.new_socket("Color", in_out="OUTPUT", socket_type="NodeSocketColor")

    n = _Nodes(group)
    source = n.new("NodeGroupInput")
    out = n.new("NodeGroupOutput")
    scene = cel_look["scene"]
    hatch = cel_look["hatch"]
    key = (*cel_look["key"], 1.0)
    paper = (*cel_look["paper"], 1.0)

    geometry = n.new("ShaderNodeNewGeometry")
    camera = n.new("ShaderNodeCameraData")
    light = n.new("ShaderNodeCombineXYZ", name="light", label="light")
    normal = geometry.outputs["Normal"]
    view = n.vector("NORMALIZE", geometry.outputs["Incoming"])
    position = geometry.outputs["Position"]

    t_lit = n.value("t_lit", scene["t_lit"])
    t_deep = n.value("t_deep", scene["t_deep"])
    t_cross = n.value("t_cross", scene["t_cross"])
    rim_reach = n.value("rim", scene["rim"])
    per_metre = n.value("hatch_per_m", 40.0)
    focal = n.value("focal", 1000.0)
    device = n.value("device", 1.0)

    facing_light = n.vector("DOT_PRODUCT", normal, light.outputs["Vector"])
    facing_view = n.vector("DOT_PRODUCT", normal, view)
    in_shade = n.math("LESS_THAN", facing_light, t_lit)
    in_deep = n.math("LESS_THAN", facing_light, t_deep)
    colour = n.mix(source.outputs["Lit"], source.outputs["Shade"], in_shade)
    colour = n.mix(colour, source.outputs["Deep"], in_deep)

    # The hatch's pitch on the sheet, in pixels: what one period of it spans
    # at this depth, foreshortened as the surface turns away (5.2's fwidth).
    depth = camera.outputs["View Z Depth"]
    turned = n.math("MAXIMUM", facing_view, 0.2)
    pitch = n.math("DIVIDE", n.math("MULTIPLY", focal, turned), n.math("MULTIPLY", depth, per_metre))
    duty_settings = hatch["duty"]
    duty = n.math(
        "MINIMUM",
        n.math("MULTIPLY_ADD", n.math("SUBTRACT", t_deep, facing_light), duty_settings["gain"], source.outputs["HatchMin"]),
        duty_settings["maximum"],
    )

    def lines(direction, scale, width, own_pitch, under):
        """Key lines along `direction` at `scale` times the pitch, `width`
        of each period inked, antialiased and faded: the colour under them."""

        across = n.math("MULTIPLY", n.vector("DOT_PRODUCT", position, tuple(Vector(direction).normalized())), n.math("MULTIPLY", per_metre, scale))
        centred = n.math("SUBTRACT", n.math("FRACT", n.math("ADD", n.math("SUBTRACT", across, n.math("MULTIPLY", width, 0.5)), 0.5)), 0.5)
        inside = n.math("SUBTRACT", n.math("MULTIPLY", width, 0.5), n.math("ABSOLUTE", centred))
        cover = n.math("MULTIPLY_ADD", inside, own_pitch, 0.5, clamp=True)
        drawn = n.mix(under, key, cover)
        mean = n.mix(under, key, width)
        fade = n.smoothstep(hatch["fade_px"][0], hatch["fade_px"][1], n.math("MULTIPLY", own_pitch, device))

        return n.mix(mean, drawn, fade)

    hatched = lines(hatch["direction"], 1.0, duty, pitch, colour)
    colour = n.mix(colour, hatched, n.math("MULTIPLY", in_deep, source.outputs["Hatch"]))
    cross_pitch = n.math("DIVIDE", pitch, hatch["cross_scale"])
    crossed = lines(hatch["cross_direction"], hatch["cross_scale"], hatch["cross_duty"], cross_pitch, colour)
    in_cross = n.math("MULTIPLY", n.math("LESS_THAN", facing_light, t_cross), n.math("MULTIPLY", source.outputs["Cross"], source.outputs["Hatch"]))
    colour = n.mix(colour, crossed, in_cross)

    # Paper on turning edges, where N.H passes the family's threshold.
    halfway = n.vector("NORMALIZE", n.vector("ADD", view, light.outputs["Vector"]))
    shine = n.math("GREATER_THAN", n.vector("DOT_PRODUCT", normal, halfway), source.outputs["Spec"])
    turning = n.math("LESS_THAN", facing_view, 0.5)
    has_spec = n.math("GREATER_THAN", source.outputs["Spec"], 0.0)
    colour = n.mix(colour, paper, n.math("MULTIPLY", n.math("MULTIPLY", shine, turning), has_spec))

    # The spot leaking round the far edge.
    edge = n.math("LESS_THAN", facing_view, rim_reach)
    away = n.math("LESS_THAN", facing_light, 0.05)
    colour = n.mix(colour, source.outputs["Spot"], n.math("MULTIPLY", n.math("MULTIPLY", edge, away), source.outputs["Rim"]))

    group.links.new(colour, out.inputs["Color"])

    return group


def _print_material(cel_look, family_key, spot, look_colours, original):
    """A printed material: `family_key` in `spot`, through the print group."""

    mat = bpy.data.materials.new(f"{REVIEW_PREFIX}print_{family_key}")
    mat.use_nodes = True
    tree = mat.node_tree
    tree.nodes.clear()
    n = _Nodes(tree)
    output = n.new("ShaderNodeOutputMaterial")
    emission = n.new("ShaderNodeEmission")
    tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])

    if family_key in EFFECTS:
        tree.nodes.remove(emission)
        effect(tree, output, mat, to_linear(spot), name=base_name(original))

        return mat

    if family_key == "shadow":
        _shadow(n, tree, output, emission, cel_look, mat)

        return mat

    if family_key == "image":
        # A portrait face, posterised to paper, k .55 and key (6.3).
        texture = n.new("ShaderNodeTexImage", image=grid_image())
        value = n.new("ShaderNodeRGBToBW")
        tree.links.new(texture.outputs["Color"], value.inputs["Color"])
        paper, key = cel_look["inks"]["paper"], cel_look["inks"]["key"]
        middle = to_linear(overprint(paper, key, 0.55, 0.0, paper))
        colour = n.mix(to_linear(key), middle, n.math("GREATER_THAN", value.outputs["Val"], 0.12))
        colour = n.mix(colour, to_linear(paper), n.math("GREATER_THAN", value.outputs["Val"], 0.45))
        tree.links.new(colour, emission.inputs["Color"])

        return mat

    printed = bands(cel_look, family_key, spot, look_colours)

    if "flat" in printed:
        emission.inputs["Color"].default_value = (*to_linear(printed["flat"]), 1.0)

        return mat

    _run_group(n, tree, emission, printed, spot)

    return mat


def _run_group(n, tree, emission, printed, spot, colours=None):
    node = n.new("ShaderNodeGroup")
    node.node_tree = bpy.data.node_groups[PRINT_GROUP]

    for band in ("lit", "shade", "deep"):
        socket = node.inputs[band.title()]

        if colours is not None and band in colours:
            tree.links.new(colours[band], socket)
        else:
            socket.default_value = (*to_linear(printed[band]), 1.0)

    node.inputs["Spot"].default_value = (*to_linear(spot), 1.0)

    for name, key in (("Hatch", "hatch"), ("HatchMin", "hatch_min"), ("Cross", "cross"), ("Spec", "spec"), ("Rim", "rim")):
        if colours is not None and key in colours:
            tree.links.new(colours[key], node.inputs[name])
        else:
            node.inputs[name].default_value = printed[key]

    tree.links.new(node.outputs["Color"], emission.inputs["Color"])


def _palette_material(cel_look, spot, original):
    """The first-person palette (`viewmodel_parts/common.py`) printed: each
    cell's family's bands, read from small images by the parts' coordinates."""

    from viewmodel_parts.common import CELLS, SURFACES

    mat = bpy.data.materials.new(f"{REVIEW_PREFIX}print_palette")
    mat.use_nodes = True
    tree = mat.node_tree
    tree.nodes.clear()
    n = _Nodes(tree)
    output = n.new("ShaderNodeOutputMaterial")
    emission = n.new("ShaderNodeEmission")
    tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])

    strips = {band: [(0.0, 0.0, 0.0)] * CELLS for band in ("lit", "shade", "deep", "flags")}

    for surface, (cell, _) in SURFACES.items():
        family_key = family_name(cel_look, "bone" if surface == "bone_dark" else surface)
        printed = bands(cel_look, family_key, spot)

        if "flat" in printed:
            colour = to_linear(printed["flat"])
            strips["lit"][cell] = strips["shade"][cell] = strips["deep"][cell] = colour
            strips["flags"][cell] = (0.0, 0.0, 0.0)
            continue

        for band in ("lit", "shade", "deep"):
            strips[band][cell] = to_linear(printed[band])

        # Hatch, specular threshold and rim, packed in one strip.
        strips["flags"][cell] = (printed["hatch"], printed["spec"], printed["rim"])

    colours = {}

    for band, cells in strips.items():
        image = bpy.data.images.new(f"{REVIEW_PREFIX}palette_{band}", CELLS, 1, alpha=False, float_buffer=True)
        image.colorspace_settings.name = "Non-Color"
        image.pixels = [channel for colour in cells for channel in (*colour, 1.0)]
        texture = n.new("ShaderNodeTexImage", image=image, interpolation="Closest")
        colours[band] = texture.outputs["Color"]

    flags = n.new("ShaderNodeSeparateColor")
    tree.links.new(colours.pop("flags"), flags.inputs["Color"])
    colours["hatch"] = flags.outputs["Red"]
    colours["spec"] = flags.outputs["Green"]
    colours["rim"] = flags.outputs["Blue"]
    printed = {"hatch_min": cel_look["hatch"]["duty"]["minimum"], "cross": 1.0}
    _run_group(n, tree, emission, printed, spot, colours)

    return mat


def _shadow(n, tree, output, emission, cel_look, mat):
    """A floor shadow as a key hatch, heaviest under the figure and gone at
    the rim, see-through between its lines and kept out of the knockout."""

    mat.surface_render_method = "BLENDED"
    coordinates = n.new("ShaderNodeTexCoord")
    centred = n.vector("MULTIPLY_ADD", coordinates.outputs["UV"], (2.0, 2.0, 0.0), (-1.0, -1.0, 0.0))
    falloff = n.math("SUBTRACT", 1.0, n.vector("LENGTH", centred), clamp=True)
    geometry = n.new("ShaderNodeNewGeometry")
    across = n.math("MULTIPLY", n.vector("DOT_PRODUCT", geometry.outputs["Position"], (0.7071, 0.7071, 0.0)), 70.0)
    inked = n.math("LESS_THAN", n.math("FRACT", across), n.math("MULTIPLY", falloff, 0.6))
    emission.inputs["Color"].default_value = (*cel_look["key"], 1.0)
    clear = n.new("ShaderNodeBsdfTransparent")
    mix = n.new("ShaderNodeMixShader")
    # Under half its coverage, so the knockout never rings it.
    tree.links.new(n.math("MULTIPLY", inked, 0.48), mix.inputs["Fac"])
    tree.links.new(clear.outputs["BSDF"], mix.inputs[1])
    tree.links.new(emission.outputs["Emission"], mix.inputs[2])
    tree.links.new(mix.outputs["Shader"], output.inputs["Surface"])


def effect(tree, output, mat, glow, strength=1.0, name=None):
    """Draws an effect's material as the game does, tinted `glow`, into a
    cleared `tree`, and says whether `mat` is one.

    The glow is the soft spot of its texture coordinates, added to what is
    behind; the shade is the same spot as darkness, blended over it; the
    light is flat. Unlit, so the same in both looks. `name` is the effect's,
    when `mat` is a copy drawn in its place."""

    name = name or base_name(mat)

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

    n = _Nodes(tree)
    coordinates = n.new("ShaderNodeTexCoord")
    reach = n.vector("LENGTH", n.vector("MULTIPLY_ADD", coordinates.outputs["UV"], (2.0, 2.0, 0.0), (-1.0, -1.0, 0.0)))
    falloff = n.math("SUBTRACT", 1.0, reach, clamp=True)

    return n.math("ADD", n.math("MULTIPLY", n.math("POWER", falloff, 2.2), 0.75), n.math("MULTIPLY", n.math("POWER", falloff, 8.0), 0.25))


# Dressing the scene ---------------------------------------------------------

_ORIGINALS = {}


def _figure(obj, key, default):
    """What `obj`'s figure carries under `key`: the nearest ancestor's."""

    node = obj

    while node is not None:
        if key in node:
            value = node[key]

            return value.to_dict() if hasattr(value, "to_dict") else value

        node = node.parent

    return default


def _figure_light(cel_look, obj):
    return _figure(obj, LIGHT_PROPERTY, cel_look["light"])


def _spot_colour(cel_look, light):
    if light is None:
        # A husk has no spot; its rim and light parts never print it.
        return cel_look["inks"]["paper"]

    return cel_look["lights"][light]


def _meshes():
    return [obj for obj in bpy.data.objects if obj.type == "MESH" and not obj.name.startswith(REVIEW_PREFIX)]


def _remember():
    for obj in _meshes():
        for index, slot in enumerate(obj.material_slots):
            _ORIGINALS.setdefault((obj.name, index), slot.material)


def style(cel_look, look_colours=None):
    """Prints every mesh: each material slot gets its family's printed
    material in its figure's light. `look_colours` maps `body`, `trim` and
    `detail` to a husk's NFT colours (0xRRGGBB). `restore` undoes it."""

    _remember()
    _print_group(cel_look)
    made = {}

    for obj in _meshes():
        light = _figure_light(cel_look, obj)
        spot = _spot_colour(cel_look, light)
        colours_of = _figure(obj, LOOK_PROPERTY, look_colours)

        for index, slot in enumerate(obj.material_slots):
            original = _ORIGINALS.get((obj.name, index))

            if original is None:
                continue

            sources(original)
            name = base_name(original)
            palette = palette_image(original) is not None
            family_key = "palette" if palette else family_name(cel_look, name)
            colours = tuple(sorted((colours_of or {}).items()))
            cache = (family_key, light, colours, name if family_key in EFFECTS else "")

            if cache not in made:
                if palette:
                    made[cache] = _palette_material(cel_look, spot, original)
                else:
                    made[cache] = _print_material(cel_look, family_key, spot, colours_of, original)

            slot.material = made[cache]


def silhouette(cel_look):
    """Every mesh in flat key: the key-only silhouette (6.1)."""

    _remember()
    mat = bpy.data.materials.get(SILHOUETTE_MATERIAL)

    if mat is None:
        mat = bpy.data.materials.new(SILHOUETTE_MATERIAL)
        mat.use_nodes = True
        tree = mat.node_tree
        tree.nodes.clear()
        output = tree.nodes.new("ShaderNodeOutputMaterial")
        emission = tree.nodes.new("ShaderNodeEmission")
        tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])

    mat.node_tree.nodes["Emission"].inputs["Color"].default_value = (*cel_look["key"], 1.0)

    for obj in _meshes():
        for slot in obj.material_slots:
            slot.material = mat


def restore():
    """Puts every material slot back as it was before `style`."""

    for (name, index), mat in _ORIGINALS.items():
        obj = bpy.data.objects.get(name)

        if obj is not None and index < len(obj.material_slots):
            obj.material_slots[index].material = mat

    _ORIGINALS.clear()

    for mat in list(bpy.data.materials):
        if mat.name.startswith(f"{REVIEW_PREFIX}print_") and mat.users == 0:
            bpy.data.materials.remove(mat)


def aim(cel_look, camera, focal, px_per_pt, hatch_per_m):
    """Sets what changes from tile to tile: the light (fixed to `camera`),
    the hatch's density on the model and how many sheet pixels a metre is at
    one metre (`focal`), so the hatch can fade."""

    group = bpy.data.node_groups.get(PRINT_GROUP)

    if group is None:
        return

    direction = light_direction(cel_look, camera)
    node = group.nodes["light"]

    for axis, value in zip("XYZ", direction):
        node.inputs[axis].default_value = value

    group.nodes["focal"].outputs[0].default_value = focal
    group.nodes["hatch_per_m"].outputs[0].default_value = hatch_per_m
    # A device pixel is a third of a point on the phone (DSF 3).
    group.nodes["device"].outputs[0].default_value = 3.0 / max(px_per_pt, 1e-6)


def light_direction(cel_look, camera):
    """The scene's light, in the world: (right, up, toward the viewer) in
    `camera`'s frame."""

    return (camera.matrix_world.to_3x3() @ Vector(cel_look["scene"]["light"])).normalized()


def hatch_per_metre(cel_look, size_m):
    """Hatch lines a metre on a model `size_m` across, so they land
    `hatch_pt` apart at its usual framing (5.2), or the scene's fixed
    `hatch_per_m`."""

    scene = cel_look["scene"]

    if "hatch_per_m" in scene:
        return float(scene["hatch_per_m"])

    return scene["figure_pt"] / (max(size_m, 1e-3) * scene["hatch_pt"])


# Ink ------------------------------------------------------------------------


def _ink_material(cel_look):
    mat = bpy.data.materials.get(INK_MATERIAL) or bpy.data.materials.new(INK_MATERIAL)
    mat.use_nodes = True
    # The hull is inside out: culling the faces that point at the camera
    # leaves only the rim behind the model.
    mat.use_backface_culling = True
    tree = mat.node_tree
    tree.nodes.clear()
    output = tree.nodes.new("ShaderNodeOutputMaterial")
    emission = tree.nodes.new("ShaderNodeEmission")
    emission.inputs["Color"].default_value = (*cel_look["key"], 1)
    tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])

    return mat


def _ink_group(cel_look):
    """The geometry nodes that add the hull (4.1 to 4.4): drop the faces that
    draw no line, weld by position, smooth the push normals over a few
    rings, push out by the baked weight times the light-side term and a
    surface-fixed wobble at the given width, turn inside out, ink, and join
    to the model."""

    group = bpy.data.node_groups.get(INK_GROUP)

    if group is not None:
        bpy.data.node_groups.remove(group)

    group = bpy.data.node_groups.new(INK_GROUP, "GeometryNodeTree")
    interface = group.interface
    interface.new_socket("Geometry", in_out="INPUT", socket_type="NodeSocketGeometry")
    interface.new_socket("Width", in_out="INPUT", socket_type="NodeSocketFloat")
    interface.new_socket("Ink", in_out="INPUT", socket_type="NodeSocketMaterial")
    interface.new_socket("Light", in_out="INPUT", socket_type="NodeSocketVector")
    interface.new_socket("Seed", in_out="INPUT", socket_type="NodeSocketVector")
    interface.new_socket("Rings", in_out="INPUT", socket_type="NodeSocketInt")
    interface.new_socket("Geometry", in_out="OUTPUT", socket_type="NodeSocketGeometry")

    line = cel_look["line"]
    side = line["side"]
    wobble = line["wobble"]
    nodes, links = group.nodes, group.links
    source = nodes.new("NodeGroupInput")
    result = nodes.new("NodeGroupOutput")

    def named(name, data_type):
        node = nodes.new("GeometryNodeInputNamedAttribute")
        node.data_type = data_type
        node.inputs["Name"].default_value = name

        return node.outputs["Attribute"]

    def math(operation, a, b=None, c=None):
        node = nodes.new("ShaderNodeMath")
        node.operation = operation

        for socket, value in zip(node.inputs, (a, b, c)):
            if value is None:
                continue
            if isinstance(value, (int, float)):
                socket.default_value = value
            else:
                links.new(value, socket)

        return node.outputs["Value"]

    drop = nodes.new("GeometryNodeDeleteGeometry")
    drop.domain = "FACE"
    links.new(source.outputs["Geometry"], drop.inputs["Geometry"])
    links.new(named(UNLINED_ATTRIBUTE, "BOOLEAN"), drop.inputs["Selection"])

    weld = nodes.new("GeometryNodeMergeByDistance")
    weld.inputs["Distance"].default_value = line["weld"]
    links.new(drop.outputs["Geometry"], weld.inputs["Geometry"])

    normal = nodes.new("GeometryNodeInputNormal")
    blur = nodes.new("GeometryNodeBlurAttribute")
    blur.data_type = "FLOAT_VECTOR"
    links.new(normal.outputs["Normal"], blur.inputs["Value"])
    links.new(source.outputs["Rings"], blur.inputs["Iterations"])
    smooth = nodes.new("ShaderNodeVectorMath")
    smooth.operation = "NORMALIZE"
    links.new(blur.outputs["Value"], smooth.inputs[0])

    # side = heavy - drop x smoothstep(from, to, N.L)
    facing = nodes.new("ShaderNodeVectorMath")
    facing.operation = "DOT_PRODUCT"
    links.new(smooth.outputs["Vector"], facing.inputs[0])
    links.new(source.outputs["Light"], facing.inputs[1])
    step = nodes.new("ShaderNodeMapRange")
    step.interpolation_type = "SMOOTHSTEP"
    step.clamp = True
    links.new(facing.outputs["Value"], step.inputs["Value"])
    step.inputs["From Min"].default_value = side["from"]
    step.inputs["From Max"].default_value = side["to"]
    side_term = math("MULTIPLY_ADD", step.outputs["Result"], -side["drop"], side["heavy"])

    # wobble = 1 + amount x noise(P x frequency + seed), in -1..1.
    position = nodes.new("GeometryNodeInputPosition")
    seeded = nodes.new("ShaderNodeVectorMath")
    seeded.operation = "MULTIPLY_ADD"
    links.new(position.outputs["Position"], seeded.inputs[0])
    seeded.inputs[1].default_value = (wobble["frequency"],) * 3
    links.new(source.outputs["Seed"], seeded.inputs[2])
    noise = nodes.new("ShaderNodeTexNoise")
    noise.noise_dimensions = "3D"
    noise.inputs["Scale"].default_value = 1.0
    noise.inputs["Detail"].default_value = 0.0
    links.new(seeded.outputs["Vector"], noise.inputs["Vector"])
    wobble_term = math("MULTIPLY_ADD", math("SUBTRACT", noise.outputs["Fac"], 0.5), 2 * wobble["amount"], 1.0)

    weight = math("MULTIPLY", named(ink.ATTRIBUTE, "FLOAT"), named(LINE_ATTRIBUTE, "FLOAT"))
    weight = math("MULTIPLY", weight, side_term)
    weight = math("MULTIPLY", weight, wobble_term)
    weight = math("MULTIPLY", weight, source.outputs["Width"])

    offset = nodes.new("ShaderNodeVectorMath")
    offset.operation = "SCALE"
    links.new(smooth.outputs["Vector"], offset.inputs["Vector"])
    links.new(weight, offset.inputs["Scale"])

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


def _line_attributes(cel_look, obj):
    """Writes the family line weight by vertex, and which faces draw none.
    Returns whether any face draws a line."""

    mesh = obj.data
    materials = [_ORIGINALS.get((obj.name, index), slot.material) for index, slot in enumerate(obj.material_slots)]
    weights = [line_weight(cel_look, base_name(mat)) if mat else 0.0 for mat in materials] or [1.0]
    face_weight = [weights[min(polygon.material_index, len(weights) - 1)] for polygon in mesh.polygons]

    if not any(face_weight):
        return False

    sums = [0.0] * len(mesh.vertices)
    counts = [0] * len(mesh.vertices)

    for polygon, weight in zip(mesh.polygons, face_weight):
        if weight <= 0:
            continue

        for index in polygon.vertices:
            sums[index] += weight
            counts[index] += 1

    for name, domain, data_type, values in (
        (LINE_ATTRIBUTE, "POINT", "FLOAT", [total / count if count else 0.0 for total, count in zip(sums, counts)]),
        (UNLINED_ATTRIBUTE, "FACE", "BOOLEAN", [weight <= 0 for weight in face_weight]),
    ):
        attribute = mesh.attributes.get(name)

        if attribute is not None:
            mesh.attributes.remove(attribute)

        attribute = mesh.attributes.new(name, data_type, domain)
        attribute.data.foreach_set("value", values)

    return True


def add_ink(cel_look):
    """Gives every mesh that should be outlined its hull, with the weights
    the build baked (`ink.read`). Returns them."""

    group = _ink_group(cel_look)
    mat = _ink_material(cel_look)
    rings = cel_look["line"]["rings"].get(cel_look["kind"], 2)
    inked = []

    for obj in _meshes():
        if not obj.material_slots or not obj.data.polygons:
            continue

        if not _line_attributes(cel_look, obj):
            continue

        ink.read(obj, cel_look["kind"])
        modifier = obj.modifiers.new(INK_MODIFIER, "NODES")
        modifier.node_group = group
        modifier[_socket(group, "Ink")] = mat
        modifier[_socket(group, "Rings")] = rings
        checksum = zlib.crc32(base_name(obj).encode())
        modifier[_socket(group, "Seed")] = ((checksum % 997) * 0.37, (checksum // 997 % 991) * 0.41, (checksum // 7 % 983) * 0.29)
        # Framing reads the viewport's evaluation; the hull is the render's.
        modifier.show_viewport = False
        inked.append(obj)

    return inked


def set_ink(cel_look, camera, focal, px_per_pt, shown=True):
    """Sets every hull's width for a tile seen by `camera`: `line_pt` points
    on screen at `px_per_pt` sheet pixels a point, whatever its depth, and
    the light it leans away from."""

    line_px = cel_look["scene"]["line_pt"] * px_per_pt
    light = light_direction(cel_look, camera)
    forward = camera.matrix_world.to_3x3() @ Vector((0, 0, -1))

    for obj in _meshes():
        modifier = obj.modifiers.get(INK_MODIFIER)

        if modifier is None:
            continue

        modifier.show_render = shown

        if not shown or obj.hide_render:
            continue

        centre = obj.matrix_world @ (sum((Vector(corner) for corner in obj.bound_box), Vector()) / 8)
        depth = max((centre - camera.matrix_world.translation).dot(forward), 0.01)
        scale = sum(obj.matrix_world.to_scale()) / 3
        modifier[_socket(modifier.node_group, "Width")] = line_px * depth / focal / max(scale, 1e-6)
        local = (obj.matrix_world.to_3x3().inverted() @ light).normalized()
        modifier[_socket(modifier.node_group, "Light")] = tuple(local)
        # A modifier's inputs set from Python do not tag it for evaluation.
        obj.update_tag()


def remove_ink():
    for obj in bpy.data.objects:
        modifier = obj.modifiers.get(INK_MODIFIER) if obj.type == "MESH" else None

        if modifier is not None:
            obj.modifiers.remove(modifier)

    group = bpy.data.node_groups.get(INK_GROUP)

    if group is not None:
        bpy.data.node_groups.remove(group)


# The knockout ---------------------------------------------------------------


def knockout(pixels, radius, ground, paper, caption_rows=0):
    """Composites a transparent render (`pixels`, rows bottom first, RGBA
    straight, sRGB as saved) over `ground` (linear), with a paper margin
    `radius` pixels wide round
    everything it draws (4.1): the alpha dilated by a soft disc. Shapes under
    half cover (a floor shadow's hatch) and the top `caption_rows` (the
    caption's) are left out of the margin. Returns the opaque pixels."""

    import numpy

    alpha = pixels[:, :, 3]
    solid = numpy.where(alpha >= 0.5, alpha, 0.0)
    # The caption, in the top rows, is not a figure.
    if caption_rows:
        solid[-caption_rows:, :] = 0.0
    grown = solid.copy()
    reach = int(math.ceil(radius + 1))

    for dy in range(-reach, reach + 1):
        for dx in range(-reach, reach + 1):
            distance = math.hypot(dx, dy)
            weight = max(0.0, min(1.0, radius + 0.5 - distance))

            if weight <= 0 or (dx == 0 and dy == 0):
                continue

            shifted = numpy.zeros_like(solid)
            shifted[max(dy, 0) : solid.shape[0] + min(dy, 0), max(dx, 0) : solid.shape[1] + min(dx, 0)] = solid[
                max(-dy, 0) : solid.shape[0] + min(-dy, 0), max(-dx, 0) : solid.shape[1] + min(-dx, 0)
            ]
            numpy.maximum(grown, shifted * weight, out=grown)

    # The saved tile is sRGB, and so is what is put round it.
    result = numpy.empty_like(pixels)
    result[:, :, :3] = numpy.asarray(to_srgb(ground[:3]))
    result[:, :, :3] += (numpy.asarray(to_srgb(paper[:3])) - result[:, :, :3]) * grown[:, :, None]
    result[:, :, :3] += (pixels[:, :, :3] - result[:, :, :3]) * alpha[:, :, None]
    result[:, :, 3] = 1.0

    return result
