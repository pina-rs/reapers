import 'dart:math' as math;
import 'dart:ui';

import 'package:flutter_scene/scene.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:vector_math/vector_math.dart' as vm;

import 'nft_image.dart';

/// The shader for a husk's solid surfaces.
const huskMaterialAsset = 'assets/materials/husk.fmat';

/// The shader for the dark patch under a husk that stands on the floor.
const shadowMaterialAsset = 'assets/materials/shadow.fmat';

/// The model of one vessel.
String huskModelAsset(Vessel vessel) => 'assets/models/husk_${vessel.name}.glb';

/// What a husk is doing, as far as showing it goes.
typedef HuskMood = ({
  /// How fast it is covering ground, in metres a second.
  double speed,

  /// How wide its mouth is open, from 0 to 1.
  double gape,
});

/// One husk: a collection's creature, dressed as one NFT.
///
/// The rig only ever draws what the rules say. It is told where the husk is
/// with [place], what it is doing with [animate], and what happened with
/// [flash] and [dissolve]; it decides nothing about the hunt.
///
/// A creature is built from rigid parts on pivots, and this is where they
/// move: legs swing as fast as the creature is going, wings beat, the head
/// turns to watch the player. See art/blender/README.md for the parts.
final class HuskRig {
  HuskRig._({
    required this.root,
    required this.vessel,
    required this.light,
    required this._model,
    required this._lift,
    required this._materials,
  }) {
    _visit(_model, 0, (node, _) {
      if (_moving.contains(node.name)) {
        _rest[node.name] = (node, node.localTransform.clone());
      }
    });
  }

  /// Loads [vessel] and dresses it as [relics].
  ///
  /// A creature is one NFT: its colours and the thing it wears come from
  /// that NFT's look. A rift's husk is a whole collection: each of its cubes
  /// wears the portrait of the next relic in turn.
  ///
  /// `Scene.initializeStaticResources` must have completed first.
  static Future<HuskRig> load({
    required Vessel vessel,
    required List<Nft> relics,
  }) async {
    final model = await loadScene(huskModelAsset(vessel));
    final look = relics.first.look;
    final pictures = vessel.isSolo
        ? const <NftImage>[]
        : [for (final relic in relics) await NftImage.load(relic)];
    final textures = [
      for (final picture in pictures) await Texture2D.fromImage(picture.image),
    ];
    final light = vessel.isSolo ? _colour(look.accent) : pictures.first.light;
    final materials = <String, PreprocessedMaterial>{};
    final primitives = <(MeshPrimitive, int)>[];

    _visit(model, 0, (node, part) {
      final worn = _traitName.firstMatch(node.name);

      if (worn != null) {
        node.visible = int.parse(worn.group(1)!) == look.trait;
      }

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
      final picture = textures.isEmpty ? 0 : part % textures.length;
      final key = '$name/$picture';
      var material = materials[key];

      if (material == null) {
        material = await loadFmatMaterial(
          name == 'shadow' ? shadowMaterialAsset : huskMaterialAsset,
        );
        material.name = name;

        if (name != 'shadow') {
          _style(material.parameters, name, look, light);
        }

        if (textures.isNotEmpty && name == 'canvas') {
          final texture = textures[picture];

          material.parameters.setTexture(
            'image',
            texture.gpuTexture,
            sampler: texture.sampledSampler,
          );
        }

        materials[key] = material;
      }

      primitive.material = material;
    }

    final lift = Node(name: 'husk_lift')..add(model);

    return HuskRig._(
      root: Node(name: 'husk_rig')..add(lift),
      vessel: vessel,
      light: light,
      model: model,
      lift: lift,
      materials: materials,
    );
  }

  /// The node to add to a scene.
  final Node root;

  final Vessel vessel;

  /// The colour this husk glows: the light in its eyes.
  final Color light;

  final Node _model;

  /// Sits between [root] and the model, and carries the rise and fall of a
  /// step or a wingbeat, so that neither the placement nor the model's own
  /// transform is disturbed.
  final Node _lift;

  final Map<String, PreprocessedMaterial> _materials;

  /// Every pivot the game moves, with the transform the model gave it.
  final _rest = <String, (Node, vm.Matrix4)>{};

  double _seconds = 0;
  double _stride = 0;
  double _effort = 0;
  double _gape = 0;
  double _flash = 0;
  double _punch = 0;
  double _watchYaw = 0;
  double _watchPitch = 0;

  /// The pivots a creature may have. See art/blender/README.md.
  static const _moving = {
    'head',
    'neck',
    'jaw',
    'leg_l',
    'leg_r',
    'leg_fl',
    'leg_fr',
    'leg_bl',
    'leg_br',
    'arm_l',
    'arm_r',
    'wing_l',
    'wing_r',
    'tail',
  };

  /// How fast a strike's flash fades, as a share lost each second.
  static const _flashDecay = 7.0;

  /// How far a head turns to keep watching the player, either way. It is
  /// kept small because seals sit on heads: the rules fix a seal to the
  /// body, so a head that turned far would leave its seal behind.
  static final _headTurn = radians(20);
  static final _headTilt = radians(14);

  /// How much of its eye colour edges a creature's coat. Just enough to
  /// lift a dark coat off a dark room, and no more: at full strength a
  /// creature reads as a glowing outline instead of an animal.
  static const _rim = 0.12;

  /// Puts the husk where the rules say it is, facing the way they say.
  void place(HuskPose pose) {
    final at = pose.position;
    final size = 1 + _punch * 0.06;

    // The model faces -z, so a husk whose heading is straight back at the
    // player (pi) needs no turn at all.
    root.localTransform =
        vm.Matrix4.translationValues(at.x, at.y, at.z) *
        vm.Matrix4.rotationY(pose.heading - math.pi) *
        vm.Matrix4.diagonal3Values(size, size, size);

    // Where the player is, from where the husk stands: to its right when
    // positive, and above it when positive.
    _watchYaw = wrapAngle(
      pose.yaw + math.pi - pose.heading,
    ).clamp(-_headTurn, _headTurn);
    _watchPitch = (-pose.pitch).clamp(-_headTilt, _headTilt);
  }

  /// Advances everything about the husk that is only for show.
  void animate(double deltaSeconds, {HuskMood mood = (speed: 0, gape: 0)}) {
    _seconds += deltaSeconds;

    final fade = math.exp(-_flashDecay * deltaSeconds);

    _flash *= fade;
    _punch *= fade;
    _setAll('flash', _flash);

    // How hard it is working, from standing (0) through ambling to flat out
    // (1). It eases, so a creature does not snap between gaits.
    final target = vessel.boltSpeed <= 0
        ? 0.0
        : (mood.speed / vessel.boltSpeed).clamp(0.0, 1.0);
    final ease = 1 - math.exp(-8 * deltaSeconds);

    _effort += (target - _effort) * ease;
    _gape += (mood.gape - _gape) * (1 - math.exp(-14 * deltaSeconds));
    // Strides come faster the faster it goes, and never quite stop: a
    // creature at rest still shifts its weight.
    _stride += deltaSeconds * (2.2 + 9 * _effort);

    final swing = (0.08 + 0.75 * math.sqrt(_effort)) * math.sin(_stride);
    final t = _seconds;

    for (final MapEntry(key: name, value: (node, rest)) in _rest.entries) {
      final turn = switch (name) {
        // A four-legged gait moves diagonal pairs together.
        'leg_fl' || 'leg_br' || 'leg_l' || 'arm_r' => vm.Matrix4.rotationX(
          swing,
        ),
        'leg_fr' || 'leg_bl' || 'leg_r' || 'arm_l' => vm.Matrix4.rotationX(
          -swing,
        ),
        // Wings beat all the time, harder when it is fleeing. Left and
        // right mirror each other.
        'wing_l' => vm.Matrix4.rotationZ(_beat(t)),
        'wing_r' => vm.Matrix4.rotationZ(-_beat(t)),
        'tail' => vm.Matrix4.rotationY(
          (0.18 + 0.3 * _effort) * math.sin(_stride * 0.5 + 1),
        ),
        'neck' => vm.Matrix4.rotationX(
          0.05 * math.sin(t * 0.8) + 0.1 * _effort * math.sin(_stride * 2),
        ),
        // The head keeps the player in sight, and looks about a little.
        'head' =>
          vm.Matrix4.rotationY(_watchYaw + 0.06 * math.sin(t * 0.7)) *
              vm.Matrix4.rotationX(_watchPitch + 0.04 * math.sin(t * 1.1)),
        'jaw' => vm.Matrix4.rotationX(-0.75 * _gape),
        _ => vm.Matrix4.identity(),
      };

      node.localTransform = rest * turn;
    }

    // A walker rises and falls with each step; a flier with each beat.
    _lift.localTransform = vm.Matrix4.translationValues(
      0,
      vessel.gait == Gait.air
          ? 0.02 * math.sin(t * 2 * math.pi * _beatsPerSecond)
          : 0.03 * _effort * math.sin(_stride * 2).abs(),
      0,
    );
  }

  double get _beatsPerSecond => 2.6 + 2.4 * _effort;

  /// How far a wing is raised at [seconds], in radians.
  double _beat(double seconds) =>
      0.15 +
      (0.5 + 0.2 * _effort) * math.sin(seconds * 2 * math.pi * _beatsPerSecond);

  /// Lights the husk up for an instant, as a strike lands. [strength] runs
  /// from 0 to 1.
  void flash(double strength) {
    _flash = math.max(_flash, strength);
    _punch = math.max(_punch, strength);
  }

  /// How much of the husk has come apart, from 0 (whole) to 1 (gone).
  set dissolve(double amount) => _setAll('dissolve', amount.clamp(0, 1));

  void _setAll(String parameter, double value) {
    for (final material in _materials.values) {
      material.parameters.setFloat(parameter, value);
    }
  }

  static void _style(
    MaterialParameters parameters,
    String name,
    NftLook look,
    Color light,
  ) {
    parameters.setColor('glow', light);

    switch (name) {
      case 'body':
        parameters
          ..setColor('tint', _colour(look.body))
          ..setFloat('surface_roughness', 0.72)
          ..setFloat('rim_strength', _rim);
      case 'trim':
        parameters
          ..setColor('tint', _colour(look.trim))
          ..setFloat('surface_roughness', 0.7)
          ..setFloat('rim_strength', _rim);
      case 'detail':
        parameters
          ..setColor('tint', _colour(look.detail))
          ..setFloat('surface_roughness', 0.45)
          ..setFloat('rim_strength', _rim);
      case 'dark':
        parameters
          ..setColor('tint', const Color(0xFF050507))
          ..setFloat('surface_roughness', 0.9)
          ..setFloat('rim_strength', 0.15);
      case 'accent':
        parameters
          ..setColor('tint', const Color(0xFF000000))
          ..setFloat('glow_strength', 6)
          ..setFloat('rim_strength', 0);
      case 'glow':
        parameters
          ..setColor('tint', const Color(0xFF000000))
          ..setFloat('glow_strength', 7)
          ..setFloat('rim_strength', 0);
      // The rift's husk: cubes of gilt frame around a collection's portraits.
      case 'canvas':
        parameters
          ..setFloat('image_mix', 1)
          ..setFloat('image_glow', 0.35)
          ..setFloat('surface_roughness', 0.65)
          ..setFloat('rim_strength', 0.25);
      default:
        parameters
          ..setColor('tint', const Color(0xFF2B2018))
          ..setFloat('surface_roughness', 0.6)
          ..setFloat('rim_strength', 0.9);
    }
  }

  static Color _colour(int rgb) => Color(0xFF000000 | rgb);

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
  static final _traitName = RegExp(r'^trait_(\d+)$');
}
