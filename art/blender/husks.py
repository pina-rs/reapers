"""Builds one GLB for each vessel: the body a dead NFT wears as a husk.

    cargo xtask build-art husks --preview

Every supported collection has a creature of its own, built by the module of
the same name in `creature_kinds/`. The sizes and the seal positions come
from the rules (`Vessel` in `reapers_rules`), passed in as `--specs`; nothing
here invents a number the rules depend on.

After a creature is built, one portrait is rendered for every NFT of its
collection, dressed in that NFT's own colours. Those portraits are the NFTs'
pictures in the game, so no collection's artwork is ever used.

The one vessel that is not a creature is the reliquary stack, the body of a
rift: a tower of cubes that wears a whole collection's portraits.

See README.md for the conventions a creature has to follow.
"""

import argparse
import importlib
import json
import math
import os
import random
import sys

sys.path.append(os.path.dirname(os.path.abspath(__file__)))

import bmesh  # noqa: E402
import bpy  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

from lib import cel, core, creature, ink, preview, review  # noqa: E402

# Vessels built by a module in creature_kinds/, in build order.
CREATURES = ("ape", "bird", "penguin", "llama", "robot")


def finish(obj, **masks):
    """Paints masks on a part that carries no image."""

    return core.paint_masks(obj, **masks)


def finish_image(obj):
    """Paints masks on an image-bearing part, leaving its UVs alone."""

    return core.paint_masks(obj, rest_position=False)


def to_blender(offset):
    """A position in the rules' frame (x right, y up, z toward the player)
    as a Blender position."""

    return Vector((offset[0], -offset[2], offset[1]))


def image_grid(name, mat, parent, origin, across, up, columns, rows, uv=(0.0, 0.0, 1.0, 1.0), warp=None):
    """A flat sheet showing part of the image.

    The sheet starts at `origin` and spans the vectors `across` and `up`.
    `uv` is the (u0, v0, u1, v1) rectangle of the image it shows, and
    `warp(point, s, t)` may move each point, for cloth and paper that is not
    quite flat.
    """

    mesh = bmesh.new()
    layer = mesh.loops.layers.uv.new("image")
    origin, across, up = Vector(origin), Vector(across), Vector(up)
    grid = []

    for row in range(rows + 1):
        line = []

        for column in range(columns + 1):
            s, t = column / columns, row / rows
            point = origin + across * s + up * t

            line.append((mesh.verts.new(warp(point, s, t) if warp else point), s, t))

        grid.append(line)

    for row in range(rows):
        for column in range(columns):
            corners = [grid[row][column], grid[row][column + 1], grid[row + 1][column + 1], grid[row + 1][column]]
            face = mesh.faces.new([corner[0] for corner in corners])

            for loop, (_, s, t) in zip(face.loops, corners):
                loop[layer].uv = (uv[0] + (uv[2] - uv[0]) * s, uv[1] + (uv[3] - uv[1]) * t)

    return core.object_from_bmesh(name, mesh, mat, parent)


# Reliquary Stack --------------------------------------------------------


# Each storey, bottom first: its crate's half-width, how far it has slid
# along x, and how far it is rolled about y and turned about z (radians).
# The tower leans to its left as it climbs and every crate sits a little
# askew on the one below, so it reads as something piled up, not built.
STOREYS = (
    (1.0, 0.0, 0.0, 0.06),
    (0.87, 0.1, -0.035, -0.2),
    (0.94, 0.24, 0.04, 0.28),
    (0.84, 0.42, -0.035, -0.1),
    (0.64, 0.6, 0.05, 0.22),
)
# The top storey is a casket, smaller than the crates below it, and its
# gabled roof rises this far over its eaves, with finials this long at the
# gables. The casket's own height is whatever the rules' half-height leaves
# above the top seam once the roof is on.
CASKET_LIP = 0.1
CASKET_RISE = 0.42
CASKET_FINIAL = 0.16
# The ridge's finials stand this far above the ridge line.
CASKET_CREST = 0.06
# How far the tower may stand above its half-height, and how far out from
# its axis it may reach, in half-widths, before the build refuses it.
STACK_OVER = 0.05
STACK_REACH = 1.5
# How tall the gap between two crates is, which the seam fills, and how
# thick a crate's frame members are.
SEAM_GAP = 0.34
MEMBER = 0.16
# The farthest a seam's centre strays from the seals' line.
SEAM_ASIDE = 0.25
# How deep a portrait sits behind the face of its frame.
RECESS = 0.07
# The seals are wax bosses scaled up to the tower's size.
STACK_SEAL_RADIUS = 0.16
STACK_SEAL_HEIGHT = 0.06


def reliquary_stack(spec):
    """A leaning tower of reliquaries: four picture-frame crates piled askew
    and a smaller casket with a gabled roof on top, each wearing a portrait
    on every side, held apart by seams of crystal that are its seals.

    Every crate is its own node, `cube_<storey>`, holding its frame
    (`block_<storey>`) and its four portrait faces (`panel_<storey>_<face>`),
    so the game can let one fall when the seam under it breaks. The seams
    (`seam_<index>`) and their seal bosses (`seal_<index>`) hang from the
    root, at the rules' seal heights, and stay where they are."""

    root = core.empty("husk")
    seams = sorted(seal["offset"][1] for seal in spec["seals"])
    spacing = seams[1] - seams[0]
    centres = [seams[0] - spacing / 2 + spacing * index for index in range(len(seams) + 1)]
    frame = core.material("frame", color=(1.0, 1.0, 1.0), roughness=0.42, metallic=0.72, pbr="stone")
    glow = core.material("glow", color=(1.0, 1.0, 1.0), roughness=0.20)
    canvas = core.material("canvas")
    half_height = (spacing - SEAM_GAP) / 2
    floor = -spec["halfHeight"]
    top = len(centres) - 1
    # The casket stands on the top seam and stops where its roof brings the
    # tower up to its half-height.
    casket_floor = centres[top] - half_height
    roof = CASKET_LIP + CASKET_RISE + CASKET_CREST + CASKET_FINIAL
    casket_half_height = (spec["halfHeight"] - roof - casket_floor) / 2

    for index, (centre, (half, slide, roll, turn)) in enumerate(zip(centres, STOREYS)):
        tall = half_height

        if index == top:
            tall = casket_half_height
            centre = casket_floor + tall

        cube = core.empty(f"cube_{index}", root, location=(slide, 0, centre))
        cube.rotation_euler = (0, roll, turn)
        parts = _crate(index, half, tall, frame)

        if index == 0:
            parts.append(_box("plinth", (-half - 0.1, -half - 0.1, floor - centre), (half + 0.1, half + 0.1, -tall), frame))

        if index == top:
            parts.extend(_casket(half, tall, frame))

        block = core.join(parts, f"block_{index}")
        block.parent = cube
        core.crisp(block, 0.03, angle=30, segments=1)
        finish(block, wear_boost=1.8)

        for face in range(4):
            _portrait(index, face, half, tall, canvas, cube)

    # Each seam's flat front stands as far forward as its seal.
    front = max(seal["offset"][2] for seal in spec["seals"])
    made = []

    for index, height in enumerate(seams):
        # Centred between the crates it parts, but never so far aside that
        # its seal slips off its flat front.
        between = (STOREYS[index][1] + STOREYS[index + 1][1]) / 2
        aside = max(-SEAM_ASIDE, min(SEAM_ASIDE, between))
        made.append(_seam(index, Vector((aside, 0, height)), front, glow, root))

    creature.studs(root, spec, radius=STACK_SEAL_RADIUS, height=STACK_SEAL_HEIGHT, body=made)

    for index, gap in enumerate(creature.seal_gaps(root, spec, body=made)):
        print(f"SEAL seal_{index} stands {gap * 1000:+.0f} mm off its seam")

        if not -STACK_SEAL_HEIGHT / 2 <= gap <= STACK_SEAL_HEIGHT * 0.75:
            raise RuntimeError(f"reliquaryStack: seal_{index} is {gap * 1000:+.0f} mm off its seam; widen the seam so its front meets the seal")

    _check_stack_bounds(root, spec)

    return root


def _check_stack_bounds(root, spec):
    """Refuses a tower that has outgrown the body the rules give it: the
    room clears `halfHeight` above its middle and the hunt keeps to a
    footprint of `halfWidth`, so a crown or a shard past them would stand
    through a ceiling or a wall the game thinks is clear."""

    bpy.context.view_layer.update()
    high, reach, farthest = -math.inf, 0.0, None

    for node in root.children_recursive:
        if node.type != "MESH":
            continue

        for vertex in node.data.vertices:
            at = node.matrix_world @ vertex.co
            high = max(high, at.z)

            if max(abs(at.x), abs(at.y)) > reach:
                reach, farthest = max(abs(at.x), abs(at.y)), node.name

    print(f"BOUNDS reliquaryStack top {high:.2f} m (half-height {spec['halfHeight']:.2f}), reach {reach:.2f} m ({reach / spec['halfWidth']:.2f} half-widths, at {farthest})")

    if high > spec["halfHeight"] + STACK_OVER:
        raise RuntimeError(f"reliquaryStack: its top is at z = {high:.2f}, over its half-height of {spec['halfHeight']:.2f}; lower the casket roof")

    if reach > STACK_REACH * spec["halfWidth"]:
        raise RuntimeError(f"reliquaryStack: {farthest} reaches {reach:.2f} m from the tower's axis, past {STACK_REACH} half-widths; shorten it or the lean")


def _box(name, low, high, mat):
    """An axis-aligned box from corner `low` to corner `high`."""

    mesh = bmesh.new()
    bmesh.ops.create_cube(mesh, size=1.0)
    low, high = Vector(low), Vector(high)

    for vertex in mesh.verts:
        vertex.co = Vector(low[axis] + (vertex.co[axis] + 0.5) * (high[axis] - low[axis]) for axis in range(3))

    return core.object_from_bmesh(name, mesh, mat, smooth=False)


def _turned(obj, quarter):
    """Turns a part about the crate's upright axis by `quarter` quarter turns."""

    obj.data.transform(Matrix.Rotation(math.pi / 2 * quarter, 4, "Z"))

    return obj


def _crate(index, half, tall, mat):
    """A crate built like four picture frames boxed together: a square post
    at each upright edge, a rail top and bottom on each face, a liner
    sloping in from the rails to the portrait, and a carcass behind."""

    inner = half - MEMBER
    parts = [_box(f"carcass_{index}", (-half + RECESS, -half + RECESS, -tall + 0.02), (half - RECESS, half - RECESS, tall - 0.02), mat)]

    # A lid and a skirt, overhanging all round: the step in the outline
    # that says crate, storey by storey.
    for way in (-1, 1):
        lip = half + 0.08
        parts.append(_box(f"lid_{index}_{way}", (-lip, -lip, way * tall - 0.06), (lip, lip, way * tall + 0.06), mat))

    for sx in (-1, 1):
        for sy in (-1, 1):
            # Posts stand a little proud of the rails, so each edge of the
            # tower is a column that catches its own band.
            proud = 0.03
            low = (min(sx * (half + proud), sx * inner), min(sy * (half + proud), sy * inner), -tall)
            high = (max(sx * (half + proud), sx * inner), max(sy * (half + proud), sy * inner), tall)
            parts.append(_box(f"post_{index}_{sx}_{sy}", low, high, mat))

    for face in range(4):
        for way in (-1, 1):
            low = (-inner, -half, way * tall - (MEMBER if way > 0 else 0))
            high = (inner, -half + MEMBER, way * tall + (0 if way > 0 else MEMBER))
            parts.append(_turned(_box(f"rail_{index}_{face}_{way}", low, high, mat), face))

        # The liner: a sloped frame from the rails' inner edge down to the
        # portrait, the plane that takes the light round each picture.
        out_w, out_h = inner, tall - MEMBER
        in_w, in_h = inner - 0.07, tall - MEMBER - 0.07
        outer = [(-out_w, -half, -out_h), (out_w, -half, -out_h), (out_w, -half, out_h), (-out_w, -half, out_h)]
        deep = -half + RECESS
        inside = [(-in_w, deep, -in_h), (in_w, deep, -in_h), (in_w, deep, in_h), (-in_w, deep, in_h)]
        parts.append(_turned(core.loft(f"liner_{index}_{face}", [inside, outer], mat), face))

    return parts


def _portrait(index, face, half, tall, mat, cube):
    """The portrait face on side `face` of crate `index`: a square sheet in
    the frame's opening, the whole of one portrait across it (UV 0 to 1,
    upright as seen from outside). Face 0 looks to the front (-Y), 1 to the
    tower's left (+X), 2 to the back and 3 to its right."""

    angle = math.pi / 2 * face
    outward = Vector((math.sin(angle), -math.cos(angle), 0))
    across = Vector((math.cos(angle), math.sin(angle), 0))
    size = min(half - MEMBER, tall - MEMBER) - 0.07
    panel = image_grid(
        f"panel_{index}_{face}",
        mat,
        cube,
        outward * (half - RECESS + 0.004) - across * size - Vector((0, 0, size)),
        across * size * 2,
        (0, 0, size * 2),
        4,
        4,
    )
    finish_image(panel)


def _casket(half, tall, mat):
    """The top crate's lid: a gabled casket roof, its ridge running front to
    back so the gable faces whoever looks up at it, with finials along the
    ridge."""

    eave = half + 0.14
    rise = CASKET_RISE
    lip = CASKET_LIP
    base = tall
    roof = [(-eave, base), (eave, base), (eave, base + lip), (0.0, base + lip + rise), (-eave, base + lip)]
    rings = [[(x, y, z) for x, z in roof] for y in (-eave, eave)]
    parts = [core.loft("roof", rings, mat, cap_start=True, cap_end=True)]

    # Slabs laid on the roof, overhanging the gables, so each pitch is one
    # broad plane with a thick edge.
    for side in (-1, 1):
        ridge = Vector((0, 0, base + lip + rise))
        eaves = Vector((side * (eave + 0.08), 0, base + lip - 0.08 * rise / eave))
        thick = 0.09
        normal = Vector((side * rise, 0, eave)).normalized()
        section = [ridge, eaves, eaves + normal * thick, ridge + normal * thick]
        rings = [[point + Vector((0, y, 0)) for point in section] for y in (-eave - 0.08, eave + 0.08)]
        parts.append(core.loft(f"pitch_{side}", rings, mat, cap_start=True, cap_end=True))

    crest = base + lip + rise + CASKET_CREST

    for step in range(5):
        y = -eave + 2 * eave * step / 4
        gable = step in (0, 4)
        size = CASKET_FINIAL if gable else CASKET_FINIAL * 0.5
        parts.append(_spike(f"finial_{step}", Vector((0, y, crest)), Vector((0, 0, 1)), size, 0.08 if gable else 0.05, mat))

    return parts


def _spike(name, at, direction, length, width, mat, sides=4):
    """A crystal or finial: a bipyramid from `at` out along `direction`."""

    profile = [(0.0, length), (width, length * 0.22), (width * 0.8, 0.0), (0.0, -length * 0.08)]
    spike = core.revolve(name, profile, mat, segments=sides, cap_top=True, cap_bottom=True)
    spike.data.transform(Vector((0, 0, 1)).rotation_difference(Vector(direction).normalized()).to_matrix().to_4x4())
    spike.data.transform(Matrix.Translation(at))

    return spike


def _seam(index, centre, front, mat, parent):
    """A seam between two crates: a band of crystal, octagonal and flat at
    the front, `front` from the tower's axis, where the seal sits, its edges
    sloping into the crates, and shards of crystal jutting from it to the
    sides and back."""

    def ring(apothem, z):
        reach = apothem / math.cos(math.pi / 8)
        points = []

        for corner in range(8):
            angle = math.pi / 8 + math.pi / 4 * corner
            points.append(centre + Vector((math.sin(angle) * reach, -math.cos(angle) * reach, z)))

        return points

    rings = [ring(0.72, SEAM_GAP * 0.62), ring(front, 0.09), ring(front, -0.09), ring(0.72, -SEAM_GAP * 0.62)]
    parts = [core.loft(f"seam_band_{index}", rings, mat, cap_start=True, cap_end=True)]
    generator = random.Random(40 + index)

    for corner, (bearing, length) in enumerate(((55, 0.72), (100, 0.52), (150, 0.62), (210, 0.58), (255, 0.66), (305, 0.7))):
        bearing = math.radians(bearing + generator.uniform(-12, 12))
        tilt = generator.uniform(-0.45, 0.55)
        direction = Vector((math.sin(bearing), -math.cos(bearing), tilt)).normalized()
        start = centre + Vector((math.sin(bearing), -math.cos(bearing), 0.0)) * 0.74
        parts.append(_spike(f"shard_{index}_{corner}", start, direction, length, 0.16, mat, sides=5))

    seam = core.join(parts, f"seam_{index}")
    seam.parent = parent
    core.shade_smooth(seam, angle=30)

    return seam


# Portraits --------------------------------------------------------------

# A specimen plate: this many pixels square, with a paper margin this share
# of its side round the window the husk stands in, the window's rule this
# many pixels wide, and the window washed with the NFT's backdrop this far
# from paper toward its ghost.
PLATE_PX = 384
PLATE_MARGIN = 0.06
PLATE_RULE_PX = 2
PLATE_WASH = 0.5
# Every plate is seen from the same place: a three-quarter view from a
# little above, the husk filling the window with this much to spare.
PLATE_VIEW = "threequarter"
PLATE_ELEVATION = 7.0
PLATE_LENS = 70
PLATE_FILL = 1.12


def specimen_plates(root, looks, directory, palette):
    """Renders every NFT's portrait as a specimen plate in Second Ink (art
    bible 3.6 and 12.5): the husk in its ghost wash, with its key line and
    paper knockout, in a window washed with its backdrop's ghost, on paper,
    with a registration mark at each corner. These are the pictures the game
    shows for the NFTs.

    Every plate of a collection is framed alike, from the creature wearing
    everything it can, so the eight read as one set at thumbnail size. The
    seals are left out; they belong to the hunt."""

    cel_look = cel.look(palette, "husk")
    ink.weigh(root, "husk")
    seals = [node for node in root.children_recursive if node.name.startswith("seal_")]

    for seal in seals:
        seal.hide_render = True

    framing, size = _plate_framing(root)

    try:
        for look in looks:
            path = os.path.join(directory, f"{look['key']}.png")
            tile = preview.row("", None, (PLATE_VIEW,), look=look)
            plate_look = {**cel_look, "vigil": _plate_wash(cel_look, look["backdrop"])}
            preview.render_grid(
                path,
                [tile],
                tile=(PLATE_PX, PLATE_PX),
                cel_look=plate_look,
                lens=PLATE_LENS,
                elevation=PLATE_ELEVATION,
                framing=framing,
                labels=False,
                figure_size=size,
            )
            _plate_frame(path, cel_look)
    finally:
        for seal in seals:
            seal.hide_render = False


def _plate_framing(root):
    """Where every plate's camera looks and how far back it stands, and how
    big the figure is: from the bounds of the husk wearing all it can, but
    neither its seals nor its floor shadow."""

    meshes = [node for node in root.children_recursive if node.type == "MESH"]
    shown = {node: node.hide_render for node in meshes}

    for node in meshes:
        node.hide_render = node.name.startswith("seal_")

    bpy.context.view_layer.update()
    corners = preview.visible_corners(("shadow", "seal_"))
    points = [node.matrix_world @ vertex.co for node in meshes if not node.hide_render and not node.name.startswith("shadow") for vertex in node.data.vertices]

    for node, hidden in shown.items():
        node.hide_render = hidden

    low = Vector(map(min, *corners))
    high = Vector(map(max, *corners))
    turn, rise = math.radians(preview.ANGLES[PLATE_VIEW]), math.radians(PLATE_ELEVATION)
    back = Vector((math.sin(turn) * math.cos(rise), -math.cos(turn) * math.cos(rise), math.sin(rise)))
    right = Vector((0, 0, 1)).cross(back).normalized()
    up = back.cross(right).normalized()
    # Centre the figure as it is seen, not its bounds: a bird's swept wings
    # fill one side of its box and leave the other empty.
    middle = (low + high) / 2
    across = [(point - middle).dot(right) for point in points]
    upward = [(point - middle).dot(up) for point in points]
    centre = middle + right * (min(across) + max(across)) / 2 + up * (min(upward) + max(upward)) / 2
    # The window is the plate less its margins; the 36 mm sensor spans it.
    half_view = 18 / PLATE_LENS * (1 - 2 * PLATE_MARGIN)
    distance = max(
        max(offset.dot(back) + abs(offset.dot(right)) * PLATE_FILL / half_view, offset.dot(back) + abs(offset.dot(up)) * PLATE_FILL / half_view)
        for offset in (point - centre for point in points)
    )
    extent = high - low

    return (tuple(centre), distance, None), max(extent.z, max(extent.x, extent.y) * 0.75)


def _plate_wash(cel_look, backdrop):
    """The window's ground: the backdrop's ghost (3.6), half printed on the
    paper, as the linear colour a render's ground takes."""

    (lit, _, _), _ = cel.ghost(cel_look, backdrop)
    paper = cel_look["inks"]["paper"]

    return cel.to_linear(tuple(blank + (ghost - blank) * PLATE_WASH for blank, ghost in zip(paper, lit)))


def _plate_frame(path, cel_look):
    """Prints the plate round a rendered window at `path`: a paper margin,
    a key rule round the window, and a registration mark (a ring and a
    cross) at each of its corners."""

    import numpy

    image = bpy.data.images.load(path)
    width, height = image.size
    pixels = numpy.empty(width * height * 4, dtype=numpy.float32)
    image.pixels.foreach_get(pixels)
    bpy.data.images.remove(image)
    pixels = pixels.reshape(height, width, 4)

    # A saved render holds sRGB, and so do the inks.
    paper = numpy.asarray(cel_look["inks"]["paper"], dtype=numpy.float32)
    key = numpy.asarray(cel_look["inks"]["key"], dtype=numpy.float32)
    margin = round(PLATE_PX * PLATE_MARGIN)
    rows, columns = numpy.mgrid[0:height, 0:width].astype(numpy.float32) + 0.5

    inside = (columns >= margin) & (columns <= width - margin) & (rows >= margin) & (rows <= height - margin)
    pixels[~inside, :3] = paper

    def ink(cover):
        """Lays key over the plate wherever `cover` (0 to 1) says."""

        cover = numpy.clip(cover, 0.0, 1.0)[:, :, None]
        pixels[:, :, :3] += (key - pixels[:, :, :3]) * cover

    # The rule: a band PLATE_RULE_PX wide, straddling the window's edge.
    edge = numpy.minimum.reduce([abs(columns - margin), abs(columns - (width - margin)), abs(rows - margin), abs(rows - (height - margin))])
    within = (columns >= margin - PLATE_RULE_PX) & (columns <= width - margin + PLATE_RULE_PX) & (rows >= margin - PLATE_RULE_PX) & (rows <= height - margin + PLATE_RULE_PX)
    ink(numpy.where(within, PLATE_RULE_PX / 2 + 0.5 - edge, 0.0))

    # The registration marks, centred in the margin off each corner.
    reach = margin * 0.34
    stroke = 0.6

    for x in (margin / 2, width - margin / 2):
        for y in (margin / 2, height - margin / 2):
            dx, dy = columns - x, rows - y
            ring = stroke + 0.5 - abs(numpy.hypot(dx, dy) - reach * 0.55)
            cross = numpy.maximum(
                numpy.where(abs(dx) <= reach, stroke + 0.5 - abs(dy), 0.0),
                numpy.where(abs(dy) <= reach, stroke + 0.5 - abs(dx), 0.0),
            )
            ink(numpy.maximum(ring, cross))

    pixels[:, :, 3] = 1.0
    result = bpy.data.images.new("Plate", width, height, alpha=False)
    result.pixels.foreach_set(pixels.ravel())
    result.filepath_raw = path
    result.file_format = "PNG"
    result.save()
    bpy.data.images.remove(result)


def build(name, spec):
    return reliquary_stack(spec) if name == "reliquaryStack" else build_creature(name, spec)


def lineup(names, specs, options):
    """`lineup_husks.png` (with `--scene`, `lineup_husks_<scene>.png`):
    the husks `names` built fast into one scene, each in its collection's
    first look, and set side by side on one floor (`review.lay_out`)."""

    palette = cel.load()
    core.reset_scene()
    figures = []

    for index, name in enumerate(names):
        review.step_aside(index)
        root = build(name, specs[name])
        ink.weigh(root, "husk")
        print(f"BUILT husk_{name} (fast: not exported) triangles={core.triangle_count(root)}")
        review.line_up_husk(root, specs[name].get("looks", []))
        figures.append((f"husk_{name}", root))

    os.makedirs(options.preview, exist_ok=True)
    stem = "lineup_husks" + (f"_{options.scene}" if options.scene else "")
    review.lay_out(figures, "husk", options.preview, palette, options.scene, stem)


def build_creature(name, spec):
    module = importlib.import_module(f"creature_kinds.{name}")
    root = module.build(spec)

    creature.check(root, spec)

    return root


def main():
    arguments = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    parser.add_argument("--specs", required=True)
    parser.add_argument("--preview")
    parser.add_argument("--cel", action="store_true", help="also render each review sheet in the cel look")
    parser.add_argument("--portraits")
    parser.add_argument("--only")
    parser.add_argument("--fast", action="store_true", help="build and render the sheets: no export, no portraits")
    parser.add_argument("--cel-only", action="store_true", help="render the sheets in the cel look alone")
    parser.add_argument("--scene", help="draw every cel sheet in this scene of cel.toml")
    parser.add_argument("--lineup", action="store_true", help="with --fast: the husks side by side, instead of each one's sheet")
    options = parser.parse_args(arguments)
    vessels = (*CREATURES, "reliquaryStack")
    wanted = set(options.only.split(",")) if options.only else set(vessels)

    if wanted - set(vessels):
        parser.error(f"no husk called {', '.join(sorted(wanted - set(vessels)))}")

    if options.fast and not options.preview:
        parser.error("--fast renders sheets instead of writing a model: give it --preview")

    if options.lineup and not options.fast:
        parser.error("--lineup builds the husks fast: give it --fast (review.py --lineup reads built models)")

    with open(options.specs, encoding="utf-8") as source:
        specs = json.load(source)

    os.makedirs(options.out, exist_ok=True)
    chosen = [name for name in vessels if name in wanted]

    if options.lineup:
        # The reliquary stack is a tower several storeys high: beside it at
        # one scale the creatures would be specks, so it lines up only when
        # named.
        lineup(chosen if options.only else list(CREATURES), specs, options)
        return

    for name in chosen:
        spec = specs[name]

        core.reset_scene()
        root = build(name, spec)

        if options.fast:
            ink.weigh(root, "husk")
            print(f"BUILT husk_{name} (fast: not exported) triangles={core.triangle_count(root)}")
        else:
            path = os.path.join(options.out, f"husk_{name}.glb")
            core.export_glb(path, root)
            print(f"BUILT {path} triangles={core.triangle_count(root)}")

        looks = spec.get("looks", [])

        if options.preview:
            os.makedirs(options.preview, exist_ok=True)

            for look in review.looks(options.cel, options.cel_only, options.scene):
                review.husk(name, looks, options.preview, creature.TRAITS, cel_look=look)

        if options.portraits and looks and not options.fast:
            os.makedirs(options.portraits, exist_ok=True)
            specimen_plates(root, looks, options.portraits, cel.load())
            print(f"BUILT {len(looks)} portraits for {name}")


if __name__ == "__main__":
    main()
