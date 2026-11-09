"""Builds what the reaper sees of itself in a hunt: its arms and weapons,
the soul capture orb, and the light they throw.

    cargo xtask build-art viewmodel

Three models come out of it:

- `viewmodel.glb`, everything the reaper sees of itself, under `viewmodel`.
  The right arm, `vm_right`, holds the grave pistol (`weapon_gravePistol`),
  the bone knife (`weapon_boneKnife`) and the scythe (`weapon_scythe`) in
  `socket_weapon`, and the game shows one. The left arm, `vm_left`, carries
  the spare soul drum (`viewmodel_soul_cylinder`) or the capture orb
  (`viewmodel_orb_held`) at `socket_palm`. A heavy swing's trail is
  `vm_trail`.
- `viewmodel_orb.glb`, the orb as it flies and lies in the world
  (`capture_orb`).
- `viewmodel_fx.glb`, every light the world shows: halos, sparks, the ring
  wave, the soul stream and the tracer, under `vmfx`.

Nothing here is animated. Every part is modelled at rest about its pivot
and the game moves, turns, shows and fades it every frame (leptoskit's part
offsets), so a hit-stop can hold any pose and a bench can freeze any frame.

Everything a frame might show is drawn every frame close to the eye, so each
moving part is one mesh with one material, painted from a palette texture
(`viewmodel_parts/common.py`), and the worst moment of every action is
checked against the budget below.
"""

import argparse
import importlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.append(HERE)

from lib import core, preview  # noqa: E402

# The most triangles the first-person view may show at once.
TRIANGLE_BUDGET = 6000

# What each action shows at its busiest, by mesh.
MOMENTS = {
    "fire": ["arm_r_forearm", "arm_r", "pistol", "pistol_drum", "vm_flash_1"],
    "reload": ["arm_r_forearm", "arm_r", "pistol", "pistol_drum", "arm_l", "soul_cylinder"],
    "throw": ["arm_r_forearm", "arm_r", "pistol", "pistol_drum", "arm_l", "orb_held"],
    "throw with the scythe": ["arm_r_forearm", "arm_r", "scythe", "arm_l", "orb_held"],
    "stab": ["arm_r_forearm", "arm_r", "knife"],
    "swing": ["arm_r_forearm", "arm_r", "scythe", "vm_trail"],
}

ORB_TRIANGLE_BUDGET = 3000
FX_TRIANGLE_BUDGET = 1500


def main():
    arguments = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    parser.add_argument("--preview")
    parser.add_argument("--specs")
    options = parser.parse_args(arguments)

    os.makedirs(options.out, exist_ok=True)
    _build_viewmodel(options)
    _build_orb(options)
    _build_fx(options)


def _part(name):
    return importlib.import_module(f"viewmodel_parts.{name}")


def _build_viewmodel(options):
    core.reset_scene()

    root = core.empty("viewmodel")
    arms = _part("arms")
    spin = arms.build_right(root)

    for module in ("grave_pistol", "bone_knife", "scythe"):
        weapon = _part(module).build(spin)
        # Each weapon's grip is its origin, at the middle of the fist.
        weapon.location = tuple(-value for value in arms.TRIGGER)

    palm = arms.build_left(root)
    _part("soul_cylinder").build(palm)
    _part("capture_orb").build_held(palm)
    _part("effects").trail(root)

    counts = _triangles(root)

    for name, count in sorted(counts.items()):
        print(f"MESH {name} triangles={count}")

    for moment, meshes in MOMENTS.items():
        shown = sum(counts[name] for name in meshes)
        print(f"MOMENT {moment} draws={len(meshes)} triangles={shown}")

        if shown > TRIANGLE_BUDGET:
            raise SystemExit(f"A {moment} shows {shown} triangles; the budget is {TRIANGLE_BUDGET}")

    if options.preview:
        _sheet(options, "viewmodel", root)

    _export(options, root, "viewmodel.glb")


def _build_orb(options):
    core.reset_scene()

    root = _part("capture_orb").build()
    triangles = core.triangle_count(root)

    if triangles > ORB_TRIANGLE_BUDGET:
        raise SystemExit(f"The orb has {triangles} triangles; the budget is {ORB_TRIANGLE_BUDGET}")

    if options.preview:
        _sheet(options, "viewmodel_orb", root)

    _export(options, root, "viewmodel_orb.glb")


def _build_fx(options):
    core.reset_scene()

    root = core.empty("vmfx")
    _part("effects").build(root)
    triangles = core.triangle_count(root)

    if triangles > FX_TRIANGLE_BUDGET:
        raise SystemExit(f"The effects have {triangles} triangles; the budget is {FX_TRIANGLE_BUDGET}")

    _export(options, root, "viewmodel_fx.glb")


def _triangles(root):
    counts = {}

    def visit(node):
        if node.type == "MESH":
            counts[node.name] = sum(len(polygon.vertices) - 2 for polygon in node.data.polygons)

        for child in node.children:
            visit(child)

    visit(root)

    return counts


def _sheet(options, name, root):
    os.makedirs(options.preview, exist_ok=True)
    centre, distance = preview.frame(margin=1.05)
    preview.render_sheet(
        os.path.join(options.preview, f"{name}.png"),
        centre=centre,
        distance=distance,
        width=520,
        height=520,
        views=("front", "threequarter", "side", "silhouette"),
        eye_height=distance * 0.18,
    )


def _export(options, root, file):
    path = os.path.join(options.out, file)
    core.export_glb(path, root)
    print(f"BUILT {path} triangles={core.triangle_count(root)}")


if __name__ == "__main__":
    main()
