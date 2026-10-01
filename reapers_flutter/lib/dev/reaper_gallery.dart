import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../render/reaper_stage.dart';

/// A developer's bench for looking at every reaper the shuffle can draw.
///
/// It is not part of the game: it exists so the models, colourways and
/// outfits can be reviewed together. Its labels are English on purpose. A
/// debug build started with `--dart-define=REAPERS_GALLERY=true` opens here
/// instead of at the game.
class ReaperGallery extends HookWidget {
  const ReaperGallery({super.key, this.initialSeed = 1});

  final int initialSeed;

  /// The next seed past [after] that the shuffle turns into a reaper of
  /// [reaperClass].
  static int _nextOf(ReaperClass reaperClass, {required int after}) {
    var seed = after + 1;

    while (ReaperGenome.shuffle(seed).reaperClass != reaperClass) {
      seed++;
    }

    return seed;
  }

  @override
  Widget build(BuildContext context) {
    final seed = useState(initialSeed);
    final outfit = useState(Outfit.shrouded);
    final genome = useMemoized(
      () => ReaperGenome.shuffle(seed.value),
      [seed.value],
    );

    return Scaffold(
      backgroundColor: const Color(0xFF07080B),
      body: Stack(
        fit: StackFit.expand,
        children: [
          ReaperStage(
            genome: genome,
            weapon: Weapon(genome.reaperClass.weapon, Temper.shrouded),
            outfit: outfit.value,
            turnsOnItsOwn: true,
          ),
          SafeArea(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    genome.name,
                    key: const Key('gallery-name'),
                    style: const TextStyle(color: Colors.white, fontSize: 20),
                  ),
                  Text(
                    '${genome.code}\n${outfit.value.name}',
                    style: const TextStyle(color: Colors.white54, fontSize: 11),
                  ),
                  const Spacer(),
                  Wrap(
                    spacing: 8,
                    children: [
                      FilledButton(
                        key: const Key('gallery-shuffle'),
                        onPressed: () => seed.value++,
                        child: const Text('Shuffle'),
                      ),
                      for (final option in ReaperClass.values)
                        TextButton(
                          key: Key('gallery-class-${option.name}'),
                          onPressed: () =>
                              seed.value = _nextOf(option, after: seed.value),
                          child: Text(option.name),
                        ),
                      for (final option in Outfit.values)
                        OutlinedButton(
                          key: Key('gallery-outfit-${option.name}'),
                          onPressed: () => outfit.value = option,
                          child: Text(option.name),
                        ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
