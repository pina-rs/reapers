"""The penguin: Pesky Penguins.

A squat upright bird cut in planes: a barrel with a hard shoulder where its
sides turn in to the neck and a flat white belly laid on its front, a head
with a raised blaze up the forehead that carries the brow seal, a hooked
bill whose short lower half drops clear of the bib, flippers that stand
off its sides with their tips splayed out, and chunky three-toed webbed
feet. It waddles, so the two legs are the parts that matter most.

It wears a ribbed knit beanie or a jewelled royal crown.
"""

import math

from mathutils import Vector

from lib import core, creature

HEAD = Vector((0, 0, 0.2))
ARM = Vector((0.27, 0.1, 0.15))


def build(spec):
    root = core.empty("husk")
    body = core.material("body", color=(1.0, 1.0, 1.0), roughness=0.58, pbr="plumage")
    trim = core.material("trim", color=(1.0, 1.0, 1.0), roughness=0.84, pbr="wool")
    detail = core.material("detail", color=(1.0, 1.0, 1.0), roughness=0.42, metallic=0.10, pbr="skin")
    dark = core.material("dark", color=(1.0, 1.0, 1.0), roughness=0.20)
    accent = core.material("accent", color=(1.0, 1.0, 1.0), roughness=0.25)
    floor = creature.floor(spec)

    torso, belly = _torso(root, body, trim)

    # A crisp feathered tail stub that wags as it walks.
    tail = creature.pivot("tail", root, (0, 0.21, -0.28))
    _tail_fan(tail, body)

    head, blaze = _head(root, body, trim, detail, dark, accent)

    for side, name in ((1, "l"), (-1, "r")):
        arm = creature.pivot(f"arm_{name}", root, creature.mirrored(ARM, side))
        _flipper(arm, side, name, body)

        leg = creature.pivot(f"leg_{name}", root, (side * 0.115, 0.0, -0.30))
        _foot(leg, side, name, floor + 0.30, detail)

    # The seals' sockets: the brow's on the blaze, the belly's on the white,
    # the flanks' in front of the flippers and the back's between them.
    creature.seat("seat_0", spec, 0, detail, head, [blaze], radius=0.041, rise=0.008, lobes=7, scallop=0.16)
    creature.seat("seat_1", spec, 1, detail, root, [belly], radius=0.05, rise=0.01)

    for index in (2, 3, 4):
        creature.seat(f"seat_{index}", spec, index, detail, root, [torso], radius=0.046, rise=0.01)

    creature.studs(root, spec)
    creature.shadow(root, spec)

    return root


def _torso(root, body, trim):
    """The barrel and the belly laid on it. The barrel is boxy enough that
    its sides are planes, and it turns in hard at the shoulder, a step the
    light breaks on; the belly is a flat white plane from the bib to the
    feet."""

    torso = creature.shell(
        "torso",
        [
            ((0, 0.0, -0.37), 0.1, 0.09, 0.1, 2.2),
            ((0, 0.0, -0.33), 0.2, 0.18, 0.2, 2.6),
            ((0, 0.0, -0.24), 0.27, 0.225, 0.245, 3.0),
            ((0, 0.0, -0.1), 0.29, 0.235, 0.255, 3.0),
            ((0, 0.0, 0.02), 0.285, 0.225, 0.255, 3.2),
            ((0, 0.0, 0.11), 0.265, 0.205, 0.245, 3.4),
            ((0, 0.0, 0.16), 0.205, 0.165, 0.2, 3.0),
            ((0, 0.0, 0.23), 0.15, 0.13, 0.15, 2.6),
            ((0, 0.0, 0.27), 0.1, 0.09, 0.09, 2.2),
        ],
        body,
        root,
        sides=24,
        start=0.02,
        end=0.02,
        crease=38,
    )

    belly = creature.shell(
        "belly",
        [
            ((0, -0.15, -0.33), 0.07, 0.035, 0.03, 2.2),
            ((0, -0.195, -0.26), 0.16, 0.04, 0.05, 2.8),
            ((0, -0.215, -0.14), 0.205, 0.04, 0.05, 3.6),
            ((0, -0.22, -0.02), 0.195, 0.04, 0.05, 3.6),
            ((0, -0.2, 0.08), 0.16, 0.035, 0.05, 3.2),
            ((0, -0.165, 0.14), 0.1, 0.03, 0.045, 2.6),
        ],
        trim,
        root,
        sides=24,
        start=0.012,
        end=0.02,
        crease=38,
    )

    return torso, belly


def _tail_fan(parent, body):
    rings = []
    for step in range(6):
        t = step / 5
        w = 0.075 * (1.0 - 0.62 * t)
        h = 0.032 * (1.0 - 0.7 * t)
        y = 0.0 + 0.155 * t
        z = 0.01 - 0.048 * t
        rings.append(
            [
                (-w, y, z),
                (-w * 0.55, y, z + h),
                (0.0, y, z + h * 1.15),
                (w * 0.55, y, z + h),
                (w, y, z),
                (w * 0.5, y, z - h * 0.7),
                (-w * 0.5, y, z - h * 0.7),
            ]
        )
    tail = core.loft("tail_fan", rings, body, parent, cap_start=True, cap_end=True)
    creature.finish(core.shade_smooth(tail, angle=50))


def _flipper(parent, side, name, body):
    """A flipper: thick at the shoulder, thinning to its tip, standing a
    finger's width off the barrel's side all the way down and splayed out
    from it toward the tip, so it reads as its own shape."""

    def at(x, y, z):
        return creature.mirrored((x, y, z), side) - creature.mirrored(ARM, side)

    creature.shell(
        f"flipper_{name}",
        [
            (at(0.245, 0.1, 0.16), 0.034, 0.05, 0.05, 2.2),
            (at(0.29, 0.105, 0.115), 0.034, 0.068, 0.058, 2.4),
            (at(0.322, 0.115, 0.02), 0.026, 0.074, 0.058, 2.4),
            (at(0.342, 0.13, -0.08), 0.019, 0.064, 0.048, 2.4),
            (at(0.362, 0.148, -0.16), 0.012, 0.044, 0.034, 2.4),
        ],
        body,
        parent,
        sides=14,
        start=0.0,
        end=0.04,
        crease=50,
        forward=(0, -1, 0),
    )


def _foot(parent, side, name, ground_z, detail):
    """Chunky stylized 3-toed webbed penguin foot planted flat on the floor."""

    blobs = [
        # Ankle column connecting cleanly inside the body.
        ("sphere", (0.0, -0.006, ground_z + 0.068), (0.048, 0.052, 0.068)),
        # Heel pad flat on the ground.
        ("sphere", (0.0, -0.028, ground_z + 0.024), (0.060, 0.064, 0.024)),
    ]
    # Three defined spreading webbed toes + webbing bridge between them.
    for toe_idx, (angle_deg, length) in enumerate(((-22, 0.135), (0, 0.155), (22, 0.135))):
        ang = math.radians(angle_deg)
        tx = math.sin(ang) * length * 0.55
        ty = -0.025 - math.cos(ang) * length * 0.55
        blobs.append(("sphere", (tx, ty, ground_z + 0.021), (0.027, length * 0.48, 0.021), (0, 0, -ang)))
        # Rounded toe tip pad.
        tip_x = math.sin(ang) * length * 0.92
        tip_y = -0.025 - math.cos(ang) * length * 0.92
        blobs.append(("sphere", (tip_x, tip_y, ground_z + 0.019), (0.024, 0.026, 0.019)))

    # Webbing membrane connecting the toes.
    blobs.append(("sphere", (0.0, -0.095, ground_z + 0.013), (0.078, 0.058, 0.012)))

    creature.form(f"foot_{name}", blobs, detail, parent, triangles=520, voxel=0.010)


def _head(root, body, trim, detail, dark, accent):
    head = creature.pivot("head", root, HEAD)

    # A chunky head, boxy enough to take broad planes, its forehead set
    # back a little so the blaze stands proud of it.
    skull = creature.shell(
        "skull",
        [
            ((0, -0.012, -0.02), 0.145, 0.13, 0.14, 2.4),
            ((0, -0.012, 0.03), 0.184, 0.168, 0.175, 2.8),
            ((0, -0.012, 0.09), 0.196, 0.168, 0.18, 3.0),
            ((0, -0.012, 0.145), 0.19, 0.162, 0.176, 3.0),
            ((0, -0.012, 0.185), 0.172, 0.148, 0.166, 2.8),
            ((0, -0.012, 0.222), 0.142, 0.12, 0.142, 2.6),
            ((0, -0.012, 0.25), 0.094, 0.08, 0.094, 2.4),
        ],
        body,
        head,
        sides=24,
        start=0.02,
        end=0.024,
        crease=40,
    )

    # The white of the face round the eyes and over the bill.
    creature.form(
        "face",
        [
            ("sphere", (0.074, -0.146, 0.062), (0.068, 0.038, 0.06), (0, 0, math.radians(-14))),
            ("sphere", (-0.074, -0.146, 0.062), (0.068, 0.038, 0.06), (0, 0, math.radians(14))),
            ("sphere", (0.0, -0.152, 0.026), (0.084, 0.032, 0.036)),
        ],
        trim,
        head,
        triangles=560,
        voxel=0.011,
    )

    # The blaze: a raised stripe of the white from the bill up over the
    # forehead to the crown, the plate the brow seal is set in. A stripe,
    # not a spot, so the seal on it never reads as a third eye.
    blaze = creature.shell(
        "blaze",
        [
            ((0, -0.166, 0.042), 0.026, 0.012, 0.012, 2.4),
            ((0, -0.174, 0.07), 0.036, 0.016, 0.014, 3.0),
            ((0, -0.174, 0.1), 0.05, 0.016, 0.014, 3.2),
            ((0, -0.168, 0.13), 0.048, 0.016, 0.014, 3.2),
            ((0, -0.152, 0.165), 0.04, 0.016, 0.014, 3.0),
            ((0, -0.122, 0.2), 0.03, 0.016, 0.014, 2.8),
            ((0, -0.08, 0.228), 0.018, 0.014, 0.012, 2.4),
        ],
        trim,
        head,
        sides=16,
        start=0.008,
        end=0.012,
        crease=40,
    )

    for side in (1, -1):
        creature.form(f"socket_{side}", [("sphere", (side * 0.074, -0.178, 0.072), (0.036, 0.022, 0.038))], dark, head, triangles=120)
        creature.eye(side, head, (side * 0.074, -0.195, 0.072), (0.017, 0.011, 0.018), accent)
        # Out to either side of the blaze, clear of the seal's socket.
        creature.brow(side, head, (side * 0.094, -0.196, 0.122), (0.046, 0.02, 0.016), dark, rotation=(0, 0, math.radians(side * 12)))

    # The hooked upper bill, and a short lower one hinged high at the back
    # of the mouth, so it drops forward of the bib and not into it.
    _beak(
        "beak_upper",
        Vector((0.0, -0.17, 0.014)),
        length=0.145,
        width=lambda t: 0.058 * (1.0 - t**2.1) + 0.005,
        top=lambda t: 0.042 * (1.0 - t**1.9) - 0.012 * (t**2.6),
        bottom=lambda t: 0.002 - 0.018 * (max(0.0, t - 0.5) / 0.5) ** 2,
        mat=detail,
        parent=head,
    )

    # The inside of the mouth under the upper bill, which shows as a dark
    # wedge when the lower one drops.
    creature.shell(
        "mouth",
        [
            ((0, -0.165, 0.012), 0.05, 0.006, 0.006, 2.4),
            ((0, -0.22, 0.01), 0.04, 0.006, 0.006, 2.4),
            ((0, -0.27, 0.004), 0.02, 0.005, 0.005, 2.2),
        ],
        dark,
        head,
        sides=12,
        start=0.0,
        end=0.008,
        forward=(0, 0, 1),
    )

    jaw = creature.pivot("jaw", head, (0, -0.158, 0.008))
    _beak(
        "beak_lower",
        Vector((0.0, 0.0, 0.0)),
        length=0.088,
        width=lambda t: 0.046 * (1.0 - t**2.2) + 0.004,
        top=lambda t: 0.004 * (1.0 - t),
        bottom=lambda t: -0.02 * (1.0 - t**1.4) - 0.004,
        mat=detail,
        parent=jaw,
        keel=True,
    )
    creature.shell(
        "mouth_floor",
        [
            ((0, -0.006, 0.004), 0.036, 0.004, 0.004, 2.4),
            ((0, -0.05, 0.003), 0.03, 0.004, 0.004, 2.4),
            ((0, -0.075, 0.002), 0.014, 0.003, 0.003, 2.2),
        ],
        dark,
        jaw,
        sides=12,
        start=0.0,
        end=0.006,
        forward=(0, 0, 1),
    )

    _beanie(creature.pivot("trait_1", head, (0, 0, 0)), detail, trim)
    _crown(creature.pivot("trait_2", head, (0, 0, 0)), detail)

    return head, blaze


def _beak(name, origin, length, width, top, bottom, mat, parent, keel=False, steps=9):
    """Lofted penguin bill with sharp culmen ridge and smooth curved cheeks."""

    rings = []
    for step in range(steps + 1):
        t = step / steps
        half, high, low = width(t), top(t), bottom(t)
        flat, ridge = (high, low) if keel else (low, high)

        def pt(across, toward):
            return origin + Vector((half * across, -length * t, flat + (ridge - flat) * toward))

        rings.append([pt(-1.0, 0.0), pt(-0.88, 0.52), pt(-0.48, 0.90), pt(0.0, 1.0), pt(0.48, 0.90), pt(0.88, 0.52), pt(1.0, 0.0)])

    bill = core.loft(name, rings, mat, parent, cap_start=True, cap_end=True)
    return creature.finish(core.shade_smooth(bill, angle=52))


def _beanie(parent, wool, band):
    """Ribbed knit beanie with rolled cuff and textured pom-pom."""

    def rib(angle, radius, height):
        rib_bump = 1.0 + 0.022 * math.cos(angle * 18) * max(0.0, min(1.0, (height - 0.175) / 0.12))
        return radius * rib_bump, height

    dome = core.revolve(
        "beanie",
        [
            (0.188, 0.168),
            (0.194, 0.195),
            (0.184, 0.232),
            (0.156, 0.268),
            (0.110, 0.296),
            (0.050, 0.312),
            (0.0, 0.316),
        ],
        wool,
        parent,
        segments=36,
        shape=rib,
        cap_bottom=True,
    )
    dome.location = (0, -0.012, 0)
    creature.finish(core.shade_smooth(dome))

    cuff = core.revolve(
        "beanie_band",
        [
            (0.180, 0.152),
            (0.204, 0.158),
            (0.212, 0.176),
            (0.211, 0.200),
            (0.202, 0.216),
            (0.182, 0.220),
        ],
        band,
        parent,
        segments=36,
        shape=lambda a, r, h: (r * (1.0 + 0.018 * math.cos(a * 18)), h),
    )
    cuff.location = (0, -0.012, 0)
    creature.finish(core.shade_smooth(cuff))

    pom = core.revolve(
        "beanie_bobble",
        [(0.0, 0.368), (0.028, 0.362), (0.046, 0.344), (0.048, 0.324), (0.032, 0.308), (0.0, 0.304)],
        band,
        parent,
        segments=18,
        shape=lambda a, r, h: (r * (1.0 + 0.08 * math.sin(a * 7 + h * 40)), h),
    )
    pom.location = (0, -0.012, 0)
    creature.finish(core.shade_smooth(pom))


def _crown(parent, gold):
    """Thickened sculpted royal crown with flared spires and beaded circlet."""

    points = 7
    outer_top, outer_bot, inner_top, inner_bot = [], [], [], []
    steps = points * 4
    for step in range(steps):
        angle = step / steps * math.tau
        ca, sa = math.cos(angle), math.sin(angle)
        tooth = abs(math.sin(angle * points / 2)) ** 1.4
        z_top = 0.278 + 0.078 * tooth
        r_bot_x, r_bot_y = 0.140, 0.134
        r_top_x, r_top_y = 0.156, 0.148
        outer_bot.append((ca * r_bot_x, sa * r_bot_y - 0.012, 0.222))
        outer_top.append((ca * r_top_x, sa * r_top_y - 0.012, z_top))
        inner_top.append((ca * (r_top_x - 0.014), sa * (r_top_y - 0.014) - 0.012, z_top - 0.004))
        inner_bot.append((ca * (r_bot_x - 0.014), sa * (r_bot_y - 0.014) - 0.012, 0.222))

    crown = core.loft("crown", [outer_bot, outer_top, inner_top, inner_bot, outer_bot], gold, parent, closed=True)
    creature.finish(core.shade_smooth(crown, angle=42))

    band = core.revolve(
        "crown_band",
        [
            (0.136, 0.214),
            (0.150, 0.218),
            (0.156, 0.228),
            (0.154, 0.244),
            (0.158, 0.252),
            (0.148, 0.260),
            (0.136, 0.260),
        ],
        gold,
        parent,
        segments=32,
        shape=lambda a, r, h: (r + 0.006 * (max(0.0, math.cos(a * points)) ** 8) * (abs(h - 0.236) < 0.012), h),
    )
    band.location = (0, -0.012, 0)
    creature.finish(core.shade_smooth(band, angle=45))
