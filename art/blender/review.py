"""Renders the review sheets again from models already built, without
rebuilding them.

    blender --background --python-exit-code 1 --python art/blender/review.py -- \\
      --out art/build/previews [--cel] [--only reaper_harvester,weapons]

It reads the full-size models a build leaves in `art/build/models` (or
`--models`), and for creatures the looks in `art/build/vessels.json` (or
`--specs`), which `cargo xtask build-art --specs-only` writes. The sheets are
the ones the drivers render with `--preview`; `--cel` adds each sheet in the
cel look of `art/blender/cel.toml`, so the art direction can be changed and
judged in a minute rather than a rebuild. See art/blender/README.md.
"""

import argparse
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.append(HERE)

import bpy  # noqa: E402

from lib import core, creature, review  # noqa: E402
from viewmodel import MOMENTS  # noqa: E402


def main():
    arguments = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser(prog="review.py")
    parser.add_argument("--models", default="art/build/models")
    parser.add_argument("--specs", default="art/build/vessels.json")
    parser.add_argument("--out", required=True)
    parser.add_argument("--cel", action="store_true", help="also render each sheet in the cel look")
    parser.add_argument("--only", help="comma-separated model names, without .glb")
    options = parser.parse_args(arguments)

    names = sorted(file[: -len(".glb")] for file in os.listdir(options.models) if file.endswith(".glb"))
    wanted = set(options.only.split(",")) if options.only else None

    if wanted is not None and wanted - set(names):
        raise SystemExit(f"No built model called {', '.join(sorted(wanted - set(names)))} in {options.models}")

    os.makedirs(options.out, exist_ok=True)

    for name in names:
        if wanted is not None and name not in wanted:
            continue

        path = os.path.join(options.models, f"{name}.glb")

        for look in review.looks(options.cel):
            sheet(name, path, options, look)


def sheet(name, path, options, look):
    if name.startswith("reaper_"):
        review.reaper(path, name.removeprefix("reaper_"), options.out, look)
        return

    core.reset_scene()
    review.import_glb(path)
    review.rest_pose()

    if name == "weapons" or name.startswith("weapon_"):
        roots = sorted(bpy.data.objects["weapons"].children, key=_kind_order)
        review.weapons(roots, options.out, look, lineup=name == "weapons")
    elif name == "viewmodel":
        review.first_person(MOMENTS, options.out, cel_look=look)
    elif name.startswith("viewmodel_"):
        review.whole(options.out, name, cel_look=look)
    elif name.startswith("husk_"):
        vessel = name.removeprefix("husk_")

        with open(options.specs, encoding="utf-8") as source:
            looks = json.load(source)[vessel].get("looks", [])

        review.husk(vessel, looks, options.out, creature.TRAITS, cel_look=look)
    else:
        raise SystemExit(f"{name}.glb is no model a driver writes")


def _kind_order(weapon):
    kind = review.snake(review.base(weapon).removeprefix("weapon_"))

    return review.WEAPON_KINDS.index(kind) if kind in review.WEAPON_KINDS else len(review.WEAPON_KINDS)


if __name__ == "__main__":
    main()
