//! The page builds headlessly with the real layout engine at a phone's width
//! and a desk's, which also checks every Tailwind class it uses: leptoskit
//! refuses a class it cannot draw when the element is built, and in the
//! browser that refusal is a blank page.

use leptoskit::native::Role;
use leptoskit_router::Routable;
use leptoskit_testing::Finder;
use leptoskit_testing::WidgetTester;
use reactive_graph::owner::Owner;
use reapers_landing::config::app_url;
use reapers_landing::l10n::En;
use reapers_landing::l10n::L10n;
use reapers_landing::l10n::Messages;
use reapers_landing::routes::Navigation;
use reapers_landing::routes::Route;

/// The page at `route`, `width` points wide, as the router draws it.
fn tester(route: Route, width: f32) -> WidgetTester {
	let owner = Owner::new();
	owner.with(|| L10n::new(&[]).provide());
	let navigation = Navigation::new(route);
	WidgetTester::new(move || owner.with(|| navigation.render()), width, 844.0)
		.expect("the page mounts")
}

/// Whether the page's accessibility tree, read top to bottom whatever is on
/// screen, has `line` on a line of its own.
fn reads(tester: &WidgetTester, line: &str) -> bool {
	tester.semantics().lines().any(|read| read.trim() == line)
}

#[test]
fn the_page_builds_at_a_phones_width_and_a_desks() {
	let mut tester = tester(Route::Index, 390.0);
	let title = format!("heading(1) {:?}", En.hero_title());
	assert!(reads(&tester, &title), "{}", tester.semantics());
	tester.resize(1440.0, 900.0).expect("the page resizes");
	assert!(reads(&tester, &title), "{}", tester.semantics());
}

#[test]
fn the_page_has_a_banner_a_main_part_and_a_colophon() {
	let tester = tester(Route::Index, 390.0);
	let semantics = tester.semantics();
	let landmarks: Vec<&str> = semantics
		.lines()
		.filter(|line| !line.starts_with(' '))
		.collect();
	assert_eq!(landmarks, ["banner", "main", "contentinfo"], "{semantics}");
}

#[test]
fn every_section_is_a_named_region() {
	let tester = tester(Route::Index, 390.0);
	for name in [
		En.hero_title(),
		En.premise_label(),
		En.hunt_title(),
		En.orb_label(),
		En.reapers_title(),
		En.husks_title(),
		En.solana_title(),
		En.play_title(),
	] {
		let region = format!("region {name:?}");
		assert!(reads(&tester, &region), "{region}\n{}", tester.semantics());
	}
}

#[test]
fn there_is_one_first_heading_and_every_section_has_a_second() {
	let tester = tester(Route::Index, 390.0);
	let semantics = tester.semantics();
	assert_eq!(semantics.matches("heading(1)").count(), 1, "{semantics}");
	for title in [
		En.hunt_title(),
		En.reapers_title(),
		En.husks_title(),
		En.solana_title(),
		En.play_title(),
		En.colophon_title(),
	] {
		let heading = format!("heading(2) {title:?}");
		assert!(reads(&tester, &heading), "{heading}\n{semantics}");
	}
}

#[test]
fn the_music_is_credited_on_the_page() {
	let tester = tester(Route::Index, 390.0);
	for line in [
		En.colophon_sanctum_credit(),
		En.colophon_hunt_credit(),
		En.colophon_rift_credit(),
	] {
		let text = format!("text {line:?}");
		assert!(reads(&tester, &text), "{text}\n{}", tester.semantics());
	}
}

#[test]
fn the_way_into_the_game_goes_to_the_game() {
	let tester = tester(Route::Index, 390.0);
	let play = tester
		.find(&Finder::aria_role(Role::Button, En.header_play()))
		.expect("the banner's way in");
	assert_eq!(play.props.href(), Some(app_url()));
}

#[test]
fn an_unknown_address_shows_the_page() {
	let route = Route::parse("/no-such-page").expect("every address has a route");
	assert!(matches!(route, Route::NotFound(_)), "{route:?}");
	let tester = tester(route, 390.0);
	let title = format!("heading(1) {:?}", En.hero_title());
	assert!(reads(&tester, &title), "{}", tester.semantics());
}

/// What the browser reads before the page's script has run: its title, its
/// description and what it says to a browser that runs no script. They are
/// written in `web/index.html`, and held here to `locales/en.arb`.
const SHELL: &str = include_str!("../web/index.html");

#[test]
fn the_shell_says_what_the_page_says() {
	assert!(SHELL.contains(&format!("<title>{}</title>", En.brand_name())));
	assert!(SHELL.contains(&format!(
		"<meta name=\"description\" content=\"{}\" />",
		En.meta_description()
	)));
	assert!(SHELL.contains(&format!("<p>{}</p>", En.no_script())));
}
