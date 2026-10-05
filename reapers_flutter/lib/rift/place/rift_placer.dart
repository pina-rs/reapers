import 'package:reapers_shared/reapers_shared.dart';

/// Works out, frame by frame, where a rift stands for one reaper.
///
/// A rift stands at one place on the earth for its whole circle. The first
/// reaper whose phone knows where it is tells the rift, and the rift takes
/// its place from that. After that every phone works out for itself which
/// way that place lies and points its reaper at it, so people standing
/// together are turned the same way.
///
/// How well that holds depends on the phone: a fix is good to a few metres
/// out of doors and worse inside, and a compass to ten degrees or so, less
/// near steel. It is enough to face the same way, not to touch the same
/// stone. See docs/gameplay.md.
final class RiftPlacer {
  RiftPlacer({DateTime Function()? now}) : _now = now ?? DateTime.now;

  final DateTime Function() _now;

  /// Frames between one telling of where this reaper stands and the next,
  /// while the rift has yet to take its place. About half a second.
  static const tellEveryFrames = 10;

  /// How far the sighting moves, each frame, toward where the rift is now
  /// seen. Where a phone thinks it is shifts by a few metres from one
  /// second to the next, and the husk should not jump with every guess.
  static const easing = 0.08;

  /// Frames a phone is given to find itself before it is taken not to know
  /// where it is. About eight seconds.
  static const framesToFindItself = 160;

  /// Where the phone last said it was.
  GeoFix? fix;

  /// Where this reaper sees the rift, or null while it stands dead ahead:
  /// no place taken yet, nothing known of where the phone is, or the rift
  /// too far off to point at.
  RiftSighting? sighting;

  int _framesSinceTold = tellEveryFrames;
  int _framesWaited = 0;

  /// Whether the phone is still being given time to find itself.
  bool get stillLooking => fix == null && _framesWaited < framesToFindItself;

  /// Takes in [frame], with the phone facing [facing] and north lying at
  /// [north] in its view, both as bearings in the hunt's space.
  ///
  /// Answers what to tell the rift about where this reaper stands, or null
  /// when there is nothing to tell: the rift already has its place, the
  /// phone does not know where it is, or it was told a moment ago.
  GeoStanding? observe(
    RiftFrame frame, {
    required double facing,
    required double? north,
  }) {
    final measured = fix;
    final here = measured != null && measured.isUsableAt(_now())
        ? measured
        : null;

    _framesWaited++;

    if (here == null || north == null) {
      sighting = null;

      return null;
    }

    _see(frame.anchor, here, north);

    final unplaced = frame.phase == RiftPhase.gathering && frame.anchor == null;

    if (!unplaced || ++_framesSinceTold < tellEveryFrames) {
      return null;
    }

    _framesSinceTold = 0;

    return GeoStanding(
      at: here.at,
      facing: wrapAngle(facing - north),
      accuracy: here.accuracy,
      observedAt: here.observedAt,
    );
  }

  void _see(GeoPoint? anchor, GeoFix here, double north) {
    final seen = anchor == null
        ? null
        : RiftSighting.of(anchor: anchor, standing: here.at, north: north);
    final before = sighting;

    sighting = seen == null || before == null
        ? seen
        : RiftSighting(
            bearing: wrapAngle(
              before.bearing +
                  wrapAngle(seen.bearing - before.bearing) * easing,
            ),
            metres: before.metres + (seen.metres - before.metres) * easing,
          );
  }
}
