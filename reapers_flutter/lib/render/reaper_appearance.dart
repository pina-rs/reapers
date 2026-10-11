import 'package:reapers_shared/reapers_shared.dart';

/// Model presentation only. These choices never alter a genome or player save.
enum ReaperForm { spirit, humanoid }

enum ReaperClothing { robe, tunic, unrobed }

final class ReaperAppearance {
  const ReaperAppearance({
    this.form = ReaperForm.spirit,
    this.clothing = ReaperClothing.robe,
    this.mantle = true,
    this.holding = true,
  });

  final ReaperForm form;
  final ReaperClothing clothing;
  final bool mantle;
  final bool holding;

  /// Each state has a floating performance and a grounded humanoid performance.
  String clipName(ReaperMotion motion) =>
      form == ReaperForm.humanoid ? 'humanoid_${motion.name}' : motion.name;

  ReaperAppearance copyWith({
    ReaperForm? form,
    ReaperClothing? clothing,
    bool? mantle,
    bool? holding,
  }) => ReaperAppearance(
    form: form ?? this.form,
    clothing: clothing ?? this.clothing,
    mantle: mantle ?? this.mantle,
    holding: holding ?? this.holding,
  );
}

/// Identical part visibility for native and browser rigs. Null leaves unrelated
/// nodes alone, including animation targets beneath hidden variant parents.
bool? reaperPartVisible(
  String name,
  ReaperGenome genome,
  Outfit outfit,
  ReaperAppearance appearance,
) {
  if (name == 'humanoid_form') return appearance.form == ReaperForm.humanoid;
  if (name == 'spirit_form') return appearance.form == ReaperForm.spirit;
  for (final (prefix, chosen, enabled) in [
    ('head_', genome.head, true),
    ('mantle_', genome.mantle, appearance.mantle),
    ('robe_', genome.robe, appearance.clothing != ReaperClothing.unrobed),
  ]) {
    if (!name.startsWith(prefix)) continue;
    final index = int.tryParse(name.substring(prefix.length));
    if (index != null) return enabled && index == chosen;
  }
  if (name.contains('_sleeve') || name.endsWith('_cuff')) {
    return appearance.clothing != ReaperClothing.unrobed;
  }
  if (name.startsWith('void_') || name.startsWith('wrap_')) {
    return outfit != Outfit.hollowed;
  }
  return null;
}
