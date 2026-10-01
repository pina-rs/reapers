import 'seeded_random.dart';
import 'vessel.dart';

/// How scarce an NFT is inside its collection, and so how hard its husk
/// fights and how much it is worth to absorb.
enum Rarity {
  common(ward: 280, xp: 40, anima: 16, lamports: 20000000),
  uncommon(ward: 420, xp: 70, anima: 28, lamports: 45000000),
  rare(ward: 760, xp: 120, anima: 48, lamports: 110000000),
  epic(ward: 1400, xp: 210, anima: 85, lamports: 260000000);

  const Rarity({
    required this.ward,
    required this.xp,
    required this.anima,
    required this.lamports,
  });

  /// The ward a husk of this rarity starts a hunt with.
  final int ward;

  /// Experience for absorbing it with a clean reap.
  final int xp;

  /// Anima for absorbing it with a clean reap.
  final int anima;

  /// The stand-in market price, in lamports (a billionth of one SOL).
  final int lamports;
}

/// A collection whose NFTs can be released into the veil and hunted.
///
/// These four are invented stand-ins. They exist so the hunt can be played
/// before any real collection is wired in; see docs/nfts.md.
enum NftCollection {
  tinSaints(
    title: 'Tin Saints',
    vessel: Vessel.frameWraith,
    rarities: _standardRarities,
  ),
  mothlightParade(
    title: 'Mothlight Parade',
    vessel: Vessel.lanternShade,
    rarities: _standardRarities,
  ),
  staticChoir(
    title: 'Static Choir',
    vessel: Vessel.shardSwarm,
    rarities: _standardRarities,
  ),
  brineHounds(
    title: 'Brine Hounds',
    vessel: Vessel.scrollEel,
    rarities: _standardRarities,
  );

  const NftCollection({
    required this.title,
    required this.vessel,
    required this.rarities,
  });

  static const _standardRarities = [
    Rarity.common,
    Rarity.common,
    Rarity.common,
    Rarity.common,
    Rarity.uncommon,
    Rarity.uncommon,
    Rarity.rare,
    Rarity.epic,
  ];

  /// The collection's name, as its creators would have published it.
  final String title;

  /// The body every husk from this collection wears.
  final Vessel vessel;

  /// The rarity of each numbered item, in serial order.
  final List<Rarity> rarities;

  /// How many absorbed items complete the collection's set.
  static const setSize = 3;
}

/// One NFT a player can own, release and hunt.
final class Nft {
  const Nft({required this.collection, required this.serial});

  /// Reads an NFT back from its [id], such as `tinSaints:3`.
  factory Nft.parse(String id) {
    final parts = id.split(':');
    final serial = parts.length == 2 ? int.tryParse(parts[1]) : null;

    if (serial == null) {
      throw FormatException('Not an NFT id', id);
    }

    final collection = NftCollection.values.byName(parts[0]);

    if (serial < 1 || serial > collection.rarities.length) {
      throw FormatException('No such item in ${collection.title}', id);
    }

    return Nft(collection: collection, serial: serial);
  }

  final NftCollection collection;

  /// The item's number inside its collection, starting at 1.
  final int serial;

  String get id => '${collection.name}:$serial';

  Rarity get rarity => collection.rarities[serial - 1];

  /// The name shown on a market listing, such as "Tin Saints #3".
  String get title => '${collection.title} #$serial';

  /// The file stem of the item's image.
  String get imageKey => '${collection.name}_$serial';

  /// A stand-in for the item's on-chain mint address.
  ///
  /// It has the shape of a Solana address so the interface can be laid out
  /// honestly, and it belongs to no real account.
  String get mockMint {
    const alphabet =
        '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
    final random = SeededRandom(hashText('mint:$id'));

    return [
      for (var index = 0; index < 44; index++)
        alphabet[random.nextInt(alphabet.length)],
    ].join();
  }

  @override
  bool operator ==(Object other) =>
      other is Nft && other.collection == collection && other.serial == serial;

  @override
  int get hashCode => Object.hash(collection, serial);

  @override
  String toString() => 'Nft($id)';
}

/// Every NFT the stand-in market lists.
final List<Nft> marketListings = [
  for (final collection in NftCollection.values)
    for (var serial = 1; serial <= collection.rarities.length; serial++)
      Nft(collection: collection, serial: serial),
];

/// Lamports in one SOL.
const lamportsPerSol = 1000000000;
