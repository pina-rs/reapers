"""The ape: Degenerate Ape Academy.

A heavy knuckle-walker with sculpted gorilla musculature: a broad trapezius
shoulder hump above a deep chest, muscular forelimbs planted on articulated
knuckle fists, and a wide crested head hung forward with heavy brow ridges,
sculpted cheekbones, and curved canine fangs inside a dark mouth.

It wears a paneled baseball cap or a solidified royal crown.
"""

import math

from mathutils import Vector

from lib import core, creature

# The joints, in the creature's own space.
HEAD = (0, -0.08, 0.3)
JAW = (0, -0.33, 0.145)
SHOULDER = (0.3, -0.05, 0.16)
HIP = (0.19, 0.4, -0.14)

BODY_VOXEL = 0.025
SKULL_VOXEL = 0.017


def build(spec):
    root = core.empty("husk")
    body = core.material("body", color=(1.0, 1.0, 1.0), roughness=0.88, pbr="fur")
    trim = core.material("trim", color=(1.0, 1.0, 1.0), roughness=0.62, pbr="skin")
    floor = creature.floor(spec)

    # Sculpted gorilla torso: broad trapezius hump, muscular shoulder blades,
    # deep ribcage tapering along a powerful sloping spine to compact haunches.
    creature.form(
        "torso",
        [
            ("sphere", (0, -0.05, 0.07), (0.33, 0.28, 0.29)),
            ("sphere", (0, 0.03, 0.33), (0.31, 0.25, 0.21)),
            # Trapezius collar & muscular upper back shoulders.
            ("sphere", (0.18, 0.02, 0.28), (0.18, 0.17, 0.16)),
            ("sphere", (-0.18, 0.02, 0.28), (0.18, 0.17, 0.16)),
            ("sphere", (0, 0.20, 0.10), (0.27, 0.22, 0.41), (math.radians(35), 0, 0)),
            ("sphere", (0, 0.41, -0.11), (0.21, 0.185, 0.175)),
            # Lower muscular flanks.
            ("sphere", (0.12, 0.34, -0.06), (0.13, 0.15, 0.14)),
            ("sphere", (-0.12, 0.34, -0.06), (0.13, 0.15, 0.14)),
        ],
        body,
        root,
        triangles=2150,
        voxel=BODY_VOXEL,
    )

    # Sculpted bare chest with subtle pectoral plates and rounded belly.
    creature.form(
        "chest",
        [
            ("sphere", (0, -0.11, 0.01), (0.24, 0.25, 0.22)),
            ("sphere", (0.075, -0.15, 0.09), (0.11, 0.13, 0.095)),
            ("sphere", (-0.075, -0.15, 0.09), (0.11, 0.13, 0.095)),
        ],
        trim,
        root,
        triangles=650,
        voxel=0.018,
    )

    _head(root, body, trim)

    for side, name in ((1, "l"), (-1, "r")):
        _arm(root, side, name, body, trim, floor)
        _leg(root, side, name, body, trim, floor)

    creature.studs(root, spec)
    creature.shadow(root, spec)

    return root


def _hang(blobs, joint):
    """Blobs drawn where they sit on the creature, moved into `joint`'s space."""

    return [(kind, _from(joint, at), *rest) for kind, at, *rest in blobs]


def _from(joint, at):
    return tuple(value - origin for value, origin in zip(at, joint))


def _mirror(joint, side):
    return (joint[0] * side, joint[1], joint[2])


def _head(root, body, trim):
    detail = core.material("detail", color=(1.0, 1.0, 1.0), roughness=0.40, metallic=0.10, pbr="bone")
    dark = core.material("dark", color=(1.0, 1.0, 1.0), roughness=0.28)
    accent = core.material("accent", color=(1.0, 1.0, 1.0), roughness=0.22)
    head = creature.pivot("head", root, HEAD)

    # Sculpted gorilla cranium with pronounced sagittal crest, flared zygomatic
    # cheeks, and heavy supraorbital brow ridge.
    creature.form(
        "skull",
        _hang(
            [
                ("sphere", (0, -0.13, 0.37), (0.205, 0.165, 0.152)),
                ("sphere", (0, -0.09, 0.46), (0.125, 0.148, 0.088)),
                ("sphere", (0, -0.15, 0.26), (0.185, 0.132, 0.098)),
                # Zygomatic cheek fur flares.
                ("sphere", (0.155, -0.18, 0.285), (0.068, 0.075, 0.062)),
                ("sphere", (-0.155, -0.18, 0.285), (0.068, 0.075, 0.062)),
                # Heavy sculpted brow bar.
                ("sphere", (0, -0.248, 0.388), (0.175, 0.062, 0.036)),
                ("sphere", (0.092, -0.252, 0.394), (0.092, 0.056, 0.042)),
                ("sphere", (-0.092, -0.252, 0.394), (0.092, 0.056, 0.042)),
            ],
            HEAD,
        ),
        body,
        head,
        triangles=1500,
        voxel=SKULL_VOXEL,
    )

    # Sculpted bare gorilla face: orbital mask, broad nose bridge with flared
    # nostril wings, muscular upper lip, and jowls framing the mouth.
    creature.form(
        "face",
        _hang(
            [
                ("sphere", (0, -0.22, 0.332), (0.162, 0.074, 0.064)),
                ("sphere", (0, -0.30, 0.278), (0.162, 0.128, 0.054)),
                # Sculpted nasal ridge and flared nostril wings.
                ("sphere", (0, -0.365, 0.308), (0.058, 0.052, 0.030)),
                ("sphere", (0.048, -0.398, 0.296), (0.028, 0.028, 0.024)),
                ("sphere", (-0.048, -0.398, 0.296), (0.028, 0.028, 0.024)),
                # Rounded sculpted upper lip seated smoothly into the jowls.
                ("sphere", (0, -0.400, 0.246), (0.116, 0.046, 0.028)),
                ("sphere", (0.098, -0.320, 0.245), (0.046, 0.068, 0.056)),
                ("sphere", (-0.098, -0.320, 0.245), (0.046, 0.068, 0.056)),
            ],
            HEAD,
        ),
        trim,
        head,
        triangles=950,
        voxel=0.013,
    )

    for side in (1, -1):
        creature.form(f"socket_{side}", _hang([("sphere", (side * 0.08, -0.286, 0.342), (0.052, 0.022, 0.036))], HEAD), dark, head, triangles=80)
        creature.eye(side, head, _from(HEAD, (side * 0.08, -0.304, 0.342)), (0.019, 0.010, 0.019), accent, triangles=48)
        creature.brow(side, head, _from(HEAD, (side * 0.083, -0.302, 0.392)), (0.058, 0.021, 0.016), body, triangles=90)
        creature.form(f"nostril_{side}", _hang([("sphere", (side * 0.034, -0.418, 0.294), (0.018, 0.014, 0.013))], HEAD), dark, head, triangles=40)
        # Sculpted rounded ape ear with slight helix flare.
        creature.form(
            f"ear_{side}",
            _hang(
                [
                    ("sphere", (side * 0.208, -0.12, 0.352), (0.032, 0.034, 0.054)),
                    ("sphere", (side * 0.218, -0.105, 0.365), (0.018, 0.022, 0.032)),
                ],
                HEAD,
            ),
            trim,
            head,
            triangles=90,
            voxel=0.008,
        )

    creature.form("mouth", _hang([("sphere", (0, -0.30, 0.19), (0.125, 0.115, 0.055))], HEAD), dark, head, triangles=110)

    # Sculpted curved upper dental arcade: four beveled incisors flanked by
    # sharp curved gorilla canines instead of blocky slabs.
    upper_teeth = []
    for idx, (angle_deg, length, width) in enumerate(((-34, 0.040, 0.014), (-15, 0.026, 0.016), (0, 0.027, 0.017), (15, 0.026, 0.016), (34, 0.040, 0.014))):
        ang = math.radians(angle_deg)
        tx = math.sin(ang) * 0.112
        ty = -0.288 - math.cos(ang) * 0.112
        tip_drop = 0.007 if abs(angle_deg) > 25 else 0.0
        upper_teeth.append(
            core.tube(
                f"tooth_u_{idx}",
                [
                    _from(HEAD, (tx, ty, 0.236)),
                    _from(HEAD, (tx, ty - 0.003, 0.236 - length * 0.55)),
                    _from(HEAD, (tx * 0.98, ty + 0.002, 0.236 - length - tip_drop)),
                ],
                lambda t, w=width, fang=abs(angle_deg) > 25: w * ((1.0 - 0.82 * t) if fang else (0.95 - 0.35 * t)),
                detail,
                sides=8,
                flatten=0.72,
            )
        )
    teeth_u = core.join(upper_teeth, "teeth_upper")
    teeth_u.parent = head
    creature.finish(core.shade_smooth(teeth_u, angle=45))

    jaw = creature.pivot("jaw", head, _from(HEAD, JAW))

    creature.form(
        "jaw_lower",
        _hang(
            [
                ("sphere", (0, -0.348, 0.115), (0.116, 0.072, 0.042)),
                ("sphere", (0, -0.396, 0.132), (0.110, 0.048, 0.028)),
                ("sphere", (0, -0.376, 0.096), (0.090, 0.056, 0.028)),
            ],
            JAW,
        ),
        trim,
        jaw,
        triangles=440,
        voxel=0.012,
    )

    creature.form("mouth_floor", _hang([("sphere", (0, -0.295, 0.158), (0.096, 0.056, 0.036))], JAW), dark, jaw, triangles=90)

    # Lower curved canines + compact lower incisors tucked cleanly inside the jaw.
    lower_teeth = []
    for idx, (angle_deg, length, width) in enumerate(((-30, 0.032, 0.013), (-11, 0.018, 0.012), (11, 0.018, 0.012), (30, 0.032, 0.013))):
        ang = math.radians(angle_deg)
        tx = math.sin(ang) * 0.104
        ty = -0.296 - math.cos(ang) * 0.098
        lower_teeth.append(
            core.tube(
                f"tooth_l_{idx}",
                [
                    _from(JAW, (tx, ty, 0.145)),
                    _from(JAW, (tx, ty - 0.002, 0.145 + length * 0.55)),
                    _from(JAW, (tx * 0.96, ty + 0.003, 0.145 + length)),
                ],
                lambda t, w=width, fang=abs(angle_deg) > 20: w * ((1.0 - 0.80 * t) if fang else (0.95 - 0.35 * t)),
                detail,
                sides=8,
                flatten=0.72,
            )
        )
    teeth_l = core.join(lower_teeth, "teeth_lower")
    teeth_l.parent = jaw
    creature.finish(core.shade_smooth(teeth_l, angle=45))

    _cap(creature.pivot("trait_1", head, (0, 0, 0)), detail, trim)
    _crown(creature.pivot("trait_2", head, (0, 0, 0)), detail)


def _arm(root, side, name, body, trim, floor):
    joint = _mirror(SHOULDER, side)
    arm = creature.pivot(f"leg_f{name}", root, joint)

    # Muscular deltoid cap, sculpted bicep/tricep, and heavy tapering gorilla forearm.
    creature.form(
        f"forelimb_{name}",
        _hang(
            [
                ("sphere", joint, (0.155, 0.165, 0.155)),
                ("sphere", (side * 0.355, -0.075, -0.03), (0.122, 0.132, 0.195)),
                ("sphere", (side * 0.370, -0.14, -0.18), (0.105, 0.112, 0.115)),
                ("sphere", (side * 0.370, -0.21, -0.32), (0.112, 0.128, 0.21), (math.radians(-18), 0, 0)),
            ],
            joint,
        ),
        body,
        arm,
        triangles=1100,
        voxel=BODY_VOXEL,
    )

    # Articulated gorilla knuckle fist with distinct metacarpal ridges and thumb.
    creature.form(
        f"fist_{name}",
        _hang(
            [
                ("sphere", (side * 0.36, -0.285, floor + 0.082), (0.095, 0.108, 0.078)),
                ("sphere", (side * 0.275, -0.315, floor + 0.068), (0.042, 0.064, 0.042), (0, 0, math.radians(side * 18))),
            ]
            + [
                ("sphere", (side * 0.36 + across, -0.368, floor + 0.056), (0.033, 0.064, 0.056))
                for across in (-0.062, -0.020, 0.022, 0.062)
            ],
            joint,
        ),
        trim,
        arm,
        triangles=480,
        voxel=0.011,
    )


def _leg(root, side, name, body, trim, floor):
    joint = _mirror(HIP, side)
    leg = creature.pivot(f"leg_b{name}", root, joint)

    creature.form(
        f"haunch_{name}",
        _hang(
            [
                ("sphere", (side * 0.25, 0.38, -0.25), (0.132, 0.160, 0.182)),
                ("sphere", (side * 0.308, 0.38, -0.39), (0.088, 0.094, 0.122), (0, math.radians(side * -14), 0)),
            ],
            joint,
        ),
        body,
        leg,
        triangles=640,
        voxel=BODY_VOXEL,
    )

    # Sculpted bare ape foot with distinct hallux (big toe) and toe pads flat on floor.
    creature.form(
        f"foot_{name}",
        _hang(
            [
                ("sphere", (side * 0.29, 0.30, floor + 0.040), (0.086, 0.152, 0.040)),
                ("sphere", (side * 0.30, 0.39, floor + 0.062), (0.086, 0.088, 0.062)),
                # Opposable big toe angled inward.
                ("sphere", (side * 0.215, 0.23, floor + 0.032), (0.034, 0.062, 0.032), (0, 0, math.radians(side * -30))),
                ("sphere", (side * 0.29, 0.165, floor + 0.030), (0.064, 0.042, 0.030)),
            ],
            joint,
        ),
        trim,
        leg,
        triangles=320,
        voxel=0.011,
    )


def _cap(parent, cloth, button):
    """Crisp paneled baseball cap with curved stitched brim."""

    dome = core.revolve(
        "cap",
        [
            (0.190, 0.432),
            (0.214, 0.448),
            (0.214, 0.482),
            (0.194, 0.525),
            (0.148, 0.562),
            (0.082, 0.584),
            (0.0, 0.592),
        ],
        cloth,
        parent,
        segments=30,
        shape=lambda a, r, h: (r * (1.0 - 0.03 * math.cos(a * 2)), h),
        cap_bottom=True,
    )
    dome.location = _from(HEAD, (0, -0.12, 0))
    creature.finish(core.shade_smooth(dome))

    # Curved athletic baseball bill sweeping out over the brow.
    rings = []
    for step in range(8):
        t = step / 7
        span = 0.175 * (1.0 - 0.42 * (t**1.8))
        y = -0.28 - 0.22 * t
        z_mid = 0.456 - 0.022 * t
        ring = []
        for s in range(12):
            u = (s / 11) * 2.0 - 1.0
            # Gentle downward arch across the width of the bill.
            arch = -0.022 * (u * u) * (0.4 + 0.6 * t)
            ring.append(_from(HEAD, (u * span, y + 0.035 * (u * u), z_mid + arch + 0.008)))
        for s in range(11, -1, -1):
            u = (s / 11) * 2.0 - 1.0
            arch = -0.022 * (u * u) * (0.4 + 0.6 * t)
            ring.append(_from(HEAD, (u * span, y + 0.035 * (u * u), z_mid + arch - 0.008)))
        rings.append(ring)

    brim = core.loft("cap_brim", rings, cloth, parent, closed=True, cap_start=True, cap_end=True)
    creature.finish(core.shade_smooth(brim, angle=50))

    btn = core.revolve(
        "cap_button",
        [(0.0, 0.606), (0.020, 0.604), (0.030, 0.596), (0.026, 0.588), (0.0, 0.586)],
        button,
        parent,
        segments=14,
    )
    btn.location = _from(HEAD, (0, -0.12, 0))
    creature.finish(core.shade_smooth(btn))


def _crown(parent, gold):
    """Thickened flared royal crown with sculpted spires and embossed band."""

    points = 7
    steps = points * 4

    def ring(across, deep, height, spike=0.0):
        loop = []
        for step in range(steps):
            angle = step / steps * math.tau
            tooth = abs(math.sin(angle * points / 2)) ** 1.4
            at = (math.cos(angle) * across, -0.10 + math.sin(angle) * deep, height + spike * tooth)
            loop.append(_from(HEAD, at))
        return loop

    crown = core.loft(
        "crown",
        [
            ring(0.150, 0.140, 0.440),
            ring(0.172, 0.162, 0.540, spike=0.092),
            ring(0.156, 0.146, 0.536, spike=0.088),
            ring(0.136, 0.126, 0.440),
            ring(0.150, 0.140, 0.440),
        ],
        gold,
        parent,
        closed=True,
    )
    creature.finish(core.shade_smooth(crown, angle=42))

    band = core.loft(
        "crown_band",
        [
            ring(0.152, 0.142, 0.466),
            ring(0.168, 0.158, 0.478),
            ring(0.170, 0.160, 0.502),
            ring(0.158, 0.148, 0.514),
            ring(0.152, 0.142, 0.466),
        ],
        gold,
        parent,
        closed=True,
    )
    creature.finish(core.shade_smooth(band, angle=45))
