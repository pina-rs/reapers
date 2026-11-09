"""The soul capture orb: an iron cup with six bone talons closed over a
core of light.

The orb's middle is the origin and Z is up. It is three draws:

- `orb_body`, the cup, its studs and the six talons, as one mesh. Two morph
  targets move the talons: `open` turns each one outward about its hinge on
  the rim, as the orb lands and drinks a soul in; `burst` flings them out
  and cracks the cup, when the soul breaks free.
- `orb_core`, the soul light inside, unlit, for the game to tint and swell.
- `orb_sigil`, the ring of runes round the cup that burns in when a soul is
  bound: gold, until the game lights it.

`build_held` makes the small copy the left hand carries before a throw,
merged into one mesh, `orb_held`, under `viewmodel_orb_held`.
"""

import math

import bpy
from mathutils import Matrix, Vector

from lib import core

from . import common, effects

RADIUS = 0.07
RIM_Z = -0.017
RIM_RADIUS = 0.064
TALONS = 6

# How far `open` turns each talon out about its hinge.
OPEN_DEGREES = 72

# How small the orb is in the hand: its cup the size of the spare drum.
HELD_SCALE = 0.4


def build(parent=None):
    root = core.empty("capture_orb", parent)
    _cup(root, segments=24)
    _talons(root, sides=6, samples=3)
    common.merge(root, "orb_body")

    core_node = core.empty("orb_core", root)
    _core(core_node, subdivisions=2).data.materials[0] = effects.light_material()

    sigil = core.empty("orb_sigil", root)
    common.paint(
        core.revolve(
            "orb_sigil_ring",
            [(0.0638, -0.0255), (0.0652, -0.0262), (0.0656, -0.03), (0.0646, -0.0338), (0.063, -0.0345)],
            core.material("gold"),
            sigil,
            segments=36,
            shape=lambda angle, radius, height: (
                radius + 0.0012 * (math.sin(angle * 12) > 0.35) * (abs(height + 0.03) < 0.003),
                height,
            ),
        )
    )

    return root


def build_held(parent):
    root = core.empty("viewmodel_orb_held", parent)
    _cup(root, segments=10, keys=False, studs=False)
    _talons(root, sides=4, samples=2, keys=False)
    _core(root, subdivisions=1)
    held = common.merge(root, "orb_held")
    held.data.transform(Matrix.Scale(HELD_SCALE, 4))

    return root


def _core(parent, subdivisions):
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=subdivisions, radius=0.041, location=(0, 0, 0))
    light = bpy.context.object
    light.name = f"{parent.name}_light"
    light.data.name = light.name
    light.data.materials.append(core.material("accent"))
    light.parent = parent
    core.shade_smooth(light)

    return light


def _cup(parent, segments, keys=True, studs=True):
    metal = core.material("metal")

    # Round-bottomed, so a pulse can rock it where it lies, with a rolled lip
    # the talons hinge on.
    cup = core.revolve(
        "orb_cup",
        [
            (0.0, -RADIUS),
            (0.016, -RADIUS + 0.001),
            (0.033, -0.064),
            (0.047, -0.054),
            (0.057, -0.042),
            (0.063, -0.03),
            (0.0665, -0.021),
            (0.067, RIM_Z + 0.001),
            (0.063, RIM_Z + 0.0035),
            (0.058, RIM_Z),
            (0.054, -0.026),
            (0.0, -0.03),
        ],
        metal,
        parent,
        segments=segments,
    )
    parts = [cup]

    # Studs on the cup between the talons' hinges, too small to see in the
    # hand.
    for stud in range(TALONS if studs else 0):
        angle = math.tau * stud / TALONS
        out = Vector((math.cos(angle), math.sin(angle), 0.0))
        rivet = core.revolve(
            f"orb_stud_{stud}",
            [(0.0, 0.0045), (0.0035, 0.0035), (0.0055, 0.0), (0.0045, -0.003), (0.0, -0.003)],
            metal,
            parent,
            segments=6,
        )
        rivet.data.transform(
            Matrix.Translation(out * 0.0585 + Vector((0, 0, -0.044))) @ out.to_track_quat("Z", "Y").to_matrix().to_4x4()
        )
        parts.append(rivet)

    if keys:
        # The cup barely moves when the orb opens; when it bursts it cracks
        # wide and sinks.
        for part in parts:
            core.add_shape_key(part, "open", lambda co: co)
            core.add_shape_key(part, "burst", lambda co: Vector((co.x * 1.12, co.y * 1.12, co.z * 0.9 - 0.01)))


def _talons(parent, sides, samples, keys=True):
    bone = core.material("bone")

    # Six talons, curved over the core like the fingers of a closed claw.
    for talon in range(TALONS):
        angle = math.tau * (talon + 0.5) / TALONS
        out = Vector((math.cos(angle), math.sin(angle), 0.0))
        hinge = out * RIM_RADIUS + Vector((0, 0, RIM_Z))
        # Round the core on a sphere a little wider than the cup, from just
        # under the rim to just short of the pole, where the tip hooks in.
        cage = RADIUS + 0.004
        centre = Vector((0, 0, -RIM_Z)) - out * RIM_RADIUS
        path = core.bezier(
            [Vector((0, 0, 0))]
            + [
                centre + (out * math.cos(math.radians(theta)) + Vector((0, 0, math.sin(math.radians(theta))))) * cage
                for theta in (8, 32, 56, 74)
            ]
            + [centre + out * 0.01 + Vector((0, 0, cage * 0.93))],
            samples=samples,
        )
        path = [hinge + point for point in path]
        claw = core.tube(
            f"orb_talon_{talon}",
            path,
            lambda t: 0.0068 * (1 - 0.8 * t) * (1 + 0.25 * abs(math.sin(t * math.pi * 3)) ** 4),
            bone,
            parent,
            sides=sides,
        )

        if not keys:
            continue

        # Each talon turns out about the rim's tangent at its hinge.
        tangent = Vector((0, 0, 1)).cross(out)
        opened = Matrix.Translation(hinge) @ Matrix.Rotation(math.radians(OPEN_DEGREES), 4, tangent) @ Matrix.Translation(-hinge)
        flung = (
            Matrix.Translation(out * 0.09 + Vector((0, 0, 0.05)))
            @ Matrix.Translation(hinge)
            @ Matrix.Rotation(math.radians(125), 4, tangent)
            @ Matrix.Rotation(math.radians(40 * (1 if talon % 2 else -1)), 4, out)
            @ Matrix.Translation(-hinge)
        )
        core.add_shape_key(claw, "open", lambda co, m=opened: m @ co)
        core.add_shape_key(claw, "burst", lambda co, m=flung: m @ co)
