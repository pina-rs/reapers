package com.reapers.app

import android.content.Context
import android.hardware.camera2.CameraAccessException
import android.hardware.camera2.CameraCharacteristics
import android.hardware.camera2.CameraManager
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)

        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, LENS_CHANNEL)
            .setMethodCallHandler { call, result ->
                if (call.method != "describe") {
                    result.notImplemented()
                    return@setMethodCallHandler
                }

                val camera = call.argument<String>("camera")

                if (camera == null) {
                    result.error("no_camera", "A camera id is required", null)
                    return@setMethodCallHandler
                }

                describeLens(camera, result)
            }
    }

    /**
     * Answers with the focal length and the sensor's size, both in
     * millimetres: what the game needs to know how wide the camera sees.
     */
    private fun describeLens(camera: String, result: MethodChannel.Result) {
        val manager = getSystemService(Context.CAMERA_SERVICE) as CameraManager

        try {
            val traits = manager.getCameraCharacteristics(camera)
            val focalLength =
                traits.get(CameraCharacteristics.LENS_INFO_AVAILABLE_FOCAL_LENGTHS)?.firstOrNull()
            val sensor = traits.get(CameraCharacteristics.SENSOR_INFO_PHYSICAL_SIZE)

            if (focalLength == null || sensor == null) {
                result.error("no_lens", "Camera $camera does not describe its lens", null)
                return
            }

            result.success(
                mapOf(
                    "focalLength" to focalLength.toDouble(),
                    "sensorWidth" to sensor.width.toDouble(),
                    "sensorHeight" to sensor.height.toDouble(),
                ),
            )
        } catch (error: CameraAccessException) {
            result.error("no_access", error.message, null)
        } catch (error: IllegalArgumentException) {
            result.error("no_camera", "There is no camera $camera", null)
        }
    }

    private companion object {
        const val LENS_CHANNEL = "reapers/lens"
    }
}
