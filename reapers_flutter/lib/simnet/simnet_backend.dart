import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:reapers_shared/reapers_shared.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../src/backend/game_backend.dart';
import '../src/backend/local_backend.dart';
import '../src/backend/rift_session.dart';
import 'simnet_chain.dart';

/// The offline game, with every step the chain is meant to take posted to
/// the [SimnetChain]: the airdrop when a reaper is bound, a purchase, the
/// escrow that releases a husk, and the burn that absorbs it.
///
/// The books are kept by a [LocalBackend] under a key of their own, so the
/// rules, and the player's progress, are exactly the offline game's. The
/// chain is told what happened after the rules allow it; it never decides.
final class SimnetBackend implements GameBackend {
  /// A keeper for the player stored in [preferences], posting to [chain].
  /// The [clock] and [random] are the books', as for a [LocalBackend].
  SimnetBackend(
    SharedPreferences preferences,
    this.chain, {
    DateTime Function()? clock,
    Random? random,
  }) {
    _books = LocalBackend(
      preferences,
      clock: clock,
      random: random,
      storageKey: storageKey,
      signReceipt: (absorption) async =>
          base64Encode(SimnetChain.signReceipt(_receipt(absorption))),
    );
  }

  static const storageKey = 'reapers.simnet.player.v2';

  late final LocalBackend _books;
  final SimnetChain chain;

  @override
  Future<PlayerState> load() => _books.load();

  @override
  Future<void> locate(GeoFix? fix) => _books.locate(fix);

  @override
  Future<PlayerState> takeOath() async {
    final state = await _books.takeOath();

    await chain.submit(
      SimnetInstruction.airdrop,
      lamports: state.lamports,
      accounts: [
        (role: SimnetRole.faucet, address: SimnetChain.faucet),
        (role: SimnetRole.wallet, address: chain.wallet),
      ],
    );

    return state;
  }

  @override
  Future<PlayerState> selectReaper(String id) => _books.selectReaper(id);

  @override
  Future<PlayerState> acquire(Nft nft) async {
    final state = await _books.acquire(nft);

    await chain.submit(
      SimnetInstruction.buy,
      lamports: nft.rarity.lamports,
      nft: nft,
      accounts: [
        (role: SimnetRole.wallet, address: chain.wallet),
        (role: SimnetRole.market, address: SimnetChain.market),
        (role: SimnetRole.mint, address: nft.mockMint),
      ],
    );

    return state;
  }

  @override
  Future<PlayerState> release(Nft nft) async {
    final state = await _books.release(nft);

    await chain.submit(
      SimnetInstruction.escrow,
      nft: nft,
      accounts: [
        (role: SimnetRole.wallet, address: chain.wallet),
        (role: SimnetRole.mint, address: nft.mockMint),
        (role: SimnetRole.escrow, address: await SimnetChain.escrowFor(nft)),
        (role: SimnetRole.program, address: SimnetChain.program),
      ],
    );

    return state;
  }

  /// Settles the hunt. An absorbed husk is burned out of escrow, and the
  /// burn carries the receipt the keeper signed; one that got away stays in
  /// escrow, still huntable.
  @override
  Future<HuntSettlement> settleHunt(int sightingId, HuntLog log) async {
    final settlement = await _books.settleHunt(sightingId, log);

    if (!settlement.outcome.absorbed) {
      return settlement;
    }

    final absorption = settlement.state.codex.last;
    final nft = absorption.nft;

    await chain.submit(
      SimnetInstruction.burn,
      nft: nft,
      receipt: _receipt(absorption),
      receiptSignature: base64Decode(absorption.signature!),
      accounts: [
        (role: SimnetRole.wallet, address: chain.wallet),
        (role: SimnetRole.mint, address: nft.mockMint),
        (role: SimnetRole.escrow, address: await SimnetChain.escrowFor(nft)),
        (role: SimnetRole.program, address: SimnetChain.program),
        (role: SimnetRole.keeper, address: SimnetChain.keeper),
      ],
    );

    return settlement;
  }

  @override
  Future<PlayerState> buyWeapon(Weapon weapon) => _books.buyWeapon(weapon);

  @override
  Future<PlayerState> wield(Weapon weapon) => _books.wield(weapon);

  @override
  Future<PlayerState> buyOutfit(Outfit outfit) => _books.buyOutfit(outfit);

  @override
  Future<PlayerState> wear(Outfit outfit) => _books.wear(outfit);

  @override
  Future<PlayerState> attune(Art art) => _books.attune(art);

  @override
  Future<RiftSession> enterRift(RiftSite site) => _books.enterRift(site);

  /// Starts over with a new wallet and a new player, for the next person to
  /// pick the phone up.
  Future<void> reset() async {
    await _books.forget();
    await chain.reset();
  }

  /// The receipt for [absorption]: the player is the burner wallet.
  Uint8List _receipt(Absorption absorption) =>
      AbsorptionReceipt.message(playerId: chain.wallet, absorption: absorption);
}
