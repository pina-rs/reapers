"""Checks the review harness against the built models, so a sheet that looks
right can be trusted to be right:

    blender --background --python-exit-code 1 --python art/blender/review_check.py

- Every reaper imports at its bind pose: the pose an import with its clips
  stripped out shows, node for node and morph for morph, whatever clip the
  importer read first.
- Every reaper's sheets show it in its idle stance: every node the spirit's
  `idle` (or the humanoid's `humanoid_idle`) moves where that clip's first
  key in the file puts it.
- The trail finds the off-hand flaps docs/animations.md lists (thrust,
  slash and parry drop `elbow_l` onto its bind pose) and none in idle or a
  walk.
- The first-person view is staged as the game places it: in a reload and a
  throw the right hand is low on the right and the left hand left of it,
  both in front of the eye, the pistol and the left palm in its view, and
  the spare drum or the orb in the left palm.
- Every reaper's face can be found for every variant, as
  `review_motions.py --face` looks for it.

It reads the full-size models in `art/build/models` (or `--models`) and
prints a `CHECK` line for each check, then fails if any did.
"""

import argparse
import json
import math
import os
import struct
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.append(HERE)

import bpy  # noqa: E402
from mathutils import Quaternion, Vector  # noqa: E402

import review_trail  # noqa: E402
from lib import core, review  # noqa: E402
from viewmodel import MOMENTS  # noqa: E402

CALLINGS = tuple(review.CLASS_WEAPONS)
# The clip each form's sheets must stand in.
STANCES = {"spirit": "idle", "humanoid": "humanoid_idle"}
# (clip, joints that must flap, joints that must not)
FLAPS = (
    ("thrust", {"elbow_l"}, set()),
    ("slash", {"elbow_l"}, set()),
    ("parry", {"elbow_l"}, set()),
    ("idle", set(), set(review_trail.JOINTS)),
    ("walk", set(), set(review_trail.JOINTS)),
)


def main():
    arguments = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser(prog="review_check.py")
    parser.add_argument("--models", default="art/build/models")
    options = parser.parse_args(arguments)
    failures = []

    def check(name, problems):
        print(f"CHECK {name}: {'ok' if not problems else '; '.join(problems)}")

        if problems:
            failures.append(name)

    for calling in CALLINGS:
        check(f"rest pose of reaper_{calling}", _rest(os.path.join(options.models, f"reaper_{calling}.glb")))

        for form, clip in STANCES.items():
            check(f"{form} stance of reaper_{calling}", _stance(options.models, calling, form, clip))
        check(f"faces of reaper_{calling}", _faces(options.models, calling))

    for clip, flapping, steady in FLAPS:
        check(f"flaps in harvester {clip}", _flaps(options.models, clip, flapping, steady))

    check("first-person staging", _first_person(os.path.join(options.models, "viewmodel.glb")))

    if failures:
        raise SystemExit(f"{len(failures)} review checks failed: {', '.join(failures)}")


def _snapshot():
    shot = {}

    for obj in bpy.data.objects:
        keys = obj.data.shape_keys if obj.type == "MESH" else None
        shot[obj.name] = (obj.matrix_world.copy(), [key.value for key in keys.key_blocks] if keys else [])

    return shot


def _rest(path):
    """What differs between a reviewed import and one without clips."""

    with open(path, "rb") as source:
        data = source.read()

    length = struct.unpack_from("<I", data, 12)[0]
    document = json.loads(data[20 : 20 + length])
    document.pop("animations")
    text = json.dumps(document).encode()
    text += b" " * (-len(text) % 4)
    binary = data[20 + length :]

    with tempfile.TemporaryDirectory() as folder:
        still = os.path.join(folder, "still.glb")

        with open(still, "wb") as out:
            out.write(struct.pack("<4sII", b"glTF", 2, 20 + len(text) + len(binary)))
            out.write(struct.pack("<I4s", len(text), b"JSON") + text + binary)

        core.reset_scene()
        bpy.ops.import_scene.gltf(filepath=still)
        expected = _snapshot()

    core.reset_scene()
    review.import_glb(path)
    found = _snapshot()
    problems = []

    for name, (matrix, keys) in expected.items():
        moved = max(abs(a - b) for row_a, row_b in zip(matrix, found[name][0]) for a, b in zip(row_a, row_b))
        morphed = max((abs(a - b) for a, b in zip(keys, found[name][1])), default=0.0)

        if moved > 1e-5 or morphed > 1e-5:
            problems.append(f"{name} is off its bind pose")

    return problems


def _first_keys(path, clip):
    """Each node's translation and rotation at the first key of `clip`, as
    the file stores them (glTF's frame)."""

    with open(path, "rb") as source:
        data = source.read()

    length = struct.unpack_from("<I", data, 12)[0]
    document = json.loads(data[20 : 20 + length])
    binary = data[20 + length + 8 :]
    animation = next(animation for animation in document["animations"] if animation["name"] == clip)
    widths = {"VEC3": 3, "VEC4": 4}
    keys = {}

    for channel in animation["channels"]:
        path_name = channel["target"]["path"]

        if path_name not in ("translation", "rotation"):
            continue

        accessor = document["accessors"][animation["samplers"][channel["sampler"]]["output"]]

        if accessor["componentType"] != 5126 or "sparse" in accessor:
            raise ValueError(f"{path}: {clip} stores {path_name} in a way this check does not read")

        view = document["bufferViews"][accessor["bufferView"]]
        start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
        value = struct.unpack_from(f"<{widths[accessor['type']]}f", binary, start)
        keys.setdefault(document["nodes"][channel["target"]["node"]]["name"], {})[path_name] = value

    return keys


def _stance(models, calling, form, clip):
    """What is off the first key of `clip` once a reaper stands in it."""

    path = os.path.join(models, f"reaper_{calling}.glb")
    core.reset_scene()
    review.import_glb(path)
    root = bpy.data.objects["reaper"]
    review.stand(root, 0, form)
    problems = []

    for name, key in _first_keys(path, clip).items():
        obj = bpy.data.objects[name]

        if "translation" in key:
            x, y, z = key["translation"]

            if (obj.location - Vector((x, -z, y))).length > 1e-4:
                problems.append(f"{name} is not where {clip} starts it")
        if "rotation" in key:
            x, y, z, w = key["rotation"]

            if abs(obj.rotation_quaternion.dot(Quaternion((w, x, -z, y)))) < 1 - 1e-6:
                problems.append(f"{name} is not turned as {clip} starts it")

    return problems


def _faces(models, calling):
    core.reset_scene()
    review.import_glb(os.path.join(models, f"reaper_{calling}.glb"))
    root = bpy.data.objects["reaper"]
    counts = review.variant_counts(root)
    shown = [f"face_{review.shown_variant(counts, 'face_', variant)}" for variant in range(review.VARIANTS)]

    return [f"variant {variant} shows {name}, which is not in the model" for variant, name in enumerate(shown) if review.find(root, name) is None]


def _flaps(models, clip, flapping, steady):
    root, _ = review.reaper_for_motion(models, "harvester")
    first, last = review.clip_frames(clip)
    turns = {joint: [] for joint in review_trail.JOINTS}

    for frame in range(round(first), round(last) + 1):
        review.play(clip, (frame - first) / (last - first))

        for joint in turns:
            turns[joint].append(review.find(root, joint).matrix_basis.to_quaternion())

    found = {joint for joint, rotations in turns.items() if review_trail.flaps(rotations)}
    problems = [f"no flap found in {joint}" for joint in sorted(flapping - found)]
    problems += [f"a flap found in {joint}" for joint in sorted(steady & found)]

    return problems


def _first_person(path):
    core.reset_scene()
    review.import_glb(path)
    numbers = review.rig()
    stages = review.moment_stages(numbers)
    problems = [f"no staging for {moment}" for moment in MOMENTS if moment not in stages]
    rest = {obj: obj.matrix_basis.copy() for obj in bpy.data.objects}

    def centre(name):
        obj = bpy.data.objects[name]

        return sum((obj.matrix_world @ Vector(corner) for corner in obj.bound_box), Vector()) / 8

    for moment, held in (("reload", "soul_cylinder"), ("throw", "orb_held")):
        for obj, basis in rest.items():
            obj.matrix_basis = basis

        for part, frame in review.place(numbers, *stages[moment]).items():
            obj = bpy.data.objects[part]
            obj.matrix_basis = rest[obj] @ review.blender_offset(frame)

        bpy.context.view_layer.update()
        # The eye is at the origin looking down Blender's +Y, +Z up: the
        # right hand low on the right and the left hand left of it, both in
        # front of the eye.
        right, palm = centre("arm_r"), bpy.data.objects["socket_palm"].matrix_world.translation
        reach = math.tan(numbers["fov"] / 2)

        if not (right.x > 0 and right.z < 0 and right.y > 0.2):
            problems.append(f"in a {moment} the right hand is at {tuple(round(value, 3) for value in right)}, not low on the right in front of the eye")
        if not (palm.x < right.x - 0.05 and palm.y > 0.2):
            problems.append(f"in a {moment} the left palm is at {tuple(round(value, 3) for value in palm)}, not left of the right hand in front of the eye")
        # The right hand itself may run off the bottom of the view; the
        # pistol it holds may not.
        for what, point in (("pistol", centre("pistol")), ("left palm", palm)):
            if point.y > 0 and abs(point.z) / point.y > reach:
                problems.append(f"in a {moment} the {what} is out of the eye's view")
        if (centre(held) - palm).length > 0.05:
            problems.append(f"in a {moment} the {held} is {(centre(held) - palm).length * 100:.0f} cm from the palm")

    return problems


if __name__ == "__main__":
    main()
