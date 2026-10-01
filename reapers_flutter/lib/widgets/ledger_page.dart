import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

import '../theme/veil.dart';
import 'ledger.dart';

/// A page of the ledger: a way back, a heading in two sizes, and a column of
/// content that scrolls.
class LedgerPage extends HookWidget {
  const LedgerPage({
    super.key,
    required this.overline,
    required this.children,
    this.title,
    this.trailing,
  });

  /// The small heading: which part of the game this is.
  final String overline;

  /// The large heading, when the page has something to say.
  final String? title;

  /// Set at the top right, level with the way back: a balance, a count.
  final Widget? trailing;

  final List<Widget> children;

  @override
  Widget build(BuildContext context) => Scaffold(
    body: SafeArea(
      child: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: Veil.contentWidth),
          child: CustomScrollView(
            slivers: [
              SliverPadding(
                padding: const EdgeInsets.fromLTRB(24, 8, 24, 40),
                sliver: SliverList.list(
                  children: [
                    Row(
                      children: [
                        LedgerBack(
                          onPressed: () => Navigator.of(context).maybePop(),
                        ),
                        const Spacer(),
                        ?trailing,
                      ],
                    ),
                    const SizedBox(height: 18),
                    Text(overline.toUpperCase(), style: Veil.label()),
                    if (title != null) ...[
                      const SizedBox(height: 10),
                      Text(title!, style: Veil.display(34)),
                    ],
                    const SizedBox(height: 8),
                    ...children,
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    ),
  );
}
