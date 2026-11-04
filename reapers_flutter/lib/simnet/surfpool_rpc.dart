import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';

/// Why a call to a local Solana RPC did not finish.
enum SurfpoolFailure {
  /// The endpoint could not be reached at all.
  unreachable,

  /// The endpoint did not answer in time.
  timedOut,

  /// The endpoint answered with an HTTP status that is not 200.
  http,

  /// The JSON-RPC response reported an error.
  rpc,

  /// The response was not the JSON-RPC the client asked for.
  malformed,

  /// A sent transaction reported an error instead of confirming.
  transaction,
}

/// A failed call to a local Solana RPC, with the method and enough detail to
/// tell a developer what went wrong. Only the simnet's developer panel shows
/// the detail; ordinary play never depends on this call.
final class SurfpoolRpcException implements Exception {
  const SurfpoolRpcException(this.failure, this.method, this.detail);

  final SurfpoolFailure failure;

  /// The JSON-RPC method that failed, such as `getHealth`.
  final String method;

  /// A short machine-facing explanation, for a log or the developer panel.
  final String detail;

  @override
  String toString() =>
      'SurfpoolRpcException($method, ${failure.name}): $detail';
}

/// What the local validator says about one signature.
typedef SurfpoolSignatureStatus = ({int? slot, bool confirmed, String? error});

/// A real Solana JSON-RPC endpoint, for the Surfpool validator a developer
/// runs beside the game.
///
/// This speaks only standard Solana methods. It never receives the simnet's
/// signed descriptions as if they were Solana transactions, and it is not a
/// way to coordinate players: Surfpool is a validator, not a game server.
final class SurfpoolRpc {
  SurfpoolRpc({
    required Uri endpoint,
    http.Client? client,
    this.timeout = const Duration(seconds: 4),
    this.pollInterval = const Duration(milliseconds: 200),
  }) : endpoint = validate(endpoint),
       _client = client ?? http.Client(),
       _ownsClient = client == null;

  /// Checks an endpoint is one this development build may talk to.
  ///
  /// Cleartext HTTP is allowed on loopback, an emulator's host alias and a
  /// private network, where a Surfpool on the same machine or LAN lives. A
  /// public host must use HTTPS.
  static Uri validate(Uri endpoint) {
    final scheme = endpoint.scheme;
    final host = endpoint.host;

    if ((scheme != 'http' && scheme != 'https') ||
        host.isEmpty ||
        endpoint.userInfo.isNotEmpty ||
        endpoint.hasQuery ||
        endpoint.hasFragment ||
        (scheme == 'http' && !_isLocalNetwork(host))) {
      throw const FormatException(
        'REAPERS_SURFPOOL_URL must be http://localhost:8899, '
        'http://127.0.0.1:8899, an emulator/LAN address, '
        'or an HTTPS Solana RPC URL',
      );
    }

    return endpoint;
  }

  /// The endpoint every call goes to.
  final Uri endpoint;

  /// How long one HTTP request may take.
  final Duration timeout;

  /// How long to wait between signature status checks.
  final Duration pollInterval;

  final http.Client _client;
  final bool _ownsClient;
  int _nextId = 0;

  /// Asks the validator whether it is ready. Throws unless it answers `ok`.
  Future<void> health() async {
    final result = await _call('getHealth');

    if (result != 'ok') {
      throw SurfpoolRpcException(
        SurfpoolFailure.malformed,
        'getHealth',
        'expected "ok", got ${jsonEncode(result)}',
      );
    }
  }

  /// The genesis hash the validator reports, for telling one cluster from
  /// another.
  Future<String> genesisHash() async {
    final result = await _call('getGenesisHash');

    return switch (result) {
      final String hash when hash.isNotEmpty => hash,
      _ => throw SurfpoolRpcException(
        SurfpoolFailure.malformed,
        'getGenesisHash',
        'the result was not a hash',
      ),
    };
  }

  /// The lamport balance of [address], as the validator reports it.
  ///
  /// A balance is a whole, non-negative number of lamports; anything else is
  /// an answer this client does not understand, not one to round or flip.
  Future<int> balance(String address) async {
    _requireAddress(address, 'getBalance');
    final result = await _call('getBalance', [address]);

    return switch (result) {
      {'value': final int value} when value >= 0 => value,
      _ => throw SurfpoolRpcException(
        SurfpoolFailure.malformed,
        'getBalance',
        'the result had no non-negative whole lamports',
      ),
    };
  }

  /// Asks the local faucet for [lamports] to [address] and returns the
  /// signature, without waiting for the transfer to confirm.
  ///
  /// Solana's faucet is not idempotent: when the answer to this call is
  /// lost, whether the faucet paid is unknowable from here. A caller that
  /// must not risk paying twice writes the returned signature down before
  /// calling [awaitConfirmation].
  Future<String> requestAirdrop(
    String address, {
    int lamports = 1_000_000_000,
  }) async {
    _requireAddress(address, 'requestAirdrop');

    if (lamports <= 0) {
      throw ArgumentError.value(lamports, 'lamports', 'must be positive');
    }

    return switch (await _call('requestAirdrop', [address, lamports])) {
      final String value when value.isNotEmpty => value,
      _ => throw SurfpoolRpcException(
        SurfpoolFailure.malformed,
        'requestAirdrop',
        'the result was not a signature',
      ),
    };
  }

  /// One reading of [signature]'s status.
  Future<SurfpoolSignatureStatus?> signatureStatus(String signature) async {
    final result = await _call('getSignatureStatuses', [
      [signature],
      {'searchTransactionHistory': true},
    ]);
    final values = switch (result) {
      {'value': final List<dynamic> values} => values,
      _ => throw SurfpoolRpcException(
        SurfpoolFailure.malformed,
        'getSignatureStatuses',
        'the result had no value list',
      ),
    };
    final first = values.firstOrNull;

    if (first == null) {
      return null;
    }

    return switch (first) {
      {
        'slot': final int slot,
        'err': final Object? error,
        'confirmationStatus': final String status,
      } =>
        (
          slot: slot,
          confirmed: status == 'confirmed' || status == 'finalized',
          error: error == null ? null : jsonEncode(error),
        ),
      _ => throw SurfpoolRpcException(
        SurfpoolFailure.malformed,
        'getSignatureStatuses',
        'the status entry was not a status',
      ),
    };
  }

  /// Waits until [signature] is confirmed, or [timeout] passes.
  Future<SurfpoolSignatureStatus> awaitConfirmation(
    String signature, {
    Duration? timeout,
  }) async {
    final limit = timeout ?? this.timeout;
    final watch = Stopwatch()..start();

    while (true) {
      final status = await signatureStatus(signature);

      if (status?.error case final error?) {
        throw SurfpoolRpcException(
          SurfpoolFailure.transaction,
          'getSignatureStatuses',
          error,
        );
      }

      if (status != null && status.confirmed) {
        return status;
      }

      if (watch.elapsed >= limit) {
        throw SurfpoolRpcException(
          SurfpoolFailure.timedOut,
          'getSignatureStatuses',
          'no confirmed status for $signature',
        );
      }

      await Future<void>.delayed(pollInterval);
    }
  }

  /// Releases the HTTP client this object made, if it made it.
  void close() {
    if (_ownsClient) {
      _client.close();
    }
  }

  Future<Object?> _call(
    String method, [
    List<Object?> params = const [],
  ]) async {
    final id = _nextId++;

    try {
      final response = await _client
          .post(
            endpoint,
            headers: const {'content-type': 'application/json'},
            body: jsonEncode({
              'jsonrpc': '2.0',
              'id': id,
              'method': method,
              'params': params,
            }),
          )
          .timeout(timeout);

      if (response.statusCode != 200) {
        throw SurfpoolRpcException(
          SurfpoolFailure.http,
          method,
          'HTTP ${response.statusCode}',
        );
      }

      final decoded = switch (jsonDecode(response.body)) {
        final Map<String, dynamic> body => body,
        final Map<dynamic, dynamic> body => body.cast<String, dynamic>(),
        _ => throw SurfpoolRpcException(
          SurfpoolFailure.malformed,
          method,
          'the response was not a JSON object',
        ),
      };

      if (decoded['jsonrpc'] != '2.0' || decoded['id'] != id) {
        throw SurfpoolRpcException(
          SurfpoolFailure.malformed,
          method,
          'the response did not match this request',
        );
      }

      if (decoded['error'] case final Map<dynamic, dynamic> error) {
        final message = error['message'];
        final code = error['code'];
        throw SurfpoolRpcException(
          SurfpoolFailure.rpc,
          method,
          [
            if (code != null) 'code $code',
            if (message != null) '$message',
          ].join(': '),
        );
      }

      if (!decoded.containsKey('result')) {
        throw SurfpoolRpcException(
          SurfpoolFailure.malformed,
          method,
          'the response had neither a result nor an error',
        );
      }

      return decoded['result'];
    } on SurfpoolRpcException {
      rethrow;
    } on TimeoutException {
      throw SurfpoolRpcException(
        SurfpoolFailure.timedOut,
        method,
        'no answer within ${timeout.inMilliseconds} ms',
      );
    } on http.ClientException catch (error) {
      throw SurfpoolRpcException(
        SurfpoolFailure.unreachable,
        method,
        error.message,
      );
    } on FormatException catch (error) {
      throw SurfpoolRpcException(
        SurfpoolFailure.malformed,
        method,
        error.message,
      );
    }
  }

  static void _requireAddress(String address, String method) {
    if (address.isEmpty) {
      throw SurfpoolRpcException(
        SurfpoolFailure.malformed,
        method,
        'an empty address',
      );
    }
  }

  /// Whether [host] is where a developer's local validator is likely to be.
  static bool _isLocalNetwork(String host) {
    if (host == 'localhost' ||
        host == '127.0.0.1' ||
        host == '::1' ||
        host == '10.0.2.2' ||
        host == 'host.docker.internal' ||
        host.endsWith('.local')) {
      return true;
    }

    final parts = host.split('.');

    if (parts.length != 4) {
      return false;
    }

    final octets = [for (final part in parts) int.tryParse(part)];

    if (octets.any((octet) => octet == null || octet < 0 || octet > 255)) {
      return false;
    }

    return switch (octets) {
      [10, _, _, _] => true,
      [192, 168, _, _] => true,
      [172, final second?, _, _] => second >= 16 && second <= 31,
      [169, 254, _, _] => true,
      _ => false,
    };
  }
}

/// One airdrop this install has asked a local validator for, pending or
/// confirmed.
typedef SurfpoolFaucetReceipt = ({
  String wallet,
  String genesis,
  String signature,
  int? slot,
  DateTime at,
  bool confirmed,
});

/// The faucet's written record cannot be read, or a new one cannot be saved.
///
/// This is never quietly reset. A record that is forgotten without being
/// asked about would let the next tap pay the faucet a second time.
final class SurfpoolFaucetDataException implements Exception {
  const SurfpoolFaucetDataException(this.message);

  final String message;

  @override
  String toString() => 'SurfpoolFaucetDataException: $message';
}

/// Pays the simnet's burner wallet from a local validator once per chain.
///
/// The moment the faucet returns a signature it is written down with the
/// validator's genesis and `confirmed: false`, before the transfer is
/// awaited. Every later tap, and a freshly reopened app, settles that
/// signature again instead of asking the faucet a second time, and only a
/// confirmed status turns the record into a receipt. Nothing here touches
/// the game's books: the validator's SOL is real only inside the local
/// cluster.
final class SurfpoolFaucet {
  SurfpoolFaucet(this._preferences, this._rpc) : _now = DateTime.now;

  /// Only for tests that need a fixed clock.
  SurfpoolFaucet.at(this._preferences, this._rpc, DateTime Function() now)
    : _now = now;

  static const storageKey = 'reapers.surfpool.faucet.v1';

  final SharedPreferences _preferences;
  final SurfpoolRpc _rpc;

  /// The faucet's amount, in lamports. It is a local demo amount, not the
  /// game's simulated wallet.
  static const lamports = 1_000_000_000;

  final DateTime Function() _now;
  final _pending = <String, Future<SurfpoolFaucetReceipt?>>{};

  /// The airdrop already asked for [wallet] on this or an earlier validator,
  /// pending or confirmed.
  SurfpoolFaucetReceipt? receiptFor(String wallet) => _read()[wallet];

  /// Airdrops [lamports] to [wallet] once per validator chain, settling the
  /// request that is already written down when there is one.
  ///
  /// A tap that arrives while the first is in flight shares its future, and
  /// a confirmed receipt for the same genesis is reused, so while a
  /// signature is known neither a retry nor a double tap can ask the faucet
  /// twice. A transaction error is definitive: the transfer failed on chain,
  /// the record is cleared, and the next call may ask again. But if the
  /// request's own answer is lost before a signature comes back, the
  /// outcome is unknowable — the faucet may still have paid — so the failure
  /// is rethrown with nothing written, and the next call may ask twice. No
  /// client can do better against a faucet that is not idempotent. A
  /// restarted Surfpool has a new genesis and no memory of the old airdrop,
  /// so it may be paid again.
  Future<SurfpoolFaucetReceipt?> fund(String wallet) =>
      _pending[wallet] ??= _pay(wallet).whenComplete(() {
        _pending.remove(wallet);
      });

  Future<SurfpoolFaucetReceipt> _pay(String wallet) async {
    final genesis = await _rpc.genesisHash();
    final known = _read()[wallet];

    if (known != null && known.genesis == genesis) {
      if (known.confirmed) {
        return known;
      }

      // A request for this chain is already outstanding. Settle its
      // signature; asking again could pay twice.
      return _settle(wallet, known);
    }

    final signature = await _rpc.requestAirdrop(wallet, lamports: lamports);
    final asked = (
      wallet: wallet,
      genesis: genesis,
      signature: signature,
      slot: null,
      at: _now().toUtc(),
      confirmed: false,
    );
    await _store(wallet, asked);

    return _settle(wallet, asked);
  }

  /// Waits for [asked]'s transfer and writes the receipt when it confirms.
  ///
  /// A transaction error clears the record, because a failed transfer is
  /// known to have moved nothing. Any other failure leaves the record, so a
  /// later call settles the same signature instead of paying again.
  Future<SurfpoolFaucetReceipt> _settle(
    String wallet,
    SurfpoolFaucetReceipt asked,
  ) async {
    try {
      final status = await _rpc.awaitConfirmation(asked.signature);
      final paid = (
        wallet: asked.wallet,
        genesis: asked.genesis,
        signature: asked.signature,
        slot: status.slot,
        at: asked.at,
        confirmed: true,
      );
      await _store(wallet, paid);

      return paid;
    } on SurfpoolRpcException catch (error) {
      if (error.failure == SurfpoolFailure.transaction) {
        await _clear(wallet, asked.signature);
      }

      rethrow;
    }
  }

  /// Replaces [wallet]'s record, and says so only when the write landed.
  Future<void> _store(String wallet, SurfpoolFaucetReceipt receipt) async {
    final saved = {..._read()}..[wallet] = receipt;

    if (!await _preferences.setString(storageKey, _encode(saved))) {
      throw const SurfpoolFaucetDataException(
        'the faucet record could not be saved',
      );
    }
  }

  /// Forgets [wallet]'s record of [signature], after that transfer failed.
  Future<void> _clear(String wallet, String signature) async {
    final saved = {..._read()};

    if (saved[wallet]?.signature != signature) {
      return;
    }

    saved.remove(wallet);

    if (!await _preferences.setString(storageKey, _encode(saved))) {
      throw const SurfpoolFaucetDataException(
        'the failed airdrop could not be cleared',
      );
    }
  }

  String _encode(Map<String, SurfpoolFaucetReceipt> saved) => jsonEncode({
    'wallets': {
      for (final MapEntry(:key, :value) in saved.entries)
        key: {
          'genesis': value.genesis,
          'signature': value.signature,
          'slot': value.slot,
          'at': value.at.toIso8601String(),
          'confirmed': value.confirmed,
        },
    },
  });

  Map<String, SurfpoolFaucetReceipt> _read() {
    final stored = _preferences.getString(storageKey);

    if (stored == null) {
      return const {};
    }

    final Object? decoded;

    try {
      decoded = jsonDecode(stored);
    } on FormatException catch (error) {
      throw SurfpoolFaucetDataException(
        'the saved faucet record is not JSON: ${error.message}',
      );
    }

    return switch (decoded) {
      {'wallets': final Map wallets} => {
        for (final MapEntry(:key, :value) in wallets.entries)
          '$key': _decodeReceipt('$key', value),
      },
      _ => throw const SurfpoolFaucetDataException(
        'the saved faucet record has no wallet map',
      ),
    };
  }

  SurfpoolFaucetReceipt _decodeReceipt(String wallet, Object? value) {
    final entry = switch (value) {
      final Map entry => entry.cast<String, dynamic>(),
      _ => throw const SurfpoolFaucetDataException(
        'a saved faucet entry is not an object',
      ),
    };
    final (genesis, signature) = switch ((
      entry['genesis'],
      entry['signature'],
    )) {
      (final String genesis, final String signature)
          when genesis.isNotEmpty && signature.isNotEmpty =>
        (genesis, signature),
      _ => throw const SurfpoolFaucetDataException(
        'a saved faucet entry has no genesis or signature',
      ),
    };
    final confirmed = switch (entry['confirmed']) {
      final bool confirmed => confirmed,
      _ => throw const SurfpoolFaucetDataException(
        'a saved faucet entry does not say whether it is confirmed',
      ),
    };
    final DateTime at;

    try {
      at = switch (entry['at']) {
        final String at => DateTime.parse(at),
        _ => throw const SurfpoolFaucetDataException(
          'a saved faucet entry has no time',
        ),
      };
    } on FormatException {
      throw const SurfpoolFaucetDataException(
        'a saved faucet entry has an unreadable time',
      );
    }

    return (
      wallet: wallet,
      genesis: genesis,
      signature: signature,
      slot: switch (entry['slot']) {
        final int slot => slot,
        null => null,
        _ => throw const SurfpoolFaucetDataException(
          'a saved faucet entry has a slot that is not a slot',
        ),
      },
      at: at,
      confirmed: confirmed,
    );
  }
}
