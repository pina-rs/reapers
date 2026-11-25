"""Key-pose curves: what each joint does between the poses a clip names.

A clip is a list of keys, `(time, pose)` or `(time, pose, manner)`, with
`time` running from 0 (the clip's start) to 1 (its end).

- A `stance` is a whole pose: a joint it leaves out goes back to its bind
  pose. `REST` is the empty stance, and every one-shot starts and ends on it.
- A `pose` is a breakdown: a joint it leaves out holds what the keys before
  it left it at. A joint never snaps back to rest because a key forgot it;
  it returns only where a stance (or the pose itself) says so.

A key's manner is how the joints it names pass it. It is one manner for
all of them, or a dict of manners by joint with `"*"` for the others:

- `AUTO`, the default: shape-preserving. A joint flows through a key its
  path runs on through or turns at, as a blade looping round does, slows
  as the turn tightens, and eases to a stop at a key where it turns back.
  A joint a key holds rather than names always passes it this way.
- `PASS`: crosses the key at speed, as a body crosses the top of a jump:
  as fast as it came in, up to what the move after it can slow from
  steadily without stalling and picking up again.
- `STRIKE`: a contact. It speeds up all the way into the key and crosses
  it at the top of its speed, `DRIVE` times as fast as the move into it
  averaged, slowed only where the move after it is too short to carry
  that speed without passing its next key. The follow-through keys after
  it are `CARRY`.
- `CARRY`: a follow-through. The joint only slows through it, never
  stalling and picking up again: it crosses the key no faster than the
  move after it can slow from steadily, and no slower than the move into
  it can slow to steadily from the speed it came in at, so a blade's speed
  after its contact dies away in one long fall rather than in pulses.
- `STOP`: eases into the key and out again: a held pose, an apex, the
  wind-up a body anticipates from.
- `over(amount, apex, settle)`: arrives at speed, carries on past the key by
  `amount` of the move that brought it there, peaking `apex` seconds later,
  and settles back onto the key `settle` seconds after that when there is
  room before the next key, or turns from the apex into the next move when
  there is not, as a blade carries through its follow-through.
- `stab(apex, settle)`: a contact that drives on past its key and settles
  back onto it, as a point drives in or a palm lands: it crosses the key
  as a `STRIKE` does, carries on as far as that speed takes it in `apex`
  seconds, slowing all the way, and settles back `settle` seconds later.

A turn is interpolated as a rotation vector (a log-quaternion: an axis
scaled by its angle) rather than Euler axis by Euler axis, so between two
keys about one axis a joint turns about that axis, and a path between keys
about different axes is a smooth arc with no Euler coupling. Poses are
written as Euler angles in degrees, local XYZ in Blender's Z-up frame, and
each is wound as its angles turn from rest: (200, 0, 0) is 200 degrees one
way, not 160 the other. The one joint called `move` is a translation in
metres and is interpolated as it is.

Curves are cubic Hermite, so velocity is continuous through every key that
is not a stop. A joint's speed through a key follows the path, not its
axes: where the path turns by more than a right angle it slows, and where
it doubles back it stops, whatever the key's manner. A follow-through that
loops round (each key turning the path gently) keeps a blade moving; one
that comes back the way it went stops it dead.
"""

import math

from mathutils import Euler, Quaternion, Vector

AUTO = "auto"
PASS = "pass"
STOP = "stop"
STRIKE = "strike"
CARRY = "carry"
# How much faster than the move into it averaged a joint crosses a contact.
# A Hermite segment from rest speeds up all the way to its end while its
# end slope is at least 1.5x its average, and is a steady push (constant
# acceleration) at 2x: between the two the joint is still speeding up as
# it crosses, so its top speed is the contact.
DRIVE = 1.8
# The translation joint: every other joint is a turn.
TRANSLATION = "move"


class Pose(dict):
    """A breakdown: the joints it names; the others hold."""

    whole = False


class Stance(Pose):
    """A whole pose: the joints it names; the others go back to rest."""

    whole = True


def pose(**joints):
    return Pose(joints)


def stance(**joints):
    return Stance(joints)


REST = Stance()


class Over:
    """Carries past a key and settles back onto it (see the module). A
    stab (`drive`) crosses the key as a strike does, and by default
    (`amount` None) carries on as far as that speed takes it."""

    def __init__(self, amount, apex, settle, drive=False):
        self.amount, self.apex, self.settle, self.drive = amount, apex, settle, drive

    def __repr__(self):
        if self.drive:
            return f"stab({self.apex}, {self.settle}, {self.amount})"

        return f"over({self.amount}, {self.apex}, {self.settle})"


def over(amount=0.12, apex=0.07, settle=0.12):
    """A manner: carry `amount` of the incoming move past the key, peaking
    `apex` seconds after it, and settle back `settle` seconds later."""

    return Over(amount, apex, settle)


def stab(apex=0.05, settle=0.12, amount=None):
    """A contact's manner: cross the key at full speed, drive on past it as
    far as that speed carries it in `apex` seconds (or by `amount` of the
    move that brought it there, for a joint that must stop short, as an
    elbow locks), and settle back onto it `settle` seconds later."""

    return Over(amount, apex, settle, drive=True)


# How a joint arrives at a key it will carry past: at the speed that brought
# it there, kept low enough that it reaches its apex without turning back.
_ARRIVE = "arrive"
# A stab's: as a `STRIKE` crosses its contact, the apex taking the place of
# the follow-through.
_DRIVE = "drive"


def turn(euler):
    """Euler degrees as a rotation vector in degrees, wound as the angles
    turn from rest."""

    if not any(euler):
        return Vector()

    steps = 1 + int(max(abs(angle) for angle in euler) // 40)
    wound = Vector()

    for step in range(1, steps + 1):
        axis, angle = Euler([math.radians(a * step / steps) for a in euler]).to_quaternion().to_axis_angle()
        axis = Vector(axis)
        # Of the same turn written about `axis` (angle + 2πn), the one nearest
        # the last step's.
        options = [axis * math.degrees(angle + math.tau * n) for n in (-1, 0, 1)]
        wound = min(options, key=lambda option: (option - wound).length)

    return wound


def quaternion(vector):
    """A rotation vector in degrees as a quaternion."""

    angle = vector.length

    if angle < 1e-9:
        return Quaternion()

    return Quaternion(vector / angle, math.radians(angle))


class Track:
    """One joint's curve through its keys."""

    def __init__(self, keys, seconds):
        """`keys` are `(time, value, manner)` with `value` a Vector."""

        self.keys = _carry_through(keys, seconds)
        times = [time for time, _, _ in self.keys]
        values = [value for _, value, _ in self.keys]
        self.times, self.values = times, values
        self.tangents = [_tangent(times, values, index, manner) for index, (_, _, manner) in enumerate(self.keys)]

        # In order, so a follow-through slows from the speed its contact
        # (or the follow-through key before it) actually crosses at.
        for index, (_, _, manner) in enumerate(self.keys[1:-1], start=1):
            if manner == PASS:
                self.tangents[index] = _slowing(times, values, self.tangents, index)
            elif manner == CARRY:
                self.tangents[index] = _dying(times, values, self.tangents, index)

    def at(self, phase):
        times, values, tangents = self.times, self.values, self.tangents

        if phase <= times[0]:
            return values[0].copy()

        if phase >= times[-1]:
            return values[-1].copy()

        index = next(i for i in range(len(times) - 1) if phase <= times[i + 1])
        span = times[index + 1] - times[index]
        t = (phase - times[index]) / span
        t2, t3 = t * t, t * t * t

        return (
            (2 * t3 - 3 * t2 + 1) * values[index]
            + (t3 - 2 * t2 + t) * span * tangents[index]
            + (-2 * t3 + 3 * t2) * values[index + 1]
            + (t3 - t2) * span * tangents[index + 1]
        )


def _carry_through(keys, seconds):
    """`keys` with every carried-through key expanded into the key (arriving
    at speed), its apex past it, and the key again once it has settled."""

    out = []

    for index, (time, value, manner) in enumerate(keys):
        if not isinstance(manner, Over):
            out.append((time, value, manner))
            continue

        if index == 0 or index == len(keys) - 1:
            out.append((time, value, STOP))
            continue

        before = keys[index - 1][1]
        apex, settle = manner.apex / seconds, manner.settle / seconds
        room = keys[index + 1][0] - time
        carry = min(apex, room / 3)

        # A stab's move in ends at `DRIVE` times its average speed, and by
        # default the carry slows from that to its apex steadily, which a
        # Hermite segment ending at rest does at half its start speed.
        amount = DRIVE / 2 * carry / (time - keys[index - 1][0]) if manner.amount is None else manner.amount
        out.append((time, value, _DRIVE if manner.drive else _ARRIVE))

        # The carry takes at most a third of the way to the next key, so the
        # move after it is never crammed. It settles back onto the key only
        # when the settle fits in half the way; otherwise it turns from its
        # apex straight into the next move, as a blade carried through its
        # follow-through turns into the recovery.
        out.append((time + carry, value + amount * (value - before), STOP))

        if apex + settle <= room / 2:
            out.append((time + apex + settle, value.copy(), AUTO))

    return out


def _tangent(times, values, index, manner):
    last = len(times) - 1

    if manner == STOP or (manner == AUTO and index in (0, last)):
        return Vector((0, 0, 0))

    slopes = [(values[i + 1] - values[i]) / (times[i + 1] - times[i]) for i in (index - 1, index) if 0 <= i < last]

    if manner == PASS:
        if index in (0, last):
            return slopes[0]

        # A Hermite segment from rest speeds up all the way to its end while
        # its end slope is at least 1.5x its average, and the segment after
        # stays short of its key while its start slope is at most 3x its
        # own.
        incoming, outgoing = slopes

        return _through(incoming, outgoing, min(1.5 * incoming.length, 3 * outgoing.length))

    if manner in (STRIKE, _DRIVE):
        # Speeding up all the way in, and no faster than the move after it
        # can carry without passing its next key: a Hermite segment ending
        # at rest stays short of its key while its start slope is at most
        # 3x its average.
        incoming, outgoing = slopes

        return _through(incoming, outgoing, min(DRIVE * incoming.length, 3 * outgoing.length))

    if manner == _ARRIVE:
        # The speed it came in at, but no faster than reaches the apex (the
        # next key) without passing it: a Hermite segment ending at rest
        # stays monotone while its starting slope is at most 3x its average.
        incoming, outgoing = slopes
        limit = 3 * outgoing

        return Vector([math.copysign(min(abs(i), abs(o)), o) if i * o > 0 else 0.0 for i, o in zip(incoming, limit)])

    # AUTO between two keys: Fritsch-Butland's harmonic mean of the speeds,
    # which on a straight path keeps the curve from passing either key.
    incoming, outgoing = slopes
    a, b = incoming.length, outgoing.length

    return _through(incoming, outgoing, 0.0 if a + b == 0 else 2 * a * b / (a + b))


def _slowing(times, values, tangents, index):
    """A pass key's tangent, no faster than lets the segment after it slow
    all the way to the next key: a Hermite segment's speed falls steadily
    while its start slope is at most 3x its average less twice its end
    slope. Faster, it would dip in the middle and pick up again, a stop
    just after the contact a swing should be carrying through."""

    span = times[index + 1] - times[index]
    average = (values[index + 1] - values[index]).length / span
    tangent = tangents[index]
    limit = max(average, 3 * average - 2 * tangents[index + 1].length)

    return tangent if tangent.length <= limit else tangent.normalized() * limit


def _dying(times, values, tangents, index):
    """A follow-through key's tangent: its AUTO direction, at a speed the
    move into it slows steadily to from the speed it came in at, and the
    move after it slows steadily from. A Hermite segment's speed falls all
    the way from its start slope `a` to its end slope `b` while `2a + b` is
    at least 3x its average and `a + 2b` at most. Where the move into the
    key is faster than the speed it came in at, the joint must pick up
    again and the key keeps its AUTO speed."""

    tangent = tangents[index]

    if tangent.length < 1e-12:
        return tangent

    span = times[index] - times[index - 1]
    average = (values[index] - values[index - 1]).length / span
    entry = tangents[index - 1].length
    low, high = max(0.0, 3 * average - 2 * entry), max(0.0, (3 * average - entry) / 2)

    if low > high:
        return tangent

    span = times[index + 1] - times[index]
    onward = (values[index + 1] - values[index]).length / span
    # And slow enough to slow steadily into the next key at rest.
    high = min(high, 3 * onward)

    return tangent.normalized() * min(max(tangent.length, low), max(low, high))


def _through(incoming, outgoing, speed):
    """The velocity through a key between two moves: along the bisector of
    their directions at `speed`, slowed as the path doubles back, from full
    speed through a turn of up to a right angle to a stop where it reverses.
    Taken as a path rather than axis by axis, so a turn whose rotation vector
    swings round keeps moving where one of its components turns back."""

    a, b = incoming.length, outgoing.length

    if a < 1e-12 or b < 1e-12:
        return Vector((0, 0, 0))

    direction = incoming / a + outgoing / b

    if direction.length < 1e-12:
        return Vector((0, 0, 0))

    keep = max(0.0, min(1.0, 1.0 + incoming.dot(outgoing) / (a * b)))

    return direction.normalized() * speed * keep


def compile(keys, seconds):
    """Every joint's `Track` through `keys`, a clip `seconds` long."""

    joints = set().union(*(key[1].keys() for key in keys))
    held = {joint: Vector() for joint in joints}
    per_joint = {joint: [] for joint in joints}

    for key in keys:
        time, shown = key[0], key[1]
        manner = key[2] if len(key) > 2 else AUTO

        for joint in joints:
            if joint in shown:
                raw = shown[joint]
                value = Vector(raw) if joint == TRANSLATION else turn(raw)
                chosen = manner.get(joint, manner.get("*", AUTO)) if isinstance(manner, dict) else manner
            elif shown.whole:
                value, chosen = Vector(), manner.get("*", AUTO) if isinstance(manner, dict) else manner
            else:
                value, chosen = held[joint].copy(), AUTO

            held[joint] = value
            per_joint[joint].append((time, value, chosen))

    return {joint: Track(track, seconds) for joint, track in per_joint.items()}


def sample(tracks, phase):
    """Every joint of compiled `tracks` at `phase`: a rotation vector in
    degrees for a turn, metres for `move`."""

    return {joint: track.at(phase) for joint, track in tracks.items()}
