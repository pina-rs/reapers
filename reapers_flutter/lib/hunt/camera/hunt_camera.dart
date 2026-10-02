import 'dart:async';

import 'package:camera/camera.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import 'camera_optics.dart';

/// The phone's rear camera for the length of a hunt: the world drawn behind
/// the husk, and, while the room is being read, the frames it is read from.
///
/// A hunt can be played with no camera at all (a simulator, a refused
/// permission), in which case [controller] simply stays empty.
final class HuntCamera {
  /// The open camera, or null while there is none.
  final controller = ValueNotifier<CameraController?>(null);

  /// The camera's lens. A phone that will not describe its own is taken to
  /// have a typical one.
  CameraOptics optics = CameraOptics.typical;

  /// Whether [optics] came from the phone rather than from a guess.
  bool opticsKnown = false;

  final _settled = Completer<void>();
  bool _opening = false;
  bool _disposed = false;
  bool _watching = false;

  /// Completes once [open] has either opened the camera or found that
  /// there is none to open.
  Future<void> get settled => _settled.future;

  /// What the camera feed takes in, or null while there is no feed.
  LensView? get lens {
    final preview = controller.value?.value.previewSize;

    if (preview == null) {
      return null;
    }

    return optics.view(
      longSide: preview.longestSide,
      shortSide: preview.shortestSide,
    );
  }

  /// Opens the rear camera. Safe to call more than once.
  Future<void> open() async {
    if (_opening || _disposed) {
      return;
    }

    _opening = true;

    CameraController? opened;

    try {
      final cameras = await availableCameras();
      final rear = cameras
          .where((lens) => lens.lensDirection == CameraLensDirection.back)
          .firstOrNull;

      if (rear == null || _disposed) {
        return;
      }

      opened = CameraController(
        rear,
        ResolutionPreset.high,
        enableAudio: false,
        // One packed buffer on Android and plain pixels on iOS: the two
        // layouts the room reading knows how to take apart.
        imageFormatGroup: defaultTargetPlatform == TargetPlatform.android
            ? ImageFormatGroup.nv21
            : ImageFormatGroup.bgra8888,
      );

      await opened.initialize();

      final described = await CameraOptics.describe(rear);

      if (_disposed) {
        unawaited(opened.dispose());

        return;
      }

      if (described != null) {
        optics = described;
        opticsKnown = true;
      }

      controller.value = opened;
    } on CameraException catch (error) {
      // No camera is a supported way to play, not a failure: the fallback
      // backdrop takes over. The reason is still worth a line in the log
      // for whoever is wondering why.
      debugPrint('Hunt camera unavailable: ${error.code}');
      unawaited(opened?.dispose());
    } on MissingPluginException {
      // A platform with no camera plugin at all, such as a desktop.
      debugPrint('Hunt camera unavailable: no camera plugin');
    } finally {
      _settled.complete();
    }
  }

  /// Hands every frame the camera takes to [onFrame] until [stopWatching].
  ///
  /// Answers false, and watches nothing, when there is no camera to watch.
  Future<bool> watch(void Function(CameraImage frame) onFrame) async {
    final camera = controller.value;

    if (camera == null || _watching || _disposed) {
      return false;
    }

    _watching = true;
    await camera.startImageStream(onFrame);

    return true;
  }

  Future<void> stopWatching() async {
    final camera = controller.value;

    if (camera == null || !_watching) {
      return;
    }

    _watching = false;
    await camera.stopImageStream();
  }

  void dispose() {
    _disposed = true;
    _watching = false;
    unawaited(controller.value?.dispose());
    controller.dispose();
  }
}
