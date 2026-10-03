import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter_scene/scene.dart';
import 'package:vector_math/vector_math.dart' as vm;

/// The lighting and lens of the spirit realm.
///
/// Every 3D surface in the game goes through one of these, so a reaper in the
/// sanctum and a husk over the camera feed belong to the same world.
abstract final class VeilLook {
  /// Lights [scene] for a reaper standing alone in the dark.
  ///
  /// A cold key from above and in front leaves the face in the shadow of
  /// whatever the reaper wears on its head; the shader's rim does the rest.
  static void sanctum(Scene scene) {
    scene
      ..renderScale = _renderScale
      ..environment = _nightSky
      ..directionalLight = DirectionalLight(
        direction: vm.Vector3(0.35, -1, 0.55),
        color: vm.Vector3(0.82, 0.9, 1),
        intensity: 4.6,
        castsShadow: true,
        shadowSoftness: 0.12,
        shadowMaxDistance: 12,
      )
      ..environmentSettings = EnvironmentSettings(
        toneMapping: ToneMappingMode.aces,
        exposure: 1.3,
        environmentIntensity: 0.55,
        bloomEnabled: true,
        bloomThreshold: 1,
        bloomIntensity: 0.32,
        bloomScatter: 0.78,
        ambientOcclusionEnabled: true,
        ambientOcclusionMethod: AmbientOcclusionMethod.groundTruth,
        ambientOcclusionIntensity: 1.1,
        ambientOcclusionPower: 1.6,
        colorGradingEnabled: true,
        contrast: 1.1,
        saturation: 0.92,
        temperature: -0.06,
        vignetteEnabled: true,
        vignetteIntensity: 0.55,
        vignetteRadius: 0.7,
      );
  }

  /// Lights [scene] for husks drawn over the live camera feed.
  ///
  /// Nothing here may darken or tint the whole frame: the background is the
  /// real world, and it has to stay honest.
  static void hunt(Scene scene) {
    scene
      ..renderScale = _renderScale
      ..environment = _nightSky
      ..directionalLight = DirectionalLight(
        direction: vm.Vector3(-0.3, -1, 0.4),
        color: vm.Vector3(0.8, 0.9, 1),
        // Kept low enough that a pale coat still shows its shape.
        intensity: 1.7,
      )
      ..environmentSettings = EnvironmentSettings(
        toneMapping: ToneMappingMode.aces,
        exposure: 1.0,
        environmentIntensity: 0.6,
        bloomEnabled: true,
        bloomThreshold: 1,
        bloomIntensity: 0.4,
        bloomScatter: 0.8,
      );
  }

  /// The most physical pixels the 3D is drawn with for each logical one.
  ///
  /// A phone screen can be three times as dense as that, which for a small
  /// GPU is more than twice the pixels to light and bloom each frame. The
  /// Seeker's could not keep up: it drew the sanctum at about 30 frames a
  /// second, and its slowest frames ran long enough for its Vulkan driver
  /// to take the GPU for lost. At arm's length two is as sharp to the eye.
  static const _density = 2.0;

  static double get _renderScale {
    final view = ui.PlatformDispatcher.instance.implicitView;

    return view == null ? 1 : math.min(1, _density / view.devicePixelRatio);
  }

  /// A dim, cold sky to light surfaces from. It is never drawn.
  ///
  /// Baking it is one long job on the GPU, so it is baked once, the first
  /// time a scene is lit, and every scene after shares it. Baked afresh for
  /// each scene, at the size flutter_scene bakes by default, it held the
  /// Seeker's GPU for about 0.4 s every time a hunt opened, which its Vulkan
  /// driver takes for a lost GPU: it resets, and every picture uploaded
  /// before then, the letters of the text among them, is drawn broken. A
  /// smooth sky that only lights things needs a quarter of that size.
  static final _nightSky = EnvironmentMap.fromSky(
    GradientSkySource(
      zenithColor: vm.Vector3(0.05, 0.08, 0.16),
      horizonColor: vm.Vector3(0.1, 0.15, 0.2),
      groundColor: vm.Vector3(0.012, 0.012, 0.016),
      sunDirection: vm.Vector3(-0.4, 0.7, -0.5),
      sunColor: vm.Vector3(1.6, 1.9, 2.4),
      sunSharpness: 40,
    ),
    faceResolution: 64,
    equirectWidth: 256,
  );
}
