import 'dart:convert';
import 'dart:math';

import 'package:reapers_shared/reapers_shared.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'game_backend.dart';
import 'rift_session.dart';

/// Keeps a player's books on the phone itself, with no server.
///
/// The rules are the shared ones, so nothing plays differently offline. The
/// difference is trust: this keeper draws its own seeds and reads its own
/// clock, which is fine for one player's own game and is exactly what the
/// server exists to take away.
final class LocalBackend implements GameBackend {
  @override
  Future<void> locate(GeoFix? fix) async {}

  LocalBackend(
    this._preferences, {
    DateTime Function()? clock,
    Random? random,
  }) : _clock = clock ?? (() => DateTime.now().toUtc()),
       _random = random ?? Random.secure();

  /// Where the player is stored. The name carries the schema's generation,
  /// so a future incompatible format starts fresh instead of failing to read.
  static const storageKey = 'reapers.player.v2';

  final SharedPreferences _preferences;
  final DateTime Function() _clock;
  final Random _random;

  @override
  Future<PlayerState> load() async {
    final stored = _preferences.getString(storageKey);

    if (stored == null) {
      return PlayerState.initial();
    }

    return PlayerState.fromJson(
      (jsonDecode(stored) as Map).cast<String, dynamic>(),
    );
  }

  @override
  Future<PlayerState> takeOath() =>
      _change((state) => state.takeOath(seed: _seed()));

  @override
  Future<PlayerState> acquire(Nft nft) =>
      _change((state) => state.acquire(nft));

  @override
  Future<PlayerState> release(Nft nft) =>
      _change((state) => state.release(nft, seed: _seed(), now: _clock()));

  @override
  Future<HuntSettlement> settleHunt(int sightingId, HuntLog log) async {
    final settlement = (await load()).judgeHunt(
      sightingId,
      log,
      now: _clock(),
      feedingSeed: _seed(),
    );

    await _save(settlement.state);

    return settlement;
  }

  @override
  Future<PlayerState> buyWeapon(Weapon weapon) =>
      _change((state) => state.buyWeapon(weapon));

  @override
  Future<PlayerState> wield(Weapon weapon) =>
      _change((state) => state.wield(weapon));

  @override
  Future<PlayerState> buyOutfit(Outfit outfit) =>
      _change((state) => state.buyOutfit(outfit));

  @override
  Future<PlayerState> wear(Outfit outfit) =>
      _change((state) => state.wear(outfit));

  @override
  Future<RiftSession> enterRift(RiftSite site) async {
    final entrant = (await load()).entrantFor(site);
    final room = RiftRoom(site: site, seed: _seed());
    final opened = _clock();

    return LocalRiftSession(
      room: room,
      seat: room.join(entrant),
      microseconds: () => _clock().difference(opened).inMicroseconds,
      onSealed: (reward) => _change(
        (state) => state.settleRift(xp: reward.xp, anima: reward.anima),
      ),
    );
  }

  Future<PlayerState> _change(PlayerState Function(PlayerState) apply) async {
    final next = apply(await load());

    await _save(next);

    return next;
  }

  Future<void> _save(PlayerState state) =>
      _preferences.setString(storageKey, jsonEncode(state.toJson()));

  int _seed() => _random.nextInt(1 << 32);
}
