// THESIS: Read the decisions, then exercise the exact rules they explain.
// OWN-WORLD: Veil ink, bone, Fraunces and the established ruled ledger.
// STORY: Permanent birth; earned evolution; observable luck; an honest NFT plan.
// FIRST VIEWPORT: A bounded reading column with four discoverable chapters.
// FORM: Precisely scoped codex extension; no replacement identity or comp.
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../l10n/soul_lookups.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';
import '../widgets/soul_ledger.dart';

/// A readable decision record using real rules and unsaved illustrations.
/// The random boundary is injectable so a test can reproduce a draw.
class DecisionLedger extends HookWidget {
  const DecisionLedger({super.key, this.random});

  static const route = '/decisions';
  final Random? random;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final chapter = useState(0);
    final scroll = useScrollController();
    final draw = useMemoized(() => random ?? Random.secure(), [random]);
    final labels = [
      l10n.ledgerEvolution,
      l10n.ledgerChance,
      l10n.ledgerNfts,
      l10n.ledgerDecisions,
    ];

    return Scaffold(
      appBar: AppBar(title: Text(l10n.appTitle, style: Veil.display(22))),
      body: SafeArea(
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 840),
            child: SingleChildScrollView(
              controller: scroll,
              padding: const EdgeInsets.fromLTRB(24, 24, 24, 64),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Semantics(
                    header: true,
                    child: Text(l10n.ledgerTitle, style: Veil.display(42)),
                  ),
                  const SizedBox(height: 18),
                  _Prose(l10n.ledgerIntro, style: Veil.body(size: 17)),
                  const SizedBox(height: 24),
                  Wrap(
                    spacing: 8,
                    runSpacing: 8,
                    children: [
                      for (var index = 0; index < labels.length; index++)
                        ChoiceChip(
                          key: Key('ledger-chapter-$index'),
                          label: Text(labels[index]),
                          selected: chapter.value == index,
                          onSelected: (_) {
                            chapter.value = index;
                            scroll.jumpTo(0);
                          },
                        ),
                    ],
                  ),
                  const SizedBox(height: 18),
                  _Prose(
                    l10n.ledgerCurrent,
                    style: Veil.body(color: Veil.ash, size: 13),
                  ),
                  const Divider(height: 40),
                  switch (chapter.value) {
                    0 => const _Evolution(),
                    1 => _Chance(random: draw),
                    2 => const _NftFuture(),
                    _ => const _Decisions(),
                  },
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// All gifts reuse the codex's current and future-effect presentation.
class _Evolution extends HookWidget {
  const _Evolution();

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final calling = useState(ReaperClass.harvester);
    final selected = useState(ReaperGift.furrow);
    final level = useState(1);
    final marks = useState(0);
    final disposition = useState(Temperament.patient);
    final gift = selected.value;
    final genome = ReaperGenome.borrowed.copyWith(
      reaperClass: calling.value,
      // Given-name and epithet are zero. This preserves the R1 gift mapping.
      gaze: gift.index % SoulRules.giftsPerClass,
      tatter: ReaperGenome.borrowed.tatter + disposition.value.index,
    );
    final memory = SoulMemory(
      unseen: gift.path == SoulPath.unseen ? marks.value : 0,
      stillness: gift.path == SoulPath.stillness ? marks.value : 0,
      rhythm: gift.path == SoulPath.rhythm ? marks.value : 0,
    );

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _Prose(l10n.ledgerTraits),
        LedgerHeading(l10n.ledgerCalling),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (final office in ReaperClass.values)
              ChoiceChip(
                label: Text(className(l10n, office)),
                selected: calling.value == office,
                onSelected: (_) {
                  calling.value = office;
                  selected.value = ReaperGift.values.firstWhere(
                    (candidate) => candidate.reaperClass == office,
                  );
                },
              ),
          ],
        ),
        _Entry(
          title: className(l10n, calling.value),
          body:
              '${classLine(l10n, calling.value)}\n\n'
              '${perkName(l10n, calling.value.perk)}: '
              '${perkLine(l10n, calling.value.perk)}',
        ),
        LedgerHeading(l10n.ledgerGift),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (final candidate in ReaperGift.values.where(
              (candidate) => candidate.reaperClass == calling.value,
            ))
              ChoiceChip(
                key: Key('ledger-gift-${candidate.name}'),
                label: Text(giftName(l10n, candidate)),
                selected: gift == candidate,
                onSelected: (_) => selected.value = candidate,
              ),
          ],
        ),
        const SizedBox(height: 22),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (final item in Temperament.values)
              ChoiceChip(
                label: Text(temperamentName(l10n, item)),
                selected: disposition.value == item,
                onSelected: (_) => disposition.value = item,
              ),
          ],
        ),
        const SizedBox(height: 22),
        _Prose(l10n.ledgerLevelHelp, style: Veil.body(color: Veil.ash)),
        LedgerHeading(l10n.ledgerLevel, trailing: '${level.value}'),
        Slider(
          key: const Key('ledger-level'),
          value: level.value.toDouble(),
          min: 1,
          max: Progression.levelCap.toDouble(),
          divisions: Progression.levelCap - 1,
          label: l10n.studioLevelValue(level.value),
          onChanged: (value) => level.value = value.round(),
        ),
        Text(l10n.ledgerPathMarks, style: Veil.body()),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (final count in SoulRules.rankMarks)
              ChoiceChip(
                key: Key('ledger-marks-$count'),
                label: Text('$count ${soulPathName(l10n, gift.path)}'),
                selected: marks.value == count,
                onSelected: (_) => marks.value = count,
              ),
          ],
        ),
        const SizedBox(height: 16),
        LedgerRow(
          label: l10n.ledgerTotalXp,
          value: '${Progression.xpForLevel(level.value)}',
        ),
        LedgerRow(
          label: l10n.ledgerBaseStrike,
          value:
              '×${(1 + Progression.strikePerLevel * (level.value - 1)).toStringAsFixed(2)}',
        ),
        _Prose(l10n.ledgerTradeoff, style: Veil.body(color: Veil.ash)),
        ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: Veil.contentWidth),
          child: SoulLedger(
            genome: genome,
            level: level.value,
            memory: memory,
          ),
        ),
        _Entry(
          title: l10n.ledgerEquipment,
          body: l10n.ledgerEquipmentHelp,
        ),
        for (final temper in Temper.values)
          ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(temperName(l10n, temper), style: Veil.display(22)),
            subtitle: Text(
              l10n.ledgerWeaponGate(
                temper.levelRequired,
                temper.price,
                temper.strike.toString(),
              ),
            ),
          ),
        for (final outfit in Outfit.values)
          ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(outfitName(l10n, outfit), style: Veil.display(22)),
            subtitle: Text(
              l10n.ledgerOutfitGate(
                outfit.levelRequired,
                outfit.price,
              ),
            ),
          ),
        const SizedBox(height: 12),
        _Prose(
          l10n.ledgerSetBonus((Progression.strikePerSet * 100).round()),
          style: Veil.body(),
        ),
        _Entry(
          title: l10n.ledgerTable,
          body: l10n.ledgerProgressionFormula(
            (Progression.strikePerLevel * 100).round(),
          ),
        ),
        // The table scrolls at enlarged text sizes instead of clipping values.
        // SelectionArea retains Text's readable semantics while allowing copy.
        SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          child: SelectionArea(
            child: DataTable(
              horizontalMargin: 0,
              columnSpacing: 24,
              columns: [
                DataColumn(label: Text(l10n.ledgerLevelColumn)),
                DataColumn(label: Text(l10n.ledgerXpColumn), numeric: true),
                DataColumn(label: Text(l10n.ledgerStrikeColumn), numeric: true),
              ],
              rows: [
                for (var next = 1; next <= Progression.levelCap; next++)
                  DataRow(
                    cells: [
                      DataCell(Text('$next')),
                      DataCell(Text('${Progression.xpForLevel(next)}')),
                      DataCell(
                        Text(
                          '×${(1 + Progression.strikePerLevel * (next - 1)).toStringAsFixed(2)}',
                        ),
                      ),
                    ],
                  ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

/// Draws use fresh entropy here, while arithmetic stays in the shared rules.
class _Chance extends HookWidget {
  const _Chance({required this.random});
  final Random random;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final rarity = useState(Rarity.common);
    final grade = useState(ReapGrade.clean);
    final ambush = useState(false);
    final feeding = useState<Feeding?>(null);
    final birth = useState<ReaperGenome?>(null);
    final outcome = HuntOutcome.absorbed(
      grade: grade.value,
      ambushed: ambush.value,
      ticks: 0,
    );
    final weight = Feeding.values.fold(0, (sum, item) => sum + item.weight);
    final omenWeight = Omen.values.fold(0, (sum, item) => sum + item.weight);
    final expected = Feeding.values.fold(
      0.0,
      (sum, item) =>
          sum + outcome.xpFor(rarity.value, item) * item.weight / weight,
    );

    void changeScenario() => feeding.value = null;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _Entry(title: l10n.ledgerLuckHeading, body: l10n.ledgerLuckBody),
        _Entry(title: l10n.ledgerBirthOdds, body: l10n.ledgerBirthBody),
        LedgerButton(
          key: const Key('ledger-draw-birth'),
          label: l10n.ledgerDraw,
          onPressed: () =>
              birth.value = ReaperGenome.shuffle(random.nextInt(0x100000000)),
        ),
        if (birth.value case final genome?) ...[
          const SizedBox(height: 16),
          Semantics(
            liveRegion: true,
            child: _Prose(
              l10n.ledgerBirthPreview(
                genome.name,
                className(l10n, genome.reaperClass),
                giftName(l10n, ReaperSoul.fromGenome(genome).gift),
                temperamentName(
                  l10n,
                  ReaperSoul.fromGenome(genome).temperament,
                ),
                omenName(l10n, genome.omen),
              ),
              style: Veil.body(),
            ),
          ),
        ],
        LedgerHeading(l10n.ledgerOmenOdds),
        for (final omen in Omen.values)
          LedgerRow(
            label: omenName(l10n, omen),
            value: l10n.ledgerOmenRow(
              (omen.weight * 100 / omenWeight).toStringAsFixed(0),
            ),
          ),
        _Entry(title: l10n.ledgerScenario, body: l10n.ledgerRollEmpty),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (final item in Rarity.values)
              ChoiceChip(
                label: Text(rarityName(l10n, item)),
                selected: rarity.value == item,
                onSelected: (_) {
                  rarity.value = item;
                  changeScenario();
                },
              ),
          ],
        ),
        LedgerHeading(l10n.ledgerGrade),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (final item in ReapGrade.values)
              ChoiceChip(
                label: Text(switch (item) {
                  ReapGrade.ragged => l10n.huntGradeRagged,
                  ReapGrade.clean => l10n.huntGradeClean,
                  ReapGrade.perfect => l10n.huntGradePerfect,
                }),
                selected: grade.value == item,
                onSelected: (_) {
                  grade.value = item;
                  changeScenario();
                },
              ),
          ],
        ),
        SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(l10n.ledgerAmbush),
          value: ambush.value,
          onChanged: (value) {
            ambush.value = value;
            changeScenario();
          },
        ),
        LedgerHeading(l10n.ledgerFeedingOdds),
        for (final item in Feeding.values)
          ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(feedingName(l10n, item), style: Veil.display(22)),
            subtitle: Text(
              l10n.ledgerChanceRow(
                (item.weight * 100 / weight).toStringAsFixed(0),
                item.share.toString(),
                outcome.xpFor(rarity.value, item),
              ),
            ),
          ),
        const SizedBox(height: 16),
        _Prose(
          l10n.ledgerAverage(
            expected.toStringAsFixed(2),
            outcome.animaFor(rarity.value),
          ),
          style: Veil.body(),
        ),
        const SizedBox(height: 20),
        LedgerButton(
          key: const Key('ledger-roll-feeding'),
          label: l10n.ledgerRoll,
          onPressed: () =>
              feeding.value = Feeding.roll(random.nextInt(0x100000000)),
        ),
        if (feeding.value case final rolled?) ...[
          const SizedBox(height: 16),
          Semantics(
            liveRegion: true,
            child: _Prose(
              l10n.ledgerRollResult(
                feedingName(l10n, rolled),
                outcome.xpFor(rarity.value, rolled),
                outcome.animaFor(rarity.value),
              ),
              style: Veil.display(24),
            ),
          ),
        ],
        const SizedBox(height: 14),
        _Prose(l10n.ledgerRollNotice, style: Veil.body(color: Veil.ash)),
        _Entry(
          title: l10n.ledgerEncounter,
          body: l10n.ledgerEncounterBody(
            PlayerState.surfaceMinSeconds,
            PlayerState.surfaceMaxSeconds,
          ),
        ),
        _Entry(title: l10n.ledgerTrust, body: l10n.ledgerTrustBody),
        _Prose(l10n.ledgerNoMutation, style: Veil.body(color: Veil.ash)),
      ],
    );
  }
}

class _NftFuture extends HookWidget {
  const _NftFuture();
  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _Entry(title: l10n.ledgerNftHeading, body: l10n.ledgerNftBody),
        _Entry(
          title: l10n.ledgerNftCurrentTitle,
          body: l10n.ledgerNftCurrentBody,
        ),
        _Entry(title: l10n.ledgerNftNextTitle, body: l10n.ledgerNftNextBody),
        _Entry(title: l10n.ledgerNftRiftsTitle, body: l10n.ledgerNftRiftsBody),
        _Entry(title: l10n.ledgerNftLaterTitle, body: l10n.ledgerNftLaterBody),
        const Divider(height: 36),
        _Prose(l10n.ledgerNftStatus, style: Veil.body(color: Veil.ash)),
        _Entry(title: l10n.ledgerNftEconomy, body: l10n.ledgerNftEconomyBody),
      ],
    );
  }
}

class _Decisions extends HookWidget {
  const _Decisions();
  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _Entry(title: l10n.ledgerDesignHeading, body: l10n.ledgerDecisionBody),
        _Entry(title: l10n.ledgerPhysicalTitle, body: l10n.ledgerPhysicalBody),
        _Entry(title: l10n.ledgerIdentityTitle, body: l10n.ledgerIdentityBody),
        _Entry(title: l10n.ledgerMemoryTitle, body: l10n.ledgerMemoryBody),
        _Entry(title: l10n.ledgerFairTitle, body: l10n.ledgerFairBody),
        _Entry(title: l10n.ledgerResearchTitle, body: l10n.ledgerResearchBody),
        _Entry(title: l10n.ledgerBalanceTitle, body: l10n.ledgerBalanceBody),
        _Entry(title: l10n.ledgerSourcesTitle, body: l10n.ledgerSourcesBody),
        _Entry(title: l10n.ledgerNeedsTitle, body: l10n.ledgerNeedsBody),
      ],
    );
  }
}

/// A heading and a readable entry, without decorative category eyebrows.
class _Entry extends HookWidget {
  const _Entry({required this.title, required this.body});
  final String title;
  final String body;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 24, bottom: 16),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Semantics(header: true, child: Text(title, style: Veil.display(28))),
        const SizedBox(height: 12),
        _Prose(body),
      ],
    ),
  );
}

/// A legible reading measure with copyable text and readable semantics.
class _Prose extends HookWidget {
  const _Prose(this.text, {this.style});
  final String text;
  final TextStyle? style;

  @override
  Widget build(BuildContext context) => ConstrainedBox(
    constraints: const BoxConstraints(maxWidth: Veil.contentWidth),
    child: SelectionArea(child: Text(text, style: style ?? Veil.body())),
  );
}
