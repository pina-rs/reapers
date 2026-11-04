// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'simnet_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning
/// The simnet, in a build that has one.

@ProviderFor(simnetChain)
final simnetChainProvider = SimnetChainProvider._();

/// The simnet, in a build that has one.

final class SimnetChainProvider
    extends $FunctionalProvider<SimnetChain?, SimnetChain?, SimnetChain?>
    with $Provider<SimnetChain?> {
  /// The simnet, in a build that has one.
  SimnetChainProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'simnetChainProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$simnetChainHash();

  @$internal
  @override
  $ProviderElement<SimnetChain?> $createElement($ProviderPointer pointer) =>
      $ProviderElement(pointer);

  @override
  SimnetChain? create(Ref ref) {
    return simnetChain(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(SimnetChain? value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<SimnetChain?>(value),
    );
  }
}

String _$simnetChainHash() => r'306648b2f2b8b5b4025248e6b18b5cc2bde0cf9d';

/// Every simnet transaction, oldest first, as each one is confirmed.

@ProviderFor(simnetLedger)
final simnetLedgerProvider = SimnetLedgerProvider._();

/// Every simnet transaction, oldest first, as each one is confirmed.

final class SimnetLedgerProvider
    extends
        $FunctionalProvider<
          AsyncValue<List<SimnetTransaction>>,
          List<SimnetTransaction>,
          Stream<List<SimnetTransaction>>
        >
    with
        $FutureModifier<List<SimnetTransaction>>,
        $StreamProvider<List<SimnetTransaction>> {
  /// Every simnet transaction, oldest first, as each one is confirmed.
  SimnetLedgerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'simnetLedgerProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$simnetLedgerHash();

  @$internal
  @override
  $StreamProviderElement<List<SimnetTransaction>> $createElement(
    $ProviderPointer pointer,
  ) => $StreamProviderElement(pointer);

  @override
  Stream<List<SimnetTransaction>> create(Ref ref) {
    return simnetLedger(ref);
  }
}

String _$simnetLedgerHash() => r'5c6da28a5b5f1d440d481b11db2b28900ffb7a54';
