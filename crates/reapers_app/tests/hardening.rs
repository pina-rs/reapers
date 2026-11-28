//! Regression tests for Pillar 3 app hardening: render-scope cleanups,
//! platform system back handling, ref-counted busy state, rate-limit safe
//! session refresh, serialized local backend mutations, and ICU plurals.

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use async_trait::async_trait;
use futures::executor::block_on;
use leptoskit::prelude::*;
use leptoskit_testing::Finder;
use leptoskit_testing::WidgetTester;
use reactive_graph::owner::Owner;
use reactive_graph::owner::on_cleanup;
use reactive_graph::owner::provide_context;
use reactive_graph::traits::GetUntracked;
use reactive_graph::traits::Set;
use reapers_app::app::RenderScope;
use reapers_app::backend::GameBackend;
use reapers_app::backend::LocalBackend;
use reapers_app::backend::ReapersApi;
use reapers_app::backend::ServerApi;
use reapers_app::config::BuildFlags;
use reapers_app::l10n::En;
use reapers_app::l10n::L10n;
use reapers_app::l10n::Messages;
use reapers_app::platform;
use reapers_app::platform::Header;
use reapers_app::platform::HttpReply;
use reapers_app::platform::HttpTransport;
use reapers_app::platform::MemoryPrefs;
use reapers_app::platform::PlatformError;
use reapers_app::platform::Prefs;
use reapers_app::routes::Navigation;
use reapers_app::routes::Route;
use reapers_app::screens::handle_system_back;
use reapers_app::state::Game;
use reapers_app::state::Overlay;
use reapers_rules::Nft;
use reapers_rules::NftCollection;
use serde_json::Value;

#[test]
fn render_scope_runs_cleanup_only_when_replaced_without_dropping_live_mounts() {
	let root = Owner::new();
	let scope = RenderScope::new(root);
	let mounted = Rc::new(Cell::new(0usize));
	let saw_zero_during_rerender = Rc::new(Cell::new(false));

	// First render mounts the screen.
	{
		let mounted = Rc::clone(&mounted);
		let _ = scope.render(move || {
			mounted.set(mounted.get() + 1);
			let unmount = send_wrapper::SendWrapper::new(Rc::clone(&mounted));
			on_cleanup(move || {
				let unmount = unmount.take();
				unmount.set(unmount.get().saturating_sub(1));
			});
			View()
		});
	}
	assert_eq!(mounted.get(), 1);

	// Re-rendering the same screen creates the new mount before cleaning up
	// the old one, so `mounted` never drops to zero mid-transition.
	{
		let mounted = Rc::clone(&mounted);
		let saw_zero = Rc::clone(&saw_zero_during_rerender);
		let _ = scope.render(move || {
			mounted.set(mounted.get() + 1);
			let unmount = send_wrapper::SendWrapper::new(Rc::clone(&mounted));
			let saw_zero = send_wrapper::SendWrapper::new(saw_zero);
			on_cleanup(move || {
				let unmount = unmount.take();
				let next = unmount.get().saturating_sub(1);
				unmount.set(next);
				if next == 0 {
					saw_zero.take().set(true);
				}
			});
			View()
		});
	}
	assert_eq!(mounted.get(), 1);
	assert!(!saw_zero_during_rerender.get());

	// Navigating away to a different screen drops the previous owner and runs
	// its cleanup, bringing `mounted` cleanly to zero.
	let _ = scope.render(View);
	assert_eq!(mounted.get(), 0);
	assert!(saw_zero_during_rerender.get());
}

#[test]
fn system_back_prioritizes_arrival_overlay_studio_book_hunt_and_history() {
	let owner = Owner::new();
	let scope = Rc::new(RenderScope::new(owner.clone()));
	let navigation = Navigation::new(Route::Index);
	let game = owner.with(|| {
		L10n::new(&[]).provide();
		reapers_app::screens::install_navigation(navigation.clone());
		provide_context(navigation.clone());
		let game = Game::new(BuildFlags::default(), Rc::new(MemoryPrefs::new()));
		provide_context(game);
		block_on(game.open());
		game
	});
	let nav_for_render = navigation.clone();
	let scope_for_render = Rc::clone(&scope);
	let mut tester = WidgetTester::new(
		move || scope_for_render.render(|| nav_for_render.render()),
		390.0,
		844.0,
	)
	.expect("mounts");

	// While the arrival curtain is up, Back is consumed without navigating.
	assert!(!game.arrived.get_untracked());
	assert!(handle_system_back(game));

	// Lift the arrival and bind a reaper so we are in the sanctum.
	game.arrived.set(true);
	block_on(async {
		let keeper = game.keeper().expect("local keeper");
		let bound = keeper.take_oath().await.expect("oath");
		game.books.set(reapers_app::state::Books::Open(bound));
	});
	tester.pump();

	// At the root with no overlay, Back returns false (let the OS exit).
	assert!(!handle_system_back(game));

	// With an overlay open, Back dismisses the overlay instead of exiting.
	game.overlay.set(Some(Overlay::Unreachable));
	tester.pump();
	assert!(handle_system_back(game));
	assert_eq!(game.overlay.get_untracked(), None);

	// Inside Studio with the Book of Becoming open, Back closes the book first,
	// then leaves Studio on the second Back.
	navigation.navigate(Route::Studio);
	tester.pump();
	tester
		.tap(&Finder::test_id("studio-book-open"))
		.expect("open book");
	tester.pump();
	assert!(tester.find(&Finder::test_id("studio-book")).is_ok());
	assert!(handle_system_back(game));
	tester.pump();
	assert!(tester.find(&Finder::test_id("studio-view")).is_ok());
	assert!(handle_system_back(game));
	tester.pump();
	assert_eq!(navigation.current(), Route::Index);

	// Inside a practice hunt, Back leaves cleanly via hunt::leave and returns
	// to the sanctum.
	navigation.navigate(Route::Practice);
	tester.pump();
	assert!(tester.find(&Finder::test_id("hunt-view")).is_ok());
	assert!(handle_system_back(game));
	tester.pump();
	assert_eq!(navigation.current(), Route::Index);
}

#[test]
fn overlapping_attempts_keep_busy_refcounted_until_all_finish() {
	let owner = Owner::new();
	let game = owner.with(|| {
		let game = Game::new(BuildFlags::default(), Rc::new(MemoryPrefs::new()));
		provide_context(game);
		block_on(game.open());
		game
	});

	let (tx1, rx1) =
		futures::channel::oneshot::channel::<Result<i32, reapers_app::backend::KeeperError>>();
	let (tx2, rx2) =
		futures::channel::oneshot::channel::<Result<i32, reapers_app::backend::KeeperError>>();
	let done1 = Rc::new(Cell::new(false));
	let done2 = Rc::new(Cell::new(false));

	let d1 = Rc::clone(&done1);
	game.attempt(
		move |_| async move { rx1.await.expect("sender alive") },
		move |_| d1.set(true),
	);
	let d2 = Rc::clone(&done2);
	game.attempt(
		move |_| async move { rx2.await.expect("sender alive") },
		move |_| d2.set(true),
	);
	platform::run_until_stalled();
	assert!(game.busy.get_untracked());

	// Completing the first attempt must NOT clear `game.busy` while the second
	// is still pending.
	tx1.send(Ok(1)).expect("send 1");
	platform::run_until_stalled();
	assert!(done1.get());
	assert!(!done2.get());
	assert!(game.busy.get_untracked());

	// Once the second completes, `game.busy` returns to false.
	tx2.send(Ok(2)).expect("send 2");
	platform::run_until_stalled();
	assert!(done2.get());
	assert!(!game.busy.get_untracked());
}

struct StubTransport {
	replies: RefCell<Vec<HttpReply>>,
	urls: RefCell<Vec<String>>,
}

impl StubTransport {
	fn new(replies: Vec<HttpReply>) -> Self {
		Self {
			replies: RefCell::new(replies),
			urls: RefCell::new(Vec::new()),
		}
	}
}

#[async_trait(?Send)]
impl HttpTransport for StubTransport {
	async fn post(
		&self,
		url: &str,
		_headers: Vec<Header>,
		_body: Vec<u8>,
		_timeout: Duration,
	) -> Result<HttpReply, PlatformError> {
		self.urls.borrow_mut().push(url.to_owned());
		let mut replies = self.replies.borrow_mut();
		if replies.is_empty() {
			return Err(PlatformError::Failed("unexpected request".into()));
		}
		Ok(replies.remove(0))
	}
}

#[test]
fn server_api_preserves_refresh_token_on_rate_limit_and_forgets_on_401() {
	block_on(async {
		let secrets = Rc::new(MemoryPrefs::new());
		secrets
			.set("reapers.server.refresh", Value::from("keep-me"))
			.await
			.unwrap();

		// 429 Too Many Requests on /auth/refresh must preserve the stored token
		// and MUST NOT fall through to POST /auth/guest.
		let transport = Rc::new(StubTransport::new(vec![HttpReply {
			status: 429,
			body: b"slow down".to_vec(),
		}]));
		let api = ServerApi::new("https://keeper.invalid", transport.clone(), secrets.clone());
		assert!(api.load().await.is_err());
		assert_eq!(
			secrets.get("reapers.server.refresh").await.unwrap(),
			Some(Value::from("keep-me")),
			"429 must not wipe the refresh token"
		);
		assert_eq!(transport.urls.borrow().len(), 1);
		assert!(transport.urls.borrow()[0].ends_with("/auth/refresh"));

		// 401 Unauthorized on /auth/refresh DOES forget the invalid token and
		// opens a fresh guest account.
		let guest_body = serde_json::to_vec(&serde_json::json!({
			"account": "guest-1",
			"access_token": "fresh-access",
			"refresh_token": "fresh-refresh",
			"expires_in": 900
		}))
		.unwrap();
		let transport2 = Rc::new(StubTransport::new(vec![
			HttpReply {
				status: 401,
				body: b"expired".to_vec(),
			},
			HttpReply {
				status: 200,
				body: guest_body,
			},
			HttpReply {
				status: 500,
				body: b"boom".to_vec(),
			},
		]));
		let api2 = ServerApi::new(
			"https://keeper.invalid",
			transport2.clone(),
			secrets.clone(),
		);
		let _ = api2.load().await;
		assert_eq!(
			secrets.get("reapers.server.refresh").await.unwrap(),
			Some(Value::from("fresh-refresh")),
			"401 should rotate to a fresh guest session"
		);
	});
}

#[test]
fn local_backend_serializes_concurrent_mutations_without_losing_updates() {
	block_on(async {
		let prefs = Rc::new(MemoryPrefs::new());
		let backend = LocalBackend::new(prefs);
		let _ = backend.take_oath().await.expect("oath");

		let nft_a = Nft::new(NftCollection::DegenApes, 1);
		let nft_b = Nft::new(NftCollection::PeskyPenguins, 2);
		let (res_a, res_b) =
			futures::future::join(backend.acquire(nft_a), backend.acquire(nft_b)).await;
		res_a.expect("acquire A");
		res_b.expect("acquire B");

		let final_state = backend.load().await.expect("load");
		assert!(final_state.vault().contains(&nft_a));
		assert!(final_state.vault().contains(&nft_b));
	});
}

#[test]
fn icu_plurals_and_probe_localization_render_grammatically() {
	assert_eq!(En.roster_count(1), "1 reaper");
	assert_eq!(En.roster_count(3), "3 reapers");
	assert_eq!(En.soul_milestone(5, 1, "Unseen"), "Level 5 · 1 Unseen mark");
	assert_eq!(
		En.soul_milestone(12, 4, "Unseen"),
		"Level 12 · 4 Unseen marks"
	);
	assert_eq!(
		En.soul_next(5, 1, "Stillness", 0),
		"Next awakening: level 5 and 1 Stillness mark. You have 0."
	);
	assert_eq!(
		En.soul_next(12, 2, "Stillness", 1),
		"Next awakening: level 12 and 2 Stillness marks. You have 1."
	);
	assert_eq!(
		En.gift_return_light_effect(1),
		"Five hits in cadence prepare a light that forgives your next flinch, once per hunt."
	);
	assert_eq!(
		En.gift_return_light_effect(2),
		"Five hits in cadence prepare a light that forgives your next flinch, up to 2 times per hunt."
	);
	assert_eq!(
		En.gift_unbroken_effect(1),
		"One miss preserves cadence each hunt. Its timer keeps running."
	);
	assert_eq!(
		En.gift_unbroken_effect(3),
		"Up to 3 misses preserve cadence each hunt. Its timer keeps running."
	);
	assert_ne!(En.probe_title(), "");
	assert_ne!(En.hunt_retreat(), "");
	assert_ne!(En.hunt_settle_failed_title(), "");
}

#[test]
fn depth_probe_renders_localized_copy_and_mounts_cleanly() {
	let owner = Owner::new();
	let scope = Rc::new(RenderScope::new(owner.clone()));
	owner.with(|| {
		L10n::new(&[]).provide();
	});
	let scope_for_render = Rc::clone(&scope);
	let tester = WidgetTester::new(
		move || scope_for_render.render(reapers_app::dev::probe::page),
		390.0,
		844.0,
	)
	.expect("depth probe mounts");
	assert!(tester.find(&Finder::test_id("depth-probe")).is_ok());
	assert!(tester.find(&Finder::test_id("probe-start")).is_ok());
	assert!(tester.find(&Finder::text(En.probe_title())).is_ok());
}

/// Answers each request with the next reply, keeping the URL and headers
/// it came with.
struct RecordingTransport {
	replies: RefCell<Vec<HttpReply>>,
	requests: RefCell<Vec<(String, Vec<Header>)>>,
}

#[async_trait(?Send)]
impl HttpTransport for RecordingTransport {
	async fn post(
		&self,
		url: &str,
		headers: Vec<Header>,
		_body: Vec<u8>,
		_timeout: Duration,
	) -> Result<HttpReply, PlatformError> {
		self.requests.borrow_mut().push((url.to_owned(), headers));
		let mut replies = self.replies.borrow_mut();
		if replies.is_empty() {
			return Err(PlatformError::Failed("unexpected request".into()));
		}
		Ok(replies.remove(0))
	}
}

#[test]
fn a_hunt_sent_again_carries_the_same_idempotency_key() {
	block_on(async {
		let guest = serde_json::to_vec(&serde_json::json!({
			"account": "guest-1",
			"access_token": "access",
			"refresh_token": "refresh",
			"expires_in": 900
		}))
		.unwrap();
		let lost = || {
			HttpReply {
				status: 503,
				body: b"the answer was lost".to_vec(),
			}
		};
		let transport = Rc::new(RecordingTransport {
			replies: RefCell::new(vec![
				HttpReply {
					status: 200,
					body: guest,
				},
				lost(),
				lost(),
				lost(),
			]),
			requests: RefCell::new(Vec::new()),
		});
		let api = ServerApi::new(
			"https://keeper.invalid",
			transport.clone(),
			Rc::new(MemoryPrefs::new()),
		);

		assert!(api.settle_hunt(7, vec![1, 2, 3]).await.is_err());
		assert!(api.settle_hunt(7, vec![1, 2, 3]).await.is_err());
		assert!(api.settle_hunt(7, vec![1, 2, 4]).await.is_err());

		let keys: Vec<Option<String>> = transport
			.requests
			.borrow()
			.iter()
			.filter(|(url, _)| url.ends_with("/api/settle_hunt"))
			.map(|(_, headers)| {
				headers
					.iter()
					.find(|(name, _)| *name == "idempotency-key")
					.map(|(_, value)| value.clone())
			})
			.collect();

		assert_eq!(keys.len(), 3);
		assert!(keys[0].is_some(), "settle_hunt carries an idempotency key");
		assert_eq!(keys[0], keys[1], "the same hunt, the same key");
		assert_ne!(keys[0], keys[2], "another hunt, another key");
	});
}
