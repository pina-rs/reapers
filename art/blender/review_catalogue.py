"""Render every exported performance against a floor in five timed poses.

Blender CLI: --class harvester --form humanoid --out art/build/motion-review
The resulting 32 labelled strips are an inspection aid, not a quality score.
"""

import argparse
import json
import os
import sys
import bpy
import numpy

sys.path.insert(0, os.path.dirname(__file__))
from lib import core, preview
from reapers import show_variant, hold_weapon, CLASSES


def activate_clip(name, seconds, phase):
    """Mute all other NLA tracks, including facial tracks, then sample."""
    for obj in bpy.data.objects:
        animations = [obj.animation_data]
        if obj.type == "MESH" and obj.data.shape_keys:
            animations.append(obj.data.shape_keys.animation_data)
        for animation in animations:
            if animation:
                animation.action = None
                for track in animation.nla_tracks:
                    track.mute = track.name != name
    frame = 1 + phase * seconds * bpy.context.scene.render.fps
    bpy.context.scene.frame_set(int(frame), subframe=frame % 1)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--class", dest="calling", default="harvester", choices=CLASSES)
    parser.add_argument("--form", choices=("spirit", "humanoid"), default="humanoid")
    parser.add_argument("--out", required=True)
    parser.add_argument("--only", help="Comma-separated clip names")
    options = parser.parse_args(sys.argv[sys.argv.index("--") + 1 :])
    os.makedirs(options.out, exist_ok=True)
    core.reset_scene()
    bpy.ops.import_scene.gltf(filepath=f"reapers_flutter/assets/models/reaper_{options.calling}.glb")
    root = bpy.data.objects["reaper"]
    show_variant(root, 0, 0, 0)
    for obj in root.children_recursive:
        if obj.name in ("humanoid_form", "spirit_form"):
            hidden = obj.name != options.form + "_form"
            obj.hide_render = hidden
            for child in obj.children_recursive:
                child.hide_render = hidden
        if obj.name.startswith("robe_") and obj.type == "MESH" and options.form == "humanoid":
            obj.data.shape_keys.key_blocks["short_robe"].value = 1
    weapons = {kind: hold_weapon(root, kind) for kind in {CLASSES[options.calling], "spear", "scythe"}}
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.render.resolution_x = 300
    scene.render.resolution_y = 380
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.display.shading.light = "STUDIO"
    scene.display.shading.color_type = "MATERIAL"
    scene.display.shading.show_shadows = True
    scene.display.shading.show_cavity = True
    scene.display.shading.background_type = "WORLD"
    scene.world = bpy.data.worlds.new("ReviewWorld")
    scene.world.color = (0.045, 0.055, 0.07)
    bpy.ops.mesh.primitive_plane_add(size=200)
    floor = bpy.context.object
    floor.name = "review_floor"
    floor.data.materials.append(core.material("floor", color=(0.12, 0.15, 0.18)))
    camera = preview._camera((3.8, -6.5, 2.4), (0, 0, 1.15), lens=58)
    label_data = bpy.data.curves.new("review_label", "FONT")
    label_data.size = 0.1
    label = bpy.data.objects.new("review_label", label_data)
    bpy.context.collection.objects.link(label)
    label.parent = camera
    label.location = (-0.65, 0.79, -3)
    label.data.materials.append(core.material("label", color=(1, 1, 1)))
    specs = json.load(open("art/build/motions.json"))
    wanted = set(options.only.split(",")) if options.only else {spec["name"] for spec in specs}
    unknown = wanted - {spec["name"] for spec in specs}
    if unknown:
        raise ValueError(f"Unknown clips: {sorted(unknown)}")
    strips = []
    for index, spec in enumerate(specs):
        if spec["name"] not in wanted:
            continue
        clip = ("humanoid_" if options.form == "humanoid" else "") + spec["name"]
        phases = [0, 0.15, 0.45, 0.65, 0.85] if not spec["loops"] else [0, 0.125, 0.25, 0.5, 0.75]
        if spec.get("impact") is not None:
            phases[2] = spec["impact"]
        tiles = []
        kind = {"finisherImpale": "spear", "finisherReap": "scythe"}.get(spec["name"], CLASSES[options.calling])
        for phase in phases:
            activate_clip(clip, spec["seconds"], phase)
            for weapon_kind, weapon in weapons.items():
                hidden = spec["unarmed"] or weapon_kind != kind
                weapon.hide_render = hidden
                for child in weapon.children_recursive:
                    child.hide_render = hidden
            bpy.context.view_layer.update()
            label_data.body = f'{index+1:02d} {spec["name"]}\n{phase:.0%}'
            path = os.path.join(options.out, f'{spec["name"]}-{phase}.png')
            scene.render.filepath = path
            if phase == 0:
                # Workbench retains stale shadow buffers after object visibility
                # changes. Prime them before capturing the new performance.
                bpy.ops.render.render()
            bpy.ops.render.render(write_still=True)
            tiles.append(path)
        path = os.path.join(options.out, f'{index+1:02d}-{spec["name"]}.png')
        preview._stitch(tiles, path, 300, 380)
        strips.append(path)
    # Four pages keep all poses readable when reviewing the whole catalogue.
    for start in range(0, len(strips), 8):
        chunk = strips[start : start + 8]
        sheet = numpy.zeros((380 * len(chunk), 1500, 4), dtype=numpy.float32)
        for row, path in enumerate(reversed(chunk)):
            img = bpy.data.images.load(path)
            pixels = numpy.empty(1500 * 380 * 4, dtype=numpy.float32)
            img.pixels.foreach_get(pixels)
            sheet[row * 380 : (row + 1) * 380] = pixels.reshape(380, 1500, 4)
            bpy.data.images.remove(img)
        result = bpy.data.images.new("Catalogue", 1500, 380 * len(chunk), alpha=False)
        result.pixels.foreach_set(sheet.ravel())
        result.filepath_raw = os.path.join(options.out, f"page-{start//8+1}.png")
        result.file_format = "PNG"
        result.save()
    print(f"REVIEWED {len(strips)} clips in {options.out}")


if __name__ == "__main__":
    main()
