"""The soul capture orb: an iron cup with six bone claws closed over a
core of light.

The orb's middle is the origin and Z is up. The claws are horns: narrow
D-sections hinged on the rim, spaced wide apart there, rising up the core's
side and arching in over it to points well above it that hook down toward
the light, so the core shows between them as six wedges of light and the
orb's outline is a crown at 64 pt (art bible 6.5). One claw, the thumb, is
half as long again and hooks right over the top past the axis: the orb's one
asymmetry. The cup is iron, its sigil iron too until the game burns it in,
so the core is the only light on the orb.

It is three draws:

- `orb_body`, the cup, its studs and the six claws, as one mesh. Two morph
  targets move the claws: `open` turns each one outward about its hinge on
  the rim, as the orb lands and drinks a soul in; `burst` flings them out
  and cracks the cup, when the soul breaks free.
- `orb_core`, the soul light inside, unlit, for the game to tint and swell.
- `orb_sigil`, the ring of runes round the cup that burns in when a soul is
  bound: iron, until the game lights it gold.

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
CORE_RADIUS = 0.041
TALONS = 6

# Each claw's spine, (distance out from the axis, height), from its hinge
# on the rim to its point: up the core's side, arching in over it, and
# hooked down at its point, which stands well above the core (its top is
# CORE_RADIUS). The thumb's is half as long again and runs on over the top,
# past the axis.
TALON = ((0.0635, -0.0185), (0.0690, -0.0040), (0.0705, 0.0100), (0.0660, 0.0255), (0.0565, 0.0385), (0.0445, 0.0465), (0.0335, 0.0480), (0.0270, 0.0425))
THUMB = ((0.0635, -0.0185), (0.0700, -0.0030), (0.0720, 0.0140), (0.0670, 0.0320), (0.0545, 0.0480), (0.0370, 0.0590), (0.0170, 0.0630), (0.0010, 0.0600), (-0.0075, 0.0520))
# Which claw is the thumb: the one toward the eye as the hand holds it.
THUMB_INDEX = 4
# Half a claw's width at its root, and its depth: six of them leave a gap
# of four centimetres at the rim, where the core shows through.
ROOT_HALF_WIDTH = 0.0135
ROOT_DEPTH = 0.0085
# A talon's section: a D, flat inside against the core, rounded outside.
SECTION = ((-1.0, 0.0), (-0.72, 0.62), (0.0, 1.0), (0.72, 0.62), (1.0, 0.0), (0.0, -0.32))
HELD_SECTION = ((-1.0, 0.0), (0.0, 1.0), (1.0, 0.0), (0.0, -0.32))

# How far `open` turns each talon out about its hinge.
OPEN_DEGREES = 72

# How small the orb is in the hand: its cup the size of the spare drum.
HELD_SCALE = 0.4


def build(parent=None):
    root = core.empty("capture_orb", parent)
    _cup(root, segments=24)
    _talons(root, SECTION, stations=9)
    common.merge(root, "orb_body")

    core_node = core.empty("orb_core", root)
    _core(core_node, subdivisions=2).data.materials[0] = effects.light_material()

    # Iron, so the cup prints in key and the core is the orb's one light;
    # the game burns it in gold when a soul is bound.
    sigil = core.empty("orb_sigil", root)
    common.paint(
        core.revolve(
            "orb_sigil_ring",
            [(0.0636, -0.0245), (0.0658, -0.0256), (0.0664, -0.030), (0.0650, -0.0342), (0.0628, -0.0352)],
            core.material("metal"),
            sigil,
            segments=36,
            shape=lambda angle, radius, height: (
                radius + 0.0018 * (math.sin(angle * 12) > 0.30) * (abs(height + 0.03) < 0.0036),
                height,
            ),
        )
    )

    return root


def build_held(parent):
    root = core.empty("viewmodel_orb_held", parent)
    _cup(root, segments=10, keys=False, studs=False)
    _talons(root, HELD_SECTION, stations=5, keys=False)
    _core(root, subdivisions=1)
    held = common.merge(root, "orb_held")
    held.data.transform(Matrix.Scale(HELD_SCALE, 4))

    return root


def _core(parent, subdivisions):
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=subdivisions, radius=CORE_RADIUS, location=(0, 0, 0))
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

    # Studs on the cup under the gaps between the talons, too small to see
    # in the hand.
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


def _spine(points, share):
    """A point `share` of the way along a talon's spine, eased through its
    points, as (out, height)."""

    place = share * (len(points) - 1)
    index = min(int(place), len(points) - 2)
    local = place - index
    smooth = local * local * (3 - 2 * local)
    (r0, z0), (r1, z1) = points[index], points[index + 1]

    return r0 + (r1 - r0) * smooth, z0 + (z1 - z0) * smooth


def _talons(parent, section, stations, keys=True):
    bone = core.material("bone")

    for talon in range(TALONS):
        angle = math.tau * (talon + 0.5) / TALONS
        out = Vector((math.cos(angle), math.sin(angle), 0.0))
        across = Vector((-math.sin(angle), math.cos(angle), 0.0))
        up = Vector((0, 0, 1))
        points = THUMB if talon == THUMB_INDEX else TALON
        rings = []

        for station in range(stations):
            share = station / stations
            r, z = _spine(points, share)
            ahead = _spine(points, min(share + 0.02, 1.0))
            behind = _spine(points, max(share - 0.02, 0.0))
            # Square to the spine, in the plane of the axis: outward.
            dr, dz = ahead[0] - behind[0], ahead[1] - behind[1]
            length = math.hypot(dr, dz)
            normal = out * (dz / length) - up * (dr / length)
            centre = out * r + up * z
            half = ROOT_HALF_WIDTH * (1 - share) ** 0.8 + 0.0012
            thick = ROOT_DEPTH * (1 - 0.7 * share) + 0.0008
            # Broad near the rim, so it is dished to the cup's round.
            # (The thumb crosses the axis, so the round is never taken
            # tighter than the core's.)
            dish = half * half / (2 * max(abs(r), CORE_RADIUS))
            rings.append([centre + across * u * half + normal * (v * thick - u * u * dish) for u, v in section])

        r, z = points[-1]
        claw = common.skin(f"orb_talon_{talon}", rings, bone, parent, tip=out * r + up * z, smooth=True)

        if not keys:
            continue

        # Each talon turns out about the rim's tangent at its hinge.
        hinge = out * RIM_RADIUS + up * RIM_Z
        opened = Matrix.Translation(hinge) @ Matrix.Rotation(math.radians(OPEN_DEGREES), 4, across) @ Matrix.Translation(-hinge)
        flung = (
            Matrix.Translation(out * 0.09 + Vector((0, 0, 0.05)))
            @ Matrix.Translation(hinge)
            @ Matrix.Rotation(math.radians(125), 4, across)
            @ Matrix.Rotation(math.radians(40 * (1 if talon % 2 else -1)), 4, out)
            @ Matrix.Translation(-hinge)
        )
        core.add_shape_key(claw, "open", lambda co, m=opened: m @ co)
        core.add_shape_key(claw, "burst", lambda co, m=flung: m @ co)
