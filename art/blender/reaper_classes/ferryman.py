"""The ferryman: the one who carries the dead across and cannot be moved.

Silhouette: a hunched, heavy cowl on a broad, low body, one shoulder loaded
with rope, and a pole taller than the figure with a lantern at its top. The
outline reads as a boulder beside a tall vertical line.
"""

import math

from mathutils import Vector, noise

from lib import cloth, core
from lib.figure import NECK_HEIGHT, SHOULDER_HEIGHT, arms, finish

HUNCH = 0.24
ROBE_TOP = SHOULDER_HEIGHT + 0.08
ROBE_HEM = 0.07

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


def build():
    """The one who carries the dead across: a heavy cowl on a broad body."""

    root = core.empty("reaper")
    cloth_mat = core.material("cloth")
    alt = core.material("cloth_alt")
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
        finish(
            cloth.robe(
                f"robe_{index}",
                cloth_mat,
                root,
                top=ROBE_TOP,
                profile=ROBE_PROFILE,
                folds=(4, 7, 12),
                hunch=forward,
                **hem,
            ),
            seed=index,
        )

    arms(
        root,
        weapon_side_path=[(-0.29, -forward, SHOULDER_HEIGHT - 0.1), (-0.47, -forward - 0.02, 1.33), (-0.45, -forward - 0.24, 1.25)],
        off_side_path=[(0.29, -forward, SHOULDER_HEIGHT - 0.1), (0.44, -forward + 0.03, 1.25), (0.45, -forward - 0.04, 0.97)],
        sleeve_shoulder=0.1,
        sleeve_cuff=0.14,
        droop=0.22,
        hand_scale=1.55,
    )

    head = core.empty("head", root, location=(0, -forward, NECK_HEIGHT))
    _heads(head, alt, forward)
    _mantles(root, alt)

    return root


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
    the hem. Returns the cowl, the darkness that fills it and its rim.
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

        # The cape hangs in pleats: ridges of uneven size that deepen toward
        # its hem.
        pleat = (0.5 + 0.5 * math.sin(around * 7 + phase)) ** 2 * (0.6 + 0.4 * math.sin(around * 3 + phase * 1.7))

        return _clear(point, CAPE_PAD + look["fold"] * _ease((1.67 - point.z) / 0.2) * pleat)

    out = Vector((1, 0, 0)).cross(_along(crest, 0) - _along(bib, 0))
    out.x = 0
    out.normalize()

    rings = [[ring(step / 8 * 0.5, around, 1 - THICK) for around in arounds] for step in range(8, -1, -1)]

    # The rim is rolled, as a heavy hem is.
    for step in (3, 2, 1):
        roll = math.pi * step / 4
        scale = 1 - THICK * (1 - math.cos(roll)) / 2

        rings.append([ring(0, around, scale) + out * (0.02 * math.sin(roll)) for around in arounds])

    rings += [[outside(step / 34, around) for around in arounds] for step in range(35)]

    # Closed under the cape as well as behind the lining, so the cloth has
    # one unambiguous outside.
    hood = core.loft(f"cowl_{index}", rings, mat, cap_start=True, cap_end=True)
    core.roughen(hood, 0.01, 4.0, seed=index + 1.0)

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

    return core.shade_smooth(hood), void, [ring(0, around) for around in arounds]


def _heads(head, alt, forward):
    """Three cowls. Each is filled with darkness, and none has a face."""

    offset = Vector(head.location)

    def place(obj, group):
        obj.parent = group
        obj.location = Vector(obj.location) - offset

        return obj

    # Each crest stop is (half-width, how far behind the neck, height), from
    # the brow over the crown and down the back to the hem.
    looks = [
        # A deep, blunt cowl, heavy on the shoulders.
        dict(
            crest=[
                (0.105, -0.36, 1.94),
                (0.18, -0.22, 2.07),
                (0.21, -0.04, 2.09),
                (0.22, 0.13, 2.01),
                (0.245, 0.22, 1.88),
                (0.29, 0.27, 1.76),
                (0.34, 0.34, 1.64),
                (0.37, 0.42, 1.51),
                (0.38, 0.49, 1.37),
            ],
            bib=[(-0.22, 1.68), (-0.245, 1.57), (-0.19, 1.37)],
            arch=0.26,
            cut=0.3,
            tatter=0.08,
            fold=0.035,
        ),
        # A narrower cowl whose long peak has fallen down its back.
        dict(
            crest=[
                (0.1, -0.34, 1.95),
                (0.17, -0.22, 2.08),
                (0.19, -0.05, 2.15),
                (0.205, 0.15, 2.19),
                (0.23, 0.245, 1.96),
                (0.28, 0.27, 1.77),
                (0.33, 0.33, 1.65),
                (0.355, 0.4, 1.55),
                (0.36, 0.45, 1.46),
            ],
            bib=[(-0.22, 1.68), (-0.245, 1.59), (-0.205, 1.47)],
            arch=0.5,
            cut=0.27,
            tatter=0.12,
            fold=0.04,
        ),
        # A broad cowl whose brow hangs over the opening like a storm flap,
        # with a string of the fares it took.
        dict(
            crest=[
                (0.135, -0.46, 1.86),
                (0.195, -0.3, 2.02),
                (0.22, -0.08, 2.06),
                (0.23, 0.13, 2.0),
                (0.25, 0.23, 1.87),
                (0.29, 0.28, 1.75),
                (0.34, 0.35, 1.63),
                (0.37, 0.43, 1.5),
                (0.39, 0.49, 1.38),
            ],
            bib=[(-0.225, 1.69), (-0.245, 1.58), (-0.2, 1.4)],
            arch=0.2,
            cut=0.27,
            tatter=0.05,
            fold=0.03,
        ),
    ]

    for index, look in enumerate(looks):
        group = core.empty(f"head_{index}", head)
        hood, void, rim = _cowl(index, alt, forward, look)

        if index == 1:
            hood = core.join([hood, _fallen_peak(alt, forward)], hood.name)

        place(finish(hood, seed=index), group)
        place(void, group)

        if index == 2:
            place(_fares(rim[COLUMNS - 9]), group)


def _fallen_peak(alt, forward):
    """The long peak of a hood, fallen off the crown and lying down the back
    of its cape, on the pole side so that it stays clear of the mantle."""

    path = core.bezier(
        [
            (0.0, 0.12 - forward, 2.05),
            (0.0, 0.215 - forward, 2.09),
            (-0.01, 0.265 - forward, 2.03),
            (-0.035, 0.285 - forward, 1.9),
            (-0.075, 0.29 - forward, 1.78),
            (-0.115, 0.335 - forward, 1.66),
            (-0.145, 0.4 - forward, 1.55),
            (-0.16, 0.45 - forward, 1.46),
        ],
        samples=6,
    )

    def girth(share):
        """It tapers to nothing, past a knot tied near its end."""

        return 0.004 + 0.05 * (1 - share) ** 1.2 + 0.013 * math.exp(-(((share - 0.88) / 0.035) ** 2))

    return core.tube("fallen_peak", path, girth, alt, sides=10)


def _fares(top):
    """A string of coins hung from one corner of a brow: the fares taken."""

    cord = core.tube("fare_cord", [top, top + Vector((0, 0, -0.2))], 0.003, core.material("rope"), sides=4)
    coins = []

    for fare, (drop, turn) in enumerate(((0.085, 0.3), (0.14, -0.5), (0.2, 0.1))):
        coin = core.revolve(
            f"fare_{fare}",
            [(0.0, 0.004), (0.021, 0.004), (0.026, 0.0), (0.021, -0.004), (0.0, -0.004)],
            core.material("gold"),
            segments=14,
        )
        coin.location = top + Vector((0, 0, -drop))
        coin.rotation_euler = (math.pi / 2, 0, turn)
        coins.append(coin)

    return finish(core.join([cord] + coins, "fares"), wear_boost=1.6)


def _mantles(root, alt):
    """What a ferryman carries on one shoulder: always the off side, so the
    pole arm stays clear."""

    _rope_coil(core.empty("mantle_0", root))
    _half_cape(core.empty("mantle_1", root), alt)
    _net(core.empty("mantle_2", root))


def _rope_coil(group):
    """A mooring line coiled over the off shoulder and across the body, made
    fast on the chest with its end left hanging."""

    rope = core.material("rope")
    thickness = 0.02
    pitch = thickness * 1.9
    steps = 56
    fast = math.radians(-51)

    def centre(step):
        angle = fast + math.tau * step / steps
        height = 1.1 + 0.56 * (0.5 + 0.5 * math.cos(angle)) ** 1.5
        width, depth, middle = _body(height)

        # Up on the shoulder the line lies over the sleeve and the cowl's
        # cape, and does not follow the robe in toward the neck.
        pad = 0.02 + 0.035 * _ease((height - 1.34) / 0.12) + _sleeve_lift(angle, height)

        return Vector((math.cos(angle) * (max(width, 0.31) + pad), middle + math.sin(angle) * (max(depth, 0.22) + pad), height))

    def frame(step):
        here = centre(step)
        tangent = (centre(step + 1) - centre(step - 1)).normalized()
        out = Vector((here.x, here.y - _body(here.z)[2], 0)).normalized()
        out = out.lerp(Vector((0, 0, 1)), _ease((here.z - 1.5) / 0.16)).normalized()
        across = tangent.cross(out).normalized()

        return here, tangent, across, across.cross(tangent)

    # Nine turns of the line, heaped four, three and two.
    lay = [(-1.5, 0.0), (-0.5, 0.0), (0.5, 0.0), (1.5, 0.0), (-1.0, 0.87), (0.0, 0.87), (1.0, 0.87), (-0.5, 1.73), (0.5, 1.73)]
    parts = []

    for turn, (shift, lift) in enumerate(lay):
        path = []

        for step in range(steps + 1):
            here, _, across, out = frame(step)
            angle = math.tau * step / steps
            wander = 0.16 * math.sin(angle * 3 + turn * 1.9)
            rise = 0.1 * abs(math.sin(angle * 2 + turn * 2.6))

            path.append(here + across * ((shift + wander) * pitch) + out * ((lift + rise) * pitch + thickness))

        parts.append(core.tube(f"coil_{turn}", path, thickness, rope, sides=7, cap=False))

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
    fall += [_around(fast - 0.06, height, 0.035) for height in (1.36, 1.24, 1.1)]
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


def _half_cape(group, alt):
    """A tarred cape over the off shoulder: stiff, creased and cut on the
    slant, held by a line around the body."""

    start, end = math.radians(-64), math.radians(100)
    top = 1.745
    columns, rows = 44, 24

    def hem(along):
        """Short over the chest and the arm, long down the back."""

        crack = 0.03 * abs(noise.noise(Vector((along * 9.0, 3.0, 1.0))))

        return 1.44 - 0.12 * _ease(along / 0.45) - 0.27 * _ease((along - 0.45) / 0.5) + crack

    def crease(along):
        """Tarred cloth does not ripple: it breaks into flat planes."""

        return abs((along * 5.5 + 0.3) % 1.0 - 0.5) * 2

    grid = []

    for row in range(rows + 1):
        line = []

        for column in range(columns + 1):
            along = column / columns
            height = top - (top - hem(along)) * row / rows
            hang = _ease((1.42 - height) / 0.3)

            # Lower down it stands off the robe, whose folds move under it.
            line.append(_over_shoulder(start + (end - start) * along, height, 0.009 + hang * (0.035 + 0.03 * crease(along))))

        grid.append(line)

    cape = core.loft("half_cape", grid, alt, closed=False)

    # A bound edge, which is what makes it read as cut cloth and not a shell.
    border = [line[0] for line in grid] + grid[-1][1:] + [line[-1] for line in reversed(grid[:-1])]
    welt = core.tube("half_cape_welt", border, 0.009, alt, sides=6)

    cape = core.join([cape, welt], "half_cape")
    cape.parent = group
    finish(core.shade_smooth(cape, angle=40), seed=7)

    # The line runs from the cape's front edge under the pole arm and round
    # to its back edge. It starts flat to the chest, under the cowl's cape.
    line = []

    for step in range(41):
        share = step / 40
        angle = start - (math.tau - (end - start)) * share

        line.append(_around(angle, 1.47 - 0.2 * math.sin(share * math.pi) - 0.12 * share, 0.013 + 0.015 * _ease(share * 6)))

    finish(core.tube("half_cape_line", line, 0.012, core.material("rope"), group, sides=7))


def _net(group):
    """A net thrown over the off shoulder, weighted with floats."""

    rope = core.material("rope")
    start, end = math.radians(-66), math.radians(150)
    top = 1.73
    cells = 18
    rows = 9
    strands = []
    floats = []

    def knot(column, row):
        """A knot of the net, counted across and down. The hem sags under
        each float and rises between them."""

        along = column / cells
        hem = 1.4 - 0.24 * math.sin(along * math.pi) ** 0.7 - 0.02 * math.cos(column * math.pi)
        height = top - (top - hem) * row / rows

        return _over_shoulder(start + (end - start) * along, height, 0.007 + 0.04 * _ease((1.42 - height) / 0.25))

    # Two sets of strands cross on the slant to make the mesh.
    for lean in (-1, 1):
        for strand in range(-rows, cells + 1):
            path = []

            for row in range(rows + 1):
                column = strand + row if lean > 0 else strand + rows - row

                if 0 <= column <= cells:
                    path.append(knot(column, row))

            if len(path) > 1:
                strands.append(core.tube(f"net_{lean}_{strand}", path, 0.0045, rope, sides=4, cap=False))

    for line, row in enumerate((0, rows)):
        path = [knot(column / 2, row) for column in range(cells * 2 + 1)]

        strands.append(core.tube(f"net_line_{line}", path, 0.009, rope, sides=5))

    mesh = core.join(strands, "net")
    mesh.parent = group
    finish(mesh, wear_boost=1.5)

    for column in range(0, cells + 1, 2):
        cork = core.revolve(
            f"float_{column}",
            [(0.0, 0.034), (0.018, 0.03), (0.034, 0.009), (0.034, -0.009), (0.018, -0.03), (0.0, -0.034)],
            core.material("wood"),
            segments=10,
        )
        cork.location = knot(column, rows) + Vector((0, 0, -0.034))
        floats.append(cork)

    corks = core.join(floats, "net_floats")
    corks.parent = group
    finish(core.shade_smooth(corks), wear_boost=1.4)
