//! Every rift there is, where each one opens, and whether it will open for
//! this reaper.

use leptoskit::prelude::*;
use reapers_rules::RiftSite;

use super::open;
use crate::l10n::use_l10n;
use crate::routes::Rift;
use crate::state::Game;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::ui::LedgerButton;
use crate::ui::aside;
use crate::ui::body;
use crate::ui::display;
use crate::ui::figure;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_page;

/// The rifts.
pub fn page() -> Element {
	let game = Game::get();
	let words = use_l10n();
	let Some(player) = game.player() else {
		return View();
	};
	let demo = game.flags.demo_rift;
	let mut entries = vec![
		aside(words.rifts_stand_in()),
		gap(8),
		aside(words.rifts_location_note()),
	];
	if demo {
		entries.push(gap(14));
		entries.push(aside(words.rifts_demo_note()).test_id("rifts-demo-note"));
	}
	for site in RiftSite::ALL {
		entries.push(site_block(site, demo || player.attuned_to(site), demo));
	}
	ledger_page(
		words.rifts_overline(),
		Some(words.rifts_title()),
		None,
		entries,
	)
}

/// The site's position as it would be written on a chart.
fn coordinates(site: RiftSite) -> String {
	let part = |value: f64, positive: char, negative: char| {
		format!(
			"{:.4}° {}",
			value.abs(),
			if value >= 0.0 { positive } else { negative }
		)
	};
	format!(
		"{}  {}",
		part(site.latitude(), 'N', 'S'),
		part(site.longitude(), 'E', 'W')
	)
}

/// One rift: the place, whose dead come through it, and the way in.
fn site_block(site: RiftSite, attuned: bool, demo: bool) -> Element {
	let words = use_l10n();
	let way_in = if attuned {
		let mut button = LedgerButton::new(if demo {
			words.rifts_demo_open()
		} else {
			words.rifts_open()
		})
		.test_id(format!("rift-enter-{}", site.name()));
		if demo {
			button = button.detail(words.rifts_demo_detail());
		}
		button.on_press(move || {
			open(Rift {
				site: site.name().to_owned(),
			});
		})
	} else {
		body(words.rifts_attune(site.collection().title()), 14, ASH)
			.test_id(format!("rift-shut-{}", site.name()))
	};
	View()
		.class("pt-[26px] gap-[6px]")
		.child(
			View()
				.class("flex-row items-center gap-3")
				.child(View().class("flex-1").child(label(
					&words.rifts_site(site.city(), site.collection().title()),
					10,
					ASH,
				)))
				.child(figure(coordinates(site), 11, ASH)),
		)
		.child(display(site.title(), 28, BONE))
		.child(gap(6))
		.child(way_in)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_site_is_written_as_on_a_chart() {
		let written = coordinates(RiftSite::ALL[0]);
		assert!(written.contains('°'));
		assert!(written.contains('N') || written.contains('S'));
	}
}
