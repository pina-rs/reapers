import '../arts.dart';
import '../vec3.dart';

/// A thrown art on its way.
///
/// The rules decide when it lands and what it does there. This is also all
/// the view needs to draw it in flight: where it left the hand, where it is
/// going, and how far along it is.
final class ArtFlight {
  const ArtFlight({
    required this.art,
    required this.from,
    required this.target,
    required this.homing,
    required this.thrownAt,
    required this.ticks,
  });

  final Art art;

  /// Where it left the reaper's hand, in metres from the eye.
  final Vec3 from;

  /// Where it was thrown, in metres from the eye. A homing art goes to
  /// wherever the husk is instead.
  final Vec3 target;

  /// Whether it follows the husk.
  final bool homing;

  /// The tick it was thrown on.
  final int thrownAt;

  /// How many ticks it is in the air.
  final int ticks;

  /// The tick it lands on.
  int get landsAt => thrownAt + ticks;

  /// How far along its flight it is at [tick], from 0 to 1.
  double progress(int tick) => ((tick - thrownAt) / ticks).clamp(0.0, 1.0);

  /// Where it is at [tick], going to [goal]: the husk it follows, or the
  /// place it was thrown. Fire is lobbed, so it rises and falls; the others
  /// fly straight.
  Vec3 position(int tick, Vec3 goal) {
    final along = progress(tick);
    final straight = from + (goal - from) * along;

    if (art != Art.cinder) {
      return straight;
    }

    final lob =
        ArtRules.lobMetres + ArtRules.lobPerMetre * (goal - from).length;

    return straight + Vec3.up * (lob * 4 * along * (1 - along));
  }
}
