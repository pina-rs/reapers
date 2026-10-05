import 'dart:ui';

import 'package:three_js/three_js.dart' as three;

import 'reaper_look.dart';

/// Adapts the authored mobile surface to WebGL's material slots. The burn,
/// hollowing, grime, grain and ember arithmetic stays in reaper.fmat.
String webReaperSurface(String source) {
  final match = RegExp(r'fragment\s*\{([\s\S]*)\}\s*$').firstMatch(source);
  if (match == null) throw const FormatException('Missing reaper surface');

  return match
      .group(1)!
      .replaceAll('material_params.', 'uReaper_')
      .replaceAll('GetVertexColor()', 'vReaperMask')
      .replaceAll(
        'GetWorldNormal()',
        '(gl_FrontFacing ? normalize(vNormal) : -normalize(vNormal))',
      )
      .replaceAll('GetViewDirection()', 'normalize(vViewPosition)')
      .replaceAll('GetUV0()', 'vReaperRest.xy')
      .replaceAll('GetUV1().x', 'vReaperRest.z')
      .replaceAll('GetTime()', 'uReaper_time')
      .replaceAll('PrepareMaterial(material);', '');
}

/// One material per mesh, with stable uniform objects for cheap outfit changes.
final class ReaperWebMaterial extends three.MeshStandardMaterial {
  ReaperWebMaterial(String surfaceName, String source, ReaperLook look)
    : super.fromMap({'name': surfaceName, 'side': three.DoubleSide}) {
    update(look);
    time = 0;
    customProgramCacheKey = () => 'reaper-surface-v1';
    onBeforeCompile = (three.Parameters shader, three.Renderer renderer) {
      if (!shader.vertexShader.contains('void main() {')) {
        throw const FormatException('Missing WebGL vertex entry point');
      }
      for (final marker in [
        'void main() {',
        '#include <color_fragment>',
        '#include <roughnessmap_fragment>',
        '#include <metalnessmap_fragment>',
        '#include <emissivemap_fragment>',
      ]) {
        if (!shader.fragmentShader.contains(marker)) {
          throw FormatException('Missing WebGL surface slot: $marker');
        }
      }
      shader.uniforms!.addAll(_uniforms);
      shader.vertexShader = '$_vertexDeclarations\n${shader.vertexShader}'
          .replaceFirst('void main() {', '''void main() {
            vReaperMask = reaperMask;
            vReaperRest = reaperRest;
          ''');
      shader.fragmentShader = shader.fragmentShader
          .replaceFirst('void main() {', '''$_fragmentDeclarations
            ${webReaperSurface(source)}
            void main() {
          ''')
          .replaceFirst('#include <color_fragment>', '''
            MaterialInputs reaperInputs;
            Surface(reaperInputs);
            diffuseColor = reaperInputs.base_color;
          ''')
          .replaceFirst('#include <roughnessmap_fragment>', '''
            #include <roughnessmap_fragment>
            roughnessFactor = reaperInputs.roughness;
          ''')
          .replaceFirst('#include <metalnessmap_fragment>', '''
            #include <metalnessmap_fragment>
            metalnessFactor = reaperInputs.metallic;
          ''')
          .replaceFirst('#include <emissivemap_fragment>', '''
            #include <emissivemap_fragment>
            totalEmissiveRadiance = reaperInputs.emissive;
          ''');
    };
  }

  final _uniforms = <String, Map<String, Object>>{};

  void _set(String key, Object value) =>
      (_uniforms['uReaper_$key'] ??= {})['value'] = value;

  set time(double value) => _set('time', value);

  void update(ReaperLook look) {
    final surface = look.surfaceFor(name);
    _set('tint', _color(surface.tint));
    _set('surface_roughness', surface.roughness);
    _set('surface_metallic', surface.metallic);
    _set('grime', surface.grime);
    _set('wear', surface.wear);
    _set('weave', surface.weave);
    _set('char_level', surface.burns ? look.charLevel : 0.0);
    _set('char_tint', three.Vector4(0.03, 0.028, 0.03, 1));
    _set('ember', _color(look.ember));
    _set('ember_strength', surface.burns ? look.emberStrength : 0.0);
    _set('hollow_level', surface.burns ? look.hollowLevel : 0.0);
    _set('rim', _color(look.rim));
    _set('rim_strength', surface.grime == 0 ? 0.0 : look.rimStrength);
    _set('glow', _color(look.accent));
    _set('glow_strength', surface.glowStrength);
  }

  // Mobile's source_color hint makes this conversion at material binding.
  // WebGL works in linear light; supplying sRGB directly washes out the dyes.
  static three.Vector4 _color(Color color) => three.Vector4(
    three.Color.srgbToLinear(color.r),
    three.Color.srgbToLinear(color.g),
    three.Color.srgbToLinear(color.b),
    color.a,
  );
}

const _vertexDeclarations = '''
attribute vec4 reaperMask;
attribute vec3 reaperRest;
varying vec4 vReaperMask;
varying vec3 vReaperRest;
''';

const _fragmentDeclarations = '''
varying vec4 vReaperMask;
varying vec3 vReaperRest;
uniform vec4 uReaper_tint;
uniform float uReaper_surface_roughness;
uniform float uReaper_surface_metallic;
uniform float uReaper_grime;
uniform float uReaper_wear;
uniform float uReaper_weave;
uniform float uReaper_char_level;
uniform vec4 uReaper_char_tint;
uniform vec4 uReaper_ember;
uniform float uReaper_ember_strength;
uniform float uReaper_hollow_level;
uniform vec4 uReaper_rim;
uniform float uReaper_rim_strength;
uniform vec4 uReaper_glow;
uniform float uReaper_glow_strength;
uniform float uReaper_time;
struct MaterialInputs {
  vec4 base_color;
  vec3 normal;
  float metallic;
  float roughness;
  float occlusion;
  vec3 emissive;
};
''';
