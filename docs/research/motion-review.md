# Reaper motion review

## Reference and criteria

The starting humanoid had narrow cylindrical legs, no readable patella or calf, reversed knee flexion, and the same hovering root animation as the spirit. Each key pose used smoothstep independently, stopping velocity even on an attack's contact pose. The torso and pelvis rotated as one rigid piece.

The replacement uses these references:

- [Jason Martinsen, Animation Mentor: basic human walk cycle](https://www.animationmentor.com/blog/tutorial-animating-human-walk-cycle/) and its [video tutorial](https://www.youtube.com/watch?v=Oda0Au-gj4c). Contact/down/passing/up poses, linear support-foot travel, hip/chest opposition, arm overlap and foot roll guide the gait.
- [Polygon Treehouse: Anatomy of a Walk Cycle, part 2](https://www.polygon-treehouse.com/blog/2019/2/1/walk-pt2). The body lowers as weight transfers; the supported side of the pelvis carries the body. These are useful visual checks even for stylized characters.
- [Jason Shum, Gnomon: Combat Animation for Games](https://www.thegnomonworkshop.com/workshops/combat-animation-for-games). Public workshop outline and lesson descriptions. Strong key poses, aggressive timing, breakdowns and clean arcs guide the combat pass. The paid course was not accessed.

The models remain stylized spirits. The goal is credible anatomy and mechanics, not motion capture or photorealism. Numerical rig checks cannot grade acting, appeal, silhouette or perceived impact. Those need rendered review.

The linked walk video's player failed during research. Its published written breakdown was reviewed; the video was not watched. The combat reference is the public course outline, not its paid lessons.

## Expected performance by state

| State | Visual acceptance target |
| --- | --- |
| Idle | Humanoid weight over feet, slight breathing and weight shift; spirit hover. |
| Creep | Short cautious steps, lowered hips, continuous floor support. |
| Walk | Alternating support and swing, visible forward knee flexion, heel-to-toe roll, opposing arms. |
| Run | Faster cadence, bent recovery knee, brief flight and forward chest lean. |
| Sprint | Longer stride, higher recovery foot, stronger arm swing and forward lean. |
| Jump | Crouch before takeoff, airborne apex, bent knees on landing, settled recovery. |
| Land | Weight drops into knees, then rises without a hovering rebound. |
| Dodge left | Guard, lateral weight transfer, flexed knees and recovery. |
| Dodge right | Mirrored lateral weight transfer and recovery. |
| Phantom dash | Loaded anticipation, fast forward excursion, controlled return. |
| Dash left | Counterload before leftward burst, lower centre of mass, return to start. |
| Dash right | Mirrored burst and return. |
| Laugh | Chest pulses and head overlap; facial joy; balanced feet. |
| Cry | Bowed head, hand near face, restrained chest tremor; facial sorrow. |
| Angry pose | Gather tension then open the silhouette; stable lower body. |
| Fear | Recoil into a wider stance; hands protect the face. |
| Bow | Chest hinges above the pelvis; head follows; feet support the weight. |
| Victory | Raised arm, delayed head turn and restrained lift through the chest. |
| GigaChad | Deliberate broad chest and chin pose, held without shaking. |
| Sigma | Head turn leads the expression; chest follows with a settled hold. |
| Shrug | Asymmetric head tilt and open arms, readable hold. |
| Absolute cinema | Open symmetrical gesture with controlled elbows and stable feet. |
| T-pose | Clean arm extension and held symmetry; intentionally static apex. |
| Anime charge | Lowered stance, increasing chest tension, expansion and settle. |
| Slash | Coil, rapid hip/chest turn, extended contact, weapon follows through. |
| Thrust | Draw back with bent elbow, transfer forward, extend through contact, recover guard. |
| Parry | Raise guard, absorb a small recoil, redirect and settle. |
| Reaping arc | Deeper coil and broad sweep; decelerate beyond contact without spinning backwards. |
| Impale | Loaded elbow and torso, fast extension, readable contact, lift and recovery. |
| Soul crush | Reach and gather, sudden inward contraction, lingering wrist follow-through. |
| Palm burst | Chamber the palm, drive hips and chest forward, extend then retract. |
| Rising uppercut | Crouch and coil, leg drive, rising arm, airborne follow-through and landing. |

## Reproduce the review

```sh
fvm dart tool/build_art.dart reapers
blender --background --python-exit-code 1 --python art/blender/review_catalogue.py -- --class harvester --form humanoid --out art/build/motion-review/humanoid
blender --background --python-exit-code 1 --python art/blender/review_catalogue.py -- --class harvester --form spirit --out art/build/motion-review/spirit
```

Each strip samples five times from the actual exported GLB. The attack strip includes the shared contact marker. The catalogue equips a spear for impale and a scythe for reaping, so it can show every state; studio availability follows the selected held object. Four catalogue pages collect the strips for a full pass. In the studio, compare normal speed and quarter speed, then scrub the same contact poses. Inspect several class silhouettes as well as the bare humanoid body.

## Review observations

The Harvester catalogue was inspected across all 32 states in both forms. The humanoid's thighs, kneecaps and calves now remain readable beneath the short tunic. The walk shows a bent forward knee, a supported foot and a separate lifted swing foot. Jump and uppercut include a lower-body preload and landing compression. Humanoid reactions no longer lift the pelvis with the spirit's hover track.

The first pass exposed two acting problems that were corrected: the cry raised a hand above the head, and the running carry swung the scythe overhead. Cry now brings the hand near the face. Running keeps the carrying arm closer to the torso and bends the free arm. The combat pass separates the pelvis from the chest and gives strikes a loaded elbow, extension through contact, and a later follow-through.

The Hollow was also reviewed in walk, run, slash and palm burst. Its knees flex forward, the carried skull stays attached to the left wrist, and palm burst uses the free right hand. Its running skull carry is intentionally more animated than the Harvester's weapon carry.

The Ferryman was checked in walk, run, thrust and parry, and the Mourner in walk, cry, slash and uppercut. Both keep the same foot mechanics. The Ferryman's shorter arm gestures preserve its heavier silhouette. The Mourner's head stays restrained while its previously concealed arm participates in reactions.

Large hats can obscure a hand near the face in a front view. The three-quarter and side views remain necessary when judging those gestures. The catalogue's memes retain deliberate held poses; T-pose and GigaChad should not be judged by the same continuous movement criterion as walking or slashing.

The browser review checked humanoid and spirit switching while paused, the unrobed anatomy, movement/reaction/meme selection, and finisher contact poses. Laugh and cry exposed an obstructed Harvester close-up, corrected by lowering the camera below the brim. Final close-ups show the eyes and mouth, and the uppercut and reaping contact poses fit the full-figure view. The fresh studio reported no browser console errors during this pass.

The subsequent clothing pass corrected the narrow spirit robe intersecting the humanoid pelvis. Humanoid robes and tunics now have separate fitted targets, leaving clearance around the hips and walking legs. The pelvis belongs to the humanoid group. Spirit travel holds the arms steady and uses restrained hover and forward lean; humanoid travel retains arm opposition. Rear walk renders checked the Harvester's torn second robe, the Hollow's trailing fourth coat and the Mourner's burned second veil at maximum gauntness. The hems retain their authored tears and openings, so feet and portions of the lower legs can remain visible through damaged cloth.

To inspect that specific clothing case, add `--clothing robe --rear --variant 1 --gaunt 1 --only walk,run` to the catalogue command. `--variant` is zero-based. The exported-asset tests check pelvis clearance across every robe cut and class, along with the absence of a cyclic arm swing in spirit travel.

## Limits

Foot IK is solved in Blender and baked into clips. Support-foot travel is calibrated for an in-place preview. A game controller must supply matching forward speed, terrain adaptation and transitions. These clips do not implement enemy reactions, collisions or motion matching. Lateral dashes remain stylized bounded excursions and recover to the preview origin. Rigid limb sections overlap around joints; there is no skin deformation, muscle simulation or cloth collision.
