import 'dart:async';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:camera/camera.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

import '../hunt/camera/hunt_camera.dart';
import '../hunt/camera_backdrop.dart';
import '../hunt/scan/depth/camera_picture.dart';
import '../hunt/scan/depth/depth_model.dart';
import '../theme/veil.dart';

/// A bench for the depth model: the camera, what the model makes of it, and
/// how long it takes. For development only; a release never shows it.
///
/// Its words are plain English and stay out of the translations: a player
/// never reads them.
class DepthProbe extends HookWidget {
  const DepthProbe({super.key});

  @override
  Widget build(BuildContext context) {
    final camera = useMemoized(HuntCamera.new);
    final controller = useValueListenable(camera.controller);
    final picture = useState<ui.Image?>(null);
    final report = useState('Loading the model');

    useEffect(() => camera.dispose, [camera]);

    useEffect(() {
      if (controller == null) {
        return null;
      }

      var stopped = false;
      var busy = false;
      DepthModel? model;

      Future<void> look(CameraImage frame) async {
        final loaded = model;

        if (busy || stopped || loaded == null) {
          return;
        }

        busy = true;

        try {
          final watch = Stopwatch()..start();
          final cut = CameraPicture.from(
            frame,
            sensorOrientation: controller.description.sensorOrientation,
            width: DepthModel.width,
            height: DepthModel.height,
          );

          if (cut == null) {
            report.value = 'Cannot read ${frame.format.group.name} frames';

            return;
          }

          final cutting = watch.elapsedMilliseconds;
          final depths = await loaded.estimate(cut.pixels);
          final total = watch.elapsedMilliseconds;
          final sorted = Float32List.fromList(depths)..sort();
          final summary =
              'cut $cutting ms, model ${total - cutting} ms\n'
              'nearest ${sorted.first.toStringAsFixed(2)} m, '
              'middle ${sorted[sorted.length ~/ 2].toStringAsFixed(2)} m, '
              'farthest ${sorted.last.toStringAsFixed(2)} m\n'
              'lens ${camera.opticsKnown ? 'read from the phone' : 'assumed'}: '
              '${camera.optics.focalLength} mm on '
              '${camera.optics.sensorLong} x ${camera.optics.sensorShort} mm';

          debugPrint('Depth probe: ${summary.replaceAll('\n', '; ')}');

          final drawn = await _draw(depths, sorted.first, sorted.last);

          if (stopped) {
            drawn.dispose();

            return;
          }

          picture.value?.dispose();
          picture.value = drawn;
          report.value = summary;
        } finally {
          busy = false;
        }
      }

      Future<void> start() async {
        model = await DepthModel.load();

        if (stopped) {
          return;
        }

        if (model == null) {
          report.value = 'This build cannot run the depth model';

          return;
        }

        await camera.watch((frame) => unawaited(look(frame)));
      }

      unawaited(start());

      return () {
        stopped = true;
        unawaited(camera.stopWatching());
        unawaited(model?.close());
      };
    }, [controller]);

    return Scaffold(
      backgroundColor: Veil.ink,
      body: Stack(
        fit: StackFit.expand,
        children: [
          CameraBackdrop(camera: camera, fallback: const SizedBox.expand()),
          SafeArea(
            child: Padding(
              padding: const EdgeInsets.all(20),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(report.value, style: Veil.figure(size: 13)),
                  const Spacer(),
                  if (picture.value case final drawn?)
                    SizedBox(
                      width: 150,
                      child: AspectRatio(
                        aspectRatio: DepthModel.width / DepthModel.height,
                        child: RawImage(image: drawn, fit: BoxFit.fill),
                      ),
                    ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  /// Paints [depths] as a picture: near is bright, far is dark.
  static Future<ui.Image> _draw(Float32List depths, double near, double far) {
    final pixels = Uint8List(depths.length * 4);
    final span = far - near;

    for (var index = 0; index < depths.length; index++) {
      final shade = span <= 0
          ? 0
          : (255 * (1 - (depths[index] - near) / span)).round();

      pixels[index * 4] = shade;
      pixels[index * 4 + 1] = shade;
      pixels[index * 4 + 2] = shade;
      pixels[index * 4 + 3] = 255;
    }

    final done = Completer<ui.Image>();

    ui.decodeImageFromPixels(
      pixels,
      DepthModel.width,
      DepthModel.height,
      ui.PixelFormat.rgba8888,
      done.complete,
    );

    return done.future;
  }
}
