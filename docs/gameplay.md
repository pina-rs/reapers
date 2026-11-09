# How Reapers plays

This is the rulebook as the code plays it. Every number here lives in `reapers_rules`, which both the game and the server run, so this file and the code can be checked against each other. Where a number is a first guess that needs playtesting on real phones, it says so.

The rules' conformance tests replay, hunt by hunt and to the bit, fixtures written by the rules' first implementation (`crates/reapers_rules/tests/fixtures`), so saves and hunt logs from before the port still play the same way. A number changed on purpose changes its test too.

## The loop

1. Take the oath. The shuffle binds you to one reaper. You do not choose it.
2. Buy an NFT into your vault (a simulated market for now; see [nfts.md](nfts.md)).
3. Release it into the veil. It becomes a husk and surfaces 6 to 14 seconds later, with a "husk sighted" notice.
4. Hunt it through your phone. Trace its sigil to draw it out, weaken it with the weapons on your crosshair, then catch it in a soul orb.
5. Absorbing it feeds your reaper: experience (with a roll of luck), anima, and an entry in your codex. The NFT is gone for good: absorbing is a sink.
6. Spend anima in the armory. Level up. Absorb three of a collection to complete its set.
7. Carry one of a collection's dead and that collection's rift opens to you: a husk too big for one reaper, fought by a circle of four at a real place.

Up to three husks can be loose at once. A husk that escapes stays loose and can be hunted again.

## What is different about it

The brief said defeating an NFT "should be just tapping on the phone". Tapping alone is what every AR game already does, so the hunt keeps the buttons and builds two things around them that only work because the phone is in your hand:

- The phone is the sight. Where it points is the crosshair, and every shot, stab, swing, throw and orb goes there. The two thumbs only say when: FIRE on the right, ORB on the left.
- Stillness is the skill. The husk notices fast movement and punishes you for flinching, and the orb that catches it is timed by holding on while a ring closes. The hard thing in this game is holding a phone still while something shrieks at you.

In a rift, everyone in the circle is told to hold still at the same instant. Four people on a bridge freezing together is the picture the raid is built around.

## A hunt

The hunt runs at a fixed 60 ticks a second whatever the display does. You never see your reaper during it: you are the reaper.

### Drawing it out

A hunt opens with "Something is nearby". The husk has not shown itself yet: you draw it out by tracing its sigil.

A sigil is a handful of lights hung in the air around you, in the husk's own colour. Turn until the ring in the middle of the screen rests on the first light and hold the phone still: the ring fills, the light catches with a knock, and a thread runs from it to the ring. Find the next, and the next. When the last light catches, the husk gathers itself into view, with "It is here".

- The sigil hangs only once the phone has said where it is pointing, and its first light is put where you face: the whole shape is still around you, but finding the first one is not a turn to somewhere you were never looking. While the phone is finding its bearing the screen says so; if it has none, the hunt says to drag instead.
- The lights catch only in order. The next one breathes and is bright against the room, and the ones to come are drawn faintly, so the whole shape is there to be seen.
- When the next light is out of view, an arrow at the edge of the screen points to it, and a line under the count says which way to turn. The arrow is read through the camera's own right and up, so it stays right when the phone is rolled, and a light behind you names a side rather than flickering across the back.
- A light only fills while you turn slower than 0.7 rad/s. Faster, and it holds where it is, and the line says to slow down. Rest on it and it says to hold still. Look away and it drains twice as fast as it filled.
- The ring at the middle is drawn on ink and edged in shadow, as is every light, so the next one can be found over a bright room as well as a dark one. The count above it says how many have caught, in words as well as in marks.
- The music draws nearer with every light that catches.

The sigil is drawn from the hunt's seed, so the same husk always has the same one. It hangs around wherever you face when you raise the blade, turned so its first light is where you are looking.

**A rarer husk has a harder sigil.** The lights stand in a row of evenly spaced columns, one to a column, each at one of three heights and never at the height of the one beside it. A common sigil runs straight along the row. Harder ones double back and cross themselves, but a trace never passes over a light it does not stop at, so the order can always be read from the shape.

| Rarity | Lights | Spread across | Ring must come within | Hold each for | Crosses itself | A caught light goes out after |
| --- | --- | --- | --- | --- | --- | --- |
| Common | 3 | 90° | 8° | 0.6 s | never | never |
| Uncommon | 4 | 110° | 7° | 0.7 s | once | never |
| Rare | 5 | 130° | 6° | 0.8 s | twice | 6 s without the next |
| Epic | 6 | 160° | 5.5° | 0.9 s | three times | 5 s without the next |

A light that goes out takes the thread back one light, and the thread reddens and fades as it comes near to going out ("It is slipping"). It never goes out while the ring is resting on the next light. A simulated reaper turning at an easy 1 rad/s traces a common sigil in about 4 seconds and an epic one in about 12. One turning at 0.3 rad/s loses lights on an epic sigil at normal and none at easy.

The testing difficulty widens or narrows how near the ring must come (×1.75 at easy, ×0.85 at hard) and lengthens how long a light waits at easy (×2). It never changes the sigil's shape.

After 60 seconds there is a "Hunt anyway" button, for anyone who cannot turn far enough or whose room will not let them.

#### Reading the room

Tracing the sigil is also how the phone reads the room. The lights hang where the phone has to look:

- For a husk that walks, between 14° and 42° below the horizon, so the camera sees the floor from under your feet to several metres out.
- For one that flies, between 6° below and 26° above: about head height, where it will be.

The arc the husk will keep to is the sigil's, out to its outermost lights and 15° beyond on either side (half of what the camera takes in across): 120° for a common husk, 190° for an epic one.

**A phone that can see** (a camera and the platform's own depth: ARCore's Depth API on Android, ARKit's scene depth on an iPhone with LiDAR) reads the room while you trace, whenever the phone turns slower than 0.7 rad/s, which is what holding on a light does. From what it sees it learns:

- **How far below the phone the floor is.** The screen shows it as it is found. Sit down and it comes up to meet you.
- **How far the way is clear in each slice** of a circle cut into 72 slices of 5°: to the sofa, the table leg, the wall. A small dial in the corner shows them, spokes as long as the way is clear and red where it is tight: so little clear floor that the husk can only stand there as close as it ever comes. A thin arc round the dial is the arc the husk will keep to.

No room is too small: where there is little clear floor the husk stands closer, and it does not refuse to appear. A slice inside the arc that the phone never saw is left open.

The floor is whatever level surface lies lowest in what you show it. Point at nothing but a table top, and the husk will stand on the table.

**A phone that cannot see** (a refused camera, a phone without platform depth such as the Seeker, the browser) traces the same sigil and learns nothing about the floor or the furniture. [environment.md](environment.md) says where the depth comes from.

Either way the result is the hunt's plan:

- **The arc** the husk keeps to. It is never anywhere that was not traced. A plan narrower than 100° is not allowed: it would pen the husk in.
- **The height of your eye above the floor**, measured, or 1.45 m when it could not be.
- **How far the way is clear along each slice**, when the phone could see.

The plan is about 80 bytes, or 230 when it also says what stands in the room. A phone that reads the room cell by cell also hands over the room itself, its grid (see [A room read cell by cell](#a-room-read-cell-by-cell)), which adds its own length in 4 bytes and the grid's: 51 bytes for a floor with a desk on it, 160 for a room with a desk, a chair and a sofa, and never more than about 30 KB for the largest and most broken-up 10 by 10 m grid. It is recorded with the hunt, and the server replays the hunt inside the same plan. The sigil itself is not recorded: it decides when the hunt starts and where, and nothing about what it pays.

### Where a husk is

A husk has a real place around you, in metres, and it moves through it. Its movement, facing, keepaway and charge endpoint are measured from the stance: the origin where the room was first read. The phone's recorded eye may move within that stance without moving the husk's world.

- A husk that walks stands on the floor: the floor the phone measured, or one taken to be 1.45 m below it. It ambles a short way every second or three.
- A husk that flies keeps to a band of air around head height and drifts about in it.
- It turns its body to face the way it is going, a little at a time, and its head turns to watch you.
- It stays between its nearest and farthest distance from you (see the table of husks), and never goes past what stands in its way. Where the room gives it less than its nearest distance it comes closer, down to 0.8 m and no nearer.
- It keeps half its own width clear of whatever is in the way, and it starts drawing in before it gets there: the room it has opens out by 2 m for every radian to the side of an obstacle. So it runs round the front of a sofa in a curve instead of into it.
- It does not have to stay in front of things. Where the room has a ledge or something to hide behind, it leaps: see the next section.

### The room it lives in

A husk treats your furniture the way a cat does. On a phone that read the room, the plan says, for every direction, what stands at the end of the clear floor: how tall it is, whether its top is level enough to stand on, and whether there is room to hide behind it (floor or a wall seen beyond it, with space for the husk between).

- **Ledges.** A husk that walks leaps up onto a level top no lower than 0.3 m and no higher than it can jump: an ape 1.2 m, a llama 1.0 m, a robot 0.8 m, a penguin 0.55 m (so a penguin cannot get onto a counter). A bird comes down to perch on any ledge in reach. It lands 0.6 of its half width in from the front edge.
- **Cover.** A husk that walks hides behind anything at least half as tall as it is, as far behind it as the room allows and no more than 1.5 m past its farthest distance. Only what rises above the cover can be seen.
- **When.** When an ambling husk is ready to move it makes for a ledge or cover within 50° 35% of the time, when there is one. A bolting husk does so 45% of the time. Up on a ledge or behind cover, it stays put 40% of the time it is ready to move, and otherwise leaps down into the open in front.
- **Leaps** go at three quarters of its bolting speed and take at least 34 ticks. A leap rises high enough for the husk's feet to clear the front edge of what it leaps onto, off or over by 12 cm, where it passes it. It cannot be stopped in the air: a husk that is stunned, frozen or broken in mid-leap lands first.
- **Behind cover, out of reach.** A seal the cover hides cannot open and cannot be struck. A blow at the part of the body below the top of the cover hits the cover and is a miss.
- **It comes out to fight.** A husk draws breath only once it has landed, and one in cover leaps out as it does, so it never shrieks from hiding. It never charges from a ledge or from cover either: it comes down first.

The husk is also drawn as hidden. The phone keeps a picture of how far away the nearest thing is in every direction it looked while the sigil was traced, and a husk behind the counter is cut away wherever the counter is in front of it, so the camera's own picture of the counter shows instead. The phone only turns where the player stands, so that picture still lines up with the room during the fight, whichever way the phone points.

### A room read cell by cell

A phone that reads the room as a grid of 10 cm cells, each a level top, something standing, a wall or floor it could not see, hands the hunt that grid. A husk that walks then leaves the slices behind and moves through the room itself. Everything else about the hunt is as above: the same chances, the same moments, the same fight.

- **Its body is the creature table's.** On the grid a husk is the size the room was built for: an ape 0.60 m tall, a penguin 0.48 m, a llama 0.80 m, a robot 0.62 m. Its body and its seals shrink to match, so it is a smaller thing to hit. Off the grid every husk keeps the size in the table of husks.
- **Where it may be.** Never nearer than 1.5 m across the floor, except from the moment it charges until it is back out. Nowhere it stands, perches or hides is nearer than 1.6 m (a cell more, since a straight walk between two cells can pass half a cell nearer), and its routes go round you. When it takes up open floor it picks a distance between 2.5 and 3.5 m along the way it means to go, and the floor the camera can see nearest that. It perches and hides no farther than 5.0 m.
- **Where it goes.** A perch is the edge of anything raised at least 0.3 m, facing you. A hiding place is floor the camera could not see behind something; one near enough to the edge to look over is a peek. It only makes for one it can get to from where it stands, by walking and leaping.
- **How high it jumps.** An ape leaps up 1.0 m and down 1.5 m, and 1.2 m across a gap; a llama 0.9 up, 1.4 down, 1.1 across; a robot 0.8, 1.3, 0.9; a penguin 0.5, 0.9, 0.6. So a penguin gets onto a 75 cm desk by the 45 cm chair beside it, never in one bound.
- **How it walks.** It speeds up and slows down at 4 m/s² and turns at up to 240° a second. It slows into a corner so that it swings no more than 5 cm wide of the way on, and stops to turn where the way bends by more than 60°.
- **How it leaps.** It slows to a stand at the edge, facing the landing, gathers itself for 10 to 20 ticks (longer the higher the leap), flies the exact arc gravity (9.81 m/s²) gives it, touches down for 6 to 12 ticks and stands up again over 12 to 24, longer the farther it fell. The surface under it changes only at the edge of a leap, never in between. It cannot be stopped from the moment it gathers itself until it has stood up.
- **It comes down to fight.** As before, it never shrieks from hiding and never charges from a ledge or from cover: it walks and leaps down to open floor first, and nothing but the end of the fight stops it on the way. With no way down from where it is, it does not charge this time.
- **The charge** runs straight at you across the surface it stands on, to within 0.9 m, and stops short of anything in its way rather than run through the sofa or up the desk. Afterwards it walks back out past 1.5 m, even while it draws breath.
- **Cover is the grid's.** A seal is hidden, and a blow misses, wherever any cell between your eye and it stands taller than the line to it. An art thrown at nothing lands on the first surface along its throw. This heightfield is approximate: it cannot see beneath overhangs, unknown cells count as floor, and height and position are quantised. Without a grid the plan approximates cover as thin walls.
- **A husk that flies** keeps to its band of air and to the slices, even in a room read as a grid.

`cargo run --release -p reapers_rules --example pacing` prints the same bot against a penguin in a furnished room after the open table. A level-one reaper breaks a common husk there in 21 to 32 seconds (22 to 31 in the open), and a level-ten one an epic husk in 76 to 108 (76 to 109).

### Stalk

The husk has not noticed you. A marker at the edge of the screen shows which way it is.

- Turn faster than 1.7 rad/s and it starts to notice, sooner the faster you turn. It notices on its own after 25 seconds.
- Hold the crosshair within 9° of it while turning slower than 0.6 rad/s for 66 ticks (1.1 s) and you spring an ambush: every seal you can see opens for 170 ticks, your first hit lands twice as hard, and the husk is worth 10% more.
- If it notices you, or you fire first, the fight starts without the ambush and the husk bolts: it runs, or darts through the air, 45° to 105° round you at full speed.

### The crosshair and the four weapons

The crosshair is the middle of the screen. Where the phone points is where every shot, stab, swing, throw and orb goes. Nothing you touch says where: your thumbs only say when.

- **FIRE**, under the right thumb, uses whatever is in hand. One press is one use; holding it down does not fire again. It answers the moment the thumb lands.
- **ORB**, under the left thumb, is held and let go. See [Capture](#capture).
- **The rail** above FIRE holds four weapons. Tap one to draw it. Drawing takes 21 ticks, and nothing in the new hand can act until it is drawn.

Hits, visible seals, cover, ambush aim, reach, and orb and art throws are judged from the phone's recorded eye on that tick. Reach is measured from the phone; a dive is a virtual lunge whose reach is checked at its start. Orbs and arts that miss land along the ray from the eye.

Every weapon does a share of the reaper's hit, `H` (see [Reapers](#reapers)). A hit on an open seal is critical and lands twice as hard.

| Slot | Weapon | A press | Share of `H` | Reaches the husk when |
| --- | --- | --- | --- | --- |
| 1 | Grave pistol | One shot | 0.55 | the crosshair is on an open seal, or within the aim assist of its body |
| 2 | Bone knife | A dive with up to three stabs, or one stab in arm's reach | 0.8 a stab | the crosshair is within 2° of its body, and it is within 4.5 m (a dive) or 1.8 m (arm's reach) |
| 3 | The class weapon | A dive with one heavy swing, or a swing in arm's reach | the weapon's share, 0.8 to 1.85 | the crosshair is within the weapon's reach + 3° of its body, and it is within 3.5 m |
| 4 | The art, if the reaper knows one | A throw at the crosshair | see [Arts](#arts) | |

**The pistol** holds six rounds and fires at most one every 18 ticks. The aim assist is 3.5° for a reaper of middling reach, a quarter of a degree wider or narrower for each point of reach above or below 3. FIRE on an empty gun starts an 84-tick reload, which runs on if you change weapons. Press FIRE again between ticks 48 and 60 of the reload and it finishes at once, and every round of that magazine lands ×1.25: an active reload. Press earlier and the reload jams for 30 ticks more. The pistol fires even when the enemy is outside the crosshair or out of view; a shot that lands nowhere on the husk is a miss.

**The bone knife** dives. A press with the husk on the crosshair and within 4.5 m spends a dive: 13 ticks in, 30 at the husk, 18 back out. The first stab lands as you arrive, and each press in those 30 ticks stabs again, at least 10 ticks apart, three stabs at most. The knife holds three dives and gets one back every 180 ticks (3 s). A husk within 1.8 m, mid-charge or stunned after a parry, needs no dive: each press stabs, at least 10 ticks apart, and spends nothing. A stab at a stunned husk is critical wherever it lands.

**The class weapon** is the reaper's own scythe, pole, flail, spear, whip or bell, or a greatsword or axe bought for it. A press with the husk on the crosshair and within 3.5 m dives in, swings once (it lands 13 ticks in, plus the weapon's ticks per swing) and comes back out 18 ticks later. It spends nothing. Within 1.8 m it swings without a dive, at most once every weapon's ticks per swing. A swing on an open seal lands twice its share; one on the body lands its share (×1.4 for the axe). Every swing that lands stops a fighting husk in its tracks for 12 ticks (28 for the greatsword); not one that is drawing breath, shrieking, charging or in the air.

FIRE still stabs or swings when no husk can be reached: it spends the normal attack cooldown, never spends a dive, and counts as a miss. Only a reachable husk starts a dive. A press while drawing, cooling down, reloading or shrieking keeps those rules.

In a camera hunt, a missed shot is ray-tested against the room. A measured wall or object takes a visible impact: a chipped bullet mark, a knife or blade cut, a spear puncture, a blunt crack, a bell ripple, a scorch from Cinder, frost from Rime or a dust ripple from Gale. Melee marks reach only 1.8 m. These are surface effects, kept for the hunt; they neither deform real objects nor change a combat result. No surface hit means no invented wall mark.

**Cadence.** Hits closer together than 78 ticks build cadence: +10% each, up to +50% (+100% with the Chain perk). A miss breaks it. A plain reload takes 84 ticks, so it breaks the cadence; an active reload does not.

### Fight

Each husk has four or five seals: on its brow, its chest, its flanks, its back. Up to two are open at a time, each for 2.5 to 4.3 seconds.

- A seal can only be hit from a side you can see. One well round the far side of the body (more than about 105° from the nearest point) is hidden, so a husk with its back to you shows different seals from one facing you. From any angle at least two are in view.
- A hit counts as on an open seal when the crosshair is within the seal cone of it: `3.2° + 0.55° × reach + weapon reach`, whatever is in hand. A hit on a seal shuts it for 44 ticks.
- The fight lasts 150 seconds. The clock stops while the husk lies broken, and again while an orb holds it: a capture must not time out. When the clock runs out the husk escapes.

What the screen shows:

- **The ring under the husk** says what it is about to do, and a caption under the crosshair says it in words. White: nothing is coming, attack. Amber: it is drawing breath or shrieking, hold still. Red: it is charging, swing. Blue: it is stunned and close, dive in. Gold: it is weak enough for an orb, or broken, throw one.
- **The crosshair** brightens when a press now would reach the husk, takes the husk's colour on an open seal, and flares when a hit lands.
- **Above FIRE** is what the weapon in hand has left: rounds, a reload with its active window marked in gold, dives, throws, or whether the husk is in reach.
- **ORB** shows the reaper's soul with a notch at what an orb costs, and says why it cannot be thrown when it cannot.
- During a shriek a meter shows how close the phone is to flinching, and while the husk lies broken another shows how long it stays down.

A press a moment early is not lost: FIRE pressed while the weapon in hand is busy waits up to 8 ticks for it and is played when it is ready. A press that would have to wait longer is dropped, and one that has waited is never played into a shriek. A press during a reload or a dive is played at once, since there its timing is the player's.

These are readings of the rules, not extra bonuses or penalties. Practice results show unbroken holds, parries and the longest run of hits. They offer the same stray again, with the same seed and room, so its rhythm can be learned without tracing the sigil again or returning to the sanctum. Practice earns nothing.

### Shriek

Every 6.5 to 10 seconds the husk stops where it is, turns on you and draws breath (54 ticks of warning, its mouth opening, its ring amber), then shrieks for 66 ticks.

- Move faster than your flinch speed, press FIRE, or let go of an orb, and you flinch: you lose one resolve and your cadence. The first 7 ticks are forgiven. Holding ORB down is not a flinch.
- Out of resolve, the husk escapes.
- Hold still through the whole shriek and every seal in view opens for 170 ticks, and your soul gains 10. It is facing you, so that is its front.
- Two or three seconds after a shriek it bolts again.

The warning is a darkening of the edges of the screen and a line of text. It is deliberately not a flash: see the safety notes below.

### Charge

A husk fights back. Every 8 to 14 seconds of the fight it turns on you and comes: a walker runs at you along the floor, a flier swoops down to eye level. It takes 66 ticks (1.1 s) to arrive, to within 0.9 m. Its ring turns red, and a ring on the screen closes on it as it comes.

- Swing the phone across it at 2.0 rad/s or faster and you turn it aside. That is a flick of the wrist: a third of a turn a second. A swing in the first 10 ticks does not count, since you were probably still turning to follow it.
- Turned aside, it stands stunned where it got to for 100 ticks, usually in arm's reach, with every seal in view open, and your soul gains 10. Its ring turns blue: the knife needs no dive, and every stab is critical. Then it bolts.
- Do nothing and it strikes: you lose one resolve and your cadence, the edges of the screen bruise, and it runs off. If it catches you mid-dive it strikes for double: two resolve. Out of resolve, the husk escapes.
- You can keep firing at it while it comes.

A shriek asks you to freeze and a charge asks you to swing, so the two are never close together: there are always at least 90 ticks between one ending and the other beginning. A husk in a rift does not charge.

### Arts

A reaper that has absorbed a husk carries something of it: an art, thrown rather than struck. Three are known, and each is carried by particular creatures.

| Art | What it is | Carried by |
| --- | --- | --- |
| Cinder | Fire, lobbed | Degenerate Ape Academy, Solarians |
| Rime | Ice, thrown straight | Pesky Penguins, Sollamas |
| Gale | Wind | Thugbirdz |

**Learning.** The first husk absorbed that carries an art teaches it, at rank one. Four of them make rank two, ten rank three. Nothing buys an art: what a reaper can throw is a record of what it has hunted, read from its codex, so it travels with an NFT reaper. The first art a reaper learns is made ready at once. In the armory a reaper may make any art it knows the one it throws.

**Throwing.** The art is the fourth weapon on the rail. With it in hand, FIRE throws it at the crosshair at once, while a throw is ready, and throws are at least 24 ticks apart. Drawing it takes 21 ticks, as for any weapon. FIRE in a shriek is a flinch and throws nothing.

A reaper starts the fight with one throw ready and holds at most two (three at rank three). Another is readied every 9 seconds (8 at rank two, 7 at rank three), and an unflinching hold readies one at once. An art thrown at a husk that has not noticed you goes unseen until it lands; then the fight begins, without the ambush.

| | Cinder | Rime | Gale |
| --- | --- | --- | --- |
| Flies at | 6 m/s, lobbed, to where it was thrown | 14 m/s, straight, following the husk | 12 m/s, following the husk |
| Catches the husk when | it bursts within 0.8 m of the husk's body | thrown at the husk, and nothing stands between | thrown within 22° of the husk |
| Cover | arcs over it | breaks on it | goes round it |
| On landing | ×2.0 / 2.5 / 3.0 of `H` | ×1.5 of `H` | ×0.8 / 1.0 / 1.2 of `H` |
| Then | it burns for 150 ticks, taking another ×1.5 / 2.0 / 2.5 of `H`, and bolts 20 ticks later, into the open | it is frozen for 150 / 190 / 230 ticks, every seal in view open; a charge stops dead; the next hit on a seal shatters the ice for ×2.0 / 2.25 / 2.5 | it is knocked off its ledge or out from behind its cover, a charge is turned aside, and it stands stunned 90 / 110 / 130 ticks with every seal in view open |

A husk that is burning, or that the wind has shaken (for 4 seconds after it comes round), will not run for cover.

A simulated careful harvester at level six takes a rare ape in a furnished kitchen about 5% sooner with Cinder at rank one, 8% sooner with Rime and 2% sooner with Gale, over ten hunts each. Drawing the art and drawing the gun again costs nearly as much as an art takes off the ward, so an art is worth throwing for what it does to the husk, not for its damage: Cinder and Gale bring a husk out of cover, Rime and Gale stop a charge.

### Capture

A husk is not cut down. It is weakened, and then caught in a soul orb.

**Soul.** Every hit fills the reaper's soul by 120 × the share of the husk's whole ward it took, so breaking a ward always pays for at least two orbs. An unflinching hold and a parry add 10 each. Soul starts each hunt at 0 and holds at most 100.

**When an orb can be thrown.** Once the husk's ward is down to 60% of what it started with (its ring turns gold) or it is broken, and the reaper has 40 soul, which is what an orb costs. One orb is in the air at a time. ORB says which of these is missing when one is.

**The throw.** Hold ORB and a ring shrinks from the edge of the screen onto the husk, in 72 ticks (1.2 s), then starts again. Let go to throw. Where the ring is when ORB is let go grades the throw: in its outer third a plain throw, in its middle third a great one, in its inner third an excellent one. A hold shorter than 6 ticks, or a throw with the crosshair off the husk, is plain. The orb strikes the husk if the crosshair is within 6° of its body; otherwise it misses and falls, and its soul is spent.

**The chance it holds** is `P = min(0.97, R × (3 − 2w) ÷ 3 × K × C + pity)`, where:

- `R` is the husk's rarity: common 0.8, uncommon 0.6, rare 0.4, epic 0.25.
- `w` is the share of its ward the husk has left, from 0 (broken) to 1. A broken husk is three times as easy to hold as a whole one.
- `K` is the grade: plain ×1, great ×1.3, excellent ×1.5.
- `C` is `0.85 + 0.05 × capture`, from the reaper's capture rating.
- `pity` is 0.08 for every orb this husk has already broken out of, up to 0.24.

**What happens**, counted in ticks from the moment ORB is let go:

| Ticks | |
| --- | --- |
| 0 to 27 | The orb is in the air. At 27 it strikes the husk, and the roll is drawn. |
| 27 to 48 | The husk is drawn into the orb as light. |
| 48 to 66 | The orb drops onto the surface under where the husk was. |
| 66, 114 and 168 | It pulses: the husk is fighting inside it. |
| 222 | After a held breath, the result, about 3.7 seconds after the throw. |

While the orb holds it, the husk can do nothing, nothing can be done to it, and the fight clock stops: a hunt cannot time out during a capture.

**The roll** is one number `u` between 0 and 1, from `SeededRandom(mixSeed([seed, 0x0B5, attempt]))`, where `seed` is the hunt's and `attempt` counts the orbs that have struck this husk before, from 0. The same hunt always rolls the same. If `u` is under `P`, the orb holds and the husk is captured. If not, it breaks on a pulse that tells the truth about how close it came: with `m = (u − P) ÷ (1 − P)`, under a third it breaks on the third pulse, under two thirds on the second, and otherwise on the first. A husk that breaks out gets 10% of its whole ward back (never above 60%), stands stunned for 48 ticks, and the fight goes on.

The roll comes from the hunt's seed, and the phone knows the seed. A modified client that knows it could work out every roll in advance and only let go of orbs that will hold. The server's replay cannot tell that apart from luck. The later hardening is an orb rolled by the keeper, from a seed the player never sees, as feeding is rolled now.

**The grade pays.** A captured husk pays experience and anima by its throw: plain ×1.0, great ×1.25, excellent ×1.5. The codex keeps the grade. Saves written before the orb keep the grades that pay the same: a ragged cut reads as plain, a clean as great and a perfect as excellent.

**Broken.** Take the whole ward and the husk collapses for `3.5 s + 0.5 s × capture`: the broken window. It cannot move, shriek or charge, nothing you fire does anything to it, and the fight clock stops. Throw an orb before the window ends. If it passes without one, the husk gathers itself, 30% of its ward back, and runs. The second window to pass is its escape (the fourth, on easy).

**In a rift** the ward is shared, and no orb can be thrown until it breaks. Then every reaper still standing has the room's window (8 seconds) to land one. Orbs cost no soul there and one may be thrown as soon as the last has fallen, and the first to strike the husk holds at once, with nothing rolled: landing it is that reaper's cut. A plain throw is a ragged cut, a great one a clean cut, an excellent one a perfect cut. A reaper whose window passes without a cut has made none.

### Testing difficulty

A build that is not a release has a "Testing difficulty" button on the screen before each hunt. It starts on easy, so that someone who has never played can bring a husk down and see a whole hunt. It changes what a hunt asks, never what it pays.

| | Easy | Normal | Hard |
| --- | --- | --- | --- |
| The husk's ward | ×0.35 | ×1 | ×1.3 |
| The seal cone and the pistol's aim assist | ×1.75 | ×1 | ×0.85 |
| How fast you may move in a shriek | ×3 | ×1 | ×0.8 |
| Flinches and blows you can take | +3 | +0 | +0 |
| The broken window | ×2 | ×1 | ×0.8 |
| How far off the husk an orb may be thrown | ×2 | ×1 | ×0.8 |
| Broken windows that may pass | 4 | 2 | 2 |
| How fast the husk moves | ×0.6 | ×1 | ×1.15 |
| How long the fight may run | 225 s | 150 s | 75 s |
| How long a charge takes to arrive | ×1.6 | ×1 | ×0.85 |
| The swing that turns a charge aside | ×0.6 | ×1 | ×1.15 |
| How near the ring must come to a sigil's light | ×1.75 | ×1 | ×0.85 |
| How long a caught light waits for the next | ×2 | ×1 | ×1 |

A simulated beginner (slow to turn and to react, every other shot wide, no active reloads, never drawing the knife, letting ORB go early) captured all 60 of its uncommon husks on easy and on normal, and 26 of 60 on hard. The difficulty is recorded with the hunt. The server replays at the recorded difficulty, and refuses anything but normal when it runs in production. A release build has no button and plays on normal.

### Feeding

Two reapers who capture the same husk equally well do not get exactly the same out of it. When a husk is absorbed, the keeper of the books rolls how well the reaper fed:

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
| Harvester | 3 | 5 | 3 | 2 | Cleave: a hit on a seal also catches every other open seal, for 40% each | Scythe |
| Ferryman | 3 | 2 | 3 | 5 | Steadfast: the first flinch of a hunt costs nothing | Lantern pole |
| Weigher | 4 | 2 | 4 | 3 | Measure: open seals are numbered in the order they opened, and a hit on the first lands 20% harder | Balance flail |
| Chooser | 5 | 4 | 2 | 2 | Chosen blow: the hit after an unflinching hold lands three times as hard | Spear |
| Hollow | 4 | 3 | 2 | 4 | Chain: cadence builds to +100% | Chain whip |
| Mourner | 1 | 4 | 5 | 3 | Toll: each unflinching hold adds 80 ticks to the broken window | Bell |

The perks work the same whichever weapon is in hand: a shot, a stab and a swing are each one hit. An art is not.

What the ratings do:

- Strike: the reaper's hit, `H = 4.0 × (0.85 + 0.1 × strike) × temper × level × sets`. Each weapon does a share of it: see [The crosshair and the four weapons](#the-crosshair-and-the-four-weapons).
- Reach: the seal cone, `3.2° + 0.55° × reach + weapon reach`, and the pistol's aim assist, `3.5° + 0.25° × (reach − 3)`.
- Capture: the broken window, `3.5 s + 0.5 s × capture`, and the orb's odds, ×`(0.85 + 0.05 × capture)`.
- Resilience: resolve is `2 + (resilience + 1) ÷ 2`, and flinch speed is `0.42 + 0.035 × resilience` rad/s.

A reaper also has a look the shuffle draws: three heads, three mantles, four robes and three colourways per class, plus stature, gauntness, gaze and tatter, an aura, a name, and an omen (none 80%, ashen 12%, gilded 6%, eclipsed 2%). Omens change colour only.

### Owning several reapers

The oathbound starter stays with its player. Online players may also import up to 32 verified reaper NFTs from the configured Metaplex Core collection. This bounds the ownership check; it is not a mint supply. Each reaper has its own experience, weapons, outfits, codex and hunt records. Anima, the simulated husk wallet and the vault belong to the account. Switching does not duplicate an absorbed husk or let it be bought again by the same account.

Finish loose hunts before selecting another reaper. Ownership is checked before online mutations. Selling a selected NFT returns its unfinished husks to the account's vault without rewards and selects the starter. Its earned progression remains attached to that NFT's server record for the next owner. Offline growth cannot be imported into that record.

Birth traits are immutable for owned reapers. The studio freely changes an unsaved specimen to demonstrate the possibilities; it does not change the player or mint an NFT. NFT reapers earn growth through replayed solo hunts. Rifts remain available only to the non-tradable starter until their results can be replayed.

### Levels

Level cap 30. Going from level `n` to the next costs `90 + 55(n − 1) + 6(n − 1)²` experience. Each level adds 7% of a level-one hit. Each completed set adds 3%.

### Birth gifts and the soul's memory

Every R1 reaper has one of its class's three gifts and one of four
temperaments. These are permanent birth properties, derived by `ReaperSoul`
from the stored genome. Colours, auras and rare omens do not change them.
There are 72 mechanical birth combinations. The [research and decision record](research/reaper-identity-and-growth.md)
explains the original lore and why these rules use actions the game already has.

| Temperament | Benefit | Cost |
| --- | --- | --- |
| Patient | Broken window +30 ticks (0.5 s) | Heavy swing +2 ticks |
| Keen | Seal cone and aim assist +0.6° | Broken window −24 ticks (0.4 s) |
| Stalwart | Resolve +1 | Hit ×0.95 |
| Hungry | Hit ×1.08 | Flinch speed ×0.92 |

Each reaper keeps three independent memories. A successful solo absorption
grants at most one mark in each, calculated from the replay:

- **Unseen:** the hunt began with an ambush.
- **Stillness:** the player completed at least one unflinching hold.
- **Rhythm:** at least five landed hits in a row within the cadence window.
  Free cadence granted by a gift does not count as landed hits. A real miss,
  flinch, charge hit or expired cadence breaks the landed-hit streak.

Escapes, abandoned hunts, practice and rifts grant no soul marks. Repeating
shriek holds in one encounter cannot farm extra marks. The memories, like XP,
travel with that reaper's canonical NFT record. Old saves retain their XP and
equipment, but start with zero memories rather than invented historical deeds.

| Awakening | Minimum level | Marks in the gift's path | Gift strength |
| --- | --- | --- | --- |
| Birth | 1 | 0 | 1 |
| First | 5 | 3 | 2 |
| Second | 12 | 12 | 3 |
| Third | 20 | 30 | 4 |

Both requirements must be met. Money, feeding luck or unrelated practice does
not substitute for the required marks. These are initial playtest thresholds.
The following `S` is the strength in that table, never a random roll.

| Class | Gift | Path | Actual effect |
| --- | --- | --- | --- |
| Harvester | The Last Furrow | Unseen | Ambush cleave's extra seals take `0.4 + 0.15S` of the primary blow. |
| Harvester | Aftermath | Stillness | After a hold, the next cleave leaves caught extra seals open for at least `170 + 24S` ticks, instead of shutting them. |
| Harvester | The Gathering | Rhythm | Every third cadence hit reopens its struck seal for `44 + 24S` ticks. |
| Ferryman | Safe Passage | Unseen | An ambush protects against one un-parried charge and stuns the husk for `30S` ticks. This automatic protection is not a player parry. |
| Ferryman | The Anchor | Stillness | A hold restores one lost resolve, at most `S` times per hunt; never above starting resolve. |
| Ferryman | Returning Light | Rhythm | Five cadence hits prepare one flinch forgiveness, at most `S` preparations per hunt. The class's first free flinch is used before this light. |
| Weigher | Foreknowledge | Unseen | The ambush's exposed seals stay open `30S` ticks longer. |
| Weigher | The Verdict | Stillness | A hold prepares the next in-order hit for ×`(1 + 0.25S)` damage. An out-of-order hit does not spend it. |
| Weigher | Equilibrium | Rhythm | Three in-order hits reopen all seals for `44 + 24S` ticks. An out-of-order hit resets this count. |
| Chooser | First Light | Unseen | The first hit of an ambush gains a further ×`(1 + 0.25S)`. |
| Chooser | Reprisal | Stillness | A player parry prepares Chosen Blow and a further ×`(1 + 0.25S)` on the next hit on a seal. |
| Chooser | The Last Word | Rhythm | At cadence ≥3, a hit on a seal while the ward is ≤35% of its starting value gains ×`(1 + 0.25S)`. |
| Hollow | Old Hunger | Unseen | An ambush starts cadence at `S`; the normal cadence timer still applies. |
| Hollow | The Echo | Stillness | A hold restores cadence from the shriek's warning plus `S`, capped at ten, and restarts its timer. |
| Hollow | Unbroken | Rhythm | At most `S` real misses preserve cadence per hunt; its existing timer continues. The actual landed-hit streak still breaks. |
| Mourner | Hush | Unseen | An ambush delays the first shriek by `30S` ticks. |
| Mourner | Keening | Stillness | A hold prepares the next hit on a seal for ×`(1 + 0.25S)` damage. |
| Mourner | The Refrain | Rhythm | Every third cadence hit adds `15S` ticks to the broken window, at most six times per hunt. |

Gifts act only at their stated trigger; they add no paid training, cooldown
skips or second spell bar. The studio's Soul view shows their exact current
and future effects, temperament costs, marks and original lore. Studio memory
buttons are explicitly illustrative and never write the saved player.

Combat log version **10** (or **11** with a room grid, **12** with a recorded moving eye) identifies these rules, including swings into empty space: it carries the room's cover in the plan, and each tick's crosshair, turn speed, FIRE, ORB and weapon slot. Older logs are refused with
an update message, never silently reinterpreted. Staging and production refuse
the simulated acquire, release, settlement and rift endpoints until real husk
acquisition, escrow and confirmed burning replace them. Practice remains
available. This is a launch guard, not a claim those transactions exist.

### Weapons and outfits

Any class can buy any weapon. Each kind has its own feel:

| Weapon | Hits per swing | Reach | Ticks per swing |
| --- | --- | --- | --- |
| Scythe | ×1.15 | +1.6° | 26 |
| Lantern pole | ×1.0 | +0.8° | 22 |
| Balance flail | ×1.0 | +0.4° | 24 |
| Spear | ×1.45 | +0° | 32 |
| Chain whip | ×0.8 | +0.6° | 16 |
| Bell | ×1.3 | +2.4° | 28 |
| Greatsword | ×1.85 | +1.2° | 40 |
| Axe | ×1.5 | +0.2° | 32 |

The greatsword and the axe are heavy, and no class starts with them. Each has one thing of its own:

- **Greatsword.** Every swing that lands stops the husk in its tracks for 28 ticks.
- **Axe.** A swing on the body, away from any seal, does 70% of a hit on a seal, not 50%.

The first temper of a heavy weapon costs 320 anima and needs level 3, for any class.

Each weapon has four tempers, and each outfit is one of the same four states. A temper changes the strike. An outfit changes only how the reaper looks.

| State | Weapon strike | Weapon level | Weapon price | Outfit level | Outfit price |
| --- | --- | --- | --- | --- | --- |
| Shrouded | ×1.0 | 1 | free for your class, 120 anima otherwise | 1 | owned |
| Sworn | ×1.3 | 4 | 220 | 5 | 300 |
| Scorched | ×1.7 | 9 | 640 | 12 | 900 |
| Hollowed | ×2.2 | 15 | 1500 | 20 | 2400 |

## Husks

A husk is what is left of an NFT. Every supported collection has a creature of its own, and each NFT dresses it: three colours, the colour of the light in its eyes, and at most one thing it wears. The one epic of each collection is gilded. An NFT's picture in the game is a portrait of its own creature.

Its ward, its worth and its sigil come from its rarity (see [Drawing it out](#drawing-it-out) for the sigils).

| Rarity | Ward | Experience | Anima | Simulated price |
| --- | --- | --- | --- | --- |
| Common | 280 | 40 | 16 | 0.02 SOL |
| Uncommon | 420 | 70 | 28 | 0.045 SOL |
| Rare | 760 | 120 | 48 | 0.11 SOL |
| Epic | 2800 | 210 | 85 | 0.26 SOL |

| Collection | Creature | Gets about | Stays | Ambles | Bolts |
| --- | --- | --- | --- | --- | --- |
| Degenerate Ape Academy | A heavy ape on its knuckles, 1.1 m tall | On the floor | 2.6 to 4.6 m away | 0.7 m/s | 2.6 m/s |
| Thugbirdz | A hunched bird in a visor | In the air, 0.15 m below the eye to 0.85 m above it | 2.2 to 3.6 m | 0.5 m/s | 3.2 m/s |
| Pesky Penguins | A squat penguin that waddles, 0.9 m tall | On the floor | 2.2 to 4.2 m | 0.45 m/s | 1.8 m/s |
| Sollamas | A long-necked llama under a striped blanket, 1.44 m tall | On the floor | 2.8 to 4.8 m | 0.6 m/s | 2.8 m/s |
| Solarians | A robot of mismatched parts on two legs, 1.12 m tall | On the floor | 2.4 to 4.4 m | 0.5 m/s | 2.0 m/s |

The body of a rift is not a creature: it is the reliquary stack, a tower of cubes 10 m tall and 17 m away, each face wearing one of the collection's portraits. It does not move. Four seals at the seams.

A level-one reaper takes a common husk in 22 to 31 seconds, depending on its class, and cannot take an epic one. A level-ten reaper with a scorched weapon takes an epic in 76 to 109 seconds. `cargo run --release -p reapers_rules --example pacing` prints the whole table for every class and level.

As it prints now:

```text
--- level 1, shrouded ---
harvester  common: 8/8 24s       uncommon: 8/8 33s     rare: 8/8 62s         epic: 0/8 --
ferryman   common: 8/8 28s       uncommon: 8/8 38s     rare: 8/8 72s         epic: 0/8 --
weigher    common: 8/8 22s       uncommon: 8/8 32s     rare: 8/8 60s         epic: 0/8 --
chooser    common: 8/8 22s       uncommon: 8/8 32s     rare: 8/8 59s         epic: 0/8 --
hollow     common: 8/8 24s       uncommon: 8/8 34s     rare: 8/8 64s         epic: 0/8 --
mourner    common: 8/8 31s       uncommon: 8/8 43s     rare: 8/8 82s         epic: 0/8 --
--- level 5, sworn ---
harvester  common: 8/8 16s       uncommon: 8/8 22s     rare: 8/8 41s         epic: 7/8 124s
ferryman   common: 8/8 19s       uncommon: 8/8 25s     rare: 8/8 47s         epic: 6/8 140s
weigher    common: 8/8 15s       uncommon: 8/8 21s     rare: 8/8 38s         epic: 7/8 113s
chooser    common: 8/8 16s       uncommon: 8/8 21s     rare: 8/8 39s         epic: 7/8 115s
hollow     common: 8/8 16s       uncommon: 8/8 22s     rare: 8/8 42s         epic: 7/8 125s
mourner    common: 8/8 20s       uncommon: 8/8 30s     rare: 8/8 53s         epic: 3/8 152s
--- level 10, scorched ---
harvester  common: 8/8 10s       uncommon: 8/8 15s     rare: 8/8 27s         epic: 8/8 82s
ferryman   common: 8/8 12s       uncommon: 8/8 18s     rare: 8/8 32s         epic: 8/8 96s
weigher    common: 8/8 10s       uncommon: 8/8 14s     rare: 8/8 25s         epic: 8/8 76s
chooser    common: 8/8 10s       uncommon: 8/8 15s     rare: 8/8 26s         epic: 8/8 82s
hollow     common: 8/8 10s       uncommon: 8/8 15s     rare: 8/8 28s         epic: 8/8 89s
mourner    common: 8/8 13s       uncommon: 8/8 20s     rare: 8/8 35s         epic: 8/8 109s
--- level 16, hollowed ---
harvester  common: 8/8 9s        uncommon: 8/8 11s     rare: 8/8 20s         epic: 8/8 54s
ferryman   common: 8/8 9s        uncommon: 8/8 12s     rare: 8/8 22s         epic: 8/8 63s
weigher    common: 8/8 9s        uncommon: 8/8 10s     rare: 8/8 19s         epic: 8/8 50s
chooser    common: 8/8 9s        uncommon: 8/8 11s     rare: 8/8 19s         epic: 7/8 50s
hollow     common: 8/8 9s        uncommon: 8/8 10s     rare: 8/8 19s         epic: 7/8 55s
mourner    common: 8/8 10s       uncommon: 8/8 13s     rare: 8/8 24s         epic: 8/8 70s
```

`cargo run --release -p reapers_rules --example rift_pacing` prints how rifts go for circles of different sizes and levels:

```text
--- level 1 ---
1 reaper  + 3 shades: sealed 8/8  98s, 275 xp
2 reapers + 2 shades: sealed 8/8  83s, 275 xp
4 reapers + 0 shades: sealed 8/8  64s, 267 xp
--- level 6 ---
1 reaper  + 3 shades: sealed 8/8  98s, 275 xp
2 reapers + 2 shades: sealed 8/8  83s, 275 xp
4 reapers + 0 shades: sealed 8/8  64s, 267 xp
--- level 12 ---
1 reaper  + 3 shades: sealed 8/8  98s, 275 xp
2 reapers + 2 shades: sealed 8/8  83s, 275 xp
4 reapers + 0 shades: sealed 8/8  64s, 267 xp
--- level 20 ---
1 reaper  + 3 shades: sealed 8/8  98s, 275 xp
2 reapers + 2 shades: sealed 8/8  83s, 275 xp
4 reapers + 0 shades: sealed 8/8  64s, 267 xp
one idle reaper + 3 shades: sealed 0/8
```

Why these five, and the hundred or so they were chosen from, is in [research/solana-collections.md](research/solana-collections.md).

## Rifts

A rift is a whole dead collection piled into one body. It takes a circle of four.

- **Where.** Each rift has a site: London Bridge (Degenerate Ape Academy), Shibuya Crossing (Thugbirdz), Third Mainland Bridge in Lagos (Sollamas), Brooklyn Bridge (Pesky Penguins), Golden Gate Bridge (Solarians). A rift can be entered from anywhere, and where its husk stands depends on where you are: see "Standing in the same place" below.
- **Who.** A rift opens only to a reaper who has absorbed at least one NFT from its collection. The way to a collection's raid runs through buying that collection.
- **Gathering.** Reapers join and stand ready. When everyone present is ready there is a three-second count. A rift waits at most 60 seconds. Seats nobody takes are filled by shades: bots one level below the circle's average, slower to react and wider of the mark than a careful person.
- **The ward.** It is sized to the circle: the sum of what each reaper's pistol takes off a second, firing at the body with plain reloads, × 2.5 for the seals, cadence and active reloads of a careful reaper, × 70 seconds. A rift is as hard for four beginners as for four veterans. What it asks for is company and care, not levels.
- **The fight.** Each reaper fights the husk through their own phone, with their own seals, exactly as in a hunt but with nothing to stalk. The husk stands 14 to 40 m away, beyond any dive, so in a rift only the pistol and the arts reach it. Every hit anyone lands comes off the one shared ward. The shrieks come at the same instant for everyone. Flinch away all your resolve and you are cast out.
- **The limit.** 120 seconds. Four careful reapers break the ward and seal the rift in about 64 seconds; one with three shades in about 98 seconds.
- **Sealing.** When the ward breaks everyone has 8 seconds to land an orb (see [Capture](#capture)): in a rift an orb costs nothing and always holds, and landing it is that reaper's cut. Half the reapers still standing, rounded up, must make their cut, and at least one cut must be a person's: shades can break a ward but only the living can shut a rift.
- **Pay.** 220 experience and 90 anima for a ragged cut and a fair share of the work. Your share of the ward scales that from 60% (did nothing) to 100% (a quarter of the ward or more). The cut scales it: a ragged cut (a plain throw) ×1.0, a clean one (a great throw) ×1.25, a perfect one (an excellent throw) ×1.5, no cut ×0.75, and cast out ×0.5. Leave early and you are paid nothing. A failed rift costs nothing.

A development demo build (`REAPERS_DEMO_RIFT`) waives only the collection gate so several people can meet one enemy before they own its dead. The fight, the ward, the shrieks, the cuts and the pay are the same rules above; each player is paid once from their own books. The flag is ignored by release builds, and staging and production servers refuse the demo endpoint.

### Standing in the same place

A rift's husk stands at one place on the earth, and everyone in the circle is pointed at it. Four people in a park all turn the same way.

- **The rift keeps the place.** Whoever hosts the rift (the server, for a circle of people) holds one latitude and longitude for it and sends it to everyone in the circle.
- **How the place is chosen.** While the circle gathers, each phone that knows where it is says so. The rift takes its place from the first one it hears: at the site itself if that reaper is within 250 m of it, and otherwise 25 m in front of them, along the way their phone was pointing. It does not move after that.
- **How a phone finds it.** Each phone takes its own position, works out the compass bearing from there to the rift, and uses its compass to turn that into a direction in its own view. The husk is drawn 14 to 40 m away, whatever the real distance, so that it fits the screen and its seals can be struck.
- **Which readings count.** A location must be accurate to 25 m or better, no older than 30 seconds, and no more than 5 seconds ahead of the keeper's clock. Coordinates must be finite and within the earth's bounds; compass bearings must be finite. An unusable reading leaves the husk dead ahead until a better one arrives.
- **Location sharing.** Only an open rift in the foreground listens for location. The phone shares at most one fresh reading every 5 seconds; the server stores the latest reading in the authenticated player's books, never a trail. Pausing or leaving sends a clear after any upload already in flight. If the clear cannot reach the server, its location read stops returning the reading after 30 seconds. Declining permission does not prevent playing.
- **Too far away.** A reaper more than 150 m from where the rift stands sees it dead ahead, as before. So does a phone that cannot say where it is, or has no compass.
- **During the fight** the husk stays where it was when the fight began. Stand still, as for any hunt.

How well this works is down to the phone. Out of doors a phone knows where it is to within 3 to 10 m, and at 25 m that is 7 to 20 degrees of bearing. A compass is good to about 10 degrees, and much worse near steel, cars or indoors. So people standing together face the same way, give or take; they do not all touch the same stone. Indoors, expect it to be rough.

**Tried against a running server** on 3 October 2026: a simulator and a second client signed in as guests, each absorbed a Degenerate Ape through the server (the hunt replayed and the receipt signed), entered London Bridge within the same minute, and were seated in one circle with two shades. The second client said where it stood; the rift took its place 25 m in front of it, and every frame to both carried that one place from the gathering through the fight, the capture and the seal. A Seeker also hunted and sealed a rift against the same server. What has not been tried is two phones that both know where they are and which way north lies, standing together: the Seeker is the only such phone here.

Compass north is used as it comes. True north differs from it by a few degrees in most places and by 13 in New York and San Francisco. For a rift planted in front of a reaper that makes no difference, since everyone uses the same compass north. For a rift at its real site it moves the husk by that angle for everyone alike.

## Music

Three recordings, each made by someone else and released for anyone to use. Nothing was composed or generated for the game.

| Where | Piece | By |
| --- | --- | --- |
| The sanctum and every page of the ledger | Ossuary 1 - A Beginning | Kevin MacLeod. CC BY 4.0. |
| A hunt | Penumbra | Kevin MacLeod. CC BY 4.0. |
| A rift | Oppressive Gloom | Kevin MacLeod. CC BY 4.0. |

Horror, not concert music: the sanctum is a slow drone with a pale melody over it, the hunt something in the shadows that cannot quite be heard, and a rift a dark march. Chosen on 19 October 2026 by the composer's own descriptions and their measured loudness; nobody has listened to them in the game yet.

A hunt never switches tune. It has one piece, and the hunt changes how that piece is heard: how loud, and how muffled, as if through the veil.

| Moment | Loudness | Veil |
| --- | --- | --- |
| The sigil, before a light has caught | 0.22 | 0.9 |
| The sigil, as its lights catch | rises to 0.30 | falls to 0.8 |
| Stalking, unseen | 0.30 | 0.8 |
| Stalking, steadied on the husk | up to 0.45 | down to 0.5 |
| The fight | 0.8 | clear |
| The husk stunned | 1.0 | clear |
| It draws breath | falls from 0.8 to 0.12 | rises to 0.9 |
| The shriek | 0.06 | full |
| It charges | 1.0 | clear |
| Ward broken | 0.45 | 0.45 |
| Absorbed | silence | |
| Escaped | 0.25 | 0.85 |

So the music is one more thing telling you what to do. It comes clear when you may strike, drains away to almost nothing for the moment you must hold still, swells when the husk comes at you, and is gone by the time the husk has come apart.

The veil is a low-pass filter: clear is everything up to 16 kHz, full is only what lies under 320 Hz. Music can be turned off on the colophon page, which also carries the credits two of the pieces require.

## Safety

The game asks people to hold a phone up outdoors, so:

- The recorded eye must stay within a 1.0 m horizontal circle of the stance and between 0.50 m and 2.20 m above the floor. The fight pauses outside that stance or when positional tracking is unreliable. The bound is provisional until tested on phones; it limits input, not proof that a sensor was really there.
- You only ever turn on the spot. Nothing in the game asks you to walk while looking at the screen, and every hunt opens with a line saying so.
- Nothing flashes. A strike lights the husk in its own colour rather than to white, and the shriek warning darkens the screen instead of flashing it.
- Everything the hunt needs is on screen as text as well as colour, and every target is at least 44 points across.

## Reading the environment

What the game does, on a phone that can see:

- It shows the real camera feed, drawn through the camera's own lens so that husks hold their place as you turn.
- It measures how far down the floor is, and husks that walk stand on it with a soft shadow under them.
- It finds what stands on the floor and how far away, how tall it is and whether its top is level, and husks go round it, leap onto it and hide behind it.
- It fades the husk when the room hides the middle of its body, using the same visibility answer as the fight.
- It only puts a husk where you traced its sigil.

On a phone that cannot see, the floor is taken to be 1.45 m down and nothing is known about the furniture.

What it still does not know anywhere:

- **Tracking movement.** Positional hunts keep the husk in the charted room and judge actions from the phone's recorded eye. The Seeker only reports rotation and uses the origin; the positional experience still needs validation on ARKit or ARCore hardware.
- **Anything it has not been shown.** The room is read once, before the hunt. Move a chair during it and the husk walks through the chair, and the chair's old place still hides it.
- **Behind what it could not see.** A husk hides behind something only where the camera saw floor or a wall beyond it. Behind a cupboard taller than your eye, nothing is known, so nothing hides there.
- **The ceiling.** A husk that flies keeps clear of walls and cupboards, but its band of air tops out 0.85 m above your eye whatever the ceiling's height.
- **What things are.** It knows how tall something is and whether its top is level. It does not know a sofa from a counter.

[environment.md](environment.md) says how the seeing works, how far it can be trusted, and what would come next.

## What is simulated, and what is not built

- **The NFTs.** The five collections are real ones from Solana's first wave. Nothing else about them is: no wallet, no mint, no chain, and the creatures are our own models. See [nfts.md](nfts.md).
- **Rifts are not fenced.** A rift knows where it stands and points everyone at it, but nothing stops a reaper in another city from joining the same circle, and nothing checks that a phone is where it says. Sharing one place to within centimetres needs a positioning service (ARCore's Geospatial API, or ARKit's), which the Seeker does not have.
- **Rift reports are trusted.** A lone hunt is replayed on the server before it pays. A rift takes each reaper's word for the damage they dealt.
- **Rifts have no cost and no cooldown.** That is why they pay like one epic husk and no better. Before launch they need a key, a cooldown or both.
- **Receipts are signed but go nowhere.** Nothing posts them to a chain yet.
- **Reaper against reaper** is not built.
- **Sound effects** are not built. Music and haptics are.
- **The numbers** were tuned against a simulated player, not people. Expect to move them after the first playtest on real phones.

### Combat audio

Accepted shots play recorded pistol reports even when they miss. Reloads,
blade cuts, heavy strikes and parries have separate cues; cooldown presses
remain quiet. Each vessel has a distinct attack tell before its charge or
shriek, and a landed charge or unforgiven flinch plays the reaper's hurt cue
as the rules remove resolve. Encounter seeds already vary attack timing;
ground creatures rush across the floor, birds swoop and the stationary rift
stack shrieks. Music and effects can be switched independently on the colophon.
