//! Rifts hosted by the keeper, one reaper and several at once, each paid
//! from their own books, against a real PostgreSQL.

mod support;

use futures::StreamExt;
use leptoskit_auth::Account;
use leptoskit_db::Id;
use leptoskit_rpc as rpc;
use reapers_api::keeper::Keeper as _;
use reapers_api::player::Acquire;
use reapers_api::player::Load;
use reapers_api::player::Release;
use reapers_api::player::SettleHunt;
use reapers_api::player::TakeOath;
use reapers_rules::GeoPoint;
use reapers_rules::GeoStanding;
use reapers_rules::HuntBot;
use reapers_rules::HuntLoadout;
use reapers_rules::PlayerState;
use reapers_rules::Refusal;
use reapers_rules::RiftFrame;
use reapers_rules::RiftPhase;
use reapers_rules::RiftReport;
use reapers_rules::RiftRules;
use reapers_rules::RiftSeat;
use reapers_server::keeper::Shared;
use reapers_server::keeper::now;
use support::*;

const SITE: &str = "londonBridge";

/// Enters the rift at `site` as `who`, through the operation's keeper.
async fn enter(
	world: &World,
	who: Option<Id<Account>>,
	site: &str,
	demo: bool,
) -> rpc::Result<futures::stream::BoxStream<'static, rpc::Result<RiftFrame>>> {
	let cx = rpc::QueryCx::new(world.db.clone(), world.identity(who));

	Shared(std::sync::Arc::clone(&world.keeper))
		.enter_rift(&cx, site.to_owned(), demo)
		.await
}

/// Sends `report` from `who`'s seat, through the operation's keeper.
async fn report(world: &World, who: Id<Account>, report: RiftReport) -> rpc::Result<()> {
	let cx = rpc::ActionCx::new(world.db.clone(), world.identity(Some(who)));

	Shared(std::sync::Arc::clone(&world.keeper))
		.report_rift(&cx, report)
		.await
}

/// One reaper at a rift, played by the shared bot: it stands ready, keeps
/// its own fight in step with the frames the keeper sends, and reports how
/// that fight stands.
struct Reaper {
	account: Id<Account>,
	loadout: HuntLoadout,
	/// Where this reaper's phone says it is, when it can say.
	standing: Option<GeoStanding>,
	frames: Vec<RiftFrame>,
}

impl Reaper {
	fn new(account: Id<Account>, state: &PlayerState) -> Self {
		Self {
			account,
			loadout: state.loadout(),
			standing: None,
			frames: Vec::new(),
		}
	}

	fn last(&self) -> &RiftFrame {
		self.frames.last().expect("a frame")
	}

	/// Plays the rift at `SITE` until the keeper ends it.
	async fn play(&mut self, world: &World, demo: bool) {
		let mut stream = enter(world, Some(self.account), SITE, demo)
			.await
			.expect("the rift opens");

		report(
			world,
			self.account,
			RiftReport {
				ready: true,
				standing: self.standing,
				..RiftReport::default()
			},
		)
		.await
		.expect("ready");

		let mut seat: Option<(RiftSeat, HuntBot)> = None;

		while let Some(frame) = stream.next().await {
			let frame = frame.expect("a frame");

			self.frames.push(frame.clone());

			if frame.phase == RiftPhase::Gathering || frame.phase.is_over() {
				continue;
			}

			let (seat, bot) = seat.get_or_insert_with(|| {
				let seat =
					RiftSeat::new(frame.seed, frame.site, frame.ward_max, &self.loadout, None)
						.expect("a valid loadout");
				let bot = HuntBot::careful(&seat.sim);

				(seat, bot)
			});
			let counted = frame.you().map_or(0.0, |you| you.dealt);

			while seat.sim.tick() < frame.tick && !seat.sim.is_over() {
				seat.sync(frame.ward, counted);

				let input = bot.decide(&seat.sim);

				seat.step(&input);
			}

			// A report that arrives after the rift ended has nowhere to go.
			let _ = report(world, self.account, seat.report()).await;
		}
	}
}

/// Takes `who` through the oath and one absorbed Tin Saint, which is what
/// opens the London Bridge rift to them.
async fn attune(world: &World, who: Id<Account>) -> PlayerState {
	const SAINT: &str = "degenApes:1";

	world.call(who, TakeOath {}).await.unwrap();
	world
		.call(who, Acquire { nft: SAINT.into() })
		.await
		.unwrap();
	world
		.call(
			who,
			Release {
				nft: SAINT.into(),
				escrow: None,
			},
		)
		.await
		.unwrap();
	world.surface_everything(who).await;

	let released = world.call(who, Load {}).await.unwrap();
	let sighting = released.sightings()[0];
	let settled = world
		.call(
			who,
			SettleHunt {
				sighting: sighting.id,
				log: packed(&play(&released, &sighting)),
			},
		)
		.await
		.unwrap();

	assert!(settled.settlement.outcome.absorbed());

	settled.settlement.state
}

#[tokio::test]
async fn when_nobody_is_signed_in_then_the_rift_is_shut() {
	let world = World::new().await;

	assert_unauthorized(enter(&world, None, SITE, false).await.map(drop));
	assert_unauthorized(
		Shared(std::sync::Arc::clone(&world.keeper))
			.report_rift(
				&rpc::ActionCx::new(world.db.clone(), world.identity(None)),
				RiftReport::default(),
			)
			.await,
	);
}

#[tokio::test]
async fn when_the_reaper_carries_none_of_its_dead_then_they_are_refused() {
	let world = World::new().await;
	let player = world.player().await;

	world.call(player, TakeOath {}).await.unwrap();

	assert_refused(
		enter(&world, Some(player), SITE, false).await.map(drop),
		Refusal::NotAttuned,
	);
}

#[tokio::test]
async fn when_a_visitor_has_no_oath_then_the_demo_is_refused() {
	let world = World::new().await;
	let player = world.player().await;

	assert_refused(
		enter(&world, Some(player), SITE, true).await.map(drop),
		Refusal::OathRequired,
	);
}

#[tokio::test]
async fn when_the_site_does_not_exist_then_they_are_refused() {
	let world = World::new().await;
	let player = world.player().await;

	attune(&world, player).await;

	assert_refused(
		enter(&world, Some(player), "nowhere", false)
			.await
			.map(drop),
		Refusal::NotAttuned,
	);
}

#[tokio::test]
async fn a_stale_standing_ends_the_stream_with_a_location_refusal() {
	let world = World::new().await;
	let player = world.player().await;

	attune(&world, player).await;

	let mut stream = enter(&world, Some(player), SITE, false).await.unwrap();
	let stale = RiftReport {
		ready: true,
		standing: Some(GeoStanding {
			at: GeoPoint::new(51.5, -0.08),
			facing: 1.0,
			accuracy: 4.0,
			observed_at: Some(now().plus_seconds(-60)),
		}),
		..RiftReport::default()
	};

	assert_refused(
		report(&world, player, stale).await,
		Refusal::LocationUnusable,
	);

	let mut ended = None;

	while let Some(frame) = stream.next().await {
		if let Err(error) = frame {
			ended = Some(error);
		}
	}

	assert_refused(ended.map_or(Ok(()), Err), Refusal::LocationUnusable);
	assert_eq!(world.call(player, Load {}).await.unwrap().rifts_sealed(), 0);
}

#[tokio::test]
async fn when_one_reaper_comes_then_shades_fill_the_circle_and_sealing_pays() {
	let world = World::new().await;
	let player = world.player().await;
	let before = attune(&world, player).await;
	let mut reaper = Reaper::new(player, &before);

	reaper.play(&world, false).await;

	let first = &reaper.frames[0];
	let last = reaper.last();

	assert_eq!(first.phase, RiftPhase::Gathering);
	assert_eq!(first.members.len(), 1);
	assert_eq!(first.members[0].name, before.genome().name());
	assert_eq!(last.phase, RiftPhase::Sealed, "{last:?}");
	assert_eq!(last.members.len(), RiftRules::CIRCLE);
	assert_eq!(last.members.iter().filter(|member| member.shade).count(), 3);
	assert!(last.you().unwrap().cut.is_some());
	assert!(last.xp > 0);

	let after = world.call(player, Load {}).await.unwrap();

	assert_eq!(after.xp(), before.xp() + last.xp);
	assert_eq!(after.anima(), before.anima() + last.anima);
	assert_eq!(after.rifts_sealed(), 1);
}

#[tokio::test]
async fn when_a_reaper_never_cuts_then_the_rift_is_not_sealed_and_pays_nothing() {
	let world = World::new().await;
	let player = world.player().await;
	let before = attune(&world, player).await;
	let stream = enter(&world, Some(player), SITE, false).await.unwrap();

	report(&world, player, RiftReport::readiness(true))
		.await
		.unwrap();

	let frames: Vec<RiftFrame> = stream.map(|frame| frame.expect("a frame")).collect().await;
	let last = frames.last().unwrap();

	assert_eq!(last.phase, RiftPhase::Failed);
	assert_eq!(last.xp, 0);

	let after = world.call(player, Load {}).await.unwrap();

	assert_eq!(after.xp(), before.xp());
	assert_eq!(after.rifts_sealed(), 0);
}

#[tokio::test]
async fn a_circle_everyone_left_is_not_where_the_next_reaper_waits() {
	let world = World::new().await;
	let player = world.player().await;

	attune(&world, player).await;

	// Enters, sees the circle gather, and goes.
	let mut gone = enter(&world, Some(player), SITE, false).await.unwrap();

	gone.next().await.expect("a frame").expect("the circle");
	drop(gone);
	tokio::time::sleep(std::time::Duration::from_millis(50)).await;

	let stream = enter(&world, Some(player), SITE, false).await.unwrap();

	report(&world, player, RiftReport::readiness(true))
		.await
		.unwrap();

	let frames = tokio::time::timeout(
		std::time::Duration::from_secs(20),
		stream
			.map(|frame| frame.expect("a frame"))
			.collect::<Vec<_>>(),
	)
	.await
	.expect("the new circle moves on and ends");

	assert!(frames.last().unwrap().phase.is_over());
}

#[tokio::test]
async fn a_reaper_sits_in_one_rift_at_a_time() {
	let world = World::new().await;
	let player = world.player().await;

	attune(&world, player).await;

	let _held = enter(&world, Some(player), SITE, false).await.unwrap();

	assert!(matches!(
		enter(&world, Some(player), SITE, true).await.map(drop),
		Err(rpc::Error::Conflict(_))
	));
}

#[tokio::test]
async fn when_two_reapers_come_then_they_share_one_circle_and_one_ward() {
	let world = World::new().await;
	let (one, two) = (world.player().await, world.player().await);
	let mut first = Reaper::new(one, &attune(&world, one).await);
	let mut second = Reaper::new(two, &attune(&world, two).await);

	tokio::join!(first.play(&world, false), second.play(&world, false));

	assert_eq!(first.last().phase, RiftPhase::Sealed);
	assert_eq!(second.last().phase, RiftPhase::Sealed);
	assert_eq!(first.last().seed, second.last().seed);
	assert_eq!(first.last().ward_max, second.last().ward_max);
	assert_eq!(
		first
			.last()
			.members
			.iter()
			.filter(|member| !member.shade)
			.count(),
		2
	);
	assert_ne!(first.last().seat, second.last().seat);
	assert!(first.last().you().unwrap().dealt > 0.0);
	assert!(second.last().you().unwrap().dealt > 0.0);
	assert_eq!(world.call(one, Load {}).await.unwrap().rifts_sealed(), 1);
	assert_eq!(world.call(two, Load {}).await.unwrap().rifts_sealed(), 1);
}

#[tokio::test]
async fn when_two_reapers_join_the_demo_then_they_share_one_enemy_and_each_is_paid_once() {
	let world = World::new().await;
	let (one, two) = (world.player().await, world.player().await);

	world.call(one, TakeOath {}).await.unwrap();
	world.call(two, TakeOath {}).await.unwrap();

	let first_before = world.call(one, Load {}).await.unwrap();
	let second_before = world.call(two, Load {}).await.unwrap();
	let mut first = Reaper::new(one, &first_before);
	let mut second = Reaper::new(two, &second_before);

	tokio::join!(first.play(&world, true), second.play(&world, true));

	let (a, b) = (first.last(), second.last());

	assert_eq!(a.seed, b.seed);
	assert_eq!(a.ward_max, b.ward_max);
	assert_eq!(a.ward, b.ward);
	assert_eq!(
		a.members
			.iter()
			.map(|member| &member.name)
			.collect::<Vec<_>>(),
		b.members
			.iter()
			.map(|member| &member.name)
			.collect::<Vec<_>>()
	);
	assert_eq!(a.members.iter().filter(|member| !member.shade).count(), 2);
	assert_eq!(
		a.members.iter().filter(|member| member.shade).count(),
		2,
		"the empty seats are the ordinary shades"
	);
	assert_eq!(a.you().unwrap().name, first_before.genome().name());
	assert_eq!(b.you().unwrap().name, second_before.genome().name());
	assert_eq!(a.phase, RiftPhase::Sealed);
	assert_eq!(b.phase, RiftPhase::Sealed);
	assert!(a.xp > 0);
	assert!(b.xp > 0);

	let first_after = world.call(one, Load {}).await.unwrap();
	let second_after = world.call(two, Load {}).await.unwrap();

	assert_eq!(first_after.xp(), first_before.xp() + a.xp);
	assert_eq!(second_after.xp(), second_before.xp() + b.xp);
	assert_eq!(first_after.anima(), first_before.anima() + a.anima);
	assert_eq!(second_after.anima(), second_before.anima() + b.anima);
	assert_eq!(first_after.rifts_sealed(), 1);
	assert_eq!(second_after.rifts_sealed(), 1);

	// Reading the books again is not a second payment.
	assert_eq!(
		world.call(one, Load {}).await.unwrap().xp(),
		first_after.xp()
	);
	assert_eq!(
		world.call(two, Load {}).await.unwrap().xp(),
		second_after.xp()
	);
}

#[tokio::test]
async fn when_a_demo_and_an_ordinary_circle_share_a_site_they_do_not_mix() {
	let world = World::new().await;
	let (one, two) = (world.player().await, world.player().await);
	let mut first = Reaper::new(one, &attune(&world, one).await);

	world.call(two, TakeOath {}).await.unwrap();

	let mut second = Reaper::new(two, &world.call(two, Load {}).await.unwrap());

	tokio::join!(first.play(&world, false), second.play(&world, true));

	assert_eq!(
		first
			.last()
			.members
			.iter()
			.filter(|member| !member.shade)
			.count(),
		1,
		"the ordinary circle has only its own reaper"
	);
	assert_eq!(
		second
			.last()
			.members
			.iter()
			.filter(|member| !member.shade)
			.count(),
		1,
		"the demo circle has only its own reaper"
	);
	assert_eq!(first.last().phase, RiftPhase::Sealed);
	assert_eq!(second.last().phase, RiftPhase::Sealed);
}

#[tokio::test]
async fn when_they_say_where_they_stand_then_the_rift_takes_one_place_for_both() {
	let world = World::new().await;
	let (one, two) = (world.player().await, world.player().await);
	let garden = GeoPoint::new(52.2, 0.12);
	let beside = garden.moved(2.0, 4.0);
	let mut first = Reaper::new(one, &attune(&world, one).await);
	let mut second = Reaper::new(two, &attune(&world, two).await);

	first.standing = Some(GeoStanding {
		at: garden,
		facing: 1.2,
		accuracy: 4.0,
		observed_at: Some(now()),
	});
	second.standing = Some(GeoStanding {
		at: beside,
		facing: -2.0,
		accuracy: 6.0,
		observed_at: Some(now()),
	});

	tokio::join!(first.play(&world, false), second.play(&world, false));

	let anchor = first.last().anchor.expect("an anchor");

	// Whichever of them the rift heard first, it stands in one place, 25
	// metres in front of that one, and both are told the same.
	assert_eq!(second.last().anchor, Some(anchor));
	assert!(
		[garden.distance_to(&anchor), beside.distance_to(&anchor)]
			.iter()
			.any(|metres| (metres - RiftRules::PLANT_METRES).abs() < 0.01)
	);
	assert_eq!(first.last().phase, RiftPhase::Sealed);
}
