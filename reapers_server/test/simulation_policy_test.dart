import 'package:reapers_server/src/player/player_endpoint.dart';
import 'package:reapers_server/src/generated/protocol.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod/serverpod.dart';
import 'package:test/test.dart';

void main() {
  test('deployed servers refuse the invented NFT economy', () {
    for (final mode in [
      ServerpodRunMode.staging,
      ServerpodRunMode.production,
    ]) {
      expect(
        () => PlayerEndpoint.requireSimulation(mode),
        throwsA(
          isA<Refused>().having(
            (error) => error.refusal,
            'reason',
            Refusal.marketUnavailable.name,
          ),
        ),
      );
    }
    for (final mode in [ServerpodRunMode.development, ServerpodRunMode.test]) {
      expect(() => PlayerEndpoint.requireSimulation(mode), returnsNormally);
    }
  });
}
