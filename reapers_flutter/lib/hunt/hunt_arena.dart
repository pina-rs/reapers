import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:flutter_scene/scene.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:vector_math/vector_math.dart' as vm;

import '../render/husk_rig.dart';
import '../render/veil_look.dart';
import 'camera/camera_optics.dart';
import 'camera/hunt_camera.dart';
import 'camera_backdrop.dart';
import 'hunt_controller.dart';
import 'hunt_hud.dart';
import 'pose/adaptive_pose_source.dart';
import 'pose/ar_pose.dart';
import 'veil_backdrop.dart';

/// The vertical field of view the hunt is drawn with when there is no camera
/// feed to match: a simulator, a desktop, a refused permission.
const huntFieldOfView = 62 * math.pi / 180;

/// Everything a husk is fought in: the scene it is drawn in, the phone's
/// pose, and, once a fight is [engage]d, the rules of that fight.
///
/// A lone hunt and a reaper's place in a rift are both fought here. The arena
/// does not know which it is showing.
final class HuntArena {
  HuntArena({
    required this.vessel,
    required this.relics,
    PoseSource? poseSource,
  }) : poseSource = poseSource ?? AdaptivePoseSource();

  /// Set with `--dart-define=REAPERS_AUTOPLAY=true` to have every fight play
  /// itself. For watching, testing and recording; never for a release.
  static const autoplay = bool.fromEnvironment('REAPERS_AUTOPLAY');

  final Vessel vessel;

  /// The NFTs the husk is. One for an ordinary husk, which is that NFT's
  /// creature in its colours; a rift's husk is a whole collection.
  final List<Nft> relics;

  /// Made on first use, which is when the husk is loaded: everything about
  /// a fight except drawing it works without a GPU.
  late final scene = Scene();

  final slashes = <Slash>[];

  /// Where the phone points. Unless another is given, this follows the
  /// phone's own motion and falls back to a dragging finger.
  final PoseSource poseSource;

  /// The world behind the husk.
  final feed = HuntCamera();

  /// Ticks once a frame, for anything that paints from the rules.
  final frame = ValueNotifier(0);

  /// Set once a finished fight's husk has come apart completely.
  final gone = ValueNotifier(false);

  late final HuskRig rig;

  /// The fight being shown, or null while the husk only waits.
  HuntController? controller;

  /// How far the husk has come apart, from 0 (whole) to 1 (gone).
  double comeApart = 0;

  /// How far a husk that has just appeared has come together, from 0 (not
  /// there yet) to 1 (whole).
  double _gathered = 1;

  /// The size of the view, kept so that a strike the player did not touch
  /// the screen for can still leave its mark in the right place.
  Size viewSize = Size.zero;

  bool _loaded = false;
  HuskPose? _rest;
  int _slashCount = 0;

  /// How long a husk takes to come apart once the fight is over.
  static const _dissolveSeconds = 1.7;

  /// How long a husk takes to come together when it first appears.
  static const _gatherSeconds = 1.1;

  /// How far a husk whose ward has broken has already come apart.
  static const _brokenDissolve = 0.2;

  /// Where the phone points right now.
  ArPose get pose => controller?.pose ?? poseSource.pose;

  /// The vertical field of view the husk is drawn with, in radians.
  ///
  /// Over a camera feed it is the feed's own, so that a husk is seen through
  /// the same lens as the floor it stands on and stays put when the phone
  /// turns.
  double get fieldOfView {
    final lens = feed.lens;

    if (lens == null || viewSize.isEmpty) {
      return huntFieldOfView;
    }

    return coveringFieldOfView(
      lens,
      viewWidth: viewSize.width,
      viewHeight: viewSize.height,
    );
  }

  /// Loads the husk and stands it in the scene.
  Future<void> load() async {
    await Scene.initializeStaticResources();
    VeilLook.hunt(scene);

    rig = await HuskRig.load(vessel: vessel, relics: relics);
    _loaded = true;
    // Until it has somewhere to be, a husk is not there at all.
    rig.root.visible = _rest != null;
    _placeAtRest();
    scene.add(rig.root);
  }

  /// Stands the husk at [pose] until a fight is engaged.
  void rest(HuskPose pose) {
    _rest = pose;
    _placeAtRest();
  }

  /// Hands the arena the fight it is to show.
  ///
  /// A husk that was not in view until now gathers itself out of nothing
  /// where the fight begins.
  void engage(HuntSim sim) {
    if (_rest == null) {
      _gathered = 0;
    }

    controller = autoplay
        ? AutoplayHuntController(sim: sim)
        : HuntController(sim: sim, poseSource: poseSource);
    rest(sim.pose);
  }

  /// The view through the phone.
  Camera get camera {
    final current = pose;

    return PerspectiveCamera(
      position: vm.Vector3.zero(),
      target: _vector(current.forward),
      up: _vector(current.up),
      fovRadiansY: fieldOfView,
      fovNear: 0.05,
      fovFar: 120,
    );
  }

  /// While nothing is being fought: the husk drifts and the rules wait.
  void idle(double deltaSeconds) {
    poseSource.advance(deltaSeconds);
    rig.animate(deltaSeconds);
    frame.value++;
  }

  /// Advances the fight and everything drawn from it, and returns what
  /// happened. Before a fight is engaged this only idles.
  List<HuntEvent> advance(double deltaSeconds) {
    final controller = this.controller;

    if (controller == null) {
      idle(deltaSeconds);

      return const [];
    }

    final sim = controller.sim;
    final events = controller.frame(deltaSeconds);

    for (final event in events) {
      switch (event) {
        case StrikeLanded(:final empowered, :final seal):
          rig.flash(empowered ? 1 : 0.7);

          if (autoplay) {
            _markStrike(sim.pose.sealDirection(seal));
          }
        case StrikeGlanced():
          rig.flash(0.3);
        case Broken():
          rig.flash(1);
        default:
          break;
      }
    }

    if (sim.isOver) {
      comeApart = math.min(1, comeApart + deltaSeconds / _dissolveSeconds);
      gone.value = comeApart >= 1;
    } else {
      comeApart = sim.phase == HuntPhase.broken ? _brokenDissolve : 0;
    }

    _gathered = math.min(1, _gathered + deltaSeconds / _gatherSeconds);

    rig
      ..dissolve = math.max(comeApart, 1 - _gathered)
      ..place(sim.pose)
      ..animate(
        deltaSeconds,
        mood: (
          speed: sim.speed,
          // Its mouth opens as it draws breath and stays open for the
          // shriek.
          gape: switch (sim.phase) {
            HuntPhase.windup => sim.shriekProgress,
            HuntPhase.shriek => 1.0,
            _ => 0.0,
          },
        ),
      );

    for (final slash in slashes) {
      slash.age += deltaSeconds;
    }

    slashes.removeWhere((slash) => slash.spent);
    frame.value++;

    return events;
  }

  /// A tap at [position] on a view of [size]: a strike along the ray through
  /// that point.
  void strike(Offset position, Size size) {
    final controller = this.controller;

    if (controller == null) {
      return;
    }

    final striking = switch (controller.sim.phase) {
      HuntPhase.stalk ||
      HuntPhase.fight ||
      HuntPhase.windup ||
      HuntPhase.shriek => true,
      HuntPhase.broken || HuntPhase.absorbed || HuntPhase.escaped => false,
    };

    if (!striking) {
      return;
    }

    controller.tap(
      controller.pose.rayThrough(
        x: position.dx / size.width * 2 - 1,
        y: 1 - position.dy / size.height * 2,
        verticalFov: fieldOfView,
        aspect: size.width / size.height,
      ),
    );
    _addSlash(position);
  }

  void dispose() {
    feed.dispose();
    poseSource.dispose();
    frame.dispose();
    gone.dispose();
  }

  void _placeAtRest() {
    final rest = _rest;

    if (_loaded && rest != null) {
      rig
        ..root.visible = true
        ..place(rest);
    }
  }

  /// Leaves a slash where [direction] falls on the screen.
  void _markStrike(Vec3 direction) {
    final point = pose.project(
      direction,
      verticalFov: fieldOfView,
      aspect: viewSize.aspectRatio,
    );

    if (point == null || viewSize.isEmpty) {
      return;
    }

    _addSlash(
      Offset(
        (point.x + 1) / 2 * viewSize.width,
        (1 - point.y) / 2 * viewSize.height,
      ),
    );
  }

  void _addSlash(Offset at) {
    _slashCount++;
    slashes.add(
      Slash(
        at: at,
        // Cuts alternate direction, like a blade drawn back and forth.
        angle: (_slashCount.isEven ? -0.6 : 0.6) + at.dx * 0.002,
        strength: 0.6,
      ),
    );
  }

  static vm.Vector3 _vector(Vec3 value) =>
      vm.Vector3(value.x, value.y, value.z);
}

/// The arena as the player sees it: the world behind, the husk in it, the
/// marks of the fight over it, and the touches that become strikes.
///
/// Screens lay their own words over this. It shows no text of its own.
class HuntArenaView extends HookWidget {
  const HuntArenaView({
    super.key,
    required this.arena,
    required this.loaded,
    required this.running,
    required this.onEvents,
    this.onIdle,
    this.showSealOrder = false,
  });

  final HuntArena arena;

  /// Whether [HuntArena.load] has finished.
  final bool loaded;

  /// Whether the engaged fight is being played. While false the husk only
  /// drifts.
  final bool running;

  /// Told what happened each frame the fight runs.
  final void Function(List<HuntEvent> events) onEvents;

  /// Called each frame the fight is not running, with the seconds since the
  /// last: for whatever goes on before a fight, such as reading the room.
  final void Function(double deltaSeconds)? onIdle;

  /// Whether open seals are numbered in the order they opened.
  final bool showSealOrder;

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) {
      final size = constraints.biggest;
      final controller = arena.controller;

      arena.viewSize = size;

      return Stack(
        fit: StackFit.expand,
        children: [
          CameraBackdrop(
            camera: arena.feed,
            fallback: VeilBackdrop(
              pose: () => arena.pose,
              verticalFov: huntFieldOfView,
              repaint: arena.frame,
            ),
          ),
          // A cold wash over the world, so the camera feed reads as the
          // other side of the veil rather than as a photograph.
          const IgnorePointer(child: _VeilWash()),
          if (loaded)
            SceneView(
              arena.scene,
              onTick: (_, deltaSeconds) {
                if (running) {
                  onEvents(arena.advance(deltaSeconds));
                } else {
                  arena.idle(deltaSeconds);
                  onIdle?.call(deltaSeconds);
                }
              },
              cameraBuilder: (_) => arena.camera,
            ),
          if (loaded && controller != null)
            IgnorePointer(
              child: CustomPaint(
                painter: HuntHudPainter(
                  controller: controller,
                  light: arena.rig.light,
                  verticalFov: arena.fieldOfView,
                  slashes: arena.slashes,
                  showSealOrder: showSealOrder,
                  absorbed: () => arena.comeApart,
                  repaint: arena.frame,
                ),
              ),
            ),
          if (loaded && running && !HuntArena.autoplay)
            _Touch(arena: arena, size: size),
        ],
      );
    },
  );
}

/// Where fingers meet the fight. On a phone that feels its own motion, a
/// touch is a strike the moment it lands. Anywhere else, a drag looks around
/// and a tap strikes.
class _Touch extends HookWidget {
  const _Touch({required this.arena, required this.size});

  final HuntArena arena;
  final Size size;

  @override
  Widget build(BuildContext context) {
    final source = arena.poseSource;
    // How far the view turns for each point a finger travels. A drag turns
    // three times as far as the view it crosses, so a full turn is a few
    // swipes rather than a dozen.
    final radiansPerPoint = arena.fieldOfView / size.height * 3;

    return Listener(
      behavior: HitTestBehavior.opaque,
      onPointerDown: (event) {
        if (source.followsDevice) {
          arena.strike(event.localPosition, size);
        }
      },
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapUp: (details) {
          if (!source.followsDevice) {
            arena.strike(details.localPosition, size);
          }
        },
        onPanUpdate: (details) => source.drag(
          details.delta.dx * radiansPerPoint,
          -details.delta.dy * radiansPerPoint,
        ),
      ),
    );
  }
}

class _VeilWash extends HookWidget {
  const _VeilWash();

  @override
  Widget build(BuildContext context) => const DecoratedBox(
    decoration: BoxDecoration(
      gradient: RadialGradient(
        radius: 1.1,
        colors: [Color(0x1A0A1420), Color(0x4D05080E), Color(0xB3030407)],
        stops: [0, 0.65, 1],
      ),
    ),
  );
}
