import 'dart:math';
import 'dart:typed_data';

import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod/serverpod.dart';
import 'package:serverpod_auth_idp_server/core.dart';

import '../receipts/absorption_receipt.dart';
import 'player_books.dart';

/// Everything a player can ask of their own books.
///
/// Each method finds the player from the session, so no method takes an
/// identifier from the client, and each change is one shared rule applied
/// to the stored state. Anything the server must decide (a seed, the time)
/// is decided here, never sent by the game.
class PlayerEndpoint extends Endpoint {
  /// How much kinder the server's replay of a hunt is than the phone's own
  /// play of it. See `HuntStats.derive`.
  static const replayLeniency = 1.03;

  static final _random = Random.secure();

  @override
  bool get requireLogin => true;

  /// The player as they stand now, with their books opened on first use.
  Future<PlayerState> load(Session session) =>
      PlayerBooks.change(session, (state) => state);

  /// Binds a reaper drawn from a seed only the server knows.
  Future<PlayerState> takeOath(Session session) =>
      PlayerBooks.change(session, (state) => state.takeOath(seed: _seed()));

  /// Buys the NFT [nftId] names into the vault.
  Future<PlayerState> acquire(Session session, String nftId) =>
      PlayerBooks.change(session, (state) => state.acquire(_nft(nftId)));

  /// Releases the NFT [nftId] names from the vault into the veil.
  Future<PlayerState> release(Session session, String nftId) =>
      PlayerBooks.change(
        session,
        (state) => state.release(_nft(nftId), seed: _seed(), now: _now()),
      );

  /// Replays the recorded hunt for [sightingId] and settles what it earned.
  ///
  /// [log] is the packed `HuntLog`. The verdict is the replay's; nothing the
  /// game claims about how the hunt went is read. How well the reaper feeds
  /// on an absorbed husk is rolled here, from a seed the game never sees,
  /// and the result is signed so that it can be proven to a chain later:
  /// see `AbsorptionReceipt`.
  Future<HuntSettlement> settleHunt(
    Session session,
    int sightingId,
    ByteData log,
  ) async {
    late final HuntSettlement settlement;

    await PlayerBooks.change(session, (state) async {
      final judged = state.judgeHunt(
        sightingId,
        _log(log),
        now: _now(),
        feedingSeed: _seed(),
        leniency: replayLeniency,
      );

      settlement = HuntSettlement(
        state: await _signed(session, judged),
        outcome: judged.outcome,
        xp: judged.xp,
        anima: judged.anima,
        feeding: judged.feeding,
      );

      return settlement.state;
    });

    return settlement;
  }

  /// The public key absorption receipts are signed with, as 64 hexadecimal
  /// digits. It is what a receipt is checked against, here or on a chain.
  Future<String> receiptKey(Session session) async => [
    for (final byte in (await _signer(session)).publicKey)
      byte.toRadixString(16).padLeft(2, '0'),
  ].join();

  /// [judged]'s state, with the absorption it recorded signed. A husk that
  /// got away leaves nothing to sign.
  static Future<PlayerState> _signed(
    Session session,
    HuntSettlement judged,
  ) async {
    if (!judged.outcome.absorbed) {
      return judged.state;
    }

    final absorption = judged.state.codex.last;
    final signer = await _signer(session);
    final message = await AbsorptionReceipt.message(
      playerId: '${session.authenticated!.authUserId}',
      absorption: absorption,
    );

    return judged.state.signAbsorption(
      absorption.nft,
      await signer.sign(message),
    );
  }

  /// The signer for this server. Development and tests run without the
  /// secret and sign with a key anyone can derive; a deployed server is not
  /// allowed to start that way.
  static Future<ReceiptSigner> _signer(Session session) =>
      ReceiptSigner.fromSecret(
        session.passwords[ReceiptSigner.secretName] ??
            ReceiptSigner.developmentSecret,
      );

  /// Buys the weapon [weaponId] names with anima.
  Future<PlayerState> buyWeapon(Session session, String weaponId) =>
      PlayerBooks.change(
        session,
        (state) => state.buyWeapon(_weapon(weaponId)),
      );

  /// Takes the bought weapon [weaponId] names in hand.
  Future<PlayerState> wield(Session session, String weaponId) =>
      PlayerBooks.change(session, (state) => state.wield(_weapon(weaponId)));

  /// Buys the outfit [outfit] names with anima.
  Future<PlayerState> buyOutfit(Session session, String outfit) =>
      PlayerBooks.change(session, (state) => state.buyOutfit(_outfit(outfit)));

  /// Puts on the bought outfit [outfit] names.
  Future<PlayerState> wear(Session session, String outfit) =>
      PlayerBooks.change(session, (state) => state.wear(_outfit(outfit)));

  static DateTime _now() => DateTime.now().toUtc();

  static int _seed() => _random.nextInt(1 << 32);

  /// The identifiers below come from the game, so a malformed one is a
  /// refusal the player can be told about, not a server error.
  static Nft _nft(String id) => _parsed(() => Nft.parse(id), Refusal.notListed);

  static Weapon _weapon(String id) =>
      _parsed(() => Weapon.parse(id), Refusal.notOwned);

  static Outfit _outfit(String name) =>
      _parsed(() => Outfit.values.byName(name), Refusal.notOwned);

  static HuntLog _log(ByteData bytes) => _parsed(
    () => HuntLog.decode(
      bytes.buffer.asUint8List(bytes.offsetInBytes, bytes.lengthInBytes),
    ),
    Refusal.huntUnproven,
  );

  static T _parsed<T>(T Function() parse, Refusal otherwise) {
    try {
      return parse();
    } on FormatException {
      throw RuleBreach(otherwise);
    } on ArgumentError {
      throw RuleBreach(otherwise);
    }
  }
}
