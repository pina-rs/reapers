import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'generated/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[Locale('en')];

  /// No description provided for @appTitle.
  ///
  /// In en, this message translates to:
  /// **'Reapers'**
  String get appTitle;

  /// No description provided for @commonBack.
  ///
  /// In en, this message translates to:
  /// **'Back'**
  String get commonBack;

  /// No description provided for @commonLoading.
  ///
  /// In en, this message translates to:
  /// **'Parting the veil…'**
  String get commonLoading;

  /// No description provided for @commonExperience.
  ///
  /// In en, this message translates to:
  /// **'Experience'**
  String get commonExperience;

  /// No description provided for @commonAnima.
  ///
  /// In en, this message translates to:
  /// **'Anima'**
  String get commonAnima;

  /// No description provided for @commonLevel.
  ///
  /// In en, this message translates to:
  /// **'Level'**
  String get commonLevel;

  /// No description provided for @commonSol.
  ///
  /// In en, this message translates to:
  /// **'{amount} SOL'**
  String commonSol(String amount);

  /// No description provided for @commonRefusedTitle.
  ///
  /// In en, this message translates to:
  /// **'Refused'**
  String get commonRefusedTitle;

  /// No description provided for @commonDismiss.
  ///
  /// In en, this message translates to:
  /// **'Understood'**
  String get commonDismiss;

  /// No description provided for @commonFailedTitle.
  ///
  /// In en, this message translates to:
  /// **'Something came loose'**
  String get commonFailedTitle;

  /// No description provided for @commonFailedBody.
  ///
  /// In en, this message translates to:
  /// **'{reason}'**
  String commonFailedBody(String reason);

  /// No description provided for @commonRetry.
  ///
  /// In en, this message translates to:
  /// **'Try again'**
  String get commonRetry;

  /// No description provided for @commonUnreachable.
  ///
  /// In en, this message translates to:
  /// **'The keeper of the books could not be reached. Nothing was changed.'**
  String get commonUnreachable;

  /// No description provided for @thresholdOverline.
  ///
  /// In en, this message translates to:
  /// **'Beyond the veil'**
  String get thresholdOverline;

  /// No description provided for @thresholdTagline.
  ///
  /// In en, this message translates to:
  /// **'The dead market did not stay dead. What it left behind still drifts between worlds, and someone has to bring it in.'**
  String get thresholdTagline;

  /// No description provided for @thresholdOath.
  ///
  /// In en, this message translates to:
  /// **'Take the oath'**
  String get thresholdOath;

  /// No description provided for @thresholdOathDetail.
  ///
  /// In en, this message translates to:
  /// **'Be bound to a reaper'**
  String get thresholdOathDetail;

  /// No description provided for @thresholdShade.
  ///
  /// In en, this message translates to:
  /// **'Walk as a shade'**
  String get thresholdShade;

  /// No description provided for @thresholdShadeDetail.
  ///
  /// In en, this message translates to:
  /// **'A practice hunt'**
  String get thresholdShadeDetail;

  /// No description provided for @oathOverline.
  ///
  /// In en, this message translates to:
  /// **'The oath'**
  String get oathOverline;

  /// No description provided for @oathTitle.
  ///
  /// In en, this message translates to:
  /// **'You do not choose your reaper.'**
  String get oathTitle;

  /// No description provided for @oathBody.
  ///
  /// In en, this message translates to:
  /// **'The shuffle draws one for you, once. Its class, its face, its light and its name are bound to you from then on. What you make of it is yours.'**
  String get oathBody;

  /// No description provided for @oathSpeak.
  ///
  /// In en, this message translates to:
  /// **'I am bound'**
  String get oathSpeak;

  /// No description provided for @oathTurning.
  ///
  /// In en, this message translates to:
  /// **'The shuffle turns…'**
  String get oathTurning;

  /// No description provided for @oathDrawnOverline.
  ///
  /// In en, this message translates to:
  /// **'The shuffle has drawn'**
  String get oathDrawnOverline;

  /// No description provided for @oathEnter.
  ///
  /// In en, this message translates to:
  /// **'Enter the sanctum'**
  String get oathEnter;

  /// No description provided for @oathOmen.
  ///
  /// In en, this message translates to:
  /// **'Omen'**
  String get oathOmen;

  /// No description provided for @sanctumOverline.
  ///
  /// In en, this message translates to:
  /// **'Sanctum'**
  String get sanctumOverline;

  /// No description provided for @sanctumLoose.
  ///
  /// In en, this message translates to:
  /// **'Loose in the veil'**
  String get sanctumLoose;

  /// No description provided for @sanctumNothingLoose.
  ///
  /// In en, this message translates to:
  /// **'Nothing is loose. Release an NFT from your vault and it will surface here as a husk.'**
  String get sanctumNothingLoose;

  /// No description provided for @sanctumSurfacing.
  ///
  /// In en, this message translates to:
  /// **'Surfacing in {seconds}s'**
  String sanctumSurfacing(int seconds);

  /// No description provided for @sanctumHunt.
  ///
  /// In en, this message translates to:
  /// **'Hunt'**
  String get sanctumHunt;

  /// No description provided for @sanctumSightedBanner.
  ///
  /// In en, this message translates to:
  /// **'Husk sighted: {name}'**
  String sanctumSightedBanner(String name);

  /// No description provided for @sanctumVault.
  ///
  /// In en, this message translates to:
  /// **'Vault'**
  String get sanctumVault;

  /// No description provided for @sanctumVaultDetail.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =0{Empty} =1{1 held} other{{count} held}}'**
  String sanctumVaultDetail(int count);

  /// No description provided for @sanctumArmory.
  ///
  /// In en, this message translates to:
  /// **'Armory'**
  String get sanctumArmory;

  /// No description provided for @sanctumCodex.
  ///
  /// In en, this message translates to:
  /// **'Codex'**
  String get sanctumCodex;

  /// No description provided for @sanctumCodexDetail.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =0{Nothing absorbed} =1{1 absorbed} other{{count} absorbed}}'**
  String sanctumCodexDetail(int count);

  /// No description provided for @sanctumRifts.
  ///
  /// In en, this message translates to:
  /// **'Rifts'**
  String get sanctumRifts;

  /// No description provided for @sanctumRiftsDetail.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =0{Hunt together} =1{1 sealed} other{{count} sealed}}'**
  String sanctumRiftsDetail(int count);

  /// No description provided for @sanctumColophon.
  ///
  /// In en, this message translates to:
  /// **'Colophon'**
  String get sanctumColophon;

  /// No description provided for @sanctumColophonDetail.
  ///
  /// In en, this message translates to:
  /// **'Music and credits'**
  String get sanctumColophonDetail;

  /// No description provided for @colophonOverline.
  ///
  /// In en, this message translates to:
  /// **'Colophon'**
  String get colophonOverline;

  /// No description provided for @colophonTitle.
  ///
  /// In en, this message translates to:
  /// **'What this is made of'**
  String get colophonTitle;

  /// No description provided for @colophonMusic.
  ///
  /// In en, this message translates to:
  /// **'Music'**
  String get colophonMusic;

  /// No description provided for @colophonMusicOn.
  ///
  /// In en, this message translates to:
  /// **'On'**
  String get colophonMusicOn;

  /// No description provided for @colophonMusicOff.
  ///
  /// In en, this message translates to:
  /// **'Off'**
  String get colophonMusicOff;

  /// No description provided for @colophonMusicHeading.
  ///
  /// In en, this message translates to:
  /// **'Music'**
  String get colophonMusicHeading;

  /// No description provided for @colophonSanctum.
  ///
  /// In en, this message translates to:
  /// **'In the sanctum'**
  String get colophonSanctum;

  /// No description provided for @colophonHunt.
  ///
  /// In en, this message translates to:
  /// **'On the hunt'**
  String get colophonHunt;

  /// No description provided for @colophonRift.
  ///
  /// In en, this message translates to:
  /// **'In a rift'**
  String get colophonRift;

  /// No description provided for @colophonBachPiece.
  ///
  /// In en, this message translates to:
  /// **'J.S. Bach, Prelude No. 8 in E-flat minor, BWV 853'**
  String get colophonBachPiece;

  /// No description provided for @colophonBachCredit.
  ///
  /// In en, this message translates to:
  /// **'Kimiko Ishizaka, piano. From the Open Well-Tempered Clavier, released into the public domain. welltemperedclavier.org'**
  String get colophonBachCredit;

  /// No description provided for @colophonAnabasisPiece.
  ///
  /// In en, this message translates to:
  /// **'Anabasis I'**
  String get colophonAnabasisPiece;

  /// No description provided for @colophonAnabasisCredit.
  ///
  /// In en, this message translates to:
  /// **'\'Anabasis I\' by Scott Buckley - released under CC-BY 4.0. www.scottbuckley.com.au'**
  String get colophonAnabasisCredit;

  /// No description provided for @colophonKatabasisPiece.
  ///
  /// In en, this message translates to:
  /// **'Katabasis I'**
  String get colophonKatabasisPiece;

  /// No description provided for @colophonKatabasisCredit.
  ///
  /// In en, this message translates to:
  /// **'\'Katabasis I\' by Scott Buckley - released under CC-BY 4.0. www.scottbuckley.com.au'**
  String get colophonKatabasisCredit;

  /// No description provided for @colophonSightHeading.
  ///
  /// In en, this message translates to:
  /// **'Reading the room'**
  String get colophonSightHeading;

  /// No description provided for @colophonSight.
  ///
  /// In en, this message translates to:
  /// **'How far away things are'**
  String get colophonSight;

  /// No description provided for @colophonDepthPiece.
  ///
  /// In en, this message translates to:
  /// **'Depth Anything V2, small, metric, indoor'**
  String get colophonDepthPiece;

  /// No description provided for @colophonDepthCredit.
  ///
  /// In en, this message translates to:
  /// **'Lihe Yang and others. Released under the Apache License 2.0. It runs on this phone; no picture leaves it.'**
  String get colophonDepthCredit;

  /// No description provided for @colophonTypeHeading.
  ///
  /// In en, this message translates to:
  /// **'Type'**
  String get colophonTypeHeading;

  /// No description provided for @colophonType.
  ///
  /// In en, this message translates to:
  /// **'Set in'**
  String get colophonType;

  /// No description provided for @colophonTypePiece.
  ///
  /// In en, this message translates to:
  /// **'Fraunces and DM Sans'**
  String get colophonTypePiece;

  /// No description provided for @colophonTypeCredit.
  ///
  /// In en, this message translates to:
  /// **'Both released under the SIL Open Font License 1.1.'**
  String get colophonTypeCredit;

  /// No description provided for @sanctumPractice.
  ///
  /// In en, this message translates to:
  /// **'Practice hunt'**
  String get sanctumPractice;

  /// No description provided for @sanctumPracticeDetail.
  ///
  /// In en, this message translates to:
  /// **'A stray'**
  String get sanctumPracticeDetail;

  /// No description provided for @sanctumStatStrike.
  ///
  /// In en, this message translates to:
  /// **'Strike'**
  String get sanctumStatStrike;

  /// No description provided for @sanctumStatReach.
  ///
  /// In en, this message translates to:
  /// **'Reach'**
  String get sanctumStatReach;

  /// No description provided for @sanctumStatCapture.
  ///
  /// In en, this message translates to:
  /// **'Capture'**
  String get sanctumStatCapture;

  /// No description provided for @sanctumStatResilience.
  ///
  /// In en, this message translates to:
  /// **'Resilience'**
  String get sanctumStatResilience;

  /// No description provided for @vaultOverline.
  ///
  /// In en, this message translates to:
  /// **'Vault'**
  String get vaultOverline;

  /// No description provided for @vaultTitle.
  ///
  /// In en, this message translates to:
  /// **'What you hold, and what is for sale.'**
  String get vaultTitle;

  /// No description provided for @vaultStandIn.
  ///
  /// In en, this message translates to:
  /// **'A simulated market. No real SOL changes hands and no real NFT is touched. The collections are real ones from Solana\'s first wave; the creatures are our own models of them, and nothing here is licensed yet.'**
  String get vaultStandIn;

  /// No description provided for @vaultBuy.
  ///
  /// In en, this message translates to:
  /// **'Buy'**
  String get vaultBuy;

  /// No description provided for @vaultRelease.
  ///
  /// In en, this message translates to:
  /// **'Release into the veil'**
  String get vaultRelease;

  /// No description provided for @vaultHeld.
  ///
  /// In en, this message translates to:
  /// **'Held'**
  String get vaultHeld;

  /// No description provided for @vaultLoose.
  ///
  /// In en, this message translates to:
  /// **'Loose'**
  String get vaultLoose;

  /// No description provided for @vaultAbsorbed.
  ///
  /// In en, this message translates to:
  /// **'Absorbed'**
  String get vaultAbsorbed;

  /// No description provided for @vaultMint.
  ///
  /// In en, this message translates to:
  /// **'Mint'**
  String get vaultMint;

  /// No description provided for @vaultWard.
  ///
  /// In en, this message translates to:
  /// **'Ward'**
  String get vaultWard;

  /// No description provided for @vaultWorth.
  ///
  /// In en, this message translates to:
  /// **'Worth'**
  String get vaultWorth;

  /// No description provided for @vaultWorthValue.
  ///
  /// In en, this message translates to:
  /// **'{xp} xp · {anima} anima'**
  String vaultWorthValue(int xp, int anima);

  /// No description provided for @vaultReleaseNote.
  ///
  /// In en, this message translates to:
  /// **'Releasing cannot be undone. The NFT becomes a husk, and you will have to hunt it down.'**
  String get vaultReleaseNote;

  /// No description provided for @armoryOverline.
  ///
  /// In en, this message translates to:
  /// **'Armory'**
  String get armoryOverline;

  /// No description provided for @armoryWeapons.
  ///
  /// In en, this message translates to:
  /// **'Weapons'**
  String get armoryWeapons;

  /// No description provided for @armoryOutfits.
  ///
  /// In en, this message translates to:
  /// **'Outfits'**
  String get armoryOutfits;

  /// No description provided for @armoryWield.
  ///
  /// In en, this message translates to:
  /// **'Wield'**
  String get armoryWield;

  /// No description provided for @armoryWielded.
  ///
  /// In en, this message translates to:
  /// **'In hand'**
  String get armoryWielded;

  /// No description provided for @armoryWear.
  ///
  /// In en, this message translates to:
  /// **'Wear'**
  String get armoryWear;

  /// No description provided for @armoryWorn.
  ///
  /// In en, this message translates to:
  /// **'Worn'**
  String get armoryWorn;

  /// No description provided for @armoryNeedsLevel.
  ///
  /// In en, this message translates to:
  /// **'Level {level}'**
  String armoryNeedsLevel(String level);

  /// No description provided for @armoryFree.
  ///
  /// In en, this message translates to:
  /// **'Yours'**
  String get armoryFree;

  /// No description provided for @armoryAnima.
  ///
  /// In en, this message translates to:
  /// **'{amount} anima'**
  String armoryAnima(int amount);

  /// No description provided for @armoryStrike.
  ///
  /// In en, this message translates to:
  /// **'Strike ×{value}'**
  String armoryStrike(String value);

  /// No description provided for @armoryTempo.
  ///
  /// In en, this message translates to:
  /// **'{value} strikes a second'**
  String armoryTempo(String value);

  /// No description provided for @armoryOutfitNote.
  ///
  /// In en, this message translates to:
  /// **'An outfit changes how your reaper looks, never how it hunts.'**
  String get armoryOutfitNote;

  /// No description provided for @codexOverline.
  ///
  /// In en, this message translates to:
  /// **'Codex'**
  String get codexOverline;

  /// No description provided for @codexTitle.
  ///
  /// In en, this message translates to:
  /// **'Everything you have absorbed.'**
  String get codexTitle;

  /// No description provided for @codexEmpty.
  ///
  /// In en, this message translates to:
  /// **'Nothing yet. Every husk you reap is written here.'**
  String get codexEmpty;

  /// No description provided for @codexSet.
  ///
  /// In en, this message translates to:
  /// **'{count} of {total}'**
  String codexSet(int count, int total);

  /// No description provided for @codexSetComplete.
  ///
  /// In en, this message translates to:
  /// **'Set complete · strike +3%'**
  String get codexSetComplete;

  /// No description provided for @codexSetProgress.
  ///
  /// In en, this message translates to:
  /// **'{remaining, plural, =1{1 more completes the set} other{{remaining} more complete the set}}'**
  String codexSetProgress(int remaining);

  /// No description provided for @codexLost.
  ///
  /// In en, this message translates to:
  /// **'Husks that got away'**
  String get codexLost;

  /// No description provided for @riftsOverline.
  ///
  /// In en, this message translates to:
  /// **'Rifts'**
  String get riftsOverline;

  /// No description provided for @riftsTitle.
  ///
  /// In en, this message translates to:
  /// **'Some husks are too much for one reaper.'**
  String get riftsTitle;

  /// No description provided for @riftsStandIn.
  ///
  /// In en, this message translates to:
  /// **'A rift takes a circle of four. Stand at its site and it is there. Anywhere else, it takes its place 25 metres in front of the first reaper to arrive, and everyone who joins sees it in that same spot. Shades take the places nobody else does.'**
  String get riftsStandIn;

  /// No description provided for @riftsSite.
  ///
  /// In en, this message translates to:
  /// **'{city} · {collection}'**
  String riftsSite(String city, String collection);

  /// No description provided for @riftsOpen.
  ///
  /// In en, this message translates to:
  /// **'Enter'**
  String get riftsOpen;

  /// No description provided for @riftsAttune.
  ///
  /// In en, this message translates to:
  /// **'Absorb one of the {collection} and it will open for you.'**
  String riftsAttune(String collection);

  /// No description provided for @riftLabel.
  ///
  /// In en, this message translates to:
  /// **'Rift'**
  String get riftLabel;

  /// No description provided for @riftCircle.
  ///
  /// In en, this message translates to:
  /// **'The circle'**
  String get riftCircle;

  /// No description provided for @riftSeatEmpty.
  ///
  /// In en, this message translates to:
  /// **'An empty place'**
  String get riftSeatEmpty;

  /// No description provided for @riftSeatEmptyDetail.
  ///
  /// In en, this message translates to:
  /// **'A shade will stand in'**
  String get riftSeatEmptyDetail;

  /// No description provided for @riftSeatShade.
  ///
  /// In en, this message translates to:
  /// **'Shade'**
  String get riftSeatShade;

  /// No description provided for @riftSeatYou.
  ///
  /// In en, this message translates to:
  /// **'You'**
  String get riftSeatYou;

  /// No description provided for @riftSeatReady.
  ///
  /// In en, this message translates to:
  /// **'Ready'**
  String get riftSeatReady;

  /// No description provided for @riftSeatWaiting.
  ///
  /// In en, this message translates to:
  /// **'Arriving'**
  String get riftSeatWaiting;

  /// No description provided for @riftFinding.
  ///
  /// In en, this message translates to:
  /// **'Finding where you stand, so the rift can take its place.'**
  String get riftFinding;

  /// No description provided for @riftUnplaced.
  ///
  /// In en, this message translates to:
  /// **'This phone cannot say where it is, so the rift stands dead ahead of you.'**
  String get riftUnplaced;

  /// No description provided for @riftStands.
  ///
  /// In en, this message translates to:
  /// **'The rift stands {metres} m from you. Everyone here sees it in the same place: turn until you find it.'**
  String riftStands(int metres);

  /// No description provided for @riftStandsFar.
  ///
  /// In en, this message translates to:
  /// **'The rift stands too far from here to point at, so you see it dead ahead.'**
  String get riftStandsFar;

  /// No description provided for @riftMeasured.
  ///
  /// In en, this message translates to:
  /// **'(You: {latitude}, {longitude}, within {accuracy} m.)'**
  String riftMeasured(int accuracy, String latitude, String longitude);

  /// No description provided for @riftStandReady.
  ///
  /// In en, this message translates to:
  /// **'Stand ready'**
  String get riftStandReady;

  /// No description provided for @riftWaiting.
  ///
  /// In en, this message translates to:
  /// **'Waiting for the circle'**
  String get riftWaiting;

  /// No description provided for @riftBegins.
  ///
  /// In en, this message translates to:
  /// **'Begins in {seconds}'**
  String riftBegins(int seconds);

  /// No description provided for @riftEntering.
  ///
  /// In en, this message translates to:
  /// **'Finding the rift…'**
  String get riftEntering;

  /// No description provided for @riftLeave.
  ///
  /// In en, this message translates to:
  /// **'Leave the circle'**
  String get riftLeave;

  /// No description provided for @riftCastOut.
  ///
  /// In en, this message translates to:
  /// **'You were cast out. The circle fights on.'**
  String get riftCastOut;

  /// No description provided for @riftCutMade.
  ///
  /// In en, this message translates to:
  /// **'Your cut is made. Hold for the circle.'**
  String get riftCutMade;

  /// No description provided for @riftCutMissed.
  ///
  /// In en, this message translates to:
  /// **'The cut got away from you. Hold for the circle.'**
  String get riftCutMissed;

  /// No description provided for @riftCuts.
  ///
  /// In en, this message translates to:
  /// **'Cuts {made} of {needed}'**
  String riftCuts(int made, int needed);

  /// No description provided for @riftSealedLabel.
  ///
  /// In en, this message translates to:
  /// **'Sealed'**
  String get riftSealedLabel;

  /// No description provided for @riftSealedTitle.
  ///
  /// In en, this message translates to:
  /// **'The rift is shut'**
  String get riftSealedTitle;

  /// No description provided for @riftSealedBody.
  ///
  /// In en, this message translates to:
  /// **'{site} is quiet again.'**
  String riftSealedBody(String site);

  /// No description provided for @riftFailedLabel.
  ///
  /// In en, this message translates to:
  /// **'Unsealed'**
  String get riftFailedLabel;

  /// No description provided for @riftFailedTitle.
  ///
  /// In en, this message translates to:
  /// **'The rift stays open'**
  String get riftFailedTitle;

  /// No description provided for @riftFailedBody.
  ///
  /// In en, this message translates to:
  /// **'The circle could not close it this time. Nothing was lost but the chance.'**
  String get riftFailedBody;

  /// No description provided for @riftShare.
  ///
  /// In en, this message translates to:
  /// **'{percent}%'**
  String riftShare(int percent);

  /// No description provided for @riftMemberCut.
  ///
  /// In en, this message translates to:
  /// **'Cut'**
  String get riftMemberCut;

  /// No description provided for @riftMemberNoCut.
  ///
  /// In en, this message translates to:
  /// **'No cut'**
  String get riftMemberNoCut;

  /// No description provided for @riftMemberCastOut.
  ///
  /// In en, this message translates to:
  /// **'Cast out'**
  String get riftMemberCastOut;

  /// No description provided for @riftLostTitle.
  ///
  /// In en, this message translates to:
  /// **'The rift is gone'**
  String get riftLostTitle;

  /// No description provided for @riftLostBody.
  ///
  /// In en, this message translates to:
  /// **'The way to it closed: {reason}'**
  String riftLostBody(String reason);

  /// No description provided for @classHarvester.
  ///
  /// In en, this message translates to:
  /// **'Harvester'**
  String get classHarvester;

  /// No description provided for @classFerryman.
  ///
  /// In en, this message translates to:
  /// **'Ferryman'**
  String get classFerryman;

  /// No description provided for @classWeigher.
  ///
  /// In en, this message translates to:
  /// **'Weigher'**
  String get classWeigher;

  /// No description provided for @classChooser.
  ///
  /// In en, this message translates to:
  /// **'Chooser'**
  String get classChooser;

  /// No description provided for @classHollow.
  ///
  /// In en, this message translates to:
  /// **'Hollow'**
  String get classHollow;

  /// No description provided for @classMourner.
  ///
  /// In en, this message translates to:
  /// **'Mourner'**
  String get classMourner;

  /// No description provided for @classHarvesterLine.
  ///
  /// In en, this message translates to:
  /// **'The field hand of death, who cuts down many at once.'**
  String get classHarvesterLine;

  /// No description provided for @classFerrymanLine.
  ///
  /// In en, this message translates to:
  /// **'The one who carries the dead across and cannot be moved.'**
  String get classFerrymanLine;

  /// No description provided for @classWeigherLine.
  ///
  /// In en, this message translates to:
  /// **'The judge who measures a spirit before taking it.'**
  String get classWeigherLine;

  /// No description provided for @classChooserLine.
  ///
  /// In en, this message translates to:
  /// **'The one who picks who falls, then strikes once.'**
  String get classChooserLine;

  /// No description provided for @classHollowLine.
  ///
  /// In en, this message translates to:
  /// **'The rider with no head, who has already lost what you fear to lose.'**
  String get classHollowLine;

  /// No description provided for @classMournerLine.
  ///
  /// In en, this message translates to:
  /// **'The voice that announces a death before it arrives.'**
  String get classMournerLine;

  /// No description provided for @perkCleave.
  ///
  /// In en, this message translates to:
  /// **'Cleave'**
  String get perkCleave;

  /// No description provided for @perkSteadfast.
  ///
  /// In en, this message translates to:
  /// **'Steadfast'**
  String get perkSteadfast;

  /// No description provided for @perkMeasure.
  ///
  /// In en, this message translates to:
  /// **'Measure'**
  String get perkMeasure;

  /// No description provided for @perkChosenBlow.
  ///
  /// In en, this message translates to:
  /// **'Chosen blow'**
  String get perkChosenBlow;

  /// No description provided for @perkChain.
  ///
  /// In en, this message translates to:
  /// **'Chain'**
  String get perkChain;

  /// No description provided for @perkToll.
  ///
  /// In en, this message translates to:
  /// **'Toll'**
  String get perkToll;

  /// No description provided for @perkCleaveLine.
  ///
  /// In en, this message translates to:
  /// **'A strike also lands on every open seal near it.'**
  String get perkCleaveLine;

  /// No description provided for @perkSteadfastLine.
  ///
  /// In en, this message translates to:
  /// **'The first flinch of a hunt costs nothing.'**
  String get perkSteadfastLine;

  /// No description provided for @perkMeasureLine.
  ///
  /// In en, this message translates to:
  /// **'Seals are numbered. Struck in order, they break faster.'**
  String get perkMeasureLine;

  /// No description provided for @perkChosenBlowLine.
  ///
  /// In en, this message translates to:
  /// **'Hold firm through a shriek and the next strike lands three times as hard.'**
  String get perkChosenBlowLine;

  /// No description provided for @perkChainLine.
  ///
  /// In en, this message translates to:
  /// **'Cadence builds twice as far.'**
  String get perkChainLine;

  /// No description provided for @perkTollLine.
  ///
  /// In en, this message translates to:
  /// **'Each shriek you hold through lengthens the reap.'**
  String get perkTollLine;

  /// No description provided for @weaponScythe.
  ///
  /// In en, this message translates to:
  /// **'Scythe'**
  String get weaponScythe;

  /// No description provided for @weaponLanternPole.
  ///
  /// In en, this message translates to:
  /// **'Lantern pole'**
  String get weaponLanternPole;

  /// No description provided for @weaponBalanceFlail.
  ///
  /// In en, this message translates to:
  /// **'Balance flail'**
  String get weaponBalanceFlail;

  /// No description provided for @weaponSpear.
  ///
  /// In en, this message translates to:
  /// **'Spear'**
  String get weaponSpear;

  /// No description provided for @weaponChainWhip.
  ///
  /// In en, this message translates to:
  /// **'Chain whip'**
  String get weaponChainWhip;

  /// No description provided for @weaponBell.
  ///
  /// In en, this message translates to:
  /// **'Bell'**
  String get weaponBell;

  /// No description provided for @temperShrouded.
  ///
  /// In en, this message translates to:
  /// **'Shrouded'**
  String get temperShrouded;

  /// No description provided for @temperSworn.
  ///
  /// In en, this message translates to:
  /// **'Sworn'**
  String get temperSworn;

  /// No description provided for @temperScorched.
  ///
  /// In en, this message translates to:
  /// **'Scorched'**
  String get temperScorched;

  /// No description provided for @temperHollowed.
  ///
  /// In en, this message translates to:
  /// **'Hollowed'**
  String get temperHollowed;

  /// No description provided for @outfitShroudedLine.
  ///
  /// In en, this message translates to:
  /// **'Undyed cloth, rope and wood.'**
  String get outfitShroudedLine;

  /// No description provided for @outfitSwornLine.
  ///
  /// In en, this message translates to:
  /// **'Dyed dark, shod in iron, carved with marks.'**
  String get outfitSwornLine;

  /// No description provided for @outfitScorchedLine.
  ///
  /// In en, this message translates to:
  /// **'Charred cloth and pitted metal, still smouldering.'**
  String get outfitScorchedLine;

  /// No description provided for @outfitHollowedLine.
  ///
  /// In en, this message translates to:
  /// **'Parts of you are simply gone. The light is all that is left.'**
  String get outfitHollowedLine;

  /// No description provided for @rarityCommon.
  ///
  /// In en, this message translates to:
  /// **'Common'**
  String get rarityCommon;

  /// No description provided for @rarityUncommon.
  ///
  /// In en, this message translates to:
  /// **'Uncommon'**
  String get rarityUncommon;

  /// No description provided for @rarityRare.
  ///
  /// In en, this message translates to:
  /// **'Rare'**
  String get rarityRare;

  /// No description provided for @rarityEpic.
  ///
  /// In en, this message translates to:
  /// **'Epic'**
  String get rarityEpic;

  /// No description provided for @omenNone.
  ///
  /// In en, this message translates to:
  /// **'None'**
  String get omenNone;

  /// No description provided for @omenAshen.
  ///
  /// In en, this message translates to:
  /// **'Ashen'**
  String get omenAshen;

  /// No description provided for @omenGilded.
  ///
  /// In en, this message translates to:
  /// **'Gilded'**
  String get omenGilded;

  /// No description provided for @omenEclipsed.
  ///
  /// In en, this message translates to:
  /// **'Eclipsed'**
  String get omenEclipsed;

  /// No description provided for @refusalOathAlreadyTaken.
  ///
  /// In en, this message translates to:
  /// **'You are already bound. The oath is taken once.'**
  String get refusalOathAlreadyTaken;

  /// No description provided for @refusalOathRequired.
  ///
  /// In en, this message translates to:
  /// **'Only the oathbound may do that.'**
  String get refusalOathRequired;

  /// No description provided for @refusalNotListed.
  ///
  /// In en, this message translates to:
  /// **'The market does not list that.'**
  String get refusalNotListed;

  /// No description provided for @refusalAlreadyHeld.
  ///
  /// In en, this message translates to:
  /// **'You already hold that, or have absorbed it.'**
  String get refusalAlreadyHeld;

  /// No description provided for @refusalNotInVault.
  ///
  /// In en, this message translates to:
  /// **'That is not in your vault.'**
  String get refusalNotInVault;

  /// No description provided for @refusalInsufficientFunds.
  ///
  /// In en, this message translates to:
  /// **'Your wallet does not hold enough SOL.'**
  String get refusalInsufficientFunds;

  /// No description provided for @refusalInsufficientAnima.
  ///
  /// In en, this message translates to:
  /// **'You have not gathered enough anima.'**
  String get refusalInsufficientAnima;

  /// No description provided for @refusalLevelTooLow.
  ///
  /// In en, this message translates to:
  /// **'Your reaper is not strong enough yet.'**
  String get refusalLevelTooLow;

  /// No description provided for @refusalNotOwned.
  ///
  /// In en, this message translates to:
  /// **'You have not bought that.'**
  String get refusalNotOwned;

  /// No description provided for @refusalAlreadyOwned.
  ///
  /// In en, this message translates to:
  /// **'You already own that.'**
  String get refusalAlreadyOwned;

  /// No description provided for @refusalVeilCrowded.
  ///
  /// In en, this message translates to:
  /// **'Three husks are already loose. Bring one in first.'**
  String get refusalVeilCrowded;

  /// No description provided for @refusalUnknownSighting.
  ///
  /// In en, this message translates to:
  /// **'That husk is no longer loose.'**
  String get refusalUnknownSighting;

  /// No description provided for @refusalNotSurfaced.
  ///
  /// In en, this message translates to:
  /// **'That husk has not surfaced yet.'**
  String get refusalNotSurfaced;

  /// No description provided for @refusalHuntUnproven.
  ///
  /// In en, this message translates to:
  /// **'The record of that hunt does not hold up.'**
  String get refusalHuntUnproven;

  /// No description provided for @refusalNotAttuned.
  ///
  /// In en, this message translates to:
  /// **'This rift only answers a reaper who carries one of its dead. Absorb an NFT from its collection first.'**
  String get refusalNotAttuned;

  /// No description provided for @scanLabel.
  ///
  /// In en, this message translates to:
  /// **'Reading the room'**
  String get scanLabel;

  /// No description provided for @scanTitle.
  ///
  /// In en, this message translates to:
  /// **'Something is nearby.'**
  String get scanTitle;

  /// No description provided for @scanFloor.
  ///
  /// In en, this message translates to:
  /// **'It keeps to the ground. Tip the phone down and sweep it slowly across the floor in front of you, so the veil knows where it can stand.'**
  String get scanFloor;

  /// No description provided for @scanFloorDrag.
  ///
  /// In en, this message translates to:
  /// **'It keeps to the ground. Drag down to look at the floor, then drag slowly across it, so the veil knows where it can stand.'**
  String get scanFloorDrag;

  /// No description provided for @scanAir.
  ///
  /// In en, this message translates to:
  /// **'It keeps to the air. Look around slowly at about head height, so the veil knows where it can go.'**
  String get scanAir;

  /// No description provided for @scanAirDrag.
  ///
  /// In en, this message translates to:
  /// **'It keeps to the air. Drag slowly from side to side at about head height, so the veil knows where it can go.'**
  String get scanAirDrag;

  /// No description provided for @scanReading.
  ///
  /// In en, this message translates to:
  /// **'Reading. Keep going.'**
  String get scanReading;

  /// No description provided for @scanSlower.
  ///
  /// In en, this message translates to:
  /// **'Slower. Nothing can be made out.'**
  String get scanSlower;

  /// No description provided for @scanLower.
  ///
  /// In en, this message translates to:
  /// **'Too high. Tip the phone down.'**
  String get scanLower;

  /// No description provided for @scanHigher.
  ///
  /// In en, this message translates to:
  /// **'Too low. Tip the phone up.'**
  String get scanHigher;

  /// No description provided for @scanFindRoom.
  ///
  /// In en, this message translates to:
  /// **'There is no room for it here. Turn toward somewhere more open.'**
  String get scanFindRoom;

  /// No description provided for @scanFloorFound.
  ///
  /// In en, this message translates to:
  /// **'Floor {metres} m below the phone'**
  String scanFloorFound(String metres);

  /// No description provided for @scanSkip.
  ///
  /// In en, this message translates to:
  /// **'Hunt anyway'**
  String get scanSkip;

  /// No description provided for @scanProgress.
  ///
  /// In en, this message translates to:
  /// **'Read'**
  String get scanProgress;

  /// No description provided for @scanFound.
  ///
  /// In en, this message translates to:
  /// **'It is here'**
  String get scanFound;

  /// No description provided for @feedingMeagre.
  ///
  /// In en, this message translates to:
  /// **'Meagre ×0.85'**
  String get feedingMeagre;

  /// No description provided for @feedingFair.
  ///
  /// In en, this message translates to:
  /// **'Fair ×1'**
  String get feedingFair;

  /// No description provided for @feedingRich.
  ///
  /// In en, this message translates to:
  /// **'Rich ×1.25'**
  String get feedingRich;

  /// No description provided for @feedingFeast.
  ///
  /// In en, this message translates to:
  /// **'A feast ×2'**
  String get feedingFeast;

  /// No description provided for @huntFeeding.
  ///
  /// In en, this message translates to:
  /// **'Feeding'**
  String get huntFeeding;

  /// No description provided for @huntReceipt.
  ///
  /// In en, this message translates to:
  /// **'Receipt'**
  String get huntReceipt;

  /// No description provided for @huntReceiptSigned.
  ///
  /// In en, this message translates to:
  /// **'Signed by the keeper'**
  String get huntReceiptSigned;

  /// No description provided for @huntReceiptUnsigned.
  ///
  /// In en, this message translates to:
  /// **'Unsigned, played offline'**
  String get huntReceiptUnsigned;

  /// No description provided for @huntSightedLabel.
  ///
  /// In en, this message translates to:
  /// **'Husk sighted'**
  String get huntSightedLabel;

  /// No description provided for @huntPracticeLabel.
  ///
  /// In en, this message translates to:
  /// **'A stray, for practice'**
  String get huntPracticeLabel;

  /// No description provided for @huntSafety.
  ///
  /// In en, this message translates to:
  /// **'Stand still to hunt, and stay aware of what is around you. You only need to turn on the spot.'**
  String get huntSafety;

  /// No description provided for @huntBegin.
  ///
  /// In en, this message translates to:
  /// **'Raise the blade'**
  String get huntBegin;

  /// No description provided for @huntHowStalk.
  ///
  /// In en, this message translates to:
  /// **'Find it and hold the ring on it, still, to ambush it.'**
  String get huntHowStalk;

  /// No description provided for @huntHowStrike.
  ///
  /// In en, this message translates to:
  /// **'Tap the lit seals on its body to wear down its ward.'**
  String get huntHowStrike;

  /// No description provided for @huntHowHold.
  ///
  /// In en, this message translates to:
  /// **'When it draws breath, freeze. Do not move and do not tap.'**
  String get huntHowHold;

  /// No description provided for @huntHowCharge.
  ///
  /// In en, this message translates to:
  /// **'When it runs at you, swing the phone hard across it.'**
  String get huntHowCharge;

  /// No description provided for @huntHowReap.
  ///
  /// In en, this message translates to:
  /// **'When its ward breaks, sweep the phone slowly along the arc. That is the reap.'**
  String get huntHowReap;

  /// No description provided for @huntDifficulty.
  ///
  /// In en, this message translates to:
  /// **'Testing difficulty'**
  String get huntDifficulty;

  /// No description provided for @difficultyEasy.
  ///
  /// In en, this message translates to:
  /// **'Easy'**
  String get difficultyEasy;

  /// No description provided for @difficultyNormal.
  ///
  /// In en, this message translates to:
  /// **'Normal'**
  String get difficultyNormal;

  /// No description provided for @difficultyHard.
  ///
  /// In en, this message translates to:
  /// **'Hard'**
  String get difficultyHard;

  /// No description provided for @huntWard.
  ///
  /// In en, this message translates to:
  /// **'Ward'**
  String get huntWard;

  /// No description provided for @huntResolve.
  ///
  /// In en, this message translates to:
  /// **'Resolve'**
  String get huntResolve;

  /// No description provided for @huntReapWindow.
  ///
  /// In en, this message translates to:
  /// **'Reap'**
  String get huntReapWindow;

  /// No description provided for @huntSeconds.
  ///
  /// In en, this message translates to:
  /// **'{seconds}s'**
  String huntSeconds(int seconds);

  /// No description provided for @huntCadence.
  ///
  /// In en, this message translates to:
  /// **'Cadence ×{hits}'**
  String huntCadence(int hits);

  /// No description provided for @huntPromptStalk.
  ///
  /// In en, this message translates to:
  /// **'Turn slowly. It has not seen you.'**
  String get huntPromptStalk;

  /// No description provided for @huntPromptStalkDrag.
  ///
  /// In en, this message translates to:
  /// **'Drag to look around. Move slowly: it has not seen you.'**
  String get huntPromptStalkDrag;

  /// No description provided for @huntPromptSteady.
  ///
  /// In en, this message translates to:
  /// **'Hold steady on it.'**
  String get huntPromptSteady;

  /// No description provided for @huntPromptStrike.
  ///
  /// In en, this message translates to:
  /// **'Strike the open seals.'**
  String get huntPromptStrike;

  /// No description provided for @huntPromptWindup.
  ///
  /// In en, this message translates to:
  /// **'It is drawing breath.'**
  String get huntPromptWindup;

  /// No description provided for @huntPromptHold.
  ///
  /// In en, this message translates to:
  /// **'Do not flinch'**
  String get huntPromptHold;

  /// No description provided for @huntPromptCharge.
  ///
  /// In en, this message translates to:
  /// **'It is coming. Swing across it.'**
  String get huntPromptCharge;

  /// No description provided for @huntParried.
  ///
  /// In en, this message translates to:
  /// **'Turned aside'**
  String get huntParried;

  /// No description provided for @huntStruck.
  ///
  /// In en, this message translates to:
  /// **'It struck you'**
  String get huntStruck;

  /// No description provided for @huntPromptMark.
  ///
  /// In en, this message translates to:
  /// **'Bring the blade to the mark.'**
  String get huntPromptMark;

  /// No description provided for @huntPromptSweep.
  ///
  /// In en, this message translates to:
  /// **'Sweep along the arc. Slowly.'**
  String get huntPromptSweep;

  /// No description provided for @huntAmbush.
  ///
  /// In en, this message translates to:
  /// **'Ambush'**
  String get huntAmbush;

  /// No description provided for @huntAmbushBonus.
  ///
  /// In en, this message translates to:
  /// **'+10%'**
  String get huntAmbushBonus;

  /// No description provided for @huntNoticed.
  ///
  /// In en, this message translates to:
  /// **'It sees you'**
  String get huntNoticed;

  /// No description provided for @huntFlinched.
  ///
  /// In en, this message translates to:
  /// **'Flinched'**
  String get huntFlinched;

  /// No description provided for @huntSteadfast.
  ///
  /// In en, this message translates to:
  /// **'Steadfast'**
  String get huntSteadfast;

  /// No description provided for @huntHeldFirm.
  ///
  /// In en, this message translates to:
  /// **'Unflinching'**
  String get huntHeldFirm;

  /// No description provided for @huntBroken.
  ///
  /// In en, this message translates to:
  /// **'Ward broken'**
  String get huntBroken;

  /// No description provided for @huntRecovered.
  ///
  /// In en, this message translates to:
  /// **'It gathers itself'**
  String get huntRecovered;

  /// No description provided for @huntReapedLabel.
  ///
  /// In en, this message translates to:
  /// **'Reaped'**
  String get huntReapedLabel;

  /// No description provided for @huntLostLabel.
  ///
  /// In en, this message translates to:
  /// **'Lost'**
  String get huntLostLabel;

  /// No description provided for @huntGradePerfect.
  ///
  /// In en, this message translates to:
  /// **'A perfect reap'**
  String get huntGradePerfect;

  /// No description provided for @huntGradeClean.
  ///
  /// In en, this message translates to:
  /// **'A clean reap'**
  String get huntGradeClean;

  /// No description provided for @huntGradeRagged.
  ///
  /// In en, this message translates to:
  /// **'A ragged reap'**
  String get huntGradeRagged;

  /// No description provided for @huntEscapedTitle.
  ///
  /// In en, this message translates to:
  /// **'It slipped away'**
  String get huntEscapedTitle;

  /// No description provided for @huntEscapedResolve.
  ///
  /// In en, this message translates to:
  /// **'You flinched until nothing was left to hold it with. It is still out there.'**
  String get huntEscapedResolve;

  /// No description provided for @huntEscapedTime.
  ///
  /// In en, this message translates to:
  /// **'The fight ran too long and the husk found a way out. It is still out there.'**
  String get huntEscapedTime;

  /// No description provided for @huntEscapedReap.
  ///
  /// In en, this message translates to:
  /// **'The reap was left unfinished too often. It is still out there.'**
  String get huntEscapedReap;

  /// No description provided for @huntAbsorbed.
  ///
  /// In en, this message translates to:
  /// **'{name} is part of you now.'**
  String huntAbsorbed(String name);

  /// No description provided for @huntPracticeDone.
  ///
  /// In en, this message translates to:
  /// **'A stray, so nothing was gained and nothing was owed.'**
  String get huntPracticeDone;

  /// No description provided for @huntLeave.
  ///
  /// In en, this message translates to:
  /// **'Lower the blade'**
  String get huntLeave;

  /// No description provided for @huntFailedTitle.
  ///
  /// In en, this message translates to:
  /// **'The veil would not part'**
  String get huntFailedTitle;

  /// No description provided for @huntFailedBody.
  ///
  /// In en, this message translates to:
  /// **'The hunt could not be started: {reason}'**
  String huntFailedBody(String reason);
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
