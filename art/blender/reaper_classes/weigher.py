"""The weigher: the judge who measures a spirit before taking it.

Silhouette: the figure is a balance. A tall, narrow column of a body with
square, level shoulders, a flat chest and a hard step in at the waist, is
the post; two tall gilt ears are its finial, over a faceless bone mask in a
long hood. Everything about it is level and symmetrical but its one
asymmetry: the staff, planted upright off its centre line on the weapon
side, so the beam crosses it below the shoulders, well clear of the head,
one pan hanging out far past that side and the other before the chest.

Three families on show: the cloth of the robe, the hood and the mantles;
the gilt of the ears, the class's light; and the bone of the mask and the
hands, the pale planes the eye goes to.
"""

import math

import bpy
from mathutils import Vector

from lib import cloth, core
from lib.figure import arms, finish, idle_path

# The weigher stands taller than the other classes, the head lifted on a
# long cowl so the beam, which the planted staff holds 1.65 m up at its
# high end, clears the chin by 0.29 m: about 15% of a reaper's standing
# height (docs/art-bible.md 6.2).
SHOULDER = 1.68
NECK = 1.92
HEAD = NECK + 0.15

# The mask is longer than a face: it runs from the collar to above the brow.
MASK_CHIN = NECK + 0.02
MASK_CROWN = HEAD + 0.15
MASK_HALF_WIDTH = 0.096
MASK_BULGE = 0.075

# The hood the mask sits in.
HOOD_CENTRE = Vector((0, 0.02, HEAD + 0.02))
HOOD_RADII = (0.15, 0.165, 0.205)

# The robe: a carved post. Square at the shoulder, a flat chest, then a
# hard step in at the waist (`WAIST`, 14% narrower) to a straight fluted
# skirt.
ROBE_TOP = SHOULDER + 0.07
ROBE_HEM = 0.05
WAIST = 1.1
ROBE_PROFILE = (
    (0.0, 0.12, 0.11),
    (0.022, 0.265, 0.165),
    (0.1, 0.265, 0.18),
    (0.355, 0.255, 0.18),
    (0.39, 0.22, 0.155),
    (1.0, 0.24, 0.185),
)
# Where the chest is cut flat, in front of the middle, and how far down it
# runs: to the waist.
CHEST_PLANE = -0.135

# Where the staff stands in the idle stance: off the centre line on the
# weapon side, held out a hand's width past the robe's line so a gap of
# ground shows between staff and column from the front, and any weapon held
# there stands clear of the body, its foot on the floor (the balance-flail's
# runs 0.95 below its grip). The staff leans out by the beam's own tip, so
# the beam hangs level and square to the front (lib/motion.py `HOLDS`): its
# far pan hangs out past the weapon side, the near one before the chest.
STAFF_GRIP = Vector((-0.41, -0.4, 0.975))
SHOULDER_X = 0.215


def build():
    """The judge: upright, symmetrical and faceless."""

    root = core.empty("reaper")
    cloth_mat = core.material("cloth")

    # A fluted column that has barely frayed: regular flutes, a level hem.
    hems = [
        dict(folds=(9, 18, 27), fold_depth=0.12, tatter=0.06, tears=2, slits=0, seed=31),
        dict(folds=(7, 14, 21), fold_depth=0.15, tatter=0.12, tears=4, slits=2, seed=32),
        dict(folds=(11, 22, 33), fold_depth=0.09, tatter=0.03, tears=0, slits=0, seed=33),
        dict(folds=(8, 16, 24), fold_depth=0.14, tatter=0.16, tears=3, slits=3, seed=34),
    ]

    for index, hem in enumerate(hems):
        robe = cloth.robe(
            f"robe_{index}",
            cloth_mat,
            root,
            top=ROBE_TOP,
            hem=ROBE_HEM,
            profile=ROBE_PROFILE,
            rings=60,
            segments=96,
            **hem,
        )
        finish(_carved(robe), seed=index)

    # The weapon hand closes on the staff off the centre line; the other arm
    # hangs straight down the side, a fist, as a statue's does. The elbows
    # stay inside the robe's line, so the column runs straight.
    arms(
        root,
        weapon_side_path=idle_path("weigher", "r", (-SHOULDER_X, 0, SHOULDER - 0.03), (-0.3, -0.1, 1.3), STAFF_GRIP),
        off_side_path=idle_path("weigher", "l", (SHOULDER_X, 0, SHOULDER - 0.03), (0.25, -0.03, 1.3), Vector((0.26, -0.1, 0.98))),
        sleeve_shoulder=0.078,
        sleeve_cuff=0.1,
        droop=0.1,
        off_curl=0.85,
        tatter=0.12,
    )

    head = core.empty("head", root, location=(0, 0, NECK))
    _heads(head)
    _mantles(root)

    return root


def _carved(robe):
    """Cuts the chest flat: in front of `CHEST_PLANE`, from the shoulders to
    the waist, the robe is pressed back onto one plane, which takes a single
    band, and its folds with it. The same press is made in every shape key,
    so the game's sway keeps it."""

    vertices = robe.data.vertices
    moves = []

    for vertex in vertices:
        co = vertex.co
        # Full height of the chest, easing out over the step at the waist.
        reach = max(0.0, min(1.0, (co.z - WAIST) / 0.08))
        proud = min(0.0, co.y - CHEST_PLANE)
        moves.append(-proud * 0.85 * reach * reach * (3 - 2 * reach))

    for vertex, move in zip(vertices, moves):
        vertex.co.y += move

    if robe.data.shape_keys:
        for block in robe.data.shape_keys.key_blocks:
            for point, move in zip(block.data, moves):
                point.co.y += move

    robe.data.update()

    return robe


def _mask_row(share):
    """The mask at `share` of the way from chin to crown: its height, how far
    forward its rim sits, its half-width and how far its face stands proud of
    that rim."""

    # A long shield: widest at the brow, drawn to a blunt point at the chin.
    width = MASK_HALF_WIDTH * (0.55 + 0.45 * math.sin(min(1.0, share / 0.62) * math.pi / 2)) * math.sqrt(max(0.0, 1 - max(0.0, (share - 0.78) / 0.22) ** 2.2))
    rim = -0.095 + 0.03 * (2 * share - 1) ** 2

    return MASK_CHIN + (MASK_CROWN - MASK_CHIN) * share, rim, width, MASK_BULGE * (0.4 + 0.6 * width / MASK_HALF_WIDTH)


def _mask(name, mat, parent=None, rows=18, segments=28):
    """A faceless bone mask cut in planes: two broad cheeks meeting at a
    ridge down the middle, a brow shelf across the top, the whole of it a
    blunt shield. Planes take a band edge cleanly; relief would print as
    mud."""

    rings = []

    for row in range(rows + 1):
        share = 0.01 + 0.98 * row / rows
        height, rim, width, bulge = _mask_row(share)
        # The brow: a shelf that juts over the eye line.
        shelf = 0.012 * math.exp(-(((share - 0.7) / 0.05) ** 2))
        ring = []

        for segment in range(segments):
            angle = math.tau * segment / segments
            across, depth = math.cos(angle), math.sin(angle)

            if depth > 0:
                # The back: a shallow closed shell inside the hood.
                ring.append((across * width, rim + depth * 0.016, height))
            else:
                # The front: two planes meeting at the ridge, a wedge.
                front = -depth
                wedge = 1 - 0.55 * abs(across)
                ring.append((across * width, rim - front * (bulge * wedge + shelf), height))

        rings.append(ring)

    mask = core.loft(name, rings, mat, parent, cap_start=True, cap_end=True)

    return core.shade_smooth(mask, angle=30)


def _ear(name, side, base, height, width, splay, lean, cup=0.024):
    """One tall ear: a jackal's, a gilt blade broad at the root and drawn
    straight to a point, cupped toward the front with its hollow lined in
    the hood's cloth, standing from the crown at `base`. `splay` turns it
    out from the middle, `lean` tips it back. Returns the gilt and the
    lining."""

    # The inner edge runs straight from root to tip; the outer one bellies.
    outline = [
        (-width * 0.85, 0.0),
        (-width * 0.62, height * 0.45),
        (0.0, height),
        (width * 0.52, height * 0.62),
        (width * 0.92, height * 0.3),
        (width, 0.0),
    ]
    lining = [(x * 0.62 + width * 0.06, z * 0.8 + height * 0.06) for x, z in outline]
    parts = []

    for label, shape, thickness, mat, forward in ((name, outline, 0.024, core.material("gold"), 0.0), (f"{name}_lining", lining, 0.01, core.material("cloth"), 0.012)):
        plate = core.plate(label, [(side * x, z) for x, z in shape], thickness, mat, bevel=0.0)

        # Cupped: the edges come forward of the middle, so the ear catches a
        # band edge down its length.
        for vertex in plate.data.vertices:
            across = vertex.co.x / width
            vertex.co.y -= cup * across * across + forward

        plate.data.update()
        plate.rotation_euler = (-lean, side * splay, 0)
        plate.location = base
        core.activate(plate)
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
        parts.append(core.crisp(plate, 0.004 if thickness > 0.02 else 0.002))

    return parts


def _cowl(name, mat, opening, fall, rings=20, segments=40):
    """A close hood: an egg around the head, lying fore and aft. The front is
    cut away to `opening`, a share of the egg's width, and the edge is turned
    in, which is the hole the mask sits in. Behind that the lower half hangs
    square to `fall` below the centre, so the hood drops straight from the
    temples to the shoulders and there is no neck."""

    radii = HOOD_RADII
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

            if rise < 0:
                # Below the middle the hood squares off over the shoulders.
                square = 1 - 0.7 * hang
                drop = abs(math.sin(angle)) ** square
                across = math.copysign(abs(across) ** square, across) * (1 + 0.35 * hang * drop)
                rise = -drop * hem

            loop.append(HOOD_CENTRE + Vector((radii[0] * scale * across, depth, rise)))

        loops.append(loop)

    return core.loft(name, loops, mat, cap_end=True)


def _heads(head):
    """Three heads, each a bone mask in a close hood under gilt ears. The
    mask never changes; the ears do."""

    gold = core.material("gold")
    bone = core.material("bone")
    hood = core.material("cloth")
    offset = Vector(head.location)

    def place(obj, group):
        obj.parent = group
        obj.location = Vector(obj.location) - offset

        return obj

    crown = MASK_CROWN - 0.02
    variants = [
        # Tall and pricked, a jackal's on watch.
        dict(height=0.5, width=0.075, splay=math.radians(9), lean=math.radians(6), apart=0.085),
        # Broad and splayed wide, laid out like a balance's arms.
        dict(height=0.42, width=0.095, splay=math.radians(20), lean=math.radians(10), apart=0.095),
        # Tall and narrow, with the sun held between them: a gilt disc.
        dict(height=0.48, width=0.065, splay=math.radians(12), lean=math.radians(4), apart=0.1, disc=0.1),
    ]

    for index, variant in enumerate(variants):
        group = core.empty(f"head_{index}", head)

        # The mask and its ears outlast the figure: hollowing never eats them.
        mask = _mask(f"mask_{index}", bone)
        place(finish(mask, hollow=lambda co: 1.0, wear_boost=1.4, seed=index), group)

        # Gilt and lining are separate parts: the game draws a part in one
        # family, by its name (crates/reapers_app/src/render/parts.rs).
        parts = []
        linings = []

        for side in (-1, 1):
            base = Vector((side * variant["apart"], HOOD_CENTRE.y + 0.02, crown))
            ear, lining = _ear(f"ear_{index}_{side}", side, base, variant["height"], variant["width"], variant["splay"], variant["lean"])
            parts.append(ear)
            linings.append(lining)

        if "disc" in variant:
            disc = core.revolve(
                f"sun_{index}",
                [(0.0, 0.012), (variant["disc"] * 0.92, 0.012), (variant["disc"], 0.0), (variant["disc"] * 0.92, -0.012), (0.0, -0.012)],
                gold,
                segments=32,
            )
            disc.rotation_euler = (math.pi / 2, 0, 0)
            disc.location = (0, HOOD_CENTRE.y + 0.03, crown + variant["disc"] + 0.03)
            core.activate(disc)
            bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
            parts.append(core.crisp(disc, 0.004))

        ears = core.join(parts, f"ears_{index}")
        place(finish(ears, hollow=lambda co: 1.0, wear_boost=1.6, seed=index), group)
        place(finish(core.join(linings, f"ear_linings_{index}"), seed=index), group)

        # A close hood, squared over the shoulders: the cloth at the throat
        # is part of it.
        cowl = _cowl(f"wrap_{index}", hood, opening=0.8, fall=0.3 + 0.04 * index)
        place(finish(core.shade_smooth(cowl, angle=50), seed=index), group)

        # The darkness that fills the hood, seen in the gap round the mask.
        void = core.sculpt(
            f"void_{index}",
            [("sphere", (0, HOOD_CENTRE.y - 0.02, HEAD + 0.01), (0.125, 0.12, 0.17))],
            core.material("void"),
            voxel=0.02,
            smooth=1,
        )
        place(core.decimate(void, 0.5), group)


def _band(name, path, across, width, mat, parent=None, thickness=0.014, columns=4, closed=False):
    """A flat, stiff strip laid along `path`. `across` gives, for each point
    of the path, the direction the strip's width runs in, which is what lets
    it lie against the body under it. A `closed` strip is a loop."""

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

                ring.append(point + side * share * width / 2 + normal * (face * thickness / 2))

        rings.append(ring)

    if closed:
        rings.append(rings[0])

    return core.loft(name, rings, mat, parent, cap_start=not closed, cap_end=not closed)


def _plumb(name, top, mat, drop=0.11, girth=0.024):
    """A plumb weight hanging from `top`, like the ones a scale is set with:
    a faceted bob, six-sided, so its planes take the bands."""

    top = Vector(top)

    return core.tube(
        name,
        [top - Vector((0, 0, drop * step / 6)) for step in range(7)],
        lambda t: girth * (0.25 + 0.75 * max(0.0, math.sin(t**1.4 * math.pi)) ** 0.7),
        mat,
        sides=6,
    )


def _mantles(root):
    """What sits on a weigher's shoulders. Every one squares them, and every
    one is the robe's cloth, so the gilt stays on the ears and the pans."""

    cloth_mat = core.material("cloth")

    # A stiff stole: two broad bands from behind the neck down the chest,
    # level and straight, each held plumb by a weight.
    group = core.empty("mantle_0", root)
    parts = []

    for side in (-1, 1):
        x = side * 0.115
        knots = [(x * 0.6, 0.12, ROBE_TOP + 0.005), (x * 0.92, -0.02, ROBE_TOP - 0.015), (x, -0.135, SHOULDER - 0.07), (x, -0.155, 1.3), (x, -0.16, 0.92)]
        path = core.bezier(knots, samples=5)
        parts.append(_band(f"stole_{side}", path, [(1, 0, 0)] * len(path), 0.1, cloth_mat))
        parts.append(_plumb(f"stole_weight_{side}", (x, -0.16, 0.92), cloth_mat, drop=0.14, girth=0.034))

    mantle = core.join(parts, "stole")
    mantle.parent = group
    finish(mantle, wear_boost=1.6, seed=2)

    # A broad collar plate in three tiers, lying flat on the shoulders and
    # cut square at the sides, with a counterweight down the back.
    group = core.empty("mantle_1", root)
    profile = [(0.0, -0.004), (0.0, 0.012)]

    for tier in range(3):
        inner = tier / 3
        outer = (tier + 1) / 3
        lift = 0.012 - tier * 0.004

        profile.extend([(inner + 0.03, lift + 0.006), (outer - 0.03, lift + 0.004), (outer, lift - 0.004)])

    profile.extend([(1.0, -0.008), (0.0, -0.004)])

    def collar(angle, share, lift):
        """Lays the plate over the body: square over the shoulders, falling
        down the chest and back."""

        wide = 0.12 + 0.2 * share
        deep = 0.11 + 0.06 * share
        across, along = math.cos(angle), math.sin(angle)
        # A superellipse: the sides run straight out over the shoulders.
        radius = 1 / ((abs(across) / wide) ** 4 + (abs(along) / deep) ** 4) ** 0.25
        chest = along * along
        height = ROBE_TOP + 0.01 - (0.03 * share) * (1 - chest) - 0.2 * share**1.4 * chest

        return radius, height + lift

    plate = core.revolve("collar", profile, cloth_mat, group, segments=64, shape=collar)
    finish(core.shade_smooth(plate, angle=40), wear_boost=1.6, seed=4)

    counterweight = core.plate(
        "collar_counterweight",
        [(-0.02, 1.62), (0.02, 1.62), (0.024, 1.42), (0.055, 1.3), (0.032, 1.2), (0.0, 1.17), (-0.032, 1.2), (-0.055, 1.3), (-0.024, 1.42)],
        0.016,
        cloth_mat,
        group,
        bevel=0.004,
    )
    counterweight.location = (0, 0.18, 0)
    finish(counterweight, wear_boost=1.6)

    _tabbed_cape(core.empty("mantle_2", root), cloth_mat)


def _tabbed_cape(group, mat):
    """A stiff cape cut as a tabard: a short yoke, wide and square over the
    shoulders, and hung from it three long panels before the body and one
    behind, each drawn in to its foot. It makes the shoulders a lintel with
    lappets hung from it, symmetrical, and every panel one plane."""

    top = ROBE_TOP + 0.02
    yoke = ROBE_TOP - 0.13

    def ring(angle, z):
        """The point on the cape at `angle` round the body and height `z`:
        out over the shoulders at once below the neck, then drawn in."""

        spread = _ease((top - z) / 0.09)
        taper = max(0.0, (yoke - z) / 0.4)
        wide = 0.12 + 0.22 * spread - 0.06 * taper
        deep = 0.12 + 0.085 * spread + 0.01 * taper
        across, along = math.cos(angle), math.sin(angle)
        radius = 1 / ((abs(across) / wide) ** 5 + (abs(along) / deep) ** 5) ** 0.2

        return Vector((across * radius, along * radius, z))

    # The yoke, all the way round.
    columns = 72
    rows = [[ring(math.tau * column / columns, top - (top - yoke) * row / 5) for column in range(columns)] for row in range(6)]
    parts = [core.loft("cape_yoke", rows, mat)]

    # The panels, as (angle round the body, half-width in radians at the
    # yoke, how far below the yoke they hang). Each narrows to 60% at its
    # foot, cut straight across.
    panels = ((-math.pi / 2, 0.3, 0.42), (-math.pi / 2 - 0.92, 0.3, 0.32), (-math.pi / 2 + 0.92, 0.3, 0.32), (math.pi / 2, 0.62, 0.36))

    for index, (centre, half, depth) in enumerate(panels):
        grid = []

        for row in range(9):
            share = row / 8
            z = yoke + 0.02 - (depth + 0.02) * share
            width = half * (1 - 0.4 * share)
            grid.append([ring(centre + width * (2 * column / 8 - 1), z) for column in range(9)])

        parts.append(core.loft(f"cape_panel_{index}", grid, mat, closed=False))

    cape = core.join(parts, "cape")
    cape.parent = group
    cape = core.apply_modifier(cape, "SOLIDIFY", thickness=0.02, offset=1.0)
    finish(core.shade_smooth(cape, angle=40), seed=6)


def _ease(value):
    """A smooth step from 0 to 1."""

    value = max(0.0, min(1.0, value))

    return value * value * (3 - 2 * value)
