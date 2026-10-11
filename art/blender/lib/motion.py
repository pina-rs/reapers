"""Shared spirit and humanoid performances, baked to named glTF clips.

Pose angles are local XYZ degrees in Blender's Z-up frame. Each clip keys
all joints, including resting joints, so switching never inherits a stale pose.
The rig is articulated rigid cloth, matching the existing node-based renderer.
"""

import math

import bpy
from mathutils import Euler, Quaternion, Vector

from . import body, combat, core, curves, locomotion

JOINTS = ("motion", "spine", "head", "arm_r", "arm_l", "elbow_r", "elbow_l", "wrist_r", "wrist_l") + body.LEG_JOINTS

# Subtle timing/size accents preserve silhouettes without losing shared clips.
AMPLITUDE = {"harvester": 1.0, "ferryman": 0.85, "weigher": 0.8, "chooser": 1.05, "hollow": 1.12, "mourner": 0.75}


def pose(**joints):
    """A sparse pose, with absent channels returning to the bind pose."""
    return joints


REST = pose()
GUARD = pose(arm_r=(-28, 12, -12), arm_l=(-48, -18, 18), elbow_r=(-28, 0, 0), elbow_l=(-42, 0, 0), head=(10, 0, 0))

# Each move has deliberate anticipation, a readable apex and recovery. Positions
# are offsets, so authored class proportions and hunched necks remain intact.
KEYS = {
    "jump": [
        (0, REST),
        (0.18, pose(move=(0, 0, -0.04), motion=(9, 0, 0), arm_l=(18, 0, 0))),
        (0.48, pose(move=(0, 0, 0.65), motion=(-10, 0, 0), arm_r=(-38, 0, 0), arm_l=(-65, -20, 0))),
        (0.8, pose(move=(0, 0, 0.08), motion=(8, 0, 0))),
        (1, REST),
    ],
    "land": [
        (0, pose(move=(0, 0, 0.2))),
        (0.26, pose(move=(0, 0, -0.04), motion=(16, 0, 0), arm_l=(-25, -22, 0))),
        (0.62, pose(move=(0, 0, 0.055), motion=(-5, 0, 0))),
        (1, REST),
    ],
    "dash": [
        (0, REST),
        (0.2, pose(move=(0, 0.1, 0), motion=(-7, 0, 0))),
        (0.48, pose(move=(0, -0.8, 0.12), motion=(32, 0, 0), arm_r=(25, 0, 0), arm_l=(50, -12, 0))),
        (0.7, pose(move=(0, -0.9, 0.06), motion=(12, 0, 0))),
        (1, REST),
    ],
    "laugh": [
        (0, REST),
        (0.2, pose(head=(-24, 0, 0), motion=(-10, 0, 0), arm_l=(-45, -18, 0), elbow_l=(-45, 0, 0))),
        (0.82, pose(head=(-20, 0, 0), motion=(-6, 0, 0), arm_l=(-45, -18, 0), elbow_l=(-45, 0, 0))),
        (1, REST),
    ],
    "cry": [
        (0, REST),
        (0.25, pose(head=(20, 0, 0), motion=(10, 0, 0), arm_l=(-35, -10, -15), elbow_l=(-100, 0, 0))),
        (0.8, pose(head=(24, 0, -8), motion=(12, 0, 0), arm_l=(-40, -10, -15), elbow_l=(-105, 0, 0))),
        (1, REST),
    ],
    "rage": [
        (0, REST),
        (0.28, pose(head=(20, 0, 0), motion=(10, 0, 0), arm_r=(-15, 30, 0), arm_l=(-15, -30, 0), elbow_r=(-65, 0, 0), elbow_l=(-65, 0, 0))),
        (0.55, pose(head=(-25, 0, 0), move=(0, 0, 0.1), arm_r=(0, 50, 0), arm_l=(0, -50, 0), elbow_r=(-20, 0, 0))),
        (0.8, GUARD),
        (1, REST),
    ],
    "fear": [
        (0, REST),
        (0.22, pose(move=(0, 0.28, 0.04), motion=(-12, 0, 0), head=(-12, 0, 15), arm_r=(-80, 15, 0), arm_l=(-100, -25, 0), elbow_l=(-30, 0, 0))),
        (0.75, pose(move=(0, 0.2, 0), motion=(-10, 0, 0), arm_r=(-80, 15, 0), arm_l=(-100, -25, 0))),
        (1, REST),
    ],
    "bow": [
        (0, REST),
        (0.35, pose(motion=(28, 0, 0), head=(20, 0, 0), arm_l=(-35, -15, -25), elbow_l=(-65, 0, 0))),
        (0.65, pose(motion=(28, 0, 0), head=(20, 0, 0), arm_l=(-35, -15, -25), elbow_l=(-65, 0, 0))),
        (1, REST),
    ],
    "victory": [
        (0, REST),
        (0.25, pose(arm_r=(-105, 20, 0), elbow_r=(-20, 0, 0), head=(-15, 0, 0), move=(0, 0, 0.12))),
        (0.75, pose(arm_r=(-110, 25, 0), elbow_r=(-15, 0, 0), head=(-15, 0, -15), arm_l=(0, -25, 0), move=(0, 0, 0.12))),
        (1, REST),
    ],
    "gigachad": [
        (0, REST),
        (0.3, pose(motion=(-8, 0, -12), head=(-18, 0, 24), arm_r=(0, 28, 0), arm_l=(0, -28, 0), elbow_r=(-50, 0, 0), elbow_l=(-50, 0, 0))),
        (0.78, pose(motion=(-8, 0, -12), head=(-18, 0, 24), arm_r=(0, 28, 0), arm_l=(0, -28, 0), elbow_r=(-50, 0, 0), elbow_l=(-50, 0, 0))),
        (1, REST),
    ],
    "sigma": [
        (0, REST),
        (0.28, pose(head=(12, 0, -30), motion=(0, 0, 10))),
        (0.45, pose(head=(-12, 0, 12), motion=(-4, 0, 10))),
        (0.8, pose(head=(-12, 0, 12), motion=(-4, 0, 10))),
        (1, REST),
    ],
    "shrug": [
        (0, REST),
        (0.3, pose(head=(0, -12, 0), arm_r=(0, 55, 0), arm_l=(0, -55, 0), elbow_r=(-15, 0, 0), elbow_l=(-15, 0, 0), wrist_l=(0, 0, 70))),
        (0.7, pose(head=(0, 12, 0), arm_r=(0, 55, 0), arm_l=(0, -55, 0), elbow_r=(-15, 0, 0), elbow_l=(-15, 0, 0))),
        (1, REST),
    ],
    "absoluteCinema": [
        (0, REST),
        (
            0.28,
            pose(
                arm_r=(-10, 125, 0),
                arm_l=(-10, -125, 0),
                elbow_r=(-25, 0, 0),
                elbow_l=(-25, 0, 0),
                wrist_r=(0, -40, 0),
                wrist_l=(0, 40, 0),
                head=(-8, 0, 0),
            ),
        ),
        (
            0.8,
            pose(
                arm_r=(-10, 125, 0),
                arm_l=(-10, -125, 0),
                elbow_r=(-25, 0, 0),
                elbow_l=(-25, 0, 0),
                wrist_r=(0, -40, 0),
                wrist_l=(0, 40, 0),
                head=(-8, 0, 0),
            ),
        ),
        (1, REST),
    ],
    "tPose": [(0, REST), (0.3, pose(arm_r=(0, 90, 0), arm_l=(0, -90, 0))), (0.8, pose(arm_r=(0, 90, 0), arm_l=(0, -90, 0))), (1, REST)],
    "animeCharge": [
        (0, REST),
        (0.2, GUARD),
        (
            0.72,
            pose(move=(0, 0, 0.16), motion=(12, 0, 0), head=(15, 0, 0), arm_r=(0, 40, 0), arm_l=(0, -40, 0), elbow_r=(-80, 0, 0), elbow_l=(-80, 0, 0)),
        ),
        (0.86, pose(move=(0, 0, 0.3), motion=(-8, 0, 0), head=(-25, 0, 0), arm_r=(0, 75, 0), arm_l=(0, -75, 0))),
        (1, REST),
    ],
}
KEYS.update(combat.KEYS)


def sample(name, phase):
    """Samples a clip in place. Translation is a bounded dramatic excursion."""
    wave = math.sin(phase * math.tau)
    if name in ("idle", "creep", "walk", "run", "sprint"):
        lean, swing, lift = {"idle": (0, 3, 0.025), "creep": (10, 5, 0.018), "walk": (5, 14, 0.04), "run": (18, 27, 0.07), "sprint": (32, 12, 0.10)}[name]
        return pose(
            move=(0.012 * wave, 0, lift * (1 - math.cos(phase * math.tau))),
            motion=(lean, 2 * wave, 3 * wave),
            head=(-lean * 0.5, 0, 5 * wave),
            arm_r=(swing * wave, 8, 0),
            arm_l=(-swing * wave, -8, 0),
            elbow_r=(-12, 0, 0),
            elbow_l=(-20 - 8 * wave, 0, 0),
        )
    if name in ("dodgeLeft", "dodgeRight"):
        side = 1 if name == "dodgeLeft" else -1
        keys = [
            (0, REST),
            (0.18, GUARD),
            (0.48, pose(move=(side * 0.6, 0, 0.08), motion=(10, side * -22, 0), arm_r=(-25, 20, 0), arm_l=(-40, -20, 0))),
            (0.7, pose(move=(side * 0.6, 0, 0))),
            (1, REST),
        ]
    elif name in ("dashLeft", "dashRight"):
        # +X is the reaper's left. Bank into a fast lateral burst, with
        # trailing arms and a longer excursion than the guarded dodge.
        side = 1 if name == "dashLeft" else -1
        keys = [
            (0, REST),
            (0.16, pose(move=(-side * 0.1, 0, -0.025), motion=(0, -side * 8, 0))),
            (0.4, pose(move=(side * 0.95, 0, 0.1), motion=(12, side * 30, 0), head=(-6, -side * 12, 0), arm_r=(22, side * 18, 0), arm_l=(35, side * 18, 0))),
            (0.65, pose(move=(side * 1.05, 0, 0.05), motion=(5, side * 12, 0), arm_r=(10, side * 8, 0), arm_l=(18, side * 8, 0))),
            (1, REST),
        ]
    else:
        keys = KEYS[name]
    result = curves.sample(keys, phase)
    if name in ("laugh", "cry", "animeCharge"):
        tremble = math.sin(phase * math.tau * (7 if name == "animeCharge" else 5)) * math.sin(phase * math.pi) ** 2
        rotation = result.get("motion", (0, 0, 0))
        result["motion"] = (rotation[0] + 1.5 * tremble, rotation[1], rotation[2])
    return result


def variants(specs):
    """Keep 32 studio states while exporting distinct performances per form."""
    return [dict(spec, motion=spec["name"], humanoid=human, name=("humanoid_" if human else "") + spec["name"]) for human in (False, True) for spec in specs]


def performance(name, phase, humanoid, calling):
    """Sample the whole body once, including overlapping chest and head timing."""
    value = sample(name, phase)
    wave = math.cos(math.tau * phase)
    if humanoid and name in locomotion.GAITS:
        swing = {"creep": 9, "walk": 22, "run": 38, "sprint": 48}[name]
        lean = {"creep": 9, "walk": 3, "run": 12, "sprint": 19}[name]
        value.update(
            motion=(lean, -2 * wave, -7 * wave),
            head=(-lean * 0.6, 0, 3 * wave),
            # The object-carrying arm stays close; the free arm drives the run.
            arm_r=(-swing * 0.35 * wave, 10, 0),
            arm_l=(swing * wave, -10, 0),
            elbow_r=(-32, 0, 0),
            elbow_l=((-24 if name in ("creep", "walk") else -80) - 8 * math.sin(math.tau * phase), 0, 0),
        )
    # Hips lead the strike; the head settles last. Endpoints remain exact.
    overlap = math.sin(math.pi * phase) ** 2
    lead = sample(name, min(1, phase + 0.024 * overlap)) if name in combat.KEYS else value
    delayed = sample(name, max(0, phase - 0.025 * overlap))
    angles = value.get("motion", (0, 0, 0))
    positions = {joint: Vector() for joint in JOINTS}
    rotations = {joint: Quaternion() for joint in JOINTS}
    if humanoid:
        move, yaw, feet = locomotion.grounded(name, phase, lead)
        # Lower the pelvis when a long stride needs more reach. Preserve bone
        # lengths and planted ankles instead of stretching or clipping a leg.
        yaw_rotation = Euler((0, 0, math.radians(yaw))).to_quaternion()
        for side, sign in (("l", 1), ("r", -1)):
            hip_offset = yaw_rotation @ Vector((sign * locomotion.HIP_X, 0, 0))
            ankle = feet[side][0]
            horizontal = Vector((move.x + hip_offset.x - ankle.x, move.y + hip_offset.y - ankle.y))
            vertical = math.sqrt((2 * locomotion.LENGTH - 0.012) ** 2 - horizontal.length_squared)
            move.z = min(move.z, ankle.z + vertical - locomotion.HIP_Z)
        root_angles = (0, 0, yaw)
        spine_angles = (angles[0], angles[1], angles[2] - yaw)
    else:
        move = Vector(value.get("move", (0, 0, 0)))
        root_angles = (angles[0] * 0.18, angles[1] * 0.4, angles[2] * 0.45)
        spine_angles = (angles[0] * 0.82, angles[1] * 0.6, angles[2] * 0.55)
    rotations["motion"] = Euler(tuple(math.radians(v) for v in root_angles)).to_quaternion()
    rotations["spine"] = Euler(tuple(math.radians(v) for v in spine_angles)).to_quaternion()
    positions["motion"] = move
    for joint in JOINTS:
        if joint in ("motion", "spine") or joint in body.LEG_JOINTS:
            continue
        source = joint
        mirror = calling == "hollow" and name in ("finisherPalm", "finisherSoulCrush")
        if mirror and joint.endswith(("_l", "_r")):
            source = joint[:-1] + ("l" if joint.endswith("_r") else "r")
        pose_source = delayed if joint.startswith(("head", "wrist")) and not (humanoid and name in locomotion.GAITS) else value
        a = pose_source.get(source, (0, 0, 0))
        if mirror and source != joint:
            a = (a[0], -a[1], -a[2])
        gain = AMPLITUDE[calling]
        if calling == "mourner" and joint == "head":
            gain *= 0.25
        rotations[joint] = Euler(tuple(math.radians(v * gain) for v in a)).to_quaternion()
    if humanoid:
        root_rotation = rotations["motion"]
        root_position = Vector((0, 0, 1.05)) + move
        for side, sign in (("l", 1), ("r", -1)):
            hip = root_position + root_rotation @ Vector((sign * locomotion.HIP_X, 0, locomotion.HIP_Z - 1.05))
            ankle, foot_rotation = feet[side]
            try:
                thigh, knee, shin = locomotion.solve(hip, ankle, root_rotation @ Vector((0, -1, 0)))
            except ValueError as error:
                raise ValueError(f"{name} phase={phase:.3f} {side}: {error}") from error
            rotations[f"leg_{side}"] = root_rotation.inverted() @ thigh
            rotations[f"knee_{side}"] = knee
            rotations[f"foot_{side}"] = shin.inverted() @ foot_rotation
    return positions, rotations


def build(root, calling, specs):
    """Bake form-specific clips onto one hierarchy with a chest and pelvis."""
    children = list(root.children)
    pivot = core.empty("motion", root, location=(0, 0, 1.05))
    for child in children:
        child.parent = pivot
        child.location.z -= 1.05
    spine = core.empty("spine", pivot, location=(0, 0, 0.13))
    for child in list(pivot.children):
        if child.name in ("head", "arm_l", "arm_r", "spirit_body") or child.name.startswith("mantle_"):
            child.parent = spine
            child.location.z -= 0.13
    nodes = {node.name: node for node in root.children_recursive}
    joints = {name: nodes[name] for name in JOINTS}
    rest = {name: node.location.copy() for name, node in joints.items()}
    bpy.context.scene.render.fps = 30
    offset = 1
    for spec in specs:
        frames = round(spec["seconds"] * 30)
        samples = [performance(spec["motion"], frame / frames, spec["humanoid"], calling) for frame in range(frames + 1)]
        for joint, node in joints.items():
            node.animation_data_create()
            node.animation_data.action = None
            node.rotation_mode = "QUATERNION"
            previous = None
            for frame, (positions, rotations) in enumerate(samples):
                rotation = rotations[joint]
                if previous is not None and previous.dot(rotation) < 0:
                    rotation = -rotation
                previous = rotation
                node.rotation_quaternion = rotation
                node.location = rest[joint] + positions[joint]
                node.keyframe_insert(data_path="rotation_quaternion", frame=frame)
                node.keyframe_insert(data_path="location", frame=frame)
            action = node.animation_data.action
            action.name = f'{spec["name"]}_{joint}'
            slot = node.animation_data.action_slot
            track = node.animation_data.nla_tracks.new()
            track.name = spec["name"]
            strip = track.strips.new(spec["name"], offset, action)
            strip.action_slot = slot
            strip.extrapolation = "NOTHING"
            node.animation_data.action = None
            node.location = rest[joint]
            node.rotation_quaternion = Quaternion()
        offset += frames + 5
    bpy.context.scene.frame_set(0)
    return joints
