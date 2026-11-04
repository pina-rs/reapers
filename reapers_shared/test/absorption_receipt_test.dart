import 'dart:typed_data';

import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

void main() {
  const playerId = '550e8400-e29b-41d4-a716-446655440000';
  final at = DateTime.utc(2026, 10, 1, 12);
  final absorption = Absorption(
    nft: const Nft(collection: NftCollection.thugbirdz, serial: 3),
    grade: ReapGrade.perfect,
    feeding: Feeding.rich,
    xp: 0x01020304,
    anima: 77,
    nonce: 0xCAFEF00D,
    at: at,
  );

  test('a receipt is laid out as the documentation says', () {
    final message = AbsorptionReceipt.message(
      playerId: playerId,
      absorption: absorption,
    );
    final data = ByteData.sublistView(message);

    expect(message, hasLength(AbsorptionReceipt.length));
    expect(String.fromCharCodes(message.sublist(0, 8)), 'RPRSABS1');
    expect(message.sublist(8, 40), isNot(everyElement(0)));
    expect(message.sublist(40, 72), isNot(message.sublist(8, 40)));
    expect(data.getUint8(72), ReapGrade.perfect.index);
    expect(data.getUint8(73), Feeding.rich.index);
    expect(message.sublist(74, 78), [4, 3, 2, 1], reason: 'little-endian');
    expect(data.getUint32(78, Endian.little), 77);
    expect(data.getInt64(82, Endian.little), at.millisecondsSinceEpoch ~/ 1000);
    expect(data.getUint64(90, Endian.little), 0xCAFEF00D);
  });

  test('the player field is the SHA-256 of the player id', () {
    final message = AbsorptionReceipt.message(
      playerId: playerId,
      absorption: absorption,
    );

    // The first bytes of SHA-256 over the UTF-8 of the id above, pinned so a
    // change of hash or encoding cannot pass unnoticed.
    expect(message.sublist(8, 12), [0xa3, 0xa9, 0xe1, 0xed]);
    expect(
      AbsorptionReceipt.message(
        playerId: 'someone else',
        absorption: absorption,
      ).sublist(8, 40),
      isNot(message.sublist(8, 40)),
    );
  });
}
