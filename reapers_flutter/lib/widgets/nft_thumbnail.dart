import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../render/nft_image.dart';
import 'sketch.dart';

/// An NFT's image, squared off and ruled like a plate in a catalogue.
class NftThumbnail extends HookWidget {
  const NftThumbnail({super.key, required this.nft, required this.size});

  final Nft nft;
  final double size;

  @override
  Widget build(BuildContext context) {
    final decoration = useMemoized(
      () => Sketch.box(seed: Sketch.seedFor(nft.id)),
      [nft.id],
    );

    return Container(
      width: size,
      height: size,
      foregroundDecoration: decoration,
      child: Image.asset(
        nftImageAsset(nft),
        fit: BoxFit.cover,
        semanticLabel: nft.title,
      ),
    );
  }
}
