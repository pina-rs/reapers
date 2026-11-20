"""The penguin: Pesky Penguins.

A squat, stylized upright bird with a heavy sculpted brow, sleek streamlined
flippers, hooked curved bill, and chunky three-toed webbed feet. It waddles,
so the two legs are the parts that matter most.

It wears a ribbed knit beanie or a jewelled royal crown.
"""

import math

import bmesh
from mathutils import Vector

from lib import core, creature


def build(spec):
    root = core.empty("husk")
    body = core.material("body", color=(1.0, 1.0, 1.0), roughness=0.58, pbr="plumage")
    trim = core.material("trim", color=(1.0, 1.0, 1.0), roughness=0.84, pbr="wool")
    detail = core.material("detail", color=(1.0, 1.0, 1.0), roughness=0.42, metallic=0.10, pbr="skin")
    dark = core.material("dark", color=(1.0, 1.0, 1.0), roughness=0.20)
    accent = core.material("accent", color=(1.0, 1.0, 1.0), roughness=0.25)
    floor = creature.floor(spec)

    # Plump, smooth emperor-penguin barrel tapering cleanly to the shoulders
    # and rounding smoothly into the lower stance.
    _torso(root, body, trim)

    # A crisp feathered tail stub that wags as it walks.
    tail = creature.pivot("tail", root, (0, 0.21, -0.28))
    _tail_fan(tail, body)

    _head(root, body, trim, detail, dark, accent)

    for side, name in ((1, "l"), (-1, "r")):
        arm = creature.pivot(f"arm_{name}", root, (side * 0.235, 0.0, 0.13))
        _flipper(arm, side, name, body)

        leg = creature.pivot(f"leg_{name}", root, (side * 0.115, 0.0, -0.30))
        _foot(leg, side, name, floor + 0.30, detail)

    creature.studs(root, spec)
    creature.shadow(root, spec)

    return root


def _torso(root, body, trim):
    """Lofted sleek barrel torso and curved pale belly shield."""

    # (z, half_x, front_y, back_y) stops from lower rump up to shoulder collar.
    stops = [
        (-0.37, 0.10, 0.09, 0.10),
        (-0.33, 0.195, 0.185, 0.20),
        (-0.24, 0.255, 0.235, 0.245),
        (-0.12, 0.272, 0.255, 0.250),
        (0.02, 0.250, 0.238, 0.230),
        (0.14, 0.208, 0.195, 0.195),
        (0.23, 0.165, 0.155, 0.155),
        (0.27, 0.11, 0.105, 0.105),
    ]
    segments = 32
    rings = []
    for z, rx, fy, by in stops:
        ring = []
        for step in range(segments):
            angle = math.tau * step / segments
            ca, sa = math.cos(angle), math.sin(angle)
            ry = by if sa > 0 else fy
            # Subtle proud chest curve in front.
            chest = 0.014 * max(0.0, -sa) ** 2 * max(0.0, 1.0 - abs(z + 0.06) / 0.28)
            ring.append((ca * rx, sa * ry - chest, z))
        rings.append(ring)

    torso = core.loft("torso", rings, body, root, cap_start=True, cap_end=True)
    creature.finish(core.shade_smooth(torso))

    # Crisp arched pale chest/belly panel hugging the front of the barrel.
    belly_stops = [
        (-0.33, 0.072, -0.145),
        (-0.26, 0.156, -0.228),
        (-0.14, 0.188, -0.264),
        (-0.01, 0.175, -0.254),
        (0.11, 0.140, -0.214),
        (0.20, 0.088, -0.168),
    ]
    b_rings = []
    b_segs = 28
    for z, half_w, front_y in belly_stops:
        ring = []
        for step in range(b_segs):
            angle = math.tau * step / b_segs
            ca, sa = math.cos(angle), math.sin(angle)
            x = ca * half_w
            y = front_y + (0.038 * sa if sa < 0 else 0.085 * sa) + 0.056 * (ca * ca)
            ring.append((x, y, z))
        b_rings.append(ring)

    belly = core.loft("belly", b_rings, trim, root, cap_start=True, cap_end=True)
    creature.finish(core.shade_smooth(belly))


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
    """Streamlined hydrofoil penguin flipper curving cleanly down and back."""

    # Stations: (t, x_off, y_off, z_off, chord_y, thick_x)
    stations = [
        (0.0, -0.052, 0.008, 0.024, 0.048, 0.020),
        (0.14, 0.002, 0.004, -0.015, 0.078, 0.028),
        (0.32, 0.034, 0.002, -0.085, 0.088, 0.029),
        (0.56, 0.054, 0.008, -0.175, 0.084, 0.024),
        (0.78, 0.064, 0.020, -0.255, 0.064, 0.017),
        (0.92, 0.066, 0.030, -0.305, 0.038, 0.011),
        (1.0, 0.062, 0.038, -0.330, 0.014, 0.005),
    ]
    rings = []
    segs = 14
    for _, xo, yo, zo, chord, thick in stations:
        ring = []
        for s in range(segs):
            a = math.tau * s / segs
            # Airfoil-ish cross-section: plumper leading edge (-Y), tapered trailing edge (+Y).
            ca = math.cos(a)
            sa = math.sin(a)
            local_y = sa * chord
            foil_taper = 1.0 - 0.35 * max(0.0, sa)
            local_x = ca * thick * foil_taper
            # Slight concave inner cup toward the body.
            cup = -side * 0.008 * (1.0 - sa * sa)
            ring.append((side * xo + local_x + cup, yo + local_y, zo))
        rings.append(ring)

    flipper = core.loft(f"flipper_{name}", rings, body, parent, cap_start=True, cap_end=True)
    creature.finish(core.shade_smooth(flipper, angle=55))


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
    head = creature.pivot("head", root, (0, 0, 0.2))

    # Clean stylized rounded-bevel penguin head with sculpted brow overhang
    # and cheek contour (no raw protruding cube corners!).
    skull_stops = [
        (-0.02, 0.145, 0.138, 0.140, 0.0),
        (0.03, 0.184, 0.178, 0.174, 0.0),
        (0.09, 0.194, 0.188, 0.180, 0.008),
        (0.145, 0.188, 0.188, 0.175, 0.024),
        (0.182, 0.174, 0.184, 0.166, 0.020),
        (0.218, 0.146, 0.148, 0.145, 0.004),
        (0.248, 0.098, 0.098, 0.098, 0.0),
        (0.264, 0.038, 0.038, 0.038, 0.0),
    ]
    segs = 32
    rings = []
    for z, rx, fy, by, brow in skull_stops:
        ring = []
        for s in range(segs):
            a = math.tau * s / segs
            ca, sa = math.cos(a), math.sin(a)
            # Soft squircle profile keeps the chunky Pesky silhouette without harsh cube seams.
            sx = math.copysign(abs(ca) ** 0.88, ca) * rx
            ry = by if sa > 0 else fy
            sy = math.copysign(abs(sa) ** 0.88, sa) * ry
            if sa < 0:
                # Sculpted brow ledge projecting over the eyes.
                front = -sa
                sy -= brow * (front**1.6) * (0.75 + 0.25 * math.cos(a * 2))
            ring.append((sx, sy - 0.012, z))
        rings.append(ring)

    skull = core.loft("skull", rings, body, head, cap_start=True, cap_end=True)
    creature.finish(core.shade_smooth(skull, angle=58))

    # Crisp sculpted white facial cheek masks framing the eyes and beak,
    # seated cleanly behind the dark eye sockets so eyes pop crisply.
    creature.form(
        "face",
        [
            ("sphere", (0.072, -0.144, 0.092), (0.068, 0.038, 0.064), (0, 0, math.radians(-14))),
            ("sphere", (-0.072, -0.144, 0.092), (0.068, 0.038, 0.064), (0, 0, math.radians(14))),
            ("sphere", (0.0, -0.154, 0.052), (0.084, 0.032, 0.038)),
        ],
        trim,
        head,
        triangles=620,
        voxel=0.011,
    )

    # Dark recessed eye sockets, glowing eyes, and sculpted expressive brow ledges.
    for side in (1, -1):
        creature.form(f"socket_{side}", [("sphere", (side * 0.074, -0.178, 0.104), (0.036, 0.022, 0.038))], dark, head, triangles=120)
        creature.eye(side, head, (side * 0.074, -0.195, 0.104), (0.017, 0.011, 0.018), accent)
        creature.brow(side, head, (side * 0.074, -0.196, 0.152), (0.050, 0.018, 0.014), body, rotation=(0, math.radians(side * -7), math.radians(side * 12)))

    # Sculpted curved penguin beak (upper ridge + hooked tip, lower hinged bill).
    _beak(
        "beak_upper",
        Vector((0.0, -0.168, 0.040)),
        length=0.145,
        width=lambda t: 0.058 * (1.0 - t**2.1) + 0.005,
        top=lambda t: 0.042 * (1.0 - t**1.9) - 0.012 * (t**2.6),
        bottom=lambda t: 0.002 - 0.018 * (max(0.0, t - 0.5) / 0.5) ** 2,
        mat=detail,
        parent=head,
    )

    jaw = creature.pivot("jaw", head, (0, -0.165, 0.028))
    _beak(
        "beak_lower",
        Vector((0.0, 0.0, 0.0)),
        length=0.110,
        width=lambda t: 0.046 * (1.0 - t**2.2) + 0.004,
        top=lambda t: 0.004 * (1.0 - t),
        bottom=lambda t: -0.022 * (1.0 - t**1.4) - 0.004,
        mat=detail,
        parent=jaw,
        keel=True,
    )

    _beanie(creature.pivot("trait_1", head, (0, 0, 0)), detail, trim)
    _crown(creature.pivot("trait_2", head, (0, 0, 0)), detail)


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
