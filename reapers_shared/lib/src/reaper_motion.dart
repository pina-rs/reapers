import 'reaper_class.dart';

/// Authored clip IDs and timings shared by the art builder and both renderers.
/// These are presentation timings; they never award damage or settle a hunt.
enum ReaperMotion {
  idle(4, MotionGroup.movement, loops: true),
  creep(3.2, MotionGroup.movement, loops: true),
  walk(2, MotionGroup.movement, loops: true),
  run(1, MotionGroup.movement, loops: true),
  sprint(0.7, MotionGroup.movement, loops: true),
  jump(1.4, MotionGroup.movement),
  land(0.7, MotionGroup.movement),
  dodgeLeft(0.8, MotionGroup.movement),
  dodgeRight(0.8, MotionGroup.movement),
  dash(0.8, MotionGroup.movement),
  dashLeft(0.8, MotionGroup.movement),
  dashRight(0.8, MotionGroup.movement),
  laugh(2.4, MotionGroup.reaction, unarmed: true),
  cry(3.2, MotionGroup.reaction, unarmed: true),
  rage(2, MotionGroup.reaction, unarmed: true),
  fear(1.8, MotionGroup.reaction, unarmed: true),
  bow(2.4, MotionGroup.reaction),
  victory(2.8, MotionGroup.reaction),
  gigachad(3, MotionGroup.meme, unarmed: true),
  sigma(2.4, MotionGroup.meme, unarmed: true),
  shrug(2.2, MotionGroup.meme, unarmed: true),
  absoluteCinema(3, MotionGroup.meme, unarmed: true),
  tPose(2.4, MotionGroup.meme, unarmed: true),
  animeCharge(2.4, MotionGroup.combat, unarmed: true),
  slash(1.2, MotionGroup.combat, impact: 0.45),
  thrust(1, MotionGroup.combat, impact: 0.45),
  parry(1.2, MotionGroup.combat),
  finisherReap(2.8, MotionGroup.finisher, impact: 0.52),
  finisherImpale(2.6, MotionGroup.finisher, impact: 0.46),
  finisherSoulCrush(3, MotionGroup.finisher, unarmed: true, impact: 0.6),
  finisherPalm(2.2, MotionGroup.finisher, unarmed: true, impact: 0.5),
  finisherUppercut(2.4, MotionGroup.finisher, unarmed: true, impact: 0.48);

  const ReaperMotion(
    this.seconds,
    this.group, {
    this.loops = false,
    this.unarmed = false,
    this.impact,
  });

  final double seconds;
  final MotionGroup group;
  final bool loops;
  final bool unarmed;

  /// Normalized contact point for a future synchronized enemy reaction.
  final double? impact;

  bool supports(WeaponKind weapon) => switch (this) {
    finisherImpale => weapon == WeaponKind.spear,
    finisherReap =>
      weapon == WeaponKind.scythe || weapon == WeaponKind.chainWhip,
    _ => true,
  };
}

/// Studio filters; these do not add player progression or unlock rules.
enum MotionGroup { movement, reaction, meme, combat, finisher }
