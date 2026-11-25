"""Builds one GLB for each reaper class.

    cargo xtask build-art reapers --preview

With `--fast` it builds each class's geometry and renders its review
sheets straight from it, without baking the 64 clips or exporting: a
minute or two rather than fifteen (art/blender/README.md, "The fast path").

Each class lives in its own module under `reaper_classes/` and exposes
`build()`, which returns the model's root. Every model has the same node
layout, which is what lets the game assemble any genome from it:

    reaper
      robe_0 .. robe_3        hems; the genome shows one
      mantle_0 .. mantle_2    shoulders; the genome shows one
      head                    pivot at the neck
        head_0 .. head_2      heads; the genome shows one
      arm_r                   pivot at the weapon-side shoulder
        elbow_r
          wrist_r
            socket_weapon     where the weapon's grip sits
      arm_l                   pivot at the other shoulder
        elbow_l
          wrist_l

The animation build inserts a shared waist pivot above these parts and
exports the named clips, described in docs/animations.md.

The classes follow docs/research/dark-games-and-ar-hunting.md: each is told
apart by its head, its shoulder line and its weapon, as a black shape.
"""

import argparse
import importlib
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.append(HERE)

from lib import body, core, face, ink, motion, review  # noqa: E402

# The classes, in build order.
CLASSES = tuple(review.CLASS_WEAPONS)


def main():
    arguments = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    parser.add_argument("--preview")
    parser.add_argument("--cel", action="store_true", help="also render each review sheet in the cel look")
    parser.add_argument("--specs")
    parser.add_argument("--only")
    parser.add_argument("--motions", default="art/build/motions.json")
    parser.add_argument("--fast", action="store_true", help="build the geometry and render the sheets from it: no clips, no export")
    parser.add_argument("--cel-only", action="store_true", help="render the sheets in the cel look alone")
    parser.add_argument("--scene", help="draw every cel sheet in this scene of cel.toml")
    parser.add_argument("--sheets", default=",".join(review.REAPER_SHEETS), help="which sheets: spirit, humanoid, armoury")
    options = parser.parse_args(arguments)
    sheets = tuple(sheet for sheet in options.sheets.split(",") if sheet)

    if options.fast and not options.preview:
        parser.error("--fast renders sheets instead of writing a model: give it --preview")

    with open(options.motions) as source:
        motions = motion.variants(json.load(source))

    os.makedirs(options.out, exist_ok=True)

    for name in CLASSES:
        if options.only and options.only != name:
            continue

        if not os.path.exists(os.path.join(HERE, "reaper_classes", f"{name}.py")):
            continue

        core.reset_scene()
        root = importlib.import_module(f"reaper_classes.{name}").build()
        body.build(root, name)
        faces = face.build(root, name)

        if options.fast:
            # The geometry as it would ship, posed in each form's first idle
            # key by hand rather than baked into 64 clips and exported.
            joints, rest = motion.rig(root)
            ink.weigh(root, "reaper")
            print(f"BUILT reaper_{name} (fast: no clips, not exported) triangles={core.triangle_count(root)}")
            os.makedirs(options.preview, exist_ok=True)

            def stance(form, joints=joints, rest=rest, name=name):
                motion.hold(joints, rest, name, "idle", 0.0, form == "humanoid")

            for look in review.looks(options.cel, options.cel_only, options.scene):
                review.reaper_sheets(root, name, options.preview, look, stance, sheets)

            continue

        motion.build(root, name, motions)
        face.bake(faces, motions)
        path = os.path.join(options.out, f"reaper_{name}.glb")
        core.export_glb(path, root, animations=True)
        print(f"BUILT {path} triangles={core.triangle_count(root)}")

        if options.preview:
            # From the model as exported, at rest: the authoring scene is
            # left wherever the clip bake ended.
            os.makedirs(options.preview, exist_ok=True)

            for look in review.looks(options.cel, options.cel_only, options.scene):
                review.reaper(path, name, options.preview, look, sheets)


if __name__ == "__main__":
    main()
