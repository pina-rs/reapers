import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_client/reapers_client.dart';
import 'package:reapers_flutter/src/backend/game_backend.dart';
import 'package:reapers_flutter/src/backend/server_backend.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod_auth_idp_flutter/serverpod_auth_idp_flutter.dart';

/// Replace only authentication's external validation. Requests below use
/// the real generated client, serialization and HTTP transport.
final class _Session implements FlutterAuthSessionManager {
  bool reachable = false;
  bool timedOut = false;
  @override
  bool get isAuthenticated => true;
  @override
  Future<String?> get authHeaderValue async => null;
  @override
  void setCaller(Caller caller) {}
  @override
  Future<RefreshAuthKeyResult> refreshAuthKey({bool force = false}) async =>
      RefreshAuthKeyResult.skipped;
  @override
  Object noSuchMethod(Invocation invocation) => throw UnsupportedError(
    'Unexpected session call: ${invocation.memberName}',
  );
  @override
  Future<bool> initialize({
    Duration timeout = const Duration(seconds: 2),
  }) async {
    if (timedOut) throw TimeoutException('Session validation timed out');
    return reachable;
  }
}

void main() {
  late HttpServer server;
  late Client client;
  late _Session session;
  late ServerBackend backend;
  GeoFix? stored;
  var offline = false;

  setUp(() async {
    stored = null;
    offline = false;
    server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    server.listen((request) async {
      request.response.headers.contentType = ContentType.json;
      if (offline) {
        request.response.statusCode = HttpStatus.serviceUnavailable;
        await request.response.close();
        return;
      }
      if (request.uri.path == '/player/updateLocation') {
        final args = jsonDecode(
          await utf8.decoder.bind(request).join(),
        ) as Map<String, dynamic>;
        final fix = args['fix'];
        stored = fix == null
            ? null
            : GeoFix.fromJson((fix as Map).cast<String, dynamic>());
        request.response.write('null');
      } else if (request.uri.path == '/player/location') {
        request.response.write(jsonEncode(stored?.toJson()));
      } else if (request.uri.path == '/player/load') {
        request.response.write(jsonEncode(PlayerState.initial().toJson()));
      } else {
        request.response.statusCode = HttpStatus.notFound;
      }
      await request.response.close();
    });
    session = _Session();
    client = Client(
      'http://127.0.0.1:${server.port}/',
      connectionTimeout: const Duration(seconds: 1),
    )..authSessionManager = session;
    backend = ServerBackend(client);
  });

  tearDown(() async {
    client.close();
    await server.close(force: true);
  });

  test(
    'a failed session check can recover without opening another guest',
    () async {
      await expectLater(backend.load(), throwsA(isA<BackendUnavailable>()));
      session.reachable = true;
      expect((await backend.load()).oathTaken, isFalse);
    },
  );

  test('a timed-out sign-in does not poison later attempts', () async {
    session.timedOut = true;
    await expectLater(backend.load(), throwsA(isA<BackendUnavailable>()));
    session
      ..timedOut = false
      ..reachable = true;
    expect((await backend.load()).oathTaken, isFalse);
  });

  test('a server outage becomes a recoverable keeper failure', () async {
    session.reachable = true;
    offline = true;
    await expectLater(backend.load(), throwsA(isA<BackendUnavailable>()));
    offline = false;
    expect((await backend.load()).oathTaken, isFalse);
  });

  test(
    'location reaches the generated endpoint and can be read and cleared',
    () async {
      session.reachable = true;
      final fix = GeoFix(
        at: const GeoPoint(51.5, -.08),
        accuracy: 4,
        observedAt: DateTime.now().toUtc(),
      );
      await backend.locate(fix);
      final read = (await client.player.location())!;
      expect(read.at, fix.at);
      expect(read.observedAt, fix.observedAt);
      await backend.locate(null);
      expect(await client.player.location(), isNull);
    },
  );
}
