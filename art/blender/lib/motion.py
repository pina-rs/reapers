"""Thirty reusable floating-reaper performances, baked to named glTF clips.

Pose angles are local XYZ degrees in Blender's Z-up frame. Each clip keys
all joints, including resting joints, so switching never inherits a stale pose.
The rig is articulated rigid cloth, matching the existing node-based renderer.
"""

import math

import bpy
from mathutils import Vector

from . import core

JOINTS = ("motion", "head", "arm_r", "arm_l", "elbow_r", "elbow_l", "wrist_r", "wrist_l")

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
        (0.25, pose(head=(32, 0, 0), motion=(15, 0, 0), arm_l=(-100, -10, -15), elbow_l=(-60, 0, 0))),
        (0.8, pose(head=(38, 0, -8), motion=(18, 0, 0), arm_l=(-100, -10, -15), elbow_l=(-60, 0, 0))),
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
            pose(
                move=(0, 0, 0.16), motion=(12, 0, 0), head=(15, 0, 0), arm_r=(0, 40, 0), arm_l=(0, -40, 0), elbow_r=(-80, 0, 0), elbow_l=(-80, 0, 0)
            ),
        ),
        (0.86, pose(move=(0, 0, 0.3), motion=(-8, 0, 0), head=(-25, 0, 0), arm_r=(0, 75, 0), arm_l=(0, -75, 0))),
        (1, REST),
    ],
    "slash": [
        (0, REST),
        (0.3, pose(motion=(0, 0, -35), arm_r=(-50, 30, -45), wrist_r=(0, 40, 0))),
        (0.45, pose(motion=(8, 0, 30), arm_r=(-25, 0, 45), wrist_r=(0, -45, 0), arm_l=(10, -35, 0))),
        (0.62, pose(motion=(5, 0, 40), arm_r=(-20, -15, 55))),
        (1, REST),
    ],
    "thrust": [
        (0, REST),
        (0.27, pose(motion=(-5, 0, -20), arm_r=(20, 0, -10), wrist_r=(80, 0, 0))),
        (0.45, pose(move=(0, -0.3, 0.04), motion=(12, 0, 15), arm_r=(-65, 0, 10), wrist_r=(143, 0, 0))),
        (0.62, pose(move=(0, -0.3, 0.04), motion=(12, 0, 15), arm_r=(-65, 0, 10), wrist_r=(143, 0, 0))),
        (1, REST),
    ],
    "parry": [
        (0, REST),
        (0.22, pose(arm_r=(-55, 20, -30), wrist_r=(0, 0, 55), motion=(0, 0, -15))),
        (0.45, pose(move=(0, 0.1, 0), arm_r=(-50, 15, -25), wrist_r=(0, 0, 55), motion=(-8, 0, -20))),
        (0.65, GUARD),
        (1, REST),
    ],
    "finisherReap": [
        (0, REST),
        (0.25, pose(motion=(0, 0, -55), arm_r=(-40, 40, -30), wrist_r=(10, 65, 0), head=(5, 0, 30))),
        (0.45, pose(motion=(12, 0, -20), arm_r=(-15, 20, 0), wrist_r=(10, 65, 0))),
        (0.52, pose(move=(0, -0.25, 0.06), motion=(14, 0, 70), arm_r=(-10, -10, 60), wrist_r=(0, -55, 0), arm_l=(20, -45, 0))),
        (0.7, pose(motion=(0, 0, 140), arm_r=(-10, 15, 30), head=(0, 0, -35))),
        (1, REST),
    ],
    "finisherImpale": [
        (0, REST),
        (0.3, pose(motion=(-8, 0, -30), arm_r=(15, 0, -20), wrist_r=(85, 0, 0))),
        (0.46, pose(move=(0, -0.55, 0.08), motion=(20, 0, 20), arm_r=(-70, 0, 15), wrist_r=(140, 0, 0))),
        (0.68, pose(move=(0, -0.55, 0.15), motion=(0, 0, 15), arm_r=(-105, 0, 15), wrist_r=(160, 0, 0))),
        (0.85, GUARD),
        (1, REST),
    ],
    "finisherSoulCrush": [
        (0, REST),
        (0.3, pose(arm_l=(-85, -15, 0), elbow_l=(-15, 0, 0), head=(5, 0, 12))),
        (0.48, pose(arm_l=(-125, -10, 0), elbow_l=(-35, 0, 0), head=(-20, 0, 10), move=(0, 0, 0.1))),
        (0.6, pose(arm_l=(-115, -10, 0), elbow_l=(-65, 0, 0), wrist_l=(0, 0, 80), head=(15, 0, 10))),
        (0.78, pose(arm_l=(-40, -40, 0), motion=(0, 0, -15), head=(-10, 0, 20))),
        (1, REST),
    ],
    "finisherPalm": [
        (0, REST),
        (0.32, pose(motion=(0, 0, -35), arm_l=(0, -15, -25), elbow_l=(-90, 0, 0), arm_r=(-35, 20, 0))),
        (0.5, pose(move=(0, -0.4, 0.08), motion=(12, 0, 25), arm_l=(-95, -5, 15), wrist_l=(-20, 0, 0), arm_r=(20, 25, 0))),
        (0.72, pose(move=(0, -0.4, 0.08), motion=(12, 0, 25), arm_l=(-95, -5, 15), wrist_l=(-20, 0, 0))),
        (1, REST),
    ],
    "finisherUppercut": [
        (0, REST),
        (0.3, pose(move=(0, 0, -0.04), motion=(22, 0, -25), arm_r=(25, 15, 0), elbow_r=(-70, 0, 0))),
        (0.48, pose(move=(0, -0.2, 0.55), motion=(-15, 0, 35), arm_r=(-135, 10, 0), elbow_r=(-35, 0, 0), arm_l=(25, -25, 0))),
        (0.66, pose(move=(0, -0.2, 0.4), motion=(-5, 0, 40), arm_r=(-140, 10, 0), elbow_r=(-30, 0, 0))),
        (1, REST),
    ],
}


def sample(name, phase):
    """Samples a clip in place. Translation is a bounded dramatic excursion."""
    wave = math.sin(phase * math.tau)
    if name in ("idle", "creep", "walk", "run", "sprint"):
        lean, swing, lift = {"idle": (0, 3, 0.025), "creep": (10, 5, 0.018), "walk": (5, 14, 0.04), "run": (18, 27, 0.07), "sprint": (32, 12, 0.10)}[
            name
        ]
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
    else:
        keys = KEYS[name]
    for (start, a), (end, b) in zip(keys, keys[1:]):
        if phase <= end:
            t = (phase - start) / (end - start)
            t = t * t * (3 - 2 * t)
            result = {
                joint: tuple(x + (y - x) * t for x, y in zip(a.get(joint, (0, 0, 0)), b.get(joint, (0, 0, 0)))) for joint in a.keys() | b.keys()
            }
            if name in ("laugh", "cry", "animeCharge"):
                tremble = math.sin(phase * math.tau * (7 if name == "animeCharge" else 5)) * math.sin(phase * math.pi) ** 2
                rotation = result.get("motion", (0, 0, 0))
                result["motion"] = (rotation[0] + 2.5 * tremble, rotation[1], rotation[2])
            return result
    return REST


def build(root, calling, specs):
    """Adds the common pivot and bakes matching NLA tracks on every joint."""
    children = list(root.children)
    pivot = core.empty("motion", root, location=(0, 0, 1.05))
    for child in children:
        child.parent = pivot
        child.location.z -= 1.05
    nodes = {node.name: node for node in root.children_recursive}
    joints = {name: nodes[name] for name in JOINTS}
    rest = {name: node.location.copy() for name, node in joints.items()}
    scene = bpy.context.scene
    scene.render.fps = 30
    offset = 1
    for spec in specs:
        name = spec["name"]
        frames = round(spec["seconds"] * 30)
        for joint, node in joints.items():
            node.animation_data_create()
            node.animation_data.action = None
            for frame in range(frames + 1):
                value = sample(name, frame / frames)
                # The hollow keeps its own head in its left hand. Reach and
                # crush with the free right hand instead of punching with it.
                source = joint
                mirror = calling == "hollow" and name in ("finisherPalm", "finisherSoulCrush")
                if mirror and joint.endswith(("_l", "_r")):
                    source = joint[:-1] + ("l" if joint.endswith("_r") else "r")
                angles = value.get(source, (0, 0, 0))
                if mirror and source != joint:
                    angles = (angles[0], -angles[1], -angles[2])
                gain = AMPLITUDE[calling] if joint != "motion" else 1
                if calling == "mourner" and joint == "head":
                    gain *= 0.25  # Its head mesh includes a floor-length veil.
                node.rotation_euler = tuple(math.radians(angle * gain) for angle in angles)
                node.location = rest[joint] + (Vector(value.get("move", (0, 0, 0))) if joint == "motion" else Vector())
                node.keyframe_insert(data_path="rotation_euler", frame=frame)
                node.keyframe_insert(data_path="location", frame=frame)
            action = node.animation_data.action
            action.name = f"{name}_{joint}"
            slot = node.animation_data.action_slot
            track = node.animation_data.nla_tracks.new()
            track.name = name
            strip = track.strips.new(name, offset, action)
            strip.action_slot = slot
            strip.extrapolation = "NOTHING"
            node.animation_data.action = None
            node.location = rest[joint]
            node.rotation_euler = (0, 0, 0)
        offset += frames + 5
    scene.frame_set(0)
    return joints
