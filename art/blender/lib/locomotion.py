"""Grounded foot targets and two-bone IK, baked into ordinary glTF joints.

Blender is Z-up; forward is -Y. Knees point forward, never behind the leg.
The renderer needs no runtime solver. In-place stance feet move backwards at
constant speed, as on a treadmill, while the returning foot clears the floor.
"""

import math
from mathutils import Euler, Quaternion, Vector

LENGTH = 0.44
HIP_X = 0.125
HIP_Z = 1.01
ANKLE_Z = 0.13
GAITS = {
    "creep": (0.18, 0.055, 0.70, 0.10),
    "walk": (0.29, 0.12, 0.62, 0.065),
    "run": (0.42, 0.27, 0.39, 0.11),
    "sprint": (0.50, 0.36, 0.32, 0.14),
}


def smooth(t):
    """Endpoint-flat transition for lifts, settling and weight changes."""
    t = max(0, min(1, t))
    return t * t * (3 - 2 * t)


def envelope(p, start, peak, end):
    """A deliberate rise and slower release with no discontinuity."""
    return smooth((p - start) / (peak - start)) if p < peak else 1 - smooth((p - peak) / (end - peak))


def foot_cycle(name, phase, side):
    """Ankle pose with heel contact, flat support, toe off and swing clearance."""
    stride, lift, stance, _ = GAITS[name]
    p = (phase + (0.5 if side == "r" else 0)) % 1
    roll = 0
    if p < stance:
        u = p / stance
        y = -stride + 2 * stride * u
        roll = -16 * (1 - smooth(u / 0.16)) + 30 * smooth((u - 0.77) / 0.23)
        z = 0
    else:
        u = (p - stance) / (1 - stance)
        # Match the backwards stance velocity at both swing boundaries.
        slope = 2 * stride * (1 - stance) / stance
        y = stride * (1 - 2 * smooth(u)) + slope * (2 * u**3 - 3 * u**2 + u)
        z = lift * math.sin(math.pi * u) ** 2
        roll = 30 * (1 - smooth(u / 0.4)) - 16 * smooth((u - 0.7) / 0.3)
    rotation = Euler((math.radians(roll), 0, 0)).to_quaternion()
    sole = Vector((0, -0.20 if roll > 0 else 0.065, -ANKLE_Z))
    rotated = rotation @ sole
    position = Vector((HIP_X * (1 if side == "l" else -1), y + sole.y - rotated.y, z - rotated.z))
    return position, rotation


def solve(hip, ankle, pole):
    """Two equal segments, a forward knee pole and a small extension reserve."""
    direction = ankle - hip
    distance = direction.length
    if distance > LENGTH * 2 + 0.002:
        raise ValueError(f"Unreachable ankle: {distance:.4f} > {2 * LENGTH}")
    distance = min(distance, LENGTH * 2 - 0.0002)
    direction.normalize()
    forward = pole - direction * pole.dot(direction)
    forward.normalize()
    knee = hip + direction * (distance / 2) + forward * math.sqrt(max(0, LENGTH**2 - (distance / 2) ** 2))
    down = Vector((0, 0, -1))
    thigh = down.rotation_difference((knee - hip).normalized())
    shin = down.rotation_difference((ankle - knee).normalized())
    return thigh, thigh.inverted() @ shin, shin


def grounded(name, phase, performance):
    """Root translation and world foot poses for a humanoid performance."""
    p = phase
    feet = {}
    yaw = 0.0
    if name in GAITS:
        _, _, stance, crouch = GAITS[name]
        # Two compressions per cycle. Running has a real flight interval.
        bob = 0.025 * math.cos(4 * math.pi * (p - 0.04))
        root = Vector((0.024 * math.cos(math.tau * p), 0, -crouch + bob))
        yaw = 5 * math.cos(math.tau * p)
        for side in ("l", "r"):
            feet[side] = foot_cycle(name, p, side)
        if name in ("run", "sprint"):
            half = p % 0.5
            flight = math.sin(math.pi * max(0, (half - stance) / (0.5 - stance))) ** 2
            root.z += 0.07 * flight
        return root, yaw, feet

    energy = math.sin(math.pi * p) ** 2
    move = Vector(performance.get("move", (0, 0, 0)))
    root = Vector((0, 0, -0.045))
    travel = Vector()
    step = 0.0
    if name in ("jump", "finisherUppercut"):
        takeoff, touchdown = (0.22, 0.79) if name == "jump" else (0.34, 0.76)
        flight = max(0, min(1, (p - takeoff) / (touchdown - takeoff)))
        height = (0.48 if name == "jump" else 0.28) * 4 * flight * (1 - flight)
        squat = 0.17 * envelope(p, 0, takeoff, takeoff + 0.12) + 0.16 * envelope(p, touchdown - 0.02, touchdown + 0.04, 1)
        root.z += height - squat
        for side in ("l", "r"):
            lift = height + 0.10 * math.sin(math.pi * flight) ** 2
            feet[side] = (Vector((HIP_X * (1 if side == "l" else -1), 0, ANKLE_Z + lift)), Quaternion())
        return root, 0, feet
    if name == "land":
        root.z -= 0.20 * envelope(p, 0, 0.22, 1)
    elif name in ("dash", "dashLeft", "dashRight", "dodgeLeft", "dodgeRight"):
        travel = Vector((move.x, move.y, 0))
        root += travel
        root.z -= 0.09 * energy
        step = envelope(p, 0.12, 0.45, 0.82)
    elif name in ("slash", "thrust", "parry", "finisherReap", "finisherImpale", "finisherPalm", "finisherSoulCrush", "animeCharge"):
        root.y = max(-0.13, min(0.08, move.y * 0.35))
        root.x = 0.035 * math.sin(math.tau * p)
        root.z -= 0.075 * energy
        yaw = performance.get("motion", (0, 0, 0))[2] * 0.18
        step = envelope(p, 0.05, 0.34, 1)
    elif name == "fear":
        root.y = 0.07 * energy
        root.z -= 0.07 * energy
        step = 0.45 * energy
    else:
        root.z -= 0.018 * energy
        if name == "idle":
            root.x = 0.008 * math.sin(math.tau * p)
            root.z += 0.004 * math.sin(math.tau * p)

    for side, sign in (("l", 1), ("r", -1)):
        position = Vector((sign * (HIP_X + step * 0.055), -sign * step * 0.17, ANKLE_Z)) + travel
        if travel.length > 0:
            position.z += (0.045 if side == "l" else 0.075) * math.sin(math.pi * step) ** 2
        else:
            # Step into the stance and lift again to recover; hold at contact.
            position.z += 0.05 * math.sin(math.pi * step) ** 2
        feet[side] = (position, Euler((0, 0, math.radians(-sign * step * 9))).to_quaternion())
    return root, yaw, feet
