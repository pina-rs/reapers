"""The harvester: the field hand of death.

Silhouette: a wide straw hat on a body bent double, the scythe leant on
like a staff with its crescent arching over the brim. Narrow shoulders slope
away under the brim, which sits almost on them, and the robe spreads below
to a heavy hem: the shape of a man bent by a lifetime of the same work. The
head is thrust out ahead of a rounded back. The one asymmetry is the brim,
broken down on the left.

Three families: the cloth of the robe, the straw of the hat (`rope`, which
prints pale against the cloth) and the bone of the hands.
"""

import math

from mathutils import Vector, noise

from lib import cloth, core
from lib.figure import VOID_KEEP, arms, finish, idle_path

# A labourer bent double: the shoulders sit low and are carried well ahead
# of the hips, and the head lower still and further out, sunk between them,
# so the brim nearly rests on the shoulders.
SHOULDER = 1.45
NECK = SHOULDER + 0.03
HEAD = NECK + 0.13
STOOP = 0.29
HEAD_FORWARD = 0.42

# The rounded back: how far it rises behind the shoulder blades, and where.
HUMP = 0.16
HUMP_HEIGHT = SHOULDER - 0.13

# Narrow, sloping shoulders over a hem that spreads wide: weight settling.
ROBE_TOP = SHOULDER + 0.07
ROBE_HEM = 0.05
ROBE_PROFILE = ((0.0, 0.1, 0.1), (0.06, 0.185, 0.13), (0.2, 0.2, 0.15), (0.5, 0.25, 0.19), (1.0, 0.4, 0.33))

# The hat sits low, its brim just over the eyes.
BRIM = HEAD + 0.065
CROWN = 0.145

# The dent: the brim on the figure's left (+X) is broken down in a sharp
# fold, `DENT_SPREAD` radians either side of the crease.
DENT_SPREAD = 0.55

# Where the scythe is held in the idle stance: the harvester leans on it
# like a staff, its butt planted out at his right side and the snath leant
# back past the brim and in over the hat (lib/motion.py `HOLDS`), so the
# crescent arches over the brim's broken side. The snath crosses the brim's
# height 0.64 m from the hat's middle, outside the widest brim.
GRIP = Vector((-0.5, -0.2, 0.94))
HAND_SCALE = 1.55


def build():
    """The field hand of death: a wide hat on a body bent by work."""

    root = core.empty("reaper")
    cloth_mat = core.material("cloth")

    # A heavy smock that spreads to the floor, torn into a few broad tongues.
    hems = [
        dict(folds=(5, 9, 14), fold_depth=0.16, tatter=0.3, tears=7, slits=4, seed=11),
        dict(folds=(6, 11, 17), fold_depth=0.2, tatter=0.42, tears=9, slits=6, seed=12),
        dict(folds=(4, 8, 13), fold_depth=0.13, tatter=0.18, tears=5, slits=2, seed=13),
        dict(folds=(5, 10, 15), fold_depth=0.18, tatter=0.36, tears=6, slits=4, seed=14, lean=0.12),
    ]

    for index, hem in enumerate(hems):
        robe = cloth.robe(
            f"robe_{index}",
            cloth_mat,
            root,
            top=ROBE_TOP,
            hem=ROBE_HEM,
            profile=ROBE_PROFILE,
            hunch=STOOP,
            rings=40,
            segments=96,
            **hem,
        )
        finish(_hunched(robe), seed=index)

    # Leaning on the scythe at his side; the other hand hangs at the hip, a
    # heavy open claw.
    arms(
        root,
        weapon_side_path=idle_path("harvester", "r", (-0.18, -STOOP, SHOULDER - 0.02), (-0.42, -STOOP + 0.1, 1.2), GRIP, HAND_SCALE),
        off_side_path=[(0.18, -STOOP, SHOULDER - 0.02), (0.3, -STOOP + 0.06, 1.16), (0.34, -0.2, 0.88)],
        sleeve_shoulder=0.072,
        sleeve_cuff=0.115,
        droop=0.16,
        off_curl=0.22,
        hand_scale=HAND_SCALE,
        tatter=0.3,
    )
    _claw(root)

    head = core.empty("head", root, location=(0, -STOOP - 0.02, NECK))
    _heads(head)
    _mantles(root)

    return root


def _claw(root):
    """Turns the open off hand palm forward, its fingers spread down and out
    from the hip, so the bone reads on that side as well as on the snath."""

    hand = next(node for node in root.children_recursive if node.name == "arm_l_hand")
    hand.rotation_euler = (math.radians(70), math.radians(-25), math.radians(-60))


def _spine(z):
    """How far forward the middle of the body is at height `z`: the robe's
    own hunch (`cloth.robe`), carried from the hips up to the shoulders."""

    reach = max(0.0, (ROBE_TOP - z) / (ROBE_TOP - ROBE_HEM))

    return STOOP * math.sin(min(reach * 2.2, 1.0) * math.pi * 0.5) - STOOP


def _hump(co):
    """How far the back rises at `co`: most behind the shoulder blades, and
    nothing at the front or below the waist."""

    rise = math.exp(-(((co.z - HUMP_HEIGHT) / 0.16) ** 2))
    behind = max(0.0, min(1.0, (co.y - _spine(co.z)) / 0.14))

    return Vector((0.0, HUMP * rise * behind**1.5, 0.015 * rise * behind))


def _hunched(obj):
    """Rounds the back of a robe or mantle hung with `hunch=STOOP`, in its
    rest shape and every shape key alike, so the game's sway and billow keep
    the hump."""

    moves = [_hump(vertex.co) for vertex in obj.data.vertices]

    for vertex, move in zip(obj.data.vertices, moves):
        vertex.co += move

    if obj.data.shape_keys:
        for block in obj.data.shape_keys.key_blocks:
            for point, move in zip(block.data, moves):
                point.co += move

    obj.data.update()

    return obj


def _hat(name, profile, mat, tip, droop, dent, pinch=0.0, crease=0.0, seed=0.0):
    """A wide hat revolved from `profile` (radius, height above the brim
    line) round the head. The brim is tipped down over the eyes by `tip`,
    sags at the sides by `droop`, and on the left is broken down by `dent` in
    one sharp fold, the crown crushed in above it. `pinch` squeezes the crown
    fore and aft, and `crease` presses a groove down its top from front to
    back."""

    rim = max(radius for radius, _ in profile)

    def shape(angle, radius, height):
        edge = max(0.0, (radius - CROWN) / (rim - CROWN))
        crown = max(0.0, 1 - radius / CROWN) * max(0.0, height / 0.2)
        # The fold: a V along the crease at +X, deepest at the rim.
        off = abs((angle + math.pi) % math.tau - math.pi)
        fold = max(0.0, 1 - off / DENT_SPREAD)
        wobble = noise.noise(Vector((math.cos(angle) * 1.4, math.sin(angle) * 1.4, seed)))
        front = -math.sin(angle)

        across = radius * math.cos(angle)
        radius *= 1 + 0.035 * edge * wobble - 0.06 * edge * fold - 0.22 * crown * fold - pinch * crown * math.sin(angle) ** 2
        height -= crease * crown * math.exp(-((across / 0.035) ** 2))
        height -= tip * radius * front
        height -= droop * edge**2 * (0.6 + 0.4 * math.cos(2 * angle))
        height -= dent * edge**1.2 * fold**1.3
        height += 0.012 * edge * wobble

        return radius, BRIM + height

    hat = core.revolve(name, profile, mat, segments=96, shape=shape)
    hat.location = (0, -HEAD_FORWARD, 0)

    return core.shade_smooth(hat, angle=50)


def _neck(name, mat):
    """Cloth wound twice round a craned neck, from the collar to under the
    chin: two fat turns, each lower at the front, square to the neck."""

    turns = []

    for turn in range(2):
        centre_y = -STOOP + (-HEAD_FORWARD + 0.05 + STOOP) * turn
        height = ROBE_TOP - 0.02 + 0.05 * turn
        radius = 0.16 - 0.035 * turn
        path = []

        for step in range(33):
            angle = math.tau * step / 32
            path.append((math.cos(angle) * radius * 1.05, centre_y + math.sin(angle) * radius, height + 0.4 * math.sin(angle) * radius))

        turns.append(core.tube(f"{name}_{turn}", path, 0.052 - 0.008 * turn, mat, sides=10, cap=False, flatten=0.7))

    return core.shade_smooth(core.join(turns, name))


def _heads(head):
    """Three straw hats. The brim hides the face in every one, and every one
    is broken down on the left."""

    hat_mat = core.material("rope")
    band_mat = core.material("cloth")
    offset = Vector(head.location)

    def place(obj, group):
        obj.parent = group
        obj.location = Vector(obj.location) - offset

        return obj

    variants = [
        # A flat, wide brim over a low round crown.
        dict(
            profile=[(0.0, 0.13), (0.08, 0.128), (0.12, 0.11), (0.138, 0.06), (CROWN, 0.012), (0.22, 0.004), (0.57, -0.008), (0.592, -0.026), (0.575, -0.046), (0.22, -0.018), (0.1, -0.014)],
            tip=0.17,
            droop=0.05,
            dent=0.17,
            band=True,
        ),
        # A battered felt crown, creased and pinched, the brim sagging at
        # the sides.
        dict(
            profile=[(0.0, 0.215), (0.06, 0.21), (0.115, 0.17), (0.138, 0.09), (CROWN, 0.012), (0.21, 0.004), (0.52, -0.006), (0.542, -0.024), (0.525, -0.042), (0.21, -0.018), (0.1, -0.014)],
            tip=0.15,
            droop=0.13,
            dent=0.15,
            pinch=0.22,
            crease=0.06,
            band=True,
        ),
        # A shallow cone, wider than the shoulders.
        dict(
            profile=[(0.0, 0.215), (0.04, 0.205), (0.3, 0.085), (0.6, -0.05), (0.618, -0.064), (0.602, -0.078), (0.3, 0.055), (0.12, 0.1)],
            tip=0.12,
            droop=0.0,
            dent=0.15,
            band=False,
        ),
    ]

    for index, variant in enumerate(variants):
        group = core.empty(f"head_{index}", head)
        hat = _hat(
            f"hat_{index}",
            variant["profile"],
            hat_mat,
            variant["tip"],
            variant["droop"],
            variant["dent"],
            variant.get("pinch", 0.0),
            variant.get("crease", 0.0),
            seed=index * 3.1,
        )
        place(finish(hat, seed=index), group)

        if variant["band"]:
            band = core.revolve(
                f"band_{index}",
                [(CROWN - 0.004, 0.072), (CROWN + 0.01, 0.062), (CROWN + 0.012, 0.04), (CROWN + 0.008, 0.02), (CROWN - 0.004, 0.014)],
                band_mat,
                segments=48,
                shape=lambda angle, radius, height, tip=variant["tip"]: (radius, BRIM + height + tip * radius * math.sin(angle)),
            )
            band.location = (0, -HEAD_FORWARD, 0)
            place(finish(band, seed=index), group)

        # The darkness under the brim, where the face would be.
        void = core.sculpt(f"void_{index}", [("sphere", (0, -HEAD_FORWARD - 0.012, HEAD), (0.105, 0.11, 0.125))], core.material("void"), voxel=0.012)
        place(core.decimate(void, VOID_KEEP), group)
        place(finish(_neck(f"wrap_{index}", band_mat), seed=index), group)


def _robe_radii(z):
    """The robe's half-width and half-depth at height `z`, blended along
    `ROBE_PROFILE` as `cloth.robe` blends it."""

    reach = max(0.0, min(1.0, (ROBE_TOP - z) / (ROBE_TOP - ROBE_HEM)))

    for (d0, w0, t0), (d1, w1, t1) in zip(ROBE_PROFILE, ROBE_PROFILE[1:]):
        if reach <= d1:
            blend = (reach - d0) / max(d1 - d0, 1e-6)
            blend = blend * blend * (3 - 2 * blend)

            return w0 + (w1 - w0) * blend, t0 + (t1 - t0) * blend

    return ROBE_PROFILE[-1][1:]


def _back(z, x):
    """How far back the robe's back is at height `z`, `x` across: on the
    stoop and over the hump, which is what anything laid down the back
    lies on."""

    width, depth = _robe_radii(z)
    across = min(abs(x) / width, 0.95)
    y = _spine(z) + depth * math.sqrt(1 - across * across)

    return y + _hump(Vector((x, y, z))).y


def _hood_down(name, mat):
    """A hood thrown back: its rim a soft roll round the back of the neck,
    and the bag of it lying on the hump, drawn down to its point."""

    roll = []

    for step in range(25):
        angle = math.radians(-20 + 220 * step / 24)
        roll.append((math.cos(angle) * 0.175, -STOOP + 0.005 + math.sin(angle) * 0.155, ROBE_TOP - 0.005 + 0.025 * math.sin(angle)))

    rim = core.tube(f"{name}_rim", roll, lambda t: 0.03 + 0.018 * math.sin(math.pi * t), mat, sides=10, flatten=0.75)
    top = ROBE_TOP - 0.03
    length = 0.42
    rings = []

    for row in range(13):
        share = row / 12
        z = top - length * share
        half = 0.165 * (1 - share) ** 0.8 + 0.01
        bulge = 0.07 * math.sin(math.pi * min(1.0, 0.15 + share * 0.95))
        # Round the outside, from one edge to the other, then back along
        # the robe on the inside.
        across = [-1 + 2 * column / 16 for column in range(17)]
        ring = [(u * half, _back(z, u * half) + 0.025 + bulge * math.sqrt(max(0.0, 1 - u * u)), z) for u in across]
        ring += [(u * half, _back(z, u * half) + 0.02, z) for u in reversed(across[1:-1])]
        rings.append(ring)

    bag = core.loft(f"{name}_bag", rings, mat, cap_start=True, cap_end=True)

    return core.shade_smooth(core.join([rim, bag], name), angle=50)


def _scarf_end(name, mat, x, width, drop, seed=0.0, rows=16, columns=4):
    """One end of a scarf, laid down the back over the hump from under the
    wound collar, widening a little and frayed square at its foot."""

    top = ROBE_TOP - 0.03
    # Each strand of the fringe ends at its own length.
    fray = [0.0, 0.1, 0.03, 0.13, 0.05]
    rings = []

    for row in range(rows + 1):
        share = row / rows
        line = []

        for column in range(columns + 1):
            across = column / columns - 0.5
            px = x + across * width * (1 + 0.3 * share) + 0.02 * math.sin(share * 4 + seed)
            z = top - drop * share * (1 - fray[column % len(fray)] * share**6)
            line.append((px, _back(z, px) + 0.03 + 0.01 * math.sin(across * math.tau + share * 5), z))

        rings.append(line)

    return core.shade_smooth(core.loft(name, rings, mat, closed=False), angle=50)


def _straw_cape(name, mat, tongues=7, top=ROBE_TOP + 0.01, hem=0.85, rings=14, segments=84):
    """A farmer's straw rain cape: bundles of straw tied at the neck and
    falling past the elbows in `tongues` big points, each bundle a ridge
    that runs down to its point. It spreads as it falls, so the shoulder
    line under the brim slopes out to a wide skirt of straw. Hung on the
    stoop, like the robe."""

    length = top - hem
    # Each tongue's point, a little irregular, so it reads as straw rather
    # than as a turned edge.
    phase = 0.4
    grid = []

    for ring in range(rings + 1):
        depth = ring / rings
        loop = []

        for segment in range(segments):
            angle = math.tau * segment / segments
            bundle = tongues * angle / 2 + phase
            notch = abs(math.sin(bundle)) ** 0.75
            reach = depth * (1 - 0.48 * notch * (0.85 + 0.15 * math.sin(angle * 2 + 1.3)))
            share = reach * reach * (3 - 2 * reach)
            width = 0.12 + (0.46 - 0.12) * min(1.0, share * 1.25)
            thick = 0.12 + (0.34 - 0.12) * min(1.0, share * 1.25)
            ridge = 1 + 0.07 * reach * math.cos(2 * bundle)
            z = top - reach * length
            loop.append((math.cos(angle) * width * ridge, _spine(z) + math.sin(angle) * thick * ridge, z))

        grid.append(loop)

    return core.shade_smooth(core.loft(name, grid, mat), angle=40)


def _mantles(root):
    """What sits on a harvester's shoulders. Each follows the stoop, and none
    of them breaks the symmetry: the brim is the one thing that does."""

    straw = core.material("rope")
    cloth_mat = core.material("cloth")

    # A straw rain cape in seven big tongues, laid over the hump.
    cape = _straw_cape("straw_cape", straw)
    cape.parent = core.empty("mantle_0", root)
    finish(_hunched(cape), seed=3)

    # A hood thrown back, which makes the stoop heavier still.
    hood = _hood_down("hood_down", cloth_mat)
    hood.parent = core.empty("mantle_1", root)
    finish(hood, seed=4)

    # A long scarf wound twice round the neck, its two ends hanging down
    # the back side by side.
    group = core.empty("mantle_2", root)
    coils = []

    for coil in range(3):
        height = ROBE_TOP - 0.03 + coil * 0.055
        radius = 0.2 - coil * 0.025
        centre_y = -STOOP - coil * 0.03
        path = []

        for step in range(41):
            angle = math.tau * step / 40
            path.append((math.cos(angle) * radius * 1.1, centre_y + math.sin(angle) * radius * 0.85, height + 0.03 * math.sin(angle) + 0.012 * math.sin(angle * 2 + coil)))

        coils.append(core.tube(f"coil_{coil}", path, 0.052 - coil * 0.005, cloth_mat, sides=10, cap=False, flatten=0.55))

    collar = core.join(coils, "scarf")
    collar.parent = group
    finish(core.shade_smooth(collar), seed=5)

    for tail, side in enumerate((-1, 1)):
        end = _scarf_end(f"scarf_tail_{tail}", cloth_mat, side * 0.075, 0.13, 0.6 - 0.05 * tail, seed=tail * 1.7)
        end.parent = group
        finish(end, seed=tail)
