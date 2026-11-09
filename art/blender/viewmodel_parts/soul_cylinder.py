"""A spare soul drum for the grave pistol: the magazine the left hand swaps
in when it reloads. The same drum as the pistol's, on its own node, with its
axis along -Y through the origin, and one mesh, `soul_cylinder`."""

from lib import core

from . import common


def build(parent):
    root = core.empty("viewmodel_soul_cylinder", parent)
    common.soul_drum("soul_cylinder", root)

    return root
