"""A spare soul drum for the grave pistol: the magazine the left hand swaps
in when it reloads. The same drum as the pistol's, every chamber loaded, on
its own node through the origin, and one mesh, `soul_cylinder`.

The drum's chambers face the eye: in a reload the rig lifts the left hand
so the eye looks into its palm along TO_EYE (in the palm's own frame,
measured from the rig), and the drum lies in the cup with its axis along
it, so its six lights show over the fingers instead of its blank side."""

from mathutils import Vector

from lib import core

from . import common

# Where the eye is from the palm in a reload, in the palm's frame.
TO_EYE = Vector((-0.85, 0.39, 0.36))


def build(parent):
    root = core.empty("viewmodel_soul_cylinder", parent)
    drum = common.drum("soul_cylinder", root)
    # The drum is built with its chambers facing +Y.
    drum.data.transform(Vector((0, 1, 0)).rotation_difference(TO_EYE.normalized()).to_matrix().to_4x4())

    return root
