"""Shows a clip's motion in a still: the paths the weapon's tip and both
wrists take through every frame, over ghosts of the pose along the way.

    blender --background --python-exit-code 1 --python art/blender/review_trail.py -- \\
      --class harvester --clip finisherReap --out art/build/previews/trail_harvester_finisherReap.png

The pose at the clip's contact (its middle, when it has none) is drawn
solid; ghosts of the poses before and after it are tinted from cold (the
start) to warm (the end). Each path has a dot at every frame, so the gaps
between dots are the speed: wide gaps are fast, bunched dots are slow.

- A green dot is a dead stop: a frame where the point all but stops in the
  middle of the clip (under a tenth of its top speed).
- A red dot is a flap: the point turns back on itself (more than 110°)
  while still moving fast, which reads as a joint snapping or jittering.

An arc that is smooth, with dots spreading out into the strike and
bunching only where the motion means to hold, is fluid. It also prints each
path's length, top speed, stops and flaps, for comparing two builds.

It reads the full-size models a build leaves in `art/build/models` (or
`--models`) and the clips' timings in `art/build/motions.json` (or
`--motions`). `--cel` draws the solid pose in the cel look.
"""

import argparse
import math
import os
import sys

import bmesh
import bpy
from mathutils import Vector

sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from lib import cel, preview, review  # noqa: E402

PATHS = (
    # (what, colour)
    ("tip", (1.0, 0.42, 0.08)),
    ("wrist_r", (0.1, 0.85, 1.0)),
    ("wrist_l", (1.0, 0.25, 0.85)),
)
STOP = (0.1, 0.8, 0.2)
FLAP = (1.0, 0.05, 0.05)
# A dead stop is slower than this share of the path's top speed...
STOP_SHARE = 0.1
# ...and a flap turns back by more than this while faster than this share.
FLAP_TURN = 110.0
FLAP_SHARE = 0.25
GHOST_ALPHA = 0.14


def main():
    parser = argparse.ArgumentParser(prog="review_trail.py")
    parser.add_argument("--class", dest="calling", choices=tuple(review.CLASS_WEAPONS), required=True)
    parser.add_argument("--clip", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--humanoid", action="store_true")
    parser.add_argument("--weapon", choices=review.WEAPON_KINDS)
    parser.add_argument("--variant", type=int, choices=range(review.VARIANTS), default=0)
    parser.add_argument("--ghosts", type=int, default=7, help="how many ghost poses, the first and last frames included")
    parser.add_argument("--cel", action="store_true")
    parser.add_argument("--models", default="art/build/models")
    parser.add_argument("--motions", default="art/build/motions.json")
    options = parser.parse_args(sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else [])

    if options.ghosts < 2:
        raise SystemExit("--ghosts must be at least 2: the first frame and the last")

    spec = review.motion_spec(options.motions, options.clip)
    form = "humanoid" if options.humanoid else "spirit"
    clip = ("humanoid_" if options.humanoid else "") + options.clip
    root, weapons = review.reaper_for_motion(options.models, options.calling)
    kind = review.held_kind(spec, options.calling, options.weapon)
    review.dress(root, options.variant, form, weapons, kind)

    points = {"wrist_r": review.find(root, "wrist_r"), "wrist_l": review.find(root, "wrist_l")}
    tip = None if kind is None else _tip(weapons[kind])
    first, last = review.clip_frames(clip)
    frames = [*range(math.ceil(first), math.floor(last) + 1)]

    if frames[-1] != last:
        frames.append(last)

    paths = {name: [] for name, _ in PATHS}

    for frame in frames:
        review.play(clip, (frame - first) / (last - first))

        for name, node in points.items():
            paths[name].append(node.matrix_world.translation.copy())

        if tip is not None:
            weapon, local = tip
            paths["tip"].append(weapon.matrix_world @ local)

    key = spec.get("impact")
    key = 0.5 if key is None else key

    for index in range(options.ghosts):
        phase = index / (options.ghosts - 1)

        if abs(phase - key) > 0.02:
            review.play(clip, phase)
            _ghost(index, phase)

    seconds_per_frame = bpy.context.scene.render.fps_base / bpy.context.scene.render.fps

    for name, colour in PATHS:
        if paths[name]:
            _draw(name, paths[name], colour, seconds_per_frame)

    review.play(clip, key)
    look = cel.load() if options.cel else None
    os.makedirs(os.path.dirname(os.path.abspath(options.out)), exist_ok=True)
    title = f"{options.calling} {clip}, solid at {key:.0%}\norange tip, cyan right wrist, pink left wrist\ngreen: dead stop, red: flap"
    preview.render_grid(options.out, [preview.row(title, None, ("front", "side", "top"))], tile=(560, 640), cel_look=look, elevation=4.0, margin=1.08)
    print(f"BUILT {options.out}")


def _tip(weapon):
    """The weapon and the point of it farthest from its grip, in its own
    frame: the tip of a blade, the end of a chain."""

    farthest, reach = Vector(), -1.0
    to_weapon = weapon.matrix_world.inverted()

    for obj in [weapon, *weapon.children_recursive]:
        if obj.type != "MESH":
            continue

        into = to_weapon @ obj.matrix_world

        for vertex in obj.data.vertices:
            point = into @ vertex.co

            if point.length > reach:
                farthest, reach = point, point.length

    return weapon, farthest


def _material(name, colour, alpha=1.0):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    tree = mat.node_tree
    tree.nodes.clear()
    output = tree.nodes.new("ShaderNodeOutputMaterial")
    emission = tree.nodes.new("ShaderNodeEmission")
    emission.inputs["Color"].default_value = (*colour, 1)

    if alpha < 1:
        mat.surface_render_method = "BLENDED"
        clear = tree.nodes.new("ShaderNodeBsdfTransparent")
        mix = tree.nodes.new("ShaderNodeMixShader")
        mix.inputs["Fac"].default_value = alpha
        tree.links.new(clear.outputs["BSDF"], mix.inputs[1])
        tree.links.new(emission.outputs["Emission"], mix.inputs[2])
        tree.links.new(mix.outputs["Shader"], output.inputs["Surface"])
    else:
        tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])

    return mat


def _ghost(index, phase):
    """A tinted, see-through copy of every shown mesh as it is now."""

    cold, warm = Vector((0.1, 0.35, 1.0)), Vector((1.0, 0.3, 0.05))
    mat = _material(f"review_ghost_{index}", tuple(cold.lerp(warm, phase)), GHOST_ALPHA)
    depsgraph = bpy.context.evaluated_depsgraph_get()
    shown = [obj for obj in bpy.data.objects if obj.type == "MESH" and not obj.hide_render and not obj.name.startswith(cel.REVIEW_PREFIX)]

    for obj in shown:
        mesh = bpy.data.meshes.new_from_object(obj.evaluated_get(depsgraph))
        mesh.materials.clear()
        mesh.materials.append(mat)
        ghost = bpy.data.objects.new(f"review_ghost_{index}_{obj.name}", mesh)
        ghost.matrix_world = obj.matrix_world.copy()
        bpy.context.scene.collection.objects.link(ghost)


def _draw(name, points, colour, seconds_per_frame):
    """The path as a tube, with a dot at every frame, and its numbers."""

    curve = bpy.data.curves.new(f"review_path_{name}", "CURVE")
    curve.dimensions = "3D"
    curve.bevel_depth = 0.008
    spline = curve.splines.new("POLY")
    spline.points.add(len(points) - 1)

    for point, position in zip(spline.points, points):
        point.co = (*position, 1)

    curve.materials.append(_material(f"review_path_{name}", colour))
    path = bpy.data.objects.new(f"review_path_{name}", curve)
    bpy.context.scene.collection.objects.link(path)

    steps = [b - a for a, b in zip(points, points[1:])]
    speeds = [step.length / seconds_per_frame for step in steps]
    top = max(speeds) if speeds else 0.0
    stops, flaps = [], []

    for index in range(1, len(steps)):
        before, after = steps[index - 1], steps[index]
        middle = 0 < index < len(steps) - 1

        if middle and top > 0 and speeds[index] < STOP_SHARE * top and speeds[index - 1] >= STOP_SHARE * top:
            stops.append(index)
        if min(speeds[index - 1], speeds[index]) > FLAP_SHARE * top and math.degrees(before.angle(after, 0.0)) > FLAP_TURN:
            flaps.append(index)

    dot = _material(f"review_dot_{name}", colour)
    stop = _material(f"review_stop_{name}", STOP)
    flap = _material(f"review_flap_{name}", FLAP)

    for index, position in enumerate(points):
        material, radius = (flap, 0.05) if index in flaps else (stop, 0.045) if index in stops else (dot, 0.02)
        _sphere(f"review_dot_{name}_{index}", position, radius, material)

    print(f"TRAIL {name}: {sum(step.length for step in steps):.2f} m over {len(points)} frames, top speed {top:.2f} m/s, dead stops at frames {stops or 'none'}, flaps at frames {flaps or 'none'}")


def _sphere(name, location, radius, material):
    mesh = bpy.data.meshes.new(name)
    shape = bmesh.new()
    bmesh.ops.create_icosphere(shape, subdivisions=1, radius=radius)
    shape.to_mesh(mesh)
    shape.free()
    mesh.materials.append(material)
    obj = bpy.data.objects.new(name, mesh)
    obj.location = location
    bpy.context.scene.collection.objects.link(obj)


if __name__ == "__main__":
    main()
