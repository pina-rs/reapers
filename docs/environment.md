# Reading the environment

The aim: a husk should live in the room you are standing in. One that walks should run across your floor and round your furniture, one that flies should stay in your air, and it should only show itself once the phone has worked out where it can go.

This page says how the game does that, how far it can be trusted, what it still cannot do, and what would come next.

## What is built

On a phone with a camera and motion sensors, the hunt now sees the room:

1. A hunt opens with "Something is nearby", and the husk's sigil hangs in the air around you: a few lights to rest the ring on, one after another.
2. The lights are where the camera needs to look: over the floor for a husk that walks, at head height for one that flies, spread across the arc the husk will keep to. Holding still on each one is what lets the camera take a sharp picture there. About twice a second, while the phone is steady, it takes one and works out, on the phone, how far away everything in it is.
3. From those pictures it finds the floor, measures how far below the phone it is, and maps how far the floor is clear in each direction. A small dial takes the shape of the room as it goes.
4. When the last light catches, the husk gathers into view inside the sigil's arc. No room is too small: where the floor is tight the husk stands closer, down to 0.8 m.
5. A walker stands on the measured floor and runs round whatever is in the way. A flier keeps clear of walls and cupboards.

Nothing leaves the phone. What the hunt keeps of your room is about 80 bytes: the height of your eye and one distance for each of 72 directions.

Three things were decided on 2 October 2026 and shape all of this:

- **A depth model is bundled with the game.**
- **The Seeker comes first.** It has no ARCore, so the camera picture is all there is to go on.
- **The floor is measured, not assumed.** The game no longer takes you to be standing.

## How it works

### The model

- **What:** Depth Anything V2, the small model, in the version its authors fine-tuned on indoor scenes to answer in metres. It looks at one ordinary picture and says how far away each part of it is.
- **Where from:** the authors' own repository, `depth-anything/Depth-Anything-V2-Metric-Indoor-Small-hf`, at a pinned revision.
- **Licence:** Apache-2.0. The small model is the only size released under it; the larger ones are non-commercial. The licence text ships with the model in `reapers_flutter/assets/depth/`.
- **Size:** 27 MB in the build, after its weights are stored as 8-bit numbers. At full precision it is 99 MB.
- **Speed on the Seeker:** about half a second a picture on two cores, leaving the rest to draw the game. The full-precision model took 0.87 s.
- **How it is built:** `dart tool/build_depth_model.dart`. The conversion script is Python, because the model's tooling is. The result is committed, so nobody needs to run it to build the game.

It runs through ONNX Runtime (the `flutter_onnxruntime` plugin), off the thread the game draws on.

### The lens

The phone is asked for its camera's focal length and sensor size. The Seeker answers 5.24 mm on a 7.68 × 5.76 mm sensor, which is a view 72.5° tall when held upright. The game had been drawing husks through an assumed 62° lens, so they slid against the room as the phone turned. They are now drawn through the camera's own.

Only Android answers so far. On anything else the 62° guess stands.

### The floor

Each picture becomes a cloud of points around you, and gravity says which way is down. The floor is the lowest level surface that is a fair share of the level surface in the picture: lower than the table tops, and bigger than the stray points a depth model always leaves below it.

Every other length in that picture is then measured in multiples of its own floor depth. Only the floor depth, taken across all the pictures, is trusted in metres. That matters for two reasons:

- A depth model reads a whole picture long or short together. Measuring against each picture's own floor keeps one picture's furniture in step with the next one's.
- It puts a husk's feet on the floor the camera shows even when the model's metres are wrong. If everything reads 20% long, the floor is 20% lower and the sofa 20% farther, and a husk standing in front of that sofa covers exactly the same patch of the picture. Only its size is out, by the same 20%.

Once three pictures agree where the floor is, a picture that puts it more than a quarter away from them is set aside. That is what a picture of a table top with no floor in it looks like.

### What stands in the way

Around you is a map of 72 slices by 90 rings. A point on the floor marks its cell clear. A point higher than the floor and lower than the husk's head marks it taken. A slice is clear out to its first taken ring, or to the end of the floor that was seen.

A husk keeps half its own width back from that, and the room it has opens out gradually to either side of an obstacle, so that it curves round the front of a sofa instead of swerving at its corner.

For a husk that flies, what matters is whatever stands up into its band of air. Floors and ceilings are left out of that: they are not walls.

## How far to trust it

I measured what I could without being able to pick the phone up.

**Against rooms of known depth** (nine pictures from the NYU Depth v2 validation set, which come with depth measured by a sensor):

| Picture shown to the model | It reads distances | Shape still wrong after correcting for that |
| --- | --- | --- |
| 518 pixels on the short side (what it was trained at) | 3% long | 9% |
| 392 | 18% long | 9% |
| 308 | 33% long | 9% |
| 252 (what the game uses) | 45% long | 10% |
| 196 | 56% long | 12% |

- The smaller the picture, the longer it reads, but steadily: from one room to the next the factor varies by about 7% at every size. So it is divided out, by a curve fitted to this table and the camera's focal length. On the Seeker that is a factor of 1.39.
- Storing the weights as 8-bit numbers changed the answers by 1 to 2% and the accuracy not at all.
- With that correction, on nine upright crops of those pictures, the model read between 13% short and 14% long.

**The floor-finder on real pictures.** Run on the same nine pictures, which are cluttered dining and sitting rooms, the map found the floor within 17% of where the measured depth put it in eight. In the ninth, a table fills the view with no floor showing, and the table was taken for the floor; that is the case the "set aside" rule above is for. Where the floor was clear in the measured depth it was clear in the model's, and the chairs and tables were in the same places.

**Against rooms drawn by hand**, where the right answer is known exactly: 17 tests in `reapers_flutter/test/hunt/depth/room_map_test.dart` cover standing and sitting, a table that is not the floor, a sofa that blocks, a sofa with room in front, depth that is noisy, depth that is wrong by a fifth, and pictures that disagree with each other about scale.

**On the Seeker itself:** the model loads and runs, the lens is read, and the scan screen comes up and gives the right advice. The phone was lying flat on a desk the whole time, so **I have not seen it read a real room**. That is the first thing to try, and the things most likely to need adjusting are listed below.

Put together: expect the floor to be found within 10 to 15% of its real depth, and a husk's feet to sit on the floor in the picture regardless. Expect it to be fooled sometimes.

## What to check on the first real run

Debug builds print a line for every picture: whether it had floor in it, where the floor was found, how much has been read. The scan screen shows the floor depth as it is found.

- **Is the floor depth about right?** Measure from the phone to the floor. If it is consistently out by the same factor, the correction curve needs moving for this camera.
- **Do husks stand on the floor or hover?** Hovering or sinking by the same amount everywhere means the pictures are being matched to the wrong direction, which is a bug in how frames are turned upright or timed, not in the model.
- **Does the dial look like the room?** Red spokes should be where the furniture is.
- **Does the scan finish?** It now ends when the sigil is traced, whatever has been read. The first real run, before the sigil, did not finish: see below.

### What the first real run showed

On 2 October 2026 the game was scanned in a real room for the first time, on the Seeker, and its log was read as it ran.

- **The floor was found**, 1.40 m below the phone in the first pictures, which is a plausible standing height. Nobody measured it.
- **The scan never finished.** After thirty pictures it had read 80° and found none of it to have room for the husk. The rule then was that a husk needed 1.2 m of clear floor plus half its own width, all along an arc of 115°, or it would not appear. A furnished room does not have that. The rule is gone: a slice is read once a picture has shown it, and a husk in a tight place stands closer.
- **The floor moved**, from 1.40 m to 0.55 m, over the run. Either the phone was lowered or it was shown a table or a bed: the floor is the lowest level surface in view, so a husk will stand on a table if that is all it is shown. Pictures that disagree with the rest by more than a quarter are set aside, which slowed the scan while the two disagreed.

What the log cannot say is whether the map matched the room: whether the directions it called tight had furniture in them. That needs someone looking at the dial and at the room.

On 3 October 2026 the scan was reported to work, with a request for something clearer to do than sweeping: a pattern to trace, harder for harder husks. The scan is now the husk's sigil ([gameplay.md](gameplay.md#drawing-it-out)). Its lights are placed so that tracing it does what the sweep was for: the phone looks over the floor across the whole arc, and holds still at each light long enough for a sharp picture.

`--dart-define=REAPERS_DEPTH_PROBE=true` opens a bench instead of the game: the camera, the model's depth picture beside it, and the timings.

## What it cannot do

- **Follow you walking.** It knows which way the phone points, not where it is. Take a step and the husks come with you, and the map of the room is now a step out.
- **Hide a husk behind anything.** The depth pictures could do this, at the cost of running the model during the fight.
- **Notice a change.** The room is read once, before the hunt.
- **Know the ceiling.** A flier's air is capped by its own rules, not by the room.
- **Tell a sofa from a wall.** It knows something is there. Nothing can duck behind it or jump onto it.
- **See in the dark, or through glass and mirrors.** No depth model does.
- **Run on an iPhone yet.** Everything is written to, but the lens is not read there and none of it has been tried.

## What would come next

In the order I would do them:

1. **Try it in real rooms and tune it.** Everything above was tuned on nine photographs and rooms I drew.
2. **Let a walker use a table.** The map already finds level surfaces above the floor. A husk that jumps onto the coffee table is a rule and an animation away, and it is the thing here most worth seeing.
3. **Hide husks behind furniture.** Run the model a few times a second during the fight and draw its depth into the scene.
4. **The same on iPhone**, through ARKit where there is a depth sensor: real surfaces, and husks that stay put when you walk.
5. **Naming things, on the server.** You said frames may be sent if it makes things better, and asked that descriptions be sent in place of pictures. The plan is already that description, and already travels with every hunt. What a server could add is meaning: which obstacle is a sofa to hide behind, which is a table to stand on. That needs a few small pictures (three at 320 pixels is about 40 KB), a vision model, and a key for it. Nothing here sends a picture anywhere, and I would want a consent screen and a rule about not keeping them before anything did.

## The Seeker has no ARCore

For the record, since it is why all of this is done from the camera picture:

- Google Play Services for AR is not installed on the Seeker, and the phone does not report the `android.hardware.camera.ar` feature that certified devices do. Both checked over `adb` on 1 October 2026.
- No Solana Mobile device is on [Google's list of ARCore supported devices](https://developers.google.com/ar/devices).
