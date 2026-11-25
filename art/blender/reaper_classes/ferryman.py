"""The ferryman: the one who carries the dead across and cannot be moved.

Silhouette: a hunched, deep cowl sunk into a broad, low body, the off
shoulder loaded high with a heavy coil of rope, and beside it a pole taller
than the figure, planted at its foot and leant out from the body, its
lantern hung out in space from the crook. The outline reads as a lopsided
boulder beside a long slanting line. Three families on show: the cloth,
the rope (or the pale drape) and the bone of the hands.
"""

import math

from mathutils import Vector, noise

from lib import cloth, core
from lib.figure import NECK_HEIGHT, SHOULDER_HEIGHT, arms, finish, idle_path

HUNCH = 0.24
ROBE_TOP = SHOULDER_HEIGHT + 0.08
ROBE_HEM = 0.07

# How far the head is sunk into the shoulders below the shared neck: the
# cowl and the eye line inside it (lib/face.py) go down together.
SINK = 0.07
NECK = NECK_HEIGHT - SINK

# Broad at the shoulder, deep through the chest and no narrower below, so the
# figure stands like a bollard rather than hanging like a coat.
ROBE_PROFILE = ((0.0, 0.15, 0.14), (0.06, 0.33, 0.25), (0.16, 0.38, 0.29), (0.5, 0.36, 0.29), (1.0, 0.46, 0.38))

# How finely a cowl is cut around its opening, how thick its cloth is as a
# share of its size, and how far its cape stands off the robe.
COLUMNS = 56
THICK = 0.14
CAPE_PAD = 0.03

# How far anything thrown over the off shoulder must stand off the robe to
# clear the sleeve under it, as (height, distance) stops.
SLEEVE_PAD = ((1.67, 0.0), (1.6, 0.045), (1.5, 0.065), (1.4, 0.105), (1.3, 0.16), (1.2, 0.2))

# Where the pole is held in the idle stance: its shoe planted on the floor
# just before the robe at the pole side (the lantern pole's runs 1.03 below
# the grip), the pole leant out from the body and the crook turned out, so
# the lantern hangs in space beyond the hand (lib/motion.py `HOLDS`).
GRIP = Vector((-0.5, -0.44, 1.07))

# The hem hitched up at the front of the pole side, so the robe does not
# merge with the floor there (docs/art-bible.md 6.2): how far, round which
# way, and how far up the robe the lift runs out.
HITCH = 0.45
HITCH_ANGLE = math.atan2(-0.6, -1.0)
HITCH_REACH = 0.8


def build():
    """The one who carries the dead across: a heavy cowl on a broad body."""

    root = core.empty("reaper")
    cloth_mat = core.material("cloth")
    forward = HUNCH

    hems = [
        # Heavy and barely torn.
        dict(hem=ROBE_HEM + 0.05, tatter=0.14, tears=4, slits=3, fold_depth=0.2, seed=31),
        # Waterlogged: it clings, and drags lower.
        dict(hem=ROBE_HEM - 0.03, tatter=0.24, tears=6, slits=6, fold_depth=0.12, seed=32),
        # Torn into wide strips.
        dict(hem=ROBE_HEM, tatter=0.46, tears=4, slits=9, fold_depth=0.17, seed=33),
        # Dragged back by the current.
        dict(hem=ROBE_HEM, tatter=0.3, tears=6, slits=7, fold_depth=0.18, seed=34, lean=0.2),
    ]

    for index, hem in enumerate(hems):
        robe = cloth.robe(
            f"robe_{index}",
            cloth_mat,
            root,
            top=ROBE_TOP,
            profile=ROBE_PROFILE,
            folds=(4, 7, 12),
            hunch=forward,
            rings=40,
            segments=96,
            **hem,
        )
        finish(_hitched(robe), seed=index)

    # The pole hand closed on the planted pole; the other hangs under the
    # load on its shoulder.
    arms(
        root,
        weapon_side_path=idle_path("ferryman", "r", (-0.29, -forward, SHOULDER_HEIGHT - 0.1), (-0.46, -forward + 0.06, 1.38), GRIP, 1.55),
        off_side_path=[(0.29, -forward, SHOULDER_HEIGHT - 0.1), (0.44, -forward + 0.03, 1.25), (0.45, -forward - 0.04, 0.97)],
        sleeve_shoulder=0.1,
        sleeve_cuff=0.14,
        droop=0.22,
        hand_scale=1.55,
    )

    head = core.empty("head", root, location=(0, -forward, NECK))
    _heads(head, cloth_mat, forward)
    _mantles(root)

    return root


def _hitched(robe):
    """Lifts the hem at the front of the pole side (`HITCH`), in the rest
    shape and every shape key alike, so the game's sway keeps it."""

    vertices = robe.data.vertices
    hem = min(vertex.co.z for vertex in vertices)
    moves = []

    for vertex in vertices:
        _, _, middle = _body(vertex.co.z)
        angle = math.atan2(vertex.co.y - middle, vertex.co.x)
        turn = (angle - HITCH_ANGLE + math.pi) % math.tau - math.pi
        reach = max(0.0, 1 - (vertex.co.z - hem) / HITCH_REACH)
        moves.append(HITCH * math.exp(-((turn / 0.9) ** 2)) * reach**1.3)

    for vertex, lift in zip(vertices, moves):
        vertex.co.z += lift

    if robe.data.shape_keys:
        for block in robe.data.shape_keys.key_blocks:
            for point, lift in zip(block.data, moves):
                point.co.z += lift

    robe.data.update()

    return robe


def _ease(value):
    """A smooth step from 0 to 1."""

    value = max(0.0, min(1.0, value))

    return value * value * (3 - 2 * value)


def _along(curve, share):
    """The point `share` of the way along a sampled curve."""

    position = max(0.0, min(1.0, share)) * (len(curve) - 1)
    index = min(int(position), len(curve) - 2)

    return curve[index].lerp(curve[index + 1], position - index)


def _body(height):
    """Where the robe is at `height`: its half-width, its half-depth and how
    far its middle sits from the root. Everything worn is laid on this."""

    depth = max(0.0, min(1.0, (ROBE_TOP - height) / (ROBE_TOP - ROBE_HEM)))
    width, thickness = ROBE_PROFILE[-1][1:]

    for (d0, w0, t0), (d1, w1, t1) in zip(ROBE_PROFILE, ROBE_PROFILE[1:]):
        if depth <= d1:
            blend = _ease((depth - d0) / (d1 - d0))
            width, thickness = w0 + (w1 - w0) * blend, t0 + (t1 - t0) * blend
            break

    return width, thickness, HUNCH * (math.sin(min(depth * 2.2, 1.0) * math.pi * 0.5) - 1)


def _sleeve_lift(angle, height):
    """How much further out than the robe the off-side sleeve reaches at
    `angle` around the body. The pole-side sleeve is its mirror image."""

    lift = SLEEVE_PAD[-1][1]

    for (h0, p0), (h1, p1) in zip(SLEEVE_PAD, SLEEVE_PAD[1:]):
        if height >= h1:
            lift = p0 + (p1 - p0) * max(0.0, (h0 - height) / (h0 - h1))
            break

    return lift * math.exp(-((((angle + 0.25 + math.pi) % math.tau - math.pi) / 0.55) ** 2))


def _clear(point, pad):
    """Pushes a point outward until it stands `pad` off the robe and both
    sleeves, which is what makes a cape rest on the shoulders instead of
    passing through them."""

    width, depth, middle = _body(point.z)
    angle = math.atan2(point.y - middle, point.x)
    pad += _sleeve_lift(angle, point.z) + _sleeve_lift(math.pi - angle, point.z)
    inside = math.hypot(point.x / (width + pad), (point.y - middle) / (depth + pad))

    if inside >= 1 or inside < 1e-6:
        return point

    # The push fades out toward the neck, where the robe ends, so that a
    # cowl meets its cape in a slope and not a step.
    clear = Vector((point.x / inside, middle + (point.y - middle) / inside, point.z))

    return point.lerp(clear, _ease((ROBE_TOP - point.z) / 0.09))


def _around(angle, height, pad):
    """The point at `angle` around the robe at `height`, `pad` off it."""

    width, depth, middle = _body(height)

    return Vector((math.cos(angle) * (width + pad), middle + math.sin(angle) * (depth + pad), height))


def _over_shoulder(angle, height, pad):
    """A point on whatever is thrown over the off shoulder: it lies on the
    robe, and lifts over the sleeve where the arm is."""

    return _around(angle, height, pad + _sleeve_lift(angle, height))


def _cowl(index, mat, forward, look):
    """A heavy hood and its cape as one thick cloth, swept from the rim of
    the opening over the head and down onto the chest and back.

    Each ring of it spans from the `bib` line under the jaw to the `crest`
    line over the crown, so the first ring is the opening and the last is
    the hem. Returns the cowl and the darkness that fills it.
    """

    crest = core.bezier([(wide, reach - forward, height) for wide, reach, height in look["crest"]], samples=6)
    bib = core.bezier([(0, reach - forward, height) for reach, height in look["bib"]], samples=6)
    arounds = [math.tau * column / COLUMNS for column in range(COLUMNS)]
    phase = index * 2.3

    def ring(share, around, scale=1.0):
        far = _along(crest, share)
        near = _along(bib, share)
        wide = far.x
        far = Vector((0, far.y, far.z))
        middle = (far + near) / 2
        lift = math.cos(around)

        # Narrower toward the crest, so the top of the hood is a ridge.
        taper = 1 - look["arch"] * max(0.0, lift) ** 2

        return middle + scale * ((far - middle) * lift + Vector((wide * math.sin(around) * taper, 0, 0)))

    def hem(around):
        """How far down the cape reaches: least over the shoulders, so the
        arms under it stay free."""

        nick = abs(noise.noise(Vector((math.cos(around) * 2.2, math.sin(around) * 2.2, phase))))

        return 1 - look["cut"] * math.sin(around) ** 2 - look["tatter"] * nick

    def outside(step, around):
        point = ring(step * hem(around), around)

        # Heavy wet-wool folds along the hood crown and deeper pleats down the cape.
        hood_fold = (0.5 + 0.5 * math.sin(around * 6 + phase * 0.8)) * math.sin(min(step * 2.2, 1.0) * math.pi)
        pleat = (0.5 + 0.5 * math.sin(around * 7 + phase)) ** 2 * (0.6 + 0.4 * math.sin(around * 3 + phase * 1.7))
        outward = Vector((math.sin(around), 0.35 * math.cos(around), 0.25 * math.cos(around)))
        point = point + outward * (0.011 * hood_fold)

        return _clear(point, CAPE_PAD + look["fold"] * 1.25 * _ease((1.67 - point.z) / 0.2) * pleat)

    out = Vector((1, 0, 0)).cross(_along(crest, 0) - _along(bib, 0))
    out.x = 0
    out.normalize()

    rings = [[ring(step / 8 * 0.5, around, 1 - THICK) for around in arounds] for step in range(8, -1, -1)]

    # The rim is rolled, as a heavy hem is.
    for step in (3, 2, 1):
        roll = math.pi * step / 4
        scale = 1 - THICK * (1 - math.cos(roll)) / 2

        rings.append([ring(0, around, scale) + out * (0.022 * math.sin(roll)) for around in arounds])

    rings += [[outside(step / 34, around) for around in arounds] for step in range(35)]

    # Closed under the cape as well as behind the lining, so the cloth has
    # one unambiguous outside.
    hood = core.loft(f"cowl_{index}", rings, mat, cap_start=True, cap_end=True)
    core.roughen(hood, 0.008, 4.5, seed=index + 1.0)

    # The void closes the opening just inside the rim. It is a lens, dished
    # toward the viewer and closed behind, so that it too has one outside.
    seat = [ring(0.02, around, 1 - THICK) for around in arounds]
    middle = sum(seat, Vector()) / COLUMNS
    lens = [(0.06, 0.045), (0.3, 0.041), (0.55, 0.031), (0.8, 0.016), (1.06, -0.005), (0.8, 0.065), (0.45, 0.085), (0.06, 0.095)]
    void = core.loft(
        f"void_{index}",
        [[middle + (point - middle) * size - out * recess for point in seat] for size, recess in lens],
        core.material("void"),
        cap_start=True,
        cap_end=True,
    )

    return core.shade_smooth(hood), void


def _heads(head, mat, forward):
    """Three cowls. Each is filled with darkness, and none has a face."""

    offset = Vector(head.location)

    def place(obj, group):
        obj.parent = group
        obj.location = Vector(obj.location) - offset

        return obj

    # Each crest stop is (half-width, how far behind the neck, height), from
    # the brow over the crown and down the back to the hem.
    looks = [
        # A deep, blunt cowl, its peak thrust forward and down over the
        # opening, its cape spread wide over the shoulders: a boulder.
        dict(
            crest=[
                (0.12, -0.53, 1.81),
                (0.19, -0.33, 2.0),
                (0.22, -0.1, 2.03),
                (0.235, 0.1, 1.96),
                (0.26, 0.21, 1.83),
                (0.31, 0.27, 1.7),
                (0.37, 0.34, 1.58),
                (0.41, 0.42, 1.45),
                (0.43, 0.49, 1.31),
            ],
            bib=[(-0.24, 1.61), (-0.26, 1.5), (-0.2, 1.3)],
            arch=0.26,
            cut=0.3,
            tatter=0.08,
            fold=0.035,
        ),
        # A narrower cowl, as deep, whose long peak has fallen down its back.
        dict(
            crest=[
                (0.105, -0.51, 1.82),
                (0.17, -0.33, 2.0),
                (0.19, -0.1, 2.07),
                (0.205, 0.12, 2.11),
                (0.23, 0.235, 1.89),
                (0.29, 0.27, 1.7),
                (0.35, 0.33, 1.58),
                (0.39, 0.4, 1.48),
                (0.41, 0.45, 1.39),
            ],
            bib=[(-0.24, 1.61), (-0.26, 1.52), (-0.215, 1.4)],
            arch=0.5,
            cut=0.27,
            tatter=0.12,
            fold=0.04,
        ),
        # A broad cowl whose brow juts over the opening like a storm flap,
        # high enough that the eyes still show under it.
        dict(
            crest=[
                (0.135, -0.52, 1.83),
                (0.2, -0.32, 1.97),
                (0.225, -0.1, 2.0),
                (0.235, 0.12, 1.94),
                (0.26, 0.23, 1.81),
                (0.31, 0.28, 1.69),
                (0.37, 0.35, 1.57),
                (0.42, 0.43, 1.44),
                (0.45, 0.49, 1.32),
            ],
            bib=[(-0.24, 1.6), (-0.26, 1.5), (-0.21, 1.33)],
            arch=0.2,
            cut=0.27,
            tatter=0.05,
            fold=0.03,
        ),
    ]

    for index, look in enumerate(looks):
        group = core.empty(f"head_{index}", head)
        hood, void = _cowl(index, mat, forward, look)

        if index == 1:
            hood = core.join([hood, _fallen_peak(mat, forward)], hood.name)

        place(finish(hood, seed=index), group)
        place(void, group)


def _fallen_peak(mat, forward):
    """The long peak of a hood, fallen off the crown and lying down the back
    of its cape, on the pole side so that it stays clear of the mantle."""

    path = core.bezier(
        [
            (0.0, 0.1 - forward, 1.98),
            (0.0, 0.205 - forward, 2.02),
            (-0.01, 0.26 - forward, 1.96),
            (-0.035, 0.285 - forward, 1.83),
            (-0.075, 0.3 - forward, 1.71),
            (-0.12, 0.345 - forward, 1.59),
            (-0.15, 0.41 - forward, 1.48),
            (-0.165, 0.46 - forward, 1.39),
        ],
        samples=6,
    )

    def girth(share):
        """It tapers to nothing, past a knot tied near its end."""

        return 0.004 + 0.05 * (1 - share) ** 1.2 + 0.013 * math.exp(-(((share - 0.88) / 0.035) ** 2))

    return core.tube("fallen_peak", path, girth, mat, sides=10)


def _mantles(root):
    """What a ferryman carries on one shoulder: always the off side, so the
    pole arm stays clear, and always standing out past the arm, so the
    shoulder line goes lopsided."""

    _rope_coil(core.empty("mantle_0", root))
    _half_cape(core.empty("mantle_1", root), core.material("cloth_alt"))
    _net(core.empty("mantle_2", root))


# How much higher, and further out, the coil sits on the off shoulder than a
# line laid round the body would, and how far its turns cross the shoulder on
# the slant: in on the chest, by the cowl, and out at the back, down the arm,
# so from the front the coil arches over the shoulder in one round hump
# instead of ending in a stub.
COIL_RISE = 0.06
COIL_OUT = 0.09
COIL_SLANT = 0.3


def _slant(angle):
    """How far across the body the coil is set at `angle` around it."""

    return COIL_SLANT * math.sin(angle) * max(0.0, math.cos(angle))


def _rope_coil(group):
    """A mooring line coiled over the off shoulder and across the body, made
    fast on the chest with its end left hanging."""

    rope = core.material("rope")
    thickness = 0.03
    pitch = thickness * 1.9
    steps = 56
    fast = math.radians(-51)

    def centre(step):
        angle = fast + math.tau * step / steps
        # Over the shoulder the coil is heaped up and out onto the arm, so the
        # shoulder line goes lopsided, high on the coil's side.
        shoulder = max(0.0, math.cos(angle)) ** 2
        height = 1.1 + 0.56 * (0.5 + 0.5 * math.cos(angle)) ** 1.5 + COIL_RISE * shoulder
        width, depth, middle = _body(height)

        # Up on the shoulder the line lies over the sleeve and the cowl's
        # cape, and does not follow the robe in toward the neck.
        pad = 0.02 + 0.035 * _ease((height - 1.34) / 0.12) + _sleeve_lift(angle, height) + COIL_OUT * shoulder

        return Vector((math.cos(angle) * (max(width, 0.31) + pad) + _slant(angle), middle + math.sin(angle) * (max(depth, 0.22) + pad), height))

    def frame(step):
        here = centre(step)
        tangent = (centre(step + 1) - centre(step - 1)).normalized()
        out = Vector((here.x, here.y - _body(here.z)[2], 0)).normalized()
        # Over the shoulder the turns heap up on it, rather than standing out
        # sideways off it.
        out = out.lerp(Vector((0, 0, 1)), 0.75 * _ease((here.z - 1.5) / 0.16)).normalized()
        across = tangent.cross(out).normalized()

        return here, tangent, across, across.cross(tangent)

    # Six fat turns of the line, heaped three, two and one: one mass at a
    # phone's size, not a hatch of strands.
    lay = [(-1.0, 0.0), (0.0, 0.0), (1.0, 0.0), (-0.5, 0.87), (0.5, 0.87), (0.0, 1.73)]
    parts = []

    for turn, (shift, lift) in enumerate(lay):
        path = []

        for step in range(steps + 1):
            here, _, across, out = frame(step)
            angle = math.tau * step / steps
            # On the shoulder the turns lie close in one rounded heap; away
            # from it they work loose a little.
            loose = 1 - max(0.0, math.cos(fast + angle)) ** 2
            wander = 0.08 * math.sin(angle * 3 + turn * 1.9) * loose
            rise = 0.1 * abs(math.sin(angle * 2 + turn * 2.6)) * loose

            path.append(here + across * ((shift + wander) * pitch) + out * ((lift + rise) * pitch + thickness))

        parts.append(core.tube(f"coil_{turn}", path, thickness, rope, sides=8, cap=False))

    # The hitch that holds the turns together, where they begin and end.
    here, tangent, across, out = frame(0)
    hitch = []

    for step in range(61):
        angle = math.tau * step / 12
        hitch.append(
            here
            + tangent * ((step / 60 - 0.5) * 0.15)
            + across * (math.cos(angle) * pitch * 2.45)
            + out * (pitch * 1.4 + math.sin(angle) * pitch * 1.8)
        )

    parts.append(core.tube("coil_hitch", hitch, thickness * 0.8, rope, sides=6))

    # The free end falls from the hitch and ends in a stopper knot.
    fall = [here + out * (pitch * 3.0) - tangent * 0.06]
    fall += [_around(fast - 0.06, height, 0.035) + Vector((_slant(fast), 0, 0)) for height in (1.36, 1.24, 1.1)]
    parts.append(core.tube("coil_end", core.bezier(fall, samples=6), thickness * 0.8, rope, sides=6))

    knot = core.revolve(
        "coil_knot",
        [(0.0, 0.045), (0.027, 0.038), (0.04, 0.011), (0.04, -0.011), (0.027, -0.038), (0.0, -0.045)],
        rope,
        segments=10,
    )
    knot.location = fall[-1]
    parts.append(knot)

    coil = core.join(parts, "rope_coil")
    coil.parent = group
    finish(core.shade_smooth(coil), wear_boost=1.4, seed=2)


def _half_cape(group, mat):
    """A drape of old sailcloth thrown over the off shoulder: it lies on the
    cowl's cape at the neck, stands out past the arm in one stiff fold and
    falls to the thigh there, so the shoulder line runs on out over the arm.
    Cut on the slant: short over the chest, long over the arm and down the
    back. It prints a step paler than the robe (`cloth_alt`)."""

    start, end = math.radians(-72), math.radians(118)
    top = 1.745
    columns, rows = 40, 22
    # Where along the drape the off shoulder is, and so the arm under it.
    arm = -start / (end - start)

    def hem(along):
        """Short over the chest, longest over the arm, long down the back."""

        crack = 0.03 * abs(noise.noise(Vector((along * 9.0, 3.0, 1.0))))
        over_arm = math.exp(-(((along - arm) / 0.22) ** 2))

        return 1.36 - 0.2 * _ease(along / 0.5) - 0.3 * over_arm + crack

    def crease(along):
        """Tarred cloth does not ripple: it breaks into flat planes."""

        return abs((along * 4.5 + 0.3) % 1.0 - 0.5) * 2

    grid = []

    for row in range(rows + 1):
        line = []

        for column in range(columns + 1):
            along = column / columns
            height = top - (top - hem(along)) * row / rows
            hang = _ease((1.42 - height) / 0.3)
            angle = start + (end - start) * along
            # Over the arm it stands out in a stiff fold, wider as it falls.
            flare = 0.16 * math.exp(-((angle / 0.62) ** 2)) * _ease((top - height) / 0.4)

            # Lower down it stands off the robe, whose folds move under it.
            line.append(_over_shoulder(angle, height, 0.009 + flare + hang * (0.035 + 0.03 * crease(along))))

        grid.append(line)

    cape = core.loft("half_cape", grid, mat, closed=False)

    # A bound edge, which is what makes it read as cut cloth and not a shell.
    border = [line[0] for line in grid] + grid[-1][1:] + [line[-1] for line in reversed(grid[:-1])]
    welt = core.tube("half_cape_welt", border, 0.011, mat, sides=6)

    cape = core.join([cape, welt], "half_cape")
    cape.parent = group
    finish(core.shade_smooth(cape, angle=40), seed=7)


def _net(group):
    """A net thrown over the off shoulder: a coarse mesh of heavy cord, six
    cells round and four down, standing out over the arm, with its floats
    hung below its foot on short drops, where they break the outline at the
    side and the back."""

    rope = core.material("rope")
    start, end = math.radians(-62), math.radians(140)
    top = 1.73
    cells = 6
    rows = 4
    strands = []
    floats = []

    def knot(column, row):
        """A knot of the net, counted across and down. The foot sags between
        the floats and is drawn up at each."""

        along = column / cells
        hem = 1.24 - 0.22 * math.sin(along * math.pi) ** 0.7 - 0.06 * abs(math.sin(along * cells * math.pi / 2))
        height = top - (top - hem) * row / rows
        angle = start + (end - start) * along
        flare = 0.1 * math.exp(-((angle / 0.7) ** 2)) * _ease((top - height) / 0.4)

        return _over_shoulder(angle, height, 0.015 + flare + 0.05 * _ease((1.42 - height) / 0.25))

    def strand(points, radius, name):
        path = core.bezier(points, samples=4)

        return core.tube(name, path, radius, rope, sides=6, cap=False)

    # Two sets of cords cross on the slant to make the mesh.
    for lean in (-1, 1):
        for start_column in range(-rows, cells + 1):
            path = []

            for row in range(rows + 1):
                column = start_column + row if lean > 0 else start_column + rows - row

                if 0 <= column <= cells:
                    path.append(knot(column, row))

            if len(path) > 1:
                strands.append(strand(path, 0.016, f"net_{lean}_{start_column}"))

    for line, (row, radius) in enumerate(((0, 0.019), (rows, 0.022))):
        strands.append(strand([knot(column / 2, row) for column in range(cells * 2 + 1)], radius, f"net_line_{line}"))

    mesh = core.join(strands, "net")
    mesh.parent = group
    finish(core.shade_smooth(mesh), wear_boost=1.5)

    # A float on a short drop under every other knot of the foot, the drops
    # longest at the side, so the floats hang out past the arm and the hem
    # of the net.
    for column in range(0, cells + 1, 2):
        foot = knot(column, rows)
        drop = 0.1 + 0.1 * math.sin(column / cells * math.pi)
        cork = core.revolve(
            f"float_{column}",
            [(0.0, 0.065), (0.036, 0.057), (0.06, 0.018), (0.06, -0.018), (0.036, -0.057), (0.0, -0.065)],
            rope,
            segments=10,
        )
        cork.location = foot + Vector((0, 0, -drop - 0.06))
        floats.append(cork)
        floats.append(core.tube(f"float_line_{column}", [foot, foot + Vector((0, 0, -drop))], 0.008, rope, sides=5))

    corks = core.join(floats, "net_floats")
    corks.parent = group
    finish(core.shade_smooth(corks), wear_boost=1.4)
