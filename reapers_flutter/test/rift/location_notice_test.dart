import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/rift/place/location_notice.dart';
import 'package:reapers_flutter/rift/place/location_tracker.dart';
import 'package:reapers_flutter/rift/place/whereabouts.dart';
import 'package:reapers_shared/reapers_shared.dart';

final class _Phone implements Whereabouts {
  final readings = StreamController<GeoFix>.broadcast();
  @override
  Stream<GeoFix> watch() => readings.stream;
}

void main() {
  testWidgets(
    'permission refusal is explained and retry recovers with a fresh fix',
    (tester) async {
      final phone = _Phone();
      final tracker = LocationTracker(source: phone, share: (_) async {})
        ..start();
      await tester.pumpWidget(
        MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: Scaffold(
            body: ListenableBuilder(
              listenable: tracker,
              builder: (context, _) => LocationNotice(tracker: tracker),
            ),
          ),
        ),
      );
      final l10n = await AppLocalizations.delegate.load(const Locale('en'));
      phone.readings.addError(const LocationFailure(LocationIssue.denied));
      await tester.pump();
      expect(find.text(l10n.locationDenied), findsOneWidget);
      await tester.tap(find.byKey(const Key('location-recover')));
      await tester.pump();
      phone.readings.add(
        GeoFix(
          at: const GeoPoint(51.5, -.08),
          accuracy: 4,
          observedAt: DateTime.now().toUtc(),
        ),
      );
      await tester.pump();
      expect(find.byKey(const Key('location-notice')), findsNothing);
      tracker.dispose();
      await phone.readings.close();
    },
  );
}
