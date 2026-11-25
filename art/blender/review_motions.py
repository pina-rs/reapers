"""Renders one pose of an exported clip, from three angles.

    blender --background --python-exit-code 1 --python art/blender/review_motions.py -- \\
      --class harvester --clip finisherReap --phase 0.52 --out art/build/previews/finisher_reap.png

It reads the full-size models a build leaves in `art/build/models` (or
`--models`) and the clips' timings in `art/build/motions.json` (or
`--motions`), so it shows the model as exported, not the authoring scene.
`--phase` is how far through the clip, 0 at its start and 1 at its end;
`--face` frames the face instead of the figure; `--cel` renders it in the
cel look of `art/blender/cel.toml` instead.
"""

import argparse
import os
import sys

import bpy

sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from lib import cel, preview, review  # noqa: E402


def main():
    parser = argparse.ArgumentParser(prog="review_motions.py")
    parser.add_argument("--class", dest="calling", choices=tuple(review.CLASS_WEAPONS), required=True)
    parser.add_argument("--clip", required=True)
    parser.add_argument("--phase", type=float, default=0.5)
    parser.add_argument("--out", required=True)
    parser.add_argument("--face", action="store_true")
    parser.add_argument("--humanoid", action="store_true")
    parser.add_argument("--clothing", choices=("robe", "tunic", "none"), default="robe")
    parser.add_argument("--weapon", choices=review.WEAPON_KINDS)
    parser.add_argument("--variant", type=int, choices=range(review.VARIANTS), default=0)
    parser.add_argument("--cel", action="store_true")
    parser.add_argument("--models", default="art/build/models")
    parser.add_argument("--motions", default="art/build/motions.json")
    options = parser.parse_args(sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else [])

    if not 0 <= options.phase <= 1:
        raise SystemExit(f"--phase is a fraction of the clip, from 0 to 1, not {options.phase}")

    spec = review.motion_spec(options.motions, options.clip)
    form = "humanoid" if options.humanoid else "spirit"
    root, weapons = review.reaper_for_motion(options.models, options.calling)
    review.dress(root, options.variant, form, weapons, review.held_kind(spec, options.calling, options.weapon), "robe" if options.clothing == "none" else options.clothing)

    if options.clothing == "none":
        # The spectral body under the robe, with no outer cloth.
        for node in root.children_recursive:
            name = review.base(node)

            if name.startswith("robe_") or "_sleeve" in name or name.endswith("_cuff"):
                node.hide_render = True

    review.play(("humanoid_" if options.humanoid else "") + options.clip, options.phase)
    os.makedirs(os.path.dirname(os.path.abspath(options.out)), exist_ok=True)
    look = cel.look(cel.load(), "reaper", cel.class_light(cel.load(), options.calling)) if options.cel else None
    tiles = [preview.row(f"{options.calling} {options.clip} {options.phase:.0%}", None, ("front", "threequarter", "side"))]

    if options.face:
        counts = review.variant_counts(root)

        if counts["face_"] == 0:
            raise SystemExit(f"reaper_{options.calling}.glb has no face to frame")

        face = review.find(root, f"face_{review.shown_variant(counts, 'face_', options.variant)}")
        rotation = face.matrix_world.to_quaternion()
        preview.render_grid(options.out, tiles, tile=(420, 560), cel_look=look, framing=(face.matrix_world.translation, 1.5, 0.04), view_rotation=rotation)
    else:
        preview.render_grid(options.out, tiles, tile=(420, 560), cel_look=look)

    print(f"BUILT {options.out}")


if __name__ == "__main__":
    main()
