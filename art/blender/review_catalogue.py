"""Renders every exported performance against a floor in five timed poses.

    blender --background --python-exit-code 1 --python art/blender/review_catalogue.py -- \\
      --class harvester --form humanoid --out art/build/motion-review

Each clip is a labelled row of five poses (its start, two breakdowns, its
contact and its recovery; quarters for a loop), eight clips to a page, in
`page-<n>.png`. It is an inspection aid, not a quality score: Workbench, so
the whole catalogue renders in a minute, with every surface in its flat
colour from `art/blender/cel.toml`. It reads the full-size models a build
leaves in `art/build/models` (or `--models`).
"""

import argparse
import json
import os
import sys

import bpy

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from lib import cel, preview, review  # noqa: E402

WIDTH, HEIGHT = 300, 380
CLIPS_PER_PAGE = 8


def main():
    parser = argparse.ArgumentParser(prog="review_catalogue.py")
    parser.add_argument("--class", dest="calling", default="harvester", choices=tuple(review.CLASS_WEAPONS))
    parser.add_argument("--form", choices=review.FORMS, default="humanoid")
    parser.add_argument("--out", required=True)
    parser.add_argument("--only", help="comma-separated clip names")
    parser.add_argument("--clothing", choices=("robe", "tunic"), default="tunic")
    parser.add_argument("--rear", action="store_true")
    parser.add_argument("--variant", type=int, choices=range(review.VARIANTS), default=0)
    parser.add_argument("--gaunt", type=float, default=0)
    parser.add_argument("--models", default="art/build/models")
    parser.add_argument("--motions", default="art/build/motions.json")
    options = parser.parse_args(sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else [])
    os.makedirs(options.out, exist_ok=True)

    with open(options.motions, encoding="utf-8") as source:
        specs = json.load(source)

    wanted = set(options.only.split(",")) if options.only else {spec["name"] for spec in specs}
    unknown = wanted - {spec["name"] for spec in specs}

    if unknown:
        raise SystemExit(f"Unknown clips: {', '.join(sorted(unknown))}")

    root, weapons = review.reaper_for_motion(options.models, options.calling)
    review.dress(root, options.variant, options.form, weapons, None, options.clothing)

    for node in root.children_recursive:
        if review.base(node).startswith("robe_") and node.type == "MESH":
            node.data.shape_keys.key_blocks["gaunt"].value = options.gaunt

    _stage(options.rear)
    label = _label()
    rows = []

    for index, spec in enumerate(specs):
        if spec["name"] not in wanted:
            continue

        clip = ("humanoid_" if options.form == "humanoid" else "") + spec["name"]
        phases = [0, 0.125, 0.25, 0.5, 0.75] if spec["loops"] else [0, 0.15, 0.45, 0.65, 0.85]

        if spec.get("impact") is not None:
            phases[2] = spec["impact"]

        review.dress(root, options.variant, options.form, weapons, review.held_kind(spec, options.calling), options.clothing)
        tiles = []

        for phase in phases:
            review.play(clip, phase)
            label.data.body = f"{index + 1:02d} {spec['name']}\n{phase * 100:g}%"
            path = os.path.join(options.out, f"{spec['name']}-{phase}.png")
            bpy.context.scene.render.filepath = path

            if phase == phases[0]:
                # Workbench keeps stale shadow buffers after visibility
                # changes: prime them before the clip's first capture.
                bpy.ops.render.render()

            bpy.ops.render.render(write_still=True)
            tiles.append((path, None))

        rows.append(tiles)

    for start in range(0, len(rows), CLIPS_PER_PAGE):
        preview.compose(rows[start : start + CLIPS_PER_PAGE], os.path.join(options.out, f"page-{start // CLIPS_PER_PAGE + 1}.png"), WIDTH, HEIGHT)

    print(f"REVIEWED {len(rows)} clips in {options.out}")


def _stage(rear):
    """Workbench, a floor, a camera, and every surface in its flat colour."""

    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.render.resolution_x = WIDTH
    scene.render.resolution_y = HEIGHT
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.display.shading.light = "STUDIO"
    scene.display.shading.color_type = "MATERIAL"
    scene.display.shading.show_shadows = True
    scene.display.shading.show_cavity = True
    scene.display.shading.background_type = "WORLD"
    scene.world = bpy.data.worlds.new("ReviewWorld")
    scene.world.color = (0.045, 0.055, 0.07)

    palette = cel.load()

    for mat in bpy.data.materials:
        name = cel.base_name(mat)
        mat.diffuse_color = (*palette["surfaces"].get(name, palette["surfaces"]["fallback"]), 1)

    bpy.ops.mesh.primitive_plane_add(size=200)
    floor = bpy.context.object
    floor.name = "review_floor"
    ground = bpy.data.materials.new("review_floor")
    ground.diffuse_color = (0.12, 0.15, 0.18, 1)
    floor.data.materials.append(ground)
    preview._camera((3.8, 6.5 if rear else -6.5, 2.4), (0, 0, 1.15), lens=58)


def _label():
    data = bpy.data.curves.new("review_label", "FONT")
    data.size = 0.1
    label = bpy.data.objects.new("review_label", data)
    bpy.context.collection.objects.link(label)
    label.parent = bpy.context.scene.camera
    label.location = (-0.65, 0.79, -3)
    # A caption, not a thing on the floor: it casts no shadow (Workbench
    # reads the object's display setting, not its render visibility).
    label.display.show_shadows = False
    mat = bpy.data.materials.new("review_label")
    mat.diffuse_color = (1, 1, 1, 1)
    data.materials.append(mat)

    return label


if __name__ == "__main__":
    main()
