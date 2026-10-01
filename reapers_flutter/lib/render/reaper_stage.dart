import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:flutter_scene/scene.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:vector_math/vector_math.dart' as vm;

import 'reaper_rig.dart';
import 'veil_look.dart';

/// A reaper standing in the dark, turning as the player drags.
///
/// This is the only place the player sees their own reaper: in the sanctum,
/// in the armory and at the oath. During a hunt the reaper is the player.
class ReaperStage extends HookWidget {
  const ReaperStage({
    super.key,
    required this.genome,
    required this.weapon,
    required this.outfit,
    this.turnsOnItsOwn = false,
  });

  final ReaperGenome genome;

  final Weapon weapon;

  final Outfit outfit;

  /// Whether the reaper slowly turns when nobody is dragging it.
  final bool turnsOnItsOwn;

  /// Where the camera looks: about chest height on a reaper.
  static final _target = vm.Vector3(0, 1.42, 0);

  static const _distance = 8.4;
  static const _fieldOfView = 30 * math.pi / 180;

  @override
  Widget build(BuildContext context) {
    final stage = useMemoized(_Stage.new);
    final rig = useState<ReaperRig?>(null);
    final failure = useState<Object?>(null);
    final turn = useRef(0.0);
    final dragging = useRef(false);

    useEffect(() => stage.dispose, [stage]);

    useEffect(() {
      var cancelled = false;

      Future<void> load() async {
        try {
          final loaded = await stage.show(
            genome: genome,
            weapon: weapon,
            outfit: outfit,
          );

          if (!cancelled) {
            rig.value = loaded;
          }
        } catch (error) {
          if (!cancelled) {
            failure.value = error;
          }
        }
      }

      unawaited(load());

      return () => cancelled = true;
    }, [stage, genome.code]);

    // Trying on a weapon or an outfit restyles the reaper already standing
    // there, so browsing the armory never reloads it.
    useEffect(() {
      unawaited(
        rig.value?.wield(weapon).catchError((Object error) {
          failure.value = error;
        }),
      );

      return null;
    }, [rig.value, weapon]);

    useEffect(() {
      rig.value?.wear(outfit);

      return null;
    }, [rig.value, outfit]);

    if (failure.value != null) {
      return Center(child: ErrorWidget(failure.value!));
    }

    if (rig.value == null) {
      return const SizedBox.expand();
    }

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onHorizontalDragStart: (_) => dragging.value = true,
      onHorizontalDragEnd: (_) => dragging.value = false,
      onHorizontalDragUpdate: (details) =>
          turn.value += details.delta.dx * 0.012,
      child: SceneView(
        stage.scene,
        onTick: (elapsed, deltaSeconds) {
          if (turnsOnItsOwn && !dragging.value) {
            turn.value += deltaSeconds * 0.25;
          }
        },
        cameraBuilder: (_) => PerspectiveCamera(
          position: vm.Vector3(
            math.sin(turn.value) * _distance,
            _target.y - 0.75,
            -math.cos(turn.value) * _distance,
          ),
          target: _target,
          fovRadiansY: _fieldOfView,
        ),
      ),
    );
  }
}

/// The scene behind a [ReaperStage]: one reaper and the light it stands in.
final class _Stage {
  final scene = Scene();

  ReaperRig? _rig;
  bool _lit = false;

  Future<ReaperRig> show({
    required ReaperGenome genome,
    required Weapon weapon,
    required Outfit outfit,
  }) async {
    await Scene.initializeStaticResources();

    if (!_lit) {
      VeilLook.sanctum(scene);
      _lit = true;
    }

    final rig = await ReaperRig.load(
      genome: genome,
      weapon: weapon,
      outfit: outfit,
    );
    final previous = _rig;

    if (previous != null) {
      scene.remove(previous.root);
    }

    scene.add(rig.root);
    _rig = rig;

    return rig;
  }

  void dispose() {
    final rig = _rig;

    if (rig != null) {
      scene.remove(rig.root);
    }

    _rig = null;
  }
}
