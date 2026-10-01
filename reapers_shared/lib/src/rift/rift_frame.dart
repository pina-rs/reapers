import '../hunt/hunt_event.dart';
import '../reaper_class.dart';
import 'rift_site.dart';

/// Where a rift is.
enum RiftPhase {
  /// Reapers are arriving and standing ready.
  gathering,

  /// The circle is breaking the ward.
  fight,

  /// The ward is broken and the circle is cutting.
  reap,

  /// Enough of the circle cut. The rift is shut and everyone is paid.
  sealed,

  /// The circle ran out of time, of reapers, or of cuts.
  failed;

  bool get isOver => this == sealed || this == failed;
}

/// One member of a circle, as everyone in it sees them.
final class RiftMember {
  const RiftMember({
    required this.name,
    required this.reaperClass,
    required this.level,
    required this.shade,
    required this.ready,
    required this.standing,
    required this.dealt,
    required this.finished,
    required this.cut,
  });

  factory RiftMember.fromJson(Map<String, dynamic> json) {
    final cut = json['cut'] as String?;

    return RiftMember(
      name: json['name'] as String,
      reaperClass: ReaperClass.fromCode(json['class'] as String),
      level: json['level'] as int,
      shade: json['shade'] as bool,
      ready: json['ready'] as bool,
      standing: json['standing'] as bool,
      dealt: (json['dealt'] as num).toDouble(),
      finished: json['finished'] as bool,
      cut: cut == null ? null : ReapGrade.values.byName(cut),
    );
  }

  final String name;

  final ReaperClass reaperClass;

  final int level;

  /// Whether this seat is filled by a shade rather than a person.
  final bool shade;

  /// Whether they have said they are ready to begin.
  final bool ready;

  /// Whether they are still in the fight: present, and not cast out.
  final bool standing;

  /// Ward this member has removed so far.
  final double dealt;

  /// Whether their part is over: cast out, or done with the reap.
  final bool finished;

  /// How cleanly they cut, or null if they have not, or could not.
  final ReapGrade? cut;

  Map<String, dynamic> toJson() => {
    'name': name,
    'class': reaperClass.code,
    'level': level,
    'shade': shade,
    'ready': ready,
    'standing': standing,
    'dealt': dealt,
    'finished': finished,
    'cut': cut?.name,
  };
}

/// A rift as one of its reapers sees it at one moment.
///
/// Whoever runs the rift sends these several times a second. Each one is the
/// whole picture, so a frame that never arrives is simply replaced by the
/// next.
final class RiftFrame {
  const RiftFrame({
    required this.site,
    required this.seed,
    required this.phase,
    required this.seat,
    required this.tick,
    required this.ticksLeft,
    required this.countingDown,
    required this.ward,
    required this.wardMax,
    required this.members,
    required this.xp,
    required this.anima,
  });

  /// Reads a frame written by [toJson].
  factory RiftFrame.fromJson(Map<String, dynamic> json) => RiftFrame(
    site: RiftSite.values.byName(json['site'] as String),
    seed: json['seed'] as int,
    phase: RiftPhase.values.byName(json['phase'] as String),
    seat: json['seat'] as int,
    tick: json['tick'] as int,
    ticksLeft: json['ticksLeft'] as int,
    countingDown: json['countingDown'] as bool,
    ward: (json['ward'] as num).toDouble(),
    wardMax: (json['wardMax'] as num).toDouble(),
    members: [
      for (final member in json['members'] as List)
        RiftMember.fromJson((member as Map).cast<String, dynamic>()),
    ],
    xp: json['xp'] as int,
    anima: json['anima'] as int,
  );

  final RiftSite site;

  /// The seed every reaper's own fight is built from.
  final int seed;

  final RiftPhase phase;

  /// Which of [members] is the reaper this frame was made for.
  final int seat;

  /// Ticks since the fight began. Zero while the circle gathers.
  final int tick;

  /// Ticks until the current phase runs out.
  final int ticksLeft;

  /// Whether the circle is ready and the count before the fight has begun.
  final bool countingDown;

  /// What is left of the ward the whole circle is breaking.
  final double ward;

  final double wardMax;

  /// The circle, in seat order.
  final List<RiftMember> members;

  /// Experience this reaper was paid. Zero until the rift is sealed.
  final int xp;

  /// Anima this reaper was paid. Zero until the rift is sealed.
  final int anima;

  RiftMember get you => members[seat];

  /// Cuts the circle needs before the rift counts as sealed: half of those
  /// still standing, rounded up.
  int get cutsNeeded =>
      cutsNeededFor(members.where((member) => member.standing).length);

  /// Cuts the circle has made so far.
  int get cuts => members.where((member) => member.cut != null).length;

  /// Cuts a circle with [standing] reapers left needs.
  static int cutsNeededFor(int standing) =>
      standing <= 1 ? 1 : (standing + 1) ~/ 2;

  Map<String, dynamic> toJson() => {
    'site': site.name,
    'seed': seed,
    'phase': phase.name,
    'seat': seat,
    'tick': tick,
    'ticksLeft': ticksLeft,
    'countingDown': countingDown,
    'ward': ward,
    'wardMax': wardMax,
    'members': [for (final member in members) member.toJson()],
    'xp': xp,
    'anima': anima,
  };

  /// A frame never changes, so a copy of it is itself. Serverpod asks for
  /// this of every value it carries between the server and the game.
  RiftFrame copyWith() => this;
}

/// What one reaper tells the rift about their own fight.
///
/// A report is the whole of that reaper's standing, not a change to it: the
/// damage is a running total and the outcome, once set, stays set. Reports
/// can therefore be repeated, dropped or arrive late without the rift losing
/// count.
final class RiftReport {
  const RiftReport({required this.ready, this.dealt = 0, this.outcome});

  /// Reads a report written by [toJson].
  factory RiftReport.fromJson(Map<String, dynamic> json) {
    final outcome = json['outcome'] as Map?;

    return RiftReport(
      ready: json['ready'] as bool,
      dealt: (json['dealt'] as num).toDouble(),
      outcome: outcome == null
          ? null
          : HuntOutcome.fromJson(outcome.cast<String, dynamic>()),
    );
  }

  /// Whether the reaper stands ready to begin.
  final bool ready;

  /// Ward this reaper has removed since the fight began.
  final double dealt;

  /// How this reaper's part ended, or null while it is still going. A reaper
  /// who cut the husk reports an absorbed outcome with the grade of the cut.
  final HuntOutcome? outcome;

  Map<String, dynamic> toJson() => {
    'ready': ready,
    'dealt': dealt,
    'outcome': outcome?.toJson(),
  };

  /// A report never changes, so a copy of it is itself. Serverpod asks for
  /// this of every value it carries between the server and the game.
  RiftReport copyWith() => this;
}
