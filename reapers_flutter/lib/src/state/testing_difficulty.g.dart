// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'testing_difficulty.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning
/// How forgiving hunts are while the game is being tested.
///
/// A build that is not a release starts on easy, so that whoever is trying
/// the game can bring a husk down and see a whole hunt, and can be switched
/// from the hunt's own threshold. A release build is always the game as it
/// is meant to be played, and has no switch.

@ProviderFor(TestingDifficulty)
final testingDifficultyProvider = TestingDifficultyProvider._();

/// How forgiving hunts are while the game is being tested.
///
/// A build that is not a release starts on easy, so that whoever is trying
/// the game can bring a husk down and see a whole hunt, and can be switched
/// from the hunt's own threshold. A release build is always the game as it
/// is meant to be played, and has no switch.
final class TestingDifficultyProvider
    extends $NotifierProvider<TestingDifficulty, Difficulty> {
  /// How forgiving hunts are while the game is being tested.
  ///
  /// A build that is not a release starts on easy, so that whoever is trying
  /// the game can bring a husk down and see a whole hunt, and can be switched
  /// from the hunt's own threshold. A release build is always the game as it
  /// is meant to be played, and has no switch.
  TestingDifficultyProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'testingDifficultyProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$testingDifficultyHash();

  @$internal
  @override
  TestingDifficulty create() => TestingDifficulty();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(Difficulty value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<Difficulty>(value),
    );
  }
}

String _$testingDifficultyHash() => r'4f65c9867c9281c83f7140985dd9e80e2dd3e998';

/// How forgiving hunts are while the game is being tested.
///
/// A build that is not a release starts on easy, so that whoever is trying
/// the game can bring a husk down and see a whole hunt, and can be switched
/// from the hunt's own threshold. A release build is always the game as it
/// is meant to be played, and has no switch.

abstract class _$TestingDifficulty extends $Notifier<Difficulty> {
  Difficulty build();
  @$mustCallSuper
  @override
  WhenComplete runBuild() {
    final ref = this.ref as $Ref<Difficulty, Difficulty>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<Difficulty, Difficulty>,
              Difficulty,
              Object?,
              Object?
            >;
    return element.handleCreate(ref, build);
  }
}
