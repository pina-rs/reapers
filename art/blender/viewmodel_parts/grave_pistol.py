"""The grave pistol: an open-topped revolver whose barrel is a coffin, nailed
shut, on a grip of four iron vertebrae.

It is held like every weapon: the grip at the origin running up +Z, so any
hand closed around a haft holds it. The muzzle points toward -Y, and +X is
across the body (the left side, where the thumb lies).

The barrel's shroud is a coffin in plan and in section (art bible 6.5): an
irregular six-sided profile, widest a third of the way along and tapering to
a muzzle of bone, with three bone nails down its lid right of centre and a
band of the reaper's light (`gold`) round its widest ring.

The drum is the pivot `pistol_cylinder`, on the drum's own axis, which the
game turns a sixth with each shot and drops out to reload. It is one mesh,
`pistol_drum`, and it holds the ammunition count: its six chambers face the
eye, each lit while loaded, and each has a morph target, `spent_<n>`, that
puts its light out and leaves the void, with an empty, `pistol_chamber_<n>`,
where the game hangs its halo. Chamber 0 is under the bore.

`muzzle` is where a shot leaves the barrel, and holds the three frames of
its flash, `vm_flash_0` to `vm_flash_2`, which the game shows one after
another.

Everything else is one mesh, `pistol`: with the drum, two draws.
"""

import math

from lib import core

from . import common, effects

# The drum's axis, from the grip; the bore, which lines up with the drum's
# top chamber; and the muzzle (`viewmodel.rs` DRUM and MUZZLE, which a test
# reads back out of the model).
DRUM_Y = -0.027
AXIS_Z = 0.058
BORE_Z = AXIS_Z + common.CHAMBER_RING
MUZZLE_Y = -0.236

# The coffin in section, (across, up) in its own size: narrow at the lid,
# widest at the shoulders a little below it, then running in to the base.
COFFIN = [(-0.55, 0.80), (0.55, 0.80), (1.0, 0.20), (0.66, -1.0), (-0.66, -1.0), (-1.0, 0.20)]

# The coffin in plan: its size at points along the barrel, from just clear
# of the drum's front to the muzzle (art bible 6.5), widest a third along.
SHROUD = ((-0.051, 0.021), (-0.087, 0.030), (-0.126, 0.027), (-0.170, 0.021), (-0.207, 0.017))
WIDEST = -0.087
SHROUD_Z = 0.0735


def build(parent):
    root = core.empty("weapon_gravePistol", parent)
    metal = core.material("metal")
    bone = core.material("bone")

    _shroud(root, metal, bone)
    _frame(root, metal, bone)

    # The grip: four vertebrae raked back down from under the frame, and an
    # iron knob for a butt.
    common.vertebrae("pistol_grip", root, top=0.016, bottom=-0.070, rake=0.12)
    common.section(
        "pistol_butt",
        [((0, 0.0105, -0.071), 0.0135, 0.0145), ((0, 0.0112, -0.080), 0.0145, 0.0155), ((0, 0.0118, -0.088), 0.0095, 0.010)],
        common.polygon(6, math.pi / 6),
        metal,
        root,
        across=(1, 0, 0),
        up=(0, 1, 0),
    )

    # The drum, on a pivot at its own middle so the game can turn it.
    cylinder = core.empty("pistol_cylinder", root, location=(0, DRUM_Y, AXIS_Z))
    common.drum("pistol_drum", cylinder, chambers="pistol_chamber")

    muzzle = core.empty("muzzle", root, location=(0, MUZZLE_Y, BORE_Z))
    effects.muzzle_flashes(muzzle)
    common.merge(root, "pistol")

    return root


def _shroud_size(y):
    """The shroud's size `y` along the barrel, between its stations."""

    for (near, a), (far, b) in zip(SHROUD, SHROUD[1:]):
        if far <= y <= near:
            return a + (b - a) * (near - y) / (near - far)

    return SHROUD[0][1] if y > SHROUD[0][0] else SHROUD[-1][1]


def _shroud(root, metal, bone):
    """The coffin, its band, its lid's nails, its sight and its muzzle."""

    common.section("pistol_shroud", [((0, y, SHROUD_Z), size, size) for y, size in SHROUD], COFFIN, metal, root)

    # The reaper's light round the coffin's widest ring: a band a little
    # proud of it, its edges turned.
    band = []

    for offset, proud in ((0.0075, 0.0004), (0.006, 0.0019), (-0.006, 0.0019), (-0.0075, 0.0004)):
        y = WIDEST + offset
        size = _shroud_size(y) + proud
        band.append(((0, y, SHROUD_Z), size, size))

    common.section("pistol_band", band, COFFIN, core.material("gold"), root)

    # Three bone nails down the lid, right of centre.
    for index, y in enumerate((-0.108, -0.140, -0.170)):
        lid = SHROUD_Z + 0.8 * _shroud_size(y) - 0.0006
        common.section(
            f"pistol_nail_{index}",
            [((-0.0075, y, lid), 0.0048, 0.0048), ((-0.0075, y, lid + 0.0024), 0.0044, 0.0044), ((-0.0075, y, lid + 0.0036), 0.0022, 0.0022)],
            common.polygon(6),
            bone,
            root,
            across=(1, 0, 0),
            up=(0, 1, 0),
            smooth=True,
        )

    # A bone tooth for a sight.
    lid = SHROUD_Z + 0.8 * _shroud_size(-0.199) - 0.0006
    common.section(
        "pistol_sight",
        [((0, -0.199, lid), 0.0034, 0.0062), ((0, -0.200, lid + 0.0062), 0.0022, 0.0040), ((0, -0.201, lid + 0.0098), 0.0007, 0.0014)],
        common.polygon(4, math.pi / 4),
        bone,
        root,
        across=(1, 0, 0),
        up=(0, 1, 0),
    )

    # The muzzle: a lip of bone round the bore, the coffin's foot.
    common.section(
        "pistol_muzzle",
        [((0, y, BORE_Z), size, size) for y, size in ((-0.203, 0.0132), (-0.209, 0.0150), (-0.2335, 0.0150), (-0.2362, 0.0136))],
        common.polygon(10, math.pi / 10),
        bone,
        root,
        smooth=True,
    )
    common.section("pistol_bore", [((0, -0.2350, BORE_Z), 0.0088, 0.0088), ((0, -0.2366, BORE_Z), 0.0088, 0.0088)], common.polygon(10, math.pi / 10), core.material("void"), root)


def _frame(root, metal, bone):
    """The open frame under the drum, the post that carries the barrel, the
    hammer curled up behind the drum's right, the trigger and a bone guard."""

    # The frame's flat side, (y, z): up the back of the grip, under the drum
    # and up the post in front of it, under the coffin.
    common.side_plate(
        "pistol_frame",
        [
            (0.021, 0.002),
            (0.023, 0.028),
            (0.015, 0.040),
            (0.004, 0.037),
            (-0.004, 0.0305),
            (-0.050, 0.0305),
            (-0.051, 0.047),
            (-0.064, 0.052),
            (-0.069, 0.032),
            (-0.062, 0.019),
            (-0.014, 0.0185),
            (0.002, 0.006),
        ],
        0.020,
        metal,
        root,
        bevel=0.0,
    )

    # The hammer: up behind the drum's right edge, clear of its chambers as
    # the eye sees them, over its top corner and curled back into a spur.
    core.tube(
        "pistol_hammer",
        core.bezier(
            [(-0.010, 0.016, 0.040), (-0.022, 0.010, 0.058), (-0.023, 0.008, 0.076), (-0.015, 0.011, 0.088), (-0.010, 0.021, 0.092), (-0.010, 0.030, 0.087)],
            samples=2,
        ),
        lambda t: 0.0064 - 0.0026 * t,
        metal,
        root,
        sides=6,
        flatten=0.75,
    )

    core.tube(
        "pistol_trigger",
        core.bezier([(0, -0.019, 0.020), (0, -0.023, 0.011), (0, -0.021, 0.002)], samples=2),
        lambda t: 0.0034 - 0.001 * t,
        metal,
        root,
        sides=5,
        flatten=0.6,
    )

    # The guard: a hook of bone in front of the index finger.
    core.tube(
        "pistol_guard",
        core.bezier([(0, -0.060, 0.022), (0, -0.069, 0.008), (0, -0.064, -0.007), (0, -0.050, -0.011)], samples=2),
        lambda t: 0.0038 - 0.0012 * t,
        bone,
        root,
        sides=6,
    )
