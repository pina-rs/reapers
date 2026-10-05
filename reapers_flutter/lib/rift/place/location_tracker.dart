import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../../src/backend/game_backend.dart';
import 'whereabouts.dart';

/// One foreground rift's location subscription and latest-only upload queue.
/// Stop it when the app is inactive, and dispose it when the rift closes.
/// Uploads never overlap; a stop clears the server after an in-flight fix.
final class LocationTracker extends ChangeNotifier {
  LocationTracker({
    required this._source,
    required this._share,
    DateTime Function()? now,
  }) : _now = now ?? DateTime.now;

  final Whereabouts _source;
  final Future<void> Function(GeoFix?) _share;
  final DateTime Function() _now;
  StreamSubscription<GeoFix>? _subscription;
  Timer? _timer;
  GeoFix? _fix;
  GeoFix? _queued;
  DateTime? _sentAt;
  DateTime? _queuedAt;
  bool _hasQueued = false;
  bool _sending = false;
  bool _active = false;
  bool _disposed = false;
  int _generation = 0;

  LocationIssue? _issue;
  bool _sharingFailed = false;
  LocationIssue? get issue => _issue;
  bool get sharingFailed => _sharingFailed;
  GeoFix? get fix => _fix?.isUsableAt(_now()) == true ? _fix : null;
  bool get stale => _fix != null && !_fix!.isFreshAt(_now());
  bool get inaccurate => _fix != null && !_fix!.isPrecise;

  void start() {
    if (_disposed || _active) return;
    _active = true;
    _issue = null;
    final generation = ++_generation;
    _subscription = _source.watch().listen(
      (found) {
        if (!_active || generation != _generation) return;
        if (_fix != null && !found.observedAt.isAfter(_fix!.observedAt)) return;
        _fix = found;
        _issue = null;
        _tick();
      },
      onError: (Object error, StackTrace stack) {
        if (!_active || generation != _generation) return;
        _issue = error is LocationFailure
            ? error.issue
            : LocationIssue.unavailable;
        _fix = null;
        _queue(null);
        if (error is! LocationFailure) {
          FlutterError.reportError(
            FlutterErrorDetails(
              exception: error,
              stack: stack,
              library: 'Reapers location',
            ),
          );
        }
        _notify();
      },
    );
    _timer = Timer.periodic(const Duration(seconds: 1), (_) => _tick());
    _notify();
  }

  void stop() {
    if (!_active) return;
    _active = false;
    _generation++;
    _timer?.cancel();
    _timer = null;
    unawaited(_subscription?.cancel());
    _subscription = null;
    _fix = null;
    _sentAt = null;
    _queuedAt = null;
    _queue(null);
    _notify();
  }

  void retry() {
    stop();
    start();
  }

  void _tick() {
    final usable = fix;
    if (usable != null &&
        (_queuedAt == null || usable.observedAt.isAfter(_queuedAt!)) &&
        (_sentAt == null ||
            _now().difference(_sentAt!) >= RiftRules.locationReportInterval)) {
      _sentAt = _now();
      _queuedAt = usable.observedAt;
      _queue(usable);
    } else if (stale && _sentAt != null) {
      _sentAt = null;
      _queue(null);
    }
    _notify();
  }

  void _queue(GeoFix? fix) {
    _queued = fix;
    _hasQueued = true;
    if (!_sending) unawaited(_send());
  }

  Future<void> _send() async {
    _sending = true;
    while (_hasQueued) {
      final next = _queued;
      _hasQueued = false;
      try {
        await _share(next);
        _sharingFailed = false;
      } on BackendUnavailable {
        _sharingFailed = true;
      } on RuleBreach {
        _sharingFailed = true;
      } catch (error, stack) {
        _sharingFailed = true;
        FlutterError.reportError(
          FlutterErrorDetails(
            exception: error,
            stack: stack,
            library: 'Reapers location upload',
          ),
        );
      }
      _notify();
    }
    _sending = false;
  }

  void _notify() {
    if (!_disposed) notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    stop();
    super.dispose();
  }
}
