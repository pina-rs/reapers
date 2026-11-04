import 'dart:math';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/simnet/simnet_backend.dart';
import 'package:reapers_flutter/simnet/simnet_chain.dart';
import 'package:reapers_flutter/src/backend/local_backend.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:solana_kit/solana_kit.dart' as kit;

const _saint = Nft(collection: NftCollection.degenApes, serial: 1);

void main() {
  late SharedPreferences preferences;
  late DateTime now;

  SimnetChain chain() => SimnetChain.open(
    preferences,
    clock: () => now,
    random: Random(3),
    latency: Duration.zero,
  );

  SimnetBackend keeper(SimnetChain chain) =>
      SimnetBackend(preferences, chain, clock: () => now, random: Random(5));

  /// Plays the one loose husk with the shared bot and settles it.
  Future<HuntSettlement> hunt(SimnetBackend backend) async {
    final state = await backend.load();
    final sighting = state.sightings.single;
    final bot = HuntBot(
      HuntSim(
        seed: sighting.seed,
        vessel: sighting.nft.collection.vessel,
        ward: sighting.nft.rarity.ward,
        stats: HuntStats.derive(state.loadout),
      ),
    )..play();

    return backend.settleHunt(sighting.id, HuntLog(bot.recorded));
  }

  setUp(() async {
    SharedPreferences.setMockInitialValues({});
    preferences = await SharedPreferences.getInstance();
    now = DateTime.utc(2026, 11, 4, 12);
  });

  test('binding a reaper airdrops the starting SOL into a burner wallet '
      'with a real address', () async {
    final simnet = chain();
    final state = await keeper(simnet).takeOath();
    final airdrop = simnet.transactions.single;

    expect(kit.isAddress(simnet.wallet), isTrue);
    expect(airdrop.instruction, SimnetInstruction.airdrop);
    expect(airdrop.lamports, state.lamports);
    expect(airdrop.account(SimnetRole.wallet), simnet.wallet);
    expect(
      kit.getBase58Encoder().encode(airdrop.signature),
      hasLength(64),
      reason: 'an Ed25519 signature, shown in base58',
    );
  });

  test('a purchase, a release and an absorption are a buy, an escrow and a '
      'burn that carries a receipt the keeper signed', () async {
    final simnet = chain();
    final backend = keeper(simnet);

    await backend.takeOath();
    await backend.acquire(_saint);
    await backend.release(_saint);
    now = now.add(const Duration(minutes: 1));

    final settlement = await hunt(backend);

    expect(settlement.outcome.absorbed, isTrue);
    expect(simnet.transactions.map((transaction) => transaction.instruction), [
      SimnetInstruction.airdrop,
      SimnetInstruction.buy,
      SimnetInstruction.escrow,
      SimnetInstruction.burn,
    ]);

    final [_, buy, escrow, burn] = simnet.transactions;
    final escrowAddress = await SimnetChain.escrowFor(_saint);

    expect(buy.lamports, _saint.rarity.lamports);
    expect(buy.nft, _saint);
    expect(escrow.account(SimnetRole.escrow), escrowAddress);
    expect(kit.isAddress(escrowAddress), isTrue);
    expect(burn.account(SimnetRole.escrow), escrowAddress);
    expect(burn.slot, greaterThan(escrow.slot), reason: 'a minute later');

    final absorption = settlement.state.codex.single;

    expect(absorption.signature, isNotNull, reason: 'the codex keeps it');
    expect(
      burn.receipt,
      AbsorptionReceipt.message(
        playerId: simnet.wallet,
        absorption: absorption,
      ),
    );
    expect(burn.receiptVerifies, isTrue);
  });

  test('a receipt changed after signing fails the check', () async {
    final simnet = chain();
    final backend = keeper(simnet);

    await backend.takeOath();
    await backend.acquire(_saint);
    await backend.release(_saint);
    now = now.add(const Duration(minutes: 1));
    await hunt(backend);

    final burn = simnet.transactions.last;
    final forged = Uint8List.fromList(burn.receipt!)
      ..[73] = Feeding.feast.index;

    expect(
      SimnetTransaction(
        signature: burn.signature,
        slot: burn.slot,
        at: burn.at,
        instruction: burn.instruction,
        accounts: burn.accounts,
        nft: burn.nft,
        receipt: forged,
        receiptSignature: burn.receiptSignature,
      ).receiptVerifies,
      isFalse,
    );
  });

  test('a refused request posts nothing', () async {
    final simnet = chain();
    final backend = keeper(simnet);

    await backend.takeOath();
    await backend.acquire(_saint);

    await expectLater(
      backend.acquire(_saint),
      throwsA(isA<RuleBreach>()),
    );
    expect(simnet.transactions, hasLength(2));
  });

  test('the wallet and its ledger survive a restart', () async {
    final first = chain();

    await keeper(first).takeOath();

    final reopened = chain();

    expect(reopened.wallet, first.wallet);
    expect(
      reopened.transactions.map((transaction) => transaction.signature),
      first.transactions.map((transaction) => transaction.signature),
    );
  });

  test('starting over makes a new wallet and a new player, and leaves the '
      'offline game alone', () async {
    final offline = LocalBackend(preferences, clock: () => now);
    await offline.takeOath();
    final offlineSave = preferences.getString(LocalBackend.defaultStorageKey);

    final simnet = chain();
    final backend = keeper(simnet);
    final before = simnet.wallet;

    await backend.takeOath();
    await backend.reset();

    expect(simnet.wallet, isNot(before));
    expect(simnet.transactions, isEmpty);
    expect((await backend.load()).oathTaken, isFalse);
    expect(
      preferences.getString(LocalBackend.defaultStorageKey),
      offlineSave,
    );
  });
}
