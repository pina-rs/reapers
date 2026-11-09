//! What the page is built with: where the game is.

/// Where the game is played when `REAPERS_APP_URL` is not set when the page
/// is built: the game's own `leptoskit dev` on this machine.
pub const LOCAL_APP_URL: &str = "http://127.0.0.1:3010/";

/// Where the page's "play" links go: `REAPERS_APP_URL` when the page was
/// built, or the game on this machine.
pub fn app_url() -> &'static str {
	app_url_from(option_env!("REAPERS_APP_URL"))
}

fn app_url_from(variable: Option<&'static str>) -> &'static str {
	match variable.map(str::trim) {
		Some(url) if !url.is_empty() => url,
		_ => LOCAL_APP_URL,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn the_game_is_on_this_machine_unless_the_build_says_where() {
		assert_eq!(app_url_from(None), LOCAL_APP_URL);
		assert_eq!(app_url_from(Some("  ")), LOCAL_APP_URL);
		assert_eq!(
			app_url_from(Some("https://play.example/")),
			"https://play.example/"
		);
	}
}
