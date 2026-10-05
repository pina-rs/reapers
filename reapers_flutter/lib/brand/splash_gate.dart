import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

import '../l10n/l10n.dart';
import '../theme/veil.dart';
import 'reapers_mark.dart';

/// A matching native still opens into one ceremonial sweep. The mounted
/// game is quiet and inaccessible beneath it. Slow startup hands off to
/// the loading/retry UI after four seconds; motion never blocks recovery.
class SplashGate extends HookWidget {
  const SplashGate({
    super.key,
    required this.ready,
    required this.child,
    this.onFirstFrameReady,
  });
  final bool ready;
  final Widget child;
  final VoidCallback? onFirstFrameReady;

  @override
  Widget build(BuildContext context) {
    final reduced = MediaQuery.disableAnimationsOf(context);
    final ritual = useAnimationController(
      duration: const Duration(milliseconds: 1800),
    );
    final fade = useAnimationController(
      duration: const Duration(milliseconds: 220),
    );
    final elapsed = useState(false);
    final deadline = useState(false);
    final dismissed = useState(false);
    final primed = useState(false);
    useListenable(ritual);
    useListenable(fade);

    useEffect(() {
      var closed = false;
      Future<void> prime() async {
        try {
          await precacheImage(
            ReapersMark.splashImage,
            context,
          ).timeout(const Duration(seconds: 2));
        } on TimeoutException {
          // A slow decode must not keep the native screen over the retry UI.
        } finally {
          onFirstFrameReady?.call();
          if (!closed) primed.value = true;
        }
      }

      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!closed) unawaited(prime());
      });
      final limit = Timer(
        const Duration(seconds: 4),
        () => deadline.value = true,
      );
      return () {
        closed = true;
        limit.cancel();
      };
    }, const []);

    useEffect(() {
      if (!primed.value) return null;
      var closed = false;
      final duration = reduced
          ? const Duration(milliseconds: 250)
          : const Duration(milliseconds: 1800);
      if (reduced) {
        ritual.value = 1;
      } else {
        unawaited(ritual.forward());
      }
      final timer = Timer(duration, () {
        if (!closed) elapsed.value = true;
      });
      return () {
        closed = true;
        timer.cancel();
      };
    }, [primed.value, reduced]);

    useEffect(() {
      if (dismissed.value || !(deadline.value || (elapsed.value && ready))) {
        return null;
      }
      var closed = false;
      if (reduced) {
        dismissed.value = true;
      } else {
        unawaited(
          fade.forward().then((_) {
            if (!closed) dismissed.value = true;
          }),
        );
      }
      return () => closed = true;
    }, [elapsed.value, deadline.value, ready, reduced]);

    return Stack(
      fit: StackFit.expand,
      children: [
        TickerMode(
          enabled: dismissed.value,
          child: ExcludeSemantics(
            excluding: !dismissed.value,
            child: ExcludeFocus(
              excluding: !dismissed.value,
              child: IgnorePointer(ignoring: !dismissed.value, child: child),
            ),
          ),
        ),
        if (!dismissed.value)
          Opacity(
            opacity: 1 - fade.value,
            child: ColoredBox(
              color: Veil.ink,
              child: Semantics(
                label: context.l10n.appTitle,
                child: Stack(
                  fit: StackFit.expand,
                  children: [
                    const Center(child: ReapersMark(size: 288, splash: true)),
                    if (!reduced)
                      Center(
                        child: SizedBox.square(
                          dimension: 288,
                          child: CustomPaint(
                            painter: _ReapingSweep(ritual.value),
                          ),
                        ),
                      ),
                    Center(
                      child: Transform.translate(
                        offset: const Offset(0, 174),
                        child: Opacity(
                          opacity: reduced
                              ? 1
                              : Curves.easeOut.transform(
                                  ((ritual.value - .3) / .5).clamp(0, 1),
                                ),
                          child: Text(
                            context.l10n.appTitle,
                            style: Veil.display(42, weight: 360),
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
      ],
    );
  }
}

/// One fine arc closes around the scythe and dissolves, rather than a
/// perpetual spinner. The emblem itself stays fixed at its native position.
class _ReapingSweep extends CustomPainter {
  const _ReapingSweep(this.progress);
  final double progress;

  @override
  void paint(Canvas canvas, Size size) {
    final cut = Curves.easeInOutCubic.transform((progress / .75).clamp(0, 1));
    final opacity = math.sin(progress * math.pi).clamp(0.0, 1.0) * .6;
    final rect = Rect.fromCenter(
      center: size.center(Offset.zero),
      width: 236,
      height: 236,
    );
    canvas.drawArc(
      rect,
      -math.pi * .85,
      math.pi * 1.6 * cut,
      false,
      Paint()
        ..color = Veil.bone.withValues(alpha: opacity)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1,
    );
  }

  @override
  bool shouldRepaint(_ReapingSweep oldDelegate) =>
      oldDelegate.progress != progress;
}
