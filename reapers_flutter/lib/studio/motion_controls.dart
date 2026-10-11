import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../l10n/motion_lookups.dart';
import '../render/reaper_playback.dart';
import '../theme/veil.dart';

/// Playback controls for the actual authored clips on the studio specimen.
class MotionControls extends HookWidget {
  const MotionControls({
    super.key,
    required this.playback,
    required this.weapon,
    this.onInspectFace,
  });

  final ReaperPlayback playback;
  final WeaponKind? weapon;
  final VoidCallback? onInspectFace;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final group = useState(MotionGroup.movement);
    final reduced = MediaQuery.disableAnimationsOf(context);
    useListenable(playback);
    final motion = playback.motion;

    return ExpansionTile(
      key: const PageStorageKey('studio-motions'),
      tilePadding: EdgeInsets.zero,
      title: Text(l10n.motionTitle, style: Veil.body()),
      children: [
        Text(l10n.motionIntro, style: Veil.body(color: Veil.ash, size: 13)),
        const SizedBox(height: 12),
        Wrap(
          spacing: 6,
          runSpacing: 6,
          children: [
            for (final value in MotionGroup.values)
              ChoiceChip(
                label: Text(motionGroupName(l10n, value)),
                selected: group.value == value,
                onSelected: (_) => group.value = value,
              ),
          ],
        ),
        const SizedBox(height: 12),
        DropdownButtonFormField<ReaperMotion>(
          key: ValueKey((group.value, motion, weapon)),
          initialValue: motion.group == group.value ? motion : null,
          isExpanded: true,
          decoration: InputDecoration(labelText: l10n.motionSelect),
          items: [
            for (final value in ReaperMotion.values)
              if (value.group == group.value)
                DropdownMenuItem(
                  value: value,
                  enabled: value.supports(weapon),
                  child: Text(
                    motionName(l10n, value),
                    overflow: TextOverflow.ellipsis,
                    style: Veil.body(
                      color: value.supports(weapon) ? Veil.bone : Veil.ash,
                    ),
                  ),
                ),
          ],
          onChanged: (value) {
            if (value != null) playback.select(value, play: !reduced);
          },
        ),
        if (group.value == MotionGroup.finisher)
          Padding(
            padding: const EdgeInsets.only(top: 10),
            child: Text(
              l10n.motionWeaponHint,
              style: Veil.body(color: Veil.ash, size: 12),
            ),
          ),
        const SizedBox(height: 12),
        Text(motionName(l10n, motion), style: Veil.body()),
        Text(
          l10n.motionTiming(
            motion.seconds.toStringAsFixed(1),
            motion.loops ? l10n.motionLoop : l10n.motionOnce,
          ),
          style: Veil.body(color: Veil.ash, size: 12),
        ),
        if (motion.unarmed)
          Text(l10n.motionUnarmed, style: Veil.body(color: Veil.ash, size: 12)),
        Slider(
          key: const Key('motion-timeline'),
          value: playback.progress,
          label: l10n.motionPosition(playback.seconds.toStringAsFixed(2)),
          semanticFormatterCallback: (value) =>
              l10n.motionProgress((value * 100).round()),
          onChanged: playback.seek,
        ),
        Wrap(
          spacing: 8,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: [
            IconButton(
              tooltip: playback.playing && !reduced
                  ? l10n.motionPause
                  : l10n.motionPlay,
              onPressed: reduced ? null : playback.toggle,
              icon: Icon(
                playback.playing && !reduced ? Icons.pause : Icons.play_arrow,
              ),
            ),
            IconButton(
              tooltip: l10n.motionReplay,
              onPressed: reduced ? () => playback.seek(0) : playback.replay,
              icon: const Icon(Icons.replay),
            ),
            if (motion.impact case final impact?)
              TextButton(
                onPressed: () => playback.seek(impact),
                child: Text(l10n.motionImpact),
              ),
            if (onInspectFace != null &&
                (motion.group == MotionGroup.reaction ||
                    motion.group == MotionGroup.meme))
              TextButton(
                onPressed: () {
                  playback.seek(0.5);
                  onInspectFace!();
                },
                child: Text(l10n.studioInspectExpression),
              ),
            Tooltip(
              message: l10n.motionSpeed,
              child: DropdownButton<double>(
                value: playback.speed,
                items: [
                  for (final speed in [0.25, 0.5, 1.0, 1.5])
                    DropdownMenuItem(
                      value: speed,
                      child: Text(l10n.motionRate(speed.toString())),
                    ),
                ],
                onChanged: (value) {
                  if (value != null) playback.setSpeed(value);
                },
              ),
            ),
          ],
        ),
        if (reduced)
          Text(l10n.motionReduced, style: Veil.body(color: Veil.ash, size: 13)),
        const SizedBox(height: 16),
      ],
    );
  }
}
