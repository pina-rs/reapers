import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:reapers_flutter/render/reaper_appearance.dart';
import 'package:vector_math/vector_math.dart' as vm;

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

  /// Evaluate the exported hierarchy, including animated parents, at one sample.
  vm.Matrix4 world(String nodeName, String clipName, double phase) {
    final nodes = _list(json['nodes']).map(_object).toList();
    final clip = _list(json['animations'])
        .map(_object)
        .singleWhere(
          (clip) => clip['name'] == clipName,
        );
    final samplers = _list(clip['samplers']).map(_object).toList();
    final poses = <int, Map<String, List<double>>>{};
    for (final raw in _list(clip['channels'])) {
      final channel = _object(raw);
      final target = _object(channel['target']);
      final path = target['path'] as String;
      if (path == 'weights') continue;
      final sampler = samplers[channel['sampler'] as int];
      final times = floats(sampler['input'] as int);
      final values = floats(sampler['output'] as int);
      final width = values.length ~/ times.length;
      final index = (phase * (times.length - 1)).round();
      (poses[target['node'] as int] ??= {})[path] = values.sublist(
        index * width,
        (index + 1) * width,
      );
    }
    vm.Matrix4 transform(int index) {
      final node = nodes[index];
      List<double> values(String name, List<double> fallback) =>
          poses[index]?[name] ??
          (node[name] as List<Object?>?)
              ?.map((v) => (v as num).toDouble())
              .toList() ??
          fallback;
      final t = values('translation', [0, 0, 0]);
      final r = values('rotation', [0, 0, 0, 1]);
      final s = values('scale', [1, 1, 1]);
      final local = vm.Matrix4.compose(
        vm.Vector3.array(t),
        vm.Quaternion(r[0], r[1], r[2], r[3]),
        vm.Vector3.array(s),
      );
      final parent = nodes.indexWhere(
        (node) => (node['children'] as List<Object?>? ?? []).contains(index),
      );
      return parent < 0 ? local : transform(parent) * local;
    }

    return transform(nodes.indexWhere((node) => node['name'] == nodeName));
  }
}

void main() {
  for (final calling in ReaperClass.values) {
    test(
      '${calling.name} exports the complete playable rig and finite clips',
      () {
        final glb = _Glb('assets/models/reaper_${calling.name}.glb');
        final nodes = _list(glb.json['nodes']).map(_object).toList();
        final names = nodes.map((node) => node['name']).toSet();
        const joints = {
          'motion',
          'spine',
          'head',
          'arm_r',
          'arm_l',
          'elbow_r',
          'elbow_l',
          'wrist_r',
          'wrist_l',
          'leg_l',
          'leg_r',
          'knee_l',
          'knee_r',
          'foot_l',
          'foot_r',
        };
        expect(names, containsAll({...joints, 'socket_weapon'}));
        expect(
          names,
          containsAll({'spirit_body', 'spirit_form', 'humanoid_form'}),
        );
        final faces = {
          for (var i = 0; i < (calling == ReaperClass.hollow ? 1 : 3); i++)
            'face_$i',
        };
        expect(names, containsAll(faces));
        final animations = _list(glb.json['animations']).map(_object).toList();
        expect(
          animations.map((clip) => clip['name']),
          unorderedEquals([
            for (final form in ReaperForm.values)
              for (final motion in ReaperMotion.values)
                ReaperAppearance(form: form).clipName(motion),
          ]),
        );
        for (final clip in animations) {
          final clipName = clip['name'] as String;
          final motion = ReaperMotion.values.byName(
            clipName.replaceFirst('humanoid_', ''),
          );
          final samplers = _list(clip['samplers']).map(_object).toList();
          final channels = _list(clip['channels']).map(_object);
          final animated = <String>{};
          final animatedFaces = <String>{};
          var expressionMoves = false;
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
            if (target['path'] == 'weights' &&
                faces.contains(nodes[target['node'] as int]['name'])) {
              animatedFaces.add(nodes[target['node'] as int]['name'] as String);
              expressionMoves =
                  expressionMoves || values.any((value) => value > 0.2);
              expect(values, everyElement(inInclusiveRange(0, 1)));
              for (var i = 0; i < width; i++) {
                expect(values[i], closeTo(0, 1e-5));
                expect(values[values.length - width + i], closeTo(0, 1e-5));
              }
            }
            if ((motion == ReaperMotion.dashLeft ||
                    motion == ReaperMotion.dashRight) &&
                nodes[target['node'] as int]['name'] == 'motion' &&
                target['path'] == 'translation') {
              // glTF preserves X: positive is the reaper's left, viewed
              // from behind. The studio excursion must also return to rest.
              final side = motion == ReaperMotion.dashLeft ? 1 : -1;
              final lateral = [
                for (var i = 0; i < values.length; i += width)
                  (values[i] - values.first) * side,
              ];
              expect(lateral.any((x) => x > .9), isTrue);
              expect(lateral.any((x) => x < -.05), isTrue);
              for (var i = 0; i < width; i++) {
                expect(
                  values[values.length - width + i],
                  closeTo(values[i], 1e-5),
                );
              }
            }
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
          expect(
            animatedFaces,
            faces,
            reason: '${motion.name} resets every face',
          );
          if (motion.group == MotionGroup.reaction ||
              motion.group == MotionGroup.meme) {
            expect(
              expressionMoves,
              isTrue,
              reason: '${motion.name} has an expression',
            );
          }
        }
      },
    );
    test(
      '${calling.name} humanoid walks with grounded support and forward knees',
      () {
        final glb = _Glb('assets/models/reaper_${calling.name}.glb');
        vm.Vector3 at(String node, double phase) =>
            glb.world(node, 'humanoid_walk', phase).getTranslation();
        for (final phase in [.2, .3, .4]) {
          final hip = at('leg_l', phase);
          final knee = at('knee_l', phase);
          final ankle = at('foot_l', phase);
          expect((hip - knee).length, closeTo(.44, .002));
          expect((knee - ankle).length, closeTo(.44, .002));
          expect(
            ankle.y,
            closeTo(.13, .012),
            reason: 'support foot stays on the ground',
          );
          expect(
            knee.z,
            greaterThan((hip.z + ankle.z) / 2),
            reason: 'knees bend forward',
          );
        }
        expect(
          at('foot_r', .3).y,
          greaterThan(.18),
          reason: 'swing foot clears the floor',
        );
      },
    );
  }
}
