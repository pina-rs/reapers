"""Builds one GLB for each reaper class.

    cargo xtask build-art reapers --preview

With `--fast` it builds each class's geometry and renders its review
sheets straight from it, without baking the 64 clips or exporting: a
minute or two rather than fifteen (art/blender/README.md, "The fast path").
`--fast --lineup` builds every class (or those `--only` names, comma-
separated) into one scene and renders them side by side instead.

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

from lib import body, cel, core, face, figure, ink, motion, review  # noqa: E402

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
    parser.add_argument("--lineup", action="store_true", help="with --fast: the classes side by side, instead of each one's sheets")
    options = parser.parse_args(arguments)
    sheets = tuple(sheet for sheet in options.sheets.split(",") if sheet)
    wanted = set(options.only.split(",")) if options.only else set(CLASSES)

    if wanted - set(CLASSES):
        parser.error(f"no class called {', '.join(sorted(wanted - set(CLASSES)))}")

    if options.fast and not options.preview:
        parser.error("--fast renders sheets instead of writing a model: give it --preview")

    if options.lineup and not options.fast:
        parser.error("--lineup builds the classes fast: give it --fast (review.py --lineup reads built models)")

    with open(options.motions) as source:
        motions = motion.variants(json.load(source))

    os.makedirs(options.out, exist_ok=True)
    classes = [name for name in CLASSES if name in wanted and os.path.exists(os.path.join(HERE, "reaper_classes", f"{name}.py"))]

    if options.lineup:
        lineup(classes, options)
        return

    for name in classes:
        core.reset_scene()
        root, faces = build(name)

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


def build(name):
    """Builds class `name` into the scene, as every path ships it. Returns
    its root and its faces."""

    root = importlib.import_module(f"reaper_classes.{name}").build()
    figure.grow_hands(root)
    body.build(root, name)

    return root, face.build(root, name)


def lineup(classes, options):
    """`lineup_reapers.png` (with `--scene`, `lineup_reapers_<scene>.png`):
    `classes` built fast into one scene, each in its first idle key, and set
    side by side (`review.lay_out`)."""

    palette = cel.load()
    core.reset_scene()
    figures = []

    for index, name in enumerate(classes):
        review.step_aside(index)
        root, _ = build(name)
        joints, rest = motion.rig(root)
        ink.weigh(root, "reaper")
        print(f"BUILT reaper_{name} (fast: no clips, not exported) triangles={core.triangle_count(root)}")
        review.line_up_reaper(root, name, palette, lambda form, joints=joints, rest=rest, name=name: motion.hold(joints, rest, name, "idle", 0.0, form == "humanoid"))
        figures.append((f"reaper_{name}", root))

    os.makedirs(options.preview, exist_ok=True)
    stem = "lineup_reapers" + (f"_{options.scene}" if options.scene else "")
    review.lay_out(figures, "reaper", options.preview, palette, options.scene, stem)


if __name__ == "__main__":
    main()
