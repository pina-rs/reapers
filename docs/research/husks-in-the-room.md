# Husks that live in the room: research and a recommended architecture

> **A note on the code this describes.** This record was written before the port to Rust. Its sections on Flutter rendering, `flutter_scene`, the Dart `HuntSim` and `reapers_shared` paths describe the retired Flutter app, and its estimates of code size and effort are those of that attempt. The rules it recommends now live in `crates/reapers_rules/src/room`: `RoomGrid` for the room, `RoomNav` for the surfaces and the leaps between them, `Leap` for the ballistic solve and `HuskMotion` for the locomotion states. The research, the numbers and the sources stand as written.

Research for Reapers, 9 November 2026. Scope: how to make husks walk across real surfaces, jump between them believably, and look physically present over the camera feed. The constraints are the Seeker (no ARCore, Mali-G615 MC2, OpenGL ES), 3DOF tracking, a bundled monocular depth model, flutter_scene 0.23, and a deterministic 60 Hz Dart simulation that the server replays.

Sources are cited inline. Anything I could not confirm from a primary source is marked **(unverified)** and collected in section 8.

---

## 0. The short version

1. **Most of the "floating" is in the motion, not the map.** Today every move, walk or leap, is a smoothstep-eased interpolation of bearing, range and height (`HuntSim._continueMove`). The husk eases out of a standstill at take-off and eases into landing. That is the opposite of a jump: a real jump leaves at full speed, keeps its horizontal speed, and hits the ground at full speed. The shadow's ground height is also smoothstepped between the take-off and landing surfaces (`HuntSim._ground`), so during a leap off a desk the shadow drifts down through mid-air instead of staying on the desk top and then dropping to the floor at the edge. Fix both first. They cost almost nothing, and they are the largest single cue (section 1).
2. **Replace the polar plan with a gravity-aligned Cartesian level grid.** Use 10 cm cells, each holding a surface level, an obstacle height or "unknown/hidden". Send that grid, compressed, to the server; I estimate 0.5 to 2 KB (unmeasured). From it, `reapers_shared` derives walkable regions, edges, jump links and spots deterministically on both sides. This is the structure Niantic's Lightship navigation mesh uses: a 15 cm tile grid of heights flood-filled into surfaces, with jumps between surfaces as off-surface steps in A* ([source](https://raw.githubusercontent.com/niantic-lightship/ardk-upm/main/Runtime/NavigationMesh/Internal/NavMeshModel.cs)). It is also how Recast and Unity find jump links: walk the edges of walkable areas and test whether a landing exists ([Unity](https://docs.unity3d.com/2022.2/Documentation/Manual/nav-BuildingOffMeshLinksAutomatically.html)).
3. **Move husks with a small locomotion state machine in the sim:** walk, stop at the edge, look, crouch, ballistic flight at g = 9.81 m/s², land, recover. A turn-in-place state stops the Peridot-style slide ([source](https://www.ahyoungjoo.com/peridot)). Timing comes from physics, not easing.
4. **Ground it on screen, cheapest first.** The shadow is pinned to the support surface and clipped by the level grid, with darker contact under each foot and ground AO on the husk's lower body. The husk's light and colour follow the camera frame. A key-light direction comes from the panorama the sigil scan already sweeps. Camera-matched grain and a slightly lower render resolution follow. Motion blur during fast turns comes later.
5. **3DOF needs a pivot model.** A player who turns their body to look around swings the phone's camera on a circle roughly 0.3 to 0.45 m in radius **(unverified estimate)**. It does not turn about the lens. With the camera assumed fixed, a husk 3 m away that pans from screen centre to the edge can slip about 3° (about 100 px) against the floor under it. Model the camera as a fixed pivot plus an orientation-dependent offset, in both the scan and the renderer. Also keep husks 2 to 4 m away, not 0.8 m (section 6).

---

## 1. Why husks look like they float today (from the code)

What the current implementation does, read directly from the repository:

| Observation | Where | Why it reads as floating |
| --- | --- | --- |
| Every move is `smoothstep` eased over its whole duration. Walks interpolate bearing and range linearly in eased time, so paths are spirals in Cartesian space and the body accelerates and decelerates every time. | `hunt_sim.dart` `_continueMove` | Speed never settles to a gait. Legs that swing "as fast as the creature is going" then skate. |
| A leap is the same eased interpolation, with `arc * 4 * eased * (1 - eased)` added to height. | `_continueMove`, `_leapArc` | Near-zero speed at take-off and landing and a slow hang in the middle: the textbook "floaty" jump. A ballistic arc has constant horizontal speed and maximum vertical speed at both ends. |
| During a leap the ground under the husk is `smoothstep` interpolated from the take-off surface to the landing surface. | `_ground` getter, `husk_rig.dart` `_placeShadows` | The shadow drifts down through the air between desk top and floor. Real shadows stay on the desk until the body passes its edge, then jump to the floor. This is one of the strongest "is it touching?" cues, per Apple's own demo ([WWDC19 603](https://developer.apple.com/videos/play/wwdc2019/603/): "when the shadow isn't present, it becomes difficult to determine where this robot is relative to the table"). |
| On a ledge a husk never walks. It stays put 40% of the time and otherwise leaps down. | `docs/gameplay.md` | It cannot "walk across the desk" because the plan has no desk-top area, only one height per 5° slice. |
| The room is 72 slices × 90 rings, and only the first obstacle per slice is kept. At 3 m a slice is 26 cm wide. | `room_map.dart`, `hunt_plan.dart` | A desk is a run of slice fronts, not an area. Leap targets are polar points "0.6 of half width in from the front edge". |
| The occlusion map leaves out only the floor. Level tops such as a desk are kept. | `occlusion_map.dart` | A husk standing on the desk is tested against the desk top at the same depth as its feet, so its feet flicker in the 12 cm fade band. |
| A husk may stand as close as 0.8 m. | `docs/gameplay.md` | At 0.8 m, 3 cm of hand tremor slips the husk about 2° (roughly 70 px) against the floor (section 6). |
| Heading is never corrected, by design. | `attitude_filter.dart` | Gyro yaw drift makes the whole room slide slowly. Android allows the game rotation vector to drift "by the same order of magnitude as the gyroscope" ([AOSP sensor types](https://source.android.com/docs/core/interaction/sensors/sensor-types)). That doc's illustrative compensated bias is below 0.01°/s, about 0.6°/min. |
| Depth frames are unprojected from one origin, assuming the camera rotates about its lens. | `room_map.dart` | When the player turns their body, the lens moves. See section 6. |

None of this is a criticism of the polar plan's job (reach and cover). It simply cannot express "walk on the desk".

---

## 2. How the best AR products do it

### 2.1 Niantic Lightship ARDK: Gameboard (2.x) and Navigation Mesh (3.x)

This is the closest analogue, and its source is public in [niantic-lightship/ardk-upm](https://github.com/niantic-lightship/ardk-upm/tree/main/Runtime/NavigationMesh). Peridot's engineering post names "the Gameboard" as its navigation system ([Niantic, via search snippet](https://nianticlabs.com/news/engineering-peridot-ai-powered-computer-vision?hl=en); the page now redirects and 404s).

**Data structure: a square grid of height samples, grouped into "surfaces".** Defaults are in [`ModelSettings.cs`](https://raw.githubusercontent.com/niantic-lightship/ardk-upm/main/Runtime/NavigationMesh/Internal/ModelSettings.cs):

| Setting | Default | Meaning |
| --- | --- | --- |
| `TileSize` | 0.15 m | Grid cell size |
| `KernelSize` | 3 | Neighbourhood used to fit a plane per cell |
| `KernelStdDevTol` | 0.2 | Height standard deviation above which a cell is "too noisy to be walkable" |
| `MaxSlope` | 25° | Steeper cells are not flat |
| `StepHeight` | 0.1 m | Most two neighbouring cells may differ and still be one surface |

**Algorithm** ([`NavMeshModel.cs`](https://raw.githubusercontent.com/niantic-lightship/ardk-upm/main/Runtime/NavigationMesh/Internal/NavMeshModel.cs)):

1. A scan raycasts straight down at every tile centre of a square around the camera, from the camera's height, against the reconstructed mesh. The hit height becomes the tile's elevation.
2. A 3×3 kernel fits a plane and takes the standard deviation. A tile is walkable when it passes the noise, slope and minimum-elevation tests.
3. A flood fill from the scan centre adds neighbours whose height differs by at most `StepHeight`. The result is merged into existing "Surfaces" when they overlap and their heights agree within 2 × `StepHeight`. New obstacles carve holes in old surfaces.

**Jumps** ([`PathFinding.cs`](https://raw.githubusercontent.com/niantic-lightship/ardk-upm/main/Runtime/NavigationMesh/Internal/PathFinding.cs)):

- A* runs on tiles with a Euclidean heuristic. Off-surface tiles are allowed as intermediate nodes that count up an "aggregate off-surface" distance.
- A successor is discarded when `sqrt(elevationDiff² + offSurfaceDist²) > JumpDistance`.
- Jumps add a fixed `JumpPenalty`.
- A waypoint whose parent is on a different surface is tagged `SurfaceEntry`.
- There is no separate up or down limit and no path smoothing.

**Agent** ([`LightshipNavMeshAgent.cs`](https://raw.githubusercontent.com/niantic-lightship/ardk-upm/main/Runtime/NavigationMesh/LightshipNavMeshAgent.cs)):

- It lerps between waypoints at 3 waypoints/s.
- On `SurfaceEntry` it pauses 0.5 s, then jumps for 0.5 s along a lerped XZ path with a parabola of height `max(0.1, |Δy|)`.
- It has no animation hooks.

**Lesson:** the data structure is right and cheap: a grid of heights, flood-filled into surfaces, with jump links found by search. The sample motion is crude, so the realism has to come from your own locomotion layer. Peridot's own team reported that the original locomotion "had forward movement only, so the creature would often … slide on the ground and rotate while standing", and fixed it with turning animations ([portfolio of a Peridot developer](https://www.ahyoungjoo.com/peridot)).

### 2.2 ARCore Depth Lab (Google, UIST 2020)

[DepthLab](https://augmentedperception.github.io/depthlab/) ([paper](https://dl.acm.org/doi/10.1145/3379337.3415881); code at [googlesamples/arcore-depth-lab](https://github.com/googlesamples/arcore-depth-lab)) works on raw per-frame depth with no reconstruction.

- **Avatar path planning:** move the character parallel to the ground plane, then raymarch down along gravity from the top of the character into the depth map. The first hit becomes the new foot height, smoothed with a 1€ filter. The character "hovers over" obstacles. This is local, not planned, and it is exactly the "floats across the desk" look.
- **Collision:** a majority vote of the eight bounding-box corners against the depth map, because depth is noisy.
- **Shadows:** render the depth map as a screen-space mesh from the light's point of view, so real geometry receives virtual shadows. They note that flat-plane shadows show "very noticeable artifacts on non-planar objects".
- **Occlusion:** per pixel against a smoothed depth map, with "a soft blur in depth boundary regions". This is the same idea as your 12 cm fade.

**Lesson:** per-frame depth tricks give occlusion and shadows. Credible walking needs a persistent surface map.

### 2.3 Microsoft and Asobo: Young Conker and Fragments (HoloLens 1)

This is the best published precedent for a character that runs across your furniture. The source is Microsoft's [case study](https://learn.microsoft.com/en-us/windows/mixed-reality/out-of-scope/case-study-expanding-the-spatial-mapping-capabilities-of-hololens).

- The playspace is stored as **8 cm voxels aligned to the room's principal axes**, found by PCA during the scan.
- Surfaces are labelled Floor, FloorLike, **Platform** ("horizontal platform between the ground and the ceiling"), Ceiling, WallExternal and WallLike. The floor is "the largest and lowest horizontal surface with greater than 1 m² surface area."
- **Shape queries** match horizontal surfaces against constraints. "Sittable" means a surface 0.2 to 0.6 m high with at least 0.035 m² of area. A couch is two aligned rectangles.
- A placement solver takes rules and constraints and returns the best location, with fallbacks: "Having a set of fallback configurations is critical to supporting functionality across many room configurations."
- It needs a one-time scan phase, and queries only work afterwards. The same is true of Reapers.

The behavioural lesson ([Game Developer](https://www.gamedeveloper.com/design/what-asobo-studio-has-learned-from-designing-games-for-hololens)): Conker "will switch to a vertigo/teetering animation when he reaches the edge of a coffee table". They tested against "potential room configurations representative of 99% of real situation" in a virtual world. That matches Reapers' drawn rooms.

### 2.4 Apple ARKit and RealityKit

- ARKit labels planes and mesh faces: floor, table, seat, wall, ceiling, window, door. With LiDAR it reconstructs a mesh, and RealityKit can collide physics bodies against it ([summary of Apple's sample and tech talk](https://github.com/tjswope/VisualizingAndInteractingWithAReconstructedScene/blob/main/README.md)).
- RealityKit's compositing pipeline is the reference for "looks present" ([WWDC19 session 603](https://developer.apple.com/videos/play/wwdc2019/603/)):
  - grounding shadows ("one of the most important effects"), either a simple drop shadow or a "much more realistic" one;
  - motion blur from the camera's real exposure time ("when motion blur is not enabled, this content feels like it's just stuck on top of the video feed");
  - depth of field that follows the camera's focus;
  - camera grain ("when the noise comes in, it really feels like it's beneath the camera image and not just on top of it").
- Each effect can be turned off for GPU budget (`disableGroundingShadows`, `disableMotionBlur`, `disableCameraGrain`, …) ([Apple docs](https://developer.apple.com/documentation/realitykit/arview/renderoptions-swift.struct/disablegroundingshadows)). That tells you Apple treats them as tunable cost items too.

### 2.5 Snap Lens Studio World Mesh

World Mesh 2.0 builds a mesh on non-LiDAR phones from depth estimation plus 6DOF tracking. It "changes over time as the Lens keeps refining", works "better indoors than outdoors", and classifies surfaces only on LiDAR devices ([docs](https://developers.snap.com/lens-studio/features/ar-tracking/world/world-mesh-and-depth-texture), [blog](https://ar.snap.com/blog/world-mesh-2.0)). Hit tests return position and normal. I found **no official navigation or pathfinding** layer; characters on the World Mesh are placed with hit tests and physics colliders **(unverified beyond the docs)**.

### 2.6 Unity AR Foundation with NavMesh and off-mesh links

- You can bake a NavMesh at runtime from AR planes or meshes.
- **Automatic** link generation only makes **Drop-Down** and **Jump-Across** links: "the bake process walks along the edges of the NavMesh and checks if the landing location of the jump is on NavMesh. If the jump trajectory is unobstructed an Off-Mesh link is created" ([Unity manual](https://docs.unity3d.com/2022.2/Documentation/Manual/nav-BuildingOffMeshLinksAutomatically.html)).
- Upward links must be authored as `NavMeshLink`s. Make Drop Height and Jump Distance slightly larger than measured, and add 2 × agent radius to Jump Distance, because the walkable edge is pulled in by the radius.
- Link traversal is up to the game. Unity's own sample `AgentLinkMover` offers Teleport, NormalSpeed, Parabola and Curve; Parabola is a lerp plus `4h·t(1−t)` ([source](https://github.com/Unity-Technologies/NavMeshComponents/blob/master/Assets/Examples/Scripts/AgentLinkMover.cs)).

**Recast** (the engine under Unity's NavMesh) gives the sizing rule: **cell size = agent radius / 2 to / 3**, cell height = cell size / 2, and `walkableClimb` measured in cell heights ([rcConfig](https://recastnav.com/structrcConfig.html)). Mikko Mononen, Recast's author, on automatic jump links: "the interesting jump points are at navmesh edges … cull out a lot of potential jump points by first checking if the landing point is on navmesh" ([recastnavigation group](https://groups.google.com/g/recastnavigation/c/oNvcV1K0CWY)). He built the same contour-sampling idea for automatic cover in Killzone 3 ([Paris Game AI 2011](http://digestingduck.blogspot.com/2011/)).

### 2.7 Pokémon GO Reality Blending

This is occlusion only, announced in 2020 for a few Android phones. "Pokémon will be able to hide behind a real object" ([UploadVR](https://www.uploadvr.com/pokemon-go-occlusion/)). I found no public technical write-up of locomotion on surfaces **(unverified)**.

### 2.8 Summary

| Product | Navigates on | Jump links found by | Path choice | Motion quality (published) |
| --- | --- | --- | --- | --- |
| Lightship Gameboard / NavMesh | 15 cm height grid → flood-filled "surfaces" | A* steps off a surface within `JumpDistance` | Grid A*, jump penalty, 3 modes | Lerp plus parabola; game supplies animation |
| DepthLab | Raw depth per frame | none | none (local raymarch) | Hovers over obstacles |
| Asobo / HoloLens | 8 cm voxels, labelled platforms | Level design over platforms | Placement solver | Hand animation, edge teeter |
| Unity + Recast | Voxelised navmesh polygons | Edge sampling + landing test (down / across only) | A* + funnel | Game-supplied |
| RealityKit (LiDAR) | Reconstructed mesh + physics | Physics | Game-supplied | Strong compositing |

**Consensus:** a gravity-aligned height grid or voxel map, regions grown by step-height, links found at region edges by testing landings, and a game-owned locomotion layer.

---

## 3. Walkable surfaces from rotation-only monocular depth

### 3.1 What's special here

- All frames share one viewpoint, up to the pivot offset in section 6. That is a panorama, so there is no multi-view triangulation. Every point's depth comes from one network pass, and nothing averages out errors that are correlated within one frame.
- Per-frame scale already varies. Your measurements show about 7% from room to room and 9 to 10% shape error after correction (`docs/environment.md`). The existing floor-relative normalisation is the right primary fix and should stay.
- 360MonoDepth, the closest research system, aligns per-tangent-image monocular depths with "deformable multi-scale alignment followed by gradient-domain blending" ([CVPR 2022](https://openaccess.thecvf.com/content/CVPR2022/papers/Rey-Area_360MonoDepth_High-Resolution_360deg_Monocular_Depth_Estimation_CVPR_2022_paper.pdf)). That is too heavy here. The cheap equivalent is a per-frame scale (already done via the floor) plus a **median-of-ratios check on overlap** with what earlier frames put in the map. If the overlap disagrees with the floor-derived scale by more than about 10%, trust neither and down-weight the frame.

### 3.2 Recommended extraction pipeline (phone side; not deterministic, need not be)

**Step 0: filter flying pixels.** Monocular models blur depth across object boundaries. Back-projected, those pixels form ramps between a desk top and the floor behind it ([sharp-boundary depth paper](https://arxiv.org/html/2110.05885); Apple's Depth Pro lists removing flying pixels as a goal, [arXiv 2410.02073](https://arxiv.org/pdf/2410.02073)). Drop any pixel whose relative depth difference to a 4-neighbour exceeds a threshold. Start around 4 to 6% and tune on drawn rooms (no published value found). This alone stops desks from growing a ramp past their real edge.

**Step 1: unproject with gravity and the pivot model** (section 6). Then apply the existing per-frame floor scale.

**Step 2: accumulate into a fine grid of 5 cm cells.** A 6 m radius means 240 × 240 = 57,600 cells. Store per cell:

- a height histogram of 32 to 48 bins of 4 to 6 cm, u8 counts, roughly 2 to 3 MB in total and fine on a phone;
- a separate count of points flagged as level;
- **free-space carving.** For a stride-4 subset of pixels, step the ray through the grid and record, per cell, the lowest height at which a ray passed through it unobstructed. Points in a cell that lie well above a ray that crossed it are outliers. This is the visibility clean-up used in ETH's elevation mapping: "if pᵢᶻ < hᵢ − σᵢ, remove" ([Miki et al. 2022](https://ar5iv.labs.arxiv.org/html/2204.12876)). The same paper keeps the higher measurement when a cell holds both an edge and the floor below it ("we ignore the points lower than the currently estimated height"). That is the right rule for tops.

  Cost: about 12k rays × about 40 cells per frame, a few to 15 ms in Dart **(unmeasured)**, against 500 ms of inference.

**Step 3: per-cell top and level-ness.** The top is the highest bin with at least k points and not contradicted by carving. A cell is level when a 3×3 kernel plane fit has a residual of 2 to 3 cm or less and a slope of 10° or less. This is Lightship's kernel test, but tighter: their 0.2 tolerance and 25° slope are built for meshes from multi-view stereo, while your heights come from one view **(judgement)**.

**Step 4: snap to a few levels.** Real rooms have a handful of horizontal levels: floor, seat about 0.45 m, desk or table about 0.72 to 0.76 m, counter about 0.9 m. Build a weighted 1 cm histogram of the tops of level cells, smooth it with a σ of 2 cm, and take peaks covering at least 0.08 m². Merge peaks closer than 8 cm. Snap each level cell within ±(6 cm + 1 cm per metre of range) to its level; anything else is an obstacle with its own height. Averaging hundreds of points per level removes most of the monocular height noise. Region growing on height plus normal is the standard approach: PEAC clusters an organised point cloud into planes at more than 35 Hz on 640×480 ([Feng et al. ICRA 2014](https://www.merl.com/publications/docs/TR2014-066.pdf)). Snapping to levels is the 2.5D shortcut.

**Step 5: regions.** Take connected components per level. Clean them with morphological opening and closing by one cell, and drop regions under the minimum area (Recast's `minRegionArea` idea, [rcConfig](https://recastnav.com/structrcConfig.html)).

**Step 6: align the grid to the room's axes.** Take a histogram of the horizontal directions of vertical structure (walls, desk fronts), folded modulo 90°, and rotate the grid to its peak. Asobo does this with PCA for the same reason ([case study](https://learn.microsoft.com/en-us/windows/mixed-reality/out-of-scope/case-study-expanding-the-spatial-mapping-capabilities-of-hololens)): desk edges become straight runs of cells instead of staircases.

**Step 7: downsample to the 10 cm sim grid** by majority vote over 2×2 fine cells, then encode (section 7.1).

**Hidden cells.** Floor that was never seen, directly behind a known obstacle as seen from the player, is marked *hidden* rather than unknown. A husk may stand there to hide, because the player cannot see whether that floor is really there, so the map's uncertainty is invisible exactly where it is used.

### 3.3 Resolution

- Recast's rule of cell = radius / 2 to / 3 gives 8 to 15 cm for husk radii of 0.25 to 0.35 m. **Use 10 cm for the sim grid.**
- A 1.2 × 0.6 m desk is then 12 × 6 cells, and a 35 cm stool 3 × 3.
- Keep the 5 cm fine grid on the phone for rendering: shadow clipping and foot heights.
- Lightship ships with 15 cm tiles; ETH benchmarks 4 cm.
- Erode each region by the husk's radius **plus a distance-dependent margin of about 0.05 × range**, so expected edge error never puts a foot over air. At 3 m that is +15 cm.

### 3.4 How many frames

- The Seeker's view is 72.5° tall when upright (`docs/environment.md`), so roughly 55° wide **(estimate)**.
- For 30 to 40% overlap, step about 33° horizontally.
- Use two rows: about −20° pitch, which sees desk and counter tops at 2 to 4 m from a 1.4 to 1.6 m eye, and about −50°, which sees the near floor.
- A 120° hunting arc needs 4 columns × 2 rows = **8 frames, about 4 s of inference**.
- Every cell the husk will use should be seen in at least 2 frames. That gives √2 noise reduction and lets outlier frames be voted down.
- The sigil's lights can be placed to guarantee this, as they already are for the floor.

### 3.5 Monocular accuracy at the edges that matter

A desk edge at 3 m, read with 10% error, can be about 30 cm off horizontally. Its height error is much smaller: about 0.3 m × sin(17°) ≈ 9 cm before level snapping, and a few cm after **(arithmetic, not measured)**. So **heights are trustworthy and edge positions are the weak point.** Three mitigations, cheapest first:

1. Extra erosion by range (above).
2. Prefer jump take-off and landing points nearer the player, where error is smaller.
3. Optionally snap a desk's front edge to the strongest image line near its projection in the scan frame. This one is research-grade: it adds a Sobel or Hough pass per frame and is **unverified** here.

---

## 4. Creature locomotion on those surfaces

### 4.1 Split of responsibility

- **The sim** (deterministic, `reapers_shared`) owns the root: position (x, y, z), heading, the locomotion **state**, and a **phase fraction** within it.
- **The renderer** (`HuskRig`) owns everything else, all procedural: legs, crouch, squash, gaze, idles. It already works this way ("legs swing as fast as the creature is going", rigid parts on pivots). It just needs richer state from the sim.

This mirrors Overgrowth's approach of a small set of poses plus procedural layers ("13 keyed poses" for base locomotion; "First, do no harm to the gameplay"), from [GDC 2014 Animation Bootcamp](https://gdcvault.com/play/1020583/Animation-Bootcamp-An-Indie-Approach) via [Game Anim](https://www.gameanim.com/2018/03/04/indie-approach-procedural-animation/).

### 4.2 The sim's locomotion states

| State | Entered when | What the root does | Render cue |
| --- | --- | --- | --- |
| `walk` | Following a path | Constant gait speed along a string-pulled path; acceleration capped at about 3 to 6 m/s²; turn rate capped at about 180 to 270°/s | Stride-matched legs |
| `turn` | Heading error over about 60° | Root stays put and rotates at the turn rate | Step-in-place feet. This is the Peridot fix ([source](https://www.ahyoungjoo.com/peridot)) |
| `edge` | Path ends at a link's take-off point | Decelerate to stop about 0.5 to 1 body radius from the edge, facing the link | Head pitches down or up toward the landing; optional teeter (Young Conker) |
| `crouch` | After `edge`, for 0.15 to 0.35 s, scaling with jump height | Root lowers by 10 to 20% of leg length; position fixed | Anticipation squash |
| `air` | Take-off | Ballistic (below) | Stretch at take-off, tuck at apex, legs reach for the landing |
| `land` | Touchdown | Root continues 0.1 to 0.2 s with a dip proportional to impact speed √(2 g h_fall) | Squash, then dust |
| `recover` | After `land`, for 0.2 to 0.4 s, scaling with fall height | Stand up | — |
| `idle-*` | At a spot | See 4.6 | Room-aware idles |

All durations are integer ticks, chosen from the seeded RNG where they vary.

### 4.3 Ballistic jumps at real scale

Use real gravity, g = 9.81 m/s², in game metres. The husks are life-sized and the player's intuition for a 1 m animal is real gravity. A 15% monocular scale error shifts the apparent g by 15%, which is hard to notice. Kyle Pittman's GDC 2016 talk is the reference for designing jumps from height and time and for integrating exactly rather than with Euler steps ([slides](https://media.gdcvault.com/gdc2016/Presentations/Pittman_Kyle_BuildingBetterJump.pdf)): v₀ = 2h/tₕ, g = 2h/tₕ².

**Deterministic tick-exact solve.** Take-off is p₀ = (x₀, y₀, z₀), landing is p₁, and gravity g is fixed.

1. Required apex: y_apex = max(y₀, y₁) + c, where c = max(hop, clearance needed at the edge).
2. Flight time: T = √(2(y_apex − y₀)/g) + √(2(y_apex − y₁)/g). Round it **up** to whole ticks N.
3. Recompute for exactly N ticks: v_y = (y₁ − y₀ + ½ g T²)/T and v_xz = (p₁ − p₀)_xz / T.
4. At tick k: p = p₀ + v_xz·t and y = y₀ + v_y·t − ½ g t², with t = k/60.
5. Check clearance by sampling the arc every 10 cm against the grid's heights plus the husk's half-height. If it hits, raise c by 5 cm and solve again (at most 3 tries; otherwise the link is invalid).

This uses only +, −, ×, ÷ and √. IEEE 754 requires √ to be correctly rounded, so it replays bit-exactly.

**Resulting timings** (the current minimum leap is 34 ticks; these agree):

| Jump | Apex above higher end | Flight | Ticks |
| --- | --- | --- | --- |
| Floor → 0.75 m desk | 0.20 m | 0.44 + 0.20 s | ≈ 38 |
| 0.75 m desk → floor | 0.12 m above desk | 0.16 + 0.42 s | ≈ 35 |
| Floor → 0.90 m counter (ape) | 0.20 m | 0.47 + 0.20 s | ≈ 40 |
| 0.6 m gap on one level | 0.25 m | 0.45 s | ≈ 27 |

Add crouch (10 to 20 ticks) and land plus recover (18 to 36 ticks). A full desk mount takes about 1.2 s, which reads as deliberate and animal-like. For a heavier feel, Pittman's split gravity (heavier on the way down) is available, but real animals don't do it. Keep it at most 1.15× and use it only if testing says the jumps are too light.

**Body mechanics to imitate:**

- Crouch lowers the body's centre of mass before take-off, and the main work happens "during the acceleration from crouch to front toe-off" ([avian jump take-off](https://arxiv.org/pdf/1810.09554)).
- Higher jumps come with a steeper take-off and lower horizontal speed ([agility dogs, PLOS ONE](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11761639/)). So for a jump up, approach the edge closely and go up steeply.
- Cats pre-set their landing posture to the drop height, and the hindlimbs take more of the load as height grows ([Wu et al. 2019](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6721424/)). So scale squash and recovery with fall height.

### 4.4 Choosing where to go and how

- **High level.** Build a region graph whose nodes are regions and whose edges are jump links. Cost = walking time + link flight time + `jumpPenalty`. Use Dijkstra, which is tiny with tens of regions.
- **Low level.** Run A* inside a region on the 8-connected grid with octile integer costs. Break ties by cell index so the result is deterministic. Then string-pull: drop a waypoint whenever a grid line-of-sight (Bresenham) from the previous kept waypoint stays inside the eroded region. Lightship emits one waypoint per tile with no smoothing ([PathFinding.cs](https://raw.githubusercontent.com/niantic-lightship/ardk-upm/main/Runtime/NavigationMesh/Internal/PathFinding.cs)), which is why its agents zig-zag.
- **Goals.** Replace "a polar spot within 50°" with **affordance spots** computed once from the grid (section 7.3): perch edges, peek corners, hide cells, open floor. Pick with the seeded RNG, weighted by the hunt state: bolting prefers cover, ambling prefers perches. Killzone 3's automatic cover annotation, from sampled contours, is the precedent ([Mononen 2011](http://digestingduck.blogspot.com/2011/)).

### 4.5 Render-side locomotion: no skating, no hovering

1. **Stride matching.** Advance the leg-swing phase by *distance travelled / stride length*, not by time, and settle to stance when stopped. Paragon reached the same conclusion: changing play rate "rarely looks natural", so they scaled stride length instead ([Delayen interview](https://www.gameanim.com/2016/11/29/game-anim-interview-laurent-delayen/); [summary](https://www.gamedeveloper.com/programming/most-inspiring-game-animation-tech-talks-of-2016)).
2. **Foot height from the fine grid.** Each foot's ground is the 5 cm grid height under it. On the rigid-pivot rigs, bend the leg pivot so the foot lands on that height ("slope warping" takes ground under the character to place feet with IK, per the same talk). This matters most at the moment a husk steps onto or off a desk edge while walking along it.
3. **Gaze leads action.** Look at the next waypoint about 0.3 s ahead, at the landing point during `edge` and `crouch`, and at the floor below at a perch.
4. **Volume-preserving squash and stretch** on the root: y scale 0.85 to 0.9 when crouched or landing, 1.05 to 1.1 at take-off, with x and z scaled by 1/√(s_y). The rig already has a squash term.
5. **Air poses.** Legs tuck at the apex and reach toward the landing over the last 30% of the flight. Assassin's Creed 3 blended between jump animations for different heights and distances ([Wolfire GDC13 summary](https://www.wolfire.com/blog/2013/04/GDC13-Summary-Animation-Bootcamp-Part-4-6/)). With procedural rigs, the equivalent is parameterising tuck and reach by flight time.
6. Quadruped references: *The Flame in the Flood* on quadruped foot sliding ([GDC 2016](https://gdcvault.com/browse/gdc-16/play/1023209)); [Little Polygon's procedural locomotion series](https://blog.littlepolygon.com/posts/loco1/).

### 4.6 Idles that show the husk knows the room

These are cheap, because they are spots plus a pose, and they are the most persuasive. The husks are hunted, so prey behaviour fits: keep to edges and cover, and check the drop.

- **Peer over an edge.** At a perch, the head pitches toward the floor below, with an occasional teeter (Young Conker's "vertigo/teetering animation", [source](https://www.gamedeveloper.com/design/what-asobo-studio-has-learned-from-designing-games-for-hololens)).
- **Sit on a ledge facing the player, legs dangling over the front.** The legs pass *in front of* the real desk front and below its top, which is very strong proof of contact. Spots: level-region boundary cells whose outward normal faces the player within ±45°.
- **Peek over cover.** Behind cover of known height, raise the head to just above it for 0.5 to 1 s, then duck. Flinch when the reticle sweeps past.
- **Sniff or poke a surface.** The head goes down to the surface height at a nearby cell.
- **Hug walls and fronts** (thigmotaxis) when ambling. Add a small cost bonus in A* for cells next to obstacles.

---

## 5. Compositing and grounding, ranked by realism per cost on a Mali-G615 MC2

The 3D scene is already rendered to an image and masked by `occlusion.frag`, so a single composite shader is where the cheap effects belong.

| # | Technique | Realism gain | Cost | Notes |
| --- | --- | --- | --- | --- |
| 1 | **Shadow pinned to the support surface, clipped by the level grid** | Very high | ~0 GPU; one R8 texture of the grid, about 14 KB | Two blob quads: one at the support level, and one at the level below when the body overhangs an edge. The shader samples the grid at the fragment's world xz and discards where the level differs, so a husk at a desk edge casts half on the desk and half on the floor. Shadow ground changes only when the body crosses an edge. "Shadows provide a strong depth cue" ([DepthLab](https://augmentedperception.github.io/depthlab/)); Apple calls it "one of the most important effects" ([WWDC19](https://developer.apple.com/videos/play/wwdc2019/603/)) |
| 2 | **Contact hardening + per-foot contact darkening** | High | ~0 | Small, dark, sharp ellipses under each planted foot, fading with foot height. The broad blob gets larger and fainter with height (already done). Unity's NavMesh link mover and DepthLab don't do this; it's an easy edge |
| 3 | **Ground AO on the husk** | Medium-high | 1 uniform, a few ALU | Darken the husk's lower body within about 0.3 m of the support height: `ao = mix(0.55, 1, saturate((y - ground)/0.3))`. RealityKit's grounding shadows include AO ([`disableGroundingShadows` doc](https://developer.apple.com/documentation/realitykit/arview/renderoptions-swift.struct/disablegroundingshadows)) |
| 4 | **Ambient intensity + white balance from the camera frame** | High (lighting mismatch is the next most visible tell) | CPU: average a 32×24 downscale every few frames | ARCore's Ambient Intensity mode is exactly "average pixel intensity" plus per-frame "color correction" ([ARCore lighting](https://developers.google.com/ar/develop/lighting-estimation)). Drive the husk light intensity with mean luma relative to a reference, and its tint with grey-world chroma. Low-pass about 0.5 s so auto-exposure swings don't flicker |
| 5 | **Key-light direction and SH ambient from the scan panorama** | Medium-high | Once per hunt, CPU | The sigil sweep already gives frames with known orientation. Accumulate luma into a 32×16 equirectangular map, lock exposure during the scan if the camera plugin allows (**unverified** for the Flutter `camera` plugin on the Seeker), and take the brightest blob above the horizon (window or lamp) as the key light, plus L1 spherical harmonics for ambient. This is what ARCore's Environmental HDR "main directional light" and "ambient spherical harmonics" provide on ARCore devices ([docs](https://developers.google.com/ar/develop/lighting-estimation)). Google's learned version is DeepLight ([CVPR 2019](https://openaccess.thecvf.com/content_CVPR_2019/papers/LeGendre_DeepLight_Learning_Illumination_for_Unconstrained_Mobile_Mixed_Reality_CVPR_2019_paper.pdf)), too heavy here. Offset the blob shadow away from the key light |
| 6 | **Local light probe at the feet** | Medium | 1 texture fetch | Sample the downscaled camera image around the husk's projected feet, relative to frame mean, to darken a husk under a desk or brighten one in a sunbeam. Heuristic, **untested** |
| 7 | **Camera-matched grain** | Medium | A few ALU per covered pixel | Estimate camera noise σ once a second with Immerkær's 3×3 Laplacian method on a luma crop ([CVIU 1996](https://www.semanticscholar.org/paper/Fast-Noise-Variance-Estimation-Immerk%C3%A6r/1da5c5819ae1d33a2a4acc57e16cb655374054e7)), then add hash noise of that σ to husk pixels. RealityKit does the same with ARKit's grain ([WWDC19](https://developer.apple.com/videos/play/wwdc2019/603/)) |
| 8 | **Render the 3D at about 0.75× and upscale** | Medium (camera images are softer than CG) | **Saves** GPU | Klein and Murray's compositing pipeline models distortion, chromatic aberration, blur, Bayer masking, noise, sharpening and colour-space compression so CG matches a cheap camera ([TVCG 2010](https://www.robots.ox.ac.uk/~lav/Papers/klein_murray_tvcg2010/klein_murray_tvcg2010.html)). Blur and noise are the two that matter on a phone |
| 9 | **Landing dust** | Medium for weight | 8 to 16 sprites, a few frames | Tint with camera colour sampled at the landing point. Size from impact speed |
| 10 | **Motion blur during fast turns** | Medium, during turns only | 4 to 6 taps over husk pixels | Blur length ≈ angular velocity × exposure. Exposure isn't exposed **(unverified)**; assume about 1/50 s indoors. "When motion blur is not enabled, this content feels like it's just stuck on top of the video feed" ([WWDC19](https://developer.apple.com/videos/play/wwdc2019/603/)). The gyro already gives `turnSpeed` |
| 11 | **Exclude walkable tops from the occlusion map** | Fixes foot flicker on desks | ~0 | Treat level tops like the floor. Vertical fronts still hide a husk behind the desk |
| 12 | Depth of field | Low on phones | Medium-high | Skip. Phone cameras have deep focus |

---

## 6. Staying anchored with 3DOF only

### 6.1 Untracked translation becomes slip

With rotation-only tracking, any camera translation *t* shows up as angular registration error ≈ *t/d* against real things at distance *d*. Holloway's AR registration analysis: translation error adds directly while rotation error is magnified by the distance ([UNC](https://cdr.lib.unc.edu/downloads/r781wr406)).

The pixel figures below assume about 33 px/° on a screen roughly 2400 px tall showing 72.5° **(assumed screen figures)**.

| *t* (cause) | d = 0.8 m | 1.5 m | 2 m | 3 m | 4 m |
| --- | --- | --- | --- | --- | --- |
| 3 cm (hand tremor, breathing) | 2.1° / ~71 px | 1.1° / 38 px | 0.9° / 28 px | 0.6° / 19 px | 0.4° / 14 px |
| 10 cm (body sway, small shuffle) | 7.1° / 236 px | 3.8° / 126 px | 2.9° / 95 px | 1.9° / 63 px | 1.4° / 47 px |

The eye sees slip at the feet, where the husk meets the floor texture. **Recommendation:** forbid distances under 1.5 m (today the minimum is 0.8 m), and stage surface play (desk walks, perches, hides) at 2 to 4 m. Beyond about 4 m, monocular depth gets worse and the husks get small. Note the tension: nearer is better for depth accuracy, farther is better for anchoring. **2.5 to 3.5 m is the sweet spot.**

### 6.2 The turning pivot is the big one

A player turning on the spot holds the phone 0.3 to 0.45 m in front of their body's vertical axis **(unverified estimate; depends on how people hold the phone)**. When they turn by Δθ, the lens moves about r·Δθ sideways.

The scan baked each frame's lens position in as "the origin", and the renderer also assumes the origin. So a husk at a spot scanned at heading θₛ, viewed at heading θ, is off by about **r·|θ − θₛ|/d**. With r = 0.35 m:

- a husk 25° off screen centre at 3 m is out by about 2.9° (about 95 px);
- at 2 m, about 4.4°.

It slides against the floor as the phone pans across it. The occlusion cut slides the same way. This is a plausible large contributor to "floats around", **and it needs measuring on the Seeker**: put a husk on a recognisable floor tile, pan so it moves from screen centre to the edge, and watch the tile.

**Fix: a pivot (arm) model**, the 3DOF analogue of the VR neck model. Daydream required 3DoF apps to use a neck model ([Daydream requirements](https://developers.google.com/vr/distribute/daydream/design-requirements); `gvr_apply_neck_model` applies "a simple neck model translation based on the rotation", [GVR NDK](https://developers.google.com/vr/reference/ios-ndk/group/headtracking)).

- Camera position = pivot + R_yaw(θ)·(0, h_arm, −r) + R(θ, φ)·(small wrist offset).
- Use it both when unprojecting scan frames and as the render camera's position.
- **Calibrate r per player during the scan.** For candidate r in [0, 0.6] m, unproject two overlapping frames and score their disagreement, for example floor and obstacle-front positions in the overlap. Keep the best. This is a 1-D search that costs milliseconds **(proposed, untested)**.
- Pitch is mostly wrist and elbow, so give it a smaller offset (0.1 to 0.2 m) or leave it out.

### 6.3 Yaw drift

Android's game rotation vector may drift as much as the gyro. A compensated bias of about 0.01°/s, Android's illustrative figure ([AOSP](https://source.android.com/docs/core/interaction/sensors/sensor-types)), is about 1 to 2° over a 2 to 3 minute hunt. That is 5 to 10 cm at 3 m.

**Fix:** keep 2 to 4 downscaled grey keyframes from the scan (80×60, with their orientations). About once a second, when the current heading is within half a view of a keyframe, register the live frame against it. Under pure rotation the images are related by H = K·R·K⁻¹, and a small yaw or pitch error is close to a 2-D shift, so phase correlation or a ±8 px search is enough. Feed the result as a slow correction into `AttitudeFilter`. Cost: a few ms of CPU per second **(estimate)**. It needs the camera stream during the fight, which the depth probe already reads. Google's Instant Motion Tracking pairs gyro rotation with visual tracking for 6DOF on phones without SLAM ([arXiv 1907.06796](https://arxiv.org/abs/1907.06796)); it is a heavier version of the same idea.

### 6.4 Make walking part of the fiction

Android's step detector, or accelerometer step peaks, can tell when the player walks. Rather than letting the room slide, have the husks react (startle, vanish into the veil, or the scan "tears") and ask for a re-trace. This turns a tracking limit into a rule. It fits the existing "stand on the spot" safety rule and the game's "played differently" ethos.

---

## 7. Recommended architecture for Reapers

### 7.1 The room description: `RoomGrid` (in `reapers_shared`)

Gravity-aligned, Cartesian, centred on the player's pivot, rotated to the room's axes.

| Field | Encoding | Bytes |
| --- | --- | --- |
| version, cell size (fixed 10 cm), eye height (cm, u16), pivot arm r (cm, u8), grid yaw (u8, 1.4° steps), width and depth in cells (u8 each), origin cell (u8 × 2) | header | 10 |
| levels: count + height above floor in cm (u8) per level, at most 31 | table | ≤ 32 |
| cells: 1 byte each. Bits 7–5 are the kind: unknown, walkable, obstacle, wall/too tall, hidden. Bits 4–0: for walkable, the level index; for obstacle, top height in 8 cm steps (0 to 2.48 m) | row-major, then RLE + zlib (`dart:io` `ZLibCodec`, on both phone and server) | raw 14,400 for 12 × 12 m; **estimated 0.5 to 2 KB compressed** for a furnished arc (**unmeasured**) |

- Total: roughly 0.6 to 2 KB, against 225 bytes today. Small enough to travel with every hunt and be stored for replay.
- If size ever matters, crop to the cells the husk's arc can reach.
- **Why a grid and not polygons:**
  - every downstream operation (erosion, A*, link search, line of sight, clearance) becomes integer grid work, which is trivially deterministic;
  - polygon clipping and containment in floating point is where determinism and robustness bugs live;
  - Lightship and Asobo both chose grids or voxels.

**Keep** the occlusion map as is: render-only, quarter-degree, never sent. Rebuild it with the pivot model and without walkable tops. **Retire** the polar `HuntPlan` slices for walkers. Fliers can keep their air logic but take perches and obstacles from the grid.

### 7.2 Extraction (phone)

The pipeline in section 3.2:

- flying-pixel filter, pivot-model unprojection, floor scale, with an overlap check;
- 5 cm fine grid with height histograms and free-space carving;
- level-ness kernel, level snapping, regions;
- axis alignment, majority downsample to 10 cm, encode.

It runs incrementally as frames arrive. Finalisation takes under 50 ms in Dart **(estimate)**. It stays arithmetic-only like `RoomMap`, so the drawn-room tests carry over: draw a desk, a sofa and a counter and assert regions, levels and links.

### 7.3 Derivation (shared, deterministic, recomputed on the server)

From `RoomGrid` plus the vessel, build once per hunt:

1. **Standable mask:** walkable cells eroded by ⌈(radius + 0.05·range)/0.1⌉.
2. **Regions:** 8-connected components per level. Reject regions with fewer than about 6 to 9 cells.
3. **Edges:** boundary cells of each region, with an outward normal taken from neighbour differences.
4. **Jump links**, Recast-style: every third edge cell (30 cm), cast along the normal and ±25° in 10 cm steps up to the vessel's maximum leap distance. The landing is the first standable cell of a different region where:
   - −maxDrop ≤ Δh ≤ maxJumpUp (today's per-vessel jump heights; drop ≈ 1.5× jump);
   - the horizontal gap is within the leap distance;
   - the ballistic arc from 4.3 clears the grid.

   Keep the best link per (30 cm edge segment, target region). Expect tens of links. Unity's tip applies: measure gaps from region edge to region edge and add the erosion margin.
5. **Spots:**
   - *perch*: an edge cell on a raised region facing the player within ±45°, for sitting and peering;
   - *peek*: a hidden cell next to an obstacle, with an exposed cell within 2 cells, for duck and peek;
   - *hide*: a hidden cell at least one body width deep;
   - *open*: a floor cell in view, inside the husk's nearest and farthest distances.

Everything is integer or IEEE-exact arithmetic (+ − × ÷ √) with ties broken by cell index. Ballistic positions are evaluated analytically per tick.

### 7.4 Movement in `HuntSim`

- Replace `_setOff` and `_continueMove`'s eased polar interpolation for walkers with a path follower plus the state machine in 4.2. Position becomes Cartesian (x, y, z). Bearing and range are derived for the rules that need them (reach, seals, charge).
- The decision layer stays as it is: when to amble, bolt, or go to cover or a ledge, with the existing probabilities. What changes is the target, which becomes a *spot*, and the route, which becomes regions, links and grid A*.
- **Walking across the desk** falls out of this: a perch spot on the far end of the desk region is reachable by walking within the region.
- `HuskPose` gains `state`, `stateFraction`, `supportHeight` (the level under the root, which changes **discretely** at edges) and `nextFootHold` (for gaze). The renderer reads these and nothing else.
- Charge and stun: "comes down first" stays as it is. A husk on a desk that must charge first jumps down via the nearest link.

### 7.5 Renderer additions (flutter_scene, Impeller GLES)

1. Shadow material samples the grid texture; two masked blobs; per-foot contact ellipses; ground AO uniform (5.1 to 5.3).
2. Camera-driven ambient intensity and tint (5.4); key light from the scan (5.5).
3. Grain and a 0.75× render scale (5.7, 5.8).
4. Pivot-model render camera (6.2) and visual yaw correction (6.3).
5. Stride-matched legs, foot heights from the 5 cm grid, gaze, squash and stretch, air poses, idles (4.5, 4.6).
6. Dust, then motion blur (5.9, 5.10).
7. **A debug overlay** that draws the level grid, regions and links over the camera image, like Lightship's `LightshipNavMeshRenderer`. It is the fastest way to see whether a real desk became a region. Build it first.

### 7.6 Priorities

| Pri | Change | Realism win | Effort (est.) | Runtime cost |
| --- | --- | --- | --- | --- |
| 1 | **Ballistic leaps + crouch, land and recover states; constant-speed walks; shadow ground switches at the edge, not interpolated** | Highest: removes the float in motion | 2 to 3 days, sim + rig | none |
| 2 | **Shadow clipped by surface + foot contact + ground AO; exclude walkable tops from occlusion** | Very high: contact | 1 to 2 days | negligible GPU |
| 3 | **Pivot (arm) camera model in scan and render; calibrate r; minimum distance 1.5 m, staging at 2 to 4 m** | Very high *if* the on-device test confirms the slip | 1 to 2 days + device test | none |
| 4 | **`RoomGrid` + regions + jump links + spots + A* (walk across desks)** | High: the requested behaviour | 1 to 2 weeks incl. drawn-room tests and debug overlay | CPU ms once per hunt |
| 5 | Stride matching, foot heights, turn-in-place, gaze lead, edge peer and teeter, ledge sit, peek | High: "aware of the room" | 1 week | none |
| 6 | Camera ambient and white balance, scan-panorama key light | Medium-high | 2 to 3 days | small CPU |
| 7 | Grain + 0.75× render | Medium | 1 day | saves GPU |
| 8 | Visual yaw drift correction | Medium over long hunts | 3 to 5 days | a few ms/s CPU |
| 9 | Landing dust | Medium | 1 day | tiny |
| 10 | Motion blur on turns | Medium during turns | 2 days | 4 to 6 taps over husk pixels |
| 11 | Image-edge snapping of desk fronts; walking detection as fiction | Medium | research | small |

Items 1 to 3 need no new room representation and should be visible on the Seeker within a week. Item 4 is the structural change that makes "walk across the desk, then jump to the floor" possible.

---

## 8. What I could not verify

- **Peridot internals.** No GDC talk or postmortem on Dots' locomotion found. Niantic's Peridot engineering post now redirects to a 404, so it is cited from search snippets only. The Lightship details are from the current public ARDK 3.x source, and Peridot's production code may differ.
- **Pokémon GO Reality Blending:** only press coverage; no technical source.
- **Snap World Mesh navigation:** no official pathfinding docs found.
- **ARKit plane classification cases** (floor, table, seat, wall, ceiling, window, door) are from memory; Apple's page did not render for fetching.
- **The arm-pivot radius (0.3 to 0.45 m)** and the size of the resulting slip are my estimates. This is the single most important thing to measure on the Seeker.
- **Screen pixels per degree** (about 33) assumes a screen about 2400 px tall showing the full 72.5°.
- **Compressed `RoomGrid` size** (0.5 to 2 KB) and Dart timings for carving and finalisation are estimates, not measurements.
- **Flying-pixel threshold** (4 to 6% relative depth step): no published value found; tune on drawn rooms.
- **Camera exposure lock and exposure-time metadata** through the Flutter `camera` plugin on the Seeker: not checked.
- **flutter_scene 0.23** support for sampling an extra texture (the grid) in a custom `.fmat` shadow material, and for per-frame dynamic geometry: not checked against the package source.
- **Cross-platform determinism of `math.sin`, `cos` and `atan2`** between the phone (ARM) and the server (likely x86): the existing sim already uses them. The proposed new code avoids adding new transcendental calls to the movement path. Worth a replay test across architectures regardless.
- **Gyro drift on the Seeker specifically:** Android gives only an illustrative bias figure.

## 9. Key sources

- Lightship ARDK 3.x NavMesh source: [ModelSettings.cs](https://raw.githubusercontent.com/niantic-lightship/ardk-upm/main/Runtime/NavigationMesh/Internal/ModelSettings.cs), [NavMeshModel.cs](https://raw.githubusercontent.com/niantic-lightship/ardk-upm/main/Runtime/NavigationMesh/Internal/NavMeshModel.cs), [PathFinding.cs](https://raw.githubusercontent.com/niantic-lightship/ardk-upm/main/Runtime/NavigationMesh/Internal/PathFinding.cs), [LightshipNavMeshAgent.cs](https://raw.githubusercontent.com/niantic-lightship/ardk-upm/main/Runtime/NavigationMesh/LightshipNavMeshAgent.cs); [AgentConfiguration API](https://lightship.dev/docs/ardk/apiref/Niantic/Lightship/AR/NavigationMesh/AgentConfiguration/)
- DepthLab: [project](https://augmentedperception.github.io/depthlab/), [ACM](https://dl.acm.org/doi/10.1145/3379337.3415881)
- Microsoft/Asobo: [Spatial Understanding case study](https://learn.microsoft.com/en-us/windows/mixed-reality/out-of-scope/case-study-expanding-the-spatial-mapping-capabilities-of-hololens), [Game Developer interview](https://www.gamedeveloper.com/design/what-asobo-studio-has-learned-from-designing-games-for-hololens)
- Apple: [WWDC19 session 603](https://developer.apple.com/videos/play/wwdc2019/603/), [RealityKit render options](https://developer.apple.com/documentation/realitykit/arview/renderoptions-swift.struct/disablegroundingshadows)
- Google: [ARCore lighting estimation](https://developers.google.com/ar/develop/lighting-estimation), [DeepLight](https://openaccess.thecvf.com/content_CVPR_2019/papers/LeGendre_DeepLight_Learning_Illumination_for_Unconstrained_Mobile_Mixed_Reality_CVPR_2019_paper.pdf), [Instant Motion Tracking](https://arxiv.org/abs/1907.06796), [Daydream neck model requirement](https://developers.google.com/vr/distribute/daydream/design-requirements)
- Snap: [World Mesh docs](https://developers.snap.com/lens-studio/features/ar-tracking/world/world-mesh-and-depth-texture)
- Unity and Recast: [Off-mesh link generation](https://docs.unity3d.com/2022.2/Documentation/Manual/nav-BuildingOffMeshLinksAutomatically.html), [AgentLinkMover](https://github.com/Unity-Technologies/NavMeshComponents/blob/master/Assets/Examples/Scripts/AgentLinkMover.cs), [rcConfig](https://recastnav.com/structrcConfig.html), [Mononen on jump links](https://groups.google.com/g/recastnavigation/c/oNvcV1K0CWY)
- Mapping: [ETH elevation mapping (GPU)](https://ar5iv.labs.arxiv.org/html/2204.12876), [Fankhauser 2018](https://www.researchgate.net/publication/325918227_Probabilistic_Terrain_Mapping_for_Mobile_Robots_With_Uncertain_Localization), [PEAC](https://www.merl.com/publications/docs/TR2014-066.pdf), [360MonoDepth](https://openaccess.thecvf.com/content/CVPR2022/papers/Rey-Area_360MonoDepth_High-Resolution_360deg_Monocular_Depth_Estimation_CVPR_2022_paper.pdf), [Depth Pro](https://arxiv.org/pdf/2410.02073)
- Animation: [Pittman, Building a Better Jump](https://media.gdcvault.com/gdc2016/Presentations/Pittman_Kyle_BuildingBetterJump.pdf), [Rosen, Indie Procedural Animation](https://gdcvault.com/play/1020583/Animation-Bootcamp-An-Indie-Approach), [Delayen on distance matching and stride warping](https://www.gameanim.com/2016/11/29/game-anim-interview-laurent-delayen/), [Flame in the Flood quadrupeds](https://gdcvault.com/browse/gdc-16/play/1023209), [Peridot locomotion fix](https://www.ahyoungjoo.com/peridot)
- Compositing: [Klein & Murray 2010](https://www.robots.ox.ac.uk/~lav/Papers/klein_murray_tvcg2010/klein_murray_tvcg2010.html), [Immerkær noise estimation](https://www.semanticscholar.org/paper/Fast-Noise-Variance-Estimation-Immerk%C3%A6r/1da5c5819ae1d33a2a4acc57e16cb655374054e7)
- Biomechanics: [cat landing](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6721424/), [dog jump take-off](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11761639/), [avian jump take-off](https://arxiv.org/pdf/1810.09554)
- Sensors: [AOSP sensor types](https://source.android.com/docs/core/interaction/sensors/sensor-types), [Holloway registration error](https://cdr.lib.unc.edu/downloads/r781wr406)
