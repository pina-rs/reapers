import 'package:flutter/foundation.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../../simnet/simnet_backend.dart';
import '../../simnet/simnet_state.dart';
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

/// Whether this build offers the development demo rift, set with
/// `--dart-define=REAPERS_DEMO_RIFT=true`. A release build ignores it.
///
/// The demo needs a server because the server coordinates the shared enemy,
/// and it cannot be combined with the offline simnet.
const reapersDemoRift =
    !kReleaseMode && bool.fromEnvironment('REAPERS_DEMO_RIFT');

/// What the screens read for [reapersDemoRift], so a test can turn the demo
/// on. The launch itself is still validated against the constants when the
/// books are chosen.
@Riverpod(keepAlive: true)
bool demoRift(Ref ref) => reapersDemoRift;

/// Whoever keeps this player's books: the server when the build names one,
/// the phone posting to its simnet in a rehearsal build, and the phone
/// itself otherwise.
@Riverpod(keepAlive: true)
GameBackend gameBackend(Ref ref) {
  if (reapersDemoRift) {
    if (reapersSimnet) {
      throw StateError(
        'REAPERS_DEMO_RIFT needs a server: leave REAPERS_SIMNET out.',
      );
    }

    if (reapersServerUrl.isEmpty) {
      throw StateError(
        'REAPERS_DEMO_RIFT needs REAPERS_SERVER_URL pointing at a '
        'development server.',
      );
    }
  }

  if (reapersSurfpoolUrl.isNotEmpty && !reapersSimnet) {
    throw StateError(
      'REAPERS_SURFPOOL_URL belongs to the simnet: '
      'add --dart-define=REAPERS_SIMNET=true.',
    );
  }

  if (ref.watch(simnetChainProvider) case final chain?) {
    if (reapersServerUrl.isNotEmpty) {
      throw StateError(
        'REAPERS_SIMNET plays offline: build it without REAPERS_SERVER_URL.',
      );
    }

    return SimnetBackend(ref.watch(sharedPreferencesProvider), chain);
  }

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

  Future<void> attune(Art art) => _apply(() => _backend.attune(art));

  /// Hands a finished hunt to the keeper and returns its verdict.
  Future<HuntSettlement> settleHunt(int sightingId, HuntLog log) async {
    final settlement = await _backend.settleHunt(sightingId, log);

    state = AsyncData(settlement.state);

    return settlement;
  }

  /// Seats the player in the rift at [site].
  Future<RiftSession> enterRift(RiftSite site) => _backend.enterRift(site);

  /// Seats the player in the development demo circle at [site], where
  /// attunement is waived and every demo client on the server shares one
  /// enemy.
  Future<RiftSession> enterDemoRift(RiftSite site) async {
    if (_backend case final DemoRiftBackend backend) {
      return backend.enterDemoRift(site);
    }

    throw const BackendUnavailable('This build has no demo rift');
  }

  /// Reads the player again from the keeper, after something that changed
  /// them without going through this notifier: a sealed rift pays its
  /// reapers on the keeper's side.
  Future<void> refresh() => _apply(_backend.load);

  Future<void> _apply(Future<PlayerState> Function() request) async {
    state = AsyncData(await request());
  }
}
