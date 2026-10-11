"""Render contact poses from the exported GLBs, not the authoring scene.

    blender --background --python art/blender/review_motions.py -- \
      --class harvester --clip finisherReap --phase 0.52 --out art/build/contact.png
"""

import argparse
import json
import os
import sys

import bpy

sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from lib import core, preview
from reapers import hold_weapon, show_variant, CLASSES


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--class", dest="calling", choices=CLASSES, required=True)
    parser.add_argument("--clip", required=True)
    parser.add_argument("--phase", type=float, default=0.5)
    parser.add_argument("--out", required=True)
    parser.add_argument("--face", action="store_true")
    parser.add_argument("--humanoid", action="store_true")
    parser.add_argument("--clothing", choices=("robe", "tunic", "none"), default="robe")
    parser.add_argument("--weapon", choices=tuple(CLASSES.values()))
    options = parser.parse_args(sys.argv[sys.argv.index("--") + 1 :])
    clip_name = ("humanoid_" if options.humanoid else "") + options.clip
    core.reset_scene()
    bpy.ops.import_scene.gltf(filepath=f"reapers_flutter/assets/models/reaper_{options.calling}.glb")
    root = bpy.data.objects["reaper"]
    show_variant(root, 0, 0, 0)
    for obj in root.children_recursive:
        if obj.name in ("humanoid_form", "spirit_form"):
            hidden = (obj.name == "humanoid_form") != options.humanoid
            obj.hide_render = hidden
            for child in obj.children_recursive:
                child.hide_render = hidden
        if obj.name == "robe_0":
            obj.hide_render = options.clothing == "none"
            if options.humanoid and options.clothing != "none":
                obj.data.shape_keys.key_blocks["humanoid_" + options.clothing].value = 1
            elif options.clothing == "tunic":
                obj.data.shape_keys.key_blocks["short_robe"].value = 1
        if options.clothing == "none" and ("_sleeve" in obj.name or obj.name.endswith("_cuff")):
            obj.hide_render = True
    with open("art/build/motions.json") as source:
        spec = next(item for item in json.load(source) if item["name"] == options.clip)
    for obj in bpy.data.objects:
        if obj.animation_data:
            obj.animation_data.action = None
            for track in obj.animation_data.nla_tracks:
                track.mute = track.name != clip_name
        if obj.type == "MESH" and obj.data.shape_keys and obj.data.shape_keys.animation_data:
            animation = obj.data.shape_keys.animation_data
            animation.action = None
            for track in animation.nla_tracks:
                track.mute = track.name != clip_name
    # glTF import uses the current scene FPS and puts the first sample at 1.
    frame = 1 + options.phase * spec["seconds"] * bpy.context.scene.render.fps
    bpy.context.scene.frame_set(int(frame), subframe=frame % 1)
    if not spec.get("unarmed", False):
        kind = options.weapon or {"finisherImpale": "spear", "finisherReap": "scythe"}.get(options.clip, CLASSES[options.calling])
        hold_weapon(root, kind)
    # GLB import changes Blender material names; the preview restores its light.
    os.makedirs(os.path.dirname(options.out), exist_ok=True)
    centre = bpy.data.objects["face_0"].matrix_world.translation if options.face else (0, 0, 1.35)
    preview.render_sheet(
        options.out,
        centre=centre,
        distance=1.5 if options.face else 8.5,
        width=420,
        height=560,
        views=("front", "threequarter", "side"),
        eye_height=0.04 if options.face else -0.25,
        view_rotation=bpy.data.objects["face_0"].matrix_world.to_quaternion() if options.face else None,
    )
    print(f"BUILT {options.out}")


if __name__ == "__main__":
    main()
