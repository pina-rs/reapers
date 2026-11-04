import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/simnet/surfpool_rpc.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// A local stand-in for Surfpool's JSON-RPC port. Every test talks HTTP to
/// this server, so the client's transport, envelopes and retries are real.
final class _LocalRpc {
  late HttpServer _server;
  int airdrops = 0;
  int statusChecks = 0;
  int pendingStatuses = 0;
  bool errorOnHealth = false;
  bool errorOnAirdrop = false;
  bool failedStatus = false;
  int httpStatus = HttpStatus.ok;
  String? malformedBody;
  Object? balanceValue = 5_000_000_000;
  String genesis = 'EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG';
  Duration delay = Duration.zero;

  Uri get endpoint => Uri.parse('http://127.0.0.1:${_server.port}');

  Future<void> start() async {
    _server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    _server.listen((request) async {
      if (delay > Duration.zero) {
        await Future<void>.delayed(delay);
      }

      final body = (jsonDecode(await utf8.decoder.bind(request).join()) as Map)
          .cast<String, dynamic>();
      final method = body['method'] as String;
      final id = body['id'];
      Object? result;
      Map<String, Object?>? error;

      switch (method) {
        case 'getHealth':
          if (errorOnHealth) {
            error = {'code': -32000, 'message': 'validator is starting'};
          } else {
            result = 'ok';
          }
        case 'getGenesisHash':
          result = genesis;
        case 'getBalance':
          result = {
            'context': {'slot': 1},
            'value': balanceValue,
          };
        case 'requestAirdrop':
          airdrops++;
          if (errorOnAirdrop) {
            error = {'code': -32002, 'message': 'faucet is empty'};
          } else {
            result = 'airdrop-$airdrops';
          }
        case 'getSignatureStatuses':
          statusChecks++;
          if (statusChecks <= pendingStatuses) {
            result = {
              'context': {'slot': 1},
              'value': [null],
            };
          } else if (failedStatus) {
            result = {
              'context': {'slot': 1},
              'value': [
                {
                  'slot': 9,
                  'err': {
                    'InstructionError': [0, 'Custom'],
                  },
                  'confirmationStatus': 'confirmed',
                },
              ],
            };
          } else {
            result = {
              'context': {'slot': 1},
              'value': [
                {
                  'slot': 42,
                  'err': null,
                  'confirmationStatus': 'confirmed',
                },
              ],
            };
          }
        default:
          error = {'code': -32601, 'message': 'method not found'};
      }

      request.response.statusCode = httpStatus;
      request.response.headers.contentType = ContentType.json;
      request.response.write(
        malformedBody ??
            jsonEncode({
              'jsonrpc': '2.0',
              'id': id,
              if (error != null) 'error': error else 'result': result,
            }),
      );
      await request.response.close();
    });
  }

  Future<void> stop() => _server.close(force: true);
}

/// A record keeper whose saves never land, the way full storage does.
final class _FailingPreferences implements SharedPreferences {
  _FailingPreferences(this._real);

  final SharedPreferences _real;

  @override
  String? getString(String key) => _real.getString(key);

  @override
  Future<bool> setString(String key, String value) async => false;

  @override
  Object noSuchMethod(Invocation invocation) => throw UnsupportedError(
    'Unexpected preference call: ${invocation.memberName}',
  );
}

void main() {
  late _LocalRpc rpcServer;

  SurfpoolRpc rpc({Duration? timeout}) => SurfpoolRpc(
    endpoint: rpcServer.endpoint,
    timeout: timeout ?? const Duration(seconds: 2),
    pollInterval: const Duration(milliseconds: 1),
  );

  setUp(() async {
    rpcServer = _LocalRpc();
    await rpcServer.start();
    SharedPreferences.setMockInitialValues({});
  });

  tearDown(() => rpcServer.stop());

  test('health, genesis and balance are standard JSON-RPC calls', () async {
    final client = rpc();
    addTearDown(client.close);

    await client.health();
    expect(
      await client.genesisHash(),
      'EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG',
    );
    expect(
      await client.balance('11111111111111111111111111111111'),
      5_000_000_000,
    );
  });

  test('a fractional or negative balance is refused, not rounded', () async {
    final client = rpc();
    addTearDown(client.close);

    for (final value in [5.5, -1]) {
      rpcServer.balanceValue = value;

      await expectLater(
        client.balance('11111111111111111111111111111111'),
        throwsA(
          isA<SurfpoolRpcException>().having(
            (error) => error.failure,
            'failure',
            SurfpoolFailure.malformed,
          ),
        ),
      );
    }
  });

  test(
    'a requestAirdrop returns its signature, which then confirms',
    () async {
      rpcServer.pendingStatuses = 2;
      final client = rpc();
      addTearDown(client.close);

      final signature = await client.requestAirdrop(
        '11111111111111111111111111111111',
        lamports: 1_000_000_000,
      );
      expect(signature, 'airdrop-1');
      expect(rpcServer.airdrops, 1);
      expect(rpcServer.statusChecks, 0, reason: 'the request alone waits not');

      final status = await client.awaitConfirmation(signature);

      expect(status.slot, 42);
      expect(status.confirmed, isTrue);
      expect(rpcServer.statusChecks, 3, reason: 'two unknown, then confirmed');
    },
  );

  test('a transaction error is not a confirmation', () async {
    rpcServer.failedStatus = true;
    final client = rpc();
    addTearDown(client.close);

    final signature = await client.requestAirdrop(
      '11111111111111111111111111111111',
    );

    await expectLater(
      client.awaitConfirmation(signature),
      throwsA(
        isA<SurfpoolRpcException>().having(
          (error) => error.failure,
          'failure',
          SurfpoolFailure.transaction,
        ),
      ),
    );
  });

  test(
    'an RPC error, an HTTP error and a malformed body all surface',
    () async {
      final client = rpc();
      addTearDown(client.close);

      rpcServer.errorOnHealth = true;
      await expectLater(
        client.health(),
        throwsA(
          isA<SurfpoolRpcException>().having(
            (error) => error.failure,
            'failure',
            SurfpoolFailure.rpc,
          ),
        ),
      );

      rpcServer
        ..errorOnHealth = false
        ..httpStatus = HttpStatus.serviceUnavailable;
      await expectLater(
        client.health(),
        throwsA(
          isA<SurfpoolRpcException>().having(
            (error) => error.failure,
            'failure',
            SurfpoolFailure.http,
          ),
        ),
      );

      rpcServer
        ..httpStatus = HttpStatus.ok
        ..malformedBody = '{"jsonrpc":';
      await expectLater(
        client.health(),
        throwsA(
          isA<SurfpoolRpcException>().having(
            (error) => error.failure,
            'failure',
            SurfpoolFailure.malformed,
          ),
        ),
      );
    },
  );

  test('a slow validator times out instead of hanging the game', () async {
    rpcServer.delay = const Duration(milliseconds: 300);
    final client = rpc(timeout: const Duration(milliseconds: 30));
    addTearDown(client.close);

    await expectLater(
      client.health(),
      throwsA(
        isA<SurfpoolRpcException>().having(
          (error) => error.failure,
          'failure',
          SurfpoolFailure.timedOut,
        ),
      ),
    );
  });

  test('only a loopback endpoint may use cleartext HTTP', () {
    expect(
      () => SurfpoolRpc.validate(Uri.parse('http://example.com')),
      throwsFormatException,
    );
    expect(
      SurfpoolRpc.validate(Uri.parse('http://localhost:8899')).port,
      8899,
    );
    expect(
      SurfpoolRpc.validate(Uri.parse('http://10.0.2.2:8899')).host,
      '10.0.2.2',
    );
    expect(
      SurfpoolRpc.validate(Uri.parse('http://192.168.1.20:8899')).host,
      '192.168.1.20',
    );
    expect(
      SurfpoolRpc.validate(Uri.parse('https://example.com/rpc')).host,
      'example.com',
    );
  });

  test('a confirmed faucet receipt is written once and reused', () async {
    final preferences = await SharedPreferences.getInstance();
    final client = rpc();
    addTearDown(client.close);
    final faucet = SurfpoolFaucet.at(
      preferences,
      client,
      () => DateTime.utc(2026, 11, 4, 12),
    );
    const wallet = '11111111111111111111111111111111';

    final first = await faucet.fund(wallet);
    final second = await faucet.fund(wallet);

    expect(first, isNotNull);
    expect(first!.confirmed, isTrue);
    expect(second?.signature, first.signature);
    expect(first.genesis, rpcServer.genesis);
    expect(rpcServer.airdrops, 1, reason: 'a retry must not pay twice');
    expect(faucet.receiptFor(wallet)?.slot, 42);
  });

  test('a restarted validator has a new chain and may pay again', () async {
    final preferences = await SharedPreferences.getInstance();
    final client = rpc();
    addTearDown(client.close);
    final faucet = SurfpoolFaucet.at(
      preferences,
      client,
      () => DateTime.utc(2026, 11, 4, 12),
    );
    const wallet = '11111111111111111111111111111111';

    await faucet.fund(wallet);
    rpcServer.genesis = '5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d';
    final again = await faucet.fund(wallet);

    expect(again?.genesis, rpcServer.genesis);
    expect(again?.confirmed, isTrue);
    expect(rpcServer.airdrops, 2);
  });

  test('two taps before the first confirms share one airdrop', () async {
    final preferences = await SharedPreferences.getInstance();
    final client = rpc();
    addTearDown(client.close);
    final faucet = SurfpoolFaucet.at(
      preferences,
      client,
      () => DateTime.utc(2026, 11, 4, 12),
    );
    const wallet = '11111111111111111111111111111111';

    final taps = await Future.wait([faucet.fund(wallet), faucet.fund(wallet)]);

    expect(taps.first?.signature, taps.last?.signature);
    expect(rpcServer.airdrops, 1);
  });

  test(
    'a request whose confirmation times out is settled, never repeated',
    () async {
      final preferences = await SharedPreferences.getInstance();
      final client = rpc(timeout: const Duration(milliseconds: 40));
      addTearDown(client.close);
      const wallet = '11111111111111111111111111111111';

      // The faucet takes the request but never answers with a confirmation
      // in time. The signature is in hand, so the failed tap must leave it
      // written down.
      rpcServer.pendingStatuses = 1000;
      await expectLater(
        SurfpoolFaucet(preferences, client).fund(wallet),
        throwsA(
          isA<SurfpoolRpcException>().having(
            (error) => error.failure,
            'failure',
            SurfpoolFailure.timedOut,
          ),
        ),
      );
      expect(rpcServer.airdrops, 1);

      final asked = SurfpoolFaucet(preferences, client).receiptFor(wallet);

      expect(asked?.confirmed, isFalse, reason: 'still awaiting the transfer');
      expect(asked?.signature, 'airdrop-1');

      // A new instance, as when the app is reopened, settles the same
      // signature instead of asking the faucet for a second airdrop.
      rpcServer.pendingStatuses = 0;
      final paid = await SurfpoolFaucet.at(
        preferences,
        client,
        () => DateTime.utc(2026, 11, 4, 12),
      ).fund(wallet);

      expect(paid?.signature, 'airdrop-1');
      expect(paid?.confirmed, isTrue);
      expect(paid?.slot, 42);
      expect(
        rpcServer.airdrops,
        1,
        reason: 'the faucet was asked exactly once',
      );
    },
  );

  test(
    'a transfer that failed on chain is cleared, so the next call may ask '
    'again',
    () async {
      final preferences = await SharedPreferences.getInstance();
      final client = rpc();
      addTearDown(client.close);
      final faucet = SurfpoolFaucet.at(
        preferences,
        client,
        () => DateTime.utc(2026, 11, 4, 12),
      );
      const wallet = '11111111111111111111111111111111';

      rpcServer.failedStatus = true;
      await expectLater(
        faucet.fund(wallet),
        throwsA(
          isA<SurfpoolRpcException>().having(
            (error) => error.failure,
            'failure',
            SurfpoolFailure.transaction,
          ),
        ),
      );
      expect(
        faucet.receiptFor(wallet),
        isNull,
        reason: 'the failed airdrop is not kept as a receipt',
      );

      rpcServer.failedStatus = false;
      final paid = await faucet.fund(wallet);

      expect(paid?.confirmed, isTrue);
      expect(
        rpcServer.airdrops,
        2,
        reason: 'the failed transfer moved nothing',
      );
    },
  );

  test(
    'a failed faucet call writes nothing, so a retry can pay once',
    () async {
      final preferences = await SharedPreferences.getInstance();
      final client = rpc();
      addTearDown(client.close);
      final faucet = SurfpoolFaucet.at(
        preferences,
        client,
        () => DateTime.utc(2026, 11, 4, 12),
      );
      const wallet = '11111111111111111111111111111111';

      rpcServer.errorOnAirdrop = true;
      await expectLater(
        faucet.fund(wallet),
        throwsA(isA<SurfpoolRpcException>()),
      );
      expect(faucet.receiptFor(wallet), isNull);
      expect(preferences.getString(SurfpoolFaucet.storageKey), isNull);

      rpcServer.errorOnAirdrop = false;
      final paid = await faucet.fund(wallet);

      expect(paid, isNotNull);
      expect(rpcServer.airdrops, 2, reason: 'the refused call paid nothing');
      expect(faucet.receiptFor(wallet)?.signature, paid!.signature);
    },
  );

  test(
    'a save that does not land is surfaced, and nothing pretends it did',
    () async {
      final preferences = await SharedPreferences.getInstance();
      final client = rpc();
      addTearDown(client.close);
      final faucet = SurfpoolFaucet.at(
        _FailingPreferences(preferences),
        client,
        () => DateTime.utc(2026, 11, 4, 12),
      );
      const wallet = '11111111111111111111111111111111';

      await expectLater(
        faucet.fund(wallet),
        throwsA(isA<SurfpoolFaucetDataException>()),
      );
      expect(rpcServer.airdrops, 1, reason: 'the faucet took the request');
      expect(
        faucet.receiptFor(wallet),
        isNull,
        reason: 'and nothing claims to have kept it',
      );

      // The request's outcome is unknowable, so the honest next call — with
      // a record keeper that works — must ask the faucet again.
      final paid = await SurfpoolFaucet(preferences, client).fund(wallet);

      expect(paid?.confirmed, isTrue);
      expect(rpcServer.airdrops, 2);
    },
  );

  test(
    'a saved record that cannot be read is an error, not a fresh slate',
    () async {
      final client = rpc();
      addTearDown(client.close);
      const wallet = '11111111111111111111111111111111';

      for (final stored in [
        '{"wallets":',
        '{"nope": true}',
        '{"wallets": {"$wallet": []}}',
        '{"wallets": {"$wallet": {"genesis": "g", "signature": "s", '
            '"at": "2026-11-04T12:00:00.000Z"}}}',
      ]) {
        SharedPreferences.setMockInitialValues({
          SurfpoolFaucet.storageKey: stored,
        });
        final preferences = await SharedPreferences.getInstance();
        final faucet = SurfpoolFaucet.at(
          preferences,
          client,
          () => DateTime.utc(2026, 11, 4, 12),
        );

        expect(
          () => faucet.receiptFor(wallet),
          throwsA(isA<SurfpoolFaucetDataException>()),
          reason: 'unreadable for: $stored',
        );
        await expectLater(
          faucet.fund(wallet),
          throwsA(isA<SurfpoolFaucetDataException>()),
        );
        expect(
          rpcServer.airdrops,
          0,
          reason: 'a broken record must not invite a second airdrop',
        );
      }
    },
  );
}
