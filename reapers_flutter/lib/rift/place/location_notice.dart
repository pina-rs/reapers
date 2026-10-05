import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:geolocator/geolocator.dart';

import '../../l10n/l10n.dart';
import '../../theme/veil.dart';
import '../../widgets/ledger.dart';
import 'location_tracker.dart';
import 'whereabouts.dart';

/// Permission recovery stays in the rift, where its purpose is clear.
class LocationNotice extends HookWidget {
  const LocationNotice({super.key, required this.tracker});
  final LocationTracker tracker;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final message = switch (tracker.issue) {
      LocationIssue.denied => l10n.locationDenied,
      LocationIssue.blocked => l10n.locationBlocked,
      LocationIssue.services => l10n.locationServices,
      LocationIssue.unsupported => l10n.locationUnsupported,
      LocationIssue.unavailable => l10n.locationUnavailable,
      null when tracker.inaccurate => l10n.locationInaccurate,
      null when tracker.stale => l10n.locationStale,
      null when tracker.sharingFailed => l10n.locationSharingFailed,
      null => null,
    };
    if (message == null) return const SizedBox.shrink();

    final settings =
        tracker.issue == LocationIssue.blocked ||
        tracker.issue == LocationIssue.services;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const SizedBox(height: 8),
        Semantics(
          liveRegion: true,
          child: Text(
            message,
            key: const Key('location-notice'),
            style: Veil.body(color: Veil.ash, size: 13),
          ),
        ),
        if (tracker.issue != LocationIssue.unsupported)
          LedgerButton(
            key: const Key('location-recover'),
            label: settings ? l10n.locationSettings : l10n.commonRetry,
            quiet: true,
            onPressed: () async {
              if (settings) {
                final opened = tracker.issue == LocationIssue.services
                    ? await Geolocator.openLocationSettings()
                    : await Geolocator.openAppSettings();
                if (!opened && context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text(l10n.locationSettingsFailed)),
                  );
                }
              } else {
                tracker.retry();
              }
            },
          ),
      ],
    );
  }
}
