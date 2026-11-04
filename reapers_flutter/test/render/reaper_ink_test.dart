import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/render/reaper_look.dart';
import 'package:reapers_flutter/render/reaper_web_material.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:three_js/three_js.dart' as three;

/// The reaper's hand-drawn edge is authored once in the material and reused
/// by the browser. These tests hold the two halves together without a GPU.
class _Renderer extends Fake implements three.Renderer {}

void main() {
  final source = File('assets/materials/reaper.fmat').readAsStringSync();

  three.Parameters shader() => three.Parameters(
    precision: 'highp',
    vertexShader: three.meshphysicalVert,
    fragmentShader: three.meshphysicalFrag,
  )..uniforms = {};

  test('the reaper material declares ink that does not read the clock', () {
    for (final parameter in ['ink', 'ink_strength', 'ink_seed']) {
      expect(source, contains('name: $parameter'), reason: parameter);
    }

    final start = source.indexOf('float ink_width');
    final end = source.indexOf('material.base_color', start);

    expect(start, greaterThan(0));
    expect(end, greaterThan(start));
    // A hand-drawn line that depends on time would crawl every frame. The
    // edge is fixed to the rest position instead.
    expect(source.substring(start, end), isNot(contains('GetTime()')));
  });

  test('each genome fixes its own ink seed, and it never drifts', () {
    final seeds = {
      for (var seed = 0; seed < 12; seed++)
        ReaperLook.of(
          ReaperGenome.shuffle(seed),
          Outfit.shrouded,
        ).inkSeed,
    };

    expect(seeds.length, greaterThan(1));

    final first = ReaperLook.of(ReaperGenome.shuffle(42), Outfit.shrouded);
    final second = ReaperLook.of(ReaperGenome.shuffle(42), Outfit.shrouded);

    expect(first.inkSeed, second.inkSeed);
    expect(first.inkSeed, inInclusiveRange(0, 8));
    expect(first.inkStrength, greaterThan(0));
  });

  test('the browser binds the same ink and breaks the line at the surface', () {
    final look = ReaperLook.of(ReaperGenome.shuffle(42), Outfit.shrouded);
    final material = ReaperWebMaterial('cloth', source, look);
    final prepared = shader();
    material.onBeforeCompile!(prepared, _Renderer());

    expect(prepared.uniforms!['uReaper_ink'], isNotNull);
    expect(
      prepared.uniforms!['uReaper_ink_strength']['value'],
      look.inkStrength,
    );
    expect(prepared.uniforms!['uReaper_ink_seed']['value'], look.inkSeed);
    expect(prepared.fragmentShader, contains('uniform vec4 uReaper_ink;'));
    expect(
      prepared.fragmentShader,
      contains('uniform float uReaper_ink_seed;'),
    );
    expect(prepared.fragmentShader, contains('float ink_edge = smoothstep'));
    expect(
      prepared.fragmentShader,
      isNot(contains('material_params.')),
    );

    final ink = prepared.uniforms!['uReaper_ink']['value'] as three.Vector4;
    expect(ink.x, closeTo(three.Color.srgbToLinear(look.ink.r), 0.000001));
    expect(ink.y, closeTo(three.Color.srgbToLinear(look.ink.g), 0.000001));
    expect(ink.z, closeTo(three.Color.srgbToLinear(look.ink.b), 0.000001));
  });
}
