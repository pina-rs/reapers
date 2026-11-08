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
