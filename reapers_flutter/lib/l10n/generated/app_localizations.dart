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

  /// No description provided for @ledgerTitle.
  ///
  /// In en, this message translates to:
  /// **'The Book of Becoming'**
  String get ledgerTitle;

  /// No description provided for @ledgerOpen.
  ///
  /// In en, this message translates to:
  /// **'Read the design decisions'**
  String get ledgerOpen;

  /// No description provided for @ledgerIntro.
  ///
  /// In en, this message translates to:
  /// **'How a reaper becomes itself. Explore the rules in the game today, the role of luck, and the work ahead for an NFT world.'**
  String get ledgerIntro;

  /// No description provided for @ledgerEvolution.
  ///
  /// In en, this message translates to:
  /// **'Evolution'**
  String get ledgerEvolution;

  /// No description provided for @ledgerChance.
  ///
  /// In en, this message translates to:
  /// **'Chance'**
  String get ledgerChance;

  /// No description provided for @ledgerNfts.
  ///
  /// In en, this message translates to:
  /// **'NFT future'**
  String get ledgerNfts;

  /// No description provided for @ledgerDecisions.
  ///
  /// In en, this message translates to:
  /// **'Decisions'**
  String get ledgerDecisions;

  /// No description provided for @ledgerCurrent.
  ///
  /// In en, this message translates to:
  /// **'Current rules · illustrative controls · nothing is saved or traded here.'**
  String get ledgerCurrent;

  /// No description provided for @ledgerCalling.
  ///
  /// In en, this message translates to:
  /// **'Choose an office'**
  String get ledgerCalling;

  /// No description provided for @ledgerGift.
  ///
  /// In en, this message translates to:
  /// **'Choose a birth gift'**
  String get ledgerGift;

  /// No description provided for @ledgerLevel.
  ///
  /// In en, this message translates to:
  /// **'Reaper level'**
  String get ledgerLevel;

  /// No description provided for @ledgerTraits.
  ///
  /// In en, this message translates to:
  /// **'Every birth combines an office, one of its three gifts and a temperament. The fixed genome also carries its name, silhouette, colour and aura. Rare cosmetics do not grant stronger powers.'**
  String get ledgerTraits;

  /// No description provided for @ledgerLevelHelp.
  ///
  /// In en, this message translates to:
  /// **'Experience strengthens strikes. An awakening needs both the required level and remembered successful hunts in the gift’s path. Raising a level slider does not earn those memories.'**
  String get ledgerLevelHelp;

  /// No description provided for @ledgerPathMarks.
  ///
  /// In en, this message translates to:
  /// **'Try remembered hunts'**
  String get ledgerPathMarks;

  /// No description provided for @ledgerTotalXp.
  ///
  /// In en, this message translates to:
  /// **'Total XP to reach this level'**
  String get ledgerTotalXp;

  /// No description provided for @ledgerBaseStrike.
  ///
  /// In en, this message translates to:
  /// **'Strike from levels alone'**
  String get ledgerBaseStrike;

  /// No description provided for @ledgerTradeoff.
  ///
  /// In en, this message translates to:
  /// **'Temperaments carry a benefit and a cost. The complete current gift, its lore and all three future effects are below.'**
  String get ledgerTradeoff;

  /// No description provided for @ledgerTable.
  ///
  /// In en, this message translates to:
  /// **'Every level, written out'**
  String get ledgerTable;

  /// No description provided for @ledgerLevelColumn.
  ///
  /// In en, this message translates to:
  /// **'Level'**
  String get ledgerLevelColumn;

  /// No description provided for @ledgerXpColumn.
  ///
  /// In en, this message translates to:
  /// **'Total XP'**
  String get ledgerXpColumn;

  /// No description provided for @ledgerStrikeColumn.
  ///
  /// In en, this message translates to:
  /// **'Level strike'**
  String get ledgerStrikeColumn;

  /// No description provided for @ledgerEquipment.
  ///
  /// In en, this message translates to:
  /// **'What equipment changes'**
  String get ledgerEquipment;

  /// No description provided for @ledgerEquipmentHelp.
  ///
  /// In en, this message translates to:
  /// **'Any class can learn another weapon. Weapon temper multiplies strikes; outfits change the silhouette and light. Unlocking a level permits a purchase with earned anima; it does not give the item for free.'**
  String get ledgerEquipmentHelp;

  /// No description provided for @ledgerWeaponGate.
  ///
  /// In en, this message translates to:
  /// **'Weapon temper · level {level} · {price} anima · ×{factor} strike'**
  String ledgerWeaponGate(int level, int price, String factor);

  /// No description provided for @ledgerOutfitGate.
  ///
  /// In en, this message translates to:
  /// **'Outfit · level {level} · {price} anima'**
  String ledgerOutfitGate(int level, int price);

  /// No description provided for @ledgerSetBonus.
  ///
  /// In en, this message translates to:
  /// **'A completed collection set adds {bonus}% to strikes. Anima belongs to the account; XP, equipment, codex and memories belong to the individual reaper.'**
  String ledgerSetBonus(int bonus);

  /// No description provided for @ledgerLuckHeading.
  ///
  /// In en, this message translates to:
  /// **'Chance gives the same hunt a different ending.'**
  String get ledgerLuckHeading;

  /// No description provided for @ledgerLuckBody.
  ///
  /// In en, this message translates to:
  /// **'Two equally clean absorptions can feed differently. Birth, encounters and feeding use fresh random draws. Once chosen, a draw is recorded so the keeper can check the same hunt and preserve its history.'**
  String get ledgerLuckBody;

  /// No description provided for @ledgerBirthOdds.
  ///
  /// In en, this message translates to:
  /// **'The shuffle'**
  String get ledgerBirthOdds;

  /// No description provided for @ledgerBirthBody.
  ///
  /// In en, this message translates to:
  /// **'Offices are drawn equally. Names, body variants and continuous traits are drawn at birth. Gifts and temperaments are derived from those permanent fields; they are not separate rarity prizes. An omen changes appearance only.'**
  String get ledgerBirthBody;

  /// No description provided for @ledgerDraw.
  ///
  /// In en, this message translates to:
  /// **'Draw an illustrative birth'**
  String get ledgerDraw;

  /// No description provided for @ledgerBirthPreview.
  ///
  /// In en, this message translates to:
  /// **'{name} · {calling}\n{gift} · {temperament} · {omen}'**
  String ledgerBirthPreview(
    String name,
    String calling,
    String gift,
    String temperament,
    String omen,
  );

  /// No description provided for @ledgerOmenOdds.
  ///
  /// In en, this message translates to:
  /// **'Cosmetic omen odds'**
  String get ledgerOmenOdds;

  /// No description provided for @ledgerFeedingOdds.
  ///
  /// In en, this message translates to:
  /// **'Feeding odds'**
  String get ledgerFeedingOdds;

  /// No description provided for @ledgerChanceRow.
  ///
  /// In en, this message translates to:
  /// **'{chance}% · ×{factor} XP · {xp} XP'**
  String ledgerChanceRow(String chance, String factor, int xp);

  /// No description provided for @ledgerOmenRow.
  ///
  /// In en, this message translates to:
  /// **'{chance}% of birth draws'**
  String ledgerOmenRow(String chance);

  /// No description provided for @ledgerScenario.
  ///
  /// In en, this message translates to:
  /// **'Try one successful absorption'**
  String get ledgerScenario;

  /// No description provided for @ledgerGrade.
  ///
  /// In en, this message translates to:
  /// **'Reap quality'**
  String get ledgerGrade;

  /// No description provided for @ledgerAmbush.
  ///
  /// In en, this message translates to:
  /// **'The hunt began with an ambush'**
  String get ledgerAmbush;

  /// No description provided for @ledgerRoll.
  ///
  /// In en, this message translates to:
  /// **'Roll an illustrative feeding'**
  String get ledgerRoll;

  /// No description provided for @ledgerRollEmpty.
  ///
  /// In en, this message translates to:
  /// **'Choose a husk and cut, then roll. These are local demonstrations, never a purchase or a reward claim.'**
  String get ledgerRollEmpty;

  /// No description provided for @ledgerRollResult.
  ///
  /// In en, this message translates to:
  /// **'{feeding} · {xp} XP · {anima} anima'**
  String ledgerRollResult(String feeding, int xp, int anima);

  /// No description provided for @ledgerRollNotice.
  ///
  /// In en, this message translates to:
  /// **'One illustration. The percentages above are the odds; a few draws are not a prediction. Feeding changes XP only. The same successful hunt still earns its demonstrated memory marks.'**
  String get ledgerRollNotice;

  /// No description provided for @ledgerAverage.
  ///
  /// In en, this message translates to:
  /// **'Expected XP for this scenario: {xp}. Anima: {anima}. Expectation uses each rounded reward and its weight.'**
  String ledgerAverage(String xp, int anima);

  /// No description provided for @ledgerEncounter.
  ///
  /// In en, this message translates to:
  /// **'An encounter has its own draw'**
  String get ledgerEncounter;

  /// No description provided for @ledgerEncounterBody.
  ///
  /// In en, this message translates to:
  /// **'A released husk surfaces after {min}–{max} seconds in the prototype. Its seed varies seals, movement and warnings. Retrying that same released husk keeps its encounter seed; a new release receives a new one.'**
  String ledgerEncounterBody(int min, int max);

  /// No description provided for @ledgerTrust.
  ///
  /// In en, this message translates to:
  /// **'Who controls the luck?'**
  String get ledgerTrust;

  /// No description provided for @ledgerTrustBody.
  ///
  /// In en, this message translates to:
  /// **'Online feeding is drawn once by the keeper after an accepted absorption. The receipt records and signs the outcome. This proves what that keeper signed; it does not prove the keeper chose the seed without bias. Publicly verifiable randomness and a commitment protocol are future work. Offline draws are local and cannot be imported as verified NFT growth.'**
  String get ledgerTrustBody;

  /// No description provided for @ledgerNoMutation.
  ///
  /// In en, this message translates to:
  /// **'Awakenings currently deepen a permanent gift at clear milestones. There are no random permanent mutations in this version. Random evolution is an open product decision; it must be designed and versioned before it changes existing NFTs.'**
  String get ledgerNoMutation;

  /// No description provided for @ledgerNftHeading.
  ///
  /// In en, this message translates to:
  /// **'An NFT with a life, not just a portrait.'**
  String get ledgerNftHeading;

  /// No description provided for @ledgerNftBody.
  ///
  /// In en, this message translates to:
  /// **'The reaper NFT is the continuing character. Its permanent birth identity stays fixed. Its earned XP, equipment, codex and memories follow that asset to its next owner. A player can hold several reapers; each grows separately. A starter remains non-tradable.'**
  String get ledgerNftBody;

  /// No description provided for @ledgerNftCurrentTitle.
  ///
  /// In en, this message translates to:
  /// **'Implemented foundation'**
  String get ledgerNftCurrentTitle;

  /// No description provided for @ledgerNftCurrentBody.
  ///
  /// In en, this message translates to:
  /// **'Solana Kit verifies wallet signatures and finalized Metaplex Core ownership. The configured collection and permanent genome contract govern imports. Canonical server records preserve growth on transfer. The real wallet balance is separate from the prototype’s simulated market.'**
  String get ledgerNftCurrentBody;

  /// No description provided for @ledgerNftNextTitle.
  ///
  /// In en, this message translates to:
  /// **'Next: bring an owned husk'**
  String get ledgerNftNextTitle;

  /// No description provided for @ledgerNftNextBody.
  ///
  /// In en, this message translates to:
  /// **'Proposed first playable NFT release: support a small set of approved collections and let players bring an NFT they already own. Verify the rights, traits and asset standard. Escrow it before a hunt; confirm a receipt-bound burn before transferable rewards settle. Failed or expired transactions need recovery.'**
  String get ledgerNftNextBody;

  /// No description provided for @ledgerNftRiftsTitle.
  ///
  /// In en, this message translates to:
  /// **'Then: a shared veil'**
  String get ledgerNftRiftsTitle;

  /// No description provided for @ledgerNftRiftsBody.
  ///
  /// In en, this message translates to:
  /// **'Rifts can give old collections a second shared story. Before rewarding NFT reapers, group combat needs authoritative results and safe locations. New creatures require authored models and collection permission. This phase has no promised date.'**
  String get ledgerNftRiftsBody;

  /// No description provided for @ledgerNftLaterTitle.
  ///
  /// In en, this message translates to:
  /// **'Later: a history worth reading'**
  String get ledgerNftLaterTitle;

  /// No description provided for @ledgerNftLaterBody.
  ///
  /// In en, this message translates to:
  /// **'Proposed additions: story passages reacting to remembered deeds, richer cosmetic history, collection-specific encounters and optional cooperative specialisms. A new genome version can introduce future birth rules while preserving existing identities. PvP, token rewards and random mutations are not committed features.'**
  String get ledgerNftLaterBody;

  /// No description provided for @ledgerNftStatus.
  ///
  /// In en, this message translates to:
  /// **'Real husk purchases, escrow and burning are not implemented. Production blocks their simulated substitutes. The browser studio connects no wallet. iOS wallet transport, distribution and production operations still need decisions and engineering.'**
  String get ledgerNftStatus;

  /// No description provided for @ledgerNftEconomy.
  ///
  /// In en, this message translates to:
  /// **'Why growth should be worth keeping'**
  String get ledgerNftEconomy;

  /// No description provided for @ledgerNftEconomyBody.
  ///
  /// In en, this message translates to:
  /// **'The value we can design is attachment, distinct play and a legible history. Buying a grown reaper buys its earned progression, so purchasing can confer an advantage. Burned supply does not guarantee demand, liquidity or resale value. No earnings, prices or investment returns are promised.'**
  String get ledgerNftEconomyBody;

  /// No description provided for @ledgerDesignHeading.
  ///
  /// In en, this message translates to:
  /// **'Why the rules are this way.'**
  String get ledgerDesignHeading;

  /// No description provided for @ledgerDecisionBody.
  ///
  /// In en, this message translates to:
  /// **'These are the design decisions behind the implementation, with unresolved choices kept visible. The repository’s research record separates developer evidence from our inferences.'**
  String get ledgerDecisionBody;

  /// No description provided for @ledgerPhysicalTitle.
  ///
  /// In en, this message translates to:
  /// **'The phone is the blade'**
  String get ledgerPhysicalTitle;

  /// No description provided for @ledgerPhysicalBody.
  ///
  /// In en, this message translates to:
  /// **'Stillness, a careful ambush, a held shriek and one slow reap make the hunt physically different. Gifts strengthen those existing verbs instead of adding a spell bar.'**
  String get ledgerPhysicalBody;

  /// No description provided for @ledgerIdentityTitle.
  ///
  /// In en, this message translates to:
  /// **'Birth is a promise'**
  String get ledgerIdentityTitle;

  /// No description provided for @ledgerIdentityBody.
  ///
  /// In en, this message translates to:
  /// **'A fixed identity makes a reaper recognisable and tradeable. Cosmetic rarity grants no stronger gift. The R1 mapping is permanent; future mappings require a new version.'**
  String get ledgerIdentityBody;

  /// No description provided for @ledgerMemoryTitle.
  ///
  /// In en, this message translates to:
  /// **'History means something'**
  String get ledgerMemoryTitle;

  /// No description provided for @ledgerMemoryBody.
  ///
  /// In en, this message translates to:
  /// **'One successful absorption can grant one mark per demonstrated path. Repeated holds in one fight cannot farm marks. Escapes, abandoned hunts, practice and rifts grant no soul memories. Old saves keep their XP and begin with zero remembered deeds.'**
  String get ledgerMemoryBody;

  /// No description provided for @ledgerFairTitle.
  ///
  /// In en, this message translates to:
  /// **'Luck without an invisible veto'**
  String get ledgerFairTitle;

  /// No description provided for @ledgerFairBody.
  ///
  /// In en, this message translates to:
  /// **'Feeding varies the speed of XP growth. Skill and remembered practice decide awakenings. The initial system avoids a failed upgrade roll that consumes progress. Permanent random evolution remains undecided.'**
  String get ledgerFairBody;

  /// No description provided for @ledgerResearchTitle.
  ///
  /// In en, this message translates to:
  /// **'Learned from other worlds'**
  String get ledgerResearchTitle;

  /// No description provided for @ledgerResearchBody.
  ///
  /// In en, this message translates to:
  /// **'Hades connects myth to repeatable builds. Destiny’s shared verbs and Guild Wars 2’s triggered traits informed powers with readable conditions. Disco Elysium informed action-shaped memory. Shadow of War informed persistent identity; Baldur’s Gate 3 informed a birth perspective. Slay the Spire informed testing the numbers with feedback. These are design inferences, not copied systems.'**
  String get ledgerResearchBody;

  /// No description provided for @ledgerBalanceTitle.
  ///
  /// In en, this message translates to:
  /// **'Evidence before balance claims'**
  String get ledgerBalanceTitle;

  /// No description provided for @ledgerBalanceBody.
  ///
  /// In en, this message translates to:
  /// **'All eighteen gifts have exercised combat triggers. The committed pacing trials span 1,152 encounters across 72 birth builds and two growth stages. One rare-husk bot scenario exposes large starting differences; it does not establish player enjoyment or balance. Phone playtests must test defensive value, comfort and comprehension.'**
  String get ledgerBalanceBody;

  /// No description provided for @ledgerSourcesTitle.
  ///
  /// In en, this message translates to:
  /// **'Research, code and source of truth'**
  String get ledgerSourcesTitle;

  /// No description provided for @ledgerSourcesBody.
  ///
  /// In en, this message translates to:
  /// **'The interactive values read the shared game rules, not a second calculator. Durable records: docs/gameplay.md; docs/research/reaper-identity-and-growth.md; docs/research/soul-pacing-trials.md; docs/solana.md; docs/launch-readiness.md. The six offices, eighteen gifts and their lore are original Reapers fiction.'**
  String get ledgerSourcesBody;

  /// No description provided for @ledgerNeedsTitle.
  ///
  /// In en, this message translates to:
  /// **'Open decisions for the builder'**
  String get ledgerNeedsTitle;

  /// No description provided for @ledgerNeedsBody.
  ///
  /// In en, this message translates to:
  /// **'First release and age/markets; Core collection and mint process; husk rights and standards; iOS wallet/distribution; store signing; domains, RPC, backups and receipt custody; privacy, retention and deletion; real-phone playtests. Add these before public NFT trading and burning.'**
  String get ledgerNeedsBody;

  /// No description provided for @ledgerProgressionFormula.
  ///
  /// In en, this message translates to:
  /// **'XP to the next level is 90 + 55 × (level − 1) + 6 × (level − 1)². Level strikes scale by 1 + {step}% × (level − 1); equipment, completed sets, temperament and triggered gifts then apply their own effects.'**
  String ledgerProgressionFormula(int step);

  /// No description provided for @appTitle.
  ///
  /// In en, this message translates to:
  /// **'Reapers'**
  String get appTitle;

  /// No description provided for @startupOpeningBooks.
  ///
  /// In en, this message translates to:
  /// **'Opening your books…'**
  String get startupOpeningBooks;

  /// No description provided for @startupUnreadable.
  ///
  /// In en, this message translates to:
  /// **'Your books could not be opened. Try again to recover your place in the veil.'**
  String get startupUnreadable;

  /// No description provided for @riftsLocationNote.
  ///
  /// In en, this message translates to:
  /// **'Location helps your circle face the same husk, only while a rift is open and on screen. Online play shares your latest reading, never a trail. It is optional.'**
  String get riftsLocationNote;

  /// No description provided for @locationDenied.
  ///
  /// In en, this message translates to:
  /// **'Location permission was declined. The husk will stand ahead of you. Allow location to share its place with your circle.'**
  String get locationDenied;

  /// No description provided for @locationBlocked.
  ///
  /// In en, this message translates to:
  /// **'Location permission is off for Reapers. You can allow it in Settings, or play with the husk ahead of you.'**
  String get locationBlocked;

  /// No description provided for @locationServices.
  ///
  /// In en, this message translates to:
  /// **'Location services are off. Turn them on to find where your circle stands.'**
  String get locationServices;

  /// No description provided for @locationUnsupported.
  ///
  /// In en, this message translates to:
  /// **'This device cannot read its location. The husk will stand ahead of you.'**
  String get locationUnsupported;

  /// No description provided for @locationUnavailable.
  ///
  /// In en, this message translates to:
  /// **'Your location could not be read. You can try again or keep playing.'**
  String get locationUnavailable;

  /// No description provided for @locationInaccurate.
  ///
  /// In en, this message translates to:
  /// **'Waiting for a more precise location. An open view of the sky helps; the husk will stand ahead of you meanwhile.'**
  String get locationInaccurate;

  /// No description provided for @locationStale.
  ///
  /// In en, this message translates to:
  /// **'Your location reading is too old. Waiting for a fresh reading before placing the rift.'**
  String get locationStale;

  /// No description provided for @locationSharingFailed.
  ///
  /// In en, this message translates to:
  /// **'Your location could not reach the keeper. The next fresh reading will try again.'**
  String get locationSharingFailed;

  /// No description provided for @locationSettings.
  ///
  /// In en, this message translates to:
  /// **'Open Settings'**
  String get locationSettings;

  /// No description provided for @locationSettingsFailed.
  ///
  /// In en, this message translates to:
  /// **'Settings could not be opened. Open your device Settings to allow location for Reapers.'**
  String get locationSettingsFailed;

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
  /// **'The keeper of the books could not be reached. Try again to recover your place in the veil.'**
  String get commonUnreachable;

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

  /// No description provided for @riftNoCompass.
  ///
  /// In en, this message translates to:
  /// **'This phone has no compass to say which way the rift lies, so it stands dead ahead of you.'**
  String get riftNoCompass;

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
  /// **'Drawing it out'**
  String get scanLabel;

  /// No description provided for @scanTitle.
  ///
  /// In en, this message translates to:
  /// **'Something is nearby.'**
  String get scanTitle;

  /// No description provided for @sigilHow.
  ///
  /// In en, this message translates to:
  /// **'Trace its sigil to draw it out. Turn until the ring rests on the first light, hold the phone still until it catches, then find the next.'**
  String get sigilHow;

  /// No description provided for @sigilHowDrag.
  ///
  /// In en, this message translates to:
  /// **'Trace its sigil to draw it out. Drag until the ring rests on the first light, wait there until it catches, then find the next.'**
  String get sigilHowDrag;

  /// No description provided for @sigilCaught.
  ///
  /// In en, this message translates to:
  /// **'{caught} of {total} caught'**
  String sigilCaught(int caught, int total);

  /// No description provided for @sigilSteady.
  ///
  /// In en, this message translates to:
  /// **'Hold still. It cannot catch while you move.'**
  String get sigilSteady;

  /// No description provided for @sigilSlipping.
  ///
  /// In en, this message translates to:
  /// **'It is slipping. Find the next light.'**
  String get sigilSlipping;

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

  /// No description provided for @huntHowSigil.
  ///
  /// In en, this message translates to:
  /// **'Trace its sigil: rest the ring on each light in turn until it catches.'**
  String get huntHowSigil;

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

  /// No description provided for @studioAppTitle.
  ///
  /// In en, this message translates to:
  /// **'Reaper studio'**
  String get studioAppTitle;

  /// No description provided for @studioTitle.
  ///
  /// In en, this message translates to:
  /// **'A life after death.'**
  String get studioTitle;

  /// No description provided for @studioIntro.
  ///
  /// In en, this message translates to:
  /// **'Shape a specimen. Explore its growth. Everything here is a preview; your bound reaper stays yours.'**
  String get studioIntro;

  /// No description provided for @studioTraits.
  ///
  /// In en, this message translates to:
  /// **'Birth traits'**
  String get studioTraits;

  /// No description provided for @studioGrowth.
  ///
  /// In en, this message translates to:
  /// **'Growth'**
  String get studioGrowth;

  /// No description provided for @studioClass.
  ///
  /// In en, this message translates to:
  /// **'Calling'**
  String get studioClass;

  /// No description provided for @studioHead.
  ///
  /// In en, this message translates to:
  /// **'Head'**
  String get studioHead;

  /// No description provided for @studioMantle.
  ///
  /// In en, this message translates to:
  /// **'Mantle'**
  String get studioMantle;

  /// No description provided for @studioRobe.
  ///
  /// In en, this message translates to:
  /// **'Hem'**
  String get studioRobe;

  /// No description provided for @studioPalette.
  ///
  /// In en, this message translates to:
  /// **'Colourway'**
  String get studioPalette;

  /// No description provided for @studioOmen.
  ///
  /// In en, this message translates to:
  /// **'Omen'**
  String get studioOmen;

  /// No description provided for @studioAura.
  ///
  /// In en, this message translates to:
  /// **'Aura'**
  String get studioAura;

  /// No description provided for @studioStature.
  ///
  /// In en, this message translates to:
  /// **'Stature'**
  String get studioStature;

  /// No description provided for @studioGauntness.
  ///
  /// In en, this message translates to:
  /// **'Gauntness'**
  String get studioGauntness;

  /// No description provided for @studioGaze.
  ///
  /// In en, this message translates to:
  /// **'Gaze'**
  String get studioGaze;

  /// No description provided for @studioTatter.
  ///
  /// In en, this message translates to:
  /// **'Tatter'**
  String get studioTatter;

  /// No description provided for @studioDrag.
  ///
  /// In en, this message translates to:
  /// **'Drag to turn your reaper'**
  String get studioDrag;

  /// No description provided for @studioShuffle.
  ///
  /// In en, this message translates to:
  /// **'Draw another specimen'**
  String get studioShuffle;

  /// No description provided for @studioBirthTraits.
  ///
  /// In en, this message translates to:
  /// **'Birth traits are fixed when a reaper is minted. Outfits and weapons can change as it grows.'**
  String get studioBirthTraits;

  /// No description provided for @studioCopy.
  ///
  /// In en, this message translates to:
  /// **'Copy genome'**
  String get studioCopy;

  /// No description provided for @studioCopied.
  ///
  /// In en, this message translates to:
  /// **'Genome copied'**
  String get studioCopied;

  /// No description provided for @studioLevel.
  ///
  /// In en, this message translates to:
  /// **'Level'**
  String get studioLevel;

  /// No description provided for @studioStrength.
  ///
  /// In en, this message translates to:
  /// **'Strike growth'**
  String get studioStrength;

  /// No description provided for @studioDamage.
  ///
  /// In en, this message translates to:
  /// **'Strike damage'**
  String get studioDamage;

  /// No description provided for @studioGrowthLine.
  ///
  /// In en, this message translates to:
  /// **'Compared with a new reaper of this calling, carrying its starting weapon. Levels strengthen strikes; they do not widen the capture window.'**
  String get studioGrowthLine;

  /// No description provided for @studioAbsorb.
  ///
  /// In en, this message translates to:
  /// **'Absorb a specimen'**
  String get studioAbsorb;

  /// No description provided for @studioExample.
  ///
  /// In en, this message translates to:
  /// **'Illustration: a rare husk, a clean cut and fair feeding. Real rewards depend on rarity, cut, ambush and feeding.'**
  String get studioExample;

  /// No description provided for @studioWeapon.
  ///
  /// In en, this message translates to:
  /// **'Weapon temper'**
  String get studioWeapon;

  /// No description provided for @studioOutfit.
  ///
  /// In en, this message translates to:
  /// **'Outfit'**
  String get studioOutfit;

  /// No description provided for @studioCosmetic.
  ///
  /// In en, this message translates to:
  /// **'Outfits change the silhouette and light. Weapons change your strikes. In the game, both cost earned anima after reaching their required level.'**
  String get studioCosmetic;

  /// No description provided for @studioPotential.
  ///
  /// In en, this message translates to:
  /// **'See full potential'**
  String get studioPotential;

  /// No description provided for @studioMultiple.
  ///
  /// In en, this message translates to:
  /// **'Each owned reaper grows separately. Anima belongs to you; an NFT reaper’s experience, codex and equipment follow that asset when traded.'**
  String get studioMultiple;

  /// No description provided for @studioAuraMotes.
  ///
  /// In en, this message translates to:
  /// **'Motes'**
  String get studioAuraMotes;

  /// No description provided for @studioAuraEmbers.
  ///
  /// In en, this message translates to:
  /// **'Embers'**
  String get studioAuraEmbers;

  /// No description provided for @studioAuraMist.
  ///
  /// In en, this message translates to:
  /// **'Mist'**
  String get studioAuraMist;

  /// No description provided for @studioAuraWisps.
  ///
  /// In en, this message translates to:
  /// **'Wisps'**
  String get studioAuraWisps;

  /// No description provided for @studioLevelValue.
  ///
  /// In en, this message translates to:
  /// **'Level {level}'**
  String studioLevelValue(int level);

  /// No description provided for @studioVariant.
  ///
  /// In en, this message translates to:
  /// **'{number}'**
  String studioVariant(int number);

  /// No description provided for @studioStronger.
  ///
  /// In en, this message translates to:
  /// **'+{percent}%'**
  String studioStronger(int percent);

  /// No description provided for @studioCaptures.
  ///
  /// In en, this message translates to:
  /// **'{count} absorbed'**
  String studioCaptures(int count);

  /// No description provided for @studioReward.
  ///
  /// In en, this message translates to:
  /// **'+{xp} XP in this preview'**
  String studioReward(int xp);

  /// No description provided for @studioUnlock.
  ///
  /// In en, this message translates to:
  /// **'Level {level} · {anima} anima'**
  String studioUnlock(int level, int anima);

  /// No description provided for @refusalReaperBusy.
  ///
  /// In en, this message translates to:
  /// **'Finish your loose hunts before switching reapers.'**
  String get refusalReaperBusy;

  /// No description provided for @refusalWalletUnproven.
  ///
  /// In en, this message translates to:
  /// **'The wallet proof could not be verified. Connect your wallet again.'**
  String get refusalWalletUnproven;

  /// No description provided for @refusalChainUnconfigured.
  ///
  /// In en, this message translates to:
  /// **'Reaper NFT trading is waiting for the collection to open.'**
  String get refusalChainUnconfigured;

  /// No description provided for @sanctumRoster.
  ///
  /// In en, this message translates to:
  /// **'Your reapers'**
  String get sanctumRoster;

  /// No description provided for @sanctumStudio.
  ///
  /// In en, this message translates to:
  /// **'Reaper studio'**
  String get sanctumStudio;

  /// No description provided for @sanctumStudioDetail.
  ///
  /// In en, this message translates to:
  /// **'Explore traits and growth'**
  String get sanctumStudioDetail;

  /// No description provided for @rosterTitle.
  ///
  /// In en, this message translates to:
  /// **'Your reapers'**
  String get rosterTitle;

  /// No description provided for @rosterIntro.
  ///
  /// In en, this message translates to:
  /// **'Choose who hunts tonight. Every reaper carries its own growth.'**
  String get rosterIntro;

  /// No description provided for @rosterStarter.
  ///
  /// In en, this message translates to:
  /// **'Oathbound · not tradable'**
  String get rosterStarter;

  /// No description provided for @rosterNft.
  ///
  /// In en, this message translates to:
  /// **'Solana NFT'**
  String get rosterNft;

  /// No description provided for @rosterActive.
  ///
  /// In en, this message translates to:
  /// **'Hunting tonight'**
  String get rosterActive;

  /// No description provided for @rosterConnect.
  ///
  /// In en, this message translates to:
  /// **'Connect Solana wallet'**
  String get rosterConnect;

  /// No description provided for @rosterRefresh.
  ///
  /// In en, this message translates to:
  /// **'Refresh ownership'**
  String get rosterRefresh;

  /// No description provided for @rosterImport.
  ///
  /// In en, this message translates to:
  /// **'Import owned reaper'**
  String get rosterImport;

  /// No description provided for @rosterAsset.
  ///
  /// In en, this message translates to:
  /// **'Reaper NFT address'**
  String get rosterAsset;

  /// No description provided for @rosterImportHelp.
  ///
  /// In en, this message translates to:
  /// **'Paste the address of a Reapers NFT already owned by your linked wallet.'**
  String get rosterImportHelp;

  /// No description provided for @rosterBuy.
  ///
  /// In en, this message translates to:
  /// **'Find another reaper'**
  String get rosterBuy;

  /// No description provided for @rosterBuyHelp.
  ///
  /// In en, this message translates to:
  /// **'Buy from the collection’s marketplace, then import your NFT here. Your wallet handles payment.'**
  String get rosterBuyHelp;

  /// No description provided for @rosterOffline.
  ///
  /// In en, this message translates to:
  /// **'NFT ownership needs the online keeper. You can explore specimens in the studio while playing offline.'**
  String get rosterOffline;

  /// No description provided for @rosterWalletUnsupported.
  ///
  /// In en, this message translates to:
  /// **'Connect on an Android phone with a Solana wallet installed.'**
  String get rosterWalletUnsupported;

  /// No description provided for @rosterLinkHelp.
  ///
  /// In en, this message translates to:
  /// **'Connecting proves control of your wallet. It does not spend SOL or transfer an NFT.'**
  String get rosterLinkHelp;

  /// No description provided for @rosterLinked.
  ///
  /// In en, this message translates to:
  /// **'Linked wallet'**
  String get rosterLinked;

  /// No description provided for @rosterBalance.
  ///
  /// In en, this message translates to:
  /// **'Wallet SOL'**
  String get rosterBalance;

  /// No description provided for @rosterNetwork.
  ///
  /// In en, this message translates to:
  /// **'Network'**
  String get rosterNetwork;

  /// No description provided for @rosterUnconfigured.
  ///
  /// In en, this message translates to:
  /// **'NFT imports are unavailable until a reaper collection is configured. Your starter remains yours.'**
  String get rosterUnconfigured;

  /// No description provided for @rosterCount.
  ///
  /// In en, this message translates to:
  /// **'{count} reapers'**
  String rosterCount(int count);

  /// No description provided for @studioModelFailed.
  ///
  /// In en, this message translates to:
  /// **'This specimen could not be drawn.'**
  String get studioModelFailed;

  /// No description provided for @studioWebGLUnavailable.
  ///
  /// In en, this message translates to:
  /// **'Your browser needs WebGL 2 to draw this specimen. Try another browser or open the studio on Android or iOS.'**
  String get studioWebGLUnavailable;

  /// No description provided for @studioXp.
  ///
  /// In en, this message translates to:
  /// **'XP'**
  String get studioXp;

  /// No description provided for @studioCapped.
  ///
  /// In en, this message translates to:
  /// **'Max'**
  String get studioCapped;

  /// No description provided for @studioPause.
  ///
  /// In en, this message translates to:
  /// **'Pause rotation'**
  String get studioPause;

  /// No description provided for @studioRotate.
  ///
  /// In en, this message translates to:
  /// **'Rotate specimen'**
  String get studioRotate;

  /// No description provided for @refusalRosterFull.
  ///
  /// In en, this message translates to:
  /// **'Your reaper roster is full. Transfer an NFT before importing another.'**
  String get refusalRosterFull;

  /// No description provided for @refusalRiftUnproven.
  ///
  /// In en, this message translates to:
  /// **'NFT reapers can hunt solo. Rifts will open once their results can be verified.'**
  String get refusalRiftUnproven;

  /// No description provided for @soulTitle.
  ///
  /// In en, this message translates to:
  /// **'The soul remembers'**
  String get soulTitle;

  /// No description provided for @refusalHuntVersionUnsupported.
  ///
  /// In en, this message translates to:
  /// **'The hunt rules have changed. Update Reapers before trying another hunt.'**
  String get refusalHuntVersionUnsupported;

  /// No description provided for @refusalMarketUnavailable.
  ///
  /// In en, this message translates to:
  /// **'NFT hunting hasn’t opened here yet. You can practise with your reaper.'**
  String get refusalMarketUnavailable;

  /// No description provided for @soulTab.
  ///
  /// In en, this message translates to:
  /// **'Soul'**
  String get soulTab;

  /// No description provided for @soulBirth.
  ///
  /// In en, this message translates to:
  /// **'Born with a gift'**
  String get soulBirth;

  /// No description provided for @soulTemperament.
  ///
  /// In en, this message translates to:
  /// **'{name} temperament'**
  String soulTemperament(String name);

  /// No description provided for @soulIntro.
  ///
  /// In en, this message translates to:
  /// **'Your gift is fixed at birth. The way you hunt teaches it to grow. Every reaper keeps its own memories.'**
  String get soulIntro;

  /// No description provided for @soulGiftChoice.
  ///
  /// In en, this message translates to:
  /// **'Try a gift'**
  String get soulGiftChoice;

  /// No description provided for @soulMarks.
  ///
  /// In en, this message translates to:
  /// **'Hunts remembered'**
  String get soulMarks;

  /// No description provided for @soulRank.
  ///
  /// In en, this message translates to:
  /// **'Awakening {rank} / {max}'**
  String soulRank(int rank, int max);

  /// No description provided for @soulMilestone.
  ///
  /// In en, this message translates to:
  /// **'Level {level} · {marks} {path} marks'**
  String soulMilestone(int level, int marks, String path);

  /// No description provided for @soulNext.
  ///
  /// In en, this message translates to:
  /// **'Next awakening: level {level} and {marks} {path} marks. You have {earned}.'**
  String soulNext(int level, int marks, String path, int earned);

  /// No description provided for @soulComplete.
  ///
  /// In en, this message translates to:
  /// **'Every awakening earned. Your birth gift stays yours.'**
  String get soulComplete;

  /// No description provided for @soulLore.
  ///
  /// In en, this message translates to:
  /// **'What the gift remembers'**
  String get soulLore;

  /// No description provided for @soulUnseen.
  ///
  /// In en, this message translates to:
  /// **'Unseen'**
  String get soulUnseen;

  /// No description provided for @soulStillness.
  ///
  /// In en, this message translates to:
  /// **'Stillness'**
  String get soulStillness;

  /// No description provided for @soulRhythm.
  ///
  /// In en, this message translates to:
  /// **'Rhythm'**
  String get soulRhythm;

  /// No description provided for @soulUnseenHelp.
  ///
  /// In en, this message translates to:
  /// **'Absorb a husk after an ambush.'**
  String get soulUnseenHelp;

  /// No description provided for @soulStillnessHelp.
  ///
  /// In en, this message translates to:
  /// **'Absorb a husk after an unflinching hold.'**
  String get soulStillnessHelp;

  /// No description provided for @soulRhythmHelp.
  ///
  /// In en, this message translates to:
  /// **'Absorb a husk after five landed hits in cadence.'**
  String get soulRhythmHelp;

  /// No description provided for @soulMemoryHelp.
  ///
  /// In en, this message translates to:
  /// **'One mark per path per absorption. Escapes and abandoned hunts grant none. Old hunts keep their XP; memories begin with your next hunt.'**
  String get soulMemoryHelp;

  /// No description provided for @soulPreviewHelp.
  ///
  /// In en, this message translates to:
  /// **'These memories are an unsaved specimen. Real memories come from the keeper’s replay of your hunts.'**
  String get soulPreviewHelp;

  /// No description provided for @soulPreviewMark.
  ///
  /// In en, this message translates to:
  /// **'Preview hunt: {path}'**
  String soulPreviewMark(String path);

  /// No description provided for @soulPreviewReset.
  ///
  /// In en, this message translates to:
  /// **'Clear specimen memories'**
  String get soulPreviewReset;

  /// No description provided for @soulBirthLink.
  ///
  /// In en, this message translates to:
  /// **'{gift} · {temperament}'**
  String soulBirthLink(String gift, String temperament);

  /// No description provided for @soulRankEarned.
  ///
  /// In en, this message translates to:
  /// **'Your gift awakened: {gift}, awakening {rank}.'**
  String soulRankEarned(String gift, int rank);

  /// No description provided for @soulDeedsEarned.
  ///
  /// In en, this message translates to:
  /// **'The soul remembers this hunt: {paths}.'**
  String soulDeedsEarned(String paths);

  /// No description provided for @temperamentPatient.
  ///
  /// In en, this message translates to:
  /// **'Patient'**
  String get temperamentPatient;

  /// No description provided for @temperamentKeen.
  ///
  /// In en, this message translates to:
  /// **'Keen'**
  String get temperamentKeen;

  /// No description provided for @temperamentStalwart.
  ///
  /// In en, this message translates to:
  /// **'Stalwart'**
  String get temperamentStalwart;

  /// No description provided for @temperamentHungry.
  ///
  /// In en, this message translates to:
  /// **'Hungry'**
  String get temperamentHungry;

  /// No description provided for @temperamentPatientHelp.
  ///
  /// In en, this message translates to:
  /// **'Half a second longer to reap. Every swing takes two ticks longer.'**
  String get temperamentPatientHelp;

  /// No description provided for @temperamentKeenHelp.
  ///
  /// In en, this message translates to:
  /// **'A wider reach by 0.6°. Four tenths of a second less to reap.'**
  String get temperamentKeenHelp;

  /// No description provided for @temperamentStalwartHelp.
  ///
  /// In en, this message translates to:
  /// **'One more resolve. Strikes deal 5% less damage.'**
  String get temperamentStalwartHelp;

  /// No description provided for @temperamentHungryHelp.
  ///
  /// In en, this message translates to:
  /// **'Strikes deal 8% more damage. You must hold 8% steadier through a shriek.'**
  String get temperamentHungryHelp;

  /// No description provided for @giftFurrow.
  ///
  /// In en, this message translates to:
  /// **'The Last Furrow'**
  String get giftFurrow;

  /// No description provided for @giftAftermath.
  ///
  /// In en, this message translates to:
  /// **'Aftermath'**
  String get giftAftermath;

  /// No description provided for @giftGathering.
  ///
  /// In en, this message translates to:
  /// **'The Gathering'**
  String get giftGathering;

  /// No description provided for @giftPassage.
  ///
  /// In en, this message translates to:
  /// **'Safe Passage'**
  String get giftPassage;

  /// No description provided for @giftAnchor.
  ///
  /// In en, this message translates to:
  /// **'The Anchor'**
  String get giftAnchor;

  /// No description provided for @giftReturnLight.
  ///
  /// In en, this message translates to:
  /// **'Returning Light'**
  String get giftReturnLight;

  /// No description provided for @giftForeknowledge.
  ///
  /// In en, this message translates to:
  /// **'Foreknowledge'**
  String get giftForeknowledge;

  /// No description provided for @giftVerdict.
  ///
  /// In en, this message translates to:
  /// **'The Verdict'**
  String get giftVerdict;

  /// No description provided for @giftEquilibrium.
  ///
  /// In en, this message translates to:
  /// **'Equilibrium'**
  String get giftEquilibrium;

  /// No description provided for @giftFirstLight.
  ///
  /// In en, this message translates to:
  /// **'First Light'**
  String get giftFirstLight;

  /// No description provided for @giftReprisal.
  ///
  /// In en, this message translates to:
  /// **'Reprisal'**
  String get giftReprisal;

  /// No description provided for @giftLastWord.
  ///
  /// In en, this message translates to:
  /// **'The Last Word'**
  String get giftLastWord;

  /// No description provided for @giftHunger.
  ///
  /// In en, this message translates to:
  /// **'Old Hunger'**
  String get giftHunger;

  /// No description provided for @giftEcho.
  ///
  /// In en, this message translates to:
  /// **'The Echo'**
  String get giftEcho;

  /// No description provided for @giftUnbroken.
  ///
  /// In en, this message translates to:
  /// **'Unbroken'**
  String get giftUnbroken;

  /// No description provided for @giftHush.
  ///
  /// In en, this message translates to:
  /// **'Hush'**
  String get giftHush;

  /// No description provided for @giftKeening.
  ///
  /// In en, this message translates to:
  /// **'Keening'**
  String get giftKeening;

  /// No description provided for @giftRefrain.
  ///
  /// In en, this message translates to:
  /// **'The Refrain'**
  String get giftRefrain;

  /// No description provided for @giftFurrowEffect.
  ///
  /// In en, this message translates to:
  /// **'Your ambush cleave catches other open seals for {share}% of its blow.'**
  String giftFurrowEffect(int share);

  /// No description provided for @giftAftermathEffect.
  ///
  /// In en, this message translates to:
  /// **'After an unflinching hold, your next cleave leaves the other seals open for at least {seconds}s.'**
  String giftAftermathEffect(String seconds);

  /// No description provided for @giftGatheringEffect.
  ///
  /// In en, this message translates to:
  /// **'Every third hit in cadence reopens its seal for {seconds}s.'**
  String giftGatheringEffect(String seconds);

  /// No description provided for @giftPassageEffect.
  ///
  /// In en, this message translates to:
  /// **'An ambush shields you from one charge. The husk is stunned for {seconds}s.'**
  String giftPassageEffect(String seconds);

  /// No description provided for @giftAnchorEffect.
  ///
  /// In en, this message translates to:
  /// **'An unflinching hold restores one lost resolve, {uses, plural, one{once} other{up to {uses} times}} per hunt.'**
  String giftAnchorEffect(int uses);

  /// No description provided for @giftReturnLightEffect.
  ///
  /// In en, this message translates to:
  /// **'Five hits in cadence prepare a light that forgives your next flinch, up to {uses} times per hunt.'**
  String giftReturnLightEffect(int uses);

  /// No description provided for @giftForeknowledgeEffect.
  ///
  /// In en, this message translates to:
  /// **'An ambush keeps the seals open for {seconds}s longer.'**
  String giftForeknowledgeEffect(String seconds);

  /// No description provided for @giftVerdictEffect.
  ///
  /// In en, this message translates to:
  /// **'After an unflinching hold, your next in-order strike deals {bonus}% more damage.'**
  String giftVerdictEffect(int bonus);

  /// No description provided for @giftEquilibriumEffect.
  ///
  /// In en, this message translates to:
  /// **'Three in-order strikes reopen every seal for {seconds}s.'**
  String giftEquilibriumEffect(String seconds);

  /// No description provided for @giftFirstLightEffect.
  ///
  /// In en, this message translates to:
  /// **'Your first ambush strike deals {bonus}% extra damage.'**
  String giftFirstLightEffect(int bonus);

  /// No description provided for @giftReprisalEffect.
  ///
  /// In en, this message translates to:
  /// **'A parry readies Chosen Blow, with a further {bonus}% damage.'**
  String giftReprisalEffect(int bonus);

  /// No description provided for @giftLastWordEffect.
  ///
  /// In en, this message translates to:
  /// **'At three hits in cadence, seal strikes against a husk below 35% ward deal {bonus}% more damage.'**
  String giftLastWordEffect(int bonus);

  /// No description provided for @giftHungerEffect.
  ///
  /// In en, this message translates to:
  /// **'An ambush starts your cadence at {beats}.'**
  String giftHungerEffect(int beats);

  /// No description provided for @giftEchoEffect.
  ///
  /// In en, this message translates to:
  /// **'An unflinching hold restores your cadence from the warning, plus {beats}. Cadence still caps at ten.'**
  String giftEchoEffect(int beats);

  /// No description provided for @giftUnbrokenEffect.
  ///
  /// In en, this message translates to:
  /// **'Up to {uses} misses preserve cadence each hunt. Its timer keeps running.'**
  String giftUnbrokenEffect(int uses);

  /// No description provided for @giftHushEffect.
  ///
  /// In en, this message translates to:
  /// **'An ambush delays the first shriek by {seconds}s.'**
  String giftHushEffect(String seconds);

  /// No description provided for @giftKeeningEffect.
  ///
  /// In en, this message translates to:
  /// **'After an unflinching hold, your next seal strike deals {bonus}% more damage.'**
  String giftKeeningEffect(int bonus);

  /// No description provided for @giftRefrainEffect.
  ///
  /// In en, this message translates to:
  /// **'Every third hit in cadence adds {seconds}s to your reap window, up to six times per hunt.'**
  String giftRefrainEffect(String seconds);

  /// No description provided for @giftFurrowLore.
  ///
  /// In en, this message translates to:
  /// **'Nothing in the field dies alone. The first cut follows the roots beneath it.'**
  String get giftFurrowLore;

  /// No description provided for @giftAftermathLore.
  ///
  /// In en, this message translates to:
  /// **'The harvest remembers what survived. Those touched by the blade remain within its reach.'**
  String get giftAftermathLore;

  /// No description provided for @giftGatheringLore.
  ///
  /// In en, this message translates to:
  /// **'One stalk, then another. The hand finds a rhythm that the field cannot close against.'**
  String get giftGatheringLore;

  /// No description provided for @giftPassageLore.
  ///
  /// In en, this message translates to:
  /// **'A ferryman promises arrival. Even a furious passenger cannot overturn that promise.'**
  String get giftPassageLore;

  /// No description provided for @giftAnchorLore.
  ///
  /// In en, this message translates to:
  /// **'The crossing has no shore beneath it. The ferryman learns to make a shore of stillness.'**
  String get giftAnchorLore;

  /// No description provided for @giftReturnLightLore.
  ///
  /// In en, this message translates to:
  /// **'Every stroke of the pole leaves a little light behind. Enough light can guide a trembling hand home.'**
  String get giftReturnLightLore;

  /// No description provided for @giftForeknowledgeLore.
  ///
  /// In en, this message translates to:
  /// **'A careful keeper opens the ledger before the dead arrive. The first reading leaves nothing concealed.'**
  String get giftForeknowledgeLore;

  /// No description provided for @giftVerdictLore.
  ///
  /// In en, this message translates to:
  /// **'Judgement begins when the room falls silent. Only the right seal can carry the sentence.'**
  String get giftVerdictLore;

  /// No description provided for @giftEquilibriumLore.
  ///
  /// In en, this message translates to:
  /// **'Three true measures bring the scales level. For a moment, every debt lies open.'**
  String get giftEquilibriumLore;

  /// No description provided for @giftFirstLightLore.
  ///
  /// In en, this message translates to:
  /// **'The chooser sees a death before anyone else does. The first light is also the first wound.'**
  String get giftFirstLightLore;

  /// No description provided for @giftReprisalLore.
  ///
  /// In en, this message translates to:
  /// **'A death that reaches for the chooser has chosen itself. The answer arrives on the returning blade.'**
  String get giftReprisalLore;

  /// No description provided for @giftLastWordLore.
  ///
  /// In en, this message translates to:
  /// **'A spirit may argue until the end. The chooser saves the last word for what remains.'**
  String get giftLastWordLore;

  /// No description provided for @giftHungerLore.
  ///
  /// In en, this message translates to:
  /// **'The empty collar remembers every meal it missed. Silence gives that hunger a head start.'**
  String get giftHungerLore;

  /// No description provided for @giftEchoLore.
  ///
  /// In en, this message translates to:
  /// **'A shriek cannot drown out an absence. Beneath the noise, the old rhythm waits.'**
  String get giftEchoLore;

  /// No description provided for @giftUnbrokenLore.
  ///
  /// In en, this message translates to:
  /// **'There is no head to lose. Even a wandering hand cannot break the chain at once.'**
  String get giftUnbrokenLore;

  /// No description provided for @giftHushLore.
  ///
  /// In en, this message translates to:
  /// **'Before a death is announced, the mourner grants a little silence. Even the dead must wait their turn.'**
  String get giftHushLore;

  /// No description provided for @giftKeeningLore.
  ///
  /// In en, this message translates to:
  /// **'The mourner listens until the cry has passed. The next toll carries what was heard.'**
  String get giftKeeningLore;

  /// No description provided for @giftRefrainLore.
  ///
  /// In en, this message translates to:
  /// **'A bell cannot call a name only once. Each refrain gives the departing soul longer to answer.'**
  String get giftRefrainLore;
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
