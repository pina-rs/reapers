"""Builds one GLB for each reaper class.

    dart tool/build_art.dart reapers --preview

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

from lib import body, core, face, motion, preview  # noqa: E402

# Each class and the weapon kind it is shown holding in a review sheet.
CLASSES = {
    "harvester": "scythe",
    "ferryman": "lantern_pole",
    "weigher": "balance_flail",
    "chooser": "spear",
    "hollow": "chain_whip",
    "mourner": "bell",
}


def main():
    arguments = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    parser.add_argument("--preview")
    parser.add_argument("--specs")
    parser.add_argument("--only")
    parser.add_argument("--motions", default="art/build/motions.json")
    options = parser.parse_args(arguments)
    with open(options.motions) as source:
        motions = motion.variants(json.load(source))

    os.makedirs(options.out, exist_ok=True)

    for name, weapon in CLASSES.items():
        if options.only and options.only != name:
            continue

        if not os.path.exists(os.path.join(HERE, "reaper_classes", f"{name}.py")):
            continue

        core.reset_scene()
        root = importlib.import_module(f"reaper_classes.{name}").build()
        body.build(root, name)
        faces = face.build(root, name)
        motion.build(root, name, motions)
        face.bake(faces, motions)
        path = os.path.join(options.out, f"reaper_{name}.glb")
        core.export_glb(path, root, animations=True)
        print(f"BUILT {path} triangles={core.triangle_count(root)}")

        if options.preview:
            os.makedirs(options.preview, exist_ok=True)
            hold_weapon(root, weapon)

            for variant in range(3):
                show_variant(root, head=variant, mantle=variant, robe=variant)
                preview.render_sheet(os.path.join(options.preview, f"reaper_{name}_{variant}.png"))


def hold_weapon(root, kind):
    """Puts the class's weapon in its hand, for the review sheet only."""

    if not os.path.exists(os.path.join(HERE, "weapon_kinds", f"{kind}.py")):
        return

    socket = find(root, "socket_weapon")
    weapon = importlib.import_module(f"weapon_kinds.{kind}").build()
    weapon.parent = socket
    return weapon


def find(node, name):
    if node.name == name:
        return node

    for child in node.children:
        found = find(child, name)

        if found is not None:
            return found

    return None


def show_variant(root, head, mantle, robe):
    """Hides every variant but the chosen ones, as the game would."""

    def visit(node):
        if node.name == "humanoid_form":
            hide_tree(node, True)
        for prefix, chosen in (("head_", head), ("mantle_", mantle), ("robe_", robe)):
            if node.name.startswith(prefix) and node.name[len(prefix) :].isdigit():
                hide_tree(node, int(node.name[len(prefix) :]) != chosen)

        for child in node.children:
            visit(child)

    def hide_tree(node, hidden):
        node.hide_render = hidden

        for child in node.children:
            hide_tree(child, hidden)

    visit(root)


if __name__ == "__main__":
    main()
