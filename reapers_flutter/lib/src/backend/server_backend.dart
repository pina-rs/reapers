import 'dart:async';
import 'dart:typed_data';

import 'package:reapers_client/reapers_client.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod_auth_idp_flutter/serverpod_auth_idp_flutter.dart';
import 'package:serverpod_flutter/serverpod_flutter.dart';

import 'game_backend.dart';
import 'rift_session.dart';
import '../../solana/reaper_wallet.dart';

/// Keeps a player's books on the Reapers server.
///
/// The server applies the same shared rules the offline keeper does, with
/// its own seeds and its own clock, and replays every hunt before paying
/// for it. This class only carries requests there and turns the server's
/// answers back into the shared types.
final class ServerBackend implements GameBackend, DemoRiftBackend {
  ServerBackend(this._client);

  final Client _client;
  Future<void>? _signedIn;

  @override
  Future<PlayerState> load() => _ask(_client.player.load);

  @override
  Future<void> locate(GeoFix? fix) =>
      _ask(() => _client.player.updateLocation(fix));

  @override
  Future<PlayerState> takeOath() => _ask(_client.player.takeOath);

  @override
  Future<PlayerState> selectReaper(String id) =>
      _ask(() => _client.player.selectReaper(id));

  Future<ChainStatus> chainStatus() => _ask(_client.solana.status);

  Future<PlayerState> importReaper(String asset) =>
      _ask(() => _client.solana.importReaper(asset));

  Future<void> connectWallet(ChainStatus status) async {
    await _ask(() async {});
    final proof = await proveReaperWallet(
      status: status,
      challenge: (address) => _ask(() => _client.solana.challenge(address)),
    );
    final result = await _ask(
      () => _client.solana.login(proof.challengeId, proof.signature),
    );
    await _client.auth.updateSignedInUser(result);
  }

  @override
  Future<PlayerState> acquire(Nft nft) =>
      _ask(() => _client.player.acquire(nft.id));

  @override
  Future<PlayerState> release(Nft nft) =>
      _ask(() => _client.player.release(nft.id));

  @override
  Future<HuntSettlement> settleHunt(int sightingId, HuntLog log) => _ask(
    () => _client.player.settleHunt(
      sightingId,
      ByteData.sublistView(log.encode()),
    ),
  );

  @override
  Future<PlayerState> buyWeapon(Weapon weapon) =>
      _ask(() => _client.player.buyWeapon(weapon.id));

  @override
  Future<PlayerState> wield(Weapon weapon) =>
      _ask(() => _client.player.wield(weapon.id));

  @override
  Future<PlayerState> buyOutfit(Outfit outfit) =>
      _ask(() => _client.player.buyOutfit(outfit.name));

  @override
  Future<PlayerState> wear(Outfit outfit) =>
      _ask(() => _client.player.wear(outfit.name));

  @override
  Future<PlayerState> attune(Art art) =>
      _ask(() => _client.player.attune(art.name));

  @override
  Future<RiftSession> enterRift(RiftSite site) => _enterRift(site, demo: false);

  /// Seats the player in the development demo circle. The server refuses
  /// this outside development and test.
  @override
  Future<RiftSession> enterDemoRift(RiftSite site) =>
      _enterRift(site, demo: true);

  Future<RiftSession> _enterRift(RiftSite site, {required bool demo}) async {
    // Signing in is a request of its own, so a refusal to sign in is told
    // apart from a refusal to enter.
    await _ask(() async {});

    final reports = StreamController<RiftReport>();
    final frames = demo
        ? _client.rift.enterDemo(site.name, reports.stream)
        : _client.rift.enter(site.name, reports.stream);

    return _ServerRiftSession(
      reports: reports,
      frames: frames
          .handleError(
            (Object error) => throw RuleBreach(
              _refusal((error as Refused).refusal),
            ),
            test: (error) => error is Refused,
          )
          .handleError(
            (Object error) => throw BackendUnavailable('$error'),
            test: (error) => error is ServerpodClientException,
          ),
    );
  }

  /// Sends one request, signing in as a guest first when needed.
  Future<T> _ask<T>(Future<T> Function() request) async {
    try {
      await (_signedIn ??= _signIn());

      return await request();
    } on Refused catch (refused) {
      throw RuleBreach(_refusal(refused.refusal));
    } on BackendUnavailable {
      _signedIn = null;
      rethrow;
    } on ServerpodClientException catch (error) {
      // A failed sign-in must be tried again on the next request.
      _signedIn = null;

      throw BackendUnavailable('$error');
    } on TimeoutException catch (error) {
      _signedIn = null;
      throw BackendUnavailable('$error');
    }
  }

  /// Restores the stored session, or opens a guest account on first launch.
  ///
  /// A guest account is what a wallet will later be linked to; until then a
  /// player is whoever holds this install's session.
  Future<void> _signIn() async {
    if (!await _client.auth.initialize(timeout: _client.connectionTimeout)) {
      // Keep the persisted guest session on a transient outage. Creating
      // another guest here would strand this install's existing progress.
      throw const BackendUnavailable('Could not validate the stored session');
    }

    if (_client.auth.isAuthenticated) {
      return;
    }

    await _client.auth.updateSignedInUser(await _client.anonymousIdp.login());
  }

  /// The server names a refusal by the shared enum's name. A name this
  /// build does not know means the server is newer than the game.
  static Refusal _refusal(String name) =>
      Refusal.values.asNameMap()[name] ??
      (throw BackendUnavailable('Unknown refusal "$name" from a newer server'));
}

/// A rift hosted by the server: frames come down one stream and this
/// reaper's reports go up another.
final class _ServerRiftSession implements RiftSession {
  _ServerRiftSession({required this.frames, required this._reports});

  final StreamController<RiftReport> _reports;

  @override
  final Stream<RiftFrame> frames;

  @override
  void report(RiftReport report) {
    if (!_reports.isClosed) {
      _reports.add(report);
    }
  }

  /// Closing the reports is what tells the server this reaper has gone:
  /// whoever was listening to [frames] cancels, and the call ends.
  @override
  void leave() => unawaited(_reports.close());
}

/// A client for the server at [url], with sign-in sessions kept on the
/// device.
Client reapersClient(String url) =>
    Client(url, connectionTimeout: const Duration(seconds: 10))
      ..connectivityMonitor = FlutterConnectivityMonitor()
      ..authSessionManager = FlutterAuthSessionManager();
