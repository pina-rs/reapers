import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../render/reaper_appearance.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';

/// Independent presentation choices, available without levelling a specimen.
class AppearanceControls extends HookWidget {
  const AppearanceControls({
    super.key,
    required this.appearance,
    required this.weapon,
    required this.onChanged,
    required this.onWeaponChanged,
  });

  final ReaperAppearance appearance;
  final WeaponKind weapon;
  final ValueChanged<ReaperAppearance> onChanged;
  final ValueChanged<WeaponKind> onWeaponChanged;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        LedgerHeading(l10n.studioForm),
        Wrap(
          spacing: 6,
          children: [
            for (final value in ReaperForm.values)
              ChoiceChip(
                label: Text(
                  value == ReaperForm.spirit
                      ? l10n.studioSpirit
                      : l10n.studioHumanoid,
                ),
                selected: appearance.form == value,
                onSelected: (_) => onChanged(
                  appearance.copyWith(
                    form: value,
                    clothing:
                        value == ReaperForm.humanoid &&
                            appearance.clothing == ReaperClothing.robe
                        ? ReaperClothing.tunic
                        : appearance.clothing,
                  ),
                ),
              ),
          ],
        ),
        Text(l10n.studioFormHelp, style: Veil.body(color: Veil.ash, size: 12)),
        LedgerHeading(l10n.studioClothing),
        Wrap(
          spacing: 6,
          runSpacing: 6,
          children: [
            for (final value in ReaperClothing.values)
              ChoiceChip(
                label: Text(switch (value) {
                  ReaperClothing.robe => l10n.studioLongRobe,
                  ReaperClothing.tunic => l10n.studioTunic,
                  ReaperClothing.unrobed => l10n.studioNoRobe,
                }),
                selected: appearance.clothing == value,
                onSelected: (_) =>
                    onChanged(appearance.copyWith(clothing: value)),
              ),
          ],
        ),
        SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(l10n.studioShowMantle),
          value: appearance.mantle,
          onChanged: (value) => onChanged(appearance.copyWith(mantle: value)),
        ),
        SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(l10n.studioHoldObject),
          subtitle: Text(
            l10n.studioHoldHelp,
            style: Veil.body(color: Veil.ash, size: 12),
          ),
          value: appearance.holding,
          onChanged: (value) => onChanged(appearance.copyWith(holding: value)),
        ),
        if (appearance.holding)
          DropdownButtonFormField<WeaponKind>(
            key: ValueKey(weapon),
            initialValue: weapon,
            isExpanded: true,
            decoration: InputDecoration(labelText: l10n.studioHeldObject),
            items: [
              for (final value in WeaponKind.values)
                DropdownMenuItem(
                  value: value,
                  child: Text(weaponName(l10n, value)),
                ),
            ],
            onChanged: (value) {
              if (value != null) onWeaponChanged(value);
            },
          ),
      ],
    );
  }
}
