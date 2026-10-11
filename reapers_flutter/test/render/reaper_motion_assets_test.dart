import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_shared/reapers_shared.dart';

Map<String, Object?> _object(Object? value) => value as Map<String, Object?>;
List<Object?> _list(Object? value) => value as List<Object?>;

/// Reads the committed payload, so a successful script cannot mask a stale GLB.
final class _Glb {
  _Glb(String path) {
    final bytes = File(path).readAsBytesSync();
    final header = ByteData.sublistView(bytes);
    expect(header.getUint32(0, Endian.little), 0x46546c67);
    final jsonLength = header.getUint32(12, Endian.little);
    json = _object(jsonDecode(utf8.decode(bytes.sublist(20, 20 + jsonLength))));
    binary = ByteData.sublistView(bytes, 28 + jsonLength);
  }
  late final Map<String, Object?> json;
  late final ByteData binary;

  List<double> floats(int index) {
    final accessor = _object(_list(json['accessors'])[index]);
    expect(accessor['componentType'], 5126);
    final view = _object(
      _list(json['bufferViews'])[accessor['bufferView'] as int],
    );
    final offset =
        (view['byteOffset'] as int? ?? 0) +
        (accessor['byteOffset'] as int? ?? 0);
    final size = switch (accessor['type']) {
      'SCALAR' => 1,
      'VEC3' => 3,
      'VEC4' => 4,
      _ => throw StateError('Unexpected track type'),
    };
    final stride = view['byteStride'] as int? ?? size * 4;
    return [
      for (var i = 0; i < (accessor['count'] as int); i++)
        for (var component = 0; component < size; component++)
          binary.getFloat32(offset + i * stride + component * 4, Endian.little),
    ];
  }
}

void main() {
  for (final calling in ReaperClass.values) {
    test(
      '${calling.name} exports the complete playable rig and 30 finite clips',
      () {
        final glb = _Glb('assets/models/reaper_${calling.name}.glb');
        final nodes = _list(glb.json['nodes']).map(_object).toList();
        final names = nodes.map((node) => node['name']).toSet();
        const joints = {
          'motion',
          'head',
          'arm_r',
          'arm_l',
          'elbow_r',
          'elbow_l',
          'wrist_r',
          'wrist_l',
        };
        expect(names, containsAll({...joints, 'socket_weapon'}));
        final animations = _list(glb.json['animations']).map(_object).toList();
        expect(
          animations.map((clip) => clip['name']),
          unorderedEquals(ReaperMotion.values.map((motion) => motion.name)),
        );
        for (final clip in animations) {
          final motion = ReaperMotion.values.byName(clip['name'] as String);
          final samplers = _list(clip['samplers']).map(_object).toList();
          final channels = _list(clip['channels']).map(_object);
          final animated = <String>{};
          var moves = false;
          for (final channel in channels) {
            final target = _object(channel['target']);
            animated.add(nodes[target['node'] as int]['name'] as String);
            final sampler = samplers[channel['sampler'] as int];
            final times = glb.floats(sampler['input'] as int);
            final values = glb.floats(sampler['output'] as int);
            expect(times.first, closeTo(0, 1e-5));
            expect(
              times.last,
              closeTo(motion.seconds, 1e-5),
              reason: motion.name,
            );
            for (var i = 1; i < times.length; i++) {
              expect(times[i], greaterThan(times[i - 1]));
            }
            expect(values.every((v) => v.isFinite), isTrue);
            final width = values.length ~/ times.length;
            for (var i = width; i < values.length; i++) {
              if ((values[i] - values[i % width]).abs() > .001) moves = true;
            }
            if (motion.loops) {
              for (var i = 0; i < width; i++) {
                expect(
                  values[i],
                  closeTo(values[values.length - width + i], 1e-5),
                  reason: '${motion.name} loop seam',
                );
              }
            }
          }
          expect(
            animated,
            containsAll(joints),
            reason: '${motion.name} must reset every joint',
          );
          expect(moves, isTrue, reason: '${motion.name} must move');
        }
      },
    );
  }
}
