import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

/// The same mark used by the launcher and native launch screen.
class ReapersMark extends HookWidget {
  const ReapersMark({super.key, this.size = 128, this.splash = false});
  final double size;
  final bool splash;

  static const splashImage = AssetImage('assets/branding/splash_mark.png');

  @override
  Widget build(BuildContext context) => ExcludeSemantics(
    child: Image(
      image: splash
          ? splashImage
          : const AssetImage('assets/branding/mark.png'),
      width: size,
      height: size,
      fit: BoxFit.contain,
    ),
  );
}
