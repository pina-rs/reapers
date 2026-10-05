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
  String get startupOpeningBooks => 'Opening your books…';

  @override
  String get startupUnreadable =>
      'Your books could not be opened. Try again to recover your place in the veil.';

  @override
  String get riftsLocationNote =>
      'Location helps your circle face the same husk, only while a rift is open and on screen. Online play shares your latest reading, never a trail. It is optional.';

  @override
  String get locationDenied =>
      'Location permission was declined. The husk will stand ahead of you. Allow location to share its place with your circle.';

  @override
  String get locationBlocked =>
      'Location permission is off for Reapers. You can allow it in Settings, or play with the husk ahead of you.';

  @override
  String get locationServices =>
      'Location services are off. Turn them on to find where your circle stands.';

  @override
  String get locationUnsupported =>
      'This device cannot read its location. The husk will stand ahead of you.';

  @override
  String get locationUnavailable =>
      'Your location could not be read. You can try again or keep playing.';

  @override
  String get locationInaccurate =>
      'Waiting for a more precise location. An open view of the sky helps; the husk will stand ahead of you meanwhile.';

  @override
  String get locationStale =>
      'Your location reading is too old. Waiting for a fresh reading before placing the rift.';

  @override
  String get locationSharingFailed =>
      'Your location could not reach the keeper. The next fresh reading will try again.';

  @override
  String get locationSettings => 'Open Settings';

  @override
  String get locationSettingsFailed =>
      'Settings could not be opened. Open your device Settings to allow location for Reapers.';

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
      'The keeper of the books could not be reached. Try again to recover your place in the veil.';

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
      'A rift takes a circle of four. Stand at its site and it is there. Anywhere else, it takes its place 25 metres in front of the first reaper to arrive, and everyone who joins sees it in that same spot. Shades take the places nobody else does.';

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
  String get riftFinding =>
      'Finding where you stand, so the rift can take its place.';

  @override
  String get riftUnplaced =>
      'This phone cannot say where it is, so the rift stands dead ahead of you.';

  @override
  String get riftNoCompass =>
      'This phone has no compass to say which way the rift lies, so it stands dead ahead of you.';

  @override
  String riftStands(int metres) {
    return 'The rift stands $metres m from you. Everyone here sees it in the same place: turn until you find it.';
  }

  @override
  String get riftStandsFar =>
      'The rift stands too far from here to point at, so you see it dead ahead.';

  @override
  String riftMeasured(int accuracy, String latitude, String longitude) {
    return '(You: $latitude, $longitude, within $accuracy m.)';
  }

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
  String get scanLabel => 'Drawing it out';

  @override
  String get scanTitle => 'Something is nearby.';

  @override
  String get sigilHow =>
      'Trace its sigil to draw it out. Turn until the ring rests on the first light, hold the phone still until it catches, then find the next.';

  @override
  String get sigilHowDrag =>
      'Trace its sigil to draw it out. Drag until the ring rests on the first light, wait there until it catches, then find the next.';

  @override
  String sigilCaught(int caught, int total) {
    return '$caught of $total caught';
  }

  @override
  String get sigilSteady => 'Hold still. It cannot catch while you move.';

  @override
  String get sigilSlipping => 'It is slipping. Find the next light.';

  @override
  String scanFloorFound(String metres) {
    return 'Floor $metres m below the phone';
  }

  @override
  String get scanSkip => 'Hunt anyway';

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
  String get huntHowSigil =>
      'Trace its sigil: rest the ring on each light in turn until it catches.';

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

  @override
  String get studioAppTitle => 'Reaper studio';

  @override
  String get studioTitle => 'A life after death.';

  @override
  String get studioIntro =>
      'Shape a specimen. Explore its growth. Everything here is a preview; your bound reaper stays yours.';

  @override
  String get studioTraits => 'Birth traits';

  @override
  String get studioGrowth => 'Growth';

  @override
  String get studioClass => 'Calling';

  @override
  String get studioHead => 'Head';

  @override
  String get studioMantle => 'Mantle';

  @override
  String get studioRobe => 'Hem';

  @override
  String get studioPalette => 'Colourway';

  @override
  String get studioOmen => 'Omen';

  @override
  String get studioAura => 'Aura';

  @override
  String get studioStature => 'Stature';

  @override
  String get studioGauntness => 'Gauntness';

  @override
  String get studioGaze => 'Gaze';

  @override
  String get studioTatter => 'Tatter';

  @override
  String get studioDrag => 'Drag to turn your reaper';

  @override
  String get studioShuffle => 'Draw another specimen';

  @override
  String get studioBirthTraits =>
      'Birth traits are fixed when a reaper is minted. Outfits and weapons can change as it grows.';

  @override
  String get studioCopy => 'Copy genome';

  @override
  String get studioCopied => 'Genome copied';

  @override
  String get studioLevel => 'Level';

  @override
  String get studioStrength => 'Strike growth';

  @override
  String get studioDamage => 'Strike damage';

  @override
  String get studioGrowthLine =>
      'Compared with a new reaper of this calling, carrying its starting weapon. Levels strengthen strikes; they do not widen the capture window.';

  @override
  String get studioAbsorb => 'Absorb a specimen';

  @override
  String get studioExample =>
      'Illustration: a rare husk, a clean cut and fair feeding. Real rewards depend on rarity, cut, ambush and feeding.';

  @override
  String get studioWeapon => 'Weapon temper';

  @override
  String get studioOutfit => 'Outfit';

  @override
  String get studioCosmetic =>
      'Outfits change the silhouette and light. Weapons change your strikes. In the game, both cost earned anima after reaching their required level.';

  @override
  String get studioPotential => 'See full potential';

  @override
  String get studioMultiple =>
      'Each owned reaper grows separately. Anima belongs to you; an NFT reaper’s experience, codex and equipment follow that asset when traded.';

  @override
  String get studioAuraMotes => 'Motes';

  @override
  String get studioAuraEmbers => 'Embers';

  @override
  String get studioAuraMist => 'Mist';

  @override
  String get studioAuraWisps => 'Wisps';

  @override
  String studioLevelValue(int level) {
    return 'Level $level';
  }

  @override
  String studioVariant(int number) {
    return '$number';
  }

  @override
  String studioStronger(int percent) {
    return '+$percent%';
  }

  @override
  String studioCaptures(int count) {
    return '$count absorbed';
  }

  @override
  String studioReward(int xp) {
    return '+$xp XP in this preview';
  }

  @override
  String studioUnlock(int level, int anima) {
    return 'Level $level · $anima anima';
  }

  @override
  String get refusalReaperBusy =>
      'Finish your loose hunts before switching reapers.';

  @override
  String get refusalWalletUnproven =>
      'The wallet proof could not be verified. Connect your wallet again.';

  @override
  String get refusalChainUnconfigured =>
      'Reaper NFT trading is waiting for the collection to open.';

  @override
  String get sanctumRoster => 'Your reapers';

  @override
  String get sanctumStudio => 'Reaper studio';

  @override
  String get sanctumStudioDetail => 'Explore traits and growth';

  @override
  String get rosterTitle => 'Your reapers';

  @override
  String get rosterIntro =>
      'Choose who hunts tonight. Every reaper carries its own growth.';

  @override
  String get rosterStarter => 'Oathbound · not tradable';

  @override
  String get rosterNft => 'Solana NFT';

  @override
  String get rosterActive => 'Hunting tonight';

  @override
  String get rosterConnect => 'Connect Solana wallet';

  @override
  String get rosterRefresh => 'Refresh ownership';

  @override
  String get rosterImport => 'Import owned reaper';

  @override
  String get rosterAsset => 'Reaper NFT address';

  @override
  String get rosterImportHelp =>
      'Paste the address of a Reapers NFT already owned by your linked wallet.';

  @override
  String get rosterBuy => 'Find another reaper';

  @override
  String get rosterBuyHelp =>
      'Buy from the collection’s marketplace, then import your NFT here. Your wallet handles payment.';

  @override
  String get rosterOffline =>
      'NFT ownership needs the online keeper. You can explore specimens in the studio while playing offline.';

  @override
  String get rosterWalletUnsupported =>
      'Connect on an Android phone with a Solana wallet installed.';

  @override
  String get rosterLinkHelp =>
      'Connecting proves control of your wallet. It does not spend SOL or transfer an NFT.';

  @override
  String get rosterLinked => 'Linked wallet';

  @override
  String get rosterBalance => 'Wallet SOL';

  @override
  String get rosterNetwork => 'Network';

  @override
  String get rosterUnconfigured =>
      'NFT imports are unavailable until a reaper collection is configured. Your starter remains yours.';

  @override
  String rosterCount(int count) {
    return '$count reapers';
  }

  @override
  String get studioModelFailed => 'This specimen could not be drawn.';

  @override
  String get studioWebGLUnavailable =>
      'Your browser needs WebGL 2 to draw this specimen. Try another browser or open the studio on Android or iOS.';

  @override
  String get studioXp => 'XP';

  @override
  String get studioCapped => 'Max';

  @override
  String get studioPause => 'Pause rotation';

  @override
  String get studioRotate => 'Rotate specimen';

  @override
  String get refusalRosterFull =>
      'Your reaper roster is full. Transfer an NFT before importing another.';

  @override
  String get refusalRiftUnproven =>
      'NFT reapers can hunt solo. Rifts will open once their results can be verified.';

  @override
  String get soulTitle => 'The soul remembers';

  @override
  String get refusalHuntVersionUnsupported =>
      'The hunt rules have changed. Update Reapers before trying another hunt.';

  @override
  String get refusalMarketUnavailable =>
      'NFT hunting hasn’t opened here yet. You can practise with your reaper.';

  @override
  String get soulTab => 'Soul';

  @override
  String get soulBirth => 'Born with a gift';

  @override
  String soulTemperament(String name) {
    return '$name temperament';
  }

  @override
  String get soulIntro =>
      'Your gift is fixed at birth. The way you hunt teaches it to grow. Every reaper keeps its own memories.';

  @override
  String get soulGiftChoice => 'Try a gift';

  @override
  String get soulMarks => 'Hunts remembered';

  @override
  String soulRank(int rank, int max) {
    return 'Awakening $rank / $max';
  }

  @override
  String soulMilestone(int level, int marks, String path) {
    return 'Level $level · $marks $path marks';
  }

  @override
  String soulNext(int level, int marks, String path, int earned) {
    return 'Next awakening: level $level and $marks $path marks. You have $earned.';
  }

  @override
  String get soulComplete =>
      'Every awakening earned. Your birth gift stays yours.';

  @override
  String get soulLore => 'What the gift remembers';

  @override
  String get soulUnseen => 'Unseen';

  @override
  String get soulStillness => 'Stillness';

  @override
  String get soulRhythm => 'Rhythm';

  @override
  String get soulUnseenHelp => 'Absorb a husk after an ambush.';

  @override
  String get soulStillnessHelp => 'Absorb a husk after an unflinching hold.';

  @override
  String get soulRhythmHelp =>
      'Absorb a husk after five landed hits in cadence.';

  @override
  String get soulMemoryHelp =>
      'One mark per path per absorption. Escapes and abandoned hunts grant none. Old hunts keep their XP; memories begin with your next hunt.';

  @override
  String get soulPreviewHelp =>
      'These memories are an unsaved specimen. Real memories come from the keeper’s replay of your hunts.';

  @override
  String soulPreviewMark(String path) {
    return 'Preview hunt: $path';
  }

  @override
  String get soulPreviewReset => 'Clear specimen memories';

  @override
  String soulBirthLink(String gift, String temperament) {
    return '$gift · $temperament';
  }

  @override
  String soulRankEarned(String gift, int rank) {
    return 'Your gift awakened: $gift, awakening $rank.';
  }

  @override
  String soulDeedsEarned(String paths) {
    return 'The soul remembers this hunt: $paths.';
  }

  @override
  String get temperamentPatient => 'Patient';

  @override
  String get temperamentKeen => 'Keen';

  @override
  String get temperamentStalwart => 'Stalwart';

  @override
  String get temperamentHungry => 'Hungry';

  @override
  String get temperamentPatientHelp =>
      'Half a second longer to reap. Every swing takes two ticks longer.';

  @override
  String get temperamentKeenHelp =>
      'A wider reach by 0.6°. Four tenths of a second less to reap.';

  @override
  String get temperamentStalwartHelp =>
      'One more resolve. Strikes deal 5% less damage.';

  @override
  String get temperamentHungryHelp =>
      'Strikes deal 8% more damage. You must hold 8% steadier through a shriek.';

  @override
  String get giftFurrow => 'The Last Furrow';

  @override
  String get giftAftermath => 'Aftermath';

  @override
  String get giftGathering => 'The Gathering';

  @override
  String get giftPassage => 'Safe Passage';

  @override
  String get giftAnchor => 'The Anchor';

  @override
  String get giftReturnLight => 'Returning Light';

  @override
  String get giftForeknowledge => 'Foreknowledge';

  @override
  String get giftVerdict => 'The Verdict';

  @override
  String get giftEquilibrium => 'Equilibrium';

  @override
  String get giftFirstLight => 'First Light';

  @override
  String get giftReprisal => 'Reprisal';

  @override
  String get giftLastWord => 'The Last Word';

  @override
  String get giftHunger => 'Old Hunger';

  @override
  String get giftEcho => 'The Echo';

  @override
  String get giftUnbroken => 'Unbroken';

  @override
  String get giftHush => 'Hush';

  @override
  String get giftKeening => 'Keening';

  @override
  String get giftRefrain => 'The Refrain';

  @override
  String giftFurrowEffect(int share) {
    return 'Your ambush cleave catches other open seals for $share% of its blow.';
  }

  @override
  String giftAftermathEffect(String seconds) {
    return 'After an unflinching hold, your next cleave leaves the other seals open for at least ${seconds}s.';
  }

  @override
  String giftGatheringEffect(String seconds) {
    return 'Every third hit in cadence reopens its seal for ${seconds}s.';
  }

  @override
  String giftPassageEffect(String seconds) {
    return 'An ambush shields you from one charge. The husk is stunned for ${seconds}s.';
  }

  @override
  String giftAnchorEffect(int uses) {
    String _temp0 = intl.Intl.pluralLogic(
      uses,
      locale: localeName,
      other: 'up to $uses times',
      one: 'once',
    );
    return 'An unflinching hold restores one lost resolve, $_temp0 per hunt.';
  }

  @override
  String giftReturnLightEffect(int uses) {
    return 'Five hits in cadence prepare a light that forgives your next flinch, up to $uses times per hunt.';
  }

  @override
  String giftForeknowledgeEffect(String seconds) {
    return 'An ambush keeps the seals open for ${seconds}s longer.';
  }

  @override
  String giftVerdictEffect(int bonus) {
    return 'After an unflinching hold, your next in-order strike deals $bonus% more damage.';
  }

  @override
  String giftEquilibriumEffect(String seconds) {
    return 'Three in-order strikes reopen every seal for ${seconds}s.';
  }

  @override
  String giftFirstLightEffect(int bonus) {
    return 'Your first ambush strike deals $bonus% extra damage.';
  }

  @override
  String giftReprisalEffect(int bonus) {
    return 'A parry readies Chosen Blow, with a further $bonus% damage.';
  }

  @override
  String giftLastWordEffect(int bonus) {
    return 'At three hits in cadence, seal strikes against a husk below 35% ward deal $bonus% more damage.';
  }

  @override
  String giftHungerEffect(int beats) {
    return 'An ambush starts your cadence at $beats.';
  }

  @override
  String giftEchoEffect(int beats) {
    return 'An unflinching hold restores your cadence from the warning, plus $beats. Cadence still caps at ten.';
  }

  @override
  String giftUnbrokenEffect(int uses) {
    return 'Up to $uses misses preserve cadence each hunt. Its timer keeps running.';
  }

  @override
  String giftHushEffect(String seconds) {
    return 'An ambush delays the first shriek by ${seconds}s.';
  }

  @override
  String giftKeeningEffect(int bonus) {
    return 'After an unflinching hold, your next seal strike deals $bonus% more damage.';
  }

  @override
  String giftRefrainEffect(String seconds) {
    return 'Every third hit in cadence adds ${seconds}s to your reap window, up to six times per hunt.';
  }

  @override
  String get giftFurrowLore =>
      'Nothing in the field dies alone. The first cut follows the roots beneath it.';

  @override
  String get giftAftermathLore =>
      'The harvest remembers what survived. Those touched by the blade remain within its reach.';

  @override
  String get giftGatheringLore =>
      'One stalk, then another. The hand finds a rhythm that the field cannot close against.';

  @override
  String get giftPassageLore =>
      'A ferryman promises arrival. Even a furious passenger cannot overturn that promise.';

  @override
  String get giftAnchorLore =>
      'The crossing has no shore beneath it. The ferryman learns to make a shore of stillness.';

  @override
  String get giftReturnLightLore =>
      'Every stroke of the pole leaves a little light behind. Enough light can guide a trembling hand home.';

  @override
  String get giftForeknowledgeLore =>
      'A careful keeper opens the ledger before the dead arrive. The first reading leaves nothing concealed.';

  @override
  String get giftVerdictLore =>
      'Judgement begins when the room falls silent. Only the right seal can carry the sentence.';

  @override
  String get giftEquilibriumLore =>
      'Three true measures bring the scales level. For a moment, every debt lies open.';

  @override
  String get giftFirstLightLore =>
      'The chooser sees a death before anyone else does. The first light is also the first wound.';

  @override
  String get giftReprisalLore =>
      'A death that reaches for the chooser has chosen itself. The answer arrives on the returning blade.';

  @override
  String get giftLastWordLore =>
      'A spirit may argue until the end. The chooser saves the last word for what remains.';

  @override
  String get giftHungerLore =>
      'The empty collar remembers every meal it missed. Silence gives that hunger a head start.';

  @override
  String get giftEchoLore =>
      'A shriek cannot drown out an absence. Beneath the noise, the old rhythm waits.';

  @override
  String get giftUnbrokenLore =>
      'There is no head to lose. Even a wandering hand cannot break the chain at once.';

  @override
  String get giftHushLore =>
      'Before a death is announced, the mourner grants a little silence. Even the dead must wait their turn.';

  @override
  String get giftKeeningLore =>
      'The mourner listens until the cry has passed. The next toll carries what was heard.';

  @override
  String get giftRefrainLore =>
      'A bell cannot call a name only once. Each refrain gives the departing soul longer to answer.';
}
