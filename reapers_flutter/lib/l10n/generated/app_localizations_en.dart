// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'Reapers';

  @override
  String get commonBack => 'Back';

  @override
  String get commonLoading => 'Parting the veil…';

  @override
  String get commonExperience => 'Experience';

  @override
  String get commonAnima => 'Anima';

  @override
  String get commonLevel => 'Level';

  @override
  String commonSol(String amount) {
    return '$amount SOL';
  }

  @override
  String get commonRefusedTitle => 'Refused';

  @override
  String get commonDismiss => 'Understood';

  @override
  String get commonFailedTitle => 'Something came loose';

  @override
  String commonFailedBody(String reason) {
    return '$reason';
  }

  @override
  String get commonRetry => 'Try again';

  @override
  String get commonUnreachable =>
      'The keeper of the books could not be reached. Nothing was changed.';

  @override
  String get thresholdOverline => 'Beyond the veil';

  @override
  String get thresholdTagline =>
      'The dead market did not stay dead. What it left behind still drifts between worlds, and someone has to bring it in.';

  @override
  String get thresholdOath => 'Take the oath';

  @override
  String get thresholdOathDetail => 'Be bound to a reaper';

  @override
  String get thresholdShade => 'Walk as a shade';

  @override
  String get thresholdShadeDetail => 'A practice hunt';

  @override
  String get oathOverline => 'The oath';

  @override
  String get oathTitle => 'You do not choose your reaper.';

  @override
  String get oathBody =>
      'The shuffle draws one for you, once. Its class, its face, its light and its name are bound to you from then on. What you make of it is yours.';

  @override
  String get oathSpeak => 'I am bound';

  @override
  String get oathTurning => 'The shuffle turns…';

  @override
  String get oathDrawnOverline => 'The shuffle has drawn';

  @override
  String get oathEnter => 'Enter the sanctum';

  @override
  String get oathOmen => 'Omen';

  @override
  String get sanctumOverline => 'Sanctum';

  @override
  String get sanctumLoose => 'Loose in the veil';

  @override
  String get sanctumNothingLoose =>
      'Nothing is loose. Release an NFT from your vault and it will surface here as a husk.';

  @override
  String sanctumSurfacing(int seconds) {
    return 'Surfacing in ${seconds}s';
  }

  @override
  String get sanctumHunt => 'Hunt';

  @override
  String sanctumSightedBanner(String name) {
    return 'Husk sighted: $name';
  }

  @override
  String get sanctumVault => 'Vault';

  @override
  String sanctumVaultDetail(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count held',
      one: '1 held',
      zero: 'Empty',
    );
    return '$_temp0';
  }

  @override
  String get sanctumArmory => 'Armory';

  @override
  String get sanctumCodex => 'Codex';

  @override
  String sanctumCodexDetail(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count absorbed',
      one: '1 absorbed',
      zero: 'Nothing absorbed',
    );
    return '$_temp0';
  }

  @override
  String get sanctumRifts => 'Rifts';

  @override
  String sanctumRiftsDetail(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count sealed',
      one: '1 sealed',
      zero: 'Hunt together',
    );
    return '$_temp0';
  }

  @override
  String get sanctumColophon => 'Colophon';

  @override
  String get sanctumColophonDetail => 'Music and credits';

  @override
  String get colophonOverline => 'Colophon';

  @override
  String get colophonTitle => 'What this is made of';

  @override
  String get colophonMusic => 'Music';

  @override
  String get colophonMusicOn => 'On';

  @override
  String get colophonMusicOff => 'Off';

  @override
  String get colophonMusicHeading => 'Music';

  @override
  String get colophonSanctum => 'In the sanctum';

  @override
  String get colophonHunt => 'On the hunt';

  @override
  String get colophonRift => 'In a rift';

  @override
  String get colophonBachPiece =>
      'J.S. Bach, Prelude No. 8 in E-flat minor, BWV 853';

  @override
  String get colophonBachCredit =>
      'Kimiko Ishizaka, piano. From the Open Well-Tempered Clavier, released into the public domain. welltemperedclavier.org';

  @override
  String get colophonAnabasisPiece => 'Anabasis I';

  @override
  String get colophonAnabasisCredit =>
      '\'Anabasis I\' by Scott Buckley - released under CC-BY 4.0. www.scottbuckley.com.au';

  @override
  String get colophonKatabasisPiece => 'Katabasis I';

  @override
  String get colophonKatabasisCredit =>
      '\'Katabasis I\' by Scott Buckley - released under CC-BY 4.0. www.scottbuckley.com.au';

  @override
  String get colophonSightHeading => 'Reading the room';

  @override
  String get colophonSight => 'How far away things are';

  @override
  String get colophonDepthPiece => 'Depth Anything V2, small, metric, indoor';

  @override
  String get colophonDepthCredit =>
      'Lihe Yang and others. Released under the Apache License 2.0. It runs on this phone; no picture leaves it.';

  @override
  String get colophonTypeHeading => 'Type';

  @override
  String get colophonType => 'Set in';

  @override
  String get colophonTypePiece => 'Fraunces and DM Sans';

  @override
  String get colophonTypeCredit =>
      'Both released under the SIL Open Font License 1.1.';

  @override
  String get sanctumPractice => 'Practice hunt';

  @override
  String get sanctumPracticeDetail => 'A stray';

  @override
  String get sanctumStatStrike => 'Strike';

  @override
  String get sanctumStatReach => 'Reach';

  @override
  String get sanctumStatCapture => 'Capture';

  @override
  String get sanctumStatResilience => 'Resilience';

  @override
  String get vaultOverline => 'Vault';

  @override
  String get vaultTitle => 'What you hold, and what is for sale.';

  @override
  String get vaultStandIn =>
      'A simulated market. No real SOL changes hands and no real NFT is touched. The collections are real ones from Solana\'s first wave; the creatures are our own models of them, and nothing here is licensed yet.';

  @override
  String get vaultBuy => 'Buy';

  @override
  String get vaultRelease => 'Release into the veil';

  @override
  String get vaultHeld => 'Held';

  @override
  String get vaultLoose => 'Loose';

  @override
  String get vaultAbsorbed => 'Absorbed';

  @override
  String get vaultMint => 'Mint';

  @override
  String get vaultWard => 'Ward';

  @override
  String get vaultWorth => 'Worth';

  @override
  String vaultWorthValue(int xp, int anima) {
    return '$xp xp · $anima anima';
  }

  @override
  String get vaultReleaseNote =>
      'Releasing cannot be undone. The NFT becomes a husk, and you will have to hunt it down.';

  @override
  String get armoryOverline => 'Armory';

  @override
  String get armoryWeapons => 'Weapons';

  @override
  String get armoryOutfits => 'Outfits';

  @override
  String get armoryWield => 'Wield';

  @override
  String get armoryWielded => 'In hand';

  @override
  String get armoryWear => 'Wear';

  @override
  String get armoryWorn => 'Worn';

  @override
  String armoryNeedsLevel(String level) {
    return 'Level $level';
  }

  @override
  String get armoryFree => 'Yours';

  @override
  String armoryAnima(int amount) {
    return '$amount anima';
  }

  @override
  String armoryStrike(String value) {
    return 'Strike ×$value';
  }

  @override
  String armoryTempo(String value) {
    return '$value strikes a second';
  }

  @override
  String get armoryOutfitNote =>
      'An outfit changes how your reaper looks, never how it hunts.';

  @override
  String get codexOverline => 'Codex';

  @override
  String get codexTitle => 'Everything you have absorbed.';

  @override
  String get codexEmpty => 'Nothing yet. Every husk you reap is written here.';

  @override
  String codexSet(int count, int total) {
    return '$count of $total';
  }

  @override
  String get codexSetComplete => 'Set complete · strike +3%';

  @override
  String codexSetProgress(int remaining) {
    String _temp0 = intl.Intl.pluralLogic(
      remaining,
      locale: localeName,
      other: '$remaining more complete the set',
      one: '1 more completes the set',
    );
    return '$_temp0';
  }

  @override
  String get codexLost => 'Husks that got away';

  @override
  String get riftsOverline => 'Rifts';

  @override
  String get riftsTitle => 'Some husks are too much for one reaper.';

  @override
  String get riftsStandIn =>
      'A rift opens at a real place and takes a circle of four. This build does not check where you are standing, so any rift can be entered from anywhere, and shades take the places nobody else does.';

  @override
  String riftsSite(String city, String collection) {
    return '$city · $collection';
  }

  @override
  String get riftsOpen => 'Enter';

  @override
  String riftsAttune(String collection) {
    return 'Absorb one of the $collection and it will open for you.';
  }

  @override
  String get riftLabel => 'Rift';

  @override
  String get riftCircle => 'The circle';

  @override
  String get riftSeatEmpty => 'An empty place';

  @override
  String get riftSeatEmptyDetail => 'A shade will stand in';

  @override
  String get riftSeatShade => 'Shade';

  @override
  String get riftSeatYou => 'You';

  @override
  String get riftSeatReady => 'Ready';

  @override
  String get riftSeatWaiting => 'Arriving';

  @override
  String get riftStandReady => 'Stand ready';

  @override
  String get riftWaiting => 'Waiting for the circle';

  @override
  String riftBegins(int seconds) {
    return 'Begins in $seconds';
  }

  @override
  String get riftEntering => 'Finding the rift…';

  @override
  String get riftLeave => 'Leave the circle';

  @override
  String get riftCastOut => 'You were cast out. The circle fights on.';

  @override
  String get riftCutMade => 'Your cut is made. Hold for the circle.';

  @override
  String get riftCutMissed => 'The cut got away from you. Hold for the circle.';

  @override
  String riftCuts(int made, int needed) {
    return 'Cuts $made of $needed';
  }

  @override
  String get riftSealedLabel => 'Sealed';

  @override
  String get riftSealedTitle => 'The rift is shut';

  @override
  String riftSealedBody(String site) {
    return '$site is quiet again.';
  }

  @override
  String get riftFailedLabel => 'Unsealed';

  @override
  String get riftFailedTitle => 'The rift stays open';

  @override
  String get riftFailedBody =>
      'The circle could not close it this time. Nothing was lost but the chance.';

  @override
  String riftShare(int percent) {
    return '$percent%';
  }

  @override
  String get riftMemberCut => 'Cut';

  @override
  String get riftMemberNoCut => 'No cut';

  @override
  String get riftMemberCastOut => 'Cast out';

  @override
  String get riftLostTitle => 'The rift is gone';

  @override
  String riftLostBody(String reason) {
    return 'The way to it closed: $reason';
  }

  @override
  String get classHarvester => 'Harvester';

  @override
  String get classFerryman => 'Ferryman';

  @override
  String get classWeigher => 'Weigher';

  @override
  String get classChooser => 'Chooser';

  @override
  String get classHollow => 'Hollow';

  @override
  String get classMourner => 'Mourner';

  @override
  String get classHarvesterLine =>
      'The field hand of death, who cuts down many at once.';

  @override
  String get classFerrymanLine =>
      'The one who carries the dead across and cannot be moved.';

  @override
  String get classWeigherLine =>
      'The judge who measures a spirit before taking it.';

  @override
  String get classChooserLine =>
      'The one who picks who falls, then strikes once.';

  @override
  String get classHollowLine =>
      'The rider with no head, who has already lost what you fear to lose.';

  @override
  String get classMournerLine =>
      'The voice that announces a death before it arrives.';

  @override
  String get perkCleave => 'Cleave';

  @override
  String get perkSteadfast => 'Steadfast';

  @override
  String get perkMeasure => 'Measure';

  @override
  String get perkChosenBlow => 'Chosen blow';

  @override
  String get perkChain => 'Chain';

  @override
  String get perkToll => 'Toll';

  @override
  String get perkCleaveLine =>
      'A strike also lands on every open seal near it.';

  @override
  String get perkSteadfastLine => 'The first flinch of a hunt costs nothing.';

  @override
  String get perkMeasureLine =>
      'Seals are numbered. Struck in order, they break faster.';

  @override
  String get perkChosenBlowLine =>
      'Hold firm through a shriek and the next strike lands three times as hard.';

  @override
  String get perkChainLine => 'Cadence builds twice as far.';

  @override
  String get perkTollLine => 'Each shriek you hold through lengthens the reap.';

  @override
  String get weaponScythe => 'Scythe';

  @override
  String get weaponLanternPole => 'Lantern pole';

  @override
  String get weaponBalanceFlail => 'Balance flail';

  @override
  String get weaponSpear => 'Spear';

  @override
  String get weaponChainWhip => 'Chain whip';

  @override
  String get weaponBell => 'Bell';

  @override
  String get temperShrouded => 'Shrouded';

  @override
  String get temperSworn => 'Sworn';

  @override
  String get temperScorched => 'Scorched';

  @override
  String get temperHollowed => 'Hollowed';

  @override
  String get outfitShroudedLine => 'Undyed cloth, rope and wood.';

  @override
  String get outfitSwornLine => 'Dyed dark, shod in iron, carved with marks.';

  @override
  String get outfitScorchedLine =>
      'Charred cloth and pitted metal, still smouldering.';

  @override
  String get outfitHollowedLine =>
      'Parts of you are simply gone. The light is all that is left.';

  @override
  String get rarityCommon => 'Common';

  @override
  String get rarityUncommon => 'Uncommon';

  @override
  String get rarityRare => 'Rare';

  @override
  String get rarityEpic => 'Epic';

  @override
  String get omenNone => 'None';

  @override
  String get omenAshen => 'Ashen';

  @override
  String get omenGilded => 'Gilded';

  @override
  String get omenEclipsed => 'Eclipsed';

  @override
  String get refusalOathAlreadyTaken =>
      'You are already bound. The oath is taken once.';

  @override
  String get refusalOathRequired => 'Only the oathbound may do that.';

  @override
  String get refusalNotListed => 'The market does not list that.';

  @override
  String get refusalAlreadyHeld =>
      'You already hold that, or have absorbed it.';

  @override
  String get refusalNotInVault => 'That is not in your vault.';

  @override
  String get refusalInsufficientFunds =>
      'Your wallet does not hold enough SOL.';

  @override
  String get refusalInsufficientAnima => 'You have not gathered enough anima.';

  @override
  String get refusalLevelTooLow => 'Your reaper is not strong enough yet.';

  @override
  String get refusalNotOwned => 'You have not bought that.';

  @override
  String get refusalAlreadyOwned => 'You already own that.';

  @override
  String get refusalVeilCrowded =>
      'Three husks are already loose. Bring one in first.';

  @override
  String get refusalUnknownSighting => 'That husk is no longer loose.';

  @override
  String get refusalNotSurfaced => 'That husk has not surfaced yet.';

  @override
  String get refusalHuntUnproven => 'The record of that hunt does not hold up.';

  @override
  String get refusalNotAttuned =>
      'This rift only answers a reaper who carries one of its dead. Absorb an NFT from its collection first.';

  @override
  String get scanLabel => 'Reading the room';

  @override
  String get scanTitle => 'Something is nearby.';

  @override
  String get scanFloor =>
      'It keeps to the ground. Tip the phone down and sweep it slowly across the floor in front of you, so the veil knows where it can stand.';

  @override
  String get scanFloorDrag =>
      'It keeps to the ground. Drag down to look at the floor, then drag slowly across it, so the veil knows where it can stand.';

  @override
  String get scanAir =>
      'It keeps to the air. Look around slowly at about head height, so the veil knows where it can go.';

  @override
  String get scanAirDrag =>
      'It keeps to the air. Drag slowly from side to side at about head height, so the veil knows where it can go.';

  @override
  String get scanReading => 'Reading. Keep going.';

  @override
  String get scanSlower => 'Slower. Nothing can be made out.';

  @override
  String get scanLower => 'Too high. Tip the phone down.';

  @override
  String get scanHigher => 'Too low. Tip the phone up.';

  @override
  String get scanFindRoom =>
      'There is no room for it here. Turn toward somewhere more open.';

  @override
  String scanFloorFound(String metres) {
    return 'Floor $metres m below the phone';
  }

  @override
  String get scanSkip => 'Hunt anyway';

  @override
  String get scanProgress => 'Read';

  @override
  String get scanFound => 'It is here';

  @override
  String get feedingMeagre => 'Meagre ×0.85';

  @override
  String get feedingFair => 'Fair ×1';

  @override
  String get feedingRich => 'Rich ×1.25';

  @override
  String get feedingFeast => 'A feast ×2';

  @override
  String get huntFeeding => 'Feeding';

  @override
  String get huntReceipt => 'Receipt';

  @override
  String get huntReceiptSigned => 'Signed by the keeper';

  @override
  String get huntReceiptUnsigned => 'Unsigned, played offline';

  @override
  String get huntSightedLabel => 'Husk sighted';

  @override
  String get huntPracticeLabel => 'A stray, for practice';

  @override
  String get huntSafety =>
      'Stand still to hunt, and stay aware of what is around you. You only need to turn on the spot.';

  @override
  String get huntBegin => 'Raise the blade';

  @override
  String get huntHowStalk =>
      'Find it and hold the ring on it, still, to ambush it.';

  @override
  String get huntHowStrike =>
      'Tap the lit seals on its body to wear down its ward.';

  @override
  String get huntHowHold =>
      'When it draws breath, freeze. Do not move and do not tap.';

  @override
  String get huntHowCharge =>
      'When it runs at you, swing the phone hard across it.';

  @override
  String get huntHowReap =>
      'When its ward breaks, sweep the phone slowly along the arc. That is the reap.';

  @override
  String get huntDifficulty => 'Testing difficulty';

  @override
  String get difficultyEasy => 'Easy';

  @override
  String get difficultyNormal => 'Normal';

  @override
  String get difficultyHard => 'Hard';

  @override
  String get huntWard => 'Ward';

  @override
  String get huntResolve => 'Resolve';

  @override
  String get huntReapWindow => 'Reap';

  @override
  String huntSeconds(int seconds) {
    return '${seconds}s';
  }

  @override
  String huntCadence(int hits) {
    return 'Cadence ×$hits';
  }

  @override
  String get huntPromptStalk => 'Turn slowly. It has not seen you.';

  @override
  String get huntPromptStalkDrag =>
      'Drag to look around. Move slowly: it has not seen you.';

  @override
  String get huntPromptSteady => 'Hold steady on it.';

  @override
  String get huntPromptStrike => 'Strike the open seals.';

  @override
  String get huntPromptWindup => 'It is drawing breath.';

  @override
  String get huntPromptHold => 'Do not flinch';

  @override
  String get huntPromptCharge => 'It is coming. Swing across it.';

  @override
  String get huntParried => 'Turned aside';

  @override
  String get huntStruck => 'It struck you';

  @override
  String get huntPromptMark => 'Bring the blade to the mark.';

  @override
  String get huntPromptSweep => 'Sweep along the arc. Slowly.';

  @override
  String get huntAmbush => 'Ambush';

  @override
  String get huntAmbushBonus => '+10%';

  @override
  String get huntNoticed => 'It sees you';

  @override
  String get huntFlinched => 'Flinched';

  @override
  String get huntSteadfast => 'Steadfast';

  @override
  String get huntHeldFirm => 'Unflinching';

  @override
  String get huntBroken => 'Ward broken';

  @override
  String get huntRecovered => 'It gathers itself';

  @override
  String get huntReapedLabel => 'Reaped';

  @override
  String get huntLostLabel => 'Lost';

  @override
  String get huntGradePerfect => 'A perfect reap';

  @override
  String get huntGradeClean => 'A clean reap';

  @override
  String get huntGradeRagged => 'A ragged reap';

  @override
  String get huntEscapedTitle => 'It slipped away';

  @override
  String get huntEscapedResolve =>
      'You flinched until nothing was left to hold it with. It is still out there.';

  @override
  String get huntEscapedTime =>
      'The fight ran too long and the husk found a way out. It is still out there.';

  @override
  String get huntEscapedReap =>
      'The reap was left unfinished too often. It is still out there.';

  @override
  String huntAbsorbed(String name) {
    return '$name is part of you now.';
  }

  @override
  String get huntPracticeDone =>
      'A stray, so nothing was gained and nothing was owed.';

  @override
  String get huntLeave => 'Lower the blade';

  @override
  String get huntFailedTitle => 'The veil would not part';

  @override
  String huntFailedBody(String reason) {
    return 'The hunt could not be started: $reason';
  }
}
