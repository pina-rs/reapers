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
6. A husk uses the furniture: it leaps onto a counter or a table, and ducks behind one where the camera saw room beyond it. Whatever stands in front of it hides it on screen.

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

### What it is, and what is behind it

Every picture that has a floor in it is also charted for furniture, whatever the husk: in each cell of the same map, how many points stand above the floor (no higher than the eye: anything taller is a wall), how high they are, in bins of 4% of the floor depth, and which of them lie on a level surface. For each slice the map then finds:

- **Its front**, where the way stops.
- **How far back it runs**, for as long as there is something there, over a gap of one ring at most.
- **Its top**: the highest bin with three points in it. That is the cover.
- **Whether the top is a ledge**: at least 12 level points around the top, and at least 35% of the points there.
- **Room behind it**: the next thing standing beyond it, or floor seen beyond it. A husk hides there only where the gap is wider than the husk. Where nothing was seen behind (a cupboard taller than the eye), nothing hides there.

For a husk that flies, only a ledge matters, and its reach is brought in to the ledge's front, so that it can come down on it.

The plan carries all of this as three more numbers a slice: the cover's height in centimetres, where to hide behind it in decimetres, and one bit for a ledge. It is 225 bytes, up from 72.

### Hiding the husk

The same pictures also make an `OcclusionMap`: for every quarter-degree of direction all the way round, from 72° below the horizon to 48° above, how far away the nearest thing standing in the room is. The floor is left out, so it never hides a husk standing on it. Each picture is scaled to the floor the hunt stands on, as the map is.

The phone only ever turns where the player stands, so a picture taken while the sigil was traced still lines up with the view during the fight. Nothing is read again. The map becomes an image, and a fragment shader (`assets/shaders/occlusion.frag`) masks the 3D scene with it: for each pixel it finds where the ray enters the husk's body (taken as an ellipsoid), and cuts the husk away where the room's nearest thing along that ray is nearer, fading over 12 cm either side. The camera's own picture of the counter shows through.

Tested against drawn rooms in `reapers_flutter/test/hunt/depth/room_cover_test.dart` and `occlusion_map_test.dart`, including a model that reads everything 25% long. A room drawn by hand can also be played in on a simulator: `--dart-define=REAPERS_DRAWN_ROOM=true` reads a kitchen counter as if a camera saw it and draws it where the camera's picture would be.

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
- **Notice a change.** The room is read once, before the hunt. Move a chair and the husk walks through it, and the chair's old place still hides it.
- **Hide a husk exactly.** The cut follows the room as the depth model saw it, a quarter-degree at a time, and the husk as an ellipsoid. Expect its edge to be soft and a few centimetres out, more so far away.
- **Know the ceiling.** A flier's air is capped by its own rules, not by the room.
- **Tell a sofa from a counter.** It knows how tall something is and whether its top is level, not what it is.
- **See in the dark, or through glass and mirrors.** No depth model does.
- **Run on an iPhone yet.** Everything is written to, but the lens is not read there and none of it has been tried.

## What would come next

In the order I would do them:

1. **Try it in real rooms and tune it.** Everything above was tuned on nine photographs and rooms I drew.
2. **Try cover and ledges in real rooms.** Built on 19 October 2026 and tried only in rooms drawn by hand: whether real counters read as ledges, and whether the cut lines up with the real edge, needs the Seeker in a kitchen.
3. **Read the room again during the fight**, a picture every few seconds, so a chair that moves stops hiding the husk.
4. **The same on iPhone**, through ARKit where there is a depth sensor: real surfaces, and husks that stay put when you walk.
5. **Naming things, on the server.** You said frames may be sent if it makes things better, and asked that descriptions be sent in place of pictures. The plan is already that description, and already travels with every hunt. What a server could add is meaning: which obstacle is a sofa to hide behind, which is a table to stand on. That needs a few small pictures (three at 320 pixels is about 40 KB), a vision model, and a key for it. Nothing here sends a picture anywhere, and I would want a consent screen and a rule about not keeping them before anything did.

## The Seeker's GPU

On 3 October 2026, driving the game on the Seeker against a running server, the text started drawing with letters missing and NFT pictures came out as noise, and stayed that way until the game was restarted. The phone's log said why: Impeller's Vulkan backend had waited too long on the GPU, and MediaTek's driver had declared the GPU lost and reset it. Anything uploaded before the reset (the atlas the letters are drawn from, the pictures) was broken after it, and the driver went on timing out every ten seconds for as long as the game ran, taking the phone's own status bar with it.

Three things set it off, found one at a time by opening and leaving hunts while reading the log:

- **Baking the sky's lighting.** `VeilLook` baked the same night-sky environment map afresh for every scene: about 0.4 s of GPU work each time a hunt or the sanctum opened. It is now baked once, at a quarter of flutter_scene's default size, and shared.
- **Drawing at the screen's full density.** The Seeker's screen is three physical pixels to the point, and its GPU (a two-core Mali-G615) drew the lit, bloomed sanctum at about 30 frames a second. The 3D is now drawn at no more than two pixels to the point, which is as sharp at arm's length; the sanctum runs at about 45.
- **Closing the camera.** Even with both of those fixed, the reset came a tenth of a second after the camera hardware closed, every few hunts. That is between the camera plugin, the engine and the driver, not the game.

So Android draws through Impeller's OpenGL ES backend instead (`io.flutter.embedding.android.ImpellerBackend` in the manifest). There, the same moments are at worst a slow frame: eight hunts opened and closed and a minute idle gave no resets, no timeouts and no stalls at all. iPhones draw through Metal and are untouched.

What OpenGL ES costs: Flutter's own reports describe [OpenGL ES freezing some MediaTek/Mali phones](https://github.com/flutter/flutter/issues/193428) when the app goes to the background while a picture is being read back from the GPU, which flutter_scene does when it loads a texture. Nothing like it happened here. If it does, the next thing to try is Vulkan again with the camera kept open between hunts.

## The Seeker has no ARCore

For the record, since it is why all of this is done from the camera picture:

- Google Play Services for AR is not installed on the Seeker, and the phone does not report the `android.hardware.camera.ar` feature that certified devices do. Both checked over `adb` on 1 October 2026.
- No Solana Mobile device is on [Google's list of ARCore supported devices](https://developers.google.com/ar/devices).
