// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'soundtrack.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning
/// What makes the sound. The app hands in a real engine at startup; without
/// one the game is silent, which is what every test wants.

@ProviderFor(scoreEngine)
final scoreEngineProvider = ScoreEngineProvider._();

/// What makes the sound. The app hands in a real engine at startup; without
/// one the game is silent, which is what every test wants.

final class ScoreEngineProvider
    extends $FunctionalProvider<ScoreEngine, ScoreEngine, ScoreEngine>
    with $Provider<ScoreEngine> {
  /// What makes the sound. The app hands in a real engine at startup; without
  /// one the game is silent, which is what every test wants.
  ScoreEngineProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'scoreEngineProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$scoreEngineHash();

  @$internal
  @override
  $ProviderElement<ScoreEngine> $createElement($ProviderPointer pointer) =>
      $ProviderElement(pointer);

  @override
  ScoreEngine create(Ref ref) {
    return scoreEngine(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ScoreEngine value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ScoreEngine>(value),
    );
  }
}

String _$scoreEngineHash() => r'cb2b7777caccd2fe026ff726cc42ad2fb7b0c5de';

/// Whether the player wants music at all.

@ProviderFor(MusicEnabled)
final musicEnabledProvider = MusicEnabledProvider._();

/// Whether the player wants music at all.
final class MusicEnabledProvider extends $NotifierProvider<MusicEnabled, bool> {
  /// Whether the player wants music at all.
  MusicEnabledProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'musicEnabledProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$musicEnabledHash();

  @$internal
  @override
  MusicEnabled create() => MusicEnabled();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(bool value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<bool>(value),
    );
  }
}

String _$musicEnabledHash() => r'fcdcc57dc768abd8511c13e0c93e93705d3a34a0';

/// Whether the player wants music at all.

abstract class _$MusicEnabled extends $Notifier<bool> {
  bool build();
  @$mustCallSuper
  @override
  WhenComplete runBuild() {
    final ref = this.ref as $Ref<bool, bool>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<bool, bool>,
              bool,
              Object?,
              Object?
            >;
    return element.handleCreate(ref, build);
  }
}

/// The game's music: which piece is playing, and how it is heard.

@ProviderFor(soundtrack)
final soundtrackProvider = SoundtrackProvider._();

/// The game's music: which piece is playing, and how it is heard.

final class SoundtrackProvider
    extends $FunctionalProvider<Soundtrack, Soundtrack, Soundtrack>
    with $Provider<Soundtrack> {
  /// The game's music: which piece is playing, and how it is heard.
  SoundtrackProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'soundtrackProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$soundtrackHash();

  @$internal
  @override
  $ProviderElement<Soundtrack> $createElement($ProviderPointer pointer) =>
      $ProviderElement(pointer);

  @override
  Soundtrack create(Ref ref) {
    return soundtrack(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(Soundtrack value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<Soundtrack>(value),
    );
  }
}

String _$soundtrackHash() => r'214f9facf2a006a4b24cfd905edf995a349f998b';
