import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

import '../theme/veil.dart';
import 'sketch.dart';

/// The pieces every screen is ruled with: buttons, rows, meters and marks,
/// all drawn like entries in a ledger.

/// A full-width action. It is an outline until it is pressed, when it fills
/// with the reaper's light.
class LedgerButton extends HookWidget {
  const LedgerButton({
    super.key,
    required this.label,
    required this.onPressed,
    this.detail,
    this.quiet = false,
  });

  final String label;

  /// Called on a tap. A null callback draws the button as unavailable.
  final VoidCallback? onPressed;

  /// A figure shown at the right edge: a price, a count.
  final String? detail;

  /// A quiet button has no outline, for the lesser of two choices.
  final bool quiet;

  @override
  Widget build(BuildContext context) {
    final pressed = useState(false);
    final focused = useState(false);
    final light = Theme.of(context).colorScheme.primary;
    final enabled = onPressed != null;
    final foreground = !enabled
        ? Veil.ash
        : pressed.value
        ? Veil.ink
        : Veil.bone;
    final rule = focused.value
        ? light
        : enabled
        ? Veil.bone.withValues(alpha: 0.7)
        : Veil.hairline;
    // Keep one decoration for the life of the button, so a clock elsewhere on
    // the screen cannot restart the border animation once a second.
    final sketch = useMemoized(
      () => quiet && !focused.value
          ? null
          : Sketch.box(colour: rule, seed: Sketch.seedFor(label)),
      [quiet, focused.value, rule, label],
    );

    void activate() {
      HapticFeedback.selectionClick();
      onPressed?.call();
    }

    return Semantics(
      button: true,
      enabled: enabled,
      focusable: enabled,
      focused: focused.value,
      label: detail == null ? label : '$label, $detail',
      onTap: enabled ? activate : null,
      excludeSemantics: true,
      child: Material(
        type: MaterialType.transparency,
        child: InkWell(
          onTap: enabled ? activate : null,
          canRequestFocus: enabled,
          onHighlightChanged: (value) => pressed.value = value,
          onFocusChange: (value) => focused.value = value,
          splashFactory: NoSplash.splashFactory,
          highlightColor: Colors.transparent,
          hoverColor: Colors.transparent,
          focusColor: Colors.transparent,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 90),
            constraints: const BoxConstraints(minHeight: 56),
            padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 16),
            decoration: BoxDecoration(
              color: pressed.value ? light : Colors.transparent,
            ),
            // A quiet button carries no line until it is focused. The rule is
            // drawn by hand, and its seed follows the label, so two actions
            // never share exactly the same wobble.
            foregroundDecoration: sketch,
            // The label sits left and the detail right while both fit on one
            // line. On a narrow phone the detail drops under the label rather
            // than running into it.
            child: SizedBox(
              width: double.infinity,
              child: Wrap(
                alignment: WrapAlignment.spaceBetween,
                crossAxisAlignment: WrapCrossAlignment.center,
                spacing: 16,
                runSpacing: 6,
                children: [
                  Text(
                    label.toUpperCase(),
                    style: Veil.label(color: foreground, size: 13),
                  ),
                  if (detail != null)
                    Text(
                      detail!,
                      style: Veil.figure(color: foreground, size: 13),
                    )
                  else if (enabled)
                    Text('→', style: Veil.figure(color: foreground, size: 16)),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// The way back, at the top of a screen.
class LedgerBack extends HookWidget {
  const LedgerBack({super.key, required this.onPressed, this.label});

  final VoidCallback onPressed;

  /// What going back is called here. Defaults to an arrow alone.
  final String? label;

  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    label: label ?? MaterialLocalizations.of(context).backButtonTooltip,
    excludeSemantics: true,
    child: GestureDetector(
      key: const Key('ledger-back'),
      behavior: HitTestBehavior.opaque,
      onTap: onPressed,
      child: ConstrainedBox(
        constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
        // Sized to the label, so the button never grows to fill a tall
        // parent and drift away from the top of the page.
        child: Align(
          alignment: Alignment.centerLeft,
          widthFactor: 1,
          heightFactor: 1,
          child: Text(
            label == null ? '←' : '←  ${label!.toUpperCase()}',
            style: Veil.label(color: Veil.bone, size: 13),
          ),
        ),
      ),
    ),
  );
}

/// One line of a ledger: a name on the left, a figure on the right, and a
/// rule of dots between them.
class LedgerRow extends HookWidget {
  const LedgerRow({
    super.key,
    required this.label,
    required this.value,
    this.emphasised = false,
  });

  final String label;

  final String value;

  /// Whether the figure is set in the reaper's light.
  final bool emphasised;

  @override
  Widget build(BuildContext context) {
    final light = Theme.of(context).colorScheme.primary;

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 7),
      child: LayoutBuilder(
        builder: (context, constraints) {
          final labelStyle = Veil.label();
          final valueStyle = Veil.figure(
            color: emphasised ? light : Veil.bone,
            size: 15,
          );

          double width(String text, TextStyle style) {
            final painter = TextPainter(
              text: TextSpan(text: text, style: style),
              textDirection: Directionality.of(context),
              textScaler: MediaQuery.textScalerOf(context),
            )..layout();
            final measured = painter.width;
            painter.dispose();
            return measured;
          }

          final labelText = Text(label.toUpperCase(), style: labelStyle);
          final valueText = Text(value, style: valueStyle);

          // Keep every figure readable when a translated label or enlarged
          // text no longer leaves room for the dotted leader.
          if (width(label.toUpperCase(), labelStyle) +
                  width(value, valueStyle) +
                  20 >
              constraints.maxWidth) {
            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [labelText, const SizedBox(height: 4), valueText],
            );
          }

          return Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              labelText,
              const Expanded(
                child: Padding(
                  padding: EdgeInsets.fromLTRB(10, 0, 10, 3),
                  child: _Leader(),
                ),
              ),
              valueText,
            ],
          );
        },
      ),
    );
  }
}

/// The row of dots that leads the eye from a name to its figure.
class _Leader extends HookWidget {
  const _Leader();

  @override
  Widget build(BuildContext context) => const CustomPaint(
    painter: _LeaderPainter(),
    size: Size(double.infinity, 1),
  );
}

class _LeaderPainter extends CustomPainter {
  const _LeaderPainter();

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()..color = Veil.hairline;

    for (var x = 0.0; x < size.width; x += 5) {
      canvas.drawCircle(Offset(x, size.height / 2), 0.6, paint);
    }
  }

  @override
  bool shouldRepaint(_LeaderPainter oldDelegate) => false;
}

/// A thin bar with a name: how much of something is left.
class LedgerMeter extends HookWidget {
  const LedgerMeter({
    super.key,
    required this.fraction,
    required this.colour,
    required this.label,
  });

  /// From 0 (empty) to 1 (full).
  final double fraction;

  final Color colour;

  final String label;

  @override
  Widget build(BuildContext context) => Semantics(
    label: label,
    value: '${(fraction.clamp(0, 1) * 100).round()}%',
    child: Row(
      children: [
        SizedBox(
          width: 62,
          child: Text(label.toUpperCase(), style: Veil.label()),
        ),
        Expanded(
          child: SizedBox(
            height: 5,
            child: CustomPaint(
              painter: _MeterPainter(fraction.clamp(0, 1).toDouble(), colour),
            ),
          ),
        ),
      ],
    ),
  );
}

class _MeterPainter extends CustomPainter {
  const _MeterPainter(this.fraction, this.colour);

  final double fraction;
  final Color colour;

  @override
  void paint(Canvas canvas, Size size) {
    final middle = size.height / 2;

    canvas
      ..drawLine(
        Offset(0, middle),
        Offset(size.width, middle),
        Paint()
          ..color = Veil.hairline
          ..strokeWidth = 1,
      )
      ..drawRect(
        Rect.fromLTWH(0, 0, size.width * fraction, size.height),
        Paint()..color = colour,
      )
      // A notch where the bar ends, so a nearly empty bar still reads.
      ..drawRect(
        Rect.fromLTWH(size.width * fraction - 1, -3, 2, size.height + 6),
        Paint()..color = Veil.bone,
      );
  }

  @override
  bool shouldRepaint(_MeterPainter oldDelegate) =>
      oldDelegate.fraction != fraction || oldDelegate.colour != colour;
}

/// A small diamond that is either lit or spent.
class LedgerPip extends HookWidget {
  const LedgerPip({super.key, required this.lit, required this.colour});

  final bool lit;

  final Color colour;

  @override
  Widget build(BuildContext context) => Transform.rotate(
    angle: 0.785398,
    child: Container(
      width: 8,
      height: 8,
      decoration: BoxDecoration(
        color: lit ? colour : Colors.transparent,
        border: Border.all(color: lit ? colour : Veil.ash),
      ),
    ),
  );
}

/// A heading in the margin, with a rule running to the edge of the page.
class LedgerHeading extends HookWidget {
  const LedgerHeading(this.text, {super.key, this.trailing});

  final String text;

  /// A count or a figure set at the far end of the rule.
  final String? trailing;

  /// The width kept back from a long heading for the rule, and for the
  /// figure when there is one, so the heading is cut short before either is
  /// pushed off the page.
  static const _ruleWidth = 36.0;
  static const _figureWidth = 64.0;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 22, bottom: 10),
    child: LayoutBuilder(
      builder: (context, constraints) => Row(
        children: [
          Flexible(
            child: ConstrainedBox(
              constraints: BoxConstraints(
                maxWidth: math.max(
                  0,
                  constraints.maxWidth -
                      _ruleWidth -
                      (trailing == null ? 0 : _figureWidth),
                ),
              ),
              child: Text(
                text.toUpperCase(),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: Veil.label(color: Veil.bone),
              ),
            ),
          ),
          const SizedBox(width: 12),
          const Expanded(child: Divider(height: 1)),
          if (trailing != null) ...[
            const SizedBox(width: 12),
            Text(trailing!, style: Veil.figure(color: Veil.ash, size: 12)),
          ],
        ],
      ),
    ),
  );
}

/// A whole page given over to one thing that went wrong, with the way out.
class LedgerNotice extends HookWidget {
  const LedgerNotice({
    super.key,
    required this.title,
    required this.body,
    required this.action,
    required this.onAction,
  });

  final String title;
  final String body;
  final String action;
  final VoidCallback onAction;

  @override
  Widget build(BuildContext context) => SafeArea(
    child: Padding(
      padding: const EdgeInsets.all(28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(title, style: Veil.display(40)),
          const SizedBox(height: 14),
          Text(body, style: Veil.body(color: Veil.ash)),
          const SizedBox(height: 28),
          LedgerButton(label: action, onPressed: onAction),
        ],
      ),
    ),
  );
}
