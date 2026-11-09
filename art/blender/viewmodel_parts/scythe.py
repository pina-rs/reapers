"""The scythe, as the reaper holds it in its own hand: the harvester's
weapon from `weapon_kinds/scythe.py`, built at its full size and merged into
one mesh, `scythe`, under `weapon_scythe`. The game shrinks and leans it in
the fist, as it does every heavy weapon.

The harvester's scythe is modelled to be seen whole at a distance. In the
reaper's own hand it shares the view with both arms and the orb, so its
larger parts keep a little over half their faces: a throw with the scythe in
hand is the heaviest moment the view has."""

import importlib

from lib import core

from . import common

# The share of its faces a large part of the scythe keeps.
KEEP = 0.55
# A part with fewer faces than this is left whole.
SMALL = 200


def build(parent):
    root = importlib.import_module("weapon_kinds.scythe").build()
    root.parent = parent
    for part in root.children:
        if part.type == "MESH" and len(part.data.polygons) > SMALL:
            core.decimate(part, KEEP)
    common.merge(root, "scythe")

    return root
