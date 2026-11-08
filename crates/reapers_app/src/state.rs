//! The game's state, as every screen reads it.
//!
//! [`Game`] is one `Copy` handle of signals and stored values, provided as
//! context where the app starts. Screens read the player from
//! [`Game::books`], and every change goes through [`Game::change`], which
//! asks the keeper and then shows what it says: a refusal leaves the player
//! as they were and puts the rules' reason in front of them.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::future::Future;
use std::rc::Rc;
use std::time::Duration;

use reactive_graph::owner::LocalStorage;
use reactive_graph::owner::StoredValue;
use reactive_graph::owner::provide_context;
use reactive_graph::owner::use_context;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::Get;
use reactive_graph::traits::GetUntracked;
use reactive_graph::traits::GetValue;
use reactive_graph::traits::Set;
use reactive_graph::traits::SetValue;
use reactive_graph::traits::Update;
use reactive_graph::traits::UpdateValue;
use reapers_rules::Difficulty;
use reapers_rules::Nft;
use reapers_rules::PlayerState;
use reapers_rules::Refusal;
use reapers_rules::Sighting;
use reapers_rules::Timestamp;
use serde_json::Value;

use crate::backend::GameBackend;
use crate::backend::KeeperError;
use crate::backend::KeeperKind;
use crate::backend::KeeperResult;
use crate::backend::LocalBackend;
use crate::backend::RemoteBackend;
use crate::backend::UnwiredApi;
use crate::config::BuildFlags;
use crate::config::ConfigError;
use crate::config::KeeperChoice;
use crate::platform;
use crate::platform::DeviceHttp;
use crate::platform::DevicePrefs;
use crate::platform::Prefs;
use crate::simnet::SimnetBackend;
use crate::simnet::SimnetChain;
use crate::simnet::SimnetTransaction;
use crate::simnet::SurfpoolRpc;
use crate::sound::Score;
use crate::sound::Soundtrack;

/// The player's books, as far as the game has read them.
#[derive(Clone, Debug, PartialEq)]
pub enum Books {
	/// Still being read.
	Opening,
	/// The player as the keeper last said.
	Open(PlayerState),
	/// The books could not be read at all.
	Unreadable(BooksFailure),
}

/// Why the books could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BooksFailure {
	/// The keeper could not be reached.
	Unreachable,
	/// The save is damaged, or the keeper answered nonsense.
	Damaged,
	/// The build's flags cannot make a game; the reason is for a developer.
	Misbuilt(ConfigError),
}

/// What a confirmation slip says about a simnet transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slip {
	pub transaction: SimnetTransaction,
	pub finalized: bool,
}

/// Something laid over the current screen: a sheet or a notice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Overlay {
	/// The rules refused, and say why.
	Refused(Refusal),
	/// The keeper could not be reached.
	Unreachable,
	/// One NFT in full, from the vault.
	Nft(Nft),
	/// One simnet transaction in full, by signature.
	Transaction(String),
	/// Asks before the simnet starts over.
	ConfirmReset,
}

/// Everything the screens share.
#[derive(Clone, Copy)]
pub struct Game {
	pub flags: BuildFlags,
	pub books: RwSignal<Books>,
	/// Which keeper holds the books, once one is chosen.
	pub keeper_kind: RwSignal<Option<KeeperKind>>,
	/// Every simnet transaction, oldest first. Empty without a simnet.
	pub ledger: RwSignal<Vec<SimnetTransaction>>,
	/// The slip for the transaction that just landed.
	pub slip: RwSignal<Option<Slip>>,
	pub overlay: RwSignal<Option<Overlay>>,
	/// Whether a request is on its way to the keeper.
	pub busy: RwSignal<bool>,
	/// The time, a second at a time, for countdowns.
	pub now: RwSignal<Timestamp>,
	/// The husk that has just surfaced, for a few seconds after it does.
	pub sighted: RwSignal<Option<Sighting>>,
	/// Whether the music plays.
	pub music: RwSignal<bool>,
	/// How forgiving hunts are while the game is being tested.
	pub difficulty: RwSignal<Difficulty>,
	/// Whether the arrival is over and the game may be touched.
	pub arrived: RwSignal<bool>,
	/// Why the simnet's local validator could not be set up, when a build
	/// names one.
	pub surfpool_problem: RwSignal<Option<String>>,
	keeper: StoredValue<Option<Rc<dyn GameBackend>>, LocalStorage>,
	prefs: StoredValue<Rc<dyn Prefs>, LocalStorage>,
	simnet: StoredValue<Option<Rc<SimnetChain>>, LocalStorage>,
	surfpool: StoredValue<Option<Rc<SurfpoolRpc>>, LocalStorage>,
	soundtrack: StoredValue<Rc<Soundtrack>, LocalStorage>,
	waiting: StoredValue<RefCell<BTreeSet<i64>>, LocalStorage>,
}

impl Game {
	const DIFFICULTY_KEY: &'static str = "reapers.testing.difficulty";
	/// How long a simnet transaction shows as confirmed before it is
	/// finalized, and how long its slip stays up.
	pub const FINALITY: Duration = Duration::from_millis(1400);
	const MUSIC_KEY: &'static str = "reapers.music";
	/// How long the "husk sighted" banner stays.
	pub const SIGHTED_SHOWN: Duration = Duration::from_secs(5);
	pub const SLIP_SHOWN: Duration = Duration::from_millis(4200);

	/// A game with nothing read yet, keeping what it must remember in
	/// `prefs`. Call [`Self::open`] to read the books.
	pub fn new(flags: BuildFlags, prefs: Rc<dyn Prefs>) -> Self {
		crate::ui::install_lasting_owner();
		Self {
			flags,
			books: RwSignal::new(Books::Opening),
			keeper_kind: RwSignal::new(None),
			ledger: RwSignal::new(Vec::new()),
			slip: RwSignal::new(None),
			overlay: RwSignal::new(None),
			busy: RwSignal::new(false),
			now: RwSignal::new(platform::now()),
			sighted: RwSignal::new(None),
			music: RwSignal::new(true),
			difficulty: RwSignal::new(if flags.testing_difficulty() {
				Difficulty::Easy
			} else {
				Difficulty::Normal
			}),
			arrived: RwSignal::new(false),
			surfpool_problem: RwSignal::new(None),
			keeper: StoredValue::new_local(None),
			prefs: StoredValue::new_local(prefs),
			simnet: StoredValue::new_local(None),
			surfpool: StoredValue::new_local(None),
			soundtrack: StoredValue::new_local(Rc::new(Soundtrack::silent())),
			waiting: StoredValue::new_local(RefCell::new(BTreeSet::new())),
		}
	}

	/// The game this build makes, on the device's own store, provided as
	/// context and opening its books in the background.
	pub fn start() -> Self {
		let game = Self::new(BuildFlags::current(), Rc::new(DevicePrefs));
		provide_context(game);
		platform::spawn(async move {
			game.open().await;
			game.tick().await;
		});
		game
	}

	/// The game an ancestor provided.
	///
	/// # Panics
	///
	/// When none did.
	#[track_caller]
	pub fn get() -> Self {
		use_context::<Self>().expect("Game::start provides the game before any screen renders")
	}

	/// Chooses the keeper this build asks for, opens its simnet when it has
	/// one, and reads the player.
	pub async fn open(self) {
		let prefs = self.prefs.get_value();
		self.restore_settings(&*prefs).await;

		let choice = match self.flags.keeper() {
			Ok(choice) => choice,
			Err(error) => {
				self.books
					.set(Books::Unreadable(BooksFailure::Misbuilt(error)));
				return;
			}
		};

		let keeper: Rc<dyn GameBackend> = match choice {
			KeeperChoice::Local => Rc::new(LocalBackend::new(prefs)),
			KeeperChoice::Simnet => {
				match self.open_simnet(prefs.clone()).await {
					Ok(chain) => Rc::new(SimnetBackend::new(LocalBackend::new(prefs), chain)),
					Err(_) => {
						self.books.set(Books::Unreadable(BooksFailure::Damaged));
						return;
					}
				}
			}
			// TODO(integration): reapers_api. The coordinator hands
			// `RemoteBackend::new` the reapers_api client for `url` here.
			KeeperChoice::Server(_url) => Rc::new(RemoteBackend::new(UnwiredApi)),
		};
		self.keeper_kind.set(Some(keeper.kind()));
		self.keeper.set_value(Some(keeper));
		self.reload().await;
	}

	async fn open_simnet(self, prefs: Rc<dyn Prefs>) -> KeeperResult<Rc<SimnetChain>> {
		let chain =
			Rc::new(SimnetChain::open(prefs, Rc::new(platform::now), SimnetChain::LATENCY).await?);
		self.ledger.set(chain.transactions());
		chain.subscribe(move |transaction| self.posted(transaction.clone()));
		self.simnet.set_value(Some(chain.clone()));

		if let Some(url) = self.flags.surfpool_url {
			match SurfpoolRpc::new(url, Rc::new(DeviceHttp)) {
				Ok(rpc) => self.surfpool.set_value(Some(Rc::new(rpc))),
				Err(problem) => self.surfpool_problem.set(Some(problem)),
			}
		}
		Ok(chain)
	}

	/// A transaction landed: add it to the ledger and slip it in front of
	/// the player wherever they are.
	fn posted(self, transaction: SimnetTransaction) {
		let signature = transaction.signature.clone();
		self.ledger
			.update(|ledger| ledger.push(transaction.clone()));
		self.slip.set(Some(Slip {
			transaction,
			finalized: false,
		}));
		platform::spawn(async move {
			let showing = |slip: Option<Slip>| {
				slip.is_some_and(|slip| slip.transaction.signature == signature)
			};
			// A host with no timer keeps the slip up rather than flash it.
			if !platform::sleep(Self::FINALITY).await {
				return;
			}
			if showing(self.slip.get_untracked()) {
				self.slip.update(|slip| {
					if let Some(slip) = slip {
						slip.finalized = true;
					}
				});
			}
			if platform::sleep(Self::SLIP_SHOWN - Self::FINALITY).await
				&& showing(self.slip.get_untracked())
			{
				self.slip.set(None);
			}
		});
	}

	/// Reads the player again from the keeper.
	pub async fn reload(self) {
		let Some(keeper) = self.keeper() else {
			return;
		};
		self.books.set(match keeper.load().await {
			Ok(player) => Books::Open(player),
			Err(KeeperError::Damaged(_) | KeeperError::Refused(_)) => {
				Books::Unreadable(BooksFailure::Damaged)
			}
			Err(KeeperError::Unavailable(_)) => Books::Unreadable(BooksFailure::Unreachable),
		});
	}

	/// Tries the keeper again after the books could not be read.
	pub fn retry(self) {
		self.books.set(Books::Opening);
		platform::spawn(async move {
			if self.keeper().is_some() {
				self.reload().await;
			} else {
				self.open().await;
			}
		});
	}

	/// The time, a second at a time, and the husk that surfaced in it.
	async fn tick(self) {
		loop {
			let now = platform::now();
			self.now.set(now);
			self.watch_surfacing(now);
			if !platform::sleep(Duration::from_secs(1)).await {
				return;
			}
		}
	}

	/// Announces a husk that has just surfaced. It only fires for a husk
	/// that was seen waiting first, so reopening the game does not announce
	/// husks the player already knows about.
	fn watch_surfacing(self, now: Timestamp) {
		let Books::Open(player) = self.books.get_untracked() else {
			return;
		};
		let mut announced = None;
		self.waiting.update_value(|waiting| {
			let waiting = waiting.get_mut();
			for sighting in player.sightings() {
				if !sighting.has_surfaced(now) {
					waiting.insert(sighting.id);
				} else if waiting.remove(&sighting.id) {
					announced = Some(*sighting);
				}
			}
		});
		if let Some(sighting) = announced {
			platform::thud();
			self.sighted.set(Some(sighting));
			platform::spawn(async move {
				platform::sleep(Self::SIGHTED_SHOWN).await;
				if self
					.sighted
					.get_untracked()
					.is_some_and(|shown| shown.id == sighting.id)
				{
					self.sighted.set(None);
				}
			});
		}
	}

	/// The keeper, once one is chosen.
	pub fn keeper(&self) -> Option<Rc<dyn GameBackend>> {
		self.keeper.get_value()
	}

	/// The simnet, in a build that has one.
	pub fn simnet(&self) -> Option<Rc<SimnetChain>> {
		self.simnet.get_value()
	}

	/// The simnet's local validator, in a build that names one.
	pub fn surfpool(&self) -> Option<Rc<SurfpoolRpc>> {
		self.surfpool.get_value()
	}

	/// The device's store, for a panel that keeps a record of its own.
	pub fn prefs(&self) -> Rc<dyn Prefs> {
		self.prefs.get_value()
	}

	/// The player, when the books are open. Reading it subscribes the
	/// caller.
	pub fn player(&self) -> Option<PlayerState> {
		match self.books.get() {
			Books::Open(player) => Some(player),
			_ => None,
		}
	}

	/// Runs `request` against the keeper. A refusal or an unreachable
	/// keeper is explained to the player, never silently dropped. `then` is
	/// told whether it went through.
	pub fn attempt<T: 'static, F, Fut>(self, request: F, then: impl FnOnce(Option<T>) + 'static)
	where
		F: FnOnce(Rc<dyn GameBackend>) -> Fut + 'static,
		Fut: Future<Output = KeeperResult<T>> + 'static,
	{
		let Some(keeper) = self.keeper() else {
			self.overlay.set(Some(Overlay::Unreachable));
			then(None);
			return;
		};
		self.busy.set(true);
		platform::spawn(async move {
			let result = request(keeper).await;
			self.busy.set(false);
			match result {
				Ok(value) => then(Some(value)),
				Err(error) => {
					self.explain(&error);
					then(None);
				}
			}
		});
	}

	/// Asks the keeper to change the player, and shows the player it
	/// answers with.
	pub fn change<F, Fut>(self, request: F, then: impl FnOnce(bool) + 'static)
	where
		F: FnOnce(Rc<dyn GameBackend>) -> Fut + 'static,
		Fut: Future<Output = KeeperResult<PlayerState>> + 'static,
	{
		self.attempt(request, move |player| {
			let done = player.is_some();
			if let Some(player) = player {
				self.books.set(Books::Open(player));
			}
			then(done);
		});
	}

	/// Puts `error` in front of the player.
	pub fn explain(self, error: &KeeperError) {
		self.overlay.set(Some(match error.refusal() {
			Some(refusal) => Overlay::Refused(refusal),
			None => Overlay::Unreachable,
		}));
	}

	/// Starts the simnet over: a new wallet and a new player, for the next
	/// person to pick the device up.
	pub fn start_over(self, then: impl FnOnce(bool) + 'static) {
		self.attempt(
			|keeper| async move { keeper.forget().await },
			move |done| {
				if done.is_some() {
					if let Some(chain) = self.simnet() {
						self.ledger.set(chain.transactions());
					}
					self.slip.set(None);
					platform::spawn(self.reload());
				}
				then(done.is_some());
			},
		);
	}

	/// Plays `score`, or holds it while the game is out of sight.
	pub fn play(self, score: Score) {
		self.soundtrack
			.get_value()
			.play(score, self.music.get_untracked());
	}

	/// Turns the music on or off, and remembers which.
	pub fn toggle_music(self) {
		let on = !self.music.get_untracked();
		self.music.set(on);
		self.soundtrack.get_value().enable(on);
		let prefs = self.prefs();
		platform::spawn(async move {
			let _ = prefs.set(Self::MUSIC_KEY, Value::Bool(on)).await;
		});
	}

	/// Moves on to the next testing difficulty, from the hardest back to the
	/// easiest. A release build has no switch.
	pub fn cycle_difficulty(self) {
		if !self.flags.testing_difficulty() {
			return;
		}
		let next =
			Difficulty::ALL[(self.difficulty.get_untracked().index() + 1) % Difficulty::ALL.len()];
		self.difficulty.set(next);
		let prefs = self.prefs();
		platform::spawn(async move {
			let _ = prefs
				.set(Self::DIFFICULTY_KEY, Value::from(next.name()))
				.await;
		});
	}

	async fn restore_settings(self, prefs: &dyn Prefs) {
		if let Ok(Some(Value::Bool(on))) = prefs.get(Self::MUSIC_KEY).await {
			self.music.set(on);
		}
		if self.flags.testing_difficulty()
			&& let Ok(Some(Value::String(name))) = prefs.get(Self::DIFFICULTY_KEY).await
			&& let Some(difficulty) = Difficulty::from_name(&name)
		{
			self.difficulty.set(difficulty);
		}
	}
}

#[cfg(test)]
mod tests {
	use futures::executor::block_on;
	use reactive_graph::owner::Owner;
	use reapers_rules::market_listings;

	use super::*;
	use crate::platform::MemoryPrefs;

	fn game(flags: BuildFlags) -> Game {
		Game::new(flags, Rc::new(MemoryPrefs::new()))
	}

	#[test]
	fn the_offline_build_opens_a_new_player() {
		Owner::new().with(|| {
			let game = game(BuildFlags::default());
			block_on(game.open());
			assert_eq!(game.keeper_kind.get_untracked(), Some(KeeperKind::Local));
			assert!(
				matches!(game.books.get_untracked(), Books::Open(player) if !player.oath_taken())
			);
		});
	}

	#[test]
	fn a_misbuilt_build_says_why_instead_of_half_working() {
		Owner::new().with(|| {
			let game = game(BuildFlags {
				simnet: true,
				server_url: Some("http://localhost:8080/"),
				..BuildFlags::default()
			});
			block_on(game.open());
			assert_eq!(
				game.books.get_untracked(),
				Books::Unreadable(BooksFailure::Misbuilt(ConfigError::SimnetWithServer))
			);
		});
	}

	#[test]
	fn an_unwired_server_is_unreachable_and_can_be_retried() {
		Owner::new().with(|| {
			let game = game(BuildFlags {
				server_url: Some("http://localhost:8080/"),
				..BuildFlags::default()
			});
			block_on(game.open());
			assert_eq!(
				game.books.get_untracked(),
				Books::Unreadable(BooksFailure::Unreachable)
			);
		});
	}

	#[test]
	fn a_refusal_is_put_in_front_of_the_player_and_changes_nothing() {
		Owner::new().with(|| {
			let game = game(BuildFlags::default());
			block_on(game.open());
			let before = game.books.get_untracked();
			let nft = market_listings()[0];
			game.change(
				move |keeper| async move { keeper.acquire(nft).await },
				|_| {},
			);
			platform::run_until_stalled();
			assert_eq!(
				game.overlay.get_untracked(),
				Some(Overlay::Refused(Refusal::OathRequired))
			);
			assert_eq!(game.books.get_untracked(), before);
		});
	}

	#[test]
	fn the_simnet_build_slips_each_transaction_in_front_of_the_player() {
		Owner::new().with(|| {
			let game = game(BuildFlags {
				simnet: true,
				..BuildFlags::default()
			});
			block_on(game.open());
			assert_eq!(game.keeper_kind.get_untracked(), Some(KeeperKind::Simnet));
			game.change(|keeper| async move { keeper.take_oath().await }, |_| {});
			platform::run_until_stalled();
			assert_eq!(game.ledger.get_untracked().len(), 1);
			assert!(game.slip.get_untracked().is_some());
		});
	}

	#[test]
	fn music_and_difficulty_are_remembered() {
		Owner::new().with(|| {
			let prefs = Rc::new(MemoryPrefs::new());
			let game = Game::new(BuildFlags::default(), prefs.clone());
			game.toggle_music();
			game.cycle_difficulty();
			platform::run_until_stalled();

			let reopened = Game::new(BuildFlags::default(), prefs);
			block_on(reopened.open());
			assert!(!reopened.music.get_untracked());
			assert_eq!(reopened.difficulty.get_untracked(), Difficulty::Normal);
		});
	}
}
