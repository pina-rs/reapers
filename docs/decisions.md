# The Book of Becoming, and the decisions made

The playable decision record is **The Book of Becoming**, opened with the book button in Reaper studio. Build and device instructions are in [preview.md](preview.md). The [decision log](#decision-log) at the end of this page records the decisions about how the game is built.

This page explains the game implemented today and labels the proposed NFT future separately. Its controls are unsaved illustrations: no player account, wallet, camera or server is opened. The page calls the same shared arithmetic as the game; it does not maintain a second progression calculator.

## Four chapters

| Chapter | What the reader can inspect |
| --- | --- |
| Evolution | Six offices, all eighteen birth gifts, four temperaments, current effects, original lore and three future awakenings. Change level and remembered hunts independently. See equipment gates, prices, collection bonuses and all thirty XP thresholds. |
| Chance | Draw an illustrative new birth; inspect cosmetic omen odds. Choose husk rarity, cut quality and ambush, inspect the exact four feeding outcomes, then draw an unsaved example. The displayed average includes each rounded XP reward. |
| NFT future | Current wallet/ownership foundations, proposed owned-husk release and absorption, verified cooperative hunts, later history-driven expression, and the engineering and economic limits of each stage. |
| Decisions | The physical hunt, permanent identity, earned memories, skill and luck, research influences, balance evidence, source documents and outstanding operator decisions. |

## Random experiences, consistent records

The request that play should contain randomness is already reflected in the rules. A fresh birth draws its office, name, appearance and continuous traits. Omen appearance has weighted odds. Encounter seeds and surfacing delays vary. A successful absorption draws feeding from new secure server entropy. The browser's example buttons use fresh local secure entropy, but never submit it as an earned result.

The server records the draw rather than asking the phone to decide it. Replaying that recorded draw must produce the same outcome: otherwise verification could disagree with the hunt a player actually played. `HuntSim`, `PlayerState` and `RiftRoom` therefore continue to take seeds and times as inputs. They do not read a clock or draw fresh entropy internally. **A reproducible record does not mean a predictable next reward.** A released husk retains its encounter seed when retried; abandoning a hunt cannot redraw its behaviour. Feeding is drawn after a successful absorption has been accepted.

| Random element | Current rule | What stays fixed |
| --- | --- | --- |
| Birth | An office is drawn from six. The other genome fields are drawn once. | An owned reaper's birth identity; R1 maps its permanent fields to one gift and a temperament. Gifts are not separate cosmetic rarity prizes. |
| Cosmetic omen | None 80%, Ashen 12%, Gilded 6%, Eclipsed 2%. | The omen never increases power. |
| Surfacing | A delay between the shared 6–14 second bounds. | A released husk remains the same encounter when retried. |
| Feeding | Meagre 25% at ×0.85 XP; fair 50% at ×1; rich 20% at ×1.25; feast 5% at ×2. | Cut quality and ambush are earned inputs. Feeding changes XP, not anima or memory marks. |
| Awakenings | No random mutation currently. | Each gift grows at known level-and-memory gates, so a player can work toward an effect. |

A clean, non-ambushed common absorption has feeding rewards of 43, 50, 63 or 100 XP and always 20 anima. Its mean is **53.35 XP**, weighted over those rounded outcomes. The equivalent rare absorption has 128, 150, 188 or 300 XP, mean **159.60 XP**, and always 60 anima. These examples are calculated by `HuntOutcome.xpFor` and `animaFor`; changing rarity, cut or ambush changes the page's actual values.

The server's Ed25519 receipt authenticates the server's stated result. A recorded nonce lets another implementation reproduce the feeding. **Neither property proves the operator sampled fairly.** An operator-controlled seed can be chosen or withheld. Publicly verifiable randomness, or a carefully designed commitment protocol with timeout and withholding rules, remains future engineering; the page does not claim either is implemented.

Optional permanent random evolution is an open product decision. The implementation retains random births and rewards with predictable earned awakenings. If permanent mutations are adopted, define a new version, published odds, eligible events, bounded effects, replay/receipt representation and trading semantics before changing an owned R1 reaper. Do not silently rewrite existing identity mappings.

## Why evolution works this way

| Decision | Reason | Code and evidence |
| --- | --- | --- |
| Six offices with three gifts each | A calling supplies a readable hunting role; a gift provides a distinct verb or circumstance within it. | `ReaperClass`, `ReaperGift`, `HuntSim`; [identity research](research/reaper-identity-and-growth.md). |
| Four temperaments with benefits and costs | A starting reaper has a play preference, rather than merely a higher number. | `Temperament`, `ReaperSoul`; the page and shared Soul ledger show both sides. |
| Permanent R1 identity | A recognizable asset can carry a consistent history when traded. Cosmetic rarity cannot grant a stronger gift. | `ReaperGenome`, `ReaperSoul.fromGenome`; [NFT plan](nfts.md). |
| Level and deed gates together | Grinding XP alone does not fully awaken a gift. The way a reaper hunts matters. | `SoulRules`: level 5 / 3 preferred-path marks; level 12 / 12; level 20 / 30. |
| Bounded successful-hunt memories | At most one mark per path per absorption; escapes and abandoned hunts grant none. Claims come from server-replayed inputs. | `SoulMemory`, hunt settlement and server integration tests; [gameplay](gameplay.md). |
| No invented past | Older saves retain XP and equipment, but historical hunts without recorded evidence do not gain retrospective marks. | Backward-compatible `SoulMemory.zero`; persistence and replay tests. |
| Shared progression | The preview, offline game and server agree on prices, thresholds and effects. | `Progression`, `Temper`, `Outfit`, `HuntOutcome` in `reapers_rules`. |
| Independent reaper histories | Multiple owned reapers can develop separately. Anima is an account resource; XP, equipment, codex and memory stay with a reaper. | The roster, canonical growth and ownership transfer tests; [Solana integration](solana.md). |
| Skill and luck have different jobs | The phone's aiming, stillness and cadence decide the earned result; feeding adds surprise after success. | `HuntSim`, `ReapGrade`, `Feeding`; [hunt research](research/dark-games-and-ar-hunting.md). |

The research knowledge base distinguishes developer descriptions from our design inferences. Its seven game case studies informed recognizable identity, expressive specialization, legible milestones, earned story and bounded luck. They did not establish balance for Reapers. The existing 1,152 starting-build simulation shows different mechanical outcomes, not a conclusion that gifts are equally desirable or that the economy is ready to launch. Real phone playtesting remains necessary.

## How the NFT world could grow

These are stages of a proposed product, not promised dates or shipped commerce.

1. **Foundations in the codebase:** Solana Kit typed RPC, wallet-signature verification, finalized Metaplex Core ownership checks, multiple reapers and canonical growth. Deployment still needs actual configuration and complete wallet/device journeys.
2. **An owned-husk loop:** start by bringing an NFT from an approved collection. Implement release escrow, replayed absorption, a versioned signed receipt, confirmed burn and recoverable failure states. Real husk purchases, release escrow and burns are not implemented today. Prototype-market state remains blocked in production.
3. **Verified cooperative hunts:** connect rifts to verified collection history and server-authoritative participation. Ship the permission, lifecycle, privacy and location behaviour that physical play requires, with real-device evidence. Current location work does not prove resistance to spoofing.
4. **Expression from history:** consider visual scars, titles, collection memories and play specialisms. Preserve readable tradeoffs and version rules before expanding mechanics. No new token, PvP system or permanent mutation lottery is part of the implemented rules.

Trading a grown reaper carries its earned progression and may buy a mechanical advantage. Saying that cosmetic rarity grants no extra power does not remove that consequence. A burn reduces collection supply; it does not guarantee demand, higher prices or player earnings. Anima remains a non-tradeable account counter. Sustainable interest must come from enjoyable hunts, meaningful identity and cooperative play, not an assumed financial return.

## What still needs Ifiok's decisions

- The deployed network, RPC provider, verified reaper collection, approved husk collections and initial acquisition path.
- Mainnet asset/program details, production receipt authority and deployment secrets, supplied through the project's secret workflow.
- The desired fairness model before real assets are burned; signed operator draws and publicly verifiable randomness are different trust promises.
- Whether the current random birth/reward model is sufficient or permanent random evolution should be designed as a new version.
- Acceptance of grown-reaper trading advantages, and playtest targets for pacing, accessibility and balance.
- Store accounts, supported wallet journeys, privacy/retention decisions and a public preview host/domain.

[Launch readiness](launch-readiness.md) retains the complete engineering and operator checklist. This page records decisions already made; it does not turn that checklist into completed launch work.

## Implementation boundary

The book's copy is in `crates/reapers_app/locales/en.arb` (the `ledger…` strings), and every number it shows is `reapers_rules`' own: feeding odds and rewards from `Feeding` and `HuntOutcome`, gates from `SoulRules`, thresholds from `Progression`. The page itself returns with the studio on `leptoskit_3d`, which opens it from the studio's book button ([preview.md](preview.md)). Its checks must cover small screens and enlarged text, an awakening that needs both level and remembered hunts, exact feeding rewards and their rounded expectation, and proposed NFT work told apart from what is built.

## Decision log

### 2026-11-08: One Rust workspace

**Decided:** Reapers is one Rust workspace. The game moves from Flutter to [leptoskit](https://github.com/leptoskit/leptoskit), the server from Serverpod to leptoskit's server crates on PostgreSQL, and the rules from the Dart package `reapers_shared` to `crates/reapers_rules`. The on-chain program is written with [pina](https://github.com/pina-rs/pina). The repository's tools move from Dart scripts under `tool/` to `cargo xtask`.

**Why:**

- **One rule, really once.** The rules were already shared by the game and the server; the program and its receipt now need them too, and the program can only be Rust. With the game, the server, the program and the tools in one language, a receipt, a hunt log or a refusal is one type everywhere, checked by one compiler.
- **One app for the browser and the phones.** leptoskit renders the same screens in the browser and, as a Wasm guest, in native iOS and Android hosts. The Flutter game had a separate web renderer for the studio and could not run the game in the browser.
- **Solana Kit and the wallet in the same language as the program.** The game builds the program's own instructions from `reapers_program_client`, signs with the wallet, and checks receipts with `reapers_receipt`, the crate the program uses.
- **Depth from the platform.** `leptoskit_xr` brings ARCore's Depth API and ARKit's scene depth: measured depth, with the phone's pose, for nothing in the download. The bundled Depth Anything V2 model (27 MB, built by a Python conversion) is retired with the Flutter game.

**How it was kept honest:**

- The Rust rules keep the Dart rules' numbers, order of operations and save format, and are held to the bit to fixtures the Dart rules wrote (`crates/reapers_rules/tests/fixtures`). The generator stays there, retired, with the `git` command that checks out the last commit beside which it ran (`a7c2456`).
- The keeper was ported test for test from the Serverpod keeper's, against a real PostgreSQL.
- Every screen was ported with its strings; the 3D and camera views became seams with 2D stand-ins, so the game settles real hunts end to end before `leptoskit_3d` and `leptoskit_xr` fill them.
- `cargo xtask hooks` and `cargo xtask merge` keep the same guard on `main` the Dart tools had: `main` moves only to a commit the checks have just passed.

**What it costs:**

- **The Seeker no longer reads rooms.** It has no ARCore, so it has no platform depth, and plays every hunt in an unsurveyed room until a fallback exists. The same goes for iPhones without LiDAR and the browser. What the camera-only reading learned is kept in [environment.md](environment.md).
- **The 3D and the camera hunt are not done.** Until the seams are filled, reapers and husks are drawn flat and hunts are fought by the rules' own player.
- **Players' books do not move.** The Rust keeper has its own database. Nobody plays on a deployed server yet; one that has players needs an import from the Serverpod keeper's `player_record` rows first.
- **Pinned git dependencies.** leptoskit and Solana Kit come from git at one revision each, pinned in the root `Cargo.toml`, with the leptoskit CLI installed at the same revision.

**Retired:** `reapers_shared`, `reapers_flutter`, `reapers_server` (Serverpod), `reapers_client`, the pub workspace and `.fvmrc`, and the Dart tools: `check`, `hooks`, `merge`, `build_art`, `build_music`, `build_branding`, `build_depth_model` and its `export.py`, and the device journeys `test_hunt`, `test_studio` and `test_arrival`. The Flutter package's design brief (`DESIGN.md`, `PRODUCT.md`), its review surfaces and the delivery reports under `docs/verification` described that implementation and went with it; they are in history at `a7c2456`. The typefaces and the mark moved to `assets/fonts` and `assets/branding`.

### 2026-11-07: History without the generated binaries

**Decided:** The repository's history was rewritten so that no revision carries the generated binaries: every version of the `.glb` models, the depth model and the music was removed from every commit, taking the repository from 126 MB to 6 MB. What the game needs is kept once, compact, under `assets/`: models with meshopt compression and quantized attributes (gltfpack `-cc -kn -km -ke`, 72 MB to 12 MB with every node, morph target and clip kept), the music as Vorbis at quality 2, and the portraits as WebP. `cargo xtask build-art` and `build-music` make them that way.

**What it costs:** every commit before the rewrite has a new name. A clone made before it shares no history with `main` after it and must be cloned again; a branch from it must be rebased by hand onto the new history, file by file. Commit names quoted in older notes or messages do not resolve. The depth model, kept as a release asset after the rewrite, is retired by the decision above.
