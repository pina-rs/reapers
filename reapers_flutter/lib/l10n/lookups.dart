import 'package:reapers_shared/reapers_shared.dart';

import 'generated/app_localizations.dart';

/// Display names for the things the shared rules only know by identifier.
///
/// The rules stay language-free: they speak in enums, and these functions
/// turn an enum into the player's language. Every switch is exhaustive, so a
/// new class or weapon cannot ship without its name.

String className(AppLocalizations l10n, ReaperClass reaperClass) =>
    switch (reaperClass) {
      ReaperClass.harvester => l10n.classHarvester,
      ReaperClass.ferryman => l10n.classFerryman,
      ReaperClass.weigher => l10n.classWeigher,
      ReaperClass.chooser => l10n.classChooser,
      ReaperClass.hollow => l10n.classHollow,
      ReaperClass.mourner => l10n.classMourner,
    };

String classLine(AppLocalizations l10n, ReaperClass reaperClass) =>
    switch (reaperClass) {
      ReaperClass.harvester => l10n.classHarvesterLine,
      ReaperClass.ferryman => l10n.classFerrymanLine,
      ReaperClass.weigher => l10n.classWeigherLine,
      ReaperClass.chooser => l10n.classChooserLine,
      ReaperClass.hollow => l10n.classHollowLine,
      ReaperClass.mourner => l10n.classMournerLine,
    };

String perkName(AppLocalizations l10n, ClassPerk perk) => switch (perk) {
  ClassPerk.cleave => l10n.perkCleave,
  ClassPerk.steadfast => l10n.perkSteadfast,
  ClassPerk.measure => l10n.perkMeasure,
  ClassPerk.chosenBlow => l10n.perkChosenBlow,
  ClassPerk.chain => l10n.perkChain,
  ClassPerk.toll => l10n.perkToll,
};

String perkLine(AppLocalizations l10n, ClassPerk perk) => switch (perk) {
  ClassPerk.cleave => l10n.perkCleaveLine,
  ClassPerk.steadfast => l10n.perkSteadfastLine,
  ClassPerk.measure => l10n.perkMeasureLine,
  ClassPerk.chosenBlow => l10n.perkChosenBlowLine,
  ClassPerk.chain => l10n.perkChainLine,
  ClassPerk.toll => l10n.perkTollLine,
};

String weaponName(AppLocalizations l10n, WeaponKind kind) => switch (kind) {
  WeaponKind.scythe => l10n.weaponScythe,
  WeaponKind.lanternPole => l10n.weaponLanternPole,
  WeaponKind.balanceFlail => l10n.weaponBalanceFlail,
  WeaponKind.spear => l10n.weaponSpear,
  WeaponKind.chainWhip => l10n.weaponChainWhip,
  WeaponKind.bell => l10n.weaponBell,
};

String temperName(AppLocalizations l10n, Temper temper) => switch (temper) {
  Temper.shrouded => l10n.temperShrouded,
  Temper.sworn => l10n.temperSworn,
  Temper.scorched => l10n.temperScorched,
  Temper.hollowed => l10n.temperHollowed,
};

/// An outfit is named after the same four states as a temper.
String outfitName(AppLocalizations l10n, Outfit outfit) => switch (outfit) {
  Outfit.shrouded => l10n.temperShrouded,
  Outfit.sworn => l10n.temperSworn,
  Outfit.scorched => l10n.temperScorched,
  Outfit.hollowed => l10n.temperHollowed,
};

String outfitLine(AppLocalizations l10n, Outfit outfit) => switch (outfit) {
  Outfit.shrouded => l10n.outfitShroudedLine,
  Outfit.sworn => l10n.outfitSwornLine,
  Outfit.scorched => l10n.outfitScorchedLine,
  Outfit.hollowed => l10n.outfitHollowedLine,
};

String rarityName(AppLocalizations l10n, Rarity rarity) => switch (rarity) {
  Rarity.common => l10n.rarityCommon,
  Rarity.uncommon => l10n.rarityUncommon,
  Rarity.rare => l10n.rarityRare,
  Rarity.epic => l10n.rarityEpic,
};

String feedingName(AppLocalizations l10n, Feeding feeding) => switch (feeding) {
  Feeding.meagre => l10n.feedingMeagre,
  Feeding.fair => l10n.feedingFair,
  Feeding.rich => l10n.feedingRich,
  Feeding.feast => l10n.feedingFeast,
};

String omenName(AppLocalizations l10n, Omen omen) => switch (omen) {
  Omen.none => l10n.omenNone,
  Omen.ashen => l10n.omenAshen,
  Omen.gilded => l10n.omenGilded,
  Omen.eclipsed => l10n.omenEclipsed,
};

String refusalMessage(AppLocalizations l10n, Refusal refusal) =>
    switch (refusal) {
      Refusal.oathAlreadyTaken => l10n.refusalOathAlreadyTaken,
      Refusal.oathRequired => l10n.refusalOathRequired,
      Refusal.notListed => l10n.refusalNotListed,
      Refusal.alreadyHeld => l10n.refusalAlreadyHeld,
      Refusal.notInVault => l10n.refusalNotInVault,
      Refusal.insufficientFunds => l10n.refusalInsufficientFunds,
      Refusal.insufficientAnima => l10n.refusalInsufficientAnima,
      Refusal.levelTooLow => l10n.refusalLevelTooLow,
      Refusal.notOwned => l10n.refusalNotOwned,
      Refusal.alreadyOwned => l10n.refusalAlreadyOwned,
      Refusal.veilCrowded => l10n.refusalVeilCrowded,
      Refusal.unknownSighting => l10n.refusalUnknownSighting,
      Refusal.notSurfaced => l10n.refusalNotSurfaced,
      Refusal.huntUnproven => l10n.refusalHuntUnproven,
      Refusal.huntVersionUnsupported => l10n.refusalHuntVersionUnsupported,
      Refusal.marketUnavailable => l10n.refusalMarketUnavailable,
      Refusal.riftUnproven => l10n.refusalRiftUnproven,
      Refusal.notAttuned => l10n.refusalNotAttuned,
      Refusal.locationUnusable => l10n.locationInaccurate,
      Refusal.reaperBusy => l10n.refusalReaperBusy,
      Refusal.walletUnproven => l10n.refusalWalletUnproven,
      Refusal.chainUnconfigured => l10n.refusalChainUnconfigured,
      Refusal.chainUnavailable => l10n.commonUnreachable,
      Refusal.rosterFull => l10n.refusalRosterFull,
    };

String auraName(AppLocalizations l10n, Aura aura) => switch (aura) {
  Aura.motes => l10n.studioAuraMotes,
  Aura.embers => l10n.studioAuraEmbers,
  Aura.mist => l10n.studioAuraMist,
  Aura.wisps => l10n.studioAuraWisps,
};

/// A balance in lamports as SOL, to two or three places.
String solAmount(int lamports) {
  final sol = lamports / lamportsPerSol;

  return sol.toStringAsFixed(sol * 100 == (sol * 100).roundToDouble() ? 2 : 3);
}

String difficultyName(AppLocalizations l10n, Difficulty difficulty) =>
    switch (difficulty) {
      Difficulty.easy => l10n.difficultyEasy,
      Difficulty.normal => l10n.difficultyNormal,
      Difficulty.hard => l10n.difficultyHard,
    };
