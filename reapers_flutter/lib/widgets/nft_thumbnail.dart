import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../render/nft_image.dart';
import '../theme/veil.dart';

/// An NFT's image, squared off and ruled like a plate in a catalogue.
class NftThumbnail extends HookWidget {
  const NftThumbnail({super.key, required this.nft, required this.size});

  final Nft nft;
  final double size;

  @override
  Widget build(BuildContext context) => Container(
    width: size,
    height: size,
    decoration: BoxDecoration(border: Border.all(color: Veil.hairline)),
    child: Image.asset(
      nftImageAsset(nft),
      fit: BoxFit.cover,
      semanticLabel: nft.title,
    ),
  );
}
