import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/hunt_arena.dart';
import 'package:reapers_flutter/hunt/pose/pose_sources.dart';
import 'package:reapers_shared/reapers_shared.dart';

const _view = Size(402, 874);
const _saint = Nft(collection: NftCollection.degenApes, serial: 1);

HuntSim _sim() => HuntSim(
  seed: 7,
  vessel: Vessel.penguin,
  ward: Rarity.common.ward,
  stats: HuntStats.derive(HuntLoadout.borrowed),
);

/// An arena looked around by dragging, as on a simulator. The husk is never
/// loaded, so nothing here needs a GPU or a motion sensor.
HuntArena _arena() => HuntArena(
  vessel: Vessel.penguin,
  relics: const [_saint],
  poseSource: DragPoseSource(),
);

/// An arena with a hunt engaged and the view turned to face the husk.
HuntArena _facingTheHusk() {
  final arena = _arena()
    ..engage(_sim())
    ..viewSize = _view;
  final husk = arena.controller!.sim.pose;

  arena.poseSource.drag(husk.yaw, husk.pitch);
  addTearDown(arena.dispose);

  return arena;
}

/// Where [direction] falls on the screen.
Offset _onScreen(HuntArena arena, Vec3 direction) {
  final point = arena.pose.project(
    direction,
    verticalFov: huntFieldOfView,
    aspect: _view.aspectRatio,
  )!;

  return Offset(
    (point.x + 1) / 2 * _view.width,
    (1 - point.y) / 2 * _view.height,
  );
}

void main() {
  test('a drag turns the view', () {
    final arena = _facingTheHusk();
    final husk = arena.controller!.sim.pose;

    expect(arena.pose.forward.angleTo(husk.direction), lessThan(radians(0.5)));
  });

  test('a touch on the husk is a strike on the husk', () {
    final arena = _facingTheHusk();
    final controller = arena.controller!;

    arena.strike(_onScreen(arena, controller.sim.pose.direction), _view);

    final events = controller.frame(1 / HuntRules.tickRate);

    expect(events.whereType<StalkEnded>().single.ambush, isFalse);
    expect(controller.sim.phase, HuntPhase.fight);
    expect(controller.recorded.single.taps, hasLength(1));
    expect(arena.slashes, hasLength(1));
  });

  test('a touch on an open seal takes ward off it', () {
    final arena = _facingTheHusk();
    final controller = arena.controller!;
    final sim = controller.sim;

    // Strike the body to start the fight, then wait for a seal to open.
    arena.strike(_onScreen(arena, sim.pose.direction), _view);

    var seal = -1;

    for (var tick = 0; tick < 5 * HuntRules.tickRate && seal < 0; tick++) {
      controller.frame(1 / HuntRules.tickRate);
      // The view keeps facing the husk as it drifts.
      arena.poseSource.drag(
        wrapAngle(sim.pose.yaw - arena.pose.forward.yaw),
        sim.pose.pitch - arena.pose.forward.pitch,
      );
      seal =
          [
            for (var index = 0; index < sim.vessel.seals.length; index++)
              if (sim.sealOpen(index)) index,
          ].firstOrNull ??
          -1;
    }

    expect(seal, isNot(-1), reason: 'a seal opens within five seconds');

    final wardBefore = sim.ward;

    // Let the swing from the opening strike finish.
    for (var tick = 0; tick < sim.stats.swingTicks; tick++) {
      controller.frame(1 / HuntRules.tickRate);
    }

    arena.strike(_onScreen(arena, sim.pose.sealDirection(seal)), _view);

    final events = controller.frame(1 / HuntRules.tickRate);

    expect(events.whereType<StrikeLanded>().single.seal, seal);
    expect(sim.ward, lessThan(wardBefore));
  });

  test('a touch does nothing once the ward is broken', () {
    final arena = _facingTheHusk();
    final controller = arena.controller!;
    final bot = HuntBot(controller.sim);

    while (controller.sim.phase != HuntPhase.broken) {
      controller.sim.step(bot.decide());
    }

    arena.strike(_view.center(Offset.zero), _view);

    expect(arena.slashes, isEmpty);
    expect(controller.frame(1 / HuntRules.tickRate), isEmpty);
  });

  test('before a fight is engaged a touch is ignored', () {
    final arena = _arena();

    addTearDown(arena.dispose);
    arena.strike(_view.center(Offset.zero), _view);

    expect(arena.controller, isNull);
    expect(arena.slashes, isEmpty);
  });

  test('a rematch clears the previous hunt without replacing the camera', () {
    final arena = _facingTheHusk();
    final camera = arena.feed;
    final sim = arena.controller!.sim;
    final bot = HuntBot(sim);

    while (!sim.isOver) {
      sim.step(bot.decide());
    }

    arena
      ..comeApart = 1
      ..wound = 1;
    arena.gone.value = true;
    arena.engage(_sim());

    expect(arena.controller!.sim.isOver, isFalse);
    expect(arena.controller!.recorded, isEmpty);
    expect(arena.comeApart, 0);
    expect(arena.wound, 0);
    expect(arena.gone.value, isFalse);
    expect(arena.feed, same(camera));
  });
}
