# Reaper identity, memory and growth

Researched 5 October 2026. This is the decision record for the soul system,
not a claim that its starting numbers are balanced by player evidence.
The earlier [visual and AR research](dark-games-and-ar-hunting.md) remains
the source for silhouettes, stillness, short sessions and the six offices.

## The question

How can a reaper begin with a recognisable identity, become stronger in a
personal way, and make its history worth keeping when another player buys it?
Adding random damage bonuses would create expensive winners and unwanted
losers. A huge skill tree would turn a physical, quiet game into a spreadsheet.
We need a small set of understandable actions with lasting consequences.

## Primary-source evidence

### Hades: myth expressed through a repeatable build

**Evidence.** Supergiant describes repeatedly combining the Olympians' powers
while discovering builds and story events. Its own patch notes connect weapon
aspects to named figures, record changes to distinct attack behaviours, and
mention dialogue requirements acknowledging particular weapon aspects.
These are implementation examples of story and builds sharing a vocabulary.
They do not establish that every lore association caused a mechanical design.
[Hades FAQ](https://www.supergiantgames.com/blog/hades-faq/),
[Supergiant's Hades update notes](https://www.supergiantgames.com/blog/hades-updates/).

**Our inference.** Name a power after what its office believes, then make its
trigger enact that belief. A ferryman protects a crossing; a weigher rewards
the correct order. An item-description paragraph alone cannot provide identity.
Keep the established class perk and deepen its practice, rather than stacking
unrelated elemental spells onto a reaper.

### Destiny: a shared mechanical language supports combinations

**Evidence.** Bungie's Void 3.0 developer explanation separates class-specific
Aspects from shared complementary Fragments. Designers describe building new
actions around an element's fantasy, and codifying status effects so abilities
and equipment can interact consistently.
[Bungie, A Closer Look at Void 3.0, 9 February 2022](https://www.bungie.net/7/en/News/Article/51021).

**Our inference.** Reapers already has useful verbs: ambush, hold, parry, strike
in order, build cadence, reap. Reuse those exact events. A gift must change a
moment that the player can recognise, and the game must announce its activation.
Avoid a separate spell-resource bar or a second combat simulation.

### Guild Wars 2: specialisation changes moment-to-moment play

**Evidence.** ArenaNet's original elite-specialisation announcement describes
new combat styles, profession mechanics, skills and traits. Its example traits
include condition removal when evading. A player equips a limited selection,
and the announcement describes swapping outside combat.
[ArenaNet, Specializations, Part Two](https://www.guildwars2.com/en/news/specializations-part-two-reward-tracks-and-elite-specializations/).

**Our inference.** Growth needs a visible consequence beyond a larger damage
number. Reopening a seal, recovering resolve after a hold, or preserving a
rhythm through a miss changes the next decision. We retain any-class weapons
for experimentation, and do not add irreversible paid talent choices.

### Disco Elysium: actions become a character's internal history

**Evidence.** ZA/UM's Thought Cabinet explanation describes thoughts discovered
through what a character says and does, internalised over time, and resulting
in mechanical effects and sometimes new story opportunities. It explicitly
connects that system to reputation, perks and role-playing identity.
[Robert Kurvitz, Introducing The Thought Cabinet, 30 September 2019](https://discoelysium.com/devblog/2019/09/30/introducing-the-thought-cabinet).

**Our inference.** Let a reaper remember successful habits. Experience records
how much it has consumed; memory records how it hunted. Both are needed for
an awakening. We do not copy that game's time delays, slot economy, thought
names or penalties: Reapers sessions should remain short and comprehensible.

### Shadow of War: persistence makes procedural characters matter

**Evidence.** Monolith designer Chris Hoge's GDC session synopsis describes
designing memorable procedural orcs, exceptional encounters and continuing
player-character relationships. Only the published synopsis was inspected;
this document does not claim findings from watching the entire talk.
[GDC 2018, Helping Players Hate (or Love) Their Nemesis](https://www.gdcvault.com/play/1025150/Helping-Players-Hate-(or-Love)).

**Our inference.** The identity must survive repeated encounters and ownership
changes. A new owner receives the same genome, equipment, codex and soul
memory, not a fresh slot with the same portrait. We are borrowing the general
principle of persistence, not implementing the Nemesis system.

### Baldur's Gate 3: origins establish a perspective, not just a skin

**Evidence.** Larian's gameplay FAQ describes Origin characters as having
backstories, motivations and distinctive interactions. The page retains Early
Access wording, so it is used as a historical statement of the origin concept,
not as documentation of the game's current respec or release behaviour.
[Larian, Gameplay Information](https://larian.com/support/faqs/gameplay-information_52).

**Our inference.** Give each birth gift a concise voice and a reason for its
behaviour. Reveal that voice beside the actual rules and milestones. We do
not claim to have built a branching campaign, companion dialogue or quests.

### Slay the Spire: instrumentation and feedback must test the design

**Evidence.** Mega Crit's GDC session synopsis describes using metrics and
community feedback through Early Access while maintaining feel and difficulty.
The associated slide deck includes card-choice counts and confidence intervals.
[Anthony Giovannetti, GDC 2019 session](https://www.gdcvault.com/play/1025731/-Slay-the-Spire-Metrics),
[original slide deck](https://media.gdcvault.com/gdc2019/presentations/Giovannetti_Anthony_SlayTheSpire.pdf).

**Our inference.** Deterministic trials can expose unreachable triggers,
unbounded loops and pacing regressions. They cannot tell us whether a beginner
enjoys a gift on a real phone. Keep replayable observations and produce a local
pacing report; collect actual playtest feedback before claiming balance.
No new tracking service or personal-data collection is needed for these tests.
The committed [soul pacing trials](soul-pacing-trials.md) contain 1,152
reproducible encounters across all 72 birth builds, comparing birth strength
with level-20 potential. Rebuild them in `reapers_shared` with
`fvm dart tool/soul_pacing.dart`. The scenario and bot limits are stated in the
report; this one rare-husk scenario is evidence of reachable play and growth,
not a comprehensive balance claim.

## The decision: an office, a birth gift, a temperament, a memory

1. **Office:** the existing six classes and their existing perks.
2. **Birth gift:** three powers per class, one for each hunting practice.
3. **Temperament:** one of four modest advantages with explicit costs.
4. **Memory:** independent counts of successful hunts demonstrating each
   practice. These counts belong to an individual reaper, not to its account.
5. **Awakening:** birth strength plus three upgrades requiring both a level
   and the corresponding memory. A rich feeding can speed XP but cannot replace
   practice. All three paths remain visible, even when only one feeds this gift.

There are 72 mechanical birth combinations before visual traits, equipment or
history. This is not a promise that every NFT has a never-repeated power.
Two reapers can share a gift and still have different dispositions, appearances,
records and awakenings. Buying a grown NFT does buy its earned progression;
the system does not claim to eliminate every advantage of purchasing one.

### Why memories are earned on absorption

One successful hunt grants at most one mark per path. Repeated holds in the
same fight grant no extra marks. An escape grants none. This makes pausing the
fight to farm shrieks unattractive, retains a clear end to the physical session,
and avoids separately monetising training. The marks are calculated from the
server's replay, never accepted as client-supplied counters.

An Unseen mark requires an ambush; Stillness requires at least one unflinching
hold; Rhythm requires five genuinely landed consecutive hits within the cadence
window. A gift's free starting cadence does not count as five landed hits.
Rifts and practice hunts do not award soul memories in this version.

### Why birth identity is derived from R1

Existing reapers already have permanent genome codes. Inventing a second random
roll would make old NFTs dependent on when they were imported. The fixed R1
mapping derives the gift from class, given name, epithet and gaze; disposition
from stature, gauntness and tatter. Omen, palette and aura are excluded, so rare
decorations do not mean rarer strength. The studio can edit an unsaved specimen;
owned genomes cannot be changed. Future mapping changes require a new genome
version, not a silent alteration of R1.

Old records retain XP, equipment and codex. Missing memories start at zero:
we cannot invent how a historical hunt was played. Combat-log version 5 marks
the new rules. Version 4 logs are refused rather than reinterpreted. Deploy
server and clients together; version mismatches require an app update.

## The original lore

The veil is not a kingdom ruled by a single death god. It is unfinished work.
Six offices keep it moving: harvesting what has ended, carrying what cannot
cross, measuring what is owed, choosing the final moment, enduring absence,
and announcing departure. Their tools are responsibilities made visible.

A birth gift is an office's peculiar way of remembering. A temperament is how
its bearer approaches that responsibility. A hunt is a short argument between
that memory and a spirit that will not leave. Awakening means the reaper has
proved its approach often enough for the veil to answer it. This is original
fiction proposed and implemented here, not a sourced claim about a religion.
The six roles remain job titles; no named deity or sacred likeness is added.

The 18 gift names, original lore lines, exact triggers and future strengths
appear together in the studio's Soul view and the player's codex. Narrative
claims are small enough to be fulfilled by the simulation. Future story work
should react to these recorded deeds rather than add a disconnected biography.

## Implementation map

| Responsibility | Source |
| --- | --- |
| Birth mapping, gifts, dispositions, rank thresholds | `reapers_shared/lib/src/reaper_soul.dart` |
| Derived combat values | `reapers_shared/lib/src/hunt/hunt_stats.dart` |
| Trigger execution and replay observations | `reapers_shared/lib/src/hunt/hunt_sim.dart` |
| Settlement, one reward per husk, per-reaper persistence | `reapers_shared/lib/src/player_state.dart`, `reaper_state.dart` |
| Canonical NFT record and transfer | `reapers_server/lib/src/solana/reaper_ledger.dart` |
| All player-facing prose | `reapers_flutter/lib/l10n/app_en.arb` |
| Shared presentation | `reapers_flutter/lib/widgets/soul_ledger.dart` |
| Unsaved experimentation | `reapers_flutter/lib/studio/reaper_studio.dart` |
| Rules and initial balance values | [gameplay.md](../gameplay.md) |
| Launch dependencies and decision owners | [launch-readiness.md](../launch-readiness.md) |

## What remains uncertain

The awakening thresholds and power strengths are initial playtest values.
Defensive gifts require an error before helping, so perfect autoplay understates
their value. Solo trials do not establish cooperative balance. Simulations can
be automated by an attacker; replay proves consistency with inputs, not that a
human physically performed them. GPS accuracy likewise does not prove presence.
The simulated husk economy is not a real NFT burn, and an off-chain receipt is
not a confirmed chain transaction. See the launch record before enabling money.
