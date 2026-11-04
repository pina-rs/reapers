import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/state/player.dart';
import 'simnet_chain.dart';

part 'simnet_state.g.dart';

/// Whether this build rehearses the chain on the phone, set with
/// `--dart-define=REAPERS_SIMNET=true`. It plays offline, so it cannot be
/// combined with `REAPERS_SERVER_URL`.
const reapersSimnet = bool.fromEnvironment('REAPERS_SIMNET');

/// The simnet, in a build that has one.
@Riverpod(keepAlive: true)
SimnetChain? simnetChain(Ref ref) => reapersSimnet
    ? SimnetChain.open(ref.watch(sharedPreferencesProvider))
    : null;

/// Every simnet transaction, oldest first, as each one is confirmed.
@riverpod
Stream<List<SimnetTransaction>> simnetLedger(Ref ref) async* {
  final chain = ref.watch(simnetChainProvider);

  if (chain == null) {
    yield const [];
    return;
  }

  yield chain.transactions;

  await for (final _ in chain.posted) {
    yield chain.transactions;
  }
}
