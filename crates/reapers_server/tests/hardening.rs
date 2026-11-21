//! Regression tests for backend hardening: auth route scoping, security
//! headers, escrow cleanup on NFT transfer, resilient URI mismatch handling,
//! forged rift report rejection, slow-consumer rift beats, wallet sign-in
//! account lockout/guest revocation, and deployed config guards.

mod support;

use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use futures::StreamExt;
use http_body_util::BodyExt;
use leptoskit_auth::Keys;
use leptoskit_db::Connection;
use leptoskit_db::Statement;
use leptoskit_db::Value as DbValue;
use leptoskit_rpc as rpc;
use reapers_api::auth::SignedIn;
use reapers_api::auth::WalletSignIn;
use reapers_api::chain::ImportReaper;
use reapers_api::chain::WalletChallenge;
use reapers_api::keeper::Keeper as _;
use reapers_api::player::Acquire;
use reapers_api::player::Load;
use reapers_api::player::Release;
use reapers_api::player::SelectReaper;
use reapers_api::player::SettleHunt;
use reapers_api::player::TakeOath;
use reapers_rules::CaptureGrade;
use reapers_rules::HuntDeeds;
use reapers_rules::HuntOutcome;
use reapers_rules::ReaperGenome;
use reapers_rules::Refusal;
use reapers_rules::RiftPhase;
use reapers_rules::RiftReport;
use reapers_server::chain::Gateway;
use reapers_server::config::HuskCollection;
use reapers_server::config::RunMode;
use reapers_server::config::Secrets;
use reapers_server::config::SolanaConfig;
use reapers_server::config::validate_origin;
use reapers_server::escrow;
use reapers_server::escrow::Escrow;
use reapers_server::escrow::HuskKind;
use reapers_server::keeper::Shared;
use reapers_server::rift::RiftHall;
use reapers_server::server::Options;
use reapers_server::sign_in::sign_in_wallet;
use reapers_server::signer::ReceiptSigner;
use reapers_server::tables::HuskEscrow;
use serde_json::Value;
use serde_json::json;
use solana_kit_core::Address;
use solana_kit_core::crypto::Keypair;
use solana_kit_mpl_core::UpdateAuthority;
use support::*;
use tower::ServiceExt;

async fn post_http(
	router: &Router,
	path: &str,
	token: Option<&str>,
	body: Value,
) -> (StatusCode, axum::http::HeaderMap, Value) {
	let mut request = Request::post(path).header("content-type", "application/json");
	if let Some(token) = token {
		request = request.header("authorization", format!("Bearer {token}"));
	}
	let response = router
		.clone()
		.oneshot(request.body(Body::from(body.to_string())).unwrap())
		.await
		.unwrap();
	let status = response.status();
	let headers = response.headers().clone();
	let bytes = response.into_body().collect().await.unwrap().to_bytes();
	let body = serde_json::from_slice(&bytes)
		.unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
	(status, headers, body)
}

#[test]
fn deployed_config_guards_reject_dev_secrets_weak_keys_demo_husks_and_bad_origins() {
	let dev_secrets = Secrets::of([
		("database", "postgres://keeper@db/reapers"),
		("authKeys", "2026-11:AAAA"),
		("receiptSigningKey", ReceiptSigner::DEVELOPMENT_SECRET),
	]);
	assert!(dev_secrets.validate_strength(RunMode::Production).is_err());
	assert!(dev_secrets.validate_strength(RunMode::Staging).is_err());
	assert!(dev_secrets.validate_strength(RunMode::Development).is_ok());

	let short_secrets = Secrets::of([("receiptSigningKey", "too-short")]);
	assert!(
		short_secrets
			.validate_strength(RunMode::Production)
			.is_err()
	);

	let strong_secrets = Secrets::of([(
		"receiptSigningKey",
		"0123456789abcdef0123456789abcdef01234567",
	)]);
	assert!(
		strong_secrets
			.validate_strength(RunMode::Production)
			.is_ok()
	);

	assert!(validate_origin(None, RunMode::Production).is_err());
	assert!(validate_origin(Some("*"), RunMode::Development).is_err());
	assert!(validate_origin(Some("http://reapers.example"), RunMode::Production).is_err());
	assert!(validate_origin(Some("https://reapers.example/path"), RunMode::Production).is_err());
	assert_eq!(
		validate_origin(Some("https://reapers.example/"), RunMode::Production).unwrap(),
		Some("https://reapers.example".to_owned())
	);

	// Demo husks manifest rejects duplicate collection slugs and duplicate asset addresses.
	let asset = |n: u8| Address::from_bytes([n; 32]).to_string();
	let dup_asset = serde_json::json!({
		"collections": [{
			"slug": "thugbirdz",
			"address": asset(100),
			"assets": vec![asset(1); 8],
		}]
	})
	.to_string();
	assert!(HuskCollection::from_manifest(&dup_asset).is_err());

	let valid_manifest = serde_json::json!({
		"collections": [{
			"slug": "thugbirdz",
			"address": asset(100),
			"assets": (1..=8).map(asset).collect::<Vec<_>>(),
		}]
	})
	.to_string();
	let husks = HuskCollection::from_manifest(&valid_manifest).unwrap();
	let with_husks = SolanaConfig::from_vars(|_| None).unwrap().with_husks(husks);
	assert!(with_husks.validate_for_mode(RunMode::Development).is_ok());
	assert!(with_husks.validate_for_mode(RunMode::Production).is_err());
}

#[tokio::test]
async fn unused_auth_routes_are_not_mounted_and_security_headers_are_set() {
	let db = database().await;
	let reapers = reapers_server::server::start(
		&db,
		Keys::single("test", [9; 32]),
		Options {
			mode: RunMode::Test,
			gateway: Gateway::new(solana(), Chain::default()),
			signer: ReceiptSigner::development(),
			hall: quick_hall(),
			origin: None,
		},
	)
	.unwrap();
	let router = reapers.router;

	// Unused leptoskit_auth endpoints must answer 404.
	for path in [
		"/auth/email/send",
		"/auth/email/verify",
		"/auth/totp",
		"/auth/merge",
		"/auth/wallets/config",
		"/auth/passkey/sign-in/options",
	] {
		let (status, ..) = post_http(&router, path, None, json!({})).await;
		assert_eq!(status, StatusCode::NOT_FOUND, "{path} must not be mounted");
	}

	// Guest sign-in + sign-out + security headers all work.
	let (status, headers, body) =
		post_http(&router, reapers_api::auth::GUEST, None, json!(null)).await;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		headers
			.get("x-content-type-options")
			.and_then(|v| v.to_str().ok()),
		Some("nosniff")
	);
	assert!(
		headers
			.get("cache-control")
			.and_then(|v| v.to_str().ok())
			.is_some_and(|v| v.contains("no-store"))
	);

	let guest: SignedIn = serde_json::from_value(body).unwrap();
	let (status, ..) = post_http(
		&router,
		"/auth/sign-out",
		Some(&guest.access_token),
		json!({}),
	)
	.await;
	assert_eq!(status, StatusCode::NO_CONTENT);

	let (status, ..) = post_http(&router, "/api/load", Some(&guest.access_token), json!({})).await;
	assert_eq!(
		status,
		StatusCode::UNAUTHORIZED,
		"signed-out bearer token stops working immediately"
	);
}

async fn attune_for_rift(world: &World, player: leptoskit_db::Id<leptoskit_auth::Account>) {
	world.call(player, TakeOath {}).await.unwrap();
	world
		.call(
			player,
			Acquire {
				nft: "degenApes:1".into(),
			},
		)
		.await
		.unwrap();
	world
		.call(
			player,
			Release {
				nft: "degenApes:1".into(),
				escrow: None,
			},
		)
		.await
		.unwrap();
	world.surface_everything(player).await;
	let before = world.call(player, Load {}).await.unwrap();
	let sighting = before.sightings()[0];
	world
		.call(
			player,
			SettleHunt {
				sighting: sighting.id,
				log: packed(&play(&before, &sighting)),
			},
		)
		.await
		.unwrap();
}

#[tokio::test]
async fn forged_rift_reports_are_refused_and_lagged_stream_never_deadlocks() {
	let world = World::reading(
		RunMode::Test,
		RiftHall::new(Duration::from_millis(5), Some(60)),
		solana(),
	)
	.await;
	let player = world.player().await;
	attune_for_rift(&world, player).await;

	let shared = Shared(std::sync::Arc::clone(&world.keeper));
	let mut stream = shared
		.enter_rift(
			&rpc::QueryCx::new(world.db.clone(), world.identity(Some(player))),
			"londonBridge".to_owned(),
			false,
		)
		.await
		.unwrap();
	let action_cx = rpc::ActionCx::new(world.db.clone(), world.identity(Some(player)));

	shared
		.report_rift(&action_cx, RiftReport::readiness(true))
		.await
		.unwrap();

	loop {
		let frame = stream.next().await.expect("a frame").expect("ok");
		if frame.phase == RiftPhase::Fight {
			break;
		}
	}

	// Forged NaN, infinite, wildly inflated damage, and premature reap victory during Fight must fail.
	for forged in [
		RiftReport {
			ready: true,
			dealt: f64::NAN,
			outcome: None,
			standing: None,
		},
		RiftReport {
			ready: true,
			dealt: 1e300,
			outcome: None,
			standing: None,
		},
		RiftReport {
			ready: true,
			dealt: 0.0,
			outcome: Some(HuntOutcome::absorbed_with(
				CaptureGrade::Excellent,
				false,
				1,
				HuntDeeds::default(),
			)),
			standing: None,
		},
	] {
		assert_refused(
			shared.report_rift(&action_cx, forged).await,
			Refusal::RiftUnproven,
		);
	}

	// Simulate a very slow consumer: pause long enough for dozens of clock beats
	// to pass and the rift to reach a terminal phase, then drain the stream.
	tokio::time::sleep(Duration::from_millis(600)).await;
	let ended = tokio::time::timeout(Duration::from_secs(5), async {
		let mut last = None;
		while let Some(item) = stream.next().await {
			last = Some(item.expect("frame"));
		}
		last
	})
	.await
	.expect("lagged consumer must not deadlock");

	assert!(ended.is_some_and(|frame| frame.phase.is_over()));
}

fn put_reaper_nft(chain: &Chain, reaper_asset: Address, owner: Address, genome: ReaperGenome) {
	chain.put(
		&reaper_asset.to_string(),
		&solana_kit_mpl_core::PROGRAM.to_string(),
		&asset(
			owner,
			UpdateAuthority::Collection(COLLECTION.parse().unwrap()),
			&format!("https://reapers.example/reapers/{}.json", genome.code()),
		),
	);
}

async fn link_wallet_and_oath(
	world: &World,
	player: leptoskit_db::Id<leptoskit_auth::Account>,
	wallet: &Keypair,
) {
	let challenge = world
		.call(
			player,
			WalletChallenge {
				address: wallet.address().to_string(),
			},
		)
		.await
		.unwrap();
	sign_in_wallet(
		&world.keeper,
		player,
		&WalletSignIn {
			challenge_id: challenge.challenge_id,
			signature: STANDARD.encode(wallet.sign(challenge.message.as_bytes()).as_bytes()),
		},
	)
	.await
	.unwrap();
	world.call(player, TakeOath {}).await.unwrap();
}

#[tokio::test]
async fn losing_nft_reaper_frees_unabsorbed_escrow_and_mutated_uri_does_not_brick_account() {
	let world = World::new().await;
	let player = world.player().await;
	let wallet = Keypair::from_seed(&[11; 32]);
	let other_wallet = Keypair::from_seed(&[12; 32]);
	let reaper_asset = Address::from_bytes([21; 32]);
	let husk_mint = Address::from_bytes([22; 32]);
	let genome = ReaperGenome::shuffle(7);

	link_wallet_and_oath(&world, player, &wallet).await;

	put_reaper_nft(&world.chain, reaper_asset, wallet.address(), genome);
	world
		.call(
			player,
			ImportReaper {
				asset: reaper_asset.to_string(),
			},
		)
		.await
		.unwrap();
	world
		.call(
			player,
			SelectReaper {
				id: reaper_asset.to_string(),
			},
		)
		.await
		.unwrap();
	world
		.call(
			player,
			Acquire {
				nft: "degenApes:1".into(),
			},
		)
		.await
		.unwrap();

	let (escrow_pda, bump) = escrow::address_of(&husk_mint);
	world.chain.put(
		&escrow_pda.to_string(),
		&program().to_string(),
		&Escrow {
			wallet: wallet.address(),
			mint: husk_mint,
			kind: HuskKind::Core,
			bump,
		}
		.encode(),
	);
	world
		.call(
			player,
			Release {
				nft: "degenApes:1".into(),
				escrow: Some(husk_mint.to_string()),
			},
		)
		.await
		.unwrap();
	assert_eq!(HuskEscrow::query().count(&world.db).await.unwrap(), 1);

	// Transfer NFT reaper away: Load reconciles it out, returns husk to vault,
	// AND must delete the unabsorbed HuskEscrow so re-releasing succeeds.
	put_reaper_nft(&world.chain, reaper_asset, other_wallet.address(), genome);
	let reconciled = world.call(player, Load {}).await.unwrap();
	assert_eq!(reconciled.sightings(), []);
	assert_eq!(HuskEscrow::query().count(&world.db).await.unwrap(), 0);

	world
		.call(
			player,
			Release {
				nft: "degenApes:1".into(),
				escrow: Some(husk_mint.to_string()),
			},
		)
		.await
		.unwrap();
}

#[tokio::test]
async fn mutated_nft_uri_drops_asset_without_bricking_account() {
	let world = World::new().await;
	let player = world.player().await;
	let wallet = Keypair::from_seed(&[13; 32]);
	let reaper_asset = Address::from_bytes([23; 32]);
	let genome = ReaperGenome::shuffle(7);
	let other_genome = ReaperGenome::shuffle(99);

	link_wallet_and_oath(&world, player, &wallet).await;
	put_reaper_nft(&world.chain, reaper_asset, wallet.address(), genome);
	world
		.call(
			player,
			ImportReaper {
				asset: reaper_asset.to_string(),
			},
		)
		.await
		.unwrap();

	// Mutate its on-chain URI to a different genome: Load drops only that
	// asset instead of locking the player out of their books, while an
	// explicit re-import is still refused.
	put_reaper_nft(&world.chain, reaper_asset, wallet.address(), other_genome);
	let still_playable = world.call(player, Load {}).await.unwrap();
	assert_eq!(
		still_playable.reapers().len(),
		1,
		"mutated NFT drops from roster without bricking the account"
	);
	assert_refused(
		world
			.call(
				player,
				ImportReaper {
					asset: reaper_asset.to_string(),
				},
			)
			.await,
		Refusal::NotOwned,
	);
}

#[tokio::test]
async fn disabled_account_cannot_sign_in_with_wallet_and_switching_revokes_guest_session() {
	let db = database().await;
	let reapers = reapers_server::server::start(
		&db,
		Keys::single("test", [7; 32]),
		Options {
			mode: RunMode::Test,
			gateway: Gateway::new(solana(), Chain::default()),
			signer: ReceiptSigner::development(),
			hall: quick_hall(),
			origin: None,
		},
	)
	.unwrap();
	let router = reapers.router;
	let wallet = Keypair::from_seed(&[33; 32]);

	// Guest 1 links wallet.
	let (_, _, g1_body) = post_http(&router, reapers_api::auth::GUEST, None, json!(null)).await;
	let g1: SignedIn = serde_json::from_value(g1_body).unwrap();
	let (_, _, ch1) = post_http(
		&router,
		"/api/wallet_challenge",
		Some(&g1.access_token),
		json!({ "address": wallet.address().to_string() }),
	)
	.await;
	let sig1 = STANDARD.encode(
		wallet
			.sign(ch1["message"].as_str().unwrap().as_bytes())
			.as_bytes(),
	);
	let (status, ..) = post_http(
		&router,
		reapers_api::auth::WALLET,
		Some(&g1.access_token),
		json!({ "challenge_id": ch1["challenge_id"], "signature": sig1 }),
	)
	.await;
	assert_eq!(status, StatusCode::OK);

	// Guest 2 signs in with the same wallet -> switches to Guest 1 AND revokes Guest 2's guest session.
	let (_, _, g2_body) = post_http(&router, reapers_api::auth::GUEST, None, json!(null)).await;
	let g2: SignedIn = serde_json::from_value(g2_body).unwrap();
	let (_, _, ch2) = post_http(
		&router,
		"/api/wallet_challenge",
		Some(&g2.access_token),
		json!({ "address": wallet.address().to_string() }),
	)
	.await;
	let sig2 = STANDARD.encode(
		wallet
			.sign(ch2["message"].as_str().unwrap().as_bytes())
			.as_bytes(),
	);
	let (status, _, switched_body) = post_http(
		&router,
		reapers_api::auth::WALLET,
		Some(&g2.access_token),
		json!({ "challenge_id": ch2["challenge_id"], "signature": sig2 }),
	)
	.await;
	assert_eq!(status, StatusCode::OK);
	let switched: SignedIn = serde_json::from_value(switched_body).unwrap();
	assert_eq!(switched.account, g1.account);

	let (status, ..) = post_http(&router, "/api/load", Some(&g2.access_token), json!({})).await;
	assert_eq!(
		status,
		StatusCode::UNAUTHORIZED,
		"abandoned guest session must be revoked on wallet account switch"
	);

	// Disabling the wallet's owner account blocks subsequent wallet sign-ins with 403.
	db.system("test disable")
		.execute(Statement {
			sql: "UPDATE \"auth\".\"accounts\" SET \"disabled_at\" = $1 WHERE \"id\" = $2"
				.to_owned(),
			args: vec![
				DbValue::Timestamp(leptoskit_db::now()),
				DbValue::Uuid(g1.account.parse().unwrap()),
			],
		})
		.await
		.unwrap();

	let (_, _, g3_body) = post_http(&router, reapers_api::auth::GUEST, None, json!(null)).await;
	let g3: SignedIn = serde_json::from_value(g3_body).unwrap();
	let (_, _, ch3) = post_http(
		&router,
		"/api/wallet_challenge",
		Some(&g3.access_token),
		json!({ "address": wallet.address().to_string() }),
	)
	.await;
	let sig3 = STANDARD.encode(
		wallet
			.sign(ch3["message"].as_str().unwrap().as_bytes())
			.as_bytes(),
	);
	let (status, ..) = post_http(
		&router,
		reapers_api::auth::WALLET,
		Some(&g3.access_token),
		json!({ "challenge_id": ch3["challenge_id"], "signature": sig3 }),
	)
	.await;
	assert_eq!(status, StatusCode::FORBIDDEN);
}
