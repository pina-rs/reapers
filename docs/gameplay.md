# How Reapers plays

This is the rulebook as the code plays it. Every number here lives in `reapers_shared`, which both the game and the server run, so this file and the code can be checked against each other. Where a number is a first guess that needs playtesting on real phones, it says so.

## The loop

1. Take the oath. The shuffle binds you to one reaper. You do not choose it.
2. Buy an NFT into your vault (a stand-in market for now; see [nfts.md](nfts.md)).
3. Release it into the veil. It becomes a husk and surfaces 6 to 14 seconds later, with a "husk sighted" notice.
4. Hunt it through your phone. Break its ward, then reap it.
5. Absorbing it pays experience and anima, and writes it in your codex. The NFT is gone for good: absorbing is a sink.
6. Spend anima in the armory. Level up. Absorb three of a collection to complete its set.
7. Carry one of a collection's dead and that collection's rift opens to you: a husk too big for one reaper, fought by a circle of four at a real place.

Up to three husks can be loose at once. A husk that escapes stays loose and can be hunted again.

## What is different about it

The brief said defeating an NFT "should be just tapping on the phone". Tapping alone is what every AR game already does, so the hunt keeps the tap and builds two things around it that only work because the phone is in your hand:

- The phone is the blade. Where it points is where you look and where you cut.
- Stillness is the skill. The husk notices fast movement, punishes you for flinching, and can only be reaped with a slow, steady sweep. The hard thing in this game is holding a phone still while something shrieks at you.

In a rift, everyone in the circle is told to hold still at the same instant. Four people on a bridge freezing together is the picture the raid is built around.

## A hunt

The hunt runs at a fixed 60 ticks a second whatever the display does. You never see your reaper during it: you are the reaper.

### Stalk

The husk is somewhere around you, 55° to 180° from where you first faced. A marker at the edge of the screen shows which way.

- Turn faster than 1.7 rad/s and it starts to notice, sooner the faster you turn. It notices on its own after 25 seconds.
- Hold your aim within 9° of it while turning slower than 0.6 rad/s for 66 ticks (1.1 s) and you spring an ambush: every seal opens, your first strike is doubled, and the husk is worth 10% more.
- Strike it first and the fight starts without the ambush.

### Fight

Each husk has three or four seals. Up to two are open at a time, each for 2.5 to 4.3 seconds, and only where they face you.

- Tap an open seal to strike it. A struck seal shuts for 44 ticks.
- Your weapon sets how often you can strike. A tap made up to 8 ticks early is held and played when the swing finishes.
- Strikes within 78 ticks of each other build cadence: +10% each, up to +50%.
- A tap on the body away from a seal glances for a quarter of a strike. A tap that hits nothing breaks your cadence.
- The fight lasts 75 seconds. After that the husk escapes.

### Shriek

Every 6.5 to 10 seconds the husk draws breath (54 ticks of warning), then shrieks for 66 ticks.

- Move faster than your flinch speed, or tap, and you flinch: you lose one resolve and your cadence. The first 7 ticks are forgiven.
- Out of resolve, the husk escapes.
- Hold still through the whole shriek and every seal opens for 170 ticks.

The warning is a darkening of the edges of the screen and a line of text. It is deliberately not a flash: see the safety notes below.

### Reap

When the ward reaches zero the husk hangs open and an arc appears across it, 52° wide.

- Bring your aim to the start of the arc, then sweep along it.
- Sweeping faster than 2.1 rad/s, or straying from the arc, tears the cut.
- The grade decides the reward: ragged ×1.0, clean ×1.25, perfect ×1.5.
- Run out of time and the husk recovers 30% of its ward. A second unfinished reap and it escapes.

## Reapers

Six classes. Each has four ratings that always total 13, one perk and its own weapon, so no shuffle result is simply better than another.

| Class | Strike | Reach | Capture | Resilience | Perk | Weapon |
| --- | --- | --- | --- | --- | --- | --- |
| Harvester | 3 | 5 | 3 | 2 | Cleave: a strike also catches other open seals nearby, for 40% each | Scythe |
| Ferryman | 3 | 2 | 3 | 5 | Steadfast: the first flinch of a hunt costs nothing | Lantern pole |
| Weigher | 4 | 2 | 4 | 3 | Measure: seals are numbered, and striking them in order hits 20% harder | Balance flail |
| Chooser | 5 | 4 | 2 | 2 | Chosen blow: the strike after an unflinching hold lands three times as hard | Spear |
| Hollow | 4 | 3 | 2 | 4 | Chain: cadence builds to +100% | Chain whip |
| Mourner | 1 | 4 | 5 | 3 | Toll: each unflinching hold adds 80 ticks to the reap window | Bell |

What the ratings do:

- Strike: ward removed per strike is `4.0 × (0.85 + 0.1 × strike) × weapon × temper × level × sets`.
- Reach: how far from a seal a tap may land, `3.2° + 0.55° × reach + weapon reach`.
- Capture: the reap window is `3.5 s + 0.5 s × capture`, and the arc tolerance is `5° + 0.8° × capture`.
- Resilience: resolve is `2 + (resilience + 1) ÷ 2`, and flinch speed is `0.42 + 0.035 × resilience` rad/s.

A reaper also has a look the shuffle draws: three heads, three mantles, four robes and three colourways per class, plus stature, gauntness, gaze and tatter, an aura, a name, and an omen (none 80%, ashen 12%, gilded 6%, eclipsed 2%). Omens change colour only.

### Levels

Level cap 30. Going from level `n` to the next costs `90 + 55(n − 1) + 6(n − 1)²` experience. Each level adds 7% of a level-one strike. Each completed set adds 3%.

### Weapons and outfits

Any class can buy any weapon. Each kind has its own feel:

| Weapon | Strike | Reach | Ticks per swing |
| --- | --- | --- | --- |
| Scythe | ×1.15 | +1.6° | 26 |
| Lantern pole | ×1.0 | +0.8° | 22 |
| Balance flail | ×1.0 | +0.4° | 24 |
| Spear | ×1.45 | +0° | 32 |
| Chain whip | ×0.8 | +0.6° | 16 |
| Bell | ×1.3 | +2.4° | 28 |

Each weapon has four tempers, and each outfit is one of the same four states. A temper changes the strike. An outfit changes only how the reaper looks.

| State | Weapon strike | Weapon level | Weapon price | Outfit level | Outfit price |
| --- | --- | --- | --- | --- | --- |
| Shrouded | ×1.0 | 1 | free for your class, 120 anima otherwise | 1 | owned |
| Sworn | ×1.3 | 4 | 220 | 5 | 300 |
| Scorched | ×1.7 | 9 | 640 | 12 | 900 |
| Hollowed | ×2.2 | 15 | 1500 | 20 | 2400 |

## Husks

A husk's body (its vessel) comes from its collection and wears the NFT's own image. Its ward and its worth come from its rarity.

| Rarity | Ward | Experience | Anima | Stand-in price |
| --- | --- | --- | --- | --- |
| Common | 280 | 40 | 16 | 0.02 SOL |
| Uncommon | 420 | 70 | 28 | 0.045 SOL |
| Rare | 760 | 120 | 48 | 0.11 SOL |
| Epic | 1400 | 210 | 85 | 0.26 SOL |

| Vessel | Behaviour |
| --- | --- |
| Frame wraith | A floating frame. Four seals at the corner nails. |
| Lantern shade | A turning paper lantern. Three seals that can only be struck while they face you. |
| Shard swarm | Mirror shards around a core. One seal at the centre, three that orbit. |
| Scroll eel | A ribbon that swims in a slow circle around you, so you have to follow it steadily. |
| Reliquary stack | A tower of image cubes, 10 m tall and 17 m away. Rift only. Four seals at the seams. |

A husk that holds its ground (every one but the eel and the stack) darts 45° to 105° to one side a couple of seconds after each shriek.

A level-one reaper takes a common husk in about half a minute and cannot take an epic one. `dart reapers_shared/tool/pacing.dart` prints the whole table for every class and level.

## Rifts

A rift is a whole dead collection piled into one body. It takes a circle of four.

- **Where.** Each rift is at a real place: London Bridge (Tin Saints), Shibuya Crossing (Mothlight Parade), Third Mainland Bridge in Lagos (Static Choir), Brooklyn Bridge (Brine Hounds). This version does not check where you are standing. See the limits below.
- **Who.** A rift opens only to a reaper who has absorbed at least one NFT from its collection. The way to a collection's raid runs through buying that collection.
- **Gathering.** Reapers join and stand ready. When everyone present is ready there is a three-second count. A rift waits at most 60 seconds. Seats nobody takes are filled by shades: bots one level below the circle's average, slower to react and wider of the mark than a careful person.
- **The ward.** It is sized to the circle: the sum of everyone's best damage per second × 0.84 × 70 seconds. A rift is as hard for four beginners as for four veterans. What it asks for is company and care, not levels.
- **The fight.** Each reaper fights the husk through their own phone, with their own seals, exactly as in a hunt but with nothing to stalk. Every strike anyone lands comes off the one shared ward. The shrieks come at the same instant for everyone. Flinch away all your resolve and you are cast out.
- **The limit.** 120 seconds. Four careful reapers break the ward in about a minute; one with three shades in about 80 seconds.
- **Sealing.** When the ward breaks everyone has one reap window (8 seconds for the room). Half the reapers still standing, rounded up, must make their cut, and at least one cut must be a person's: shades can break a ward but only the living can shut a rift.
- **Pay.** 220 experience and 90 anima for a clean cut and a fair share of the work. Your share of the ward scales that from 60% (did nothing) to 100% (a quarter of the ward or more). The cut scales it like a reap grade; no cut is ×0.75 and cast out is ×0.5. Leave early and you are paid nothing. A failed rift costs nothing.

`dart reapers_shared/tool/rift_pacing.dart` prints how rifts go for circles of different sizes and levels.

## Safety

The game asks people to hold a phone up outdoors, so:

- You only ever turn on the spot. Nothing in the game asks you to walk while looking at the screen, and every hunt opens with a line saying so.
- Nothing flashes. A strike lights the husk in its own colour rather than to white, and the shriek warning darkens the screen instead of flashing it.
- Everything the hunt needs is on screen as text as well as colour, and every target is at least 44 points across.

## What is a stand-in, and what is not built

- **Augmented reality is three degrees of freedom.** The camera feed is real and husks hold their direction as you turn, using the gyroscope and accelerometer. Husks are not anchored to surfaces and you cannot walk around one. That needs ARKit and ARCore; the pose source is behind one interface so it can be swapped in.
- **Rifts are not located.** The sites have real coordinates but nothing checks yours. Geofencing is the missing piece.
- **Rift reports are trusted.** A lone hunt is replayed on the server before it pays. A rift takes each reaper's word for the damage they dealt.
- **Rifts have no cost and no cooldown.** That is why they pay like one epic husk and no better. Before launch they need a key, a cooldown or both.
- **Reaper against reaper** is not built.
- **Sound** is not built. Haptics are.
- **The numbers** were tuned against a simulated player, not people. Expect to move them after the first playtest on real phones.
