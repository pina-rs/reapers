"""The scythe, as the reaper holds it in its own hand: the harvester's
weapon from `weapon_kinds/scythe.py`, merged into one mesh, `scythe`, under
`weapon_scythe`. The game shrinks and leans it in the fist, as it does every
heavy weapon.

The harvester's scythe is modelled to be seen whole at a distance, its
snath taller than its bearer. Held that way in front of the eye, the snath
runs up through the sight and the crescent hangs over the middle of the
view, which the eye view forbids (art bible 6.5). So in the reaper's own
hand the snath is cut short, HEAD metres above the grip, and the head (the
crescent, its seam, the collar and the spur) is set on it SCALE times its
size and turned TURN about the snath, so the crescent rises out to the
right of the view and sweeps off its edge, away from the sight, while the
rig's hold stays as it is. The gold band under the fist is the weapon's
own.

It shares the view with both arms and the orb, so it sheds what the eye
cannot see: the rope of the grip, inside the fist, and the lashing under
the collar, a few points of noise at this size. Its larger parts keep a
third to a half of their faces: a throw with the scythe in hand is the
heaviest moment the view has."""

import importlib
import math

import bpy

from mathutils import Matrix

from lib import core

from . import common

# How high above the grip the head sits on the short snath, how big it is
# against the weapon's, and how far it is turned about the snath.
HEAD = 0.4
SCALE = 0.56
TURN = math.radians(122)
# Where the weapon's head meets its snath: the collar's top.
COLLAR_TOP = 1.5
# The weapon's parts that are the head, and are moved onto the short snath.
HEAD_PARTS = ("scythe_blade", "scythe_edge", "scythe_collar", "scythe_hook")
# The weapon's parts the reaper's own hand does without: its long snath
# (replaced by a short one), the rope inside the fist, and the lashing.
DROPPED = ("scythe_haft", "scythe_grip", "scythe_lashing")
# Where the short snath ends below the grip: out of the view.
FOOT = -0.45

# The share of its faces each of the scythe's larger parts keeps; the
# rest (the seam of light, the band, the snath) are kept whole.
KEEP = {"scythe_blade": 0.36, "scythe_hook": 0.5, "scythe_collar": 0.5}


def build(parent):
    root = importlib.import_module("weapon_kinds.scythe").build()
    root.parent = parent
    placed = Matrix.Translation((0, 0, HEAD)) @ Matrix.Rotation(TURN, 4, "Z") @ Matrix.Scale(SCALE, 4) @ Matrix.Translation((0, 0, -COLLAR_TOP))

    for part in list(root.children):
        if part.type != "MESH":
            continue

        if part.name in DROPPED:
            bpy.data.objects.remove(part)
            continue

        if part.name in HEAD_PARTS:
            own = part.matrix_basis.copy()
            part.data.transform(own.inverted() @ placed @ own)

        if part.name in KEEP:
            core.decimate(part, KEEP[part.name])

    # The short snath: straight, thinning a little toward the head.
    top = HEAD - (COLLAR_TOP - 1.245) * SCALE
    snath = core.tube("scythe_haft", [(0, 0, FOOT), (0, 0, 0.0), (0, 0, top + 0.01)], lambda t: 0.022 - 0.004 * t, core.material("wood"), root, sides=8)
    core.shade_smooth(snath, angle=40)
    common.merge(root, "scythe")

    return root

