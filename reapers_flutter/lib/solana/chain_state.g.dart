// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'chain_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(chainStatus)
final chainStatusProvider = ChainStatusProvider._();

final class ChainStatusProvider
    extends
        $FunctionalProvider<
          AsyncValue<ChainStatus?>,
          ChainStatus?,
          FutureOr<ChainStatus?>
        >
    with $FutureModifier<ChainStatus?>, $FutureProvider<ChainStatus?> {
  ChainStatusProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'chainStatusProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$chainStatusHash();

  @$internal
  @override
  $FutureProviderElement<ChainStatus?> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<ChainStatus?> create(Ref ref) {
    return chainStatus(ref);
  }
}

String _$chainStatusHash() => r'99ffa863a8aa57dcd22144873a0910932f4ad8a0';
