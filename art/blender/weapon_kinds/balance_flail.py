"""The balance-flail: the weigher's weapon.

A scale on a staff: a beam across the top of a haft like a tall "T", with a
shallow pan hanging on three chains from each end. The beam has already
tipped, so one pan hangs lower than the other.
"""

import math

from mathutils import Vector

from lib import core
from lib.armoury import binding, finish, haft

BEAM_HEIGHT = 0.59
BEAM_REACH = 0.6
TILT = math.radians(6)
PAN_RADIUS = 0.135
CHAIN_DROP = 0.25


def build():
    """The weigher's weapon: a scale on a staff, tipped so that one pan hangs
    lower than the other."""

    root = core.empty("weapon_balanceFlail")
    gold = core.material("gold")

    haft("flail_haft", root, bottom=-0.88, top=BEAM_HEIGHT, bow=0.01)
    binding("flail_grip", root, -0.09, 0.12, turns=5)

    # A shod foot, so the staff can be set down like a gavel.
    finish(
        core.revolve(
            "flail_foot",
            [(0.0, -0.95), (0.012, -0.915), (0.024, -0.88), (0.022, -0.77), (0.0, -0.77)],
            gold,
            root,
            segments=10,
        ),
        wear_boost=1.6,
    )
    finish(
        core.revolve(
            "flail_socket",
            [(0.0, BEAM_HEIGHT - 0.03), (0.031, BEAM_HEIGHT - 0.04), (0.034, BEAM_HEIGHT - 0.075), (0.027, BEAM_HEIGHT - 0.2), (0.0, BEAM_HEIGHT - 0.21)],
            gold,
            root,
            segments=12,
        ),
        wear_boost=1.6,
    )

    # The beam is deepest over the pivot and ends in a knob at each tip.
    stations = (-1.0, -0.975, -0.94, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75, 0.94, 0.975, 1.0)
    depths = (0.013, 0.023, 0.014, 0.019, 0.025, 0.031, 0.038, 0.031, 0.025, 0.019, 0.014, 0.023, 0.013)

    finish(
        core.tube(
            "flail_beam",
            [_on_beam(station * BEAM_REACH, 0.02 * (1 - station * station)) for station in stations],
            lambda t: depths[round(t * (len(depths) - 1))],
            gold,
            root,
            sides=10,
            flatten=0.5,
        ),
        wear_boost=1.8,
    )

    pivot = _on_beam(0.0, 0.02)

    finish(
        core.tube(
            "flail_pivot",
            [pivot + Vector((0, depth, 0)) for depth in (-0.03, -0.024, 0.024, 0.03)],
            lambda t: 0.034 + 0.01 * (1 - abs(2 * t - 1)) ** 0.25,
            gold,
            root,
            sides=14,
        ),
        wear_boost=1.8,
    )

    # The pin the beam turns on is the one light.
    core.tube(
        "flail_pin",
        [pivot + Vector((0, depth, 0)) for depth in (-0.04, -0.034, 0.034, 0.04)],
        lambda t: 0.009 + 0.008 * (1 - abs(2 * t - 1)) ** 0.25,
        core.material("accent"),
        root,
        sides=8,
    )

    for side, label in ((-1, "risen"), (1, "sunk")):
        tip = _on_beam(side * BEAM_REACH * 0.975, 0.0)
        hook = tip - Vector((0, 0, 0.062))
        rim = hook - Vector((0, 0, CHAIN_DROP))
        links = [_link(f"flail_shackle_{label}", (tip + hook) / 2, Vector((0, 0, 1)), Vector((0, 1, 0)), 0.07, 0.034)]

        for chain in range(3):
            angle = math.tau * (chain / 3 + 0.25)
            lug = rim + Vector((math.cos(angle), math.sin(angle), 0)) * (PAN_RADIUS - 0.006)

            links.extend(_chain(f"flail_chain_{label}_{chain}", hook, lug))

        chains = core.join(links, f"flail_chains_{label}")
        chains.parent = root
        pan = _pan(f"flail_pan_{label}", rim, gold, root)

        # The pan that weighed nothing is the first thing a hollowed scale
        # loses, from the dish up its chains, which leaves it tipped for good.
        hollow = (lambda co, low=rim.z - 0.06, high=tip.z: 0.5 * (co.z - low) / (high - low)) if side < 0 else None

        finish(chains, hollow=hollow, wear_boost=1.5)
        finish(pan, hollow=hollow, wear_boost=1.8)

    return root


def _on_beam(along, above):
    """A point `along` the tipped beam from its pivot and `above` its line.
    The +X end, across the bearer's body, is the one that has sunk."""

    return Vector(
        (
            along * math.cos(TILT) + above * math.sin(TILT),
            0,
            BEAM_HEIGHT - along * math.sin(TILT) + above * math.cos(TILT),
        )
    )


def _pan(name, rim, mat, parent, segments=32):
    """A shallow dish hanging level, with the centre of its rim at `rim`."""

    profile = (
        (0.0, -0.056),
        (0.06, -0.049),
        (0.108, -0.028),
        (PAN_RADIUS - 0.004, -0.006),
        (PAN_RADIUS, 0.003),
        (PAN_RADIUS - 0.009, 0.003),
        (0.1, -0.02),
        (0.055, -0.039),
        (0.0, -0.046),
    )
    rings = []

    for radius, height in profile:
        ring = []

        for segment in range(segments):
            angle = math.tau * segment / segments

            ring.append(rim + Vector((math.cos(angle) * radius, math.sin(angle) * radius, height)))

        rings.append(ring)

    return core.loft(name, rings, mat, parent)


def _link(name, centre, along, across, length, width, wire=0.0048, steps=6, sides=4):
    """One chain link: a closed oval of wire, long in the `along` direction."""

    out = along.cross(across)
    rings = []

    for step in range(steps + 1):
        turn = math.tau * step / steps
        spoke = along * math.cos(turn) + across * math.sin(turn)
        middle = centre + along * math.cos(turn) * (length / 2 - wire) + across * math.sin(turn) * (width / 2 - wire)
        ring = []

        for side in range(sides):
            angle = math.tau * side / sides

            ring.append(middle + (spoke * math.cos(angle) + out * math.sin(angle)) * wire)

        rings.append(ring)

    return core.loft(name, rings, core.material("metal"))


def _chain(name, start, end, pitch=0.043):
    """Links from `start` to `end`, every other one turned a quarter so that
    each passes through the next."""

    along = (end - start).normalized()
    flat = along.cross(Vector((0, 0, 1))).normalized()
    edge = along.cross(flat)
    count = max(2, round((end - start).length / pitch))
    step = (end - start).length / count

    return [
        _link(f"{name}_{index}", start.lerp(end, (index + 0.5) / count), along, flat if index % 2 else edge, step + 0.014, 0.022)
        for index in range(count)
    ]
