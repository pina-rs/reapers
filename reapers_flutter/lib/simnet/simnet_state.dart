import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/state/player.dart';
import 'simnet_chain.dart';
import 'surfpool_rpc.dart';

part 'simnet_state.g.dart';

/// Whether this build rehearses the chain on the phone, set with
/// `--dart-define=REAPERS_SIMNET=true`. It plays offline, so it cannot be
/// combined with `REAPERS_SERVER_URL`.
const reapersSimnet = bool.fromEnvironment('REAPERS_SIMNET');

/// An optional real Solana JSON-RPC endpoint for the simnet, set with
/// `--dart-define=REAPERS_SURFPOOL_URL=http://localhost:8899`.
///
/// It is a local validator's endpoint, not a second game server. Empty means
/// the simnet stays entirely on the phone.
const reapersSurfpoolUrl = String.fromEnvironment('REAPERS_SURFPOOL_URL');

/// The local validator, when a simnet build names one, or why it could not
/// be set up.
typedef SurfpoolSetup = ({SurfpoolRpc? rpc, String? problem});

/// The local validator, when a simnet build names one.
///
/// It is deliberately separate from [simnetChainProvider]: the game's
/// receipts are not Solana transactions, and no game state is coordinated
/// through the validator.
@Riverpod(keepAlive: true)
SurfpoolSetup surfpoolSetup(Ref ref) {
  if (reapersSurfpoolUrl.isEmpty) {
    return (rpc: null, problem: null);
  }

  if (!reapersSimnet) {
    return (
      rpc: null,
      problem:
          'REAPERS_SURFPOOL_URL belongs to the simnet: '
          'add --dart-define=REAPERS_SIMNET=true.',
    );
  }

  try {
    final rpc = SurfpoolRpc(endpoint: Uri.parse(reapersSurfpoolUrl));

    ref.onDispose(rpc.close);

    return (rpc: rpc, problem: null);
  } on FormatException catch (error) {
    return (rpc: null, problem: error.message);
  }
}

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
