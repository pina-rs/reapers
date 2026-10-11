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
    options = parser.parse_args(sys.argv[sys.argv.index("--") + 1 :])
    core.reset_scene()
    bpy.ops.import_scene.gltf(filepath=f"reapers_flutter/assets/models/reaper_{options.calling}.glb")
    root = bpy.data.objects["reaper"]
    show_variant(root, 0, 0, 0)
    with open("art/build/motions.json") as source:
        spec = next(item for item in json.load(source) if item["name"] == options.clip)
    for obj in bpy.data.objects:
        if obj.animation_data:
            obj.animation_data.action = None
            for track in obj.animation_data.nla_tracks:
                track.mute = track.name != options.clip
    # glTF import uses the current scene FPS and puts the first sample at 1.
    frame = 1 + options.phase * spec["seconds"] * bpy.context.scene.render.fps
    bpy.context.scene.frame_set(int(frame), subframe=frame % 1)
    if not spec.get("unarmed", False):
        hold_weapon(root, CLASSES[options.calling])
    # GLB import changes Blender material names; the preview restores its light.
    os.makedirs(os.path.dirname(options.out), exist_ok=True)
    preview.render_sheet(options.out, centre=(0, 0, 1.35), distance=8.5, width=420, height=560, views=("front", "threequarter", "side"))
    print(f"BUILT {options.out}")


if __name__ == "__main__":
    main()
