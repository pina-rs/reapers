import 'dart:async';

import 'package:camera/camera.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

import 'camera/hunt_camera.dart';

/// The world behind the hunt: the phone's rear camera, filling the screen.
///
/// When there is no camera to show (a simulator, a refused permission, a
/// device without one), [fallback] is shown instead, and the hunt plays the
/// same either way.
class CameraBackdrop extends HookWidget {
  const CameraBackdrop({
    super.key,
    required this.camera,
    required this.fallback,
  });

  /// Opened here, and closed by whoever owns it.
  final HuntCamera camera;

  final Widget fallback;

  @override
  Widget build(BuildContext context) {
    final controller = useValueListenable(camera.controller);

    useEffect(() {
      unawaited(camera.open());

      return null;
    }, [camera]);

    // The camera is released while the app is in the background, as the
    // platform requires, and reopened by rebuilding when it returns.
    useOnAppLifecycleStateChange((_, state) {
      final controller = camera.controller.value;

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
