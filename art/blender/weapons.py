"""Builds the six weapon kinds into one GLB.

    dart tool/build_art.dart weapons --preview

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

from lib import core, preview  # noqa: E402

KINDS = ["scythe", "lantern_pole", "balance_flail", "spear", "chain_whip", "bell"]


def main():
    arguments = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    parser.add_argument("--preview")
    parser.add_argument("--specs")
    parser.add_argument("--only")
    options = parser.parse_args(arguments)

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

    if options.preview:
        os.makedirs(options.preview, exist_ok=True)

        # Spread out for the review sheet only; the offsets are cleared below.
        for index, weapon in enumerate(built):
            weapon.location = ((index - (len(built) - 1) / 2) * 1.5, 0, 1.05)

        preview.render_sheet(
            os.path.join(options.preview, f"weapon_{options.only}.png" if options.only else "weapons.png"),
            centre=(0, 0, 1.3),
            distance=9.5,
            width=1500,
            height=840,
            views=("front", "silhouette"),
        )

    for weapon in built:
        weapon.location = (0, 0, 0)

    path = os.path.join(options.out, f"weapon_{options.only}.glb" if options.only else "weapons.glb")
    core.export_glb(path, root)
    print(f"BUILT {path} triangles={core.triangle_count(root)}")


if __name__ == "__main__":
    main()
