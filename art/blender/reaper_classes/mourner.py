"""The mourner: the voice that announces a death before it arrives.

Silhouette: a floor-length veil that makes the whole figure one tall
column with no arms, and a handbell held out from one side of it. The bell
is the only shape that breaks the column (docs/art-bible.md 6.2): every
head carries the veil on up, every hem reaches the floor, and everything a
mourner wears lies on the veil's one slope. At rest the bell is tipped up
toward the viewer, so its mouth, and the clapper's light inside it, faces
them.

Three families show: the veil (`cloth`), what lies over it (`cloth_alt`)
and the bones of the hands (`bone`). The bell is the one asymmetry; the
second arm stays inside the veil until a gesture brings it out.
"""

import math
import random

from mathutils import Matrix, Vector, noise

from lib import anatomy, cloth, core
from lib.figure import HEAD_HEIGHT, HEAD_RADIUS, NECK_HEIGHT, SHOULDER_HEIGHT, finish

# The veil as (height, half-width, half-depth): it starts at the crown of
# the head, clings in under the jaw, eases out over the shoulders and then
# falls to the ground in one straight slope. Everything else a mourner wears
# is placed by how far outside that surface it lies.
CROWN = (1.97, 0.09, 0.09)
NECK = (NECK_HEIGHT + 0.02, 0.115, 0.11)
SHOULDER = (SHOULDER_HEIGHT - 0.02, 0.18, 0.135)
GROUND = (0.06, 0.34, 0.31)

# The widest part of the head, and how far the skull rises above it.
BROW = HEAD_HEIGHT + 0.01
SKULL = 0.14

# The layers, so that any head can be worn with any mantle. A mantle lies
# `MANTLE_LAYER` outside the veil. The cloth of a head stands `HEAD_LAYER`
# outside it at the brow and comes down to `FLUSH` at the shoulders, where
# the slope changes and its hem shows least, so a mantle comes out from
# under it. Nothing lies over the cloth of a head.
MANTLE_LAYER = 0.005
FLUSH = 0.008
HEAD_LAYER = 0.027

# Straight ahead, as an angle around the figure.
FRONT = -math.pi / 2

# The weapon arm in the idle pose's first key (lib/motion.py, at the
# mourner's amplitude of 0.75 there), and how far the bell is tipped from
# hanging mouth down toward the viewer at rest.
IDLE_SHOULDER_R = math.radians(8 * 0.75)
IDLE_ELBOW_R = math.radians(-24 * 0.75)
BELL_TIP = math.radians(-38)


def build():
    """The voice that announces a death: one tall column and a bell."""

    root = core.empty("reaper")
    cloth_mat = core.material("cloth")
    alt = core.material("cloth_alt")

    # Every hem reaches the floor, so the column stands on it whole.
    hems = [
        # Pooling, and barely torn.
        dict(hem=0.03, tatter=0.07, tears=4, slits=2, fold_depth=0.17, seed=31),
        # Worn into long strips from the knee down.
        dict(hem=0.07, tatter=0.34, tears=6, slits=18, fold_depth=0.14, seed=32),
        # Long trailing points.
        dict(hem=0.05, tatter=0.46, tears=10, slits=4, fold_depth=0.15, seed=33),
        # Leaning, as if in a slow wind.
        dict(hem=0.07, tatter=0.2, tears=6, slits=7, fold_depth=0.16, seed=34),
    ]

    for index, hem in enumerate(hems):
        robe = cloth.robe(
            f"robe_{index}",
            cloth_mat,
            root,
            top=CROWN[0],
            profile=_slope(CROWN[0], hem["hem"]),
            folds=(5, 8, 13),
            rings=46,
            segments=120,
            **hem,
        )

        _reshape(robe, _furrow(index))

        if index == 0:
            _reshape(robe, _pool)

        if index == 3:
            _reshape(robe, _lean)

        finish(_still(robe), seed=index)

    _arms(root)

    head = core.empty("head", root, location=(0, 0, NECK_HEIGHT))
    _heads(head, cloth_mat)
    _mantles(root, alt)

    return root


def _veil(height, layer=0.0):
    """The veil's half-width and half-depth at `height`, `layer` outside it."""

    if height <= SHOULDER[0]:
        upper, lower = SHOULDER, GROUND
        blend = (upper[0] - height) / (upper[0] - lower[0])
    else:
        upper, lower = (CROWN, NECK) if height > NECK[0] else (NECK, SHOULDER)
        blend = max(0.0, min(1.0, (upper[0] - height) / (upper[0] - lower[0])))
        blend = blend * blend * (3 - 2 * blend)

    return tuple(top + (bottom - top) * blend + layer for top, bottom in zip(upper[1:], lower[1:]))


def _on_veil(angle, height, layer=0.0):
    """The point `layer` outside the veil at `angle` around it."""

    width, thickness = _veil(height, layer)

    return Vector((math.cos(angle) * width, math.sin(angle) * thickness, height))


def _slope(top, hem, layer=0.0, spread=0.0):
    """A `cloth.robe` profile that follows the veil from `top` to `hem`,
    `layer` outside it and a further `spread` outside it at the hem.

    `cloth.robe` eases between its stops, which suits the neck and
    shoulders and would bend the long fall below them into an S. The last
    two stops sit a whole fall's length outside the robe, so that the fall
    uses only the straight middle third of that curve.
    """

    length = top - hem
    stops = [((top - height) / length, width + layer, depth + layer) for height, width, depth in (CROWN, NECK, SHOULDER)]
    shoulder = stops[-1][0]
    fall = 1 - shoulder
    high = _veil(SHOULDER[0], layer)
    low = _veil(hem, layer + spread)
    # The easing turns the two ends of its middle third into 7/27 and 20/27.
    rise = [(bottom - start) * 27 / 13 for start, bottom in zip(high, low)]
    before = [start - step * 7 / 27 for start, step in zip(high, rise)]
    after = [start + step for start, step in zip(before, rise)]

    return (*stops, (shoulder - fall, *before), (1 + fall, *after))


def _still(robe):
    """Leaves every motion key of `robe` at rest.

    Blender adds a shape key switched fully on, which would export and
    preview the veil with all of its motion applied at once.
    """

    for key in robe.data.shape_keys.key_blocks[1:]:
        key.value = 0.0

    return robe


def _reshape(robe, shift):
    """Moves every vertex of `robe` by `shift(position)`.

    Every motion key is moved with the basis, so the keys still mean what
    `cloth.robe` made them mean.
    """

    keys = robe.data.shape_keys.key_blocks

    for index, vertex in enumerate(robe.data.vertices):
        move = shift(vertex.co)

        vertex.co += move

        for key in keys:
            key.data[index].co += move


def _pool(position):
    """Spreads the foot of a robe as cloth does where it meets the ground."""

    low = max(0.0, 1 - position.z / 0.36) ** 2

    return Vector((position.x * 0.26 * low, position.y * 0.26 * low, 0.03 * low))


def _lean(position):
    """Carries the foot of a robe back and away from the bell, as a slow
    wind would.

    `cloth.robe` can lean a robe too, but from the shoulders down, which
    pushes it through whatever lies on it. This starts below the mantles.
    """

    blown = max(0.0, 1 - position.z) ** 2

    return Vector((0.1 * blown, 0.28 * blown, 0.0))


def _furrow(seed):
    """Long folds that start under the shoulders and run the length of a
    robe, which `cloth.robe` only folds toward its hem.

    A furrow only ever draws the cloth inward, so nothing that lies on the
    veil is left inside it.
    """

    def shift(position):
        angle = math.atan2(position.y, position.x)
        fallen = max(0.0, min(1.0, (SHOULDER[0] - position.z) / 0.45))
        valley = (0.5 + 0.5 * math.sin(angle * 7 + 1.1 * math.sin(angle * 3 + seed) + seed * 2.3)) ** 2
        draw = 0.085 * fallen * fallen * (3 - 2 * fallen) * valley

        return Vector((-position.x * draw, -position.y * draw, 0.0))

    return shift


def _drape(name, mat, surface, rows, segments, uneven=0.0, seed=0):
    """Cloth lofted over `surface(depth, angle)`, closed at the top, with
    `depth` running from 0 at the top to 1 at the lower edge.

    `uneven` is the share of its length by which the lower edge wanders, as
    a hem does that was never cut straight.
    """

    generator = random.Random(seed)
    phases = [generator.uniform(0, math.tau) for _ in range(2)]
    rings = []

    for row in range(rows + 1):
        ring = []

        for segment in range(segments):
            angle = math.tau * segment / segments
            short = uneven * (0.5 + 0.3 * math.sin(angle * 2 + phases[0]) + 0.2 * math.sin(angle * 5 + phases[1]))
            ring.append(surface(row / rows * (1 - short), angle))

        rings.append(ring)

    return core.loft(name, rings, mat, cap_start=True)


def _aside(angle):
    """How far around from straight ahead `angle` is, in radians."""

    return (angle - FRONT + math.pi) % math.tau - math.pi


def _face(height, aside):
    """How far the cloth is pushed forward by the face under it, `aside`
    radians around from straight ahead.

    A brow, the ridge of a nose and a shallow hollow either side of it.
    Below the nose the cloth hangs free, so the push fades slowly on the way
    down. This is all the face a mourner has.
    """

    nose = BROW - 0.035

    if height > nose:
        rise = max(0.0, 1 - (height - nose) / 0.11)
        hang = rise * rise * (3 - 2 * rise)
    else:
        hang = max(0.0, (height - SHOULDER[0]) / (nose - SHOULDER[0])) ** 0.7

    plane = 0.012 * max(0.0, math.cos(aside)) ** 4
    ridge = 0.026 * math.exp(-((aside / 0.2) ** 2))
    hollow = 0.008 * math.exp(-(((abs(aside) - 0.4) / 0.17) ** 2) - ((height - BROW + 0.012) / 0.03) ** 2)

    return (plane + ridge) * hang - hollow


def _pleats(angle, share):
    """Folds in cloth that hangs from a head, as a distance to stand out.

    `share` runs from 1 at the brow to 0 where the cloth comes to rest. The
    folds are pressed flat at both, and where the cloth is drawn across the
    face.
    """

    drawn = max(0.0, math.cos(_aside(angle))) ** 2

    return 0.008 * math.sin(share * math.pi) * (1 - drawn) * (0.5 + 0.5 * math.sin(angle * 9 + 1.3 * math.sin(angle * 2)))


def _hood(height, angle):
    """Cloth that clings from the brow down to the shoulders: `HEAD_LAYER`
    outside the veil at the brow, running flush into it at the shoulders."""

    share = max(0.0, min(1.0, (height - SHOULDER[0]) / (BROW - SHOULDER[0])))
    point = _on_veil(angle, height, FLUSH + (HEAD_LAYER - FLUSH) * share + _pleats(angle, share))
    point.y -= _face(height, _aside(angle))

    return point


def _hung(height, extra=0.0):
    """The half-width and half-depth of cloth that hangs straight from the
    brow to the shoulders, as it does from a frame, and lies on the veil
    below them."""

    share = max(0.0, min(1.0, (height - SHOULDER[0]) / (BROW - SHOULDER[0])))

    return tuple(low + (high - low) * share + extra for low, high in zip(_veil(min(height, SHOULDER[0]), FLUSH), _veil(BROW, HEAD_LAYER)))


def _heads(head, mat):
    """Three things the veil hangs from, each carrying the column on above
    the head. The face under each is never more than a shape in the cloth."""

    offset = Vector(head.location)

    def place(obj, group):
        obj.parent = group
        obj.location = Vector(obj.location) - offset

        return obj

    def void(index):
        """The darkness behind the veil, for when the cloth has worn thin."""

        profile = [(HEAD_RADIUS * 0.9 * math.sin(math.pi * step / 8), HEAD_HEIGHT + HEAD_RADIUS * math.cos(math.pi * step / 8)) for step in range(9)]

        return core.revolve(f"void_{index}", profile, core.material("void"), segments=16)

    def hood(depth, dome, angle):
        """Below the brow every head is the same: cloth that clings down to
        the shoulders. `dome` is the share of the cloth's length above it."""

        return _hood(BROW + (SHOULDER[0] - BROW) * (depth - dome) / (1 - dome), angle)

    # A plain head: the veil lies straight over the skull, held by a band.
    # The skull takes the first share of the cloth's length.
    dome = 0.42

    def plain(depth, angle):
        if depth >= dome:
            return hood(depth, dome, angle)

        turn = depth / dome * math.pi / 2
        height = BROW + SKULL * math.cos(turn)
        width, thickness = _veil(BROW, HEAD_LAYER)

        return (
            math.cos(angle) * width * math.sin(turn),
            math.sin(angle) * thickness * math.sin(turn) - _face(height, _aside(angle)),
            height,
        )

    group = core.empty("head_0", head)
    place(finish(_drape("veil_0", mat, plain, rows=30, segments=112, uneven=0.05, seed=41), seed=4), group)

    # A band of the mantle's cloth round the head, low on the brow in front
    # and high behind.
    band = []

    for step in range(65):
        angle = math.tau * step / 64
        front = max(0.0, math.cos(_aside(angle)))
        height = BROW + 0.044 + 0.03 * math.sin(angle)
        ring = math.sqrt(max(0.05, 1 - ((height - BROW) / SKULL) ** 2))
        width, thickness = _veil(BROW, HEAD_LAYER + 0.005)
        push = _face(height, 0.0) * (front**2)

        band.append((math.cos(angle) * width * ring, math.sin(angle) * thickness * ring - push, height))

    place(
        finish(core.tube("circlet_0", band, 0.016, core.material("cloth_alt"), sides=8, cap=False, flatten=0.32)),
        group,
    )
    place(void(0), group)

    # A mantilla on a tall comb: the veil carried up over a comb wider than
    # the head and as tall again as it, and hung from the comb's edges
    # straight down to the shoulders. The face is no more than a shape in
    # the cloth hanging in front of it. From the front the head is an arch
    # on the column, with no neck, so the column runs unbroken from the
    # floor to the top of the comb; the comb's three lobes press through
    # the cloth at the top, which makes the arch a trefoil. From the side
    # the crest is a thin blade leaning back.
    crest = 0.4
    top = BROW + crest
    arch = 0.165
    spring = top - arch
    shoulder = _veil(SHOULDER[0], FLUSH)[0]

    def mantilla(depth, angle):
        height = top - (top - SHOULDER[0]) * depth**1.25

        if height >= spring:
            width = math.sqrt(max(0.0, arch**2 - (height - spring) ** 2))
        else:
            width = arch + (shoulder - arch) * ((spring - height) / (spring - SHOULDER[0])) ** 2

        lean = 0.0

        if height > spring:
            # The comb's lobes: one in the middle and one either side.
            across = math.cos(angle) * width / arch
            height -= 0.05 * (height - spring) / arch * (1 - math.cos(3 * math.pi * across)) / 2

        if height <= BROW:
            share = (height - SHOULDER[0]) / (BROW - SHOULDER[0])
            thickness = _hung(height, _pleats(angle, share))[1]
        else:
            rise = (height - BROW) / crest
            thickness = _hung(BROW)[1] * math.sqrt(max(0.0, 1 - rise**2))
            lean = 0.04 * rise**2

        return (math.cos(angle) * width, math.sin(angle) * thickness + lean - _face(height, _aside(angle)), height)

    group = core.empty("head_1", head)
    place(finish(_drape("veil_1", mat, mantilla, rows=36, segments=104, uneven=0.04, seed=42), seed=5), group)
    place(void(1), group)

    # A veil thrown over a crown: a tall band with five short points round
    # the front, the middle one the tallest. The cloth is lifted at each
    # point and sags between, so the column ends in a coronet; the points
    # are kept short and many, so that none reads as an ear.
    points = tuple((FRONT + step * 0.62, 0.13 - 0.022 * abs(step)) for step in (-2, -1, 0, 1, 2))
    band = 0.1
    tall = 0.5

    def crowned(depth, angle):
        if depth >= tall:
            return hood(depth, tall, angle)

        width, thickness = _veil(BROW, HEAD_LAYER)
        peak = max(lift * max(0.0, math.cos((angle - point + math.pi) % math.tau - math.pi)) ** 40 for point, lift in points)
        rim = BROW + band + peak
        share = depth / tall

        if share < 0.55:
            # Over the top of the crown, from its sagging middle out to the
            # rim, rising steeply to each point.
            out = share / 0.55
            height = BROW + band - 0.012 + (rim - BROW - band + 0.012) * out**3
            reach = 0.94 * out
        else:
            # Down the band of the crown from the rim to the brow.
            down = (share - 0.55) / 0.45
            height = rim + (BROW - rim) * down
            reach = 0.94 + 0.06 * down

        return (
            math.cos(angle) * width * reach,
            math.sin(angle) * thickness * reach - _face(height, _aside(angle)),
            height,
        )

    group = core.empty("head_2", head)
    place(finish(_drape("veil_2", mat, crowned, rows=36, segments=120, uneven=0.04, seed=43), seed=6), group)
    place(void(2), group)


def _arms(root):
    """Both shoulder pivots, and the one hand that comes out from under the
    veil to carry the bell."""

    reach = Vector((-0.45, -0.08, 1.06))
    shoulder = Vector((-0.17, 0.0, SHOULDER[0]))
    pivot = core.empty("arm_r", root, location=shoulder)
    elbow_at = Vector((-0.12, 0.03, 1.13))
    elbow = core.empty("elbow_r", pivot, location=elbow_at - shoulder)
    finish(anatomy.forearm("arm_r_upper", pivot, elbow_at - shoulder, Vector()), wear_boost=1.6)
    wrist_pivot = core.empty("wrist_r", elbow, location=reach - elbow_at)

    # The bell is tipped up toward the viewer, turned back by the idle
    # pose's first key so that it is at rest, and the hand turns with it.
    socket = core.empty("socket_weapon", wrist_pivot)
    socket.rotation_mode = "XYZ"
    socket.rotation_euler = (Matrix.Rotation(-IDLE_ELBOW_R, 3, "X") @ Matrix.Rotation(-IDLE_SHOULDER_R, 3, "Y") @ Matrix.Rotation(BELL_TIP, 3, "X")).to_euler()
    turn = math.pi / 2 + 0.2

    # Thumb up and the wrist turned in toward the body, so the bell hangs
    # below the fist and the arm stays under the cloth.
    hand = anatomy.hand("arm_r_hand", socket, curl=1.0, side=1, scale=1.45)
    hand.rotation_euler = (math.pi, 0, turn)
    finish(hand, wear_boost=1.6)

    # The forearm is hidden until the veil swings away from it.
    wrist = reach + Vector((math.sin(turn), -math.cos(turn), 0)) * 0.18
    finish(anatomy.forearm("arm_r_forearm", elbow, wrist - elbow_at, Vector()), wear_boost=1.6)

    # The second hand rests against the breast inside the veil, and comes
    # out only for the poses that need two hands. It is kept near the middle
    # of the figure, where the idle pose's outward swing cannot carry it
    # through the cloth.
    left_shoulder = Vector((0.15, 0, SHOULDER[0]))
    left_elbow = Vector((0.11, 0.03, 1.28))
    left_grip = Vector((0.04, -0.02, 1.17))
    left = core.empty("arm_l", root, location=left_shoulder)
    left_joint = core.empty("elbow_l", left, location=left_elbow - left_shoulder)
    finish(anatomy.forearm("arm_l_upper", left, left_elbow - left_shoulder, Vector()), wear_boost=1.6)
    left_wrist = core.empty("wrist_l", left_joint, location=left_grip - left_elbow)
    finish(anatomy.forearm("arm_l_forearm", left_joint, left_grip - left_elbow, Vector()), wear_boost=1.6)
    hand = anatomy.hand("arm_l_hand", left_wrist, curl=0.6, side=-1, scale=1.45)
    hand.rotation_euler = (math.radians(100), 0, math.radians(-30))
    finish(hand, wear_boost=1.6)


def _mantles(root, alt):
    """What lies over a mourner's veil. Each lies flat on its slope and is
    the same on both sides: the bell is the figure's one asymmetry."""

    # A second, shorter veil over the first, ragged at its edge.
    top = CROWN[0] - 0.04
    over = cloth.robe(
        "veil_over",
        alt,
        None,
        top=top,
        hem=0.94,
        profile=_slope(top, 0.94, MANTLE_LAYER, spread=0.03),
        folds=(4, 7, 12),
        fold_depth=0.09,
        tatter=0.34,
        tears=4,
        slits=14,
        seed=51,
        rings=26,
        segments=112,
    )
    over.shape_key_clear()
    group = core.empty("mantle_0", root)
    over.parent = group
    finish(over, seed=7)

    # A stole: two long bands that come out from under the cloth of the
    # head at the shoulders and hang straight down the front of the veil,
    # either side of the middle, each ending in a point below the knee.
    group = core.empty("mantle_1", root)
    rows = []
    start, end = SHOULDER[0] + 0.01, 0.6

    for row in range(41):
        fallen = row / 40
        height = start + (end - start) * fallen
        # Under the cloth of the head at the top, a little proud of the
        # veil below it, so that it clears every fold.
        layer = MANTLE_LAYER - 0.001 + 0.006 * min(1.0, fallen * 6)
        width, _ = _veil(height, layer)
        point = min(1.0, (1 - fallen) / 0.09)
        half = (0.05 + 0.012 * fallen) * point / width
        row_points = []

        for side in (-1, 1):
            middle = FRONT + side * 0.46

            for column in range(7):
                across = column / 6 * 2 - 1
                row_points.append(_on_veil(middle + side * across * half, height, layer + 0.002 * (1 - across * across)))

        rows.append(row_points)

    # The two bands are skinned as one sheet with a cut between them.
    bands = [core.loft(f"stole_{side}", [row[7 * side : 7 * side + 7] for row in rows], alt, closed=False) for side in (0, 1)]
    stole = core.join(bands, "stole")
    stole = core.apply_modifier(stole, "SOLIDIFY", thickness=0.004, offset=0.0)
    stole.parent = group
    finish(stole, seed=10)

    # A shawl wrapped round the shoulders from behind, its two ends crossed
    # and knotted on the breast.
    group = core.empty("mantle_2", root)
    gather = FRONT
    knot = 1.3

    def shawl(across, angle):
        near = 0.5 + 0.5 * math.cos(angle - gather)
        middle = 1.44 - (1.44 - knot) * near**1.5
        half = 0.17 - 0.13 * near**4
        fringe = 1 + 0.08 * noise.noise(Vector((math.cos(angle) * 3.0, math.sin(angle) * 3.0, 7.0))) * across
        height = middle + half * (1 - 2 * across * fringe)
        # The folds run around the body and crowd together at the knot.
        fold = 0.5 + 0.5 * math.sin(across * math.pi * 5 + angle * 2 + 0.6)
        layer = MANTLE_LAYER + 0.004 + 0.012 * fold * (0.35 + 0.65 * near) + 0.02 * near**8

        return _on_veil(angle, height, layer)

    rings = [[shawl(row / 14, math.tau * segment / 96) for segment in range(96)] for row in range(15)]
    finish(core.loft("shawl", rings, alt, group), seed=8)

    finish(
        core.sculpt(
            "shawl_knot",
            [
                ("sphere", _on_veil(gather, knot, 0.035), (0.036, 0.04, 0.042)),
                ("sphere", _on_veil(gather + 0.08, knot + 0.025, 0.035), (0.026, 0.026, 0.03)),
                ("sphere", _on_veil(gather - 0.08, knot + 0.025, 0.035), (0.026, 0.026, 0.03)),
            ],
            alt,
            group,
            voxel=0.008,
        ),
        seed=9,
    )

    for tail, (lean, seed) in enumerate(((-1, 3.0), (1, 5.0))):
        rows = []
        end = 0.76

        for row in range(19):
            fallen = row / 18
            line = []

            for column in range(9):
                side = column / 4 - 1
                ragged = 1 - 0.22 * abs(noise.noise(Vector((side * 2.3, seed, tail))))
                reach = fallen * ragged
                height = knot - 0.02 - (knot - 0.02 - end) * reach
                angle = gather + lean * (0.05 + 0.13 * reach) + side * (0.035 + 0.16 * reach)
                pleat = 0.5 + 0.5 * math.sin(side * math.pi * 2.5 + seed)

                line.append(_on_veil(angle, height, 0.03 - 0.008 * reach + 0.016 * pleat * (0.4 + 0.6 * reach)))

            rows.append(line)

        finish(core.loft(f"shawl_tail_{tail}", rows, alt, group, closed=False), seed=tail)
