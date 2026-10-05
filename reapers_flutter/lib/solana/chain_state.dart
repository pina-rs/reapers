import 'package:reapers_client/reapers_client.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/backend/server_backend.dart';
import '../src/state/player.dart';

part 'chain_state.g.dart';

@riverpod
Future<ChainStatus?> chainStatus(Ref ref) {
  final backend = ref.watch(gameBackendProvider);
  return backend is ServerBackend ? backend.chainStatus() : Future.value(null);
}
