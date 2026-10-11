import 'package:reapers_shared/reapers_shared.dart';

import 'l10n.dart';

String motionName(AppLocalizations l10n, ReaperMotion motion) =>
    switch (motion) {
      ReaperMotion.idle => l10n.motionIdle,
      ReaperMotion.creep => l10n.motionCreep,
      ReaperMotion.walk => l10n.motionWalk,
      ReaperMotion.run => l10n.motionRun,
      ReaperMotion.sprint => l10n.motionSprint,
      ReaperMotion.jump => l10n.motionJump,
      ReaperMotion.land => l10n.motionLand,
      ReaperMotion.dodgeLeft => l10n.motionDodgeLeft,
      ReaperMotion.dodgeRight => l10n.motionDodgeRight,
      ReaperMotion.dash => l10n.motionDash,
      ReaperMotion.dashLeft => l10n.motionDashLeft,
      ReaperMotion.dashRight => l10n.motionDashRight,
      ReaperMotion.laugh => l10n.motionLaugh,
      ReaperMotion.cry => l10n.motionCry,
      ReaperMotion.rage => l10n.motionRage,
      ReaperMotion.fear => l10n.motionFear,
      ReaperMotion.bow => l10n.motionBow,
      ReaperMotion.victory => l10n.motionVictory,
      ReaperMotion.gigachad => l10n.motionGigachad,
      ReaperMotion.sigma => l10n.motionSigma,
      ReaperMotion.shrug => l10n.motionShrug,
      ReaperMotion.absoluteCinema => l10n.motionAbsoluteCinema,
      ReaperMotion.tPose => l10n.motionTPose,
      ReaperMotion.animeCharge => l10n.motionAnimeCharge,
      ReaperMotion.slash => l10n.motionSlash,
      ReaperMotion.thrust => l10n.motionThrust,
      ReaperMotion.parry => l10n.motionParry,
      ReaperMotion.finisherReap => l10n.motionFinisherReap,
      ReaperMotion.finisherImpale => l10n.motionFinisherImpale,
      ReaperMotion.finisherSoulCrush => l10n.motionFinisherSoulCrush,
      ReaperMotion.finisherPalm => l10n.motionFinisherPalm,
      ReaperMotion.finisherUppercut => l10n.motionFinisherUppercut,
    };

String motionGroupName(AppLocalizations l10n, MotionGroup group) =>
    switch (group) {
      MotionGroup.movement => l10n.motionMovement,
      MotionGroup.reaction => l10n.motionReaction,
      MotionGroup.meme => l10n.motionMeme,
      MotionGroup.combat => l10n.motionCombat,
      MotionGroup.finisher => l10n.motionFinisher,
    };
