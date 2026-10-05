import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/render/reaper_look.dart';
import 'package:reapers_flutter/render/reaper_web_material.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:three_js/three_js.dart' as three;

/// Shader preparation does not need a GPU; the browser journey checks drawing.
class _Renderer extends Fake implements three.Renderer {}

void main() {
  final source = File('assets/materials/reaper.fmat').readAsStringSync();
  final genome = ReaperGenome.shuffle(42);

  three.Parameters shader() => three.Parameters(
    precision: 'highp',
    vertexShader: three.meshphysicalVert,
    fragmentShader: three.meshphysicalFrag,
  )..uniforms = {};

  test('authored surface binds to the real WebGL physical shader', () {
    final material = ReaperWebMaterial(
      'cloth',
      source,
      ReaperLook.of(genome, Outfit.shrouded),
    );
    final prepared = shader();
    material.onBeforeCompile!(prepared, _Renderer());

    expect(prepared.vertexShader, contains('vReaperMask = reaperMask;'));
    expect(prepared.fragmentShader, contains('Surface(reaperInputs);'));
    expect(
      prepared.fragmentShader,
      contains('if (keep < hollow_front) discard;'),
    );
    expect(prepared.fragmentShader, isNot(contains('material_params.')));
    expect(prepared.fragmentShader, isNot(contains('GetVertexColor()')));
    expect(prepared.fragmentShader, isNot(contains('PrepareMaterial(')));

    // Native source_color uniforms convert to linear light automatically.
    final tint = prepared.uniforms!['uReaper_tint']['value'] as three.Vector4;
    final colour = ReaperLook.of(
      genome,
      Outfit.shrouded,
    ).surfaceFor('cloth').tint;
    expect(tint.x, closeTo(three.Color.srgbToLinear(colour.r), 0.000001));
    expect(tint.y, closeTo(three.Color.srgbToLinear(colour.g), 0.000001));
    expect(tint.z, closeTo(three.Color.srgbToLinear(colour.b), 0.000001));
  });

  test(
    'upgrades update already bound uniforms while bone keeps its surface',
    () {
      final initial = ReaperLook.of(genome, Outfit.shrouded);
      final upgraded = ReaperLook.of(genome, Outfit.hollowed);

      for (final surface in ['cloth', 'bone']) {
        final material = ReaperWebMaterial(surface, source, initial);
        final prepared = shader();
        material.onBeforeCompile!(prepared, _Renderer());
        final hollow = prepared.uniforms!['uReaper_hollow_level'];
        material.update(upgraded);

        expect(prepared.uniforms!['uReaper_hollow_level'], same(hollow));
        expect(
          hollow['value'],
          surface == 'cloth' ? upgraded.hollowLevel : 0.0,
        );
        expect(
          prepared.uniforms!['uReaper_char_level']['value'],
          surface == 'cloth' ? upgraded.charLevel : 0.0,
        );
      }
    },
  );

  test(
    'missing authored surface or changed renderer slots fail explicitly',
    () {
      expect(() => webReaperSurface('material {}'), throwsFormatException);

      final material = ReaperWebMaterial(
        'cloth',
        source,
        ReaperLook.of(genome, Outfit.shrouded),
      );
      final changed = shader()
        ..fragmentShader = three.meshphysicalFrag.replaceFirst(
          '#include <color_fragment>',
          '',
        );
      expect(
        () => material.onBeforeCompile!(changed, _Renderer()),
        throwsFormatException,
      );
    },
  );
}
