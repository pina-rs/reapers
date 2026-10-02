import 'ar_pose.dart';
import 'pose_sources.dart';

/// Follows the phone when it can be felt, and a dragging finger when it
/// cannot.
///
/// A simulator, a desktop or a phone with no gyroscope never produces a
/// device pose, so the drag pose simply stays in charge. Nothing has to
/// decide in advance which kind of machine this is.
final class AdaptivePoseSource implements PoseSource {
  final _device = DevicePoseSource();
  final _drag = DragPoseSource();

  @override
  ArPose get pose => followsDevice ? _device.pose : _drag.pose;

  @override
  bool get followsDevice => _device.isReady;

  @override
  double? get north => followsDevice ? _device.north : null;

  @override
  void advance(double deltaSeconds) => _drag.advance(deltaSeconds);

  @override
  void drag(double dx, double dy) {
    if (!followsDevice) {
      _drag.drag(dx, dy);
    }
  }

  @override
  void dispose() {
    _device.dispose();
    _drag.dispose();
  }
}
