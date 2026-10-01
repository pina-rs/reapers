import 'reaper_class.dart';
import 'reaper_names.dart';
import 'seeded_random.dart';

/// A rare mark the shuffle sometimes leaves on a reaper.
///
/// An omen changes how a reaper is coloured and nothing about how it hunts.
enum Omen {
  none(weight: 80),

  /// Drained to bone and ash.
  ashen(weight: 12),

  /// Tarnished gold worked into every edge.
  gilded(weight: 6),

  /// Near-black, with the accent burning at its brightest.
  eclipsed(weight: 2);

  const Omen({required this.weight});

  /// How often the shuffle lands on this omen, out of 100.
  final int weight;
}

/// What drifts around a reaper while it waits in the sanctum.
enum Aura { motes, embers, mist, wisps }

/// Everything that makes one reaper look unlike another.
///
/// A genome is drawn once, by [ReaperGenome.shuffle], and then stored as its
/// [code]. The code is the permanent record: it does not depend on the seed
/// or on the shuffle's tables staying the same, so a reaper minted today
/// looks the same after the tables grow.
final class ReaperGenome {
  const ReaperGenome({
    required this.reaperClass,
    required this.head,
    required this.mantle,
    required this.robe,
    required this.palette,
    required this.omen,
    required this.aura,
    required this.stature,
    required this.gauntness,
    required this.gaze,
    required this.tatter,
    required this.givenName,
    required this.epithet,
  });

  /// Draws a reaper from [seed]. The same seed always draws the same reaper.
  factory ReaperGenome.shuffle(int seed) {
    final random = SeededRandom(seed);
    final reaperClass = random.pick(ReaperClass.values);

    return ReaperGenome(
      reaperClass: reaperClass,
      head: random.nextInt(headVariants),
      mantle: random.nextInt(mantleVariants),
      robe: random.nextInt(robeVariants),
      palette: random.nextInt(paletteVariants),
      omen:
          Omen.values[random.pickWeighted([
            for (final omen in Omen.values) omen.weight,
          ])],
      aura: random.pick(Aura.values),
      stature: random.nextInt(traitScale + 1),
      gauntness: random.nextInt(traitScale + 1),
      gaze: random.nextInt(traitScale + 1),
      tatter: random.nextInt(traitScale + 1),
      givenName: random.nextInt(reaperGivenNames.length),
      epithet: random.nextInt(reaperEpithets[reaperClass]!.length),
    );
  }

  /// Reads a genome back from its [code].
  ///
  /// Throws a [FormatException] when the code is malformed or names a part
  /// that does not exist, so a corrupted record is never drawn as a reaper.
  factory ReaperGenome.parse(String code) {
    final fields = code.split('-');

    if (fields.length != _fieldCount || fields.first != _version) {
      throw FormatException('Not a reaper genome', code);
    }

    final reaperClass = ReaperClass.fromCode(fields[1]);

    return ReaperGenome(
      reaperClass: reaperClass,
      head: _field(fields[2], 'h', headVariants),
      mantle: _field(fields[3], 'm', mantleVariants),
      robe: _field(fields[4], 'r', robeVariants),
      palette: _field(fields[5], 'p', paletteVariants),
      omen: Omen.values[_field(fields[6], 'o', Omen.values.length)],
      aura: Aura.values[_field(fields[7], 'a', Aura.values.length)],
      stature: _field(fields[8], 's', traitScale + 1),
      gauntness: _field(fields[9], 'g', traitScale + 1),
      gaze: _field(fields[10], 'z', traitScale + 1),
      tatter: _field(fields[11], 't', traitScale + 1),
      givenName: _field(fields[12], 'n', reaperGivenNames.length),
      epithet: _field(fields[13], 'e', reaperEpithets[reaperClass]!.length),
    );
  }

  /// The reaper lent to someone who plays without taking the oath.
  static const borrowed = ReaperGenome(
    reaperClass: ReaperClass.harvester,
    head: 0,
    mantle: 0,
    robe: 0,
    palette: 0,
    omen: Omen.none,
    aura: Aura.mist,
    stature: 500,
    gauntness: 500,
    gaze: 400,
    tatter: 500,
    givenName: 0,
    epithet: 0,
  );

  /// How many head shapes each class's head family holds.
  static const headVariants = 3;

  /// How many shoulder shapes each class's shoulder family holds.
  static const mantleVariants = 3;

  /// How many robe hems every class shares.
  static const robeVariants = 4;

  /// How many colourways each class has.
  static const paletteVariants = 3;

  /// Continuous traits are stored as whole numbers from 0 to this value, so a
  /// code round-trips exactly.
  static const traitScale = 1000;

  static const _version = 'R1';
  static const _fieldCount = 14;

  final ReaperClass reaperClass;

  /// Which head of the class's head family.
  final int head;

  /// Which shoulders of the class's shoulder family.
  final int mantle;

  /// Which hem.
  final int robe;

  /// Which of the class's colourways.
  final int palette;

  final Omen omen;

  final Aura aura;

  /// How tall the reaper stands, from 0 (shortest) to [traitScale].
  final int stature;

  /// How drawn and narrow the reaper's body is.
  final int gauntness;

  /// How large the reaper's one light burns.
  final int gaze;

  /// How ragged the hem is.
  final int tatter;

  /// Index into [reaperGivenNames].
  final int givenName;

  /// Index into the class's list in [reaperEpithets].
  final int epithet;

  /// The reaper's full name, such as "Vesper of the Last Furrow".
  String get name =>
      '${reaperGivenNames[givenName]} ${reaperEpithets[reaperClass]![epithet]}';

  /// The permanent, compact record of this genome.
  String get code => [
    _version,
    reaperClass.code,
    'h$head',
    'm$mantle',
    'r$robe',
    'p$palette',
    'o${omen.index}',
    'a${aura.index}',
    's$stature',
    'g$gauntness',
    'z$gaze',
    't$tatter',
    'n$givenName',
    'e$epithet',
  ].join('-');

  /// A continuous trait as a fraction from 0 to 1.
  static double fraction(int trait) => trait / traitScale;

  @override
  bool operator ==(Object other) => other is ReaperGenome && other.code == code;

  @override
  int get hashCode => code.hashCode;

  @override
  String toString() => 'ReaperGenome($code)';

  static int _field(String field, String prefix, int limit) {
    if (!field.startsWith(prefix)) {
      throw FormatException('Expected a "$prefix" field', field);
    }

    final value = int.tryParse(field.substring(prefix.length));

    if (value == null || value < 0 || value >= limit) {
      throw FormatException('Out of range', field);
    }

    return value;
  }
}
