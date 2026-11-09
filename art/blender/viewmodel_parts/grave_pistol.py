"""The grave pistol: a reaper's flintlock revolver, iron on a bone grip.

It is held like every weapon: the grip at the origin running up +Z, so any
hand closed around a haft holds it. The muzzle points toward -Y, and +X is
across the body.

The drum that turns a chamber with every shot is the pivot
`pistol_cylinder`, on the drum's own axis, which the game turns and drops
out; its six souls are the one light on the pistol. `muzzle` is where a
shot leaves the barrel, and holds the three frames of its flash,
`vm_flash_0` to `vm_flash_2`, which the game shows one after another.

Everything else is one mesh, `pistol`, and the drum another, `pistol_drum`:
two draws.
"""

import math

from mathutils import Matrix, Vector

from lib import core

from . import common, effects

# Heights above the grip's origin: the drum's axis, and the bore, which
# lines up with the drum's top chamber.
AXIS_Z = 0.058
BORE_Z = 0.0752
MUZZLE_Y = -0.236


def build(parent):
    root = core.empty("weapon_gravePistol", parent)
    metal = core.material("metal")
    bone = core.material("bone")

    # The grip: a carved bone, deeper than it is wide, its butt swept back
    # under the heel of the hand and swelling where the little finger sits.
    grip_path = core.bezier(
        [(0, 0.034, -0.07), (0, 0.017, -0.042), (0, 0.004, -0.006), (0, -0.002, 0.036)],
        samples=5,
    )
    core.tube(
        "pistol_grip",
        grip_path,
        lambda t: 0.0142 * (1.22 - 0.36 * t + 0.06 * math.sin(t * math.pi * 3) ** 2),
        bone,
        root,
        sides=10,
        flatten=1.3,
    )

    # An iron cap on the butt, like the knuckle at the end of a bone.
    cap = core.revolve(
        "pistol_cap",
        [(0.0, -0.012), (0.008, -0.0115), (0.0145, -0.008), (0.017, -0.002), (0.0155, 0.004), (0.0, 0.006)],
        metal,
        root,
        segments=16,
    )
    cap.data.transform(_at((0, 0.036, -0.073)) @ Matrix.Rotation(math.radians(-24), 4, "X"))

    # The frame: a recoil shield behind the drum, straps above and below it
    # and a post in front that carries the barrel.
    for name, outline, width in (
        ("pistol_shield", [(0.026, 0.033), (0.026, 0.084), (0.012, 0.093), (-0.004, 0.093), (-0.004, 0.033)], 0.031),
        ("pistol_topstrap", [(-0.004, 0.0905), (-0.066, 0.0905), (-0.066, 0.0815), (-0.004, 0.0815)], 0.016),
        ("pistol_post", [(-0.05, 0.034), (-0.05, 0.091), (-0.068, 0.088), (-0.07, 0.04)], 0.027),
        ("pistol_bottomstrap", [(-0.004, 0.0325), (-0.06, 0.034), (-0.06, 0.0375), (-0.004, 0.037)], 0.022),
    ):
        common.side_plate(name, outline, width, metal, root)

    # The drum, on a pivot at its own middle so the game can turn it.
    cylinder = core.empty("pistol_cylinder", root, location=(0, -0.027, AXIS_Z))
    common.soul_drum("pistol_drum", cylinder)

    # An octagonal barrel, flaring at the mouth like a blunderbuss, with a
    # crown of teeth round the bore.
    barrel = core.tube(
        "pistol_barrel",
        [(0, -0.064, BORE_Z), (0, -0.15, BORE_Z), (0, -0.215, BORE_Z)],
        0.0112,
        metal,
        root,
        sides=8,
    )

    # Iron bands round the barrel, where a gunsmith would have shrunk them on.
    for index, y in enumerate((-0.072, -0.128, -0.19)):
        band = core.revolve(
            f"pistol_band_{index}",
            [(0.0118, -0.004), (0.0136, -0.003), (0.0139, 0.0), (0.0136, 0.003), (0.0118, 0.004)],
            metal,
            root,
            segments=10,
        )
        band.data.transform(_axis_forward((0, y, BORE_Z)))

    flare = core.revolve(
        "pistol_flare",
        [(0.0099, 0.0), (0.0112, 0.008), (0.0138, 0.016), (0.0162, 0.021), (0.0155, 0.0235), (0.0085, 0.022)],
        metal,
        root,
        segments=16,
    )
    flare.data.transform(_axis_forward((0, -0.215, BORE_Z)))

    for tooth in range(6):
        angle = math.tau * tooth / 6 + 0.26
        across = Vector((math.cos(angle), 0, math.sin(angle)))
        base = Vector((0, MUZZLE_Y + 0.004, BORE_Z)) + across * 0.0128
        core.tube(
            f"pistol_tooth_{tooth}",
            [base, base + Vector((0, -0.009, 0)) - across * 0.002],
            lambda t: 0.0028 * (1 - 0.8 * t),
            bone,
            root,
            sides=4,
        )

    # A jawbone slung under the barrel, from the frame to near the mouth.
    core.tube(
        "pistol_jaw",
        core.bezier(
            [(0, -0.064, 0.052), (0, -0.11, 0.054), (0, -0.165, 0.061), (0, -0.196, 0.0655)],
            samples=4,
        ),
        lambda t: 0.0068 * (1 - 0.45 * t) * (1 + 0.15 * math.sin(t * math.pi * 6) ** 2),
        bone,
        root,
        sides=8,
        flatten=0.75,
    )

    # A bone tooth for a front sight.
    core.tube(
        "pistol_sight",
        [(0, -0.198, BORE_Z + 0.009), (0, -0.201, BORE_Z + 0.017)],
        lambda t: 0.0026 * (1 - 0.7 * t),
        bone,
        root,
        sides=6,
    )

    # The flintlock's cock, curled up behind the drum, its jaws closed on
    # a flint.
    core.tube(
        "pistol_cock",
        core.bezier(
            [(0, 0.017, 0.088), (0, 0.032, 0.098), (0, 0.03, 0.114), (0, 0.016, 0.118), (0, 0.006, 0.109)],
            samples=3,
        ),
        lambda t: 0.0048 * (1 - 0.25 * t),
        metal,
        root,
        sides=7,
    )
    flint = core.plate(
        "pistol_flint",
        [(-0.004, 0.103), (0.004, 0.104), (0.0045, 0.112), (-0.0035, 0.113)],
        0.009,
        metal,
        root,
    )
    flint.data.transform(Matrix.Translation((0, 0.004, 0)))

    # A trigger in a guard of bent iron.
    core.tube(
        "pistol_guard",
        core.bezier(
            [(0, 0.0, 0.034), (0, -0.006, 0.014), (0, -0.026, 0.009), (0, -0.046, 0.018), (0, -0.052, 0.034)],
            samples=3,
        ),
        0.0026,
        metal,
        root,
        sides=5,
    )
    core.tube(
        "pistol_trigger",
        core.bezier([(0, -0.024, 0.034), (0, -0.02, 0.024), (0, -0.013, 0.017)], samples=4),
        lambda t: 0.0032 * (1 - 0.4 * t),
        metal,
        root,
        sides=6,
        flatten=0.6,
    )

    muzzle = core.empty("muzzle", root, location=(0, MUZZLE_Y, BORE_Z))
    effects.muzzle_flashes(muzzle)
    common.merge(root, "pistol")

    return root


def _at(offset):
    return Matrix.Translation(offset)


def _axis_forward(offset):
    """Turns a part revolved about +Z onto the bore, pointing toward -Y, and
    moves it to `offset`."""

    return Matrix.Translation(offset) @ Matrix.Rotation(math.radians(90), 4, "X")
