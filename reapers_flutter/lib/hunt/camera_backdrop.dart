import 'dart:async';

import 'package:camera/camera.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

/// The world behind the hunt: the phone's rear camera, filling the screen.
///
/// When there is no camera to show (a simulator, a refused permission, a
/// device without one), [fallback] is shown instead, and the hunt plays the
/// same either way.
class CameraBackdrop extends HookWidget {
  const CameraBackdrop({super.key, required this.fallback});

  final Widget fallback;

  @override
  Widget build(BuildContext context) {
    final camera = useState<CameraController?>(null);

    useEffect(() {
      var disposed = false;
      CameraController? opened;

      Future<void> open() async {
        try {
          final cameras = await availableCameras();
          final rear = cameras
              .where((lens) => lens.lensDirection == CameraLensDirection.back)
              .firstOrNull;

          if (rear == null || disposed) {
            return;
          }

          final controller = CameraController(
            rear,
            ResolutionPreset.high,
            enableAudio: false,
          );

          opened = controller;
          await controller.initialize();

          if (disposed) {
            return;
          }

          camera.value = controller;
        } on CameraException catch (error) {
          // No camera is a supported way to play, not a failure: the
          // fallback backdrop takes over. The reason is still worth a line
          // in the log for whoever is wondering why.
          debugPrint('Hunt camera unavailable: ${error.code}');
        } on MissingPluginException {
          // A platform with no camera plugin at all, such as a desktop.
          debugPrint('Hunt camera unavailable: no camera plugin');
        }
      }

      unawaited(open());

      return () {
        disposed = true;
        unawaited(opened?.dispose());
      };
    }, const []);

    // The camera is released while the app is in the background, as the
    // platform requires, and reopened by rebuilding when it returns.
    useOnAppLifecycleStateChange((_, state) {
      final controller = camera.value;

      if (controller == null || !controller.value.isInitialized) {
        return;
      }

      if (state == AppLifecycleState.inactive) {
        unawaited(controller.pausePreview());
      }

      if (state == AppLifecycleState.resumed) {
        unawaited(controller.resumePreview());
      }
    });

    final controller = camera.value;
    final preview = controller?.value.previewSize;

    if (controller == null || preview == null) {
      return fallback;
    }

    // The sensor reports its size in landscape; the game is portrait.
    return SizedBox.expand(
      child: FittedBox(
        fit: BoxFit.cover,
        clipBehavior: Clip.hardEdge,
        child: SizedBox(
          width: preview.height,
          height: preview.width,
          child: CameraPreview(controller),
        ),
      ),
    );
  }
}
