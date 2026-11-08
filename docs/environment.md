# Reading the environment

The aim: a husk should live in the room you are standing in. One that walks should run across your floor and round your furniture, one that flies should stay in your air, and it should only show itself once the phone has worked out where it can go.

This page says how the game does that, where the depth comes from, what was learned doing it from the camera alone, what it still cannot do, and what would come next.

## In the leptoskit game

The Rust game (`crates/reapers_app`) reads the room differently from the Flutter one, though it keeps every number of the map itself. There is no depth model in it. Depth now comes from the phone's own AR, through `leptoskit_xr`:

1. **Following the phone.** A hunt asks the platform for `PhoneAr` (ARKit, ARCore) with planes and depth, and falls back to `Orientation` (the rotation sensor) where there is no AR, and to a dragged finger where there is neither (`hunt/pose.rs`, `hunt/xr.rs`). The browser asks only for `Orientation`: its `immersive-ar` would take the page away from the game's own canvas.
2. **Depth without the picture.** The game's Wasm never sees the platform's depth picture; it can only cast a ray and be told how far along it the depth was. So, about once a second while the sigil is traced and the phone is steady, it casts a grid of 12 by 16 rays through the view against the platform's depth, and `reapers_world::DepthFrame::from_rays` turns their distances along each ray into a depth picture (`RayGrid` says what that many rays can and cannot see).
3. **The floor is measured, not found.** The platform's planes give the floor: the lowest plane facing up, or one it calls the floor. Its height under the phone goes to `RoomMap::measure_floor`, and every picture is then scaled to it. The platform's metres are real, unlike the model's, so nothing is divided out.
4. **The rest is unchanged.** `RoomMap`, `OcclusionMap`, the cover, the ledges and `sigil_plan` are the Flutter arithmetic, ported to `crates/reapers_world` with no camera, sensor or clock in it, and tested there against the same rooms drawn by hand.
5. **Hiding the husk.** The Flutter game cut the husk away pixel by pixel with a shader. `leptoskit_3d` draws only its own materials, so the game fades the whole husk when the occlusion map says its middle is behind something. Where the host composites the scene over the camera with the platform's depth (`leptoskit_xr::host::scene3d::draw`), real depth hides it exactly; see below for where that is not yet the case.

What runs where:

| Where | Pose | Room read | Behind the husk |
| --- | --- | --- | --- |
| iPhone with LiDAR (ARKit) | 6DoF, platform | yes: depth rays and planes | the veil (see below) |
| Android with ARCore and its Depth API | 6DoF, platform | yes | the veil |
| Android without ARCore (the Seeker) | rotation sensor | no: the sigil's arc alone | the veil |
| Browser on a phone | `deviceorientation` | no | the veil |
| Desktop browser, simulator | a dragged finger | no, or the drawn room | the veil |

**Not yet on a device.** Nothing above has run on a phone: there was none to hand when it was written. Two things are known gaps:

- **The camera's picture.** The generated iOS and Android hosts draw a `Scene3d` view over the app's own views and do not yet hand it the XR session's camera picture and depth (`Backdrop::Camera` draws as transparent there). Until a host does, the hunt is fought over the veil, as on a simulator, with the husk placed in the real room by the platform's tracking. `leptoskit_xr::host::scene3d::draw` is the piece that would draw it.
- **The Seeker.** It has no ARCore, and the game no longer carries a depth model, so on the Seeker the room is not read at all and a husk keeps to the sigil's arc in the open. Putting the model back would mean running it in a host plugin, where the camera's picture is.

`REAPERS_DRAWN_ROOM=true` (debug builds) plays every hunt in the drawn kitchen, read as if the platform had seen it, with its edges drawn where the camera's picture would be. `REAPERS_DEPTH_PROBE=true` opens the depth bench: what the platform reports, and the last grid of rays as a picture, nearest brightest, with how long the look took.

The sections below describe the Flutter game, which read the room from the camera's picture with a depth model. Its measurements are kept because they are what the map's numbers were tuned on.

## What is built (Flutter)

On a phone with a camera and motion sensors, the Flutter hunt saw the room:

1. A hunt opens with "Something is nearby", and the husk's sigil hangs in the air around you: a few lights to rest the ring on, one after another.
2. The lights are where the camera needs to look: over the floor for a husk that walks, at head height for one that flies, spread across the arc the husk will keep to. Holding still on each one gives the phone a steady look there.
3. From what it sees, the phone finds the floor, measures how far below the phone it is, and maps how far the floor is clear in each direction.
4. When the last light catches, the husk gathers into view inside the sigil's arc. No room is too small: where the floor is tight the husk stands closer, down to 0.8 m.
5. A walker stands on the measured floor and runs round whatever is in the way. A flier keeps clear of walls and cupboards.
6. A husk uses the furniture: it leaps onto a counter or a table, and ducks behind one where there is room beyond it. Whatever stands in front of it hides it on screen.

Nothing leaves the phone but the plan. What the hunt keeps of your room is a few hundred bytes: the height of your eye, one distance for each of 72 directions, and for each, the height of what stands there, where to hide behind it, and whether its top is a ledge.

Three things were decided on 2 October 2026 and shape all of this:

- **A depth model is bundled with the game.**
- **The Seeker comes first.** It has no ARCore, so the camera picture is all there is to go on.
- **The floor is measured, not assumed.** The game no longer takes you to be standing.

## How it works (Flutter)

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

Every view that has a floor in it is also charted for furniture: in each cell of the same map, how many points stand above the floor (no higher than the eye: anything taller is a wall), how high they are, in bins of 4% of the floor depth, and which of them lie on a level surface. For each slice the map then finds:

- **Its front**, where the way stops.
- **How far back it runs**, for as long as there is something there, over a gap of one ring at most.
- **Its top**: the highest bin with three points in it. That is the cover.
- **Whether the top is a ledge**: at least 12 level points around the top, and at least 35% of the points there.
- **Room behind it**: the next thing standing beyond it, or floor seen beyond it. A husk hides there only where the gap is wider than the husk. Where nothing was seen behind (a cupboard taller than the eye), nothing hides there.

For a husk that flies, only a ledge matters, and its reach is brought in to the ledge's front, so that it can come down on it.

The plan carries all of this as three more numbers a slice: the cover's height in centimetres, where to hide behind it in decimetres, and one bit for a ledge. It is 225 bytes, up from 72. `HuntPlan` encodes it and `tests/room_cover.rs` in `reapers_rules` checks how husks use it.

### Hiding the husk

The first game hid a husk behind the room with an occlusion map: for every quarter-degree of direction all the way round, from 72° below the horizon to 48° above, how far away the nearest thing standing in the room was, with the floor left out so it never hid a husk standing on it. A fragment shader found where each pixel's ray entered the husk's body (taken as an ellipsoid), and cut the husk away where the room's nearest thing along that ray was nearer, fading over 12 cm either side. The camera's own picture of the counter showed through.

Platform depth gives this for free and better: ARCore and ARKit both hand over a depth image per frame, aligned with the camera, which `leptoskit_3d` can test each pixel of the husk against as it is drawn. It follows the phone as it moves, where the map was taken once from where the player stood.

## What reading rooms from the camera alone taught

From 1 October to 7 November 2026 the game read rooms with no AR framework at all, because the phone it was built for, the Seeker, has none. A depth model looked at single camera pictures and said how far away each part was; the room was put together from those. These are its findings.

### The lens

The phone was asked for its camera's focal length and sensor size. The Seeker answers 5.24 mm on a 7.68 × 5.76 mm sensor, which is a view 72.5° tall when held upright. The game had been drawing husks through an assumed 62° lens, so they slid against the room as the phone turned. They were then drawn through the camera's own. Whatever draws the camera hunt must take the field of view from the camera, never assume one; ARCore and ARKit both report it.

### The floor

Each picture became a cloud of points around the player, and gravity said which way was down. The floor was the lowest level surface that was a fair share of the level surface in the picture: lower than the table tops, and bigger than the stray points a depth model always leaves below it.

Every other length in that picture was then measured in multiples of its own floor depth. Only the floor depth, taken across all the pictures, was trusted in metres. That mattered for two reasons:

- A depth model reads a whole picture long or short together. Measuring against each picture's own floor kept one picture's furniture in step with the next one's.
- It put a husk's feet on the floor the camera showed even when the model's metres were wrong. If everything reads 20% long, the floor is 20% lower and the sofa 20% farther, and a husk standing in front of that sofa covers exactly the same patch of the picture. Only its size is out, by the same 20%.

Once three pictures agreed where the floor was, a picture that put it more than a quarter away from them was set aside. That is what a picture of a table top with no floor in it looks like.

### How far to trust a depth model

**Against rooms of known depth** (nine pictures from the NYU Depth v2 validation set, which come with depth measured by a sensor), Depth Anything V2 small, metric indoor:

| Picture shown to the model | It reads distances | Shape still wrong after correcting for that |
| --- | --- | --- |
| 518 pixels on the short side (what it was trained at) | 3% long | 9% |
| 392 | 18% long | 9% |
| 308 | 33% long | 9% |
| 252 (what the game used) | 45% long | 10% |
| 196 | 56% long | 12% |

- The smaller the picture, the longer it reads, but steadily: from one room to the next the factor varies by about 7% at every size. So it was divided out, by a curve fitted to this table and the camera's focal length. On the Seeker that is a factor of 1.39.
- Storing the weights as 8-bit numbers changed the answers by 1 to 2% and the accuracy not at all. That took the model from 99 MB to 27 MB.
- With that correction, on nine upright crops of those pictures, the model read between 13% short and 14% long.
- On the Seeker it took about half a second a picture on two cores, leaving the rest to draw the game. The full-precision model took 0.87 s.

**The floor-finder on real pictures.** Run on the same nine pictures, which are cluttered dining and sitting rooms, the map found the floor within 17% of where the measured depth put it in eight. In the ninth, a table fills the view with no floor showing, and the table was taken for the floor; that is the case the "set aside" rule above is for. Where the floor was clear in the measured depth it was clear in the model's, and the chairs and tables were in the same places.

**Against rooms drawn by hand**, where the right answer is known exactly, 17 tests covered standing and sitting, a table that is not the floor, a sofa that blocks, a sofa with room in front, depth that is noisy, depth that is wrong by a fifth, and pictures that disagree with each other about scale.

Put together: the floor was found within 10 to 15% of its real depth, and a husk's feet sat on the floor in the picture regardless. It was fooled sometimes.

### What the first real run showed

On 2 October 2026 the game was scanned in a real room for the first time, on the Seeker, and its log was read as it ran.

- **The floor was found**, 1.40 m below the phone in the first pictures, which is a plausible standing height. Nobody measured it.
- **The scan never finished.** After thirty pictures it had read 80° and found none of it to have room for the husk. The rule then was that a husk needed 1.2 m of clear floor plus half its own width, all along an arc of 115°, or it would not appear. A furnished room does not have that. The rule is gone: a slice is read once it has been seen, and a husk in a tight place stands closer.
- **The floor moved**, from 1.40 m to 0.55 m, over the run. Either the phone was lowered or it was shown a table or a bed: the floor is the lowest level surface in view, so a husk will stand on a table if that is all it is shown. Pictures that disagree with the rest by more than a quarter are set aside, which slowed the scan while the two disagreed.

What the log could not say is whether the map matched the room: whether the directions it called tight had furniture in them. That needed someone looking at the room.

On 3 October 2026 the scan was reported to work, with a request for something clearer to do than sweeping: a pattern to trace, harder for harder husks. The scan became the husk's sigil ([gameplay.md](gameplay.md#drawing-it-out)). Its lights are placed so that tracing it does what the sweep was for: the phone looks over the floor across the whole arc, and holds still at each light long enough for a sharp look.

## What it cannot do

- **Read the room on a phone without platform depth.** The Seeker, iPhones without LiDAR and the browser play in the open.
- **Notice a change.** The plan is made once, before the hunt. Move a chair and the husk walks through it.
- **Know the ceiling.** A flier's air is capped by its own rules, not by the room.
- **Tell a sofa from a counter.** It knows how tall something is and whether its top is level, not what it is.
- **See in the dark, or through glass and mirrors.** Neither depth sensors nor depth models do.

## What would come next

In the order I would do them:

1. **The camera hunt on `leptoskit_xr`**, with the plan made from platform depth, and the husk hidden by the depth image as it is drawn.
2. **Try it in real rooms and tune it.** Cover and ledges have only been tried in rooms drawn by hand: whether real counters read as ledges, and whether the cut lines up with the real edge, needs a phone in a kitchen.
3. **Read the room again during the fight**, so a chair that moves stops hiding the husk. Platform depth arrives every frame, so this is a matter of rebuilding the plan, not of taking more pictures.
4. **A fallback for phones without platform depth**, if the Seeker is to read rooms again: a depth model run through `leptoskit_xr`, starting from the measurements above.
5. **Naming things, on the server.** It has been said that frames may be sent if it makes things better, and asked that descriptions be sent in place of pictures. The plan is already that description, and already travels with every hunt. What a server could add is meaning: which obstacle is a sofa to hide behind, which is a table to stand on. That needs a few small pictures, a vision model, and a key for it. Nothing here sends a picture anywhere, and I would want a consent screen and a rule about not keeping them before anything did.

## The Seeker's GPU

`leptoskit_3d` makes the same choice for the Rust game: on Android it draws with OpenGL ES unless the manifest asks for Vulkan, for this reason. The environment map it lights with is baked once per device and shared by every scene.

On 3 October 2026, driving the game on the Seeker against a running server, the text started drawing with letters missing and NFT pictures came out as noise, and stayed that way until the game was restarted. The phone's log said why: Impeller's Vulkan backend had waited too long on the GPU, and MediaTek's driver had declared the GPU lost and reset it. Anything uploaded before the reset (the atlas the letters are drawn from, the pictures) was broken after it, and the driver went on timing out every ten seconds for as long as the game ran, taking the phone's own status bar with it.

Three things set it off, found one at a time by opening and leaving hunts while reading the log:

- **Baking the sky's lighting.** The same night-sky environment map was baked afresh for every scene: about 0.4 s of GPU work each time a hunt or the sanctum opened. It was then baked once, at a quarter of the default size, and shared.
- **Drawing at the screen's full density.** The Seeker's screen is three physical pixels to the point, and its GPU (a two-core Mali-G615) drew the lit, bloomed sanctum at about 30 frames a second. The 3D was then drawn at no more than two pixels to the point, which is as sharp at arm's length; the sanctum ran at about 45.
- **Closing the camera.** Even with both of those fixed, the reset came a tenth of a second after the camera hardware closed, every few hunts. That was between the camera plugin, the engine and the driver, not the game.

So the Flutter game's Android build drew through Impeller's OpenGL ES backend instead (`io.flutter.embedding.android.ImpellerBackend` in the manifest). There, the same moments are at worst a slow frame: eight hunts opened and closed and a minute idle gave no resets, no timeouts and no stalls at all. iPhones draw through Metal and are untouched.

For the leptoskit app, the same three rules hold for whatever draws the 3D and the camera on Android: bake an environment map once and share it, never per scene; cap the 3D at two pixels to the point; and before trusting a backend, open and close hunts on the Seeker while reading its log. If Vulkan resets again, OpenGL ES is what held.

## The Seeker has no ARCore

For the record, since it is why the first game read rooms from the camera picture, and why the Seeker plays in the open now:

- Google Play Services for AR is not installed on the Seeker, and the phone does not report the `android.hardware.camera.ar` feature that certified devices do. Both checked over `adb` on 1 October 2026.
- No Solana Mobile device is on [Google's list of ARCore supported devices](https://developers.google.com/ar/devices).

## Device behaviour the camera hunt must keep

Learned on the Seeker, and kept as requirements for the hunt on `leptoskit_xr`:

- The screen stays on while the game is visible. Movement and stillness do not reset the display's idle timer, so the game holds the screen awake itself; the flag stops applying when the game is backgrounded, and the phone's display-timeout setting is unchanged.
- Camera handover is serialized across hunts: a new hunt waits until the previous one has finished closing its camera. This also applies when a player leaves while the camera is still opening.
- A scripted journey that opens two hunts, completes them and an immediate rematch, then returns to the threshold, is how camera reopening was checked on the hardware. It must run in a separate install with a volatile player, never against the player's own install and save. `leptoskit device` and `leptoskit test` are where it goes next.
