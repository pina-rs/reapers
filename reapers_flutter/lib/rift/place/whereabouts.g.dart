// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'whereabouts.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning
/// What says where the phone is. The app hands in the real thing at
/// startup; without it nothing is known, which is what every test wants.

@ProviderFor(whereabouts)
final whereaboutsProvider = WhereaboutsProvider._();

/// What says where the phone is. The app hands in the real thing at
/// startup; without it nothing is known, which is what every test wants.

final class WhereaboutsProvider
    extends $FunctionalProvider<Whereabouts, Whereabouts, Whereabouts>
    with $Provider<Whereabouts> {
  /// What says where the phone is. The app hands in the real thing at
  /// startup; without it nothing is known, which is what every test wants.
  WhereaboutsProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'whereaboutsProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$whereaboutsHash();

  @$internal
  @override
  $ProviderElement<Whereabouts> $createElement($ProviderPointer pointer) =>
      $ProviderElement(pointer);

  @override
  Whereabouts create(Ref ref) {
    return whereabouts(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(Whereabouts value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<Whereabouts>(value),
    );
  }
}

String _$whereaboutsHash() => r'917ce96f4710831dafce5759f25eaaba4ec4b031';
