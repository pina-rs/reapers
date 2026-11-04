import 'package:reapers_shared/reapers_shared.dart';

import 'rift_session.dart';

/// Whoever keeps the books for a player.
///
/// Every method returns the player as they stand afterwards, or throws a
/// [RuleBreach] when the rules refuse. There are two keepers: the phone
/// itself, for someone playing with no server, and the Reapers server, which
/// judges every request with the same shared rules.
abstract interface class GameBackend {
  /// The player as they stand now, created on first use.
  Future<PlayerState> load();

  /// Shares the latest foreground fix, or clears it when tracking stops.
  Future<void> locate(GeoFix? fix);

  /// Binds a reaper. The keeper, never the player, draws it.
  Future<PlayerState> takeOath();

  Future<PlayerState> selectReaper(String id);

  /// Buys [nft] into the vault.
  Future<PlayerState> acquire(Nft nft);

  /// Releases [nft] from the vault into the veil, where it becomes a husk.
  Future<PlayerState> release(Nft nft);

  /// Judges the recorded hunt for [sightingId] and settles it.
  Future<HuntSettlement> settleHunt(int sightingId, HuntLog log);

  Future<PlayerState> buyWeapon(Weapon weapon);

  Future<PlayerState> wield(Weapon weapon);

  Future<PlayerState> buyOutfit(Outfit outfit);

  Future<PlayerState> wear(Outfit outfit);

  /// Makes [art] the one the reaper throws.
  Future<PlayerState> attune(Art art);

  /// Takes the player to the rift at [site] and seats them in its circle.
  ///
  /// Throws a [RuleBreach] when the rift does not open for them.
  Future<RiftSession> enterRift(RiftSite site);
}

/// A keeper that can seat a reaper in a development demo rift.
///
/// Only the server has one, and only development and test servers answer it.
/// The demo skips attunement; it changes no shared rule, mixes with no real
/// circle, and is unavailable from a release build.
abstract interface class DemoRiftBackend {
  /// Seats the player in the demo circle at [site].
  Future<RiftSession> enterDemoRift(RiftSite site);
}

/// The keeper could not be reached, or answered with something that is not
/// an answer. A timed-out mutation may have reached the keeper; read the
/// books again before deciding whether to repeat it.
final class BackendUnavailable implements Exception {
  const BackendUnavailable(this.reason);

  /// What went wrong, for a log or a support message. Not shown as-is.
  final String reason;

  @override
  String toString() => 'BackendUnavailable($reason)';
}
