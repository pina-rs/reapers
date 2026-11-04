import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/theme/veil.dart';
import 'package:reapers_flutter/widgets/ledger.dart';
import 'package:reapers_flutter/widgets/sketch.dart';
import 'package:skribble/skribble.dart';

void main() {
  test('a sketch frame inks paths without taking any layout room', () {
    final decoration = Sketch.box(colour: Veil.bone, seed: 3);
    final canvas = TestRecordingCanvas();

    decoration.createBoxPainter().paint(
      canvas,
      Offset.zero,
      const ImageConfiguration(size: Size(160, 80)),
    );

    expect(canvas.invocations, isNotEmpty);
    expect(
      canvas.invocations.where(
        (invocation) => invocation.invocation.memberName == #drawPath,
      ),
      isNotEmpty,
      reason: 'the rough engine draws the edge as a path',
    );
  });

  testWidgets('the frame paints over its child without resizing it', (
    tester,
  ) async {
    await tester.pumpWidget(
      const Directionality(
        textDirection: TextDirection.ltr,
        child: Center(
          child: SizedBox(
            width: 160,
            height: 80,
            child: SketchFrame(
              colour: Veil.bone,
              seed: 3,
              child: SizedBox.expand(),
            ),
          ),
        ),
      ),
    );

    expect(tester.getSize(find.byType(SketchFrame)), const Size(160, 80));
    expect(tester.takeException(), isNull);
  });

  testWidgets('a ledger button carries the same hand-drawn rule', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: Veil.theme(Veil.bone),
        home: Scaffold(
          body: LedgerButton(
            label: 'Release',
            detail: '0.02 SOL',
            onPressed: () {},
          ),
        ),
      ),
    );

    final button = tester.widget<AnimatedContainer>(
      find.byType(AnimatedContainer),
    );
    final decoration = button.foregroundDecoration;

    expect(decoration, isA<RoughBoxDecoration>());
    expect(
      (decoration! as RoughBoxDecoration).drawConfig!.seed,
      Sketch.seedFor('Release'),
    );
    expect(find.text('RELEASE'), findsOneWidget);
  });

  test('the same rule and seed always build the same box', () {
    final first = Sketch.box(seed: 9);
    final second = Sketch.box(seed: 9);

    expect(first.drawConfig, second.drawConfig);
    expect(first.seed, second.seed);
    expect(Sketch.seedFor('a'), Sketch.seedFor('a'));
    expect(Sketch.seedFor('a'), isNot(Sketch.seedFor('b')));
  });
}
