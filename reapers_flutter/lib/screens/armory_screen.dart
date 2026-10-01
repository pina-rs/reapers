import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../render/reaper_stage.dart';
import '../src/state/player.dart';
import '../theme/veil.dart';
import '../widgets/attempt.dart';
import '../widgets/ledger.dart';

/// Where anima is spent: weapons that change how a hunt plays, and outfits
/// that only change how the reaper looks.
///
/// Whatever is tapped is tried on at once, on the reaper standing above the
/// list, before anything is bought.
class ArmoryScreen extends HookConsumerWidget {
  const ArmoryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final player = ref.watch(playerProvider).requireValue;
    final genome = player.genome;
    final showOutfits = useState(false);
    final triedWeapon = useState(player.weapon!);
    final triedOutfit = useState(player.outfit);
    final notifier = ref.read(playerProvider.notifier);

    return Scaffold(
      body: Column(
        children: [
          Expanded(
            flex: 9,
            child: Stack(
              fit: StackFit.expand,
              children: [
                ReaperStage(
                  genome: genome,
                  weapon: triedWeapon.value,
                  outfit: triedOutfit.value,
                ),
                SafeArea(
                  bottom: false,
                  child: Padding(
                    padding: const EdgeInsets.fromLTRB(24, 8, 24, 0),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        LedgerBack(
                          onPressed: () => Navigator.of(context).pop(),
                          label: l10n.armoryOverline,
                        ),
                        const Spacer(),
                        Padding(
                          padding: const EdgeInsets.only(top: 16),
                          child: Text(
                            l10n.armoryAnima(player.anima),
                            key: const Key('armory-anima'),
                            style: Veil.figure(size: 14),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
          Expanded(
            flex: 11,
            child: Center(
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: Veil.contentWidth),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 24),
                      child: Row(
                        children: [
                          _Tab(
                            key: const Key('armory-tab-weapons'),
                            label: l10n.armoryWeapons,
                            chosen: !showOutfits.value,
                            onTap: () => showOutfits.value = false,
                          ),
                          const SizedBox(width: 22),
                          _Tab(
                            key: const Key('armory-tab-outfits'),
                            label: l10n.armoryOutfits,
                            chosen: showOutfits.value,
                            onTap: () => showOutfits.value = true,
                          ),
                        ],
                      ),
                    ),
                    const Divider(height: 1),
                    Expanded(
                      child: ListView(
                        padding: const EdgeInsets.fromLTRB(24, 4, 24, 28),
                        children: showOutfits.value
                            ? [
                                Padding(
                                  padding: const EdgeInsets.only(top: 14),
                                  child: Text(
                                    l10n.armoryOutfitNote,
                                    style: Veil.body(color: Veil.ash, size: 13),
                                  ),
                                ),
                                for (final outfit in Outfit.values)
                                  _OutfitRow(
                                    outfit: outfit,
                                    player: player,
                                    tried: triedOutfit.value == outfit,
                                    onTry: () => triedOutfit.value = outfit,
                                    onBuy: () => attempt(
                                      context,
                                      () => notifier.buyOutfit(outfit),
                                    ),
                                    onWear: () => attempt(
                                      context,
                                      () => notifier.wear(outfit),
                                    ),
                                  ),
                              ]
                            : [
                                for (final kind in _weaponOrder(
                                  genome.reaperClass,
                                )) ...[
                                  LedgerHeading(
                                    weaponName(l10n, kind),
                                    trailing: l10n.armoryTempo(
                                      (HuntRules.tickRate /
                                              weaponProfiles[kind]!.swingTicks)
                                          .toStringAsFixed(1),
                                    ),
                                  ),
                                  for (final temper in Temper.values)
                                    _WeaponRow(
                                      weapon: Weapon(kind, temper),
                                      player: player,
                                      tried:
                                          triedWeapon.value ==
                                          Weapon(kind, temper),
                                      onTry: () => triedWeapon.value = Weapon(
                                        kind,
                                        temper,
                                      ),
                                      onBuy: () => attempt(
                                        context,
                                        () => notifier.buyWeapon(
                                          Weapon(kind, temper),
                                        ),
                                      ),
                                      onWield: () => attempt(
                                        context,
                                        () => notifier.wield(
                                          Weapon(kind, temper),
                                        ),
                                      ),
                                    ),
                                ],
                              ],
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// Every kind of weapon, with the one this class was born holding first.
List<WeaponKind> _weaponOrder(ReaperClass reaperClass) => [
  reaperClass.weapon,
  for (final kind in WeaponKind.values)
    if (kind != reaperClass.weapon) kind,
];

class _Tab extends HookWidget {
  const _Tab({
    super.key,
    required this.label,
    required this.chosen,
    required this.onTap,
  });

  final String label;
  final bool chosen;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final light = Theme.of(context).colorScheme.primary;

    return Semantics(
      button: true,
      selected: chosen,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        child: Container(
          constraints: const BoxConstraints(minHeight: 48),
          alignment: Alignment.center,
          decoration: BoxDecoration(
            border: Border(
              bottom: BorderSide(
                color: chosen ? light : Colors.transparent,
                width: 2,
              ),
            ),
          ),
          child: Text(
            label.toUpperCase(),
            style: Veil.label(color: chosen ? Veil.bone : Veil.ash, size: 13),
          ),
        ),
      ),
    );
  }
}

/// One thing the armory sells: tap the row to try it on, tap the action to
/// buy or take it up.
class _ArmoryRow extends HookWidget {
  const _ArmoryRow({
    super.key,
    required this.title,
    required this.detail,
    required this.action,
    required this.tried,
    required this.onTry,
    required this.onAction,
    this.withinMeans = true,
  });

  final String title;
  final String detail;

  /// What the button at the end says.
  final String action;

  /// Whether the reaper above is trying this on right now.
  final bool tried;

  final VoidCallback onTry;

  /// Null when there is nothing to do: already in use, or out of reach.
  final VoidCallback? onAction;

  /// Whether the player has the anima for it. A price they cannot meet is
  /// still tappable, so the refusal can say why, but it is not lit.
  final bool withinMeans;

  @override
  Widget build(BuildContext context) {
    final light = Theme.of(context).colorScheme.primary;

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTry,
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: Row(
          children: [
            LedgerPip(lit: tried, colour: light),
            const SizedBox(width: 14),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(title, style: Veil.body(size: 15)),
                  const SizedBox(height: 2),
                  Text(detail, style: Veil.body(color: Veil.ash, size: 12)),
                ],
              ),
            ),
            Semantics(
              button: true,
              enabled: onAction != null,
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: onAction,
                child: Container(
                  constraints: const BoxConstraints(
                    minHeight: 48,
                    minWidth: 96,
                  ),
                  alignment: Alignment.centerRight,
                  child: Text(
                    action.toUpperCase(),
                    style: Veil.label(
                      color: onAction == null || !withinMeans
                          ? Veil.ash
                          : light,
                      size: 12,
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _WeaponRow extends HookWidget {
  const _WeaponRow({
    required this.weapon,
    required this.player,
    required this.tried,
    required this.onTry,
    required this.onBuy,
    required this.onWield,
  });

  final Weapon weapon;
  final PlayerState player;
  final bool tried;
  final VoidCallback onTry;
  final VoidCallback onBuy;
  final VoidCallback onWield;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final owned = player.weapons.contains(weapon);
    final price = weapon.priceFor(player.genome.reaperClass);
    final reachable = player.level >= weapon.temper.levelRequired;
    final strike = (weapon.profile.strike * weapon.temper.strike)
        .toStringAsFixed(2);

    final (action, onAction) = switch (null) {
      _ when player.weapon == weapon => (l10n.armoryWielded, null),
      _ when owned => (l10n.armoryWield, onWield),
      _ when !reachable => (
        l10n.armoryNeedsLevel(romanNumeral(weapon.temper.levelRequired)),
        null,
      ),
      _ => (price == 0 ? l10n.armoryFree : l10n.armoryAnima(price), onBuy),
    };

    return _ArmoryRow(
      key: Key('armory-weapon-${weapon.id}'),
      title: temperName(l10n, weapon.temper),
      detail: l10n.armoryStrike(strike),
      action: action,
      tried: tried,
      onTry: onTry,
      onAction: onAction,
      withinMeans: owned || player.anima >= price,
    );
  }
}

class _OutfitRow extends HookWidget {
  const _OutfitRow({
    required this.outfit,
    required this.player,
    required this.tried,
    required this.onTry,
    required this.onBuy,
    required this.onWear,
  });

  final Outfit outfit;
  final PlayerState player;
  final bool tried;
  final VoidCallback onTry;
  final VoidCallback onBuy;
  final VoidCallback onWear;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final owned = player.outfits.contains(outfit);
    final reachable = player.level >= outfit.levelRequired;

    final (action, onAction) = switch (null) {
      _ when player.outfit == outfit => (l10n.armoryWorn, null),
      _ when owned => (l10n.armoryWear, onWear),
      _ when !reachable => (
        l10n.armoryNeedsLevel(romanNumeral(outfit.levelRequired)),
        null,
      ),
      _ => (l10n.armoryAnima(outfit.price), onBuy),
    };

    return Padding(
      padding: const EdgeInsets.only(top: 8),
      child: _ArmoryRow(
        key: Key('armory-outfit-${outfit.name}'),
        title: outfitName(l10n, outfit),
        detail: outfitLine(l10n, outfit),
        action: action,
        tried: tried,
        onTry: onTry,
        onAction: onAction,
        withinMeans: owned || player.anima >= outfit.price,
      ),
    );
  }
}
