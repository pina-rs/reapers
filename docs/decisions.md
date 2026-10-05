# The Book of Becoming

The playable decision record is **The Book of Becoming**, opened with the book button in Reaper studio. The standalone browser preview also exposes `/#/decisions`, so the record can be bookmarked and refreshed independently of the model viewer. Build and device instructions are in [preview.md](preview.md).

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
| Shared progression | The preview, offline game and server agree on prices, thresholds and effects. | `Progression`, `Temper`, `Outfit`, `HuntOutcome` in `reapers_shared`. |
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

`reapers_flutter/lib/studio/decision_ledger.dart` owns the page, while `lib/widgets/soul_ledger.dart` owns the reusable exact gift presentation. Player-visible copy lives in `lib/l10n/app_en.arb`. `main_preview.dart` registers the bookmarkable route. The studio's book button also opens it inside the normal phone game.

Small-screen and enlarged-text checks exercise the reader and reward scenario. The isolated phone journey opens the actual preview, inspects an awakening, changes reward odds and distinguishes proposed NFT work. Run it with `fvm dart tool/test_studio.dart android <device-id> decisions`; use the attached physical phone by default.
