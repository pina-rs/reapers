import '../hunt/hunt_stats.dart';

/// The fixed numbers of a rift: a husk too strong for one reaper, fought by
/// a circle of them at once.
///
/// These are playtest starting points, written down in docs/gameplay.md.
abstract final class RiftRules {
  /// Reapers in a full circle. Seats nobody takes are filled by shades.
  static const circle = 4;

  /// How long a rift waits for reapers before it starts with whoever came.
  static const gatherLimitTicks = 60 * HuntRules.tickRate;

  /// The count before the fight, once every reaper present stands ready.
  static const countdownTicks = 3 * HuntRules.tickRate;

  /// The rift closes, unsealed, after this much fighting.
  static const fightLimitTicks = 120 * HuntRules.tickRate;

  /// How long the circle has to cut once the ward breaks.
  static const reapWindowTicks = 8 * HuntRules.tickRate;

  /// The ward is sized so that a circle playing carefully breaks it in about
  /// this long, whatever its levels are.
  static const breakSeconds = 70;

  /// The share of its best possible damage a careful reaper really lands:
  /// seals are not always open, shrieks stop every blade, and taps go wide.
  static const steadyShare = 0.84;

  /// A shade fights this many levels below the circle's average.
  static const shadeLevelGap = 1;

  /// Absorbed NFTs from a rift's collection needed before it opens for a
  /// reaper.
  static const attunement = 1;

  /// Experience for sealing a rift with a clean cut and a full share of the
  /// work: about what one epic husk is worth. A rift costs no NFT to enter,
  /// so it must not pay better than the hunts that do.
  static const xp = 220;

  /// Anima for the same.
  static const anima = 90;

  /// A reaper who pulled their share of the ward earns the whole reward; one
  /// who did nothing still earns this share of it for standing in the circle.
  static const idleShare = 0.6;

  /// What a reaper who was still standing but made no cut earns, as a share
  /// of a clean cut.
  static const uncutShare = 0.75;

  /// What a reaper who was cast out before the end earns.
  static const castOutShare = 0.5;

  // Where a rift stands -----------------------------------------------

  /// A reaper standing this near a rift's site, in metres, finds the rift
  /// at the site itself.
  static const siteMetres = 250.0;

  /// Anywhere else, the rift stands this far in front of the first reaper
  /// to say where they are. Everyone who comes after sees it in that same
  /// place.
  static const plantMetres = 25.0;

  /// A reaper farther than this from where the rift stands is not there in
  /// any useful sense, and sees it dead ahead, as through the veil.
  static const sightMetres = 150.0;

  /// The nearest and farthest the husk is drawn from a reaper, in metres,
  /// however near or far they stand. Nearer and it would not fit on the
  /// screen; farther and its seals would be too small to strike.
  static const nearestMetres = 14.0;
  static const farthestMetres = 40.0;

  /// The ward a rift's husk raises against [circle], one entry for every
  /// reaper and shade in it.
  ///
  /// A rift is as hard for a circle of beginners as for a circle of veterans:
  /// what it asks for is company and care, not levels.
  static double wardFor(Iterable<HuntStats> circle) {
    var perSecond = 0.0;

    for (final stats in circle) {
      perSecond += stats.strikeDamage * HuntRules.tickRate / stats.swingTicks;
    }

    return perSecond * steadyShare * breakSeconds;
  }
}
