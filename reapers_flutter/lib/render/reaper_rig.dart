import 'package:flutter_scene/scene.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:vector_math/vector_math.dart' as vm;

import 'reaper_cloth.dart';
import 'reaper_look.dart';

/// The shader every reaper surface is drawn with.
const reaperMaterialAsset = 'assets/materials/reaper.fmat';

/// The model that holds every weapon kind.
const weaponsModelAsset = 'assets/models/weapons.glb';

/// The model of one class, holding every head, mantle and robe it can wear.
String reaperModelAsset(ReaperClass reaperClass) =>
    'assets/models/reaper_${reaperClass.name}.glb';

/// One reaper, assembled from its genome and ready to add to a scene.
///
/// A class's model carries every part the shuffle can pick. The rig shows the
/// parts this genome names, hides the rest, puts the weapon in the hand and
/// swaps every material for the reaper shader, coloured by the genome's
/// colourway and the outfit being worn.
final class ReaperRig {
  ReaperRig._({
    required this.root,
    required this.genome,
    required this._model,
  });

  /// Loads and assembles [genome] holding [weapon] and wearing [outfit].
  ///
  /// `Scene.initializeStaticResources` must have completed first.
  static Future<ReaperRig> load({
    required ReaperGenome genome,
    required Weapon weapon,
    required Outfit outfit,
  }) async {
    final model = await loadScene(reaperModelAsset(genome.reaperClass));
    final root = Node(name: 'reaper_rig')..add(model);
    final rig = ReaperRig._(root: root, genome: genome, model: model);

    await rig._showGenomeParts();
    await rig.wield(weapon);
    rig.wear(outfit);
    root.addComponent(_ReaperCloth(rig));
    rig.sample(ReaperMotion.idle, 0);

    return rig;
  }

  /// The node to add to a scene. The reaper stands on this node's origin,
  /// facing its forward axis.
  final Node root;

  final ReaperGenome genome;

  /// The reaper's own surfaces, by material name.
  final _materials = <String, PreprocessedMaterial>{};

  /// The held weapon's surfaces. They are kept apart from the reaper's so
  /// that a weapon's temper and an outfit can differ.
  final _weaponMaterials = <String, PreprocessedMaterial>{};

  final Node _model;
  Node? _held;
  WeaponKind? _heldKind;
  AnimationClip? _clip;
  ReaperMotion? _motion;
  double _seconds = 0;

  /// Samples an authored clip without changing placement or game state.
  void sample(ReaperMotion motion, double seconds) {
    if (_motion != motion) {
      final animation = _model.findAnimationByName(motion.name);
      if (animation == null) {
        throw StateError('Missing reaper clip: ${motion.name}');
      }
      final previous = _clip;
      if (previous != null) _model.removeAnimationClip(previous);
      _clip = _model.createAnimationClip(animation);
      _motion = motion;
    }
    _seconds = seconds;
    _clip!.seek(seconds);
    _held?.visible = !motion.unarmed;
  }

  /// Shortest and tallest a reaper stands, as a scale on the model.
  static const _minStature = 0.95;
  static const _maxStature = 1.08;

  /// Puts [weapon] in the reaper's hand, replacing whatever it held.
  ///
  /// A weapon's temper is drawn the way the outfit of the same name is: the
  /// same model, more burned and more hollow.
  Future<void> wield(Weapon weapon) async {
    if (_heldKind != weapon.kind) {
      await _hold(weapon.kind);
    }

    final look = ReaperLook.of(genome, Outfit.values[weapon.temper.index]);

    for (final MapEntry(key: name, value: material)
        in _weaponMaterials.entries) {
      _applyLook(material.parameters, look, look.surfaceFor(name));
    }
  }

  Future<void> _hold(WeaponKind kind) async {
    final socket = _find(_model, 'socket_weapon');

    if (socket == null) {
      throw StateError('${genome.reaperClass.name} has no socket_weapon node');
    }

    final weapons = await loadScene(weaponsModelAsset);
    final held = _find(weapons, 'weapon_${kind.name}');

    if (held == null) {
      throw StateError('weapons.glb has no weapon_${kind.name} node');
    }

    _held?.detach();
    held.detach();
    socket.add(held);
    _held = held;
    _heldKind = kind;
    _weaponMaterials.clear();

    await _adoptMaterials(held, _weaponMaterials);
  }

  /// Dresses the reaper in [outfit].
  ///
  /// An outfit is the same model drawn with different numbers, so this is
  /// cheap and can be called as the player browses the armory.
  void wear(Outfit outfit) {
    final look = ReaperLook.of(genome, outfit);

    for (final MapEntry(key: name, value: material) in _materials.entries) {
      _applyLook(material.parameters, look, look.surfaceFor(name));
    }

    _visit(_model, (node) {
      if (node.name.startsWith('void_') || node.name.startsWith('wrap_')) {
        node.visible = !look.headless;
      }
    });
  }

  /// Shows the head, mantle and robe the genome names and sets its
  /// continuous traits.
  Future<void> _showGenomeParts() async {
    final chosen = {
      'head_': genome.head,
      'mantle_': genome.mantle,
      'robe_': genome.robe,
    };

    _visit(_model, (node) {
      for (final MapEntry(key: prefix, value: index) in chosen.entries) {
        final suffix = node.name.startsWith(prefix)
            ? int.tryParse(node.name.substring(prefix.length))
            : null;

        if (suffix != null) {
          node.visible = suffix == index;
        }
      }

      if (node.name.startsWith('eyes')) {
        node.scale = vm.Vector3.all(
          0.8 + 0.7 * ReaperGenome.fraction(genome.gaze),
        );
      }
    });

    final stature = ReaperGenome.fraction(genome.stature);

    _model.scale = vm.Vector3.all(
      _minStature + (_maxStature - _minStature) * stature,
    );

    await _adoptMaterials(_model, _materials);
  }

  /// Replaces every material under [node] with the reaper shader, keeping
  /// one instance for each material name in [materials] so that a look can
  /// restyle them all at once.
  static Future<void> _adoptMaterials(
    Node node,
    Map<String, PreprocessedMaterial> materials,
  ) async {
    final primitives = <MeshPrimitive>[];

    _visit(node, (visited) {
      final mesh = visited.mesh;

      if (mesh != null) {
        primitives.addAll(mesh.primitives);
      }
    });

    for (final primitive in primitives) {
      final name = primitive.material.name;
      var material = materials[name];

      if (material == null) {
        material = await loadFmatMaterial(reaperMaterialAsset);
        material.name = name;
        materials[name] = material;
      }

      primitive.material = material;
    }
  }

  static void _applyLook(
    MaterialParameters parameters,
    ReaperLook look,
    SurfaceLook surface,
  ) {
    parameters
      ..setColor('tint', surface.tint)
      ..setFloat('surface_roughness', surface.roughness)
      ..setFloat('surface_metallic', surface.metallic)
      ..setFloat('grime', surface.grime)
      ..setFloat('wear', surface.wear)
      ..setFloat('weave', surface.weave)
      ..setFloat('char_level', surface.burns ? look.charLevel : 0)
      ..setColor('ember', look.ember)
      ..setFloat('ember_strength', surface.burns ? look.emberStrength : 0)
      ..setFloat('hollow_level', surface.burns ? look.hollowLevel : 0)
      ..setColor('rim', look.rim)
      ..setFloat('rim_strength', surface.grime == 0 ? 0 : look.rimStrength)
      ..setColor('glow', look.accent)
      ..setFloat('glow_strength', surface.glowStrength);
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

  static void _visit(Node node, void Function(Node node) action) {
    action(node);

    for (final child in node.children) {
      _visit(child, action);
    }
  }
}

/// Adds secondary cloth motion without overriding authored joint animation.
final class _ReaperCloth extends Component {
  _ReaperCloth(this._rig);

  final ReaperRig _rig;

  Node? _robe;
  Map<String, int> _morphs = const {};

  @override
  void onMount() {
    ReaperRig._visit(_rig._model, (node) {
      final isShownRobe =
          node.name == 'robe_${_rig.genome.robe}' && node.morphTargetCount > 0;

      if (isShownRobe) {
        _robe = node;
        _morphs = {
          for (final (index, name) in node.morphTargetNames.indexed)
            name: index,
        };
      }
    });
  }

  @override
  void update(double deltaSeconds) {
    _stirCloth(_rig._seconds);
  }

  void _stirCloth(double t) {
    final robe = _robe;

    if (robe == null) {
      return;
    }

    final cloth = reaperClothAt(_rig._motion ?? ReaperMotion.idle, t);
    final weights = {
      'sway_x': cloth.swayX,
      'sway_y': cloth.swayY,
      'billow': cloth.billow,
      'gaunt': ReaperGenome.fraction(_rig.genome.gauntness),
      'long_hem': ReaperGenome.fraction(_rig.genome.tatter),
    };

    for (final MapEntry(key: name, value: weight) in weights.entries) {
      final index = _morphs[name];

      if (index != null) {
        robe.setMorphWeight(index, weight);
      }
    }
  }
}
