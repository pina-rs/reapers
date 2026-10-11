import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:flutter/services.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:three_js/three_js.dart' as three;
import 'package:web/web.dart' as web;

import '../l10n/l10n.dart';
import '../widgets/ledger.dart';
import 'reaper_cloth.dart';
import 'reaper_look.dart';
import 'reaper_playback.dart';
import 'reaper_web_material.dart';

/// Browser portrait using the game's GLBs and look, without Flutter GPU.
class ReaperStage extends HookWidget {
  const ReaperStage({
    super.key,
    required this.genome,
    required this.weapon,
    required this.outfit,
    this.turnsOnItsOwn = false,
    this.playback,
  });

  final ReaperGenome genome;
  final Weapon weapon;
  final Outfit outfit;
  final bool turnsOnItsOwn;
  final ReaperPlayback? playback;

  @override
  Widget build(BuildContext context) {
    final reducedMotion = MediaQuery.disableAnimationsOf(context);
    final clock = useMemoized(() => playback ?? ReaperPlayback(), [playback]);
    useEffect(() => playback == null ? clock.dispose : null, [clock]);
    final retry = useState(0);
    final supported = useMemoized(_supportsWebGL, [retry.value]);

    if (!supported) {
      return LedgerNotice(
        title: context.l10n.commonFailedTitle,
        body: context.l10n.studioWebGLUnavailable,
        action: context.l10n.commonRetry,
        onAction: () => retry.value++,
      );
    }

    return LayoutBuilder(
      builder: (context, constraints) => HookBuilder(
        builder: (context) {
          final revision = useState(0);
          final stage = useMemoized(
            () => _WebStage(
              Size(constraints.maxWidth, constraints.maxHeight),
              () {
                if (context.mounted) revision.value++;
              },
            ),
            [constraints.maxWidth, constraints.maxHeight, retry.value],
          );

          useEffect(() => stage.dispose, [stage]);
          useEffect(() {
            unawaited(stage.show(genome, weapon, outfit));
            return null;
          }, [stage, genome.code, weapon, outfit]);
          stage.turns = turnsOnItsOwn && !reducedMotion;
          stage.playback = clock;
          stage.reducedMotion = reducedMotion;
          stage.animationPreview = playback != null;

          if (stage.failed) {
            return LedgerNotice(
              title: context.l10n.commonFailedTitle,
              body: context.l10n.studioModelFailed,
              action: context.l10n.commonRetry,
              onAction: () => retry.value++,
            );
          }

          // ThreeJS also listens for gestures inside its platform view. Observe
          // pointers directly so its recognizers cannot swallow portrait turns.
          return Listener(
            behavior: HitTestBehavior.opaque,
            onPointerDown: (event) => stage.startDrag(event.pointer),
            onPointerUp: (event) => stage.endDrag(event.pointer),
            onPointerCancel: (event) => stage.endDrag(event.pointer),
            onPointerMove: (event) => stage.drag(event.pointer, event.delta),
            child: stage.viewer.build(),
          );
        },
      ),
    );
  }
}

/// Check before the rendering dependency tries to open an alert dialog.
bool _supportsWebGL() {
  final context = web.HTMLCanvasElement().getContext('webgl2');
  if (context == null) return false;

  final extension = (context as web.WebGL2RenderingContext).getExtension(
    'WEBGL_lose_context',
  );
  if (extension != null) {
    (extension as web.WEBGL_lose_context).loseContext();
  }

  return true;
}

/// Owns one browser context. Trait changes reuse decoded models; only the
/// selected calling is retained, and stale requests cannot mount after disposal.
final class _WebStage {
  _WebStage(Size size, this.changed) {
    viewer = three.ThreeJS(
      size: size,
      settings: three.Settings(
        clearColor: 0x07080B,
        antialias: true,
        enableShadowMap: false,
        screenResolution: 1.5,
        toneMapping: three.ACESFilmicToneMapping,
        toneMappingExposure: 1.1,
      ),
      onSetupComplete: () {
        if (_disposed) {
          viewer.dispose();
          return;
        }
        // The portrait handles gestures in Flutter. Keep the embedded canvas
        // from intercepting pointer input or reporting unbounded CSS sizing.
        final canvas = web.document.getElementById(
          'canvas-id${viewer.texture!.textureId}',
        ) as web.HTMLCanvasElement?;
        if (canvas == null) {
          failed = true;
          if (!_disposed) changed();
          return;
        }
        canvas.style
          ..width = '100%'
          ..height = '100%'
          ..pointerEvents = 'none';
        changed();
      },
      setup: () async {},
    );
    viewer.scene = three.Scene();
    viewer.camera = three.PerspectiveCamera(
      30,
      size.width / size.height,
      0.1,
      30,
    );
    viewer.scene.add(three.HemisphereLight(0xB8CBDD, 0x121218, 1.2));
    final key = three.DirectionalLight(0xD1E6FF, 3)
      ..position.setValues(3, 5, 4);
    viewer.scene.add(key);
    viewer.addAnimationEvent(_tick);
    _positionCamera();
  }

  final void Function() changed;
  late final three.ThreeJS viewer;
  final _loader = three.GLTFLoader()..setPath('assets/models/');
  three.Object3D? _model;
  three.Object3D? _weapons;
  three.Object3D? _held;
  ReaperClass? _calling;
  ReaperGenome? _genome;
  WeaponKind? _weaponKind;
  Future<void>? _loading;
  int _generation = 0;
  bool _disposed = false;
  bool failed = false;
  bool turns = false;
  bool dragging = false;
  int? _pointer;
  double turn = 0;
  double _seconds = 0;
  String? _materialSource;
  ReaperPlayback? playback;
  bool reducedMotion = false;
  bool animationPreview = false;
  three.AnimationMixer? _mixer;
  List<three.AnimationClip> _clips = [];
  three.AnimationAction? _action;
  ReaperMotion? _motion;

  void startDrag(int pointer) {
    _pointer ??= pointer;
    dragging = true;
  }

  void drag(int pointer, Offset delta) {
    if (_pointer == pointer && delta.dx.abs() > delta.dy.abs()) {
      turn += delta.dx * 0.012;
    }
  }

  void endDrag(int pointer) {
    if (_pointer != pointer) return;
    _pointer = null;
    dragging = false;
  }

  Future<void> show(ReaperGenome genome, Weapon weapon, Outfit outfit) async {
    final generation = ++_generation;
    final previous = _loading;
    final done = Completer<void>();
    _loading = done.future;
    if (previous != null) await previous;

    try {
      // Effects run during build; cached trait changes must notify afterward.
      await Future<void>.value();
      if (_disposed || generation != _generation) return;
      _materialSource ??= await rootBundle.loadString(
        'assets/materials/reaper.fmat',
      );
      if (_disposed || generation != _generation) return;
      if (_calling != genome.reaperClass) {
        final data = await _loader.fromAsset(
          'reaper_${genome.reaperClass.name}.glb',
        );
        if (data == null) throw StateError('Missing reaper model');
        if (_disposed || generation != _generation) {
          data.scene.dispose();
          return;
        }
        final animations = data.animations;
        if (animations == null ||
            animations.length != ReaperMotion.values.length ||
            animations.any((Object? clip) => clip is! three.AnimationClip) ||
            !ReaperMotion.values.every(
              (motion) => animations.any(
                (Object? clip) =>
                    clip is three.AnimationClip && clip.name == motion.name,
              ),
            )) {
          data.scene.dispose();
          throw StateError('Incomplete reaper animation library');
        }
        _model?.removeFromParent();
        _mixer?.stopAllAction();
        _model?.dispose();
        _model = data.scene;
        _mixer = three.AnimationMixer(data.scene);
        _clips = animations.cast<three.AnimationClip>();
        _action = null;
        _motion = null;
        _calling = genome.reaperClass;
        _held = null;
        _weaponKind = null;
        viewer.scene.add(data.scene);
        turn = 0;
      }
      if (_weapons == null) {
        final weapons = await _loader.fromAsset('weapons.glb');
        if (weapons == null) throw StateError('Missing weapon models');
        if (_disposed || generation != _generation) {
          weapons.scene.dispose();
          return;
        }
        _weapons = weapons.scene;
      }
      final model = _model!;
      _genome = genome;
      final variants = {
        'head_': genome.head,
        'mantle_': genome.mantle,
        'robe_': genome.robe,
      };
      model.traverse((node) {
        for (final entry in variants.entries) {
          if (!node.name.startsWith(entry.key)) continue;
          final index = int.tryParse(node.name.substring(entry.key.length));
          if (index != null) node.visible = index == entry.value;
        }
        if (node.name.startsWith('eyes')) {
          node.scale.setScalar(0.8 + 0.7 * ReaperGenome.fraction(genome.gaze));
        }
        if (node.name.startsWith('void_') || node.name.startsWith('wrap_')) {
          node.visible = outfit != Outfit.hollowed;
        }
      });
      model.scale.setScalar(
        0.95 + 0.13 * ReaperGenome.fraction(genome.stature),
      );
      _dress(model, ReaperLook.of(genome, outfit));

      if (_weaponKind != weapon.kind) {
        final socket = model.getObjectByName('socket_weapon');
        final source = _weapons?.getObjectByName('weapon_${weapon.kind.name}');
        if (socket == null || source == null) {
          throw StateError('Missing weapon socket');
        }
        _held?.removeFromParent();
        _held?.dispose();
        final held = source.clone();
        socket.add(held);
        _held = held;
        _weaponKind = weapon.kind;
      }
      _dress(_held!, ReaperLook.of(genome, Outfit.values[weapon.temper.index]));
      failed = false;
      _stirCloth();
      if (!_disposed) changed();
    } catch (error, stack) {
      if (!_disposed && generation == _generation) {
        FlutterError.reportError(
          FlutterErrorDetails(exception: error, stack: stack),
        );
        failed = true;
        changed();
      }
    } finally {
      done.complete();
      if (identical(_loading, done.future)) _loading = null;
    }
  }

  void _dress(three.Object3D root, ReaperLook look) {
    root.traverse((node) {
      if (node is! three.Mesh) return;
      final previous = node.material;
      final name = previous?.name ?? 'cloth';
      if (previous is ReaperWebMaterial) {
        previous.update(look);
        return;
      }
      final positions = node.geometry?.attributes['position'];
      if (positions == null) throw StateError('Reaper is missing vertices');
      // Void and light meshes intentionally omit masks. The native material
      // supplies white vertex colour for them; preserve that authored default.
      final masks =
          node.geometry!.attributes['color'] ??
          three.Float32BufferAttribute.fromList(
            List<double>.filled(positions.count.toInt() * 4, 1),
            4,
          );
      node.geometry!.attributes['reaperMask'] = masks;
      // The authored UVs hold rest coordinates, so grain stays attached to
      // swaying cloth. GLTFLoader names TEXCOORD_1 `uv2` in this release.
      final uv = node.geometry!.attributes['uv'];
      final depth = node.geometry!.attributes['uv2'];
      node.geometry!.attributes['reaperRest'] =
          three.Float32BufferAttribute.fromList([
            for (var i = 0; i < positions.count; i++) ...[
              (uv?.getX(i) ?? 0).toDouble(),
              (uv?.getY(i) ?? 0).toDouble(),
              (depth?.getX(i) ?? 0).toDouble(),
            ],
          ], 3);
      node.material = ReaperWebMaterial(name, _materialSource!, look);
      previous?.dispose();
    });
  }

  void _tick(double delta) {
    if (_disposed) return;
    final clock = playback;
    if (clock != null && _model != null) {
      if (!reducedMotion) clock.advance(math.min(delta, 0.1));
      _seconds = clock.seconds;
      if (_motion != clock.motion) {
        _action?.stop();
        final clip = _clips.singleWhere(
          (clip) => clip.name == clock.motion.name,
        );
        _action = _mixer!.clipAction(clip)!..play();
        _action!.paused = true;
        _motion = clock.motion;
      }
      _action!.time = clock.seconds;
      _mixer!.update(0);
      _held?.visible = !clock.motion.unarmed;
    }
    if (turns && !dragging) turn += math.min(delta, 0.1) * 0.25;
    _stirCloth();
    _model?.traverse((node) {
      final material = node.material;
      if (material is ReaperWebMaterial) material.time = _seconds;
    });
    _positionCamera();
  }

  void _stirCloth() {
    final genome = _genome;
    if (genome == null) return;
    final cloth = reaperClothAt(_motion ?? ReaperMotion.idle, _seconds);
    _model?.traverse((node) {
      if (!node.name.startsWith('robe_')) return;
      final weights = {
        'gaunt': ReaperGenome.fraction(genome.gauntness),
        'long_hem': ReaperGenome.fraction(genome.tatter),
        'sway_x': cloth.swayX,
        'sway_y': cloth.swayY,
        'billow': cloth.billow,
      };
      for (final entry in weights.entries) {
        final index = node.morphTargetDictionary?[entry.key];
        if (index != null) node.morphTargetInfluences[index] = entry.value;
      }
    });
  }

  void _positionCamera() {
    final distance = animationPreview ? 10.5 : 8.4;
    viewer.camera.position.setValues(
      math.sin(turn) * distance,
      0.67,
      math.cos(turn) * distance,
    );
    viewer.camera.lookAt(three.Vector3(0, 1.42, 0));
  }

  void dispose() {
    _disposed = true;
    _generation++;
    _loader.dispose();
    _mixer?.stopAllAction();
    _weapons?.dispose();
    // ThreeJS finishes asynchronous context creation before its cleanup is
    // safe. A pending context is released by onSetupComplete instead.
    if (viewer.mounted) viewer.dispose();
  }
}
