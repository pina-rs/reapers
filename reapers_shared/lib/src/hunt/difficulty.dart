/// How forgiving a hunt is.
///
/// [normal] is the game. The others exist for testing: [easy] so that
/// someone who has never hunted can bring a husk down and see the whole of a
/// hunt, and [hard] to see how it holds up when pushed. A hunt records the
/// difficulty it was played at, and the server takes anything but [normal]
/// only outside production.
///
/// A difficulty changes how much a hunt asks of the player and nothing about
/// what it pays.
enum Difficulty {
  easy(
    ward: 0.35,
    reach: 1.75,
    flinch: 3,
    window: 2.5,
    tolerance: 2,
    resolve: 3,
    reaps: 2,
    pace: 0.6,
    patience: 2,
    tell: 1.6,
    cut: 0.6,
  ),
  normal(
    ward: 1,
    reach: 1,
    flinch: 1,
    window: 1,
    tolerance: 1,
    resolve: 0,
    reaps: 0,
    pace: 1,
    patience: 1,
    tell: 1,
    cut: 1,
  ),
  hard(
    ward: 1.3,
    reach: 0.85,
    flinch: 0.8,
    window: 0.8,
    tolerance: 0.8,
    resolve: 0,
    reaps: 0,
    pace: 1.15,
    patience: 1,
    tell: 0.85,
    cut: 1.15,
  );

  const Difficulty({
    required this.ward,
    required this.reach,
    required this.flinch,
    required this.window,
    required this.tolerance,
    required this.resolve,
    required this.reaps,
    required this.pace,
    required this.patience,
    required this.tell,
    required this.cut,
  });

  /// The share of its ward a husk has.
  final double ward;

  /// How much wider or narrower the reach of a strike is: how far from a
  /// seal a tap may land and still strike it.
  final double reach;

  /// How much faster the phone may move during a shriek before it counts as
  /// a flinch.
  final double flinch;

  /// How much longer or shorter the reap window is.
  final double window;

  /// How much farther the cut may stray from the reap arc.
  final double tolerance;

  /// Flinches the reaper can take on top of their own.
  final int resolve;

  /// Unfinished reaps the husk allows on top of the usual.
  final int reaps;

  /// How fast the husk gets about, as a share of its usual speed.
  final double pace;

  /// How much longer the fight may run before the husk escapes.
  final double patience;

  /// How much longer or shorter a charging husk takes to reach the reaper:
  /// how long there is to see it coming.
  final double tell;

  /// How hard the phone has to be swung to turn a charge aside, as a share
  /// of the usual.
  final double cut;
}
