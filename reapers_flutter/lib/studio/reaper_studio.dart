// THESIS: Explore one real reaper, then demonstrate its earned growth.
// OWN-WORLD: Inherit Veil's ink, bone, specimen light and funeral-ledger kit.
// STORY: Try fixed birth traits; absorb illustrative husks; inspect upgrades.
// FIRST VIEWPORT: Large model left, bounded controls right; stack on phones.
// FORM: Incumbent model viewer with a trait/progression inspector. Seed n/a:
// this scoped extension preserves the existing sanctum and armory vocabulary.
// FINISH: unreviewed and undocumented is unfinished; this build ends with the
// finish review, the verdict, DESIGN.md, and every shipping raster carrying
// its provenance
import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../l10n/soul_lookups.dart';
import '../render/reaper_look.dart';
import '../render/reaper_stage.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';
import '../widgets/soul_ledger.dart';

typedef StudioStageBuilder = Widget Function(ReaperGenome, Weapon, Outfit);

/// An unsaved specimen using the game's models and shared progression rules.
class ReaperStudio extends HookWidget {
  const ReaperStudio({super.key, this.initialGenome, this.stageBuilder});

  final ReaperGenome? initialGenome;
  final StudioStageBuilder? stageBuilder;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final genome = useState(initialGenome ?? ReaperGenome.shuffle(11));
    final xp = useState(0);
    final memory = useState(const SoulMemory());
    final outfit = useState(Outfit.shrouded);
    final temper = useState(Temper.shrouded);
    final tab = useState(0);
    final seed = useRef(11);
    final captures = useState(0);
    final reward = useState<int?>(null);
    final spinning = useState(true);
    useEffect(() {
      if (!spinning.value) return null;
      // One slow revolution, then hold the specimen. The player can replay it.
      final stop = Timer(
        const Duration(seconds: 25),
        () => spinning.value = false,
      );
      return stop.cancel;
    }, [spinning.value]);
    final level = Progression.levelForXp(xp.value);
    final weapon = Weapon(genome.value.reaperClass.weapon, temper.value);
    final soul = ReaperSoul.fromGenome(genome.value);
    final soulRank = SoulRules.rank(
      level: level,
      memory: memory.value,
      gift: soul.gift,
    );
    final stats = HuntStats.derive(
      HuntLoadout(
        reaperClass: genome.value.reaperClass,
        level: level,
        weapon: weapon,
        completedSets: 0,
        soul: soul,
        soulRank: soulRank,
      ),
    );
    final starting = HuntStats.derive(
      HuntLoadout(
        reaperClass: genome.value.reaperClass,
        level: 1,
        weapon: Weapon(genome.value.reaperClass.weapon, Temper.shrouded),
        completedSets: 0,
        soul: soul,
      ),
    );
    final light = reaperLight(genome.value);

    void setLevel(int next) {
      xp.value = Progression.xpForLevel(next);
      reward.value = null;
      if (temper.value.levelRequired > next) temper.value = Temper.shrouded;
      if (outfit.value.levelRequired > next) outfit.value = Outfit.shrouded;
    }

    void absorb({SoulPath? path}) {
      // Illustrative clean cut and fair feeding, calculated by the same rules
      // as a real hunt. This never submits a claim or touches the saved player.
      final outcome = HuntOutcome.absorbed(
        grade: ReapGrade.clean,
        ambushed: path == SoulPath.unseen,
        ticks: 0,
        deeds: HuntDeeds(
          holds: path == SoulPath.stillness ? 1 : 0,
          maxCadence: path == SoulPath.rhythm ? SoulRules.cadenceForMark : 0,
        ),
      );
      final earned = outcome.xpFor(Rarity.rare, Feeding.fair);
      xp.value += earned;
      captures.value++;
      reward.value = earned;
      memory.value = memory.value.remember(outcome);
    }

    void tryGift(ReaperGift gift) {
      final delta = (gift.index - soul.gift.index) % SoulRules.giftsPerClass;
      var gaze = genome.value.gaze + delta;
      if (gaze > ReaperGenome.traitScale) gaze -= SoulRules.giftsPerClass;
      genome.value = genome.value.copyWith(gaze: gaze);
    }

    final portrait = Stack(
      fit: StackFit.expand,
      children: [
        stageBuilder?.call(genome.value, weapon, outfit.value) ??
            ReaperStage(
              genome: genome.value,
              weapon: weapon,
              outfit: outfit.value,
              turnsOnItsOwn: spinning.value,
            ),
        Positioned(
          left: 24,
          top: 24,
          child: Text(
            className(l10n, genome.value.reaperClass).toUpperCase(),
            style: Veil.label(color: light),
          ),
        ),
        Positioned(
          right: 16,
          top: 12,
          child: IconButton(
            tooltip: spinning.value ? l10n.studioPause : l10n.studioRotate,
            onPressed: () => spinning.value = !spinning.value,
            icon: Icon(
              spinning.value
                  ? Icons.pause_outlined
                  : Icons.rotate_right_outlined,
            ),
          ),
        ),
        Positioned(
          left: 24,
          right: 24,
          bottom: 24,
          child: IgnorePointer(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(genome.value.name, style: Veil.display(34)),
                const SizedBox(height: 10),
                Text(
                  l10n.studioDrag,
                  style: Veil.body(color: Veil.ash, size: 13),
                ),
              ],
            ),
          ),
        ),
      ],
    );

    final controls = Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(l10n.studioTitle, style: Veil.display(38)),
          const SizedBox(height: 12),
          Text(l10n.studioIntro, style: Veil.body(color: Veil.ash)),
          const SizedBox(height: 20),
          SegmentedButton<int>(
            direction: MediaQuery.textScalerOf(context).scale(14) > 20
                ? Axis.vertical
                : Axis.horizontal,
            segments: [
              ButtonSegment(value: 0, label: Text(l10n.studioTraits)),
              ButtonSegment(value: 1, label: Text(l10n.studioGrowth)),
              ButtonSegment(value: 2, label: Text(l10n.soulTab)),
            ],
            selected: {tab.value},
            onSelectionChanged: (values) => tab.value = values.single,
          ),
          if (tab.value == 0) ...[
            LedgerHeading(l10n.studioClass),
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: [
                for (final value in ReaperClass.values)
                  ChoiceChip(
                    label: Text(className(l10n, value)),
                    selected: genome.value.reaperClass == value,
                    onSelected: (_) => genome.value = genome.value.copyWith(
                      reaperClass: value,
                    ),
                  ),
              ],
            ),
            const SizedBox(height: 14),
            LedgerButton(
              label: l10n.soulBirthLink(
                giftName(l10n, soul.gift),
                temperamentName(l10n, soul.temperament),
              ),
              detail: l10n.soulTab,
              quiet: true,
              onPressed: () => tab.value = 2,
            ),
            _Variants(
              label: l10n.studioHead,
              count: ReaperGenome.headVariants,
              value: genome.value.head,
              onChanged: (value) =>
                  genome.value = genome.value.copyWith(head: value),
            ),
            _Variants(
              label: l10n.studioMantle,
              count: ReaperGenome.mantleVariants,
              value: genome.value.mantle,
              onChanged: (value) =>
                  genome.value = genome.value.copyWith(mantle: value),
            ),
            _Variants(
              label: l10n.studioRobe,
              count: ReaperGenome.robeVariants,
              value: genome.value.robe,
              onChanged: (value) =>
                  genome.value = genome.value.copyWith(robe: value),
            ),
            _Variants(
              label: l10n.studioPalette,
              count: ReaperGenome.paletteVariants,
              value: genome.value.palette,
              onChanged: (value) =>
                  genome.value = genome.value.copyWith(palette: value),
            ),
            LedgerHeading(l10n.studioOmen),
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: [
                for (final value in Omen.values)
                  ChoiceChip(
                    label: Text(omenName(l10n, value)),
                    selected: genome.value.omen == value,
                    onSelected: (_) =>
                        genome.value = genome.value.copyWith(omen: value),
                  ),
              ],
            ),
            LedgerHeading(l10n.studioAura),
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: [
                for (final value in Aura.values)
                  ChoiceChip(
                    label: Text(auraName(l10n, value)),
                    selected: genome.value.aura == value,
                    onSelected: (_) =>
                        genome.value = genome.value.copyWith(aura: value),
                  ),
              ],
            ),
            _Trait(
              label: l10n.studioStature,
              value: genome.value.stature,
              onChanged: (value) =>
                  genome.value = genome.value.copyWith(stature: value),
            ),
            _Trait(
              label: l10n.studioGauntness,
              value: genome.value.gauntness,
              onChanged: (value) =>
                  genome.value = genome.value.copyWith(gauntness: value),
            ),
            _Trait(
              label: l10n.studioGaze,
              value: genome.value.gaze,
              onChanged: (value) =>
                  genome.value = genome.value.copyWith(gaze: value),
            ),
            _Trait(
              label: l10n.studioTatter,
              value: genome.value.tatter,
              onChanged: (value) =>
                  genome.value = genome.value.copyWith(tatter: value),
            ),
            const SizedBox(height: 18),
            LedgerButton(
              label: l10n.studioShuffle,
              onPressed: () =>
                  genome.value = ReaperGenome.shuffle(++seed.value),
            ),
            const SizedBox(height: 12),
            Text(
              l10n.studioBirthTraits,
              style: Veil.body(color: Veil.ash, size: 13),
            ),
            TextButton.icon(
              icon: const Icon(Icons.copy_outlined, size: 16),
              label: Text(l10n.studioCopy),
              onPressed: () async {
                await Clipboard.setData(ClipboardData(text: genome.value.code));
                if (context.mounted) {
                  ScaffoldMessenger.of(context)
                      .showSnackBar(SnackBar(content: Text(l10n.studioCopied)));
                }
              },
            ),
          ] else if (tab.value == 1) ...[
            LedgerHeading(
              l10n.studioLevel,
              trailing: '$level / ${Progression.levelCap}',
            ),
            Slider(
              key: const Key('studio-level'),
              value: level.toDouble(),
              min: 1,
              max: Progression.levelCap.toDouble(),
              divisions: Progression.levelCap - 1,
              label: l10n.studioLevelValue(level),
              onChanged: (value) => setLevel(value.round()),
            ),
            LedgerMeter(
              fraction: Progression.levelProgress(xp.value),
              colour: light,
              label: level == Progression.levelCap
                  ? l10n.studioCapped
                  : l10n.studioXp,
            ),
            const SizedBox(height: 12),
            LedgerRow(
              label: l10n.studioStrength,
              value: l10n.studioStronger(
                ((stats.strikeDamage / starting.strikeDamage - 1) * 100)
                    .round(),
              ),
              emphasised: true,
            ),
            LedgerRow(
              label: l10n.studioDamage,
              value: stats.strikeDamage.toStringAsFixed(1),
            ),
            const SizedBox(height: 12),
            Text(
              l10n.studioGrowthLine,
              style: Veil.body(color: Veil.ash, size: 13),
            ),
            const SizedBox(height: 14),
            LedgerButton(
              label: giftName(l10n, soul.gift),
              detail: l10n.soulRank(soulRank, SoulRules.maxRank),
              quiet: true,
              onPressed: () => tab.value = 2,
            ),
            const SizedBox(height: 18),
            LedgerButton(
              key: const Key('studio-absorb'),
              label: l10n.studioAbsorb,
              detail: l10n.studioCaptures(captures.value),
              onPressed: () => absorb(),
            ),
            if (reward.value case final earned?)
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: Semantics(
                  liveRegion: true,
                  child: Text(
                    l10n.studioReward(earned),
                    style: Veil.body(color: light, size: 13),
                  ),
                ),
              ),
            const SizedBox(height: 8),
            Text(
              l10n.studioExample,
              style: Veil.body(color: Veil.ash, size: 13),
            ),
            LedgerHeading(l10n.studioWeapon),
            for (final value in Temper.values)
              _Upgrade(
                name: temperName(l10n, value),
                level: value.levelRequired,
                price: value.price,
                currentLevel: level,
                selected: value == temper.value,
                onTap: () => temper.value = value,
              ),
            LedgerHeading(l10n.studioOutfit),
            for (final value in Outfit.values)
              _Upgrade(
                name: outfitName(l10n, value),
                level: value.levelRequired,
                price: value.price,
                currentLevel: level,
                selected: value == outfit.value,
                onTap: () => outfit.value = value,
              ),
            const SizedBox(height: 12),
            Text(
              l10n.studioCosmetic,
              style: Veil.body(color: Veil.ash, size: 13),
            ),
            const SizedBox(height: 18),
            LedgerButton(
              label: l10n.studioPotential,
              onPressed: () {
                setLevel(Progression.levelCap);
                temper.value = Temper.hollowed;
                outfit.value = Outfit.hollowed;
                memory.value = SoulMemory(
                  unseen: SoulRules.rankMarks.last,
                  stillness: SoulRules.rankMarks.last,
                  rhythm: SoulRules.rankMarks.last,
                );
              },
            ),
            const SizedBox(height: 12),
            Text(
              l10n.studioMultiple,
              style: Veil.body(color: Veil.ash, size: 13),
            ),
          ] else ...[
            const SizedBox(height: 20),
            Text(l10n.soulIntro, style: Veil.body(color: Veil.ash)),
            LedgerHeading(
              l10n.studioLevel,
              trailing: '$level / ${Progression.levelCap}',
            ),
            Slider(
              key: const Key('studio-soul-level'),
              value: level.toDouble(),
              min: 1,
              max: Progression.levelCap.toDouble(),
              divisions: Progression.levelCap - 1,
              label: l10n.studioLevelValue(level),
              onChanged: (value) => setLevel(value.round()),
            ),
            LedgerHeading(l10n.soulGiftChoice),
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: [
                for (final gift in ReaperGift.values.where(
                  (gift) => gift.reaperClass == genome.value.reaperClass,
                ))
                  ChoiceChip(
                    key: Key('studio-gift-${gift.name}'),
                    label: Text(giftName(l10n, gift)),
                    selected: soul.gift == gift,
                    onSelected: (_) => tryGift(gift),
                  ),
              ],
            ),
            SoulLedger(
              genome: genome.value,
              level: level,
              memory: memory.value,
            ),
            const SizedBox(height: 20),
            Text(
              l10n.soulPreviewHelp,
              style: Veil.body(color: Veil.ash, size: 13),
            ),
            const SizedBox(height: 12),
            for (final path in SoulPath.values) ...[
              LedgerButton(
                key: Key('studio-memory-${path.name}'),
                label: l10n.soulPreviewMark(soulPathName(l10n, path)),
                detail: '${memory.value.marks(path)}',
                onPressed: () => absorb(path: path),
              ),
              const SizedBox(height: 8),
            ],
            TextButton(
              onPressed: () => memory.value = const SoulMemory(),
              child: Text(l10n.soulPreviewReset),
            ),
          ],
        ],
      ),
    );

    return Theme(
      data: Veil.theme(light),
      child: Scaffold(
        appBar: AppBar(
          title: Text(l10n.studioAppTitle, style: Veil.display(22)),
        ),
        body: SafeArea(
          child: LayoutBuilder(
            builder: (context, constraints) {
              if (constraints.maxWidth >= 720) {
                return Center(
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 1400),
                    child: Row(
                      children: [
                        Expanded(child: portrait),
                        const VerticalDivider(width: 1),
                        SizedBox(
                          width: constraints.maxWidth >= 1024 ? 430 : 340,
                          child: SingleChildScrollView(child: controls),
                        ),
                      ],
                    ),
                  ),
                );
              }
              return SingleChildScrollView(
                child: Column(
                  children: [
                    SizedBox(height: 390, child: portrait),
                    ConstrainedBox(
                      constraints: const BoxConstraints(
                        maxWidth: Veil.contentWidth,
                      ),
                      child: controls,
                    ),
                  ],
                ),
              );
            },
          ),
        ),
      ),
    );
  }
}

class _Variants extends HookWidget {
  const _Variants({
    required this.label,
    required this.count,
    required this.value,
    required this.onChanged,
  });
  final String label;
  final int count;
  final int value;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      LedgerHeading(label),
      Wrap(
        spacing: 8,
        children: [
          for (var index = 0; index < count; index++)
            ChoiceChip(
              label: Semantics(
                label: '$label ${context.l10n.studioVariant(index + 1)}',
                excludeSemantics: true,
                child: Text(context.l10n.studioVariant(index + 1)),
              ),
              selected: index == value,
              onSelected: (_) => onChanged(index),
            ),
        ],
      ),
    ],
  );
}

class _Trait extends HookWidget {
  const _Trait({
    required this.label,
    required this.value,
    required this.onChanged,
  });
  final String label;
  final int value;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    final draft = useState(value.toDouble());
    useEffect(() {
      draft.value = value.toDouble();
      return null;
    }, [value]);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        LedgerHeading(label),
        MergeSemantics(
          child: Semantics(
            label: label,
            child: Slider(
              value: draft.value,
              max: ReaperGenome.traitScale.toDouble(),
              divisions: 20,
              label: '${(draft.value / 10).round()}%',
              onChanged: (value) => draft.value = value,
              onChangeEnd: (value) => onChanged(value.round()),
            ),
          ),
        ),
      ],
    );
  }
}

class _Upgrade extends HookWidget {
  const _Upgrade({
    required this.name,
    required this.level,
    required this.price,
    required this.currentLevel,
    required this.selected,
    required this.onTap,
  });
  final String name;
  final int level;
  final int price;
  final int currentLevel;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => ListTile(
    selected: selected,
    enabled: currentLevel >= level,
    contentPadding: EdgeInsets.zero,
    leading: Icon(
      selected
          ? Icons.check_circle_outline
          : currentLevel >= level
          ? Icons.circle_outlined
          : Icons.lock_outline,
    ),
    title: Text(name),
    onTap: currentLevel >= level ? onTap : null,
    subtitle: Text(
      context.l10n.studioUnlock(level, price),
      style: Veil.body(color: Veil.ash, size: 12),
    ),
  );
}
