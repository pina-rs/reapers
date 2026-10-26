import 'dart:async';

import 'package:camera_platform_interface/camera_platform_interface.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/camera/hunt_camera.dart';

/// CameraX owns one preview globally. Opening over a live preview or
/// releasing the wrong controller fails, just as the native plugin does.
class _CameraDevice extends CameraPlatform {
  int created = 0;
  int closed = 0;
  int? active;
  Completer<void>? openingGate;
  Completer<void>? closingGate;
  final opening = Completer<void>();
  final closing = Completer<void>();
  final errors = StreamController<CameraErrorEvent>.broadcast();

  @override
  Future<List<CameraDescription>> availableCameras() async => const [
    CameraDescription(
      name: 'rear',
      lensDirection: CameraLensDirection.back,
      sensorOrientation: 90,
    ),
  ];

  @override
  Stream<DeviceOrientationChangedEvent> onDeviceOrientationChanged() =>
      const Stream.empty();

  @override
  Future<int> createCameraWithSettings(
    CameraDescription description,
    MediaSettings? settings,
  ) async {
    if (active != null) {
      throw StateError('The previous preview has not closed');
    }

    active = ++created;
    return created;
  }

  @override
  Future<void> initializeCamera(
    int cameraId, {
    ImageFormatGroup imageFormatGroup = ImageFormatGroup.unknown,
  }) async {
    if (!opening.isCompleted) opening.complete();
    await openingGate?.future;
  }

  @override
  Stream<CameraInitializedEvent> onCameraInitialized(int cameraId) =>
      Stream.value(
        CameraInitializedEvent(
          cameraId,
          1280,
          720,
          ExposureMode.auto,
          true,
          FocusMode.auto,
          true,
        ),
      );

  @override
  Stream<CameraErrorEvent> onCameraError(int cameraId) => errors.stream;

  @override
  Future<void> dispose(int cameraId) async {
    if (cameraId != active) {
      throw StateError('Released the wrong preview');
    }

    if (!closing.isCompleted) closing.complete();
    await closingGate?.future;
    active = null;
    closed++;
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  late CameraPlatform previous;
  late _CameraDevice device;

  setUp(() {
    previous = CameraPlatform.instance;
    device = _CameraDevice();
    CameraPlatform.instance = device;
  });

  tearDown(() {
    CameraPlatform.instance = previous;
  });

  test('the next hunt waits until the previous preview has closed', () async {
    final first = HuntCamera();
    final second = HuntCamera();
    final gate = Completer<void>();
    device.closingGate = gate;
    addTearDown(() async {
      if (!gate.isCompleted) gate.complete();
      await first.dispose();
      await second.dispose();
    });

    await first.open();
    final next = second.open();
    final closing = first.dispose();
    await device.closing.future;

    expect(device.created, 1);
    expect(second.controller.value, isNull);
    expect(first.dispose(), same(closing));

    gate.complete();
    await closing;
    await next;

    expect(second.controller.value!.value.isInitialized, isTrue);
    expect(device.created, 2);
    expect(device.closed, 1);
    await second.dispose();
    expect(device.closed, 2);
  });

  test('leaving during initialization releases before the next hunt', () async {
    final first = HuntCamera();
    final second = HuntCamera();
    final gate = Completer<void>();
    device.openingGate = gate;
    addTearDown(() async {
      if (!gate.isCompleted) gate.complete();
      await first.dispose();
      await second.dispose();
    });

    final opening = first.open();
    await device.opening.future;
    final closing = first.dispose();
    final next = second.open();
    gate.complete();
    await opening;
    await closing;
    await next;

    expect(device.closed, 1);
    expect(second.controller.value!.value.isInitialized, isTrue);
    await second.dispose();
    expect(device.closed, 2);
  });
}
