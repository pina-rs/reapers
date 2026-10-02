import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/sound/score.dart';
import 'package:reapers_flutter/sound/score_engine.dart';
import 'package:reapers_flutter/sound/soundtrack.dart';

/// An engine that writes down what it is told.
final class _Listening implements ScoreEngine {
  final told = <String>[];

  @override
  Future<void> play(Score piece, Mix mix, {required Duration fade}) async =>
      told.add('play ${piece.name} at ${mix.volume}');

  @override
  void shape(Mix mix) => told.add('shape ${mix.volume}/${mix.veil}');

  @override
  Future<void> silence({required Duration fade}) async => told.add('silence');

  @override
  void hold({required bool held}) => told.add(held ? 'hold' : 'carry on');

  @override
  void dispose() => told.add('dispose');
}

void main() {
  test('a piece is brought in once, however often it is asked for', () {
    final engine = _Listening();
    final soundtrack = Soundtrack(engine, enabled: true)
      ..play(Score.sanctum)
      ..play(Score.sanctum);

    expect(engine.told, ['play sanctum at 0.55']);
    expect(soundtrack.piece, Score.sanctum);
  });

  test('another piece takes over from the one playing', () {
    final engine = _Listening();

    Soundtrack(engine, enabled: true)
      ..play(Score.sanctum)
      ..play(Score.hunt, mix: Mix.waiting);

    expect(engine.told, ['play sanctum at 0.55', 'play hunt at 0.22']);
  });

  test('the engine is only told of changes that would be heard', () {
    final engine = _Listening();

    Soundtrack(engine, enabled: true)
      ..play(Score.hunt, mix: const Mix(volume: 0.5, veil: 0))
      ..shape(const Mix(volume: 0.503, veil: 0))
      ..shape(const Mix(volume: 0.8, veil: 0))
      ..shape(const Mix(volume: 0.8, veil: 0));

    expect(engine.told, ['play hunt at 0.5', 'shape 0.8/0.0']);
  });

  test('asking for the piece already playing reshapes it', () {
    final engine = _Listening();

    Soundtrack(engine, enabled: true)
      ..play(Score.hunt, mix: Mix.waiting)
      ..play(Score.hunt, mix: const Mix(volume: 0.8, veil: 0));

    expect(engine.told, ['play hunt at 0.22', 'shape 0.8/0.0']);
  });

  test('with music off the engine hears nothing', () {
    final engine = _Listening();
    final soundtrack = Soundtrack(engine, enabled: false)
      ..play(Score.hunt)
      ..shape(const Mix(volume: 1, veil: 0));

    expect(engine.told, isEmpty);
    expect(soundtrack.piece, Score.hunt, reason: 'it still knows what belongs');
  });

  test('turned back on, the music comes back as it should be now', () {
    final engine = _Listening();
    final soundtrack = Soundtrack(engine, enabled: true)..play(Score.sanctum);

    soundtrack
      ..setEnabled(enabled: false)
      ..play(Score.hunt, mix: const Mix(volume: 0.8, veil: 0))
      ..setEnabled(enabled: true);

    expect(engine.told, [
      'play sanctum at 0.55',
      'silence',
      'play hunt at 0.8',
    ]);
  });

  test('off the screen the music is held, and carries on after', () {
    final engine = _Listening();

    Soundtrack(engine, enabled: true)
      ..hold(held: true)
      ..hold(held: false);

    expect(engine.told, ['hold', 'carry on']);
  });
}
