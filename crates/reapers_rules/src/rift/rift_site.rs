//! The real places where rifts open.

use crate::geo::GeoPoint;
use crate::geo::GeoStanding;
use crate::named::named_enum;
use crate::nft_catalogue::Nft;
use crate::nft_catalogue::NftCollection;
use crate::rift::rift_rules::RiftRules;
use crate::vessel::Vessel;

/// A real place where a rift opens.
///
/// What comes through is a whole dead collection at once, piled into one
/// body, which is why no single reaper can take it. The coordinates are
/// real; this version does not yet check that a reaper is standing at them
/// (see docs/gameplay.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum RiftSite {
	LondonBridge,
	ShibuyaCrossing,
	ThirdMainlandBridge,
	BrooklynBridge,
	GoldenGateBridge,
}

named_enum!(RiftSite {
	LondonBridge => "londonBridge",
	ShibuyaCrossing => "shibuyaCrossing",
	ThirdMainlandBridge => "thirdMainlandBridge",
	BrooklynBridge => "brooklynBridge",
	GoldenGateBridge => "goldenGateBridge",
});

impl RiftSite {
	pub const fn title(self) -> &'static str {
		match self {
			Self::LondonBridge => "London Bridge",
			Self::ShibuyaCrossing => "Shibuya Crossing",
			Self::ThirdMainlandBridge => "Third Mainland Bridge",
			Self::BrooklynBridge => "Brooklyn Bridge",
			Self::GoldenGateBridge => "Golden Gate Bridge",
		}
	}

	pub const fn city(self) -> &'static str {
		match self {
			Self::LondonBridge => "London",
			Self::ShibuyaCrossing => "Tokyo",
			Self::ThirdMainlandBridge => "Lagos",
			Self::BrooklynBridge => "New York",
			Self::GoldenGateBridge => "San Francisco",
		}
	}

	pub const fn latitude(self) -> f64 {
		self.location().latitude
	}

	pub const fn longitude(self) -> f64 {
		self.location().longitude
	}

	/// Where on the earth the site is.
	pub const fn location(self) -> GeoPoint {
		match self {
			Self::LondonBridge => GeoPoint::new(51.5079, -0.0877),
			Self::ShibuyaCrossing => GeoPoint::new(35.6595, 139.7005),
			Self::ThirdMainlandBridge => GeoPoint::new(6.5005, 3.4017),
			Self::BrooklynBridge => GeoPoint::new(40.7061, -73.9969),
			Self::GoldenGateBridge => GeoPoint::new(37.8199, -122.4783),
		}
	}

	/// The collection whose dead come through here. A reaper must have
	/// absorbed one of them before the rift opens for them.
	pub const fn collection(self) -> NftCollection {
		match self {
			Self::LondonBridge => NftCollection::DegenApes,
			Self::ShibuyaCrossing => NftCollection::Thugbirdz,
			Self::ThirdMainlandBridge => NftCollection::Sollamas,
			Self::BrooklynBridge => NftCollection::PeskyPenguins,
			Self::GoldenGateBridge => NftCollection::Solarians,
		}
	}

	/// Where this site's rift stands for a circle whose first reaper is
	/// `standing`: at the site itself for a reaper who has come to it, and a
	/// little way in front of them anywhere else.
	pub fn anchor_for(self, standing: &GeoStanding) -> GeoPoint {
		if standing.at.distance_to(&self.location()) <= RiftRules::SITE_METRES {
			self.location()
		} else {
			standing.at.moved(standing.facing, RiftRules::PLANT_METRES)
		}
	}

	/// The body every rift's husk wears.
	pub const fn vessel(self) -> Vessel {
		Vessel::ReliquaryStack
	}

	/// The NFTs whose images the husk is built from, bottom cube first.
	pub fn relics(self) -> Vec<Nft> {
		let collection = self.collection();

		(1..=collection.rarities().len())
			.map(|serial| Nft::new(collection, serial))
			.collect()
	}
}
