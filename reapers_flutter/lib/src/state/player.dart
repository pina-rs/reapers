import 'package:reapers_shared/reapers_shared.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../backend/game_backend.dart';
import '../backend/local_backend.dart';
import '../backend/rift_session.dart';
import '../backend/server_backend.dart';

part 'player.g.dart';

/// The preferences store, opened once before the app starts and handed in
/// through an override.
@Riverpod(keepAlive: true)
SharedPreferences sharedPreferences(Ref ref) =>
    throw UnimplementedError('Override sharedPreferencesProvider at startup');

/// Where the Reapers server is, set at build time with
/// `--dart-define=REAPERS_SERVER_URL=https://…/`. A build without it is the
/// fully offline game.
const reapersServerUrl = String.fromEnvironment('REAPERS_SERVER_URL');

/// Whoever keeps this player's books: the server when the build names one,
/// and the phone itself otherwise.
@Riverpod(keepAlive: true)
GameBackend gameBackend(Ref ref) {
  if (reapersServerUrl.isEmpty) {
    return LocalBackend(ref.watch(sharedPreferencesProvider));
  }
  final client = reapersClient(reapersServerUrl);
  ref.onDispose(client.close);
  return ServerBackend(client);
}

/// The player: what they own and what they have done.
///
/// Every screen reads the player from here, and every change goes through
/// one of these methods, which asks the keeper and then shows what it says.
/// A refusal leaves the player as they were and rethrows the [RuleBreach]
/// for the screen to explain.
@Riverpod(keepAlive: true)
class Player extends _$Player {
  @override
  Future<PlayerState> build() => ref.watch(gameBackendProvider).load();

  GameBackend get _backend => ref.read(gameBackendProvider);

  Future<void> takeOath() => _apply(_backend.takeOath);

  Future<void> selectReaper(String id) =>
      _apply(() => _backend.selectReaper(id));

  Future<void> acquire(Nft nft) => _apply(() => _backend.acquire(nft));

  Future<void> release(Nft nft) => _apply(() => _backend.release(nft));

  Future<void> buyWeapon(Weapon weapon) =>
      _apply(() => _backend.buyWeapon(weapon));

  Future<void> wield(Weapon weapon) => _apply(() => _backend.wield(weapon));

  Future<void> buyOutfit(Outfit outfit) =>
      _apply(() => _backend.buyOutfit(outfit));

  Future<void> wear(Outfit outfit) => _apply(() => _backend.wear(outfit));

  /// Hands a finished hunt to the keeper and returns its verdict.
  Future<HuntSettlement> settleHunt(int sightingId, HuntLog log) async {
    final settlement = await _backend.settleHunt(sightingId, log);

    state = AsyncData(settlement.state);

    return settlement;
  }

  /// Seats the player in the rift at [site].
  Future<RiftSession> enterRift(RiftSite site) => _backend.enterRift(site);

  /// Reads the player again from the keeper, after something that changed
  /// them without going through this notifier: a sealed rift pays its
  /// reapers on the keeper's side.
  Future<void> refresh() => _apply(_backend.load);

  Future<void> _apply(Future<PlayerState> Function() request) async {
    state = AsyncData(await request());
  }
}
