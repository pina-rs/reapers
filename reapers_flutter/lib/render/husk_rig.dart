import 'dart:math' as math;
import 'dart:ui';

import 'package:flutter_scene/scene.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:vector_math/vector_math.dart' as vm;

import 'nft_image.dart';

/// The shader for a husk's solid surfaces.
const huskMaterialAsset = 'assets/materials/husk.fmat';

/// The shader for a husk's cloth and smoke.
const spectreMaterialAsset = 'assets/materials/spectre.fmat';

/// The model of one vessel.
String huskModelAsset(Vessel vessel) => 'assets/models/husk_${vessel.name}.glb';

/// One husk: a vessel wearing an NFT's image.
///
/// The rig only ever draws what the rules say. It is told where the husk is
/// with [place], and told what happened with [flash] and [dissolve]; it
/// decides nothing about the hunt.
final class HuskRig {
  HuskRig._({
    required this.root,
    required this.vessel,
    required this.light,
    required this._model,
    required this._solid,
    required this._spectral,
  });

  /// Loads [vessel] and dresses it in [pictures].
  ///
  /// Most husks wear one picture all over. A body built from several parts
  /// the art build numbered (`cube_0`, `cube_1` and so on) gives each part
  /// the next picture in turn, going round again if there are more parts
  /// than pictures.
  ///
  /// `Scene.initializeStaticResources` must have completed first.
  static Future<HuskRig> load({
    required Vessel vessel,
    required List<NftImage> pictures,
  }) async {
    final model = await loadScene(huskModelAsset(vessel));
    final light = pictures.first.light;
    final textures = [
      for (final picture in pictures) await Texture2D.fromImage(picture.image),
    ];
    final solid = <String, PreprocessedMaterial>{};
    final spectral = <PreprocessedMaterial>[];
    final primitives = <(MeshPrimitive, int)>[];

    _visit(model, 0, (node, part) {
      final mesh = node.mesh;

      if (mesh == null) {
        return;
      }

      for (final primitive in mesh.primitives) {
        primitives.add((primitive, part));
      }
    });

    for (final (primitive, part) in primitives) {
      final name = primitive.material.name.split('.').first;

      if (name == 'shroud') {
        final material = await loadFmatMaterial(spectreMaterialAsset);

        material.parameters.setColor('tint', light);
        spectral.add(material);
        primitive.material = material;
        continue;
      }

      final picture = part % textures.length;
      final key = '$name/$picture';
      var material = solid[key];

      if (material == null) {
        final texture = textures[picture];

        material = await loadFmatMaterial(huskMaterialAsset);
        material.name = name;
        _style(material.parameters, name, light);
        material.parameters.setTexture(
          'image',
          texture.gpuTexture,
          sampler: texture.sampledSampler,
        );
        solid[key] = material;
      }

      primitive.material = material;
    }

    return HuskRig._(
      root: Node(name: 'husk_rig')..add(model),
      vessel: vessel,
      light: light,
      model: model,
      solid: solid,
      spectral: spectral,
    );
  }

  /// The node to add to a scene.
  final Node root;

  final Vessel vessel;

  /// The colour this husk glows, taken from its image.
  final Color light;

  final Node _model;
  final Map<String, PreprocessedMaterial> _solid;
  final List<PreprocessedMaterial> _spectral;

  late final Node? _orbit = _find(_model, 'orbit');
  late final Node? _drift = _find(_model, 'drift');
  late final Node? _ribbon = _find(_model, 'ribbon');
  late final Node? _flame = _find(_model, 'flame');

  double _seconds = 0;
  double _flash = 0;
  double _punch = 0;

  /// How fast a strike's flash fades, as a share lost each second.
  static const _flashDecay = 7.0;

  /// Puts the husk where the rules say it is, turned as they say.
  void place(HuskPose pose) {
    final centre = pose.centre;
    // The model faces -z, so turning it by the husk's bearing leaves it
    // facing the player. A spinning body turns against that.
    final turn = vm.Matrix4.rotationY(pose.yaw - pose.spin);
    final size = 1 + _punch * 0.06;

    root.localTransform =
        vm.Matrix4.translationValues(centre.x, centre.y, centre.z) *
        turn *
        vm.Matrix4.diagonal3Values(size, size, size);

    _orbit?.localTransform = vm.Matrix4.rotationZ(pose.orbit);
  }

  /// Advances everything about the husk that is only for show.
  void animate(double deltaSeconds) {
    _seconds += deltaSeconds;

    final fade = math.exp(-_flashDecay * deltaSeconds);

    _flash *= fade;
    _punch *= fade;
    _setAll('flash', _flash);

    for (final material in _spectral) {
      material.parameters.setFloat('time', _seconds);
    }

    _drift?.localTransform =
        vm.Matrix4.rotationZ(-_seconds * 0.35) *
        vm.Matrix4.translationValues(0, 0.02 * math.sin(_seconds * 1.3), 0);

    final ribbon = _ribbon;

    if (ribbon != null && ribbon.morphTargetCount >= 2) {
      // Two keys a quarter-wave apart, blended with a sine and a cosine,
      // make the wave travel down the ribbon.
      ribbon.setMorphWeights([
        math.cos(_seconds * 3.2),
        math.sin(_seconds * 3.2),
      ]);
    }

    final flicker = 1 + 0.12 * math.sin(_seconds * 11) * math.sin(_seconds * 7);

    _flame?.scale = vm.Vector3.all(flicker);
  }

  /// Lights the husk up for an instant, as a strike lands. [strength] runs
  /// from 0 to 1.
  void flash(double strength) {
    _flash = math.max(_flash, strength);
    _punch = math.max(_punch, strength);
  }

  /// How much of the husk has come apart, from 0 (whole) to 1 (gone).
  set dissolve(double amount) => _setAll('dissolve', amount.clamp(0, 1));

  void _setAll(String parameter, double value) {
    for (final material in [..._solid.values, ..._spectral]) {
      material.parameters.setFloat(parameter, value);
    }
  }

  static void _style(MaterialParameters parameters, String name, Color light) {
    parameters.setColor('glow', light);

    switch (name) {
      case 'canvas':
        parameters
          ..setFloat('image_mix', 1)
          ..setFloat('image_glow', 0.35)
          ..setFloat('surface_roughness', 0.65)
          ..setFloat('rim_strength', 0.25);
      case 'paper':
        parameters
          ..setFloat('image_mix', 1)
          ..setFloat('image_glow', 1.5)
          ..setFloat('surface_roughness', 0.9)
          ..setFloat('rim_strength', 0.5);
      case 'ribbon':
        parameters
          ..setFloat('image_mix', 1)
          ..setFloat('image_glow', 0.7)
          ..setFloat('surface_roughness', 0.85)
          ..setFloat('rim_strength', 0.5);
      case 'shard':
        parameters
          ..setFloat('image_mix', 1)
          ..setFloat('image_glow', 0.6)
          ..setFloat('surface_roughness', 0.2)
          ..setFloat('surface_metallic', 0.6)
          ..setFloat('rim_strength', 1.2);
      case 'core':
        parameters
          ..setColor('tint', const Color(0xFF040406))
          ..setFloat('surface_roughness', 0.5)
          ..setFloat('rim_strength', 2.2);
      case 'glow':
        parameters
          ..setColor('tint', const Color(0xFF000000))
          ..setFloat('glow_strength', 7)
          ..setFloat('rim_strength', 0);
      default:
        parameters
          ..setColor('tint', const Color(0xFF2B2018))
          ..setFloat('surface_roughness', 0.6)
          ..setFloat('rim_strength', 0.9);
    }
  }

  static Node? _find(Node node, String name) {
    if (node.name == name) {
      return node;
    }

    for (final child in node.children) {
      final found = _find(child, name);

      if (found != null) {
        return found;
      }
    }

    return null;
  }

  /// Calls [action] for [node] and everything under it, with the number of
  /// the part each belongs to: the `N` of the nearest `cube_N` above it, or
  /// [part] where there is none.
  static void _visit(
    Node node,
    int part,
    void Function(Node node, int part) action,
  ) {
    final numbered = _partName.firstMatch(node.name);
    final own = numbered == null ? part : int.parse(numbered.group(1)!);

    action(node, own);

    for (final child in node.children) {
      _visit(child, own, action);
    }
  }

  static final _partName = RegExp(r'^cube_(\d+)$');
}
