import 'dart:typed_data';

import '../vec3.dart';
import 'difficulty.dart';
import 'hunt_plan.dart';

/// The encoded log belongs to a different combat-rules deployment.
final class UnsupportedHuntVersion extends FormatException {
  const UnsupportedHuntVersion() : super('Unsupported combat rules');
}

/// A direction stored as whole hundredths of a degree.
///
/// The simulation only ever sees directions in this form. Whole numbers are
/// what make a hunt replay exactly: the recorded log holds the very values
/// the phone played with, with nothing lost to rounding in between.
final class Aim {
  const Aim(this.yawCentidegrees, this.pitchCentidegrees);

  /// Rounds a direction onto the hundredth-of-a-degree grid.
  factory Aim.fromDirection(Vec3 direction) => Aim(
    (degrees(direction.yaw) * 100).round(),
    (degrees(direction.pitch) * 100).round().clamp(-9000, 9000),
  );

  static const ahead = Aim(0, 0);

  /// Hundredths of a degree to the right of straight ahead, in
  /// `[-18000, 18000]`.
  final int yawCentidegrees;

  /// Hundredths of a degree above the horizon, in `[-9000, 9000]`.
  final int pitchCentidegrees;

  Vec3 get direction => Vec3.fromAngles(
    radians(yawCentidegrees / 100),
    radians(pitchCentidegrees / 100),
  );

  @override
  bool operator ==(Object other) =>
      other is Aim &&
      other.yawCentidegrees == yawCentidegrees &&
      other.pitchCentidegrees == pitchCentidegrees;

  @override
  int get hashCode => Object.hash(yawCentidegrees, pitchCentidegrees);

  @override
  String toString() => 'Aim($yawCentidegrees, $pitchCentidegrees)';
}

/// Everything the player did during one tick.
final class HuntInput {
  const HuntInput({
    required this.aim,
    required this.turnSpeed,
    this.taps = const [],
  });

  /// Rounds what the phone measured this tick onto the grid the rules use.
  factory HuntInput.measured({
    required Vec3 aim,
    required double turnRadiansPerSecond,
    List<Vec3> taps = const [],
  }) => HuntInput(
    aim: Aim.fromDirection(aim),
    turnSpeed: (turnRadiansPerSecond * 1000).round().clamp(0, maxTurnSpeed),
    taps: [for (final tap in taps) Aim.fromDirection(tap)],
  );

  /// The speed, in thousandths of a radian a second, above which a recorded
  /// turn speed is clamped.
  static const maxTurnSpeed = 0xFFFF;

  /// Where the middle of the phone's view points.
  final Aim aim;

  /// How fast the phone is turning, in thousandths of a radian a second.
  final int turnSpeed;

  /// Where each tap this tick points, as a direction out from the player.
  final List<Aim> taps;

  /// [turnSpeed] in radians a second.
  double get turnRadiansPerSecond => turnSpeed / 1000;
}

/// The complete input of one hunt, tick by tick.
///
/// A phone records the log as it plays and sends it with the hunt's result.
/// The server feeds the same log to the same rules and so reaches its own
/// verdict; it never takes the phone's word for what happened.
final class HuntLog {
  HuntLog(
    this.inputs, {
    this.plan = HuntPlan.open,
    this.difficulty = Difficulty.normal,
  });

  /// Reads a log from the bytes written by [encode].
  ///
  /// Throws a [FormatException] when the bytes are not a whole log.
  factory HuntLog.decode(Uint8List bytes) {
    if (bytes.isEmpty) {
      throw const FormatException('Not a hunt log');
    }

    if (bytes.length < _headerBytes) {
      throw const FormatException('Hunt log has no plan');
    }

    if (bytes.first != _version) throw const UnsupportedHuntVersion();

    final data = ByteData.sublistView(bytes);
    final reachCount = data.getUint8(6);
    final difficulty = data.getUint8(7);

    if (difficulty >= Difficulty.values.length) {
      throw const FormatException('Hunt log has no such difficulty');
    }

    if (reachCount != 0 && reachCount != HuntPlan.slices) {
      throw const FormatException('Hunt log has a broken plan');
    }

    if (bytes.length < _headerBytes + reachCount) {
      throw const FormatException('Hunt log ends mid-plan');
    }

    final plan = HuntPlan(
      centreCentidegrees: data.getInt16(1, Endian.little),
      halfWidthCentidegrees: data.getUint16(3, Endian.little),
      eyeCentimetres: data.getUint8(5),
      reachDecimetres: bytes.sublist(_headerBytes, _headerBytes + reachCount),
    );
    final inputs = <HuntInput>[];
    var offset = _headerBytes + reachCount;

    while (offset < bytes.length) {
      if (inputs.length >= maxTicks) {
        throw const FormatException('Hunt log exceeds the encounter limit');
      }
      if (offset + _tickBytes > bytes.length) {
        throw const FormatException('Hunt log ends mid-tick');
      }

      final aim = Aim(
        data.getInt16(offset, Endian.little),
        data.getInt16(offset + 2, Endian.little),
      );
      final turnSpeed = data.getUint16(offset + 4, Endian.little);
      final tapCount = data.getUint8(offset + 6);

      if (!_validAim(aim)) throw const FormatException('Impossible hunt aim');
      offset += _tickBytes;

      if (offset + tapCount * _tapBytes > bytes.length) {
        throw const FormatException('Hunt log ends mid-tap');
      }

      final taps = [
        for (var tap = 0; tap < tapCount; tap++)
          Aim(
            data.getInt16(offset + tap * _tapBytes, Endian.little),
            data.getInt16(offset + tap * _tapBytes + 2, Endian.little),
          ),
      ];

      if (taps.any((tap) => !_validAim(tap))) {
        throw const FormatException('Impossible tap aim');
      }
      offset += tapCount * _tapBytes;

      inputs.add(HuntInput(aim: aim, turnSpeed: turnSpeed, taps: taps));
    }

    return HuntLog(
      inputs,
      plan: plan,
      difficulty: Difficulty.values[difficulty],
    );
  }

  /// The longest log the server accepts: the longest possible hunt, with
  /// room for the stalk, the fight and every reap window.
  static const maxTicks = 300 * 60;

  // The birth-gift rules change replay results. Refuse old clients rather
  // than silently reinterpret a version-four hunt under the new rules.
  static const _version = 5;
  static const _headerBytes = 8;
  static const _tickBytes = 7;
  static const _tapBytes = 4;

  static bool _validAim(Aim aim) =>
      aim.yawCentidegrees >= -18000 &&
      aim.yawCentidegrees <= 18000 &&
      aim.pitchCentidegrees >= -9000 &&
      aim.pitchCentidegrees <= 9000;

  /// One entry for every tick the hunt ran.
  final List<HuntInput> inputs;

  /// Where the husk was free to go. It is part of the record because the
  /// husk's every move depends on it.
  final HuntPlan plan;

  /// How forgiving the hunt was. Part of the record for the same reason.
  final Difficulty difficulty;

  /// Packs the log into bytes: the plan first, then seven for each tick and
  /// four more for each tap.
  ///
  /// The plan is its arc, the height of the player's eye, and how far the
  /// way is clear along each slice of the room when that was measured.
  Uint8List encode() {
    final header = ByteData(_headerBytes)
      ..setUint8(0, _version)
      ..setInt16(1, plan.centreCentidegrees, Endian.little)
      ..setUint16(3, plan.halfWidthCentidegrees, Endian.little)
      ..setUint8(5, plan.eyeCentimetres)
      ..setUint8(6, plan.reachDecimetres.length)
      ..setUint8(7, difficulty.index);
    final builder = BytesBuilder(copy: false)
      ..add(header.buffer.asUint8List())
      ..add(plan.reachDecimetres);

    for (final input in inputs) {
      final taps = input.taps.length > 255
          ? input.taps.sublist(0, 255)
          : input.taps;
      final record = ByteData(_tickBytes + taps.length * _tapBytes)
        ..setInt16(0, input.aim.yawCentidegrees, Endian.little)
        ..setInt16(2, input.aim.pitchCentidegrees, Endian.little)
        ..setUint16(
          4,
          input.turnSpeed.clamp(0, HuntInput.maxTurnSpeed),
          Endian.little,
        )
        ..setUint8(6, taps.length);

      for (var tap = 0; tap < taps.length; tap++) {
        record
          ..setInt16(
            _tickBytes + tap * _tapBytes,
            taps[tap].yawCentidegrees,
            Endian.little,
          )
          ..setInt16(
            _tickBytes + tap * _tapBytes + 2,
            taps[tap].pitchCentidegrees,
            Endian.little,
          );
      }

      builder.add(record.buffer.asUint8List());
    }

    return builder.takeBytes();
  }
}
