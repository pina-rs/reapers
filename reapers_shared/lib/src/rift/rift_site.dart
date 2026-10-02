import '../geo.dart';
import '../nft_catalogue.dart';
import '../vessel.dart';
import 'rift_rules.dart';

/// A real place where a rift opens.
///
/// What comes through is a whole dead collection at once, piled into one
/// body, which is why no single reaper can take it. The coordinates are real;
/// this version does not yet check that a reaper is standing at them (see
/// docs/gameplay.md).
enum RiftSite {
  londonBridge(
    title: 'London Bridge',
    city: 'London',
    latitude: 51.5079,
    longitude: -0.0877,
    collection: NftCollection.degenApes,
  ),
  shibuyaCrossing(
    title: 'Shibuya Crossing',
    city: 'Tokyo',
    latitude: 35.6595,
    longitude: 139.7005,
    collection: NftCollection.thugbirdz,
  ),
  thirdMainlandBridge(
    title: 'Third Mainland Bridge',
    city: 'Lagos',
    latitude: 6.5005,
    longitude: 3.4017,
    collection: NftCollection.sollamas,
  ),
  brooklynBridge(
    title: 'Brooklyn Bridge',
    city: 'New York',
    latitude: 40.7061,
    longitude: -73.9969,
    collection: NftCollection.peskyPenguins,
  ),
  goldenGateBridge(
    title: 'Golden Gate Bridge',
    city: 'San Francisco',
    latitude: 37.8199,
    longitude: -122.4783,
    collection: NftCollection.solarians,
  );

  const RiftSite({
    required this.title,
    required this.city,
    required this.latitude,
    required this.longitude,
    required this.collection,
  });

  final String title;

  final String city;

  final double latitude;

  final double longitude;

  /// The collection whose dead come through here. A reaper must have
  /// absorbed one of them before the rift opens for them.
  final NftCollection collection;

  /// Where on the earth the site is.
  GeoPoint get location => GeoPoint(latitude, longitude);

  /// Where this site's rift stands for a circle whose first reaper is
  /// [standing]: at the site itself for a reaper who has come to it, and a
  /// little way in front of them anywhere else.
  GeoPoint anchorFor(GeoStanding standing) =>
      standing.at.distanceTo(location) <= RiftRules.siteMetres
      ? location
      : standing.at.moved(
          bearing: standing.facing,
          metres: RiftRules.plantMetres,
        );

  /// The body every rift's husk wears.
  Vessel get vessel => Vessel.reliquaryStack;

  /// The NFTs whose images the husk is built from, bottom cube first.
  List<Nft> get relics => [
    for (var serial = 1; serial <= collection.rarities.length; serial++)
      Nft(collection: collection, serial: serial),
  ];
}
