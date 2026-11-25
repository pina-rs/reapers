//! The keeper as a client reaches it: one router, over HTTP, with sessions
//! from the guest and wallet sign-in routes that `leptoskit_auth` recognizes,
//! refreshes and ends as its own.

mod support;

use axum::Router;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http_body_util::BodyExt;
use leptoskit_auth::Keys;
use reapers_api::auth::SignedIn;
use reapers_rules::PlayerState;
use reapers_rules::Refusal;
use reapers_rules::Sighting;
use reapers_server::chain::Gateway;
use reapers_server::config::RunMode;
use reapers_server::server::Options;
use reapers_server::signer::ReceiptSigner;
use serde_json::Value;
use serde_json::json;
use solana_kit_core::crypto::Keypair;
use support::*;
use tower::ServiceExt;

async fn keeper() -> Router {
	keeper_for(None).await
}

/// A keeper whose pages are served from `origin`, when there is one.
async fn keeper_for(origin: Option<&str>) -> Router {
	let db = database().await;
	let reapers = reapers_server::server::start(
		&db,
		Keys::single("test", [3; 32]),
		Options {
			mode: RunMode::Test,
			gateway: Gateway::new(solana(), Chain::default()),
			signer: ReceiptSigner::development(),
			hall: quick_hall(),
			origin: origin.map(str::to_owned),
		},
	)
	.expect("the keeper starts");

	reapers.router
}

/// Posts `body` to `path`, with `token` as the bearer when there is one.
async fn post(
	router: &Router,
	path: &str,
	token: Option<&str>,
	body: Value,
) -> (StatusCode, Value) {
	let mut request = Request::post(path).header("content-type", "application/json");

	if let Some(token) = token {
		request = request.header("authorization", format!("Bearer {token}"));
	}

	let response = router
		.clone()
		.oneshot(
			request
				.body(Body::from(body.to_string()))
				.expect("a request"),
		)
		.await
		.expect("an answer");
	let status = response.status();
	let bytes = response
		.into_body()
		.collect()
		.await
		.expect("a body")
		.to_bytes();
	let body = serde_json::from_slice(&bytes)
		.unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));

	(status, body)
}

async fn guest(router: &Router) -> SignedIn {
	let (status, body) = post(router, reapers_api::auth::GUEST, None, json!(null)).await;

	assert_eq!(status, StatusCode::OK, "{body}");

	serde_json::from_value(body).expect("tokens")
}

#[tokio::test]
async fn a_guest_signs_in_at_once_and_plays_as_the_session_says() {
	let router = keeper().await;
	let signed_in = guest(&router).await;
	let (status, body) = post(
		&router,
		"/api/load",
		Some(&signed_in.access_token),
		json!({}),
	)
	.await;

	assert_eq!(status, StatusCode::OK, "{body}");

	let state: PlayerState = serde_json::from_value(body).expect("a player");

	assert!(!state.oath_taken());
	assert!(signed_in.expires_in > 0);

	let (status, body) = post(
		&router,
		"/api/take_oath",
		Some(&signed_in.access_token),
		json!({}),
	)
	.await;

	assert_eq!(status, StatusCode::OK, "{body}");

	// Another guest has books of their own.
	let other = guest(&router).await;
	let (_, body) = post(&router, "/api/load", Some(&other.access_token), json!({})).await;

	assert_ne!(other.account, signed_in.account);
	assert!(
		!serde_json::from_value::<PlayerState>(body)
			.unwrap()
			.oath_taken()
	);
}

#[tokio::test]
async fn nobody_signed_in_is_turned_away_and_a_made_up_token_signs_in_nobody() {
	let router = keeper().await;

	for token in [None, Some("not-a-session")] {
		let (status, _) = post(&router, "/api/load", token, json!({})).await;

		assert_eq!(status, StatusCode::UNAUTHORIZED, "{token:?}");
	}
}

#[tokio::test]
async fn a_refusal_reaches_the_game_by_name() {
	let router = keeper().await;
	let signed_in = guest(&router).await;
	let token = Some(signed_in.access_token.as_str());

	post(&router, "/api/take_oath", token, json!({})).await;

	let (status, body) = post(&router, "/api/take_oath", token, json!({})).await;
	let message = body
		.as_str()
		.map_or_else(|| body.to_string(), str::to_owned);

	assert_eq!(status, StatusCode::CONFLICT);
	assert!(
		message.contains("refused:oathAlreadyTaken"),
		"the refusal is named: {message}"
	);

	let named = message
		.split('|')
		.next_back()
		.and_then(reapers_api::refusal_of);

	assert_eq!(named, Some(Refusal::OathAlreadyTaken));
}

#[tokio::test]
async fn leptoskit_auth_refreshes_a_guest_session_as_its_own() {
	let router = keeper().await;
	let signed_in = guest(&router).await;
	let (status, body) = post(
		&router,
		reapers_api::auth::REFRESH,
		None,
		json!({ "refresh_token": signed_in.refresh_token }),
	)
	.await;

	assert_eq!(status, StatusCode::OK, "{body}");

	let access = body["access_token"].as_str().expect("a new access token");
	let (status, _) = post(&router, "/api/load", Some(access), json!({})).await;

	assert_eq!(status, StatusCode::OK);
	assert_ne!(access, signed_in.access_token);

	let (status, _) = post(
		&router,
		"/api/load",
		Some(&signed_in.access_token),
		json!({}),
	)
	.await;

	assert_eq!(
		status,
		StatusCode::UNAUTHORIZED,
		"the traded access token stops working"
	);
}

#[tokio::test]
async fn a_wallet_signs_in_through_its_route() {
	let router = keeper().await;
	let signed_in = guest(&router).await;
	let token = Some(signed_in.access_token.as_str());
	let wallet = Keypair::from_seed(&[1; 32]);
	let (status, challenge) = post(
		&router,
		"/api/wallet_challenge",
		token,
		json!({ "address": wallet.address().to_string() }),
	)
	.await;

	assert_eq!(status, StatusCode::OK, "{challenge}");

	let message = challenge["message"].as_str().expect("a message");
	let id = challenge["challenge_id"].as_str().expect("an id");
	let forged = STANDARD.encode(
		Keypair::from_seed(&[2; 32])
			.sign(message.as_bytes())
			.as_bytes(),
	);
	let (status, body) = post(
		&router,
		reapers_api::auth::WALLET,
		token,
		json!({ "challenge_id": id, "signature": forged }),
	)
	.await;

	assert_eq!(status, StatusCode::CONFLICT);
	assert_eq!(body["error"], "refused:walletUnproven");

	let (status, _) = post(
		&router,
		reapers_api::auth::WALLET,
		None,
		json!({ "challenge_id": id, "signature": forged }),
	)
	.await;

	assert_eq!(status, StatusCode::UNAUTHORIZED);

	let signature = STANDARD.encode(wallet.sign(message.as_bytes()).as_bytes());
	let (status, body) = post(
		&router,
		reapers_api::auth::WALLET,
		token,
		json!({ "challenge_id": id, "signature": signature }),
	)
	.await;

	assert_eq!(status, StatusCode::OK, "{body}");

	let linked: SignedIn = serde_json::from_value(body).expect("tokens");

	assert_eq!(linked.account, signed_in.account);

	let (status, _) = post(&router, "/api/load", Some(&linked.access_token), json!({})).await;

	assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn the_receipt_key_is_public_to_any_player() {
	let router = keeper().await;
	let signed_in = guest(&router).await;
	let (status, key) = post(
		&router,
		"/api/receipt_key",
		Some(&signed_in.access_token),
		json!({}),
	)
	.await;

	assert_eq!(status, StatusCode::OK, "{key}");
	assert_eq!(
		key["address"],
		ReceiptSigner::development().address().to_string()
	);
	assert_eq!(key["development"], true);
}

#[tokio::test]
async fn the_web_game_on_its_own_origin_may_call_the_keeper() {
	const ORIGIN: &str = "http://127.0.0.1:3010";

	let router = keeper_for(Some(ORIGIN)).await;
	let preflight = router
		.clone()
		.oneshot(
			Request::options("/api/load")
				.header("origin", ORIGIN)
				.header("access-control-request-method", "POST")
				.header(
					"access-control-request-headers",
					"authorization,content-type,idempotency-key",
				)
				.body(Body::empty())
				.expect("a request"),
		)
		.await
		.expect("an answer");
	let allowed = |name: &str| {
		preflight
			.headers()
			.get(name)
			.and_then(|value| value.to_str().ok())
			.map(str::to_owned)
	};

	assert!(preflight.status().is_success());
	assert_eq!(
		allowed("access-control-allow-origin").as_deref(),
		Some(ORIGIN)
	);
	assert!(
		allowed("access-control-allow-headers")
			.is_some_and(|headers| headers.contains("authorization"))
	);

	let signed_in = guest(&router).await;
	let (status, _) = post(
		&router,
		"/api/load",
		Some(&signed_in.access_token),
		json!({}),
	)
	.await;

	assert_eq!(status, StatusCode::OK);
}

/// A keeper over HTTP, and a guest on it with one released husk surfaced
/// and ready to hunt: the router, the guest's bearer token, the books and
/// the sighting.
async fn ready_to_hunt() -> (Router, String, PlayerState, Sighting) {
	let db = database().await;
	let reapers = reapers_server::server::start(
		&db,
		Keys::single("test", [3; 32]),
		Options {
			mode: RunMode::Test,
			gateway: Gateway::new(solana(), Chain::default()),
			signer: ReceiptSigner::development(),
			hall: quick_hall(),
			origin: None,
		},
	)
	.expect("the keeper starts");
	let router = reapers.router;
	let signed_in = guest(&router).await;
	let token = Some(signed_in.access_token.as_str());

	let (status, _) = post(&router, "/api/take_oath", token, json!({})).await;
	assert_eq!(status, StatusCode::OK);

	let (status, _) = post(
		&router,
		"/api/acquire",
		token,
		json!({ "nft": "degenApes:1" }),
	)
	.await;
	assert_eq!(status, StatusCode::OK);

	let (status, body) = post(
		&router,
		"/api/release",
		token,
		json!({ "nft": "degenApes:1" }),
	)
	.await;
	assert_eq!(status, StatusCode::OK, "{body}");

	let released: PlayerState = serde_json::from_value(body).expect("released state");
	let sighting = released.sightings()[0];

	// Surface the sighting in Postgres so settle_hunt can run immediately.
	let mut state_json = serde_json::to_value(&released).expect("serialize state");
	for item in state_json["sightings"].as_array_mut().expect("sightings") {
		item["surfacesAt"] = json!("2026-01-01T00:00:00.000Z");
	}
	let surfaced: PlayerState = serde_json::from_value(state_json).expect("surfaced state");
	reapers_server::tables::PlayerBooks::update()
		.set(|books| books.state, leptoskit_db::Json(surfaced.clone()))
		.run(&db)
		.await
		.expect("surface sighting");

	(router, signed_in.access_token, surfaced, sighting)
}

/// Posts `settle_hunt` with `payload`, under the `Idempotency-Key` `key`.
async fn settle_with_key(
	router: &Router,
	token: &str,
	key: &str,
	payload: &Value,
) -> (StatusCode, Value) {
	let request = Request::post("/api/settle_hunt")
		.header("content-type", "application/json")
		.header("authorization", format!("Bearer {token}"))
		.header("idempotency-key", key)
		.body(Body::from(payload.to_string()))
		.expect("request");
	let response = router.clone().oneshot(request).await.expect("response");
	let status = response.status();
	let bytes = response.into_body().collect().await.unwrap().to_bytes();
	let value = serde_json::from_slice::<Value>(&bytes)
		.unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));

	(status, value)
}

#[tokio::test]
async fn settle_hunt_replays_cached_result_for_retried_idempotency_key() {
	let (router, token, surfaced, sighting) = ready_to_hunt().await;
	let payload = json!({
		"sighting": sighting.id,
		"log": packed(&play(&surfaced, &sighting)),
	});
	let key = "0123456789abcdef0123456789abcdef";
	let (first_status, first_body) = settle_with_key(&router, &token, key, &payload).await;
	assert_eq!(first_status, StatusCode::OK, "{first_body}");
	assert!(
		!first_body["settlement"]["outcome"]["grade"].is_null(),
		"the hunt absorbed the husk: {first_body}"
	);

	// Simulate a dropped HTTP response and client retry with the same Idempotency-Key.
	let (retry_status, retry_body) = settle_with_key(&router, &token, key, &payload).await;
	assert_eq!(
		retry_status,
		StatusCode::OK,
		"retrying settle_hunt with the same Idempotency-Key must replay the cached settlement instead of returning {retry_status}: {retry_body}"
	);
	assert_eq!(retry_body, first_body);

	// The same key for another call is refused rather than answered with this one.
	let other = json!({ "sighting": sighting.id + 1, "log": payload["log"] });
	let (status, body) = settle_with_key(&router, &token, key, &other).await;
	assert_eq!(status, StatusCode::CONFLICT, "{body}");
}

#[tokio::test]
async fn an_escape_retried_with_the_same_idempotency_key_is_lost_once() {
	let (router, token, surfaced, sighting) = ready_to_hunt().await;
	let payload = json!({
		"sighting": sighting.id,
		"log": packed(&escape(&surfaced, &sighting)),
	});
	let key = "fedcba9876543210fedcba9876543210";
	let (first_status, first_body) = settle_with_key(&router, &token, key, &payload).await;
	assert_eq!(first_status, StatusCode::OK, "{first_body}");
	assert!(
		!first_body["settlement"]["outcome"]["escape"].is_null(),
		"the husk got away: {first_body}"
	);

	let (retry_status, retry_body) = settle_with_key(&router, &token, key, &payload).await;
	assert_eq!(retry_status, StatusCode::OK, "{retry_body}");
	assert_eq!(retry_body, first_body);

	let (status, body) = post(&router, "/api/load", Some(&token), json!({})).await;
	assert_eq!(status, StatusCode::OK, "{body}");
	let state: PlayerState = serde_json::from_value(body).expect("a player");

	assert_eq!(state.hunts_lost(), 1, "the escape is counted once");
}

/// A phone that times out sends the hunt again while the first request is
/// still settling. Both answers are the one settlement, and the escape is
/// counted once.
#[tokio::test]
async fn an_escape_sent_twice_at_once_with_one_key_is_lost_once() {
	let (router, token, surfaced, sighting) = ready_to_hunt().await;
	let payload = json!({
		"sighting": sighting.id,
		"log": packed(&escape(&surfaced, &sighting)),
	});
	let key = "00112233445566778899aabbccddeeff";
	let ((first_status, first_body), (second_status, second_body)) = tokio::join!(
		settle_with_key(&router, &token, key, &payload),
		settle_with_key(&router, &token, key, &payload),
	);

	assert_eq!(first_status, StatusCode::OK, "{first_body}");
	assert_eq!(second_status, StatusCode::OK, "{second_body}");
	assert_eq!(first_body, second_body);

	let (status, body) = post(&router, "/api/load", Some(&token), json!({})).await;
	assert_eq!(status, StatusCode::OK, "{body}");
	let state: PlayerState = serde_json::from_value(body).expect("a player");

	assert_eq!(state.hunts_lost(), 1, "the escape is counted once");
}
