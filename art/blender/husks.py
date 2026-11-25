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
from mathutils import Vector  # noqa: E402

from lib import core, creature, preview, review  # noqa: E402

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


def reliquary_stack(spec):
    """A tower of ornate reliquary cubes held together by crystalline energy
    seams. Each cube is its own node so the game can let one fall when the seam
    under it breaks."""

    root = core.empty("husk")
    generator = random.Random(21)
    seams = sorted(seal["offset"][1] for seal in spec["seals"])
    gap = 0.16
    spacing = seams[1] - seams[0]
    side = spacing - gap
    half = side / 2
    centres = [seams[0] - spacing / 2 + spacing * index for index in range(len(seams) + 1)]
    frame_mat = core.material("frame", color=(1.0, 1.0, 1.0), roughness=0.42, metallic=0.72, pbr="stone")
    glow_mat = core.material("glow", color=(1.0, 1.0, 1.0), roughness=0.20)

    for index, centre in enumerate(centres):
        cube = core.empty(f"cube_{index}", root, location=(0, 0, centre))
        cube.rotation_euler = (0, 0, generator.uniform(-0.16, 0.16))
        offset = Vector((generator.uniform(-0.07, 0.07), generator.uniform(-0.07, 0.07), 0))

        bpy.ops.mesh.primitive_cube_add(size=side * 0.96)
        core_block = bpy.context.object
        core_block.name = f"block_core_{index}"
        core_block.data.materials.append(frame_mat)
        core_block.location = offset
        core.apply_modifier(core_block, "BEVEL", width=0.045, segments=3)

        frame_parts = [core_block]
        # Top and bottom gothic cornice moldings + 8 armored corner brackets.
        for z_sign in (-1, 1):
            bpy.ops.mesh.primitive_cube_add(size=1.0)
            cornice = bpy.context.object
            cornice.name = f"cornice_{index}_{z_sign}"
            cornice.scale = (side * 1.02, side * 1.02, side * 0.085)
            cornice.location = offset + Vector((0, 0, z_sign * half * 0.91))
            cornice.data.materials.append(frame_mat)
            core.activate(cornice)
            bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
            core.apply_modifier(cornice, "BEVEL", width=0.015, segments=2)
            frame_parts.append(cornice)

        for sx in (-1, 1):
            for sy in (-1, 1):
                bpy.ops.mesh.primitive_cube_add(size=1.0)
                pillar = bpy.context.object
                pillar.name = f"pillar_{index}_{sx}_{sy}"
                pillar.scale = (side * 0.11, side * 0.11, side * 0.99)
                pillar.location = offset + Vector((sx * half * 0.93, sy * half * 0.93, 0))
                pillar.data.materials.append(frame_mat)
                core.activate(pillar)
                bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
                core.apply_modifier(pillar, "BEVEL", width=0.014, segments=2)
                frame_parts.append(pillar)

        block = core.join(frame_parts, f"block_{index}")
        block.parent = cube
        finish(core.shade_smooth(block, angle=42), wear_boost=1.8)

        # An image on each of the four sides, framed cleanly inside the corner pillars.
        for face in range(4):
            angle = math.pi / 2 * face
            outward = Vector((math.sin(angle), -math.cos(angle), 0))
            across = Vector((math.cos(angle), math.sin(angle), 0))
            inset = half * 0.80
            panel = image_grid(
                f"panel_{index}_{face}",
                core.material("canvas"),
                cube,
                offset + outward * (half * 0.98) - across * inset - Vector((0, 0, inset)),
                across * inset * 2,
                (0, 0, inset * 2),
                4,
                4,
            )
            finish_image(panel)

    for index, height in enumerate(seams):
        # Faceted octagonal crystalline energy core between each reliquary block.
        seam = core.revolve(
            f"seam_{index}",
            [
                (0.0, gap * 0.56),
                (side * 0.30, gap * 0.50),
                (side * 0.42, gap * 0.16),
                (side * 0.42, -gap * 0.16),
                (side * 0.30, -gap * 0.50),
                (0.0, -gap * 0.56),
            ],
            glow_mat,
            root,
            segments=8,
            cap_top=True,
            cap_bottom=True,
        )
        seam.location = (0, 0, height)
        core.shade_smooth(seam, angle=35)

    return root


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
    options = parser.parse_args(arguments)

    with open(options.specs, encoding="utf-8") as source:
        specs = json.load(source)

    os.makedirs(options.out, exist_ok=True)

    for name in (*CREATURES, "reliquaryStack"):
        if options.only and options.only != name:
            continue

        spec = specs[name]

        core.reset_scene()
        root = reliquary_stack(spec) if name == "reliquaryStack" else build_creature(name, spec)
        path = os.path.join(options.out, f"husk_{name}.glb")
        core.export_glb(path, root)
        print(f"BUILT {path} triangles={core.triangle_count(root)}")

        looks = spec.get("looks", [])

        if options.preview:
            os.makedirs(options.preview, exist_ok=True)

            for look in review.looks(options.cel):
                review.husk(name, looks, options.preview, creature.TRAITS, cel_look=look)

        if options.portraits and looks:
            os.makedirs(options.portraits, exist_ok=True)

            for look in looks:
                preview.render_portrait(os.path.join(options.portraits, f"{look['key']}.png"), look)

            print(f"BUILT {len(looks)} portraits for {name}")


if __name__ == "__main__":
    main()
