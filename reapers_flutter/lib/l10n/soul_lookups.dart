import 'package:reapers_shared/reapers_shared.dart';

import 'generated/app_localizations.dart';

String soulPathName(AppLocalizations l10n, SoulPath path) => switch (path) {
  SoulPath.unseen => l10n.soulUnseen,
  SoulPath.stillness => l10n.soulStillness,
  SoulPath.rhythm => l10n.soulRhythm,
};

String soulPathHelp(AppLocalizations l10n, SoulPath path) => switch (path) {
  SoulPath.unseen => l10n.soulUnseenHelp,
  SoulPath.stillness => l10n.soulStillnessHelp,
  SoulPath.rhythm => l10n.soulRhythmHelp,
};

String temperamentName(AppLocalizations l10n, Temperament value) =>
    switch (value) {
      Temperament.patient => l10n.temperamentPatient,
      Temperament.keen => l10n.temperamentKeen,
      Temperament.stalwart => l10n.temperamentStalwart,
      Temperament.hungry => l10n.temperamentHungry,
    };

String temperamentHelp(AppLocalizations l10n, Temperament value) =>
    switch (value) {
      Temperament.patient => l10n.temperamentPatientHelp,
      Temperament.keen => l10n.temperamentKeenHelp,
      Temperament.stalwart => l10n.temperamentStalwartHelp,
      Temperament.hungry => l10n.temperamentHungryHelp,
    };

String giftName(AppLocalizations l10n, ReaperGift gift) => switch (gift) {
  ReaperGift.furrow => l10n.giftFurrow,
  ReaperGift.aftermath => l10n.giftAftermath,
  ReaperGift.gathering => l10n.giftGathering,
  ReaperGift.passage => l10n.giftPassage,
  ReaperGift.anchor => l10n.giftAnchor,
  ReaperGift.returnLight => l10n.giftReturnLight,
  ReaperGift.foreknowledge => l10n.giftForeknowledge,
  ReaperGift.verdict => l10n.giftVerdict,
  ReaperGift.equilibrium => l10n.giftEquilibrium,
  ReaperGift.firstLight => l10n.giftFirstLight,
  ReaperGift.reprisal => l10n.giftReprisal,
  ReaperGift.lastWord => l10n.giftLastWord,
  ReaperGift.hunger => l10n.giftHunger,
  ReaperGift.echo => l10n.giftEcho,
  ReaperGift.unbroken => l10n.giftUnbroken,
  ReaperGift.hush => l10n.giftHush,
  ReaperGift.keening => l10n.giftKeening,
  ReaperGift.refrain => l10n.giftRefrain,
};

String giftLore(AppLocalizations l10n, ReaperGift gift) => switch (gift) {
  ReaperGift.furrow => l10n.giftFurrowLore,
  ReaperGift.aftermath => l10n.giftAftermathLore,
  ReaperGift.gathering => l10n.giftGatheringLore,
  ReaperGift.passage => l10n.giftPassageLore,
  ReaperGift.anchor => l10n.giftAnchorLore,
  ReaperGift.returnLight => l10n.giftReturnLightLore,
  ReaperGift.foreknowledge => l10n.giftForeknowledgeLore,
  ReaperGift.verdict => l10n.giftVerdictLore,
  ReaperGift.equilibrium => l10n.giftEquilibriumLore,
  ReaperGift.firstLight => l10n.giftFirstLightLore,
  ReaperGift.reprisal => l10n.giftReprisalLore,
  ReaperGift.lastWord => l10n.giftLastWordLore,
  ReaperGift.hunger => l10n.giftHungerLore,
  ReaperGift.echo => l10n.giftEchoLore,
  ReaperGift.unbroken => l10n.giftUnbrokenLore,
  ReaperGift.hush => l10n.giftHushLore,
  ReaperGift.keening => l10n.giftKeeningLore,
  ReaperGift.refrain => l10n.giftRefrainLore,
};

/// Copy takes every changing value from the same rules as the simulation.
String giftEffect(AppLocalizations l10n, ReaperGift gift, int strength) {
  final bonus = (SoulRules.giftStrikePerStrength * strength * 100).round();
  String seconds(int ticks) => (ticks / HuntRules.tickRate).toStringAsFixed(1);

  return switch (gift) {
    ReaperGift.furrow => l10n.giftFurrowEffect(
      ((HuntRules.cleaveShare + SoulRules.furrowCleavePerStrength * strength) *
              100)
          .round(),
    ),
    ReaperGift.aftermath => l10n.giftAftermathEffect(
      seconds(
        HuntRules.exposedTicks + SoulRules.gatheringTicksPerStrength * strength,
      ),
    ),
    ReaperGift.gathering => l10n.giftGatheringEffect(
      seconds(
        HuntRules.sealCooldownTicks +
            SoulRules.gatheringTicksPerStrength * strength,
      ),
    ),
    ReaperGift.passage => l10n.giftPassageEffect(
      seconds(
        SoulRules.passageStaggerTicksPerStrength * strength,
      ),
    ),
    ReaperGift.anchor => l10n.giftAnchorEffect(strength),
    ReaperGift.returnLight => l10n.giftReturnLightEffect(strength),
    ReaperGift.foreknowledge => l10n.giftForeknowledgeEffect(
      seconds(
        SoulRules.foreknowledgeTicksPerStrength * strength,
      ),
    ),
    ReaperGift.verdict => l10n.giftVerdictEffect(bonus),
    ReaperGift.equilibrium => l10n.giftEquilibriumEffect(
      seconds(
        HuntRules.sealCooldownTicks +
            SoulRules.equilibriumTicksPerStrength * strength,
      ),
    ),
    ReaperGift.firstLight => l10n.giftFirstLightEffect(bonus),
    ReaperGift.reprisal => l10n.giftReprisalEffect(bonus),
    ReaperGift.lastWord => l10n.giftLastWordEffect(bonus),
    ReaperGift.hunger => l10n.giftHungerEffect(strength),
    ReaperGift.echo => l10n.giftEchoEffect(strength),
    ReaperGift.unbroken => l10n.giftUnbrokenEffect(strength),
    ReaperGift.hush => l10n.giftHushEffect(
      seconds(
        SoulRules.hushTicksPerStrength * strength,
      ),
    ),
    ReaperGift.keening => l10n.giftKeeningEffect(bonus),
    ReaperGift.refrain => l10n.giftRefrainEffect(
      seconds(
        SoulRules.refrainTicksPerStrength * strength,
      ),
    ),
  };
}
