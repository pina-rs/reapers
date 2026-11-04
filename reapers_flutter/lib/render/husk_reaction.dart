import 'dart:math' as math;

/// What the hunt is doing to a husk, as far as its acting goes. The arena
/// reads it off the simulation each frame; nothing here feeds back into the
/// rules.
typedef HuskPlight = ({
  /// What is left of its ward, from 1 (untouched) to 0 (broken).
  double ward,

  /// Whether the fight has begun. Before it, the husk is unaware and calm.
  bool fighting,

  /// How far it has drawn breath to shriek or is charging, from 0 to 1. A
  /// husk that is attacking is furious, not afraid.
  double fury,

  /// Whether it is reeling from a parry, a heavy blow or a gust.
  bool staggered,

  /// Whether its ward is broken and it waits to be reaped.
  bool broken,

  /// How far along the arc the reap has cut, from 0 to 1.
  double reap,

  /// Whether it has been absorbed and is coming apart.
  bool dying,

  /// Whether it is frozen in ice: then nothing moves at all.
  bool frozen,
});

/// A husk at rest, before any hunt.
const HuskPlight calmPlight = (
  ward: 1,
  fighting: false,
  fury: 0,
  staggered: false,
  broken: false,
  reap: 0,
  dying: false,
  frozen: false,
);

/// How a husk acts out what is being done to it: it reels from blows,
/// cowers and trembles as its ward fails, rages while it attacks, struggles
/// against the reap and convulses as it comes apart.
///
/// This is only for show and decides nothing. It is plain arithmetic, with
/// no scene in it, so it can be tested; `HuskRig` turns what it says into
/// the turn of each pivot.
///
/// Angles are in radians, in the model's frame: x to the creature's left,
/// y up, z behind it.
final class HuskReaction {
  double _seconds = 0;

  // The body's sway from a blow: a lean (x and z, as a tilt in radians) and
  // a twist about the vertical, each a spring that overshoots and settles.
  //
  // A creature's body rocks at about three times a second: slower and it
  // floats, faster and it buzzes instead of reeling.
  final _leanX = _Spring(frequency: 3.2, damping: 0.34);
  final _leanZ = _Spring(frequency: 3.2, damping: 0.34);
  final _twist = _Spring(frequency: 4, damping: 0.3);
  final _squash = _Spring(frequency: 7, damping: 0.4);

  // The head snaps back further than the body, and comes back later.
  final _headBack = _Spring(frequency: 2.8, damping: 0.32);

  double _hitStop = 0;
  double _reap = 0;
  double _wince = 0;
  double _fear = 0;
  double _rage = 0;
  double _cower = 0;
  double _reel = 0;
  double _struggle = 0;
  double _death = 0;
  double _deathAge = 0;
  bool _frozen = false;

  /// How long time stands still for the husk after a heavy blow, in
  /// seconds: the instant the blade is in it.
  static const hitStopSeconds = 0.07;

  /// How fast the animation should run: near zero for an instant after a
  /// heavy blow, then 1. The springs keep moving through it, so the blow
  /// lands with a jolt rather than a pause.
  double get timeScale => _hitStop > 0 && !_frozen ? 0.08 : 1;

  /// A blow lands. [away] is the direction it drives the husk, in the
  /// model's frame, as an angle about the vertical from +x towards +z.
  /// [side] is where on the body it struck, from -1 (the creature's right)
  /// to 1 (its left), and twists it the other way. [strength] runs from 0
  /// to 1; a [heavy] one stops time for an instant.
  void hit({
    required double away,
    required double side,
    required double strength,
    bool heavy = false,
  }) {
    if (_frozen) {
      return;
    }

    final kick = strength.clamp(0.0, 1.0);
    // The spring is given a kick of speed, not of position, so the blow
    // shows as the body being driven back and then rocking forward again.
    // A full blow tips it about an eighth of a radian, for a moment. Seals
    // are struck where the rules hold them, so a husk still being fought
    // must not reel far from there; the wild motion is saved for once its
    // ward has broken, when nothing more is struck.
    final push = 2.6 * kick * (heavy ? 1.3 : 1);

    _leanX.velocity += math.cos(away) * push;
    _leanZ.velocity += math.sin(away) * push;
    _twist.velocity -= side.clamp(-1.0, 1.0) * 6 * kick;
    _squash.velocity -= 2.5 * kick;
    _headBack.velocity += 9 * kick;
    _wince = math.max(_wince, kick);

    if (heavy) {
      _hitStop = hitStopSeconds;
    }
  }

  /// Moves the acting on by [deltaSeconds] for a husk in [plight].
  void step(double deltaSeconds, HuskPlight plight) {
    _frozen = plight.frozen;

    if (_frozen) {
      // Ice holds it exactly as it was.
      _hitStop = 0;

      return;
    }

    _seconds += deltaSeconds;
    _hitStop = math.max(0, _hitStop - deltaSeconds);

    for (final spring in [_leanX, _leanZ, _twist, _squash, _headBack]) {
      spring.step(deltaSeconds);
    }

    _wince *= math.exp(-6 * deltaSeconds);

    // Fear grows as the ward fails, and fills it once the ward is broken.
    // Fury pushes it aside while the husk attacks.
    final lost = 1 - plight.ward.clamp(0.0, 1.0);
    final dread = plight.dying || plight.broken || plight.reap > 0
        ? 1.0
        : plight.fighting
        ? 0.25 + 0.75 * _smooth(lost)
        : 0.0;
    final fury = plight.fury.clamp(0.0, 1.0);

    _fear = _toward(_fear, dread * (1 - fury), 3.5, deltaSeconds);
    _rage = _toward(_rage, fury, 9, deltaSeconds);
    _cower = _toward(
      _cower,
      plight.broken || plight.reap > 0
          ? 1
          : math.max(0, _fear - 0.35) / 0.65 * 0.6,
      5,
      deltaSeconds,
    );
    _reel = _toward(_reel, plight.staggered ? 1 : 0, 10, deltaSeconds);
    _struggle = _toward(
      _struggle,
      plight.reap > 0 && !plight.dying ? 1 : 0,
      12,
      deltaSeconds,
    );

    if (plight.dying) {
      _deathAge += deltaSeconds;
      _death = _toward(_death, 1, 16, deltaSeconds);
    } else {
      _deathAge = 0;
      _death = _toward(_death, 0, 6, deltaSeconds);
    }

    _reap = plight.reap.clamp(0.0, 1.0);
  }

  /// How afraid it is, from 0 to 1.
  double get fear => _fear;

  /// How furious it is, from 0 to 1: drawing breath to shriek, or charging.
  double get rage => _rage;

  /// How hard it is flinching from the last blow, from 0 to 1.
  double get wince => _wince;

  /// How far it shrinks down into itself, from 0 to 1.
  double get cower => _cower;

  /// How far it has gone in its death throes, from 0 to 1.
  double get death => _death;

  /// The body's tilt from blows, staggering and struggling, in radians:
  /// about the z axis ([leanX], positive tips the top towards +x) and the x
  /// axis ([leanZ], positive tips it towards +z).
  double get leanX =>
      _leanX.position + _wobble(1.3, 0.12) * _reel + _thrash(0.11) * _struggle;

  double get leanZ =>
      _leanZ.position +
      _wobble(1.7, 0.1) * _reel +
      _thrash(0.08) * _struggle +
      // In its death throes it arches back.
      0.45 * _arch;

  /// How far it is pulled over towards the reaper, in radians: dragged up
  /// the arc by the reap, fighting it all the way.
  double get drag => 0.4 * _reap * _struggle;

  /// The body's twist about the vertical, in radians.
  double get twist =>
      _twist.position + _wobble(2.1, 0.18) * _reel + _thrash(0.22) * _struggle;

  /// How much the body is squashed down (negative) or stretched (positive),
  /// as a share of its height.
  double get squash => _squash.position * 0.06 - 0.08 * _cower;

  /// How far it is lifted off where it stands, as a share of its height:
  /// the reap draws it up towards the blade.
  double get rise => 0.35 * _reap * _struggle + 0.18 * _death;

  /// The head's pitch on top of where it looks, in radians: thrown back
  /// by a blow (positive), tucked down when it cowers (negative), thrown
  /// back in its death scream.
  double get headPitch =>
      0.5 * _headBack.position -
      0.32 * _cower * (1 - _struggle) +
      0.5 * _struggle * (0.6 + 0.4 * _thrash(1)) +
      0.7 * _arch;

  /// The head's turn on top of where it looks, in radians: it shakes
  /// about in terror and thrashes while it is reaped.
  double get headYaw =>
      _tremble(0.03 + 0.09 * _fear * _fear) +
      _thrash(0.45) * _struggle +
      _wobble(2.6, 0.25) * _reel;

  /// How wide its jaw hangs, from 0 to 1, on top of a shriek: teeth chatter
  /// in fear, and it screams while it is reaped and as it dies.
  double get gape {
    final chatter =
        _fear * (1 - _struggle) * 0.16 * (0.5 + 0.5 * math.sin(_seconds * 41));

    return math
        .max(
          chatter,
          math.max(
                0.35 * _wince,
                0.75 * _struggle + 0.25 * _thrash(0.4).abs(),
              ) +
              _death,
        )
        .clamp(0.0, 1.0);
  }

  /// How far its limbs flail beyond their gait, in radians: they scrabble
  /// for purchase while it is reaped, and lock out straight as it dies.
  double get flail => _thrash(0.9) * _struggle;

  /// How far its limbs splay out stiff, in radians.
  double get splay => 0.55 * _arch + 0.25 * _cower;

  /// A tremble to lay on any angle, in radians.
  double get shiver => _tremble(0.012 + 0.05 * _fear * _fear + 0.08 * _death);

  /// How wide its eyes are, as a scale: wide in terror, squeezed shut by a
  /// blow, narrowed in fury, and blown wide as it dies.
  double get eyeWidth =>
      (1 + 0.55 * _fear - 0.2 * _rage + 0.7 * _death).clamp(0.4, 2.2);

  double get eyeHeight =>
      ((1 + 0.75 * _fear - 0.55 * _rage + 0.9 * _death) * (1 - 0.88 * _wince))
          .clamp(0.08, 2.4);

  /// The brows' roll, in radians: positive lifts the inner ends, as in
  /// fear; negative drives them down, as in fury or a wince.
  double get browTilt =>
      0.5 * _fear - 0.6 * _rage - 0.35 * _wince + 0.3 * _death;

  /// How far the brows are raised, as a share of the head's size.
  double get browLift =>
      0.05 * _fear - 0.04 * _rage - 0.035 * _wince + 0.04 * _death;

  /// How bright its eyes burn, as a multiple of their usual glow: they
  /// flicker in fear, blaze in fury and flare out as it dies.
  double get eyeGlow {
    final flicker =
        0.5 + 0.5 * math.sin(_seconds * 23 + math.sin(_seconds * 7));
    final flare = _death * math.exp(-2.2 * _deathAge) * 2.5;

    return (1 + 0.5 * _fear * flicker + 0.9 * _rage + flare) *
        (1 - 0.85 * _death * (1 - math.exp(-2.2 * _deathAge)));
  }

  /// The arch of the death throes: a hard first spasm that slackens.
  double get _arch =>
      _death *
      (0.75 + 0.25 * math.sin(_deathAge * 30) * math.exp(-3 * _deathAge));

  /// A fast, uneven shake of [amplitude]: incommensurate waves, so it
  /// never settles into a pattern.
  double _tremble(double amplitude) =>
      amplitude *
      (0.55 * math.sin(_seconds * 37) +
          0.3 * math.sin(_seconds * 53.3 + 1.1) +
          0.15 * math.sin(_seconds * 71.7 + 2.3));

  /// A slower, heavier lurch of [amplitude], for limbs and a body fighting
  /// to get free.
  double _thrash(double amplitude) =>
      amplitude *
      (0.6 * math.sin(_seconds * 13.1) + 0.4 * math.sin(_seconds * 19.7 + 0.7));

  /// A drunken sway of [amplitude] at about [rate] cycles a second.
  double _wobble(double rate, double amplitude) =>
      amplitude * math.sin(_seconds * rate * 2 * math.pi);

  static double _smooth(double x) => x * x * (3 - 2 * x);

  static double _toward(double from, double to, double rate, double dt) =>
      from + (to - from) * (1 - math.exp(-rate * dt));
}

/// A damped spring about zero, stepped in small enough steps to stay steady
/// at any frame rate.
final class _Spring {
  _Spring({required double frequency, required double damping})
    : _stiffness = math.pow(2 * math.pi * frequency, 2).toDouble(),
      _drag = 2 * damping * 2 * math.pi * frequency;

  final double _stiffness;
  final double _drag;
  double position = 0;
  double velocity = 0;

  static const _substep = 1 / 240;

  void step(double deltaSeconds) {
    var left = deltaSeconds;

    while (left > 0) {
      final dt = math.min(left, _substep);

      velocity += (-_stiffness * position - _drag * velocity) * dt;
      position += velocity * dt;
      left -= dt;
    }
  }
}
