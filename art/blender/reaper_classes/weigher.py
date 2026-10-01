"""The weigher: the judge who measures a spirit before taking it.

Silhouette: a tall, narrow two-pronged crest on a smooth, faceless mask, over
a straight column of a body. The balance is held upright before the body, so
its pans hang to either side like a second pair of shoulders.
"""

import math

from mathutils import Vector

from lib import cloth, core
from lib.figure import HEAD_HEIGHT, NECK_HEIGHT, SHOULDER_HEIGHT, arms, finish

# The mask is longer than a face: it runs from the collar to above the brow.
MASK_CHIN = NECK_HEIGHT + 0.01
MASK_CROWN = HEAD_HEIGHT + 0.17
MASK_HALF_WIDTH = 0.1
MASK_BULGE = 0.07

# The hood the mask floats in, and where the crest stands in front of it.
HOOD_CENTRE = Vector((0, 0.015, HEAD_HEIGHT + 0.02))
CREST_DEPTH = -0.085


def build():
    """The judge: upright, symmetrical and faceless."""

    root = core.empty("reaper")
    cloth_mat = core.material("cloth")

    # A straight column, fluted like a pillar, with a hem that has barely
    # frayed.
    hems = [
        dict(tatter=0.1, tears=3, slits=3, fold_depth=0.2, seed=31),
        dict(tatter=0.2, tears=6, slits=7, fold_depth=0.24, seed=32),
        dict(tatter=0.04, tears=2, slits=0, fold_depth=0.16, seed=33),
        dict(tatter=0.14, tears=4, slits=4, fold_depth=0.28, seed=34),
    ]

    for index, hem in enumerate(hems):
        finish(
            cloth.robe(
                f"robe_{index}",
                cloth_mat,
                root,
                top=SHOULDER_HEIGHT + 0.06,
                hem=0.08,
                profile=((0.0, 0.11, 0.1), (0.05, 0.21, 0.125), (0.14, 0.225, 0.135), (0.5, 0.2, 0.15), (1.0, 0.25, 0.21)),
                folds=(12, 6, 18),
                **hem,
            ),
            seed=index,
        )

    # The weapon hand comes to the middle of the waist, so the staff stands
    # on the figure's centre line and the beam crosses just under the mask.
    arms(
        root,
        weapon_side_path=[(-0.2, 0, SHOULDER_HEIGHT - 0.03), (-0.265, -0.02, 1.32), (-0.19, -0.12, 1.12), (-0.14, -0.2, 1.08)],
        off_side_path=[(0.2, 0, SHOULDER_HEIGHT - 0.03), (0.26, 0.0, 1.3), (0.265, -0.03, 1.0)],
        sleeve_cuff=0.105,
        droop=0.2,
    )

    head = core.empty("head", root, location=(0, 0, NECK_HEIGHT))
    _heads(head)
    _mantles(root)

    return root


def _mask_row(share):
    """The mask at `share` of the way from chin to crown: its height, how far
    forward its rim sits, its half-width and how far its face stands proud of
    that rim."""

    egg = share**1.35
    width = MASK_HALF_WIDTH * math.sqrt(max(0.0, 1 - abs(2 * egg - 1) ** 2.4))
    rim = -0.09 + 0.03 * (2 * share - 1) ** 2
    bulge = MASK_BULGE * (0.25 + 0.75 * width / MASK_HALF_WIDTH)

    return MASK_CHIN + (MASK_CROWN - MASK_CHIN) * share, rim, width, bulge


def _stack(name, rows, mat, parent=None, segments=36, capped=False):
    """Skins a form from level ovals. Each row is (height, depth, half-width,
    front, back): an oval centred `depth` along Y that reaches `front` ahead
    of that centre and `back` behind it."""

    rings = []

    for height, depth, width, front, back in rows:
        ring = []

        for segment in range(segments):
            angle = math.tau * segment / segments
            behind = math.sin(angle)

            ring.append((math.cos(angle) * width, depth + behind * (back if behind > 0 else front), height))

        rings.append(ring)

    return core.loft(name, rings, mat, parent, cap_start=capped, cap_end=capped)


def _cowl(name, mat, radii, opening, fall, rings=24, segments=48):
    """A close hood: an egg around the head, lying fore and aft. The front is
    cut away to `opening`, a share of the egg's width, and the edge is turned
    in, which is the hole the mask floats in. Behind that the lower half
    hangs square to `fall` below the centre, in soft folds and to a rounded
    hem at the back, so the hood drops straight from the temples to the
    shoulders and there is no neck."""

    start = math.asin(opening)
    loops = []

    for ring in range(-1, rings + 1):
        # The first loop is the turned-in lip.
        lip = ring < 0
        polar = start + (math.pi - 0.06 - start) * max(ring, 0) / rings
        scale = math.sin(polar) * (0.9 if lip else 1.0)
        depth = -radii[1] * math.cos(polar) + (0.03 if lip else 0.0)
        hang = min(1.0, (polar - start) / 0.45)
        hang = hang * hang * (3 - 2 * hang)
        hem = radii[2] * scale * (1 - hang) + fall * (1 + 0.3 * max(0.0, -math.cos(polar)) ** 0.5) * hang
        loop = []

        for segment in range(segments):
            angle = math.tau * segment / segments
            across = math.cos(angle)
            rise = radii[2] * scale * math.sin(angle)
            fold = 0.0

            if rise < 0:
                square = 1 - 0.6 * hang
                drop = abs(math.sin(angle)) ** square
                across = math.copysign(abs(across) ** square, across)
                rise = -drop * hem
                fold = 0.007 * hang * drop * math.sin(polar * 14)

            loop.append(HOOD_CENTRE + Vector((radii[0] * scale * across, depth + fold, rise)))

        loops.append(loop)

    return core.loft(name, loops, mat, cap_end=True)


def _crest(name, mat, parent, prong, bar=None):
    """Two prongs rising from the crown of the mask. `prong` lists (x, z)
    knots for one of them, from the root they share to its tip; the other
    mirrors it. `bar` is the height of a rail joining the two."""

    knots = [(-x, CREST_DEPTH, z) for x, z in reversed(prong[1:])] + [(x, CREST_DEPTH, z) for x, z in prong]
    parts = [
        core.tube(
            name,
            core.bezier(knots, samples=8),
            lambda t: 0.007 + 0.016 * (1 - abs(2 * t - 1)) ** 0.7,
            mat,
            sides=8,
            flatten=0.4,
        )
    ]

    if bar is not None:
        reach = max(x for x, z in prong if z <= bar) + 0.05

        parts.append(
            core.tube(
                f"{name}_bar",
                [(-reach, CREST_DEPTH, bar), (0, CREST_DEPTH, bar), (reach, CREST_DEPTH, bar)],
                lambda t: 0.009 + 0.006 * abs(2 * t - 1) ** 6,
                mat,
                sides=8,
                flatten=0.6,
            )
        )

    result = core.join(parts, name)
    result.parent = parent

    return core.shade_smooth(result, angle=50)


def _light(name, parent, length):
    """The weigher's one light: a cold slit in the brow of the mask. It is
    built around its own origin, so the game can scale it where it sits."""

    return core.tube(
        name,
        [(0, 0, length * (step / 6 - 0.5)) for step in range(7)],
        lambda t: 0.001 + 0.0065 * math.sin(t * math.pi) ** 0.6,
        core.material("accent"),
        parent,
        sides=6,
    )


def _heads(head):
    """Three crested masks. The mask never changes; the crest does."""

    gold = core.material("gold")
    hood = core.material("cloth")
    offset = Vector(head.location)

    def place(obj):
        obj.location = Vector(obj.location) - offset

        return obj

    variants = [
        # Straight and parallel, like a tuning fork.
        dict(prong=[(0.0, 1.99), (0.043, 2.08), (0.047, 2.24), (0.047, 2.42), (0.045, 2.6)], brow=0.74, slit=0.06),
        # Bowed out and drawn back in, like the arms of a lyre.
        dict(prong=[(0.0, 1.99), (0.06, 2.07), (0.112, 2.2), (0.1, 2.35), (0.052, 2.47), (0.05, 2.6)], brow=0.76, slit=0.016),
        # Flared, with a rail across them like a beam at rest.
        dict(prong=[(0.0, 1.99), (0.045, 2.08), (0.075, 2.24), (0.115, 2.42), (0.165, 2.58)], brow=0.72, slit=0.085, bar=2.33),
    ]

    face = [(*_mask_row(0.015 + 0.97 * row / 26), 0.012) for row in range(27)]

    for index, variant in enumerate(variants):
        group = core.empty(f"head_{index}", head)

        # The mask and its crest outlast the figure: hollowing never eats them.
        mask = core.shade_smooth(_stack(f"mask_{index}", face, gold, group, capped=True), angle=50)
        place(finish(mask, hollow=lambda co: 1.0, wear_boost=1.4, seed=index))
        crest = _crest(f"crest_{index}", gold, group, variant["prong"], variant.get("bar"))
        place(finish(crest, hollow=lambda co: 1.0, wear_boost=1.6, seed=index))

        # A close hood, and the cloth at the throat that it is tucked into.
        cowl = _cowl(f"cowl_{index}", hood, (0.15, 0.165, 0.185), opening=0.82, fall=0.265)
        throat = _stack(
            f"throat_{index}",
            [
                (NECK_HEIGHT + 0.04, HOOD_CENTRE.y, 0.12, 0.085, 0.1),
                (NECK_HEIGHT, HOOD_CENTRE.y, 0.13, 0.095, 0.105),
                (NECK_HEIGHT - 0.03, HOOD_CENTRE.y, 0.15, 0.118, 0.112),
                (NECK_HEIGHT - 0.055, HOOD_CENTRE.y, 0.178, 0.136, 0.118),
            ],
            hood,
            segments=40,
        )
        wrap = core.join([cowl, throat], f"wrap_{index}")
        wrap.parent = group
        place(finish(core.shade_smooth(wrap), seed=index))

        # The darkness that fills the hood, seen in the gap around the mask.
        levels = [math.radians(-80 + 160 * level / 10) for level in range(11)]
        void = _stack(
            f"void_{index}",
            [(HOOD_CENTRE.z + 0.17 * math.sin(level), HOOD_CENTRE.y, 0.135 * math.cos(level), 0.15 * math.cos(level), 0.15 * math.cos(level)) for level in levels],
            core.material("void"),
            group,
            segments=24,
            capped=True,
        )
        place(void)

        height, rim, _, bulge = _mask_row(variant["brow"])
        light = _light(f"eyes_{index}", group, variant["slit"])
        light.location = Vector((0, rim - bulge, height)) - offset


def _band(name, path, across, width, mat, parent=None, thickness=0.012, pleat=0.004, columns=8, closed=False):
    """A flat strip of cloth laid along `path`. `across` gives, for each
    point of the path, the direction the strip's width runs in, which is what
    lets it lie against the body under it. A `closed` strip is a loop."""

    points = [Vector(point) for point in path]
    count = len(points)
    rings = []

    for index, (point, side) in enumerate(zip(points, across)):
        ahead = points[(index + 1) % count] if closed else points[min(index + 1, count - 1)]
        behind = points[index - 1] if closed else points[max(index - 1, 0)]
        side = Vector(side).normalized()
        normal = side.cross(ahead - behind).normalized()
        ring = []

        for face in (1, -1):
            for column in range(columns + 1):
                share = face * (2 * column / columns - 1)
                fold = pleat * math.cos(share * math.pi * 3)

                ring.append(point + side * share * width / 2 + normal * (fold + face * thickness / 2))

        rings.append(ring)

    if closed:
        rings.append(rings[0])

    return core.loft(name, rings, mat, parent, cap_start=not closed, cap_end=not closed)


def _weight(name, top, mat, drop=0.1, girth=0.02):
    """A plumb weight hanging from `top`, like the ones a scale is set with."""

    top = Vector(top)

    return core.tube(
        name,
        [top - Vector((0, 0, drop * step / 8)) for step in range(9)],
        lambda t: girth * (0.2 + 0.8 * max(0.0, math.sin(t**1.5 * math.pi)) ** 0.8),
        mat,
        sides=8,
    )


def _mantles(root):
    """What sits on a weigher's shoulders."""

    linen = core.material("cloth_alt")
    gold = core.material("gold")

    # A long stole over the off shoulder, held straight by a weight at each
    # end. The front end hangs lower than the back. Each knot is (y, z, turn),
    # where the turn keeps the stole flat against the round of the body.
    group = core.empty("mantle_0", root)
    line = 0.135
    knots = [
        (-0.135, 0.78, 33),
        (-0.135, 1.15, 33),
        (-0.125, 1.48, 30),
        (-0.088, 1.63, 20),
        (-0.045, 1.692, 8),
        (0.0, 1.706, 0),
        (0.045, 1.692, -8),
        (0.088, 1.63, -20),
        (0.125, 1.48, -30),
        (0.14, 1.25, -33),
        (0.14, 1.02, -33),
    ]
    samples = core.bezier(knots, samples=6)

    def turned(turn):
        return Vector((math.cos(math.radians(turn)), math.sin(math.radians(turn)), 0))

    stole = _band(
        "stole",
        [(line, depth, height) for depth, height, _ in samples],
        [turned(turn) for _, _, turn in samples],
        0.14,
        linen,
        group,
    )
    finish(stole, seed=2)

    weights = []

    for end, (depth, height, turn) in enumerate((knots[0], knots[-1])):
        middle = Vector((line, depth, height))
        reach = turned(turn) * 0.082

        weights.append(core.tube(f"stole_rod_{end}", [middle - reach, middle, middle + reach], 0.011, gold, sides=8))
        weights.append(_weight(f"stole_weight_{end}", middle - Vector((0, 0, 0.008)), gold))

    weighting = core.join(weights, "stole_weights")
    weighting.parent = group
    finish(core.shade_smooth(weighting, angle=50), wear_boost=1.6)

    # A broad collar plate in three tiers, lying on the shoulders and chest,
    # with a counterweight down the back to keep it there.
    group = core.empty("mantle_1", root)
    profile = [(0.0, -0.004), (0.0, 0.012)]

    for tier in range(3):
        inner = tier / 3
        outer = (tier + 1) / 3
        lift = 0.012 - tier * 0.004

        profile.extend([(inner + 0.03, lift + 0.006), (outer - 0.03, lift + 0.004), (outer, lift - 0.004)])

    profile.extend([(1.0, -0.008), (0.0, -0.004)])

    def collar(angle, share, lift):
        """Lays the plate over the body: its sides run out along the
        shoulders, its front and back fall steeply down the chest."""

        wide = 0.105 + 0.205 * share
        deep = 0.1 + 0.07 * share
        radius = wide * deep / math.hypot(deep * math.cos(angle), wide * math.sin(angle))
        chest = math.sin(angle) ** 2
        height = NECK_HEIGHT + 0.01 - (0.135 * share**1.8) * (1 - chest) - 0.23 * share * chest

        return radius, height + lift

    plate = core.revolve("collar", profile, gold, group, segments=72, shape=collar)
    finish(core.shade_smooth(plate, angle=40), wear_boost=1.6, seed=4)

    counterweight = core.plate(
        "collar_counterweight",
        [(-0.018, 1.53), (0.018, 1.53), (0.022, 1.34), (0.05, 1.22), (0.03, 1.13), (0.0, 1.1), (-0.03, 1.13), (-0.05, 1.22), (-0.022, 1.34)],
        0.012,
        gold,
        group,
        bevel=0.003,
    )
    counterweight.location = (0, 0.172, 0)
    finish(counterweight, wear_boost=1.6)

    # Linen wound across the chest and over both upper arms, binding them in,
    # with one loose end left hanging.
    group = core.empty("mantle_2", root)
    wraps = []

    for wrap in range(6):
        height = SHOULDER_HEIGHT - wrap * 0.048
        # The windings widen with the arms under them, and every other one
        # lies over its neighbours.
        reach = 0.285 + 0.009 * wrap + 0.007 * (wrap % 2)
        tilt = (wrap % 2 * 2 - 1) * (0.022 + 0.008 * (wrap % 3))
        path = []

        for step in range(48):
            angle = math.tau * step / 48
            across = math.cos(angle)
            along = math.sin(angle)

            path.append(
                (
                    math.copysign(abs(across) ** 0.75, across) * reach,
                    math.copysign(abs(along) ** 0.75, along) * (0.15 + 0.007 * (wrap % 2)),
                    height + tilt * math.cos(angle - wrap * 0.9) + 0.006 * math.sin(angle * 3 + wrap * 2.1),
                )
            )

        wraps.append(_band(f"winding_{wrap}", path, [(0, 0, 1)] * len(path), 0.072, linen, thickness=0.008, pleat=0.0025, columns=4, closed=True))

    winding = core.join(wraps, "winding")
    winding.parent = group
    finish(core.shade_smooth(winding, angle=40), seed=6)

    finish(
        cloth.drape(
            "winding_tail",
            linen,
            group,
            [(0.1 + 0.075 * step / 8, -0.146 + 0.01 * step / 8, SHOULDER_HEIGHT - 0.25) for step in range(9)],
            drop=0.52,
            width_gain=1.0,
            tatter=0.25,
            sag=0.01,
            folds=3,
            fold_depth=0.008,
            seed=9,
            outward=(0, -1, 0),
        ),
        seed=7,
    )
