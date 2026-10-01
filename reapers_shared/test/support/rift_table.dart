import 'package:reapers_shared/reapers_shared.dart';

/// A reaper of [reaperClass] at [level], with their class's weapon at the
/// best temper that level allows.
RiftEntrant entrant({
  String name = 'Vesper',
  ReaperClass reaperClass = ReaperClass.harvester,
  int level = 1,
}) => RiftEntrant(
  name: name,
  loadout: HuntLoadout(
    reaperClass: reaperClass,
    level: level,
    weapon: Weapon(
      reaperClass.weapon,
      Temper.values.lastWhere((temper) => temper.levelRequired <= level),
    ),
  ),
);

/// A person who plays as carefully as [HuntBot.human] does.
HuntBot careful(HuntSim sim, int person) => HuntBot.human(sim);

/// A rift played out in one place: the room, and a seat for every person in
/// it, each played by the bot [play] gives them.
///
/// This is what a host does, the game alone or the server for many, with the
/// network taken out: every tick each person's seat is synced, stepped and
/// reported, and the room is moved on.
final class RiftTable {
  RiftTable({
    required List<RiftEntrant> reapers,
    RiftSite site = RiftSite.londonBridge,
    int seed = 11,
    HuntBot Function(HuntSim sim, int person) play = careful,
  }) : room = RiftRoom(site: site, seed: seed),
       _reapers = reapers,
       _play = play {
    for (final reaper in reapers) {
      seats.add(room.join(reaper));
    }
  }

  final RiftRoom room;
  final List<RiftEntrant> _reapers;
  final HuntBot Function(HuntSim sim, int person) _play;

  /// The room seat of each person, in the order they were given.
  final seats = <int>[];

  final _fights = <RiftSeat>[];
  final _gone = <int>{};
  final _bots = <HuntBot>[];

  /// Has everyone stand ready and waits out the count.
  void begin() {
    for (final seat in seats) {
      room.report(seat, const RiftReport(ready: true));
    }

    room.advance(RiftRules.countdownTicks + 1);

    for (final reaper in _reapers) {
      final fight = RiftSeat(
        seed: room.seed,
        site: room.site,
        wardMax: room.wardMax,
        loadout: reaper.loadout,
      );

      _fights.add(fight);
      _bots.add(_play(fight.sim, _bots.length));
    }
  }

  /// Has person [index] walk away from the rift.
  void walkAway(int index) {
    _gone.add(index);
    room.leave(seats[index]);
  }

  /// Plays one tick for everyone still here.
  void step() {
    for (var index = 0; index < seats.length; index++) {
      if (_gone.contains(index)) {
        continue;
      }

      final fight = _fights[index];
      final frame = room.frameFor(seats[index]);

      fight
        ..sync(ward: frame.ward, counted: frame.you.dealt)
        ..step(_bots[index].decide());
      room.report(seats[index], fight.report);
    }

    room.advance(1);
  }

  /// Plays until the rift is over.
  void play() {
    begin();

    while (!room.phase.isOver) {
      step();
    }
  }
}
