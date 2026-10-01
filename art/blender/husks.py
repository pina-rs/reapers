"""Builds one GLB for each vessel: the body a dead NFT wears as a husk.

    dart tool/build_art.dart husks --preview

A vessel is an original shape that carries any square image, so a husk looks
like the NFT it came from without copying a collection's art. The sizes and
the seal positions come from the rules (`Vessel` in reapers_shared), passed
in as `--specs`; nothing here invents a number the rules depend on.

A husk is modelled facing -Y, like a reaper. In the rules' terms that makes
x "right as the player sees it", z "up" and -y "toward the player".

Material names pick the surface in the game, as they do for reapers:

    frame    wood, gilt and iron: whatever holds the image
    canvas   a flat surface showing the image once, edge to edge
    paper    a surface the image wraps around, lit from within
    shard    a mirror fragment showing a piece of the image
    ribbon   a strip the image repeats along
    core     the dark heart
    glow     the husk's own light
    shroud   spectral cloth and smoke, drawn translucent

Image-bearing meshes keep real texture coordinates. Everything else is
finished the same way a reaper's parts are.
"""

import argparse
import json
import math
import os
import random
import sys

sys.path.append(os.path.dirname(os.path.abspath(__file__)))

import bmesh  # noqa: E402
import bpy  # noqa: E402
from mathutils import Matrix, Vector, noise  # noqa: E402

from lib import anatomy, cloth, core, preview  # noqa: E402


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


def moulding(name, mat, parent, half_width, half_height, centre_z, profile):
    """A mitred picture-frame moulding around a rectangular opening.

    `profile` is a list of (outward, forward) pairs: how far each step of
    the moulding sits outside the opening and how far it stands toward the
    viewer.
    """

    mesh = bmesh.new()
    loops = []

    for outward, forward in profile:
        loops.append(
            [
                mesh.verts.new((x * (half_width + outward), -forward, centre_z + z * (half_height + outward)))
                for x, z in ((-1, -1), (1, -1), (1, 1), (-1, 1))
            ]
        )

    for inner, outer in zip(loops, loops[1:]):
        for index in range(4):
            follow = (index + 1) % 4
            mesh.faces.new((inner[index], inner[follow], outer[follow], outer[index]))

    result = core.object_from_bmesh(name, mesh, mat, parent, smooth=False)

    return core.apply_modifier(result, "BEVEL", width=0.004, segments=2, limit_method="ANGLE")


# Frame Wraith -----------------------------------------------------------


def frame_wraith(spec):
    """A cracked, floating picture frame trailing torn cloth, gripped from
    behind by two hands of smoke. The image is the canvas."""

    root = core.empty("husk")
    frame = core.material("frame")
    shroud = core.material("shroud")
    opening = 0.36
    centre = 0.12

    finish(
        moulding(
            "moulding",
            frame,
            root,
            opening,
            opening,
            centre,
            [(-0.012, 0.0), (0.0, 0.03), (0.018, 0.048), (0.05, 0.05), (0.062, 0.03), (0.088, 0.046), (0.108, 0.022), (0.11, -0.035), (-0.012, -0.035)],
        ),
        wear_boost=2.2,
    )

    # A pointed crest over the top rail, so the frame reads as a reliquary
    # rather than something off a living-room wall.
    crest = core.plate(
        "crest",
        [(-0.3, 0.0), (-0.2, 0.05), (-0.11, 0.035), (-0.05, 0.1), (0.0, 0.2), (0.05, 0.1), (0.11, 0.035), (0.2, 0.05), (0.3, 0.0)],
        0.045,
        frame,
        root,
        bevel=0.006,
    )
    crest.location = (0, 0, centre + opening + 0.1)
    finish(crest, wear_boost=2.2)

    def bulge(point, s, t):
        swell = math.sin(s * math.pi) * math.sin(t * math.pi)

        return point + Vector((0, -0.012 * swell + 0.004 * noise.noise(point * 9), 0))

    finish_image(
        image_grid(
            "canvas",
            core.material("canvas"),
            root,
            (-opening, -0.004, centre - opening),
            (opening * 2, 0, 0),
            (0, 0, opening * 2),
            12,
            12,
            warp=bulge,
        )
    )

    # A nail at each seal, so there is something to strike at.
    nails = []

    for index, seal in enumerate(spec["seals"]):
        position = to_blender(seal["offset"])

        nails.append(
            core.revolve(
                f"nail_{index}",
                [(0.0, 0.016), (0.014, 0.012), (0.021, 0.0), (0.008, -0.004), (0.006, -0.03), (0.0, -0.03)],
                core.material("frame"),
                segments=10,
            )
        )
        nails[-1].rotation_euler = (math.radians(90), 0, 0)
        nails[-1].location = (position.x, -0.05, position.z)

    for nail in nails:
        nail.parent = root
        finish(nail, wear_boost=2.5)

    # Torn cloth hanging from the bottom rail.
    for layer, (drop, seed, depth) in enumerate(((0.62, 3, 0.0), (0.44, 9, -0.03))):
        hanging = cloth.drape(
            f"shroud_{layer}",
            shroud,
            root,
            [(-0.44 + 0.88 * step / 16, depth + 0.012 * math.sin(step * 1.7), centre - opening - 0.1) for step in range(17)],
            drop=drop,
            width_gain=0.72,
            tatter=0.75,
            folds=9,
            fold_depth=0.05,
            sag=0.06,
            seed=seed,
            outward=(0, 1, 0),
        )
        finish(hanging)

    # Two hands of smoke grip the frame from behind.
    for side in (-1, 1):
        hand = anatomy.hand(f"hand_{side}", root, grip_radius=0.055, curl=0.82, side=side, scale=1.7)

        for slot in hand.material_slots:
            slot.material = shroud

        hand.location = (side * (opening + 0.07), 0.0, centre + 0.06)
        finish(hand)

        wrist = Vector((side * (opening + 0.07), 0.2, centre + 0.06))
        arm = core.tube(
            f"arm_{side}",
            core.bezier([wrist, wrist + Vector((side * 0.1, 0.22, -0.18)), wrist + Vector((side * 0.05, 0.4, -0.55))], samples=8),
            lambda t: 0.034 * (1 - t) ** 0.7 + 0.002,
            shroud,
            root,
            sides=10,
        )
        finish(arm)

    return root


# Lantern Shade ----------------------------------------------------------


def lantern_shade(spec):
    """A paper lantern turning around a flame. The image wraps the paper,
    and the seals are tears that only show while they face the player."""

    root = core.empty("husk")
    frame = core.material("frame")
    radius = 0.3
    half = 0.36
    wraps = 3
    segments = 72
    rows = 20

    seal_angles = [math.atan2(to_blender(seal["offset"]).y, to_blender(seal["offset"]).x) for seal in spec["seals"]]
    seal_heights = [to_blender(seal["offset"]).z for seal in spec["seals"]]

    def torn(angle, height):
        """Whether the paper has been ripped away here."""

        for seal_angle, seal_height in zip(seal_angles, seal_heights):
            around = abs((angle - seal_angle + math.pi) % math.tau - math.pi) * radius
            along = (height - seal_height) * 0.5
            ragged = 0.03 * noise.noise(Vector((angle * 5, height * 9, 3)))

            if math.hypot(around, along) < 0.07 + ragged:
                return True

        return False

    mesh = bmesh.new()
    layer = mesh.loops.layers.uv.new("image")
    grid = []

    for row in range(rows + 1):
        t = row / rows
        height = -half + 2 * half * t
        # A lantern swells between its ribs.
        swell = radius * (1 + 0.1 * math.sin(t * math.pi) + 0.012 * math.cos(t * math.pi * 10))

        grid.append(
            [
                (mesh.verts.new((math.cos(angle) * swell, math.sin(angle) * swell, height)), column / segments, t, angle, height)
                for column in range(segments + 1)
                for angle in [math.tau * column / segments - math.pi / 2]
            ]
        )

    for row in range(rows):
        for column in range(segments):
            corners = [grid[row][column], grid[row][column + 1], grid[row + 1][column + 1], grid[row + 1][column]]
            mid_angle = (corners[0][3] + corners[1][3]) / 2
            mid_height = (corners[0][4] + corners[3][4]) / 2

            if torn(mid_angle, mid_height):
                continue

            face = mesh.faces.new([corner[0] for corner in corners])

            for loop, (_, s, t, _, _) in zip(face.loops, corners):
                loop[layer].uv = (s * wraps, t)

    bmesh.ops.remove_doubles(mesh, verts=mesh.verts, dist=1e-5)
    loose = [vertex for vertex in mesh.verts if not vertex.link_faces]
    bmesh.ops.delete(mesh, geom=loose, context="VERTS")
    finish_image(core.object_from_bmesh("paper", mesh, core.material("paper"), root))

    # Hoops at top and bottom, and thin ribs between.
    for name, height, size in (("hoop_top", half, 0.02), ("hoop_bottom", -half, 0.02), ("rib_high", half * 0.4, 0.007), ("rib_low", -half * 0.4, 0.007)):
        ring_radius = radius * (1 + 0.1 * math.sin((height + half) / (2 * half) * math.pi)) + 0.004
        hoop = core.tube(
            name,
            [(math.cos(math.tau * step / 48) * ring_radius, math.sin(math.tau * step / 48) * ring_radius, height) for step in range(49)],
            size,
            frame,
            root,
            sides=8,
            cap=False,
        )
        finish(hoop, wear_boost=2.0)

    for name, height, direction in (("cap_top", half, 1), ("cap_bottom", -half, -1)):
        cap = core.revolve(
            name,
            [(radius * 0.98, height), (radius * 0.6, height + direction * 0.05), (0.05, height + direction * 0.075), (0.0, height + direction * 0.08)],
            frame,
            root,
            segments=32,
        )
        finish(cap, wear_boost=1.6)

    # A handle to hang it by, though nothing holds it now.
    finish(
        core.tube(
            "handle",
            core.bezier([(-0.2, 0, half + 0.03), (-0.17, 0, half + 0.2), (0.0, 0, half + 0.29), (0.17, 0, half + 0.2), (0.2, 0, half + 0.03)], samples=8),
            0.009,
            frame,
            root,
            sides=8,
        ),
        wear_boost=2.0,
    )

    core.sculpt(
        "flame",
        [("sphere", (0, 0, -0.05), (0.075, 0.075, 0.17)), ("sphere", (0, 0, 0.12), (0.04, 0.04, 0.12))],
        core.material("glow"),
        root,
        voxel=0.012,
    )

    # Torn paper and a tassel trail underneath.
    generator = random.Random(4)

    for strip in range(7):
        angle = math.tau * strip / 7 + generator.uniform(-0.2, 0.2)
        start = Vector((math.cos(angle) * radius * 0.85, math.sin(angle) * radius * 0.85, -half - 0.04))
        width = generator.uniform(0.05, 0.1)
        tangent = Vector((-math.sin(angle), math.cos(angle), 0)) * width

        finish(
            cloth.drape(
                f"strip_{strip}",
                core.material("shroud"),
                root,
                [start - tangent + tangent * 2 * step / 4 for step in range(5)],
                drop=generator.uniform(0.25, 0.55),
                width_gain=0.5,
                tatter=0.7,
                folds=2,
                fold_depth=0.02,
                sag=0.03,
                seed=strip,
            )
        )

    return root


# Shard Swarm ------------------------------------------------------------


def shard(name, mat, parent, centre, size, seed, extent):
    """One jagged fragment of mirror, showing the part of the image that
    would lie behind it if the swarm were a single pane."""

    generator = random.Random(seed)
    points = generator.randint(5, 7)
    mesh = bmesh.new()
    layer = mesh.loops.layers.uv.new("image")
    centre = Vector(centre)
    front, back = [], []

    for index in range(points):
        angle = math.tau * index / points + generator.uniform(-0.3, 0.3)
        reach = size * generator.uniform(0.45, 1.0)
        corner = centre + Vector((math.cos(angle) * reach, 0, math.sin(angle) * reach))

        front.append(mesh.verts.new(corner + Vector((0, -0.006, 0))))
        back.append(mesh.verts.new(corner + Vector((0, 0.006, 0))))

    faces = [mesh.faces.new(front), mesh.faces.new(list(reversed(back)))]

    for index in range(points):
        follow = (index + 1) % points
        faces.append(mesh.faces.new((front[index], back[index], back[follow], front[follow])))

    for face in faces:
        for loop in face.loops:
            loop[layer].uv = (0.5 + loop.vert.co.x / (2 * extent), 0.5 + loop.vert.co.z / (2 * extent))

    result = core.object_from_bmesh(name, mesh, mat, parent, smooth=False)
    # Move the geometry so the shard turns about its own middle.
    result.data.transform(Matrix.Translation(-centre))
    result.location = centre

    return finish_image(result)


def shard_swarm(spec):
    """Mirror shards circling a dark core. The image is split across the
    shards, and the seals ride on the three largest."""

    root = core.empty("husk")
    mat = core.material("shard")
    extent = spec["halfWidth"]

    heart = core.sculpt(
        "core",
        [("sphere", (0, 0, 0), (0.15, 0.13, 0.16)), ("sphere", (0.07, 0.02, 0.06), (0.09, 0.09, 0.09)), ("sphere", (-0.06, -0.01, -0.07), (0.1, 0.09, 0.08))],
        core.material("core"),
        root,
        voxel=0.014,
        smooth=1,
    )
    core.roughen(heart, 0.02, 9.0, seed=2)
    finish(heart)

    core.sculpt("spark", [("sphere", (0, -0.1, 0), (0.035, 0.035, 0.035))], core.material("glow"), root, voxel=0.008)

    # The game turns this node to match the rules, so the seal shards are
    # placed at the angle the rules call zero.
    orbit = core.empty("orbit", root)

    for index, seal in enumerate(spec["seals"]):
        if seal["orbitRadius"] <= 0:
            continue

        angle = seal["orbitPhase"]
        shard(
            f"seal_shard_{index}",
            mat,
            orbit,
            (math.cos(angle) * seal["orbitRadius"], -0.03, math.sin(angle) * seal["orbitRadius"]),
            0.2,
            seed=index,
            extent=extent,
        )

    drift = core.empty("drift", root)
    generator = random.Random(11)

    for index in range(11):
        angle = generator.uniform(0, math.tau)
        reach = generator.uniform(0.24, extent * 0.95)
        piece = shard(
            f"shard_{index}",
            mat,
            drift,
            (math.cos(angle) * reach, generator.uniform(-0.1, 0.18), math.sin(angle) * reach),
            generator.uniform(0.07, 0.14),
            seed=20 + index,
            extent=extent,
        )
        piece.rotation_euler = (generator.uniform(-0.5, 0.5), generator.uniform(-0.3, 0.3), generator.uniform(-0.5, 0.5))

    return root


# Scroll Eel -------------------------------------------------------------


def scroll_eel(spec):
    """A long printed ribbon that swims. The image repeats along it."""

    root = core.empty("husk")
    frame = core.material("frame")
    head = 0.62
    tail = -0.85
    length = head - tail
    repeats = 3
    columns = 96
    rows = 8

    def half_width(s):
        """The ribbon is widest behind the head and tapers to the tail."""

        return 0.17 * (0.3 + 0.7 * s**0.6)

    mesh = bmesh.new()
    layer = mesh.loops.layers.uv.new("image")
    grid = []

    for row in range(rows + 1):
        t = row / rows
        line = []

        for column in range(columns + 1):
            s = column / columns
            x = tail + length * s
            z = (t - 0.5) * 2 * half_width(s)

            line.append((mesh.verts.new((x, 0, z)), s, t))

        grid.append(line)

    for row in range(rows):
        for column in range(columns):
            corners = [grid[row][column], grid[row][column + 1], grid[row + 1][column + 1], grid[row + 1][column]]
            face = mesh.faces.new([corner[0] for corner in corners])

            for loop, (_, s, t) in zip(face.loops, corners):
                loop[layer].uv = (s * repeats, t)

    ribbon = core.object_from_bmesh("ribbon", mesh, core.material("ribbon"), root)

    # Two shape keys a quarter-wave apart. The game blends them with a sine
    # and a cosine, which makes the wave travel down the ribbon.
    def wave(phase):
        def move(co):
            s = min(1.0, max(0.0, (co.x - tail) / length))
            swing = 0.11 * (1 - s) ** 0.8 + 0.015

            co.y += swing * math.sin(s * math.tau * 1.6 + phase)
            co.z += 0.02 * math.sin(s * math.tau * 2.3 + phase) * (1 - s)

            return co

        return move

    core.add_shape_key(ribbon, "wave_a", wave(0.0))
    core.add_shape_key(ribbon, "wave_b", wave(math.pi / 2))
    finish_image(ribbon)

    # The head: the scroll's roller, with a knob at each end.
    roller = core.revolve(
        "roller",
        [(0.0, 0.26), (0.03, 0.255), (0.05, 0.23), (0.03, 0.205), (0.022, 0.19), (0.045, 0.18), (0.045, -0.18), (0.022, -0.19), (0.03, -0.205), (0.05, -0.23), (0.03, -0.255), (0.0, -0.26)],
        frame,
        root,
        segments=24,
    )
    roller.location = (head + 0.02, 0, 0)
    finish(roller, wear_boost=2.0)

    core.sculpt(
        "eye",
        [("sphere", (head + 0.02, -0.05, 0.08), (0.02, 0.02, 0.026))],
        core.material("glow"),
        root,
        voxel=0.006,
        smooth=0,
    )

    # Frayed threads off the tail.
    generator = random.Random(8)

    for thread in range(9):
        start = Vector((tail, 0, generator.uniform(-0.05, 0.05)))
        finish(
            core.tube(
                f"thread_{thread}",
                core.bezier(
                    [start, start + Vector((-0.12, generator.uniform(-0.05, 0.05), generator.uniform(-0.04, 0.04))), start + Vector((-0.3, generator.uniform(-0.1, 0.1), generator.uniform(-0.12, 0.08)))],
                    samples=6,
                ),
                lambda t: 0.004 * (1 - t) + 0.0008,
                core.material("shroud"),
                root,
                sides=5,
            )
        )

    return root


# Reliquary Stack --------------------------------------------------------


def reliquary_stack(spec):
    """A tower of image cubes held together by light. Each cube is its own
    node, so the game can let one fall when the seam under it breaks."""

    root = core.empty("husk")
    generator = random.Random(21)
    seams = sorted(seal["offset"][1] for seal in spec["seals"])
    gap = 0.16
    spacing = seams[1] - seams[0]
    side = spacing - gap
    half = side / 2
    centres = [seams[0] - spacing / 2 + spacing * index for index in range(len(seams) + 1)]

    for index, centre in enumerate(centres):
        cube = core.empty(f"cube_{index}", root, location=(0, 0, centre))
        cube.rotation_euler = (0, 0, generator.uniform(-0.16, 0.16))
        offset = Vector((generator.uniform(-0.07, 0.07), generator.uniform(-0.07, 0.07), 0))

        bpy.ops.mesh.primitive_cube_add(size=side * 0.985)
        block = bpy.context.object
        block.name = f"block_{index}"
        block.data.materials.append(core.material("frame"))
        block.location = offset
        block.parent = cube
        core.apply_modifier(block, "BEVEL", width=0.05, segments=3)
        finish(block, wear_boost=1.8)

        # An image on each of the four sides, set just proud of the block.
        for face in range(4):
            angle = math.pi / 2 * face
            outward = Vector((math.sin(angle), -math.cos(angle), 0))
            across = Vector((math.cos(angle), math.sin(angle), 0))
            inset = half * 0.86
            panel = image_grid(
                f"panel_{index}_{face}",
                core.material("canvas"),
                cube,
                offset + outward * (half * 0.99) - across * inset - Vector((0, 0, inset)),
                across * inset * 2,
                (0, 0, inset * 2),
                4,
                4,
            )
            finish_image(panel)

    for index, height in enumerate(seams):
        bpy.ops.mesh.primitive_cube_add(size=1)
        seam = bpy.context.object
        seam.name = f"seam_{index}"
        seam.scale = (side * 0.8, side * 0.8, gap * 0.9)
        seam.location = (0, 0, height)
        seam.data.materials.append(core.material("glow"))
        seam.parent = root
        core.activate(seam)
        bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)

    return root


VESSELS = {
    "frameWraith": frame_wraith,
    "lanternShade": lantern_shade,
    "shardSwarm": shard_swarm,
    "scrollEel": scroll_eel,
    "reliquaryStack": reliquary_stack,
}


def main():
    arguments = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    parser.add_argument("--specs", required=True)
    parser.add_argument("--preview")
    parser.add_argument("--only")
    options = parser.parse_args(arguments)

    with open(options.specs, encoding="utf-8") as source:
        specs = json.load(source)

    os.makedirs(options.out, exist_ok=True)

    for name, build in VESSELS.items():
        if options.only and options.only != name:
            continue

        core.reset_scene()
        root = build(specs[name])
        path = os.path.join(options.out, f"husk_{name}.glb")
        core.export_glb(path, root)
        print(f"BUILT {path} triangles={core.triangle_count(root)}")

        if options.preview:
            os.makedirs(options.preview, exist_ok=True)
            spec = specs[name]
            preview.render_sheet(
                os.path.join(options.preview, f"husk_{name}.png"),
                centre=(0, 0, 0),
                distance=max(spec["halfHeight"], spec["halfWidth"]) * 6.5,
            )


if __name__ == "__main__":
    main()
