# How Reapers plays

This is the rulebook as the code plays it. Every number here lives in `reapers_shared`, which both the game and the server run, so this file and the code can be checked against each other. Where a number is a first guess that needs playtesting on real phones, it says so.

## The loop

1. Take the oath. The shuffle binds you to one reaper. You do not choose it.
2. Buy an NFT into your vault (a simulated market for now; see [nfts.md](nfts.md)).
3. Release it into the veil. It becomes a husk and surfaces 6 to 14 seconds later, with a "husk sighted" notice.
4. Hunt it through your phone. Read the room so it has somewhere to be, break its ward, then reap it.
5. Absorbing it feeds your reaper: experience (with a roll of luck), anima, and an entry in your codex. The NFT is gone for good: absorbing is a sink.
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

### Reading the room

A hunt opens with "Something is nearby". Before the husk shows itself, you look around and the phone reads the room:

- For a husk that walks, tip the phone down and sweep it slowly across the floor in front of you.
- For one that flies, look around at about head height.
- Pointing at the ceiling or at your own feet reads nothing, and neither does turning too fast. The screen says which.

The circle around you is cut into 72 slices of 5°, and the dial on screen fills in as each is read. When an arc of 115° has been read, with room for the husk all along it, the husk gathers itself into view somewhere inside it, with "It is here".

There are two ways the room gets read, and the game picks whichever the phone can manage.

**A phone that can see** (a camera, motion sensors, and the depth model: the Seeker and other Android phones today) takes a picture about twice a second while you sweep and works out from each one how far away everything in it is. From those it learns:

- **How far below the phone the floor is.** The screen shows it as it is found. Sit down and it comes up to meet you.
- **How far the way is clear in each slice**: to the sofa, the table leg, the wall. The dial's spokes are drawn that long, so it takes the shape of your room, and a slice with no room for the husk is red.

A picture is only taken while the phone turns slower than 0.7 rad/s. A slice counts as read once it has been followed out to something standing in it, or along clear floor far enough for the husk to stand. If most of the circle has been read and nowhere has room, the screen says to turn somewhere more open. After 30 seconds without a plan there is a "Hunt anyway" button, which starts the hunt with nothing known about the room.

**A phone that cannot see** (a simulator, a refused camera, an iPhone until the depth model is tried on one) goes by where you have looked: a slice is read after 0.3 seconds of steady looking at the right height, turning slower than 1.5 rad/s. It learns nothing about the floor or the furniture.

Either way the result is the hunt's plan:

- **The arc** the husk keeps to. It is never anywhere that has not been read. A plan narrower than 100° is not allowed: it would pen the husk in.
- **The height of your eye above the floor**, measured, or 1.45 m when it could not be.
- **How far the way is clear along each slice**, when the phone could see.

The plan is about 80 bytes. It is recorded with the hunt, and the server replays the hunt inside the same plan.

### Where a husk is

A husk has a real place around you, in metres, and it moves through it.

- A husk that walks stands on the floor: the floor the phone measured, or one taken to be 1.45 m below it. It ambles a short way every second or three.
- A husk that flies keeps to a band of air around head height and drifts about in it.
- It turns its body to face the way it is going, a little at a time, and its head turns to watch you.
- It stays between its nearest and farthest distance from you (see the table of husks), and never goes past what stands in its way. Where the room gives it less than its nearest distance it comes closer, down to 1.2 m and no nearer.
- It keeps half its own width clear of whatever is in the way, and it starts drawing in before it gets there: the room it has opens out by 2 m for every radian to the side of an obstacle. So it runs round the front of a sofa in a curve instead of into it.

### Stalk

The husk has not noticed you. A marker at the edge of the screen shows which way it is.

- Turn faster than 1.7 rad/s and it starts to notice, sooner the faster you turn. It notices on its own after 25 seconds.
- Hold your aim within 9° of it while turning slower than 0.6 rad/s for 66 ticks (1.1 s) and you spring an ambush: every seal you can see opens, your first strike is doubled, and the husk is worth 10% more.
- If it notices you, or you strike it first, the fight starts without the ambush and the husk bolts: it runs, or darts through the air, 45° to 105° round you at full speed.

### Fight

Each husk has four or five seals: on its brow, its chest, its flanks, its back. Up to two are open at a time, each for 2.5 to 4.3 seconds.

- A seal can only be struck from a side you can see. One well round the far side of the body (more than about 105° from the nearest point) is hidden, so a husk with its back to you shows different seals from one facing you. From any angle at least two are in view.
- Tap an open seal to strike it. A struck seal shuts for 44 ticks.
- Your weapon sets how often you can strike. A tap made up to 8 ticks early is held and played when the swing finishes.
- Strikes within 78 ticks of each other build cadence: +10% each, up to +50%.
- A tap on the body away from a seal glances for a quarter of a strike. A tap that hits nothing breaks your cadence.
- The fight lasts 75 seconds. After that the husk escapes.

### Shriek

Every 6.5 to 10 seconds the husk stops where it is, turns on you and draws breath (54 ticks of warning, its mouth opening), then shrieks for 66 ticks.

- Move faster than your flinch speed, or tap, and you flinch: you lose one resolve and your cadence. The first 7 ticks are forgiven.
- Out of resolve, the husk escapes.
- Hold still through the whole shriek and every seal in view opens for 170 ticks. It is facing you, so that is its front.
- Two or three seconds after a shriek it bolts again.

The warning is a darkening of the edges of the screen and a line of text. It is deliberately not a flash: see the safety notes below.

### Charge

A husk fights back. Every 8 to 14 seconds of the fight it turns on you and comes: a walker runs at you along the floor, a flier swoops down to eye level. It takes 66 ticks (1.1 s) to arrive, to within 0.9 m, and a ring closes on it as it comes.

- Swing the phone across it at 2.0 rad/s or faster and you turn it aside. That is a flick of the wrist: a third of a turn a second. A swing in the first 10 ticks does not count, since you were probably still turning to follow it.
- Turned aside, it stands stunned where it got to for 100 ticks, right in front of you, with every seal in view open. Then it bolts.
- Do nothing and it strikes: you lose one resolve and your cadence, the edges of the screen bruise, and it runs off. Out of resolve, the husk escapes.
- You can keep striking its seals while it comes.

A shriek asks you to freeze and a charge asks you to swing, so the two are never close together: there are always at least 90 ticks between one ending and the other beginning. A husk in a rift does not charge.

### Reap

When the ward reaches zero the husk stops dead, hangs open, and an arc appears across it, 52° wide.

- Bring your aim to the start of the arc, then sweep along it.
- Sweeping faster than 2.1 rad/s, or straying from the arc, tears the cut.
- The grade decides the reward: ragged ×1.0, clean ×1.25, perfect ×1.5.
- Run out of time and the husk recovers 30% of its ward and runs. A second unfinished reap and it escapes.

### Testing difficulty

A build that is not a release has a "Testing difficulty" button on the screen before each hunt. It starts on easy, so that someone who has never played can bring a husk down and see a whole hunt. It changes what a hunt asks, never what it pays.

| | Easy | Normal | Hard |
| --- | --- | --- | --- |
| The husk's ward | ×0.35 | ×1 | ×1.3 |
| How far a tap may land from a seal | ×1.75 | ×1 | ×0.85 |
| How fast you may move in a shriek | ×3 | ×1 | ×0.8 |
| Flinches and blows you can take | +3 | +0 | +0 |
| The reap window | ×2.5 | ×1 | ×0.8 |
| How far the cut may stray | ×2 | ×1 | ×0.8 |
| Unfinished reaps allowed | 4 | 2 | 2 |
| How fast the husk moves | ×0.6 | ×1 | ×1.15 |
| How long the fight may run | 150 s | 75 s | 75 s |
| How long a charge takes to arrive | ×1.6 | ×1 | ×0.85 |
| The swing that turns a charge aside | ×0.6 | ×1 | ×1.15 |

A simulated beginner (slow to react, wide with every other tap) takes every creature on easy and loses most hunts on hard. The difficulty is recorded with the hunt. The server replays at the recorded difficulty, and refuses anything but normal when it runs in production. A release build has no button and plays on normal.

### Feeding

Two reapers who cut the same husk equally well do not get exactly the same out of it. When a husk is absorbed, the keeper of the books rolls how well the reaper fed:

| Feeding | Experience | Chance |
| --- | --- | --- |
| Meagre | ×0.85 | 25% |
| Fair | ×1.0 | 50% |
| Rich | ×1.25 | 20% |
| A feast | ×2.0 | 5% |

On average that is ×1.06: a small gift, never a tax. It scales experience only. Anima is not rolled.

The roll is made by the keeper from a seed the player never sees: the server's own secure random number for an online player, the phone's for an offline one. It is made once, when the husk is absorbed, so it cannot be re-rolled. The server signs the result; see [nfts.md](nfts.md).

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

A husk is what is left of an NFT. Every supported collection has a creature of its own, and each NFT dresses it: three colours, the colour of the light in its eyes, and at most one thing it wears. The one epic of each collection is gilded. An NFT's picture in the game is a portrait of its own creature.

Its ward and its worth come from its rarity.

| Rarity | Ward | Experience | Anima | Simulated price |
| --- | --- | --- | --- | --- |
| Common | 280 | 40 | 16 | 0.02 SOL |
| Uncommon | 420 | 70 | 28 | 0.045 SOL |
| Rare | 760 | 120 | 48 | 0.11 SOL |
| Epic | 1400 | 210 | 85 | 0.26 SOL |

| Collection | Creature | Gets about | Stays | Ambles | Bolts |
| --- | --- | --- | --- | --- | --- |
| Degenerate Ape Academy | A heavy ape on its knuckles, 1.1 m tall | On the floor | 2.6 to 4.6 m away | 0.7 m/s | 2.6 m/s |
| Thugbirdz | A hunched bird in a visor | In the air, 0.15 m below the eye to 0.85 m above it | 2.2 to 3.6 m | 0.5 m/s | 3.2 m/s |
| Pesky Penguins | A squat penguin that waddles, 0.9 m tall | On the floor | 2.2 to 4.2 m | 0.45 m/s | 1.8 m/s |
| Sollamas | A long-necked llama under a striped blanket, 1.44 m tall | On the floor | 2.8 to 4.8 m | 0.6 m/s | 2.8 m/s |
| Solarians | A robot of mismatched parts on two legs, 1.12 m tall | On the floor | 2.4 to 4.4 m | 0.5 m/s | 2.0 m/s |

The body of a rift is not a creature: it is the reliquary stack, a tower of cubes 10 m tall and 17 m away, each face wearing one of the collection's portraits. It does not move. Four seals at the seams.

A level-one reaper takes a common husk in about half a minute and cannot take an epic one. `dart reapers_shared/tool/pacing.dart` prints the whole table for every class and level.

Why these five, and the hundred or so they were chosen from, is in [research/solana-collections.md](research/solana-collections.md).

## Rifts

A rift is a whole dead collection piled into one body. It takes a circle of four.

- **Where.** Each rift is at a real place: London Bridge (Degenerate Ape Academy), Shibuya Crossing (Thugbirdz), Third Mainland Bridge in Lagos (Sollamas), Brooklyn Bridge (Pesky Penguins), Golden Gate Bridge (Solarians). This version does not check where you are standing. See the limits below.
- **Who.** A rift opens only to a reaper who has absorbed at least one NFT from its collection. The way to a collection's raid runs through buying that collection.
- **Gathering.** Reapers join and stand ready. When everyone present is ready there is a three-second count. A rift waits at most 60 seconds. Seats nobody takes are filled by shades: bots one level below the circle's average, slower to react and wider of the mark than a careful person.
- **The ward.** It is sized to the circle: the sum of everyone's best damage per second × 0.84 × 70 seconds. A rift is as hard for four beginners as for four veterans. What it asks for is company and care, not levels.
- **The fight.** Each reaper fights the husk through their own phone, with their own seals, exactly as in a hunt but with nothing to stalk. Every strike anyone lands comes off the one shared ward. The shrieks come at the same instant for everyone. Flinch away all your resolve and you are cast out.
- **The limit.** 120 seconds. Four careful reapers break the ward in about a minute; one with three shades in about 80 seconds.
- **Sealing.** When the ward breaks everyone has one reap window (8 seconds for the room). Half the reapers still standing, rounded up, must make their cut, and at least one cut must be a person's: shades can break a ward but only the living can shut a rift.
- **Pay.** 220 experience and 90 anima for a clean cut and a fair share of the work. Your share of the ward scales that from 60% (did nothing) to 100% (a quarter of the ward or more). The cut scales it like a reap grade; no cut is ×0.75 and cast out is ×0.5. Leave early and you are paid nothing. A failed rift costs nothing.

`dart reapers_shared/tool/rift_pacing.dart` prints how rifts go for circles of different sizes and levels.

## Music

Three recordings, each made by someone else and released for anyone to use. Nothing was composed or generated for the game.

| Where | Piece | By |
| --- | --- | --- |
| The sanctum and every page of the ledger | J.S. Bach, Prelude No. 8 in E-flat minor, BWV 853 | Kimiko Ishizaka, on a concert grand. Public domain. |
| A hunt | Anabasis I | Scott Buckley. CC BY 4.0. |
| A rift | Katabasis I | Scott Buckley. CC BY 4.0. |

A hunt never switches tune. It has one piece, and the hunt changes how that piece is heard: how loud, and how muffled, as if through the veil.

| Moment | Loudness | Veil |
| --- | --- | --- |
| Reading the room | 0.22 | 0.9 |
| Stalking, unseen | 0.30 | 0.8 |
| Stalking, steadied on the husk | up to 0.45 | down to 0.5 |
| The fight | 0.8 | clear |
| The husk stunned | 1.0 | clear |
| It draws breath | falls from 0.8 to 0.12 | rises to 0.9 |
| The shriek | 0.06 | full |
| It charges | 1.0 | clear |
| Ward broken, the reap | 0.45 | 0.45 |
| Absorbed | silence | |
| Escaped | 0.25 | 0.85 |

So the music is one more thing telling you what to do. It comes clear when you may strike, drains away to almost nothing for the moment you must hold still, swells when the husk comes at you, and is gone by the time the husk has come apart.

The veil is a low-pass filter: clear is everything up to 16 kHz, full is only what lies under 320 Hz. Music can be turned off on the colophon page, which also carries the credits two of the pieces require.

## Safety

The game asks people to hold a phone up outdoors, so:

- You only ever turn on the spot. Nothing in the game asks you to walk while looking at the screen, and every hunt opens with a line saying so.
- Nothing flashes. A strike lights the husk in its own colour rather than to white, and the shriek warning darkens the screen instead of flashing it.
- Everything the hunt needs is on screen as text as well as colour, and every target is at least 44 points across.

## Reading the environment

What the game does, on a phone that can see:

- It shows the real camera feed, drawn through the camera's own lens so that husks hold their place as you turn.
- It measures how far down the floor is, and husks that walk stand on it with a soft shadow under them.
- It finds what stands on the floor and how far away, and husks go round it.
- It only puts a husk where it has looked.

On a phone that cannot see, the floor is taken to be 1.45 m down and nothing is known about the furniture.

What it still does not know anywhere:

- **Where you walk.** The game follows the phone turning, not moving. Step sideways and the husks come with you.
- **What is in front of what.** A husk is drawn over the camera feed and is never hidden behind a chair.
- **Anything it has not been shown.** The room is read once, before the hunt. Move a chair during it and the husk walks through the chair.
- **The ceiling.** A husk that flies keeps clear of walls and cupboards, but its band of air tops out 0.85 m above your eye whatever the ceiling's height.
- **What things are.** It knows something is in the way. It does not know a sofa from a wall, so nothing hides behind one or jumps up on one.

[environment.md](environment.md) says how the seeing works, how far it can be trusted, and what would come next.

## What is simulated, and what is not built

- **The NFTs.** The five collections are real ones from Solana's first wave. Nothing else about them is: no wallet, no mint, no chain, and the creatures are our own models. See [nfts.md](nfts.md).
- **Rifts are not located.** The sites have real coordinates but nothing checks yours. Geofencing is the missing piece.
- **Rift reports are trusted.** A lone hunt is replayed on the server before it pays. A rift takes each reaper's word for the damage they dealt.
- **Rifts have no cost and no cooldown.** That is why they pay like one epic husk and no better. Before launch they need a key, a cooldown or both.
- **Receipts are signed but go nowhere.** Nothing posts them to a chain yet.
- **Reaper against reaper** is not built.
- **Sound effects** are not built. Music and haptics are.
- **The numbers** were tuned against a simulated player, not people. Expect to move them after the first playtest on real phones.
