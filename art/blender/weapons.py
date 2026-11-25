"""Builds every weapon kind into one GLB.

    cargo xtask build-art weapons --preview

Each kind lives in its own module under `weapon_kinds/` and exposes
`build()`, which returns a node named `weapon_<kind>`, with `<kind>` spelled
as the game's `WeaponKind` spells it. A weapon is modelled with the grip at
the origin and the haft running up +Z, so the game can drop any of them into
a reaper's `socket_weapon`. +X points across the reaper's body, which is
where a blade or a hanging lantern should lean.

A weapon's temper is not a second model. The game's material chars and
hollows the same mesh, so every temper is the same weapon after more has
happened to it.
"""

import argparse
import importlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.append(HERE)

from lib import core, ink, review  # noqa: E402

KINDS = review.WEAPON_KINDS


def main():
    arguments = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    parser.add_argument("--preview")
    parser.add_argument("--cel", action="store_true", help="also render each review sheet in the cel look")
    parser.add_argument("--specs")
    parser.add_argument("--only")
    parser.add_argument("--fast", action="store_true", help="build and render the sheets: no export")
    parser.add_argument("--cel-only", action="store_true", help="render the sheets in the cel look alone")
    parser.add_argument("--scene", help="draw every cel sheet in this scene of cel.toml")
    options = parser.parse_args(arguments)

    if options.fast and not options.preview:
        parser.error("--fast renders sheets instead of writing a model: give it --preview")

    os.makedirs(options.out, exist_ok=True)
    core.reset_scene()

    root = core.empty("weapons")
    built = []

    for kind in KINDS:
        if options.only and options.only != kind:
            continue

        if not os.path.exists(os.path.join(HERE, "weapon_kinds", f"{kind}.py")):
            continue

        weapon = importlib.import_module(f"weapon_kinds.{kind}").build()
        weapon.parent = root
        built.append(weapon)

    for weapon in built:
        print(f"WEAPON {weapon.name} triangles={core.triangle_count(weapon)}")

    if options.fast:
        ink.weigh(root, "weapon")
        print(f"BUILT weapons (fast: not exported) triangles={core.triangle_count(root)}")
    else:
        path = os.path.join(options.out, f"weapon_{options.only}.glb" if options.only else "weapons.glb")
        core.export_glb(path, root)
        print(f"BUILT {path} triangles={core.triangle_count(root)}")

    if options.preview:
        os.makedirs(options.preview, exist_ok=True)

        for look in review.looks(options.cel, options.cel_only, options.scene):
            review.weapons(built, options.preview, look, lineup=not options.only)


if __name__ == "__main__":
    main()
