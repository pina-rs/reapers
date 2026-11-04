// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'player.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning
/// The preferences store, opened once before the app starts and handed in
/// through an override.

@ProviderFor(sharedPreferences)
final sharedPreferencesProvider = SharedPreferencesProvider._();

/// The preferences store, opened once before the app starts and handed in
/// through an override.

final class SharedPreferencesProvider
    extends
        $FunctionalProvider<
          SharedPreferences,
          SharedPreferences,
          SharedPreferences
        >
    with $Provider<SharedPreferences> {
  /// The preferences store, opened once before the app starts and handed in
  /// through an override.
  SharedPreferencesProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'sharedPreferencesProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$sharedPreferencesHash();

  @$internal
  @override
  $ProviderElement<SharedPreferences> $createElement(
    $ProviderPointer pointer,
  ) => $ProviderElement(pointer);

  @override
  SharedPreferences create(Ref ref) {
    return sharedPreferences(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(SharedPreferences value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<SharedPreferences>(value),
    );
  }
}

String _$sharedPreferencesHash() => r'7549e653690f4f4e0483e83597dfd5488ae80fc4';

/// Whoever keeps this player's books: the server when the build names one,
/// the phone posting to its simnet in a rehearsal build, and the phone
/// itself otherwise.

@ProviderFor(gameBackend)
final gameBackendProvider = GameBackendProvider._();

/// Whoever keeps this player's books: the server when the build names one,
/// the phone posting to its simnet in a rehearsal build, and the phone
/// itself otherwise.

final class GameBackendProvider
    extends $FunctionalProvider<GameBackend, GameBackend, GameBackend>
    with $Provider<GameBackend> {
  /// Whoever keeps this player's books: the server when the build names one,
  /// the phone posting to its simnet in a rehearsal build, and the phone
  /// itself otherwise.
  GameBackendProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'gameBackendProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$gameBackendHash();

  @$internal
  @override
  $ProviderElement<GameBackend> $createElement($ProviderPointer pointer) =>
      $ProviderElement(pointer);

  @override
  GameBackend create(Ref ref) {
    return gameBackend(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(GameBackend value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<GameBackend>(value),
    );
  }
}

String _$gameBackendHash() => r'1a578983e68df4b3a131c614f4be508bbc313fa2';

/// The player: what they own and what they have done.
///
/// Every screen reads the player from here, and every change goes through
/// one of these methods, which asks the keeper and then shows what it says.
/// A refusal leaves the player as they were and rethrows the [RuleBreach]
/// for the screen to explain.

@ProviderFor(Player)
final playerProvider = PlayerProvider._();

/// The player: what they own and what they have done.
///
/// Every screen reads the player from here, and every change goes through
/// one of these methods, which asks the keeper and then shows what it says.
/// A refusal leaves the player as they were and rethrows the [RuleBreach]
/// for the screen to explain.
final class PlayerProvider extends $AsyncNotifierProvider<Player, PlayerState> {
  /// The player: what they own and what they have done.
  ///
  /// Every screen reads the player from here, and every change goes through
  /// one of these methods, which asks the keeper and then shows what it says.
  /// A refusal leaves the player as they were and rethrows the [RuleBreach]
  /// for the screen to explain.
  PlayerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'playerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$playerHash();

  @$internal
  @override
  Player create() => Player();
}

String _$playerHash() => r'2ffe5f3bc06ed1fac5086e77ecd5a76cdb5c661f';

/// The player: what they own and what they have done.
///
/// Every screen reads the player from here, and every change goes through
/// one of these methods, which asks the keeper and then shows what it says.
/// A refusal leaves the player as they were and rethrows the [RuleBreach]
/// for the screen to explain.

abstract class _$Player extends $AsyncNotifier<PlayerState> {
  FutureOr<PlayerState> build();
  @$mustCallSuper
  @override
  WhenComplete runBuild() {
    final ref = this.ref as $Ref<AsyncValue<PlayerState>, PlayerState>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<AsyncValue<PlayerState>, PlayerState>,
              AsyncValue<PlayerState>,
              Object?,
              Object?
            >;
    return element.handleCreate(ref, build);
  }
}
