"""The harvester: the field hand of death.

Silhouette: a wide hat on a tall, narrow body, with a scythe longer than the
figure. The outline reads as a "T" crossed by a long diagonal.
"""

import math
import random

from mathutils import Vector, noise

from lib import cloth, core
from lib.figure import HEAD_HEIGHT, NECK_HEIGHT, SHOULDER_HEIGHT, arms, eyes, face_wrap, finish, void_head

HUNCH = 0.06


def build():
    """The field hand of death: a wide hat on a tall, narrow body."""

    root = core.empty("reaper")
    cloth_mat = core.material("cloth")
    alt = core.material("cloth_alt")
    forward = HUNCH

    # A long, narrow coat that barely widens until it frays.
    hems = [
        dict(tatter=0.34, tears=8, slits=12, fold_depth=0.15, seed=11),
        dict(tatter=0.5, tears=13, slits=18, fold_depth=0.18, seed=12),
        dict(tatter=0.2, tears=5, slits=6, fold_depth=0.12, seed=13),
        dict(tatter=0.42, tears=7, slits=12, fold_depth=0.2, seed=14, lean=0.18),
    ]

    for index, hem in enumerate(hems):
        finish(
            cloth.robe(
                f"robe_{index}",
                cloth_mat,
                root,
                top=SHOULDER_HEIGHT + 0.06,
                hem=0.06,
                profile=((0.0, 0.11, 0.1), (0.05, 0.2, 0.125), (0.14, 0.215, 0.13), (0.42, 0.17, 0.14), (1.0, 0.3, 0.27)),
                hunch=forward,
                **hem,
            ),
            seed=index,
        )

    arms(
        root,
        weapon_side_path=[(-0.2, -forward, SHOULDER_HEIGHT - 0.03), (-0.34, -forward - 0.04, 1.36), (-0.33, -forward - 0.27, 1.3)],
        off_side_path=[(0.2, -forward, SHOULDER_HEIGHT - 0.03), (0.29, -forward + 0.02, 1.28), (0.31, -forward - 0.05, 0.98)],
    )

    head = core.empty("head", root, location=(0, -forward, NECK_HEIGHT))
    _heads(head, alt, forward)
    _mantles(root, alt)

    return root


def _heads(head, alt, forward):
    """Three hats. The brim hides the face in every one."""


    offset = Vector(head.location)

    def place(obj):
        obj.location = Vector(obj.location) - offset

        return obj

    def ragged(angle, radius, height, variant, seed):
        """Moves a point of the hat's profile: the brim sags, tips forward
        and frays, more so the further out it is."""

        edge = max(0.0, (radius - 0.16) / 0.36)
        wobble = noise.noise(Vector((math.cos(angle) * 1.7, math.sin(angle) * 1.7, seed)))
        nick = noise.noise(Vector((math.cos(angle) * 6.0, math.sin(angle) * 6.0, seed + 4)))
        sag = variant["droop"] * edge**2 * (0.7 + 0.5 * math.cos(angle * 2 + variant["twist"]))
        tip = variant["tip"] * radius * -math.sin(angle)

        return (
            radius * (1 + edge * (0.06 * wobble - variant["tear"] * max(0.0, nick) ** 1.5)),
            height - sag - tip + 0.018 * edge * wobble,
        )

    brim = HEAD_HEIGHT + 0.075
    variants = [
        # A flat, wide brim over a low crown, tipped down over the eyes.
        dict(
            profile=[(0.0, 0.135), (0.105, 0.13), (0.13, 0.1), (0.138, 0.02), (0.2, 0.008), (0.54, 0.014), (0.542, 0.004), (0.2, -0.006), (0.1, -0.006)],
            droop=0.07,
            tear=0.22,
            tip=0.16,
            twist=0.0,
        ),
        # A tall, crushed crown and a brim that sags at the sides.
        dict(
            profile=[(0.0, 0.33), (0.06, 0.32), (0.1, 0.24), (0.132, 0.02), (0.2, 0.008), (0.46, 0.01), (0.462, 0.0), (0.2, -0.006), (0.1, -0.006)],
            droop=0.2,
            tear=0.3,
            tip=0.12,
            twist=math.pi,
        ),
        # A shallow woven cone, wider than the shoulders.
        dict(
            profile=[(0.0, 0.19), (0.04, 0.18), (0.3, 0.07), (0.6, -0.05), (0.602, -0.06), (0.3, 0.045), (0.1, 0.02)],
            droop=0.0,
            tear=0.08,
            tip=0.1,
            twist=0.0,
        ),
    ]

    for index, variant in enumerate(variants):
        group = core.empty(f"head_{index}", head)
        hat = core.revolve(
            f"hat_{index}",
            [(radius, brim + height) for radius, height in variant["profile"]],
            alt,
            None,
            segments=96,
            shape=lambda angle, radius, height, v=variant, s=index: ragged(angle, radius, height, v, s * 3.1),
        )
        hat.location = (0, -forward, 0)
        hat.parent = group
        place(finish(core.shade_smooth(hat, angle=55), seed=index))

        if index == 2:
            # Ribs under the cone, so it reads as woven rather than turned.
            ribs = []

            for rib in range(18):
                angle = math.tau * rib / 18
                points = []

                for radius, height in ((0.05, 0.175), (0.3, 0.062), (0.595, -0.052)):
                    r, z = ragged(angle, radius, brim + height, variant, index * 3.1)
                    points.append((math.cos(angle) * r, math.sin(angle) * r - forward, z + 0.004))

                ribs.append(core.tube(f"rib_{rib}", points, 0.006, core.material("wood"), sides=6))

            ribbing = core.join(ribs, "ribs")
            ribbing.parent = group
            place(finish(ribbing))
        else:
            band = core.revolve(
                f"band_{index}",
                [(0.14, brim + 0.075), (0.15, brim + 0.065), (0.15, brim + 0.03), (0.142, brim + 0.022)],
                core.material("rope"),
                None,
                segments=48,
                shape=lambda angle, radius, height, v=variant: (radius, height - v["tip"] * radius * -math.sin(angle)),
            )
            band.location = (0, -forward, 0)
            band.parent = group
            place(finish(band))

        place(void_head(group, forward, f"void_{index}"))
        place(face_wrap(group, forward, f"wrap_{index}", seed=index))
        place(eyes(group, forward, name=f"eyes_{index}"))


def _mantles(root, alt):
    """What sits on a harvester's shoulders."""


    forward = HUNCH

    # A short capelet over both shoulders, frayed to points.
    capelet = cloth.robe(
        "capelet",
        alt,
        None,
        top=NECK_HEIGHT - 0.03,
        hem=SHOULDER_HEIGHT - 0.4,
        profile=((0.0, 0.1, 0.1), (0.28, 0.25, 0.15), (1.0, 0.33, 0.21)),
        folds=(4, 7, 11),
        fold_depth=0.09,
        tatter=0.6,
        tears=9,
        slits=0,
        seed=21,
        rings=18,
        segments=96,
    )
    capelet.shape_key_clear()
    capelet.location = (0, -forward, 0)
    group = core.empty("mantle_0", root)
    capelet.parent = group
    finish(capelet, seed=3)

    # A sheaf of cut stalks bound across the back, standing over one shoulder.
    group = core.empty("mantle_1", root)
    generator = random.Random(5)
    stalks = []

    for stalk in range(46):
        spread = generator.uniform(-1, 1)
        lift = generator.uniform(-1, 1)
        start = Vector((0.22 + spread * 0.035, 0.13 + lift * 0.025, 1.12 + generator.uniform(-0.05, 0.05)))
        end = Vector((-0.34 + spread * 0.16, 0.12 + lift * 0.09 - forward, 2.08 + generator.uniform(-0.14, 0.16)))

        stalks.append(
            core.tube(
                f"stalk_{stalk}",
                [start, start.lerp(end, 0.5) + Vector((0, 0.035, 0)), end],
                # Each stalk ends in a heavy ear.
                lambda t: 0.0042 + 0.011 * max(0.0, 1 - abs(t - 0.93) / 0.07) ** 2,
                core.material("rope"),
                sides=5,
            )
        )

    sheaf = core.join(stalks, "sheaf")
    sheaf.parent = group
    finish(sheaf, wear_boost=1.5)

    strap = core.tube(
        "strap",
        core.bezier(
            [
                (0.2, -0.11 - forward, 1.62),
                (0.05, -0.16 - forward * 0.6, 1.47),
                (-0.17, -0.15, 1.27),
                (-0.2, 0.06, 1.24),
                (0.0, 0.16, 1.45),
                (0.2, 0.1 - forward, 1.66),
                (0.2, -0.11 - forward, 1.62),
            ],
            samples=8,
        ),
        0.019,
        core.material("rope"),
        group,
        sides=8,
        cap=False,
    )
    finish(strap)

    # A wound collar that swallows the neck, with two long tails down the
    # back that the game lets trail.
    group = core.empty("mantle_2", root)
    coils = []

    for coil in range(4):
        height = SHOULDER_HEIGHT - 0.01 + coil * 0.052
        radius = 0.2 - coil * 0.02
        path = []

        for step in range(49):
            angle = math.tau * step / 48
            path.append(
                (
                    math.cos(angle) * radius * 1.12,
                    math.sin(angle) * radius * 0.8 - forward * (0.8 + coil * 0.07),
                    height + 0.02 * math.sin(angle * 2 + coil * 1.7) + 0.022 * math.cos(angle),
                )
            )

        coils.append(core.tube(f"coil_{coil}", path, 0.05 - coil * 0.004, alt, sides=10, cap=False, flatten=0.5))

    collar = core.join(coils, "collar")
    collar.parent = group
    finish(core.shade_smooth(collar), seed=5)

    for tail, (start, width, drop) in enumerate(((0.02, 0.16, 0.95), (-0.17, 0.11, 0.7))):
        finish(
            cloth.drape(
                f"collar_tail_{tail}",
                alt,
                group,
                [(start + width * step / 10, 0.13 - forward + 0.012 * math.sin(step), SHOULDER_HEIGHT + 0.02) for step in range(11)],
                drop=drop,
                width_gain=1.25,
                tatter=0.5,
                sag=0.07,
                seed=8 + tail,
                outward=(0, 1, 0),
            ),
            seed=tail,
        )


