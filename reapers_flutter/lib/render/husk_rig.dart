import 'dart:math' as math;
import 'dart:ui';

import 'package:flutter_scene/scene.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:vector_math/vector_math.dart' as vm;

import 'husk_reaction.dart';
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

  /// Whether it is frozen in ice.
  bool frozen,

  /// Whether it is burning.
  bool burning,
});

/// A husk doing nothing in particular.
const HuskMood restingMood = (
  speed: 0,
  gape: 0,
  frozen: false,
  burning: false,
);

/// One husk: a collection's creature, dressed as one NFT.
///
/// The rig only ever draws what the rules say. It is told where the husk is
/// with [place], what it is doing and what is being done to it with
/// [animate], and what happened with [flash], [hit] and [dissolve]; it
/// decides nothing about the hunt. How it acts all that out is
/// [HuskReaction]'s: it reels, cowers, rages, struggles and dies.
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

      if (node.name == 'shadow') {
        _shadows.add((node, node.localTransform.clone()));
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

  final _reaction = HuskReaction();

  /// The way from the player to the husk, flat, in the model's frame: the
  /// way a blow drives it.
  vm.Vector3 _away = vm.Vector3(0, 0, 1);

  /// Every pivot the game moves, with the transform the model gave it.
  final _rest = <String, (Node, vm.Matrix4)>{};

  /// The soft dark patch under a creature that walks, with the transform
  /// the model gave it. It stays on whatever the creature stands on, and is
  /// left behind on it when the creature leaps.
  final _shadows = <(Node, vm.Matrix4)>[];

  double _seconds = 0;
  double _stride = 0;
  double _effort = 0;
  double _gape = 0;
  double _flash = 0;
  double _punch = 0;
  double _watchYaw = 0;
  double _watchPitch = 0;
  double _size = 1;

  /// How far the surface under the husk is below the middle of its body,
  /// in metres, or null when there is nothing under it to cast a shadow on.
  double? _drop;

  /// How far through a leap the husk is: 0 standing, 1 tucked in the air.
  double _tuck = 0;
  bool _leaping = false;
  double _frost = 0;
  double _burn = 0;

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
    // The face: each eye, or both in one band, and a brow over each.
    'eye_l',
    'eye_r',
    'eyes',
    'brow_l',
    'brow_r',
  };

  /// How bright each glowing surface burns at rest, as `_style` sets it.
  static const _glowStrength = {'accent': 6.0, 'glow': 7.0};

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

    _size = size;
    _leaping = pose.leaping;
    _drop = switch (pose.ground) {
      final ground? => at.y - ground,
      null => null,
    };

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

    final away = vm.Matrix4.rotationY(
      math.pi - pose.heading,
    ).transform3(vm.Vector3(pose.direction.x, 0, pose.direction.z));

    if (away.length2 > 1e-6) {
      _away = away.normalized();
    }
  }

  /// Advances everything about the husk that is only for show.
  void animate(
    double deltaSeconds, {
    HuskMood mood = restingMood,
    HuskPlight plight = calmPlight,
  }) {
    final reaction = _reaction..step(deltaSeconds, plight);
    // A heavy blow stops time for the husk for an instant; its flash and
    // its reeling carry on through it, so the blow lands with a jolt.
    final played = deltaSeconds * reaction.timeScale;

    _seconds += played;

    final fade = math.exp(-_flashDecay * deltaSeconds);

    _flash *= fade;
    _punch *= fade;
    _setAll('flash', _flash);

    // Ice takes a moment to close over it and to melt; fire catches fast.
    _frost +=
        ((mood.frozen ? 1 : 0) - _frost) *
        (1 - math.exp(-(mood.frozen ? 6 : 2.5) * deltaSeconds));
    _burn +=
        ((mood.burning ? 1 : 0) - _burn) *
        (1 - math.exp(-(mood.burning ? 10 : 3) * deltaSeconds));
    _setSurfaces('frost', _frost);
    _setSurfaces('burn', _burn);
    _setSurfaces('burn_flicker', _seconds * 1.7);

    // In a leap its legs come up under it, and go down again to land.
    _tuck += ((_leaping ? 1 : 0) - _tuck) * (1 - math.exp(-14 * deltaSeconds));

    // How hard it is working, from standing (0) through ambling to flat out
    // (1). It eases, so a creature does not snap between gaits.
    final target = vessel.boltSpeed <= 0
        ? 0.0
        : (mood.speed / vessel.boltSpeed).clamp(0.0, 1.0);
    final ease = 1 - math.exp(-8 * deltaSeconds);

    _effort += (target - _effort) * ease;
    _gape +=
        (math.max(mood.gape, reaction.gape) - _gape) *
        (1 - math.exp(-14 * deltaSeconds));
    // Strides come faster the faster it goes, and never quite stop: a
    // creature at rest still shifts its weight.
    _stride += played * (2.2 + 9 * _effort);

    // Frozen, nothing moves: not a stride, not a breath.
    final still = 1 - _frost;
    final swing =
        (0.08 + 0.75 * math.sqrt(_effort)) *
        math.sin(_stride) *
        (1 - _tuck) *
        still;
    final tuck = 0.9 * _tuck + reaction.splay;
    final t = _seconds * still;
    final flail = reaction.flail * still;
    final shiver = reaction.shiver * still;
    // Arms come up to shield the face as it cowers and flinches.
    final shield = 1.1 * reaction.cower + 0.8 * reaction.wince;

    for (final MapEntry(key: name, value: (node, rest)) in _rest.entries) {
      final turn = switch (name) {
        // A four-legged gait moves diagonal pairs together.
        // In a leap the front legs reach forward and the back legs kick
        // back, the way a cat's do. Being reaped, every leg scrabbles.
        'leg_fl' || 'leg_fr' => vm.Matrix4.rotationX(
          (name == 'leg_fl' ? swing + flail : -swing - flail) - tuck + shiver,
        ),
        'arm_l' || 'arm_r' => vm.Matrix4.rotationX(
          (name == 'arm_r' ? swing + flail : -swing - flail) -
              tuck -
              shield +
              shiver,
        ),
        'leg_bl' || 'leg_br' || 'leg_l' || 'leg_r' => vm.Matrix4.rotationX(
          (name == 'leg_br' || name == 'leg_l'
                  ? swing - flail
                  : -swing + flail) +
              tuck -
              shiver,
        ),
        // Wings beat all the time, harder when it is fleeing and frantic
        // when it is reaped; afraid, it draws them in. Left and right
        // mirror each other.
        'wing_l' => vm.Matrix4.rotationZ(_wing(t, reaction)),
        'wing_r' => vm.Matrix4.rotationZ(-_wing(t, reaction)),
        'tail' => vm.Matrix4.rotationY(
          (0.18 + 0.3 * _effort) * math.sin(_stride * 0.5 + 1) +
              4 * shiver +
              flail,
        ),
        'neck' => vm.Matrix4.rotationX(
          0.05 * math.sin(t * 0.8) +
              0.1 * _effort * math.sin(_stride * 2) +
              0.4 * reaction.headPitch * still,
        ),
        // The head keeps the player in sight, and looks about a little. It
        // is thrown back by blows, ducks as it cowers and shakes in terror.
        'head' =>
          vm.Matrix4.rotationY(
                _watchYaw + 0.06 * math.sin(t * 0.7) + reaction.headYaw * still,
              ) *
              vm.Matrix4.rotationX(
                _watchPitch +
                    0.04 * math.sin(t * 1.1) +
                    0.6 * reaction.headPitch * still,
              ),
        'jaw' => vm.Matrix4.rotationX(-0.75 * _gape),
        // Eyes go wide in terror and squeeze shut at a blow.
        'eye_l' || 'eye_r' => vm.Matrix4.diagonal3Values(
          reaction.eyeWidth,
          reaction.eyeHeight,
          reaction.eyeWidth,
        ),
        // A band of eyes wraps round the skull, so it only opens and
        // narrows: widened, it would stand off the face.
        'eyes' => vm.Matrix4.diagonal3Values(1, reaction.eyeHeight, 1),
        // The inner ends of the brows lift in fear and drive down in fury.
        // The creature's left is +x, so the inner end of its left brow is
        // towards -x.
        'brow_l' || 'brow_r' =>
          vm.Matrix4.translationValues(
                0,
                reaction.browLift * vessel.halfHeight,
                0,
              ) *
              vm.Matrix4.rotationZ(
                name == 'brow_l' ? -reaction.browTilt : reaction.browTilt,
              ),
        _ => vm.Matrix4.identity(),
      };

      node.localTransform = rest * turn;
    }

    // A walker rises and falls with each step; a flier with each beat.
    final step = _tuck > 0.5
        ? 0.0
        : vessel.gait == Gait.air
        ? 0.02 * math.sin(t * 2 * math.pi * _beatsPerSecond)
        : 0.03 * _effort * math.sin(_stride * 2).abs();
    final height = 2 * vessel.halfHeight;
    final lift = step + reaction.rise * height * still;

    _lift.localTransform =
        vm.Matrix4.translationValues(0, lift, 0) * _sway(reaction, still);
    _placeShadows(lift);

    for (final material in _materials.values) {
      if (_glowStrength[material.name] case final strength?) {
        material.parameters.setFloat(
          'glow_strength',
          strength * reaction.eyeGlow,
        );
      }
    }
  }

  /// The body's lean, twist and squash from [reaction], turned about its
  /// feet so that it rocks rather than spins about its middle.
  vm.Matrix4 _sway(HuskReaction reaction, double still) {
    // Tipped by blows and staggering, and pulled over towards the reaper
    // as the reap drags it up the arc.
    final leanX = (reaction.leanX - _away.x * reaction.drag) * still;
    final leanZ = (reaction.leanZ - _away.z * reaction.drag) * still;
    final lean = math.sqrt(leanX * leanX + leanZ * leanZ);
    final squash = 1 + reaction.squash * still;
    final spread = 1 / math.sqrt(squash);
    final feet = vessel.gait == Gait.air ? 0.0 : vessel.halfHeight;
    final sway = vm.Matrix4.rotationY(reaction.twist * still);

    if (lean > 1e-5) {
      // Tipping the top towards (x, z) is a turn about up × (x, z).
      sway.rotate(vm.Vector3(leanZ / lean, 0, -leanX / lean), lean);
    }

    return vm.Matrix4.translationValues(0, -feet, 0) *
        sway *
        vm.Matrix4.diagonal3Values(spread, squash, spread) *
        vm.Matrix4.translationValues(0, feet, 0);
  }

  /// Keeps each shadow on the surface under the husk, smaller and fainter
  /// the higher the husk is above it.
  void _placeShadows(double lift) {
    final drop = _drop;

    for (final (node, rest) in _shadows) {
      if (drop == null) {
        node.visible = false;
        continue;
      }

      node.visible = true;

      // How far above where it stands the husk is, in metres.
      final height = math.max(0, drop - vessel.halfHeight);
      final spread = 1 / (1 + 1.4 * height);
      final restY = rest.getTranslation().y;
      final down = -drop / _size - lift - restY;

      node.localTransform =
          vm.Matrix4.translationValues(0, down, 0) *
          rest *
          vm.Matrix4.diagonal3Values(spread, 1, spread);
    }
  }

  double get _beatsPerSecond => 2.6 + 2.4 * _effort + 4 * _reaction.flail.abs();

  /// How far a wing is raised at [seconds], in radians: drawn in when the
  /// creature is afraid, flung wide as it dies.
  double _wing(double seconds, HuskReaction reaction) =>
      _beat(seconds) * (1 + 0.8 * reaction.flail.abs()) -
      0.35 * reaction.cower +
      0.6 * reaction.splay;

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

  /// A blow lands: on seal [seal] when it struck one, or the body as a
  /// whole. The husk is driven away from the player, twisted away from
  /// the side it was struck on and flinches; [strength] runs from 0 to 1,
  /// and a [heavy] blow stops time for an instant.
  void hit({int? seal, required double strength, bool heavy = false}) {
    final widest = vessel.seals.fold(
      0.0,
      (widest, spec) => math.max(widest, spec.offset.x.abs()),
    );
    final side = seal == null || widest == 0
        ? 0.0
        : vessel.seals[seal].offset.x / widest;

    _reaction.hit(
      away: math.atan2(_away.z, _away.x),
      side: side,
      strength: strength,
      heavy: heavy,
    );
  }

  /// How much of the husk has come apart, from 0 (whole) to 1 (gone).
  set dissolve(double amount) => _setAll('dissolve', amount.clamp(0, 1));

  /// Sets [parameter] on the husk's own surfaces: not on its shadow, which
  /// is neither frozen nor burned.
  void _setSurfaces(String parameter, double value) {
    for (final material in _materials.values) {
      if (material.name != 'shadow') {
        material.parameters.setFloat(parameter, value);
      }
    }
  }

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
