//! End-to-end HTTP + PostgreSQL performance benchmarks for `reapers_server`.
//!
//! Run with:
//! `REAPERS_PG_BIN=/usr/lib/postgresql/16/bin cargo test --release -p reapers_server --test perf_bench -- --ignored --nocapture`

mod support;

use std::time::Instant;

use axum::Router;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use http_body_util::BodyExt;
use leptoskit_auth::Keys;
use reapers_api::auth::SignedIn;
use reapers_rules::GeoFix;
use reapers_rules::GeoPoint;
use reapers_rules::PlayerState;
use reapers_rules::Timestamp;
use reapers_server::chain::Gateway;
use reapers_server::config::RunMode;
use reapers_server::server::Options;
use reapers_server::signer::ReceiptSigner;
use serde_json::Value;
use serde_json::json;
use support::*;
use tower::ServiceExt;

fn median(samples: &mut [f64]) -> f64 {
	samples.sort_by(f64::total_cmp);
	samples[samples.len() / 2]
}

async fn post_raw(
	router: &Router,
	path: &str,
	token: Option<&str>,
	idempotency: Option<&str>,
	accept_encoding: Option<&str>,
	body_bytes: Vec<u8>,
) -> (StatusCode, Vec<u8>, Option<String>) {
	let mut req = Request::post(path).header("content-type", "application/json");
	if let Some(t) = token {
		req = req.header("authorization", format!("Bearer {t}"));
	}
	if let Some(key) = idempotency {
		req = req.header("idempotency-key", key);
	}
	if let Some(enc) = accept_encoding {
		req = req.header("accept-encoding", enc);
	}
	let resp = router
		.clone()
		.oneshot(req.body(Body::from(body_bytes)).expect("req"))
		.await
		.expect("resp");
	let status = resp.status();
	let encoding = resp
		.headers()
		.get("content-encoding")
		.and_then(|v| v.to_str().ok())
		.map(str::to_owned);
	let bytes = resp.into_body().collect().await.expect("body").to_bytes();
	(status, bytes.to_vec(), encoding)
}

async fn post_json(
	router: &Router,
	path: &str,
	token: Option<&str>,
	key: Option<&str>,
	body: Value,
) -> (StatusCode, Value, usize) {
	let raw = serde_json::to_vec(&body).expect("ser");
	let (status, bytes, _) = post_raw(router, path, token, key, None, raw).await;
	let len = bytes.len();
	let val = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
	(status, val, len)
}

#[tokio::test]
#[ignore = "performance benchmark, run explicitly in --release"]
async fn server_http_end_to_end_benchmark() {
	let db = database().await;
	let reapers = reapers_server::server::start(
		&db,
		Keys::single("bench", [7; 32]),
		Options {
			mode: RunMode::Test,
			gateway: Gateway::new(solana(), Chain::default()),
			signer: ReceiptSigner::development(),
			hall: quick_hall(),
			origin: None,
		},
	)
	.expect("start");
	let router = reapers.router;

	// Guest sign-in
	let (st, body, _) = post_json(&router, reapers_api::auth::GUEST, None, None, json!(null)).await;
	assert_eq!(st, StatusCode::OK);
	let signed: SignedIn = serde_json::from_value(body).expect("signed");
	let token = signed.access_token.as_str();

	// Take oath + acquire + release + buy weapon + settle a hunt to populate realistic state
	let (st, ..) = post_json(
		&router,
		"/api/take_oath",
		Some(token),
		Some("oath-1"),
		json!({}),
	)
	.await;
	assert_eq!(st, StatusCode::OK);

	let (st, state_val, _) = post_json(
		&router,
		"/api/acquire",
		Some(token),
		Some("acq-1"),
		json!({"nft": "degenApes:1"}),
	)
	.await;
	assert_eq!(st, StatusCode::OK);
	let _ = state_val;

	let (st, state_val, _) = post_json(
		&router,
		"/api/release",
		Some(token),
		Some("rel-1"),
		json!({"nft": "degenApes:1"}),
	)
	.await;
	assert_eq!(st, StatusCode::OK);
	let mut state_json = state_val;
	if let Some(sightings) = state_json["sightings"].as_array_mut() {
		for s in sightings {
			s["surfacesAt"] = json!("2026-01-01T00:00:00.000Z");
		}
	}
	// Populate a realistic active player with 40 codex entries.
	let absorptions: Vec<Value> = reapers_rules::market_listings()
		.iter()
		.enumerate()
		.map(|(i, nft)| {
			json!({
				"nft": nft.id(),
				"grade": "clean",
				"feeding": "rich",
				"xp": 180,
				"anima": 45,
				"nonce": (i + 1) as u64,
				"at": Timestamp::from_millis(1_790_000_000_000 + (i as i64) * 60_000).to_iso8601(),
				"signature": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
			})
		})
		.collect();
	if let Some(reapers) = state_json["reapers"].as_array_mut()
		&& let Some(starter) = reapers.first_mut()
	{
		starter["codex"] = Value::Array(absorptions);
		starter["xp"] = json!(18_000);
	}
	let player: PlayerState = serde_json::from_value(state_json).expect("player");
	reapers_db::PlayerBooks::update()
		.set(|row| row.state, leptoskit_db::Json(player.clone()))
		.run(&db.system("bench seed"))
		.await
		.expect("seed books");

	let sighting = player.sightings()[0];
	let weapon_id = player.weapons()[0].id();

	// Build a valid hunt log for this sighting
	let hunt_log = play(&player, &sighting);
	let encoded_log = reapers_api::player::encode_log(&hunt_log.encode());
	let settle_req_body = serde_json::to_vec(&json!({
		"sighting": sighting.id,
		"log": encoded_log,
	}))
	.expect("settle body");
	let settle_req_bytes = settle_req_body.len();

	// Measure M20: /api/settle_hunt (with Accept-Encoding: br, gzip)
	let t0 = Instant::now();
	let (st, settle_resp_bytes, settle_enc) = post_raw(
		&router,
		"/api/settle_hunt",
		Some(token),
		Some("settle-1"),
		Some("br, gzip"),
		settle_req_body,
	)
	.await;
	let settle_ms = t0.elapsed().as_secs_f64() * 1e3;
	assert_eq!(st, StatusCode::OK);

	// Measure M17: /api/load (20 iterations, measuring both latency and wire bytes with Accept-Encoding: br, gzip)
	let mut load_ms = Vec::with_capacity(20);
	let mut load_wire_bytes = 0usize;
	let mut load_enc = None;
	for i in 0..20 {
		let key = format!("load-{i}");
		let t0 = Instant::now();
		let (st, bytes, enc) = post_raw(
			&router,
			"/api/load",
			Some(token),
			Some(&key),
			Some("br, gzip"),
			b"{}".to_vec(),
		)
		.await;
		assert_eq!(st, StatusCode::OK);
		load_ms.push(t0.elapsed().as_secs_f64() * 1e3);
		load_wire_bytes = bytes.len();
		load_enc = enc;
	}
	let (_, raw_bytes, _) = post_raw(
		&router,
		"/api/load",
		Some(token),
		Some("load-raw"),
		None,
		b"{}".to_vec(),
	)
	.await;
	let load_raw_bytes = raw_bytes.len();

	// Measure M18: /api/wield (20 iterations)
	let wield_body = serde_json::to_vec(&json!({"weapon": weapon_id})).expect("wield");
	let mut wield_ms = Vec::with_capacity(20);
	let mut wield_wire_bytes = 0usize;
	for i in 0..20 {
		let key = format!("wield-{i}");
		let t0 = Instant::now();
		let (st, bytes, _) = post_raw(
			&router,
			"/api/wield",
			Some(token),
			Some(&key),
			Some("br, gzip"),
			wield_body.clone(),
		)
		.await;
		assert_eq!(st, StatusCode::OK);
		wield_ms.push(t0.elapsed().as_secs_f64() * 1e3);
		wield_wire_bytes = bytes.len();
	}

	// Measure M19: /api/update_location (20 iterations)
	let mut locate_ms = Vec::with_capacity(20);
	let now_ms = chrono::Utc::now().timestamp_millis();
	for i in 0..20_i64 {
		let fix = GeoFix {
			at: GeoPoint::new(37.7749 + i as f64 * 0.0001, -122.4194),
			accuracy: 8.0,
			observed_at: Timestamp::from_millis(now_ms + i * 100),
		};
		let body = serde_json::to_vec(&json!({"fix": fix})).expect("fix");
		let key = format!("loc-{i}");
		let t0 = Instant::now();
		let (st, ..) = post_raw(
			&router,
			"/api/update_location",
			Some(token),
			Some(&key),
			Some("br, gzip"),
			body,
		)
		.await;
		assert_eq!(st, StatusCode::OK);
		locate_ms.push(t0.elapsed().as_secs_f64() * 1e3);
	}

	println!("=== reapers_server perf_bench (release, PG16) ===");
	println!(
		"M17_http_load_ms:          {:.3} ms | wire: {} B (raw: {} B, enc: {:?})",
		median(&mut load_ms),
		load_wire_bytes,
		load_raw_bytes,
		load_enc
	);
	println!(
		"M18_http_wield_ms:         {:.3} ms | wire: {} B",
		median(&mut wield_ms),
		wield_wire_bytes
	);
	println!(
		"M19_http_locate_ms:        {:.3} ms",
		median(&mut locate_ms)
	);
	println!(
		"M20_http_settle_hunt_ms:   {:.3} ms | req: {} B | resp: {} B (enc: {:?})",
		settle_ms,
		settle_req_bytes,
		settle_resp_bytes.len(),
		settle_enc
	);
}
