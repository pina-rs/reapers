# Second Ink: the Reapers art bible

**The world is printed in one ink. Your reaper's borrowed light is the second. You keep it in register by holding still.**

This is the one art bible for the Reapers revamp. When it and an earlier pitch disagree, this document wins.

The following teams build from it:

- the renderer (shader, hulls, light);
- art (Blender scripts under `art/blender`);
- interface (leptoskit and `leptoskit_skribble`);
- the hunt HUD;
- effects;
- motion.

Where a number was measured on a proof, the text says so. Every other number is the specification, and a number not yet measured on a device is marked *provisional*.

**Units.**
- **pt** is a CSS pixel on a 390 × 844 pt phone. The proofs render it at 3 device pixels (1170 × 2532).
- **‰** is thousandths of the view's height.
- **A tick** is 1/60 s: the rules' clock.
- **A frame** is one 60 Hz display frame, so a tick and a frame are the same length.
- **"On twos"** means held decoration redrawn at 12 fps, every fifth frame.

## The proofs

The proof images and the scripts that made them are not kept in the repository; the names below are those of the design round.

| File | What it proves | How it was made |
| --- | --- | --- |
| `final/ui-sanctum.png` | Sanctum: Ferryman on the stage, lamp waxing, tally, the one spot action | HTML and inline SVG, rough strokes seeded, Skribble faces by `@font-face`, playwright-cli from `file://` at 390 × 844, device scale factor (DSF) 3 |
| `final/ui-hunt-bright.png` | Hunt over a bright kitchen: open seal, hit frame 3 of 6, splat, scrawl, figure tag, 3D arm and pistol | same, with Blender renders composited |
| `final/ui-hunt-dark.png` | Hunt over a dark bedroom: the shriek at tick 40 of 66, stepped vignette, HOLD ring 5 pt out of register, arm dropped, FIRE "would flinch" | same |
| `final/ui-result.png` | Result: the taken husk reprinted in your duotone on a specimen plate, five named colour chips, the grade chop | same |
| `final/cel-reaper.png` | Ferryman hero (backlit); all six classes in their new lights; a 64 pt key-only silhouette test; the first-person arm with the grave pistol | `final/work/final_ink.py` in Blender 5.1 on the built models in `art/build/models`; the arm is `final/work/viewmodel_build.py` |
| `final/cel-husks.png` | Five husks in the ghost wash with their NFT and band chips; three over camera plates; one penguin in three printings (ghost, near your hue, reprint) | `final_ink.py`, husk mode |
| `final/board.png` | Captioned contact sheet of all of the above, with the palette and the three faces | HTML at 1800 × 1120, DSF 2 |

The reference implementations behind the proofs are as follows.

- **`final/work/final_ink.py`**: the shading and hull maths. It is the shader written as a Blender node tree.
- **`final/work/viewmodel_build.py`**: the arm and pistol form study.
- **`final/work/palette.py`**: every overprint, contrast and colour-vision-deficiency (CVD) figure in this document.
- **`final/html/parts/ink.js` and `hud.js`**: the rough-stroke and HUD construction.
- **`final/work/knock.sh`**: the paper knockout.

The camera plates are painted stand-ins, not photographs. A real-photo test is a gate before lock (section 16).

---

## 1. The idea and its name

**Second Ink.** Reapers is printed like a two-plate relief print:

- a **key** plate of near-black ink on bone **paper**;
- one **spot** plate.

The spot is the light of the reaper you were sworn to. It is the only colour that floods anything. The dead print in key over a ghost of their own colour. Hatching and flat bands do the work that gradients do elsewhere. The hand-drawn quality comes from printmaking: a seeded, wobbling ink line, misregistration, a deckled sheet, a chop pressed by hand. It does not come from comic gloss, and from no other game.

**Registration is stillness.** The hunt's two plates sit in register while you hold still. As you turn, the spot plate drifts out of register in the direction of the turn. Stillness is the skill this game asks for (the shriek), so the print itself shows it. No other game prints its own state this way, and it is the thing a player will describe to a friend: "the print goes out of register when I move."

What answers the owner's request:

| The owner asked for | Second Ink's answer |
| --- | --- |
| Hand-drawn fonts | Three Skribble faces (section 7), and nothing else |
| 3D with hand-drawn edges | An inverted-hull key line with light-aware weight, surface-fixed wobble and a thickness clamp (section 4). A paper knockout around every figure. Bands and object-space hatching (section 5). |
| "Like games that are 3D but look hand-drawn" | Real 3D models, lit by one fixed light and printed in two inks. The look is ours: relief print and specimen plate, no borrowed palette or trade dress. |
| Fluid motion | 60 fps springs for everything that moves. Held frames only at impacts (2 to 6 frames) and for printed decoration (section 13). |
| World-class | One idea carried through every surface, from the oath to the result card. |

The three laws, in order of precedence:

1. **The key draws the world.**
2. **The spot is yours.**
3. **Registration is stillness.**

---

## 2. Rules that never bend

1. **Borrowed light.** The spot floods only:
   - your reaper;
   - your weapons' light parts and hand bands;
   - your live HUD controls;
   - your ink where your light touches a husk (an open seal, the struck part for 4 frames, a splat, the orb, the capture);
   - your reprinted dead (codex, result).

   One spot per screen. The only exceptions are these, and each light there is labelled with its name:
   - the oath's last frame;
   - the roster;
   - the landing page's lineup;
   - registration-mark tags over other reapers in a rift.

   A husk at rest never wears any spot.
2. **Wound red `#B92A20` means loss or error, and nothing else.** That covers lost resolve, being struck, a failed transaction, a destructive confirmation and a form error.
   - It never floods a shape. It is always a mark: a strike, a tear, a cross, a torn edge.
   - It always comes with words.
   - It never appears on the hunt's camera without a paper slip under it.
   - The type system enforces this: the wound component takes a shape and a localised message, and has no constructor without both (section 15, test W).
3. **Tinted blacks, tinted whites.** No `#000000` and no `#FFFFFF` anywhere.
   - The darkest ink is key `#16121A`. Every dark ground is key screened over paper.
   - The lightest is paper `#ECE3CF`. A specular highlight is bare paper.
4. **Text plus colour.** No state is carried by colour alone. Every state has a word, a shape, or both. The old colour-coded state rings (white, amber, red, blue and gold) become shapes and captions (section 10).
5. **44 pt targets.** Every touch target is at least 44 × 44 pt. FIRE and ORB hit areas are 96 pt; tabs are 78 × 56 pt. Targets never overlap, and drawn size may be smaller than the hit area.
6. **Reduced motion.** Everything that moves has a still form (section 14).
   - It follows the OS setting and an in-game "Steady print" switch.
   - The native shells must pass the OS setting into the app (a dependency, section 14).
7. **Localisation.**
   - Every string a player reads lives in `crates/reapers_app/locales/en.arb` (`crates/reapers_landing/locales/en.arb` for the landing page).
   - No text is baked into a texture, model or image.
   - Layouts take strings 40% longer than English, and Display names fit their box (section 7).
   - Numbers, dates and currency go through the locale formatter.
   - Arrows and symbols the faces lack are drawn as icons, never typed (section 7).
8. **Dignity and hidden faces.**
   - A reaper's face is never shown. Each class hides it its own way: hood void, veil, helm, mask, or no head at all. At most two light points stand for eyes.
   - Husks are characters from real NFT collections and are treated with respect: no gore, no blood, no dismemberment, no mockery.
   - Capture is absorption into light, not a kill.
   - A husk's face and mouth are never covered by the HUD: captions sit under it and HOLD sits on the chest or below.
   - The creature's identity is never erased. The ghost wash keeps its hue, and its full colours survive as named chips on its specimen plate.
9. **No flash.**
   - Nothing changes the luminance of more than 10% of the screen within a third of a second, and no region flashes more than three times a second (WCAG 2.3.1).
   - A hit is a colour change on one region of the husk.
   - The shriek darkens in three steps over 54 ticks.
   - **No luminance switch:** the page never flips between a light and a dark theme. It is always the vigil page, and paper appears only as objects (slips, sheets, cards).
10. **The camera is the room.**
    - The camera image is never tinted, blurred or dimmed, except by the shriek's stepped vignette.
    - Paper covers at most 12% of the screen during a hunt, and 15% during a shriek.
    - Measured: 10.7% in `ui-hunt-bright.png`, 13.6% in `ui-hunt-dark.png`.
11. **Ink is seeded and cached.**
    - Every rough stroke has a seed and is drawn once.
    - Animation moves cached ink by transform only (translate, rotate, scale, opacity).
    - Nothing re-randomises per frame. The only exception is boil, which steps through three cached seeds.
12. **Models are built, never edited.** Every form change in section 6 is a change to a script under `art/blender`, followed by `cargo xtask build-art`. Render-script overrides used in the proofs are not shipped (Appendix A).

---

## 3. Palette

### 3.1 Inks and grounds

Every neutral is key at coverage `k` printed over paper. Every colour on a reaper is a pair `(k, s)`: key and spot coverage over paper.

`colour = paper × (1 − k + k·key/paper) × (1 − s + s·spot/paper)`, per sRGB channel.

| Token | Hex | k | L* | Role |
| --- | --- | --- | --- | --- |
| `paper` | `#ECE3CF` | 0 | 90.4 | The stock: slips, sheets, cards on the page, specimen plates, primary text on dark grounds, the knockout, specular highlights |
| `ash` | `#A39C93` | about .34 | 64.7 | Quiet text on vigil and scrim, inactive icons, disabled labels |
| `raised` | `#454042` | .78 | 27.6 | Cards and wells on the page, rules |
| `vigil` | `#342F33` | .86 | 20.1 | The page ground, always |
| `scrim` | `#2B272C` | .90 | 16.2 | Disabled dial fill, wells, the shriek's darkest tint. Never under small text. |
| `key` | `#16121A` | 1 | 6.1 | Every line, text on paper and on spot, husk deep hatch |
| `wound` | `#B92A20` | n/a | 41.3 | The third ink: loss and error only (rule 2) |

Paper is L* 90.4, C* 10.9, h 91. Wound is L* 41.3, C* 69.3, h 36.6.

**Value ladder** (key over paper, for drawing and tints):

| k | .24 | .36 | .55 | .60 | .78 | .85 | .86 | .90 | 1 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Hex | `#B9B1A4` | `#9F988E` | `#76706B` | `#6C6662` | `#454042` | `#363135` | `#342F33` | `#2B272C` | `#16121A` |
| L* | 72.5 | 63.2 | 47.6 | 43.6 | 27.6 | 21.0 | 20.1 | 16.2 | 6.1 |

The page grain is key fractal noise at 22% opacity, one small tile. Low-end devices may drop it.

### 3.2 The six lights (one spot per class)

A class has one light, whatever its colourway. Colourways change how much light is in the cloth, not its hue (3.4). The lights were chosen to keep at least 6.5 L* apart, so they never depend on hue alone. Hollow's and Weigher's hues sit 5° apart and are separated by 21 L*.

| Class | Light | Token | Hex | L* | C* | h | Replaces (today, `theme.rs` `accents`) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Harvester | Chaff | `light.chaff` | `#F4CD50` | 83.7 | 64.9 | 89.5 | bone white `#F3EBD8` |
| Ferryman | Lantern | `light.lantern` | `#F07428` | 62.7 | 74.5 | 54.2 | amber `#FFB347` |
| Weigher | Verdigris | `light.verdigris` | `#2FBD9F` | 69.1 | 44.1 | 174.0 | ice white `#E8F4FF` |
| Chooser | Cobalt | `light.cobalt` | `#477FE4` | 54.2 | 58.6 | 284.3 | red `#FF3B2F` (collided with wound) |
| Hollow | Wisp | `light.wisp` | `#A8F2D8` | 90.2 | 28.7 | 169.4 | orange `#FF7A1A` |
| Mourner | Mauve | `light.mauve` | `#CDB2F6` | 77.0 | 37.8 | 307.0 | pale blue `#DDEBFF` |

- **L\* gaps**, sorted: 8.5, 6.5, 7.9, 6.7, 6.5.
- **Minimum CIEDE2000 between lights** (Machado 2009 simulation at full severity):

  | Vision | Minimum ΔE | Closest pair |
  | --- | --- | --- |
  | Typical | 15.9 | verdigris and wisp |
  | Protan | 13.6 | verdigris and wisp |
  | Deutan | 13.5 | chaff and lantern |
  | Tritan | 14.7 | verdigris and wisp |

- **Against wound**, the minimum is 24.6 in typical vision (lantern) and 18.5 under tritan (lantern).

Wherever lights appear together, each one is named.

**What may sit on a light, and where a light may sit** (all values are WCAG contrast ratios):

| Light | Key on spot | Spot on vigil | Spot on scrim | Spot on paper |
| --- | --- | --- | --- | --- |
| Chaff | 12.06 | 8.56 | 9.58 | 1.20 |
| Lantern | 6.37 | 4.52 | 5.06 | 2.27 |
| Verdigris | 7.84 | 5.56 | 6.23 | 1.85 |
| Cobalt | 4.78 | 3.39 | 3.79 | 3.03 |
| Wisp | 14.40 | 10.22 | 11.44 | 1.01 |
| Mauve | 9.95 | 7.06 | 7.90 | 1.46 |

The rules that follow:

- **Text on spot is always key.** It passes for every light, from 4.78 up.
- **Spot text** appears only in Display at 22 pt (s5) and above, and only on vigil, scrim or key. Cobalt at 3.39 passes the large-text threshold, and nothing smaller is allowed for any light.
- **Spot never touches paper without key.**
  - On paper, every spot shape has a key outline, or a key underprint 1.5 pt down and right (the key plate).
  - Spot type on paper always carries that underprint. The result's "Excellent" chop is the proof.
  - Wisp on paper is 1.01:1, so this rule is not optional.

### 3.3 Reaper families: band coverages

Each material family prints three bands (lit, shade, deep) as `(k, s)` pairs. Use these exact values; the hexes per light follow in 3.4.

| Family | Lit k/s | Shade k/s | Deep k/s | Hatch | Specular (paper) |
| --- | --- | --- | --- | --- | --- |
| `cloth` (Night) | .46/.84 | .80/.50 | .93/.16 | yes | none |
| `cloth_alt` (mantle; robe in Ember) | .32/.92 | .70/.60 | .90/.20 | yes | none |
| `feather` | .56/.74 | .84/.36 | .95/.10 | yes | none |
| `bone` | 0/0 | .24/.10 | .55/.08 | no | N·H > .95 |
| `metal` | .34/0 | .64/.06 | .86/.05 | yes | N·H > .90 |
| `gold` (spot-printed parts) | 0/1 | .34/1 | .66/.90 | no | N·H > .92 |
| `wood` | .46/.55 | .72/.40 | .90/.18 | yes | none |
| `rope` | .08/.42 | .38/.46 | .74/.26 | no | none |
| `paper` (tags, seals, scrolls) | .04/.08 | .24/.12 | .50/.10 | no | none |
| `spirit` | flat .94/.10 | | | no | none |
| `void`, `dark` | flat key | | | no | none; no line |
| `glow`, `accent`, `fx_glow` | flat spot | | | no | none; no line; printed halo (5.5) |

A family name is the material name in the `.glb` (Blender suffixes such as `.001` are ignored). An unknown name falls back to `cloth`.

### 3.4 Per-class colours (lit / shade / deep)

**Robe, Night colourway (`cloth`).**

| Class | Lit | Shade | Deep |
| --- | --- | --- | --- |
| Harvester | `#8D783C` | `#42392B` | `#252023` |
| Ferryman | `#8C4D28` | `#412D25` | `#251E22` |
| Weigher | `#2D7064` | `#273737` | `#202025` |
| Chooser | `#395286` | `#2A2F41` | `#211E27` |
| Hollow | `#688A80` | `#373E40` | `#232127` |
| Mourner | `#7A6B8F` | `#3D3544` | `#242028` |

**Robe, Ember colourway (`cloth_alt` coverages on the robe).**

| Class | Lit | Shade | Deep |
| --- | --- | --- | --- |
| Harvester | `#AD9241` | `#584C33` | `#2C2627` |
| Ferryman | `#AA5826` | `#573929` | `#2C2325` |
| Weigher | `#2C8775` | `#2D4945` | `#24262A` |
| Chooser | `#3C5FA3` | `#323B55` | `#25232D` |
| Hollow | `#7BAA9B` | `#475452` | `#29272C` |
| Mourner | `#9380AF` | `#4F4659` | `#2A252E` |

**Robe, Ash colourway.** Coverages are .52/.10, .82/.06 and .94/0. The light stays on the trim, rim, eyes and weapon band.

| Class | Lit | Shade | Deep |
| --- | --- | --- | --- |
| Harvester | `#7D756A` | `#3D3738` | `#231F25` |
| Ferryman | `#7D7168` | `#3D3638` | `#231F25` |
| Weigher | `#73746E` | `#3A373A` | `#231F25` |
| Chooser | `#747172` | `#3A363B` | `#231F25` |
| Hollow | `#797771` | `#3B383B` | `#231F25` |
| Mourner | `#7B7473` | `#3C373B` | `#231F25` |

**Gilt (`gold`).**

| Class | Lit | Shade | Deep |
| --- | --- | --- | --- |
| Harvester | `#F4CD50` | `#A98D38` | `#625127` |
| Ferryman | `#F07428` | `#A6501C` | `#603218` |
| Weigher | `#2FBD9F` | `#218270` | `#1A4C45` |
| Chooser | `#477FE4` | `#3157A0` | `#233660` |
| Hollow | `#A8F2D8` | `#74A698` | `#465E5B` |
| Mourner | `#CDB2F6` | `#8E7AAD` | `#544866` |

**Neutral families,** shown for the Ferryman. They barely move between classes.

| Family | Lit | Shade | Deep |
| --- | --- | --- | --- |
| Bone | `#ECE3CF` | `#B9A896` | `#766C65` |
| Metal | `#A39C91` | `#635B57` | `#342E31` |

For any other family or light, compute the colour with `palette.py`'s `overprint(k, s, spot)`.

### 3.5 The three axes of a reaper's look

The three axes are orthogonal: each changes one thing, so any combination stays legible.

| Axis | What it changes | Values |
| --- | --- | --- |
| **Colourway** (the shuffle) | How much light is in the cloth | **Night**: robe `cloth`. **Ember**: robe `cloth_alt`. **Ash**: robe key-only (.52/.10, .82/.06, .94/0). |
| **Omen** (the shuffle) | How the spot is treated | **None** (80%): as drawn. **Ashen** (12%): the reaper's spot coverage is halved on every family (Ferryman Night lit becomes `#8B6852`); the HUD keeps the full light, because legibility comes first. **Gilded** (6%): the `metal` family prints as `gold`, so every buckle and blade is gilt in the class light, not a new gold hue. **Eclipsed** (2%): the spot leaves the body (cloth at Ash coverages); it survives as the rim, the eyes and the weapon's light, and the sanctum lamp prints as a key disc with a spot corona. |
| **Outfit** (earned) | Key marks and voids, driven by the model's `COLOR_0` masks (R grime, G char, B wear, A hollow) | **Shrouded**: no marks; spot coverage in the cloth ×0.55 (undyed). **Sworn**: a key running stitch (1 pt, dashes 4 pt on, 3 pt off, in object space) where wear B crosses 0.5. **Scorched**: char hatch climbing from the hem, a key crosshatch at any N·L where G > 1 − 0.5, edged by a 1.5 pt spot ember line at the front. **Hollowed**: char at 0.8, and faces where A > 1 − 0.42 print flat key with no line (void); headless. |

**Dependency.** `crates/reapers_app/build.rs` strips `COLOR_0` from every model today, so the outfit axis cannot draw.

- Keep the masks: rename the attribute to a custom `_MASK` attribute in the art build, so nothing draws them as colours. `build.rs` keeps `_MASK` and goes on removing `COLOR_0`.
- Until that lands, outfits are not provable, and the proofs do not show them.

### 3.6 Husks: the ghost wash

The dead print in key over a ghost of their own colour. Inputs are the NFT look's `body`, `trim` and `detail` colours (`vessels.json` looks, via `reapers_rules::nft_look`).

1. **Lit**, in CIELAB:
   - keep the hue;
   - cap chroma at C\* 16;
   - clamp L\* to 34–90.
2. **Clash rule.** If the colour's hue lies within 20° of the player's light hue, cap chroma at C\* 8 instead. The dead never wear your light. Light hues: chaff 89.5°, lantern 54.2°, verdigris 174.0°, cobalt 284.3°, wisp 169.4°, mauve 307.0°.
3. **Shade** is key at k .36 printed over lit.
4. **Deep** is key at k .60 printed over lit, floored at L\* 16 (`#282828`). Nothing on a husk is darker than L\* 16 except its line. This is the deep cap for camera legibility.
5. **Specular** is paper, only when the lit L\* is above 70, at N·H > .96, on turning edges.
6. **Eyes** (`accent`) print flat paper. `glow` prints flat key. There is no rim and no spot at rest.
7. **Hatch:** deep band only, key, minimum duty .30, no crosshatch.

| Husk | NFT body | Lit | Shade | Deep | Note |
| --- | --- | --- | --- | --- | --- |
| Pesky Penguin #1 | `#C9722E` | `#9F8370` | `#6B574D` | `#483A35` | As a Ferryman sees it (clash rule, C\* 8): `#94857C` |
| Thugbird #1 | `#2E6FD9` | `#6C718C` | `#494C60` | `#313343` | |
| Degen Ape #4 | `#3B3F46` | `#4C5058` | `#33363C` | `#282828` | L\* lifted to 34; deep at the floor |
| Solarian #1 | `#7A5F8A` | `#73647C` | `#4E4355` | `#352D3B` | |
| SolLama #5 | `#F2EEE6` | `#E6E2DA` | `#9B9795` | `#696568` | Takes the specular |

**Reprint** (the codex, the result, and the capture's last beat): the taken husk prints in your duotone. Its NFT colours map to families like this:

- `body` → (.40/.72, .74/.46, .92/.14), with hatch;
- `trim` → bone (0/0, .24/.10, .50/.08), specular .96;
- `detail` → gilt (0/1, .34/1, .66/.90).

Ferryman body: `#985D38` / `#4E382E` / `#272024`.

The NFT's own colours appear unwashed only as **named colour chips** on its specimen plate and codex entry: up to five, from the look's named colours, each with its word (Body, Trim, Beak, Eyes, Ground).

---

## 4. Line

The line is an inverted hull: a welded copy of each mesh, pushed out along smoothed normals, faces reversed, drawn with front faces culled.

- There is no screen-space edge detection.
- Every figure has at most **two hulls**: the key line, and the outer paper knockout.

### 4.1 Colour per family

| Part | Line |
| --- | --- |
| `cloth`, `cloth_alt`, `feather`, `bone`, `metal`, `wood`, `rope`, `paper`, `spirit`; husk `body`, `trim`, `detail` | Key `#16121A`, weight ×1.0 |
| `gold` (spot-printed parts: cages, pans, bands) | Key, weight ×0.75 |
| `glow`, `accent`, `fx_glow`, eyes, husk eyes | None. Light has no outline. |
| `void`, `dark`, Hollowed void faces | None. Faces are removed from the hull. |
| The outer silhouette of every figure, everywhere (page and camera) | **Paper knockout**: paper `#ECE3CF`, 1.75 pt outside the key line |

**The paper knockout.** A second hull, of the whole figure's outer silhouette only.

- It is built from the merged, welded figure. Its push is the key line's push plus 1.75 pt.
- Its depth is pushed behind the figure's bounding sphere, so it never shows inside the figure, where an arm crosses the body for example.
- If depth bias lets a sliver through, draw it with a stencil test against the figure's pixels. That is a stencil, not a post-process.
- In the proofs it is made by dilating the render's alpha (`knock.sh`), which is the same picture.

The knockout separates every figure from any ground: key line against bright rooms, paper against dark rooms and the vigil page.

### 4.2 Width

The line is screen-constant: base width × device scale, from the hull push computed per frame from distance (`push_world = width_pt × DSF / px_per_m`).

| Where | Base width | ‰ of view height |
| --- | --- | --- |
| Reaper hero on the sanctum stage (about 400 pt tall) | 2.6 pt | 3.08‰ |
| Reaper on the roster, studio, oath and lineup (about 300 pt) | 2.4 pt | 2.84‰ |
| Husk in the hunt (200–300 pt tall) | 2.25 pt key + 1.75 pt paper | 2.67‰ + 2.07‰ |
| Husk in the codex grid (64–96 pt thumbnails) | 1.5 pt key + 1.25 pt paper | 1.78‰ + 1.48‰ |
| First-person arms and weapons | 3.3 pt (3.0–3.5) | 3.91‰ |
| Minimum weight after all multipliers | 1.3 pt | 1.54‰ |
| Interface: rules | 1.4 pt | 1.66‰ |
| Interface: icons and tab icons | 1.7 pt | 2.01‰ |
| Interface: slips, cards and inputs | 1.8 pt | 2.13‰ |
| Interface: dial rings and seal rings | 2.2–2.6 pt | 2.61–3.08‰ |
| Interface: the primary button | 2.4 pt | 2.84‰ |

On tablets and the landing page, widths scale with view height through ‰, never past 1.5× the pt values.

The proofs composite scaled renders, so their lines measure 1.8–2.6 pt depending on how far each render was scaled. The runtime line is screen-constant at the values above.

### 4.3 Weight

Per vertex of the hull:

`w = part × side × wobble × contact × paint`, then clamped by thickness.

- **`part`**, from the part's largest dimension:
  - fine (under 5 cm): 0.45;
  - interior (under 12 cm): 0.7;
  - silhouette: 1.0.

  The first-person thresholds are 2 cm and 6 cm. A floor keeps `part × base` at 1.3 pt or more.
- **`side = 1.30 − 0.62 × smoothstep(−0.35, 0.75, N·L)`.** The line is heavy (×1.30) on the side away from the light and fine (×0.68) where it is lit.
- **`wobble = 1 + 0.22 × noise3(P × 5 + seed)`.**
  - `P` is the object-space position in metres, at bind pose.
  - The seed is CRC32 of the part name.
  - The wobble is fixed to the surface: it never crawls when the phone moves.
- **`contact` = ×1.2 within 8 cm of the floor**, so a figure stands on its line.
- **`paint`** is an artist's multiplier, 0 to 2 (default 1), painted as a Blender vertex group `ink_weight`. Use it sparingly: a hood's opening heavier, a seam lighter.
- **Thickness clamp** (from Nightcut): the push may never exceed **40% of the local thickness**, measured by a ray cast inward along −N at build time. Fingers, chains and ears never balloon.

The build bakes `part × wobble × contact × paint × clamp` into one custom vertex attribute, `_INK_W`. The runtime multiplies by `side` and the screen-constant width. The cost on the GPU is one multiply-add per vertex.

### 4.4 Weld and normals

Weld vertices within 1.2 mm before pushing. Smooth the push normals over:

- 2 rings of neighbours for reapers and weapons;
- 3 rings for husks (the voxel ape needs it).

This stops the line tearing at split normals and UV seams.

### 4.5 Boil

Boil steps the wobble seed through three cached values, at 7.5 fps (every 8 frames). It boils only:

| What | When |
| --- | --- |
| The husk's line | During the 54-tick **windup** only. It is the tell. It stops dead on the first tick of the shriek. |
| The orb's line | On each capture pulse, one step and back |
| The oath card | During the press, two steps |

Nothing else ever boils.

- **Never during the shriek:** when the husk shrieks, the whole print goes still, and the only thing that moves is your spot plate, if you move.
- **Never** on the HUD or the arm.
- **Never** under reduced motion.

### 4.6 Over a camera

- The line stays key and the knockout stays paper. The line never takes the room's colour.
- In a bright room the key line carries the silhouette; in a dark room the knockout and the lit band do. One of the two always contrasts. The stand-in plates prove this, and the real-photo test must confirm it (section 16).
- Nothing on a figure drawn over the camera prints darker than L\* 16 except the line itself:
  - the husk deep floor (3.6);
  - the first-person sleeve, whose deep band is k .85 / s .16 in the hunt (Ferryman `#362D2E`, L\* 19.5).

  The arm proof predates this cap and uses .93.

---

## 5. Shade

### 5.1 Bands

Three hard bands, from N·L against fixed thresholds, plus hatching in the deep band.

| Band | Where |
| --- | --- |
| Lit | N·L ≥ `t_lit` |
| Shade | `t_deep` ≤ N·L < `t_lit` |
| Deep | N·L < `t_deep`, hatched |
| Crosshatch | N·L < `t_cross`, a second hatch set (reapers only) |

Band edges never animate. They move only because the form turns.

The ambient is the shade band: no surface is darker than its deep band, and nothing goes black.

### 5.2 Hatch

In object space, so it turns with the form and never swims:

- **First set:** `u = dot(P, (0.70, 0.30, −0.70)) × hatch_per_m`.
  - It is inked where `fract(u) < duty`.
  - `duty = min(0.55, 0.24 + 0.55 × (t_deep − N·L))`, so lines thicken as the form darkens. Husks use a minimum of .30.
- **Crosshatch,** below `t_cross`: `dot(P, (0.55, −0.20, 0.80)) × hatch_per_m × 1.1`, inked where `fract < 0.30`.
- **Pitch on screen:** choose `hatch_per_m` per model so lines land 2.6–3.5 pt apart at the usual framing.
  - Stage reaper: about 35/m.
  - Husks: about 40/m for the ape and 70/m for the penguin.
  - First-person arm: 480/m.
- **Antialiasing and moiré** (must-fix):
  - **Coverage:** `cov = clamp((duty − fract(u)) / fwidth(u) + 0.5, 0, 1)`, applied symmetrically at both edges of each line.
  - **Fade out the hatch when its pitch on screen falls under 2.5 device px:**
    - `pitch_px = 1 / fwidth(u)`;
    - `fade = smoothstep(2.5, 4.0, pitch_px)`;
    - `colour = mix(mix(deep, key, duty), hatched, fade)`.

    A distant husk shows its deep band as a flat tone, never as moiré.
  - The hatch stops at the band edge. It never crosses the key line, because the hull covers it.
- Hatch lines are key. On a reaper the spaces between them keep the deep band's spot tint.

### 5.3 Per-family tints, specular and rim

- **Shade tints.**
  - A reaper's shade carries the spot (cloth shade .80/.50): its shadow is warm with borrowed light.
  - A husk's shade is key over its ghost: the dead have no light.
- **Specular** is one hard band of paper, only on the families listed in 3.3. It appears only on turning edges, where N·H exceeds the family's threshold and N·V < 0.5 (from Nightcut). It is never the spot: a print's highlight is bare paper.
- **Rim** (reapers and arms only) prints in the spot where N·V < `rim` and N·L < 0.05: the light leaking round the far edge. Husks have none.

### 5.4 Light per scene

One directional light, **fixed to the camera** and never to the room. It is given as (right, up, toward the viewer) in camera space and normalised.

- ARKit and ARCore light estimates are ignored.
- **Intensity** in a banded shader means the thresholds: raising `t_lit` puts more of the form in shade.

| Scene | Light (r, u, t) | `t_lit` | `t_deep` | `t_cross` | Rim | Hatch pitch | Line | Effect |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Sanctum stage (hero) | (−0.80, 0.50, −0.32), behind-left | .25 | −.12 | −.55 | .34 | 2.6 pt | 2.6 pt | Backlit: most of the robe falls to shade; the spot rim and the lamp disc carry the silhouette |
| Roster, studio, oath, armory renders, landing lineup | (−0.86, 0.40, 0.32), front-left | .30 | −.10 | −.55 | .30 | 2.7 pt | 2.4 pt | The costume reads |
| Hunt husks, rift, codex and result reprints | (−0.78, 0.48, 0.40), over your left shoulder | .38 | 0 | none | none | 2.6 pt | 2.25 pt + 1.75 pt | The face and mouth are always lit, because you must read the mouth and the seals |
| First-person arms | (−0.55, 0.75, 0.35), above-left | .30 | −.12 | −.55 | .30 | 480/m | 3.3 pt | The hand always reads the same; spot rim under the sleeve |

### 5.5 Glow: the light as a printed halo

The light has a glow (must-fix), and it is printed, not additive. Every light source carries a halftone halo in the spot behind it:

- the lantern flame, the pistol's loaded chambers, the knife's edge seam, the orb's core, the bell's clapper and the Hollow's skull eyes;
- the sanctum lamp is the same halo at full scale.

The halo is drawn like this:

- **Geometry:** a camera-facing disc of radius 1.6× the source, drawn behind it.
- **Dots:** spot dots on a screen-space grid of 4.4 pt pitch, rotated 22°.
- **Falloff:** each dot's radius falls from 70% coverage at the centre to 0 at the edge.

It is one quad and one fragment function per source, and it replaces today's additive `fx_glow` and the `fx_shade` disc. Over a camera a halo sits on a key dot underprint offset 1 pt, so it reads in bright rooms.

### 5.6 GPU budget

| Item | Cost |
| --- | --- |
| Base draw, per mesh | One unlit material per family. Fragment: one N·L, one N·H, one N·V, up to two dot-fract hatches with `fwidth` antialiasing, three steps. No textures except the page grain and the reliquary portraits (16). No environment maps anywhere. That is safer for the Seeker's fragile GPU. |
| Key hull, per mesh | One draw, vertex shader only (push by `_INK_W × side × width`), flat key fragment |
| Paper knockout | One draw per figure (merged silhouette), flat paper |
| Halo | One quad per light source |
| Husk | At most 4 meshes, so at most 9 draws (4 base, 4 key hulls, 1 knockout) |
| First-person view | At most 6 meshes and 6000 triangles (`viewmodel_budget.rs`), so at most 13 draws with hulls and knockout. Hull triangles are not counted against the 6000. |

**Quality tiers.** Each lower tier drops these in order:

1. **High:** everything.
2. **Medium:** drop the crosshatch first, and raise the hatch fade threshold to 3.5 px.
3. **Low:** drop the hatch (the deep band prints its mean tone) and the specular.

No tier ever drops the key line, the knockout or the halo.

Measure on the Seeker, opening and closing hunts while reading its log (AGENTS.md). Provisional target: a hunt holds 60 fps on the Seeker at High.

---

## 6. Form

### 6.1 Global rules

1. **Big masses, few forms.** Bands and hatching punish noise and reward planes. Merge small forms that fall under 2.5 × the line width at the usual framing.
2. **The 64 pt test.** Every class and every husk must be named from its key-only silhouette at 64 pt (`cel-reaper.png`, centre strip). Weapons and the class read stand outside the body line.
3. **Hands at 1.4×** on every reaper, baked into the model. Bone hands are the only bright family on a reaper and must read at stage scale.
4. **One light part per weapon** (`glow` or `accent`), so the spot lives on the tool as well as the robe. There is **one spot band** where the hand meets the weapon (`gold` family): the reaper's light running into the tool. A bought weapon (axe, greatsword) gets the same band. That band makes it yours.
5. **One asymmetry per figure**: a weapon side, a mantle on one shoulder, a torn hem rising on one side. Symmetric figures read as icons, not characters.
6. **Faces hidden** (rule 8).
7. **Materials are families.** Name every material by its family (3.3). Keep the `COLOR_0` masks as `_MASK` (3.5).

### 6.2 Reapers

| Class | Silhouette read (64 pt) | Light placement | Form changes, in `art/blender` |
| --- | --- | --- | --- |
| Harvester | Wide hat brim and the crescent | Chaff band on the snath below the upper grip; a light seam along the blade's back | Keep the scythe blade in `bone`, the brightest shape on the figure. Straw tatters stay few and big (at most 9). Asymmetry: a brim dented on the left. |
| Ferryman | The hood and the hooked pole | The lantern flame | **Flame ×2.6** (baked), so it reads as spot through the cage. The cage is `gold` with a 0.75 line. Keep the rope coil over the arm, the best mass in the set. Raise the hem's tatter at the lower left so the robe does not merge with the floor on vigil. |
| Weigher | The long ears and the beam | Pans in `gold`; a light at the beam's fulcrum | The beam is the silhouette and must clear the head by 15% of figure height. Keep the ears. **Chain links ×2** on the balance flail. |
| Chooser | The wings and the spear | Cobalt band on the shaft; light seam in the spear tip's fuller | Cobalt is the darkest light, so lean on the bone helm and spear tip for value. **Seven primaries per wing**, fewer and larger than today. |
| Hollow | The empty shoulders | Wisp in the skull lantern's eyes | No head: the void between the pauldrons is the read. **Chain whip links ×2** or the whip becomes a dotted line at hunt scale. Keep the skull lantern big, its one bone mass. |
| Mourner | The tall veil, one column | The bell's clapper | The bell is the only shape that breaks the column. At rest its mouth faces the viewer, so its inside prints deep with the clapper's halo. |

**Spirit and humanoid forms** (studio): the same rules apply. A spirit form's lower hem fades into `spirit` (flat .94/.10) with no line at the hem's lowest 10%. It floats, and has no contact weight.

### 6.3 Husks

Every husk needs a **mouth that opens visibly**: a jaw, beak or hinge of at least 25°, because the windup is read from the mouth. It also needs four or five **seal sockets** (`seal_*` nodes) on brow, chest, flanks and back, where the seals print.

| Husk (collection) | Form changes |
| --- | --- |
| Ape (Degenerate Ape Academy) | Fewer, larger forms at the shoulders and knuckles; hull normals smoothed 3 rings. The jaw and belly are its light masses: keep them. |
| Robot (Solarians) | Already graphic; box masses print beautifully. Fewer greebles on the chest panel. The visor eye prints paper. |
| Penguin (Pesky Penguins) | The best husk for this look: pale belly, dark back, crown. No change. |
| Bird (Thugbirdz) | **Body ×1.3** against the wings, and fewer primaries. The visor is its read: keep it dark (its ghost deep). |
| Llama (Sollamas) | Legs ×1.25 thicker in cross-section, so the 2.25 pt line does not swallow them. |
| Reliquary stack (rifts) | A tower of crates. The crystalline seams between cubes are its seals: open seams print your spot hachure, closed ones paper. Its portrait faces are the one texture exception: each portrait is posterised to three key values (paper, k .55, key). Open choice 9. |

### 6.4 Weapons

All weapons are held grip at the origin, haft up +Z. Each has its family split, its spot band and one light.

| Weapon | Belongs to | Silhouette | Families | Spot band | Light part |
| --- | --- | --- | --- | --- | --- |
| `scythe` | Harvester | A crescent above the head, outside the body line | Blade bone, snath wood | Below the upper grip | Seam on the blade's back |
| `lantern_pole` | Ferryman | A hook and a hanging cage | Pole wood, cage gold, hook metal | Under the hand | Flame ×2.6 with halo |
| `balance_flail` | Weigher | A beam with two pans | Beam metal, pans gold, chains metal (links ×2) | At the grip | Fulcrum stone |
| `spear` | Chooser | A long line with a leaf tip | Shaft wood, tip bone | Mid-shaft at the hand | Fuller seam |
| `chain_whip` | Hollow | A skull with a trailing chain | Links metal ×2, skull bone | At the handle | Skull eyes |
| `bell` | Mourner | A bell on a short haft | Bell metal, haft wood | At the hand | Clapper |
| `greatsword` | Bought | A long cross | Blade metal, grip rope | At the ricasso | Fuller seam |
| `axe` | Bought | A heavy head on a short haft | Head metal, haft wood | At the grip | Edge seam |

### 6.5 First-person: arms, pistol, knife and orb

You never see your reaper in the hunt: you are it, and the arm is the only reaper on screen.

**Placement** (the proof is `ui-hunt-bright.png`):

- The right arm enters from the bottom right at about 37°, with the hand at about (300, 640) pt.
- The muzzle sits at about (232, 574) pt.
- Nothing of the arm or weapon crosses the centre column x 175–215 pt, or comes within 60 pt of the sight (195, 422).
- Camera for the form study: eye (−0.26, −0.58, 0.32) m, lens 26 mm, yaw 15°, pitch −9° (pitch 3° when dropped for the shriek).

**Arm.**

| Aspect | Brief |
| --- | --- |
| Silhouette | Bone hand and forearm out of a ragged sleeve in the class cloth. The torn cuff has five tongues of unequal length (`cloth_alt`, 7 mm solidify), the longest on the outside. |
| Proportions | The bone hand at 1.6× anatomical, so fingers are at least 9 pt wide on screen. Four metacarpals to the knuckles; three-phalanx fingers wrapped round the grip with the index on the trigger. The thumb runs along the frame's left side under the drum, radii 7–9.5 mm. Three carpals. |
| Asymmetry | The cuff's tongues, the sleeve's folds (ring-deformed on 40 sides in the study), and the thumb high and the fingers low. |
| What to cut | Inner faces; the sleeve's far side beyond the view; nails; carpal detail (merge to three); sleeve sides from 40 to 24; capsule sides from 10 to 8. The study is 50 meshes and 7832 triangles. Merge to at most **6 meshes and 6000 triangles**: right sleeve and forearm (1), right hand (1, pivot `vm_wrist_r`), weapon body (1), pistol drum (1), left arm (1), effects (1). |

**Grave pistol** (from `viewmodel_build.py`; dimensions in metres along the barrel axis).

| Aspect | Brief |
| --- | --- |
| Silhouette | A coffin-plan shroud: an irregular six-sided profile `[(−.55, .80), (.55, .80), (1, .20), (.66, −1), (−.66, −1), (−1, .20)]` swept along the barrel. Rings at y .046, .085, .125, .17 and .205, with radii .021, .030, .027, .021 and .017: widest behind the drum, tapering to a bone muzzle. Nails on the lid; a short sight; a spot band (`gold`) round the shroud's widest ring. |
| Proportions | A hex drum of radius .034 holding **six chambers** at r .02. A chamber is spot (`accent`, with halo) while loaded and `void` when spent. **The ammunition count lives on the gun.** The frame starts at y −.008; hammer, guard and trigger. The grip is four `metal` vertebrae along (0, −0.40, −1.0). |
| Asymmetry | The hammer and the nails sit right of centre; the drum's loaded chamber is at the top. |
| What to cut | Nail heads under 4 mm. Ring sides at 6 (the profile is already the shape). |

**Bone knife** (`viewmodel_parts/bone_knife.py`).

| Aspect | Brief |
| --- | --- |
| Silhouette | One long fang, tip at .268 m, guard at .05, on a vertebra handle. It shares the four-vertebra grip with the pistol, so the two read as one maker. |
| Proportions | The blade is 3.5× the handle's length; its widest point is a third of the way up. Held edge out, point forward, in the same hand placement as the pistol. |
| Status on the weapon | **Three notches cut in the spine show the dives:** spot when full, `void` when spent, one back every 180 ticks. The light seam along the edge is the knife's one light. |
| Asymmetry | A curved, fang-like belly on the edge side; the spine is straight. |
| What to cut | Serrations; any texture detail on the vertebrae. One mesh. |
| Stab path | From the rest pose toward the sight along a line ending 40 pt short of it, and back, with smear frames (11.4). |

**Capture orb** (`viewmodel_parts/capture_orb.py`).

| Aspect | Brief |
| --- | --- |
| Silhouette | An iron cup with six bone talons closed over a core of light. One talon is longer, like a thumb, which is the asymmetry. |
| Shade | Cup `metal` with deep hatch; talons `bone` with paper specular; core flat spot with a halo (5.5). The orb in flight takes a 2.2 pt key line and the knockout. |
| Proportions | Talons reach 70% up the core. The core shows between them as six spot wedges, which read at 64 pt. |
| Placement | Held in the left hand, rising from the bottom left over 10 frames while ORB is held. It never covers the ORB dial's numeral. |
| What to cut | The talons' inner faces. Three draws: cup and talons, core, halo. |

**Class weapons** in the first-person hand (the scythe and others) are shrunk and leaned in the fist as today. They take the same line, band and light rules. With a heavy weapon in hand the view is at its heaviest, so the scythe keeps 36% of its large parts' faces (`viewmodel_parts/scythe.py`).

---

## 7. Type

### 7.1 Faces

All three faces are Skribble faces, SIL Open Font License 1.1, with no Reserved Font Name. They come from the Skribble checkout the workspace already pulls in through `leptoskit_skribble` (rev `20af150`, under `packages/`). Copy each TTF with its `OFL.txt` into `assets/fonts/`.

| Role | File | Use |
| --- | --- | --- |
| **Display** | `skribble_font_recursive/assets/fonts/Skribble-Black.ttf` | The stamp voice: names (reaper, husk), stamps and chops (level, grade, UNFLINCHING, HOLD), the one primary action per screen, the two dial words (Orb, Fire), stamped figures (hit tags, result tallies). Nothing else. |
| **Text** | `skribble_font_noto_sketch/assets/fonts/SkribbleNotoSketch-Regular.ttf` | Sentences: lore, notices, sheets, help, secondary buttons, errors |
| **Figures** | `skribble_font_recursive/assets/fonts/SkribbleMonoGentle-Bold.ttf` | Every number (monospaced, so naturally tabular: the clock and ward never jitter) and every uppercase label (tracked +0.08 em: captions, tabs, tags, field labels) |

### 7.2 Scale

Seven steps, ratio about 1.25. Sizes are in px (pt) with line heights.

| Step | Size / line | Faces | Uses |
| --- | --- | --- | --- |
| s1 | 11 / 14 | Figures labels | The minimum anywhere. Tab labels, chip names, small figure lines on the page. **Never in the hunt.** |
| s2 | 12 / 16 | Figures | **The hunt minimum.** Caption slips, the husk's name, ward, ammo figure, weapon tag, field labels |
| s3 | 14 / 19 | Text, Figures | Sentences, SOL balance, notices |
| s4 | 17 / 22 | Text, Figures | The hunt clock, secondary buttons, lead sentences |
| s5 | 22 / 24 | Display | Dial words, sheet titles. The smallest size the spot may carry. |
| s6 | 30 / 30 | Display | The primary action, the UNFLINCHING stamp |
| s7 | 40 / 38 | Display | Names, HOLD, hit figures, result tallies |

**Fitted Display.** A Display word in a fixed box (a name, a chop) sets at its step and shrinks in 1 px steps to fit two lines, never below s5. This is the only way off the scale, and it is what keeps long localised names on the card. The proofs show it: "Umbra of the Slow Water" at 36, "Pesky Penguin #1" at 28, "Excellent" at 25 in its chop.

### 7.3 Numerals and symbols

- **Figures** for every number in the interface. The minus is U+2212 (−27), times is U+00D7 (×2), the separator is a middle dot (1:47 · 4 / 6).
- **Levels** are Roman numerals in Display (`theme::roman_numeral`).
- **Hit figures grow with cadence** in place of a cadence meter: 40 pt with none, 48 pt from +20%, 56 pt at +50%.
- **Glyph coverage, checked against the files:**
  - Neither Recursive face (Display, Figures) nor Noto Sketch carries ⇄ (U+21C4).
  - Noto Sketch lacks ← and → (U+2190, U+2192).

  Therefore every arrow and swap mark is a drawn icon, never a glyph. The weapon tag's swap mark is drawn in the proof.
- **Fallback chain** for scripts a face lacks:
  - Display → Skribble Noto Sketch → the platform's Noto Sans for that script, at the same size and weight class.
  - Figures → Skribble Noto Sketch → platform Noto Sans Mono.
  - Noto Sketch covers Latin, Greek, Cyrillic and Devanagari. Recursive covers Latin, including Vietnamese.

  Never synthesise bold.

**Blocker: native fonts.** leptoskit does not load bundled fonts on the native backends today. The web build loads them by `@font-face`. Until leptoskit registers the three TTFs from assets (Core Text on iOS and macOS, `Typeface` on Android), native builds show system fonts and the revamp cannot ship. This is the first engineering dependency (Appendix B).

---

## 8. Interface kit

All components are built on `leptoskit_skribble`: `SkribblePage`, `WiredBox`, `WiredCard`, `WiredCircle`, `WiredBadge`, `WiredButton`, `WiredInput`, `WiredCheckbox`, `WiredRadio`, `WiredSwitch`, `WiredProgress`, `WiredHighlight`, `WiredDivider` and `Sketch`.

- They use a **Second Ink theme** built from the tokens in section 3.
- Every instance takes a fixed `seed`. Neighbours get different seeds.
- **Pass count:** 2 passes for outlines that carry state (primary button, slips, cards, dial rings), 1 pass for rules, ticks and secondary strokes, never 3.
- **The spot plate is its own cached layer** (a separate `Vector` element painted in the spot colour), so misregistration is a translate of that layer and nothing is redrawn.

| Component | Construction | States |
| --- | --- | --- |
| **Page** (`SkribblePage`) | Vigil, key grain at 22%. No gradients, shadows, blur or rounded glass. | One state |
| **Slip** (`WiredBox`, filled) | Opaque paper; rough key outline 1.8 pt, 2 passes, 2.5 pt overshoot; turned ±2° by seed; key text | **Rest.** **Entering:** rises 8 pt in 4 held steps, 333 ms. **Leaving:** 3 stepped shrinks. **Pressed** (when it is a control): a key hatch at 60°, 4 pt gap, inside. |
| **Card** (`WiredCard`) | Raised fill; rough key outline 1.6 pt, 2 passes; square corners. On a paper sheet, a key rule box instead. | **Rest.** **Selected:** the spot plate shows 3 pt down-right behind it. **Pressed:** the spot slides into register. **Focused:** paper crop marks at the four corners (2 pt wide, 6 pt long, 4 pt outside). **Disabled:** ash label, with the reason in Text s3 underneath. |
| **Primary button** (`WiredButton` Primary), one per screen at most | Spot flood; rough key outline 2.4 pt; label Display s6 in key; hatched lip on the lower 10 pt (key at 55%); 54 pt tall, full content width | **Rest:** the spot plate sits 3 pt down-right, out of register. **Pressed:** the spot slides into register and the label drops 1.5 pt (snap spring, 120 ms). Nothing scales. **Focused:** crop marks. **Busy:** the lip's hatch translates 4 pt per held frame on twos; the label stays. **Disabled:** spot replaced by a key hatch on raised; label in ash; the reason underneath ("Needs 40 soul"). |
| **Secondary button** | Paper outline 1.8 pt, 2 passes, on vigil; no fill; label Text s4 in paper; 46 pt tall | **Pressed:** paper hatch at 60°, 4 pt gap, at 60%. **Focused:** crop marks. **Disabled:** dashed ash outline (5/5), ash label. |
| **Text link** | Text s3 in paper with a 1-pass `WiredHighlight` underline in paper | **Pressed:** the underline thickens to 2.2 pt. **Visited:** none. |
| **Destructive action** (release, sell, leave) | A secondary button with a 2.2 pt wound strike under its label, and words that say the loss ("Release into the veil") | Always confirmed in a sheet. The confirm repeats the words. |
| **Tabs** (5: Sanctum, Vault, Armory, Codex, Rifts) | Rough rule above, raised 1.6 pt. Icons in a 24 pt box, 1.7 pt, 2 passes. Labels Figures s1. Targets 78 × 56 pt. | **Inactive:** ash icon and label. **Active:** spot hachure inside the icon, a 2.2 pt spot underline 28 pt long, paper icon and label. **Pressed:** the hachure prints at once, held. |
| **Sheet** (bottom) | Full paper; key text; deckled top edge (a rough polyline); handle is a key rule 36 × 3 pt; no rounded corners; no dimming behind it | **Rising:** 4 held steps, 333 ms. **Open.** **Dragging:** follows the finger at 60 fps. **Dismissing:** a settle spring. |
| **Input** (`WiredInput`) | Paper outline 1.8 pt on vigil; Text s4; label Figures s2 above in ash | **Focused:** crop marks and a 2.2 pt outline. **Filled.** **Error:** a torn wound underline (1-pass, 2.2 pt), the message in Text s3 paper below with a drawn wound cross. **Disabled:** dashed ash. |
| **Switch, checkbox, radio** | On vigil: paper outline 1.7 pt; 44 pt target | **On:** spot hachure fill, paper knob or tick, and the word "On" in Figures s1. **Off:** ash outline, the word "Off". **Focused:** crop marks. **Disabled:** dashed. |
| **Meter** (`WiredProgress`) | A key outline 1.5 pt; what remains is hatched key at 60° (2.4 pt gap); what is gone is bare paper; the figure beside it | **Draining:** the clip moves smoothly, 60 fps. **Critical** (under 15%): the figure is struck in wound with words only when the meter means a loss (resolve), never for a husk's ward. |
| **Soul dial** | A ring of 7.5 pt key under 4 pt spot round the ORB disc; a paper notch at 40 (an orb's cost) | **Under 40:** the ring prints ash, and the dial reads "Needs 40". **Gaining:** the arc grows smoothly over 300 ms. |
| **Lamp** (sanctum) | Experience to the next level as a waxing disc: the earned part solid spot, the rest a spot halftone (4.4 pt pitch at 22°, dots r 1.05), an elliptical terminator, a rough 2-pass spot ring | **Waxing:** the terminator moves smoothly when experience arrives. **Full:** the ring doubles, and "Level V" is stamped. |
| **Rarity pips** | Registration marks (crosshair in a circle) beside a name: 1 common, 2 uncommon, 3 rare, 4 epic | Plus the card's edge and finish: plain rule and flat; deckled edge; double rule with a crosshatched corner; double rule with crop marks and a halftone ground. Plus the word in Figures s1. There is no rarity colour and no rarity outline in the 3D: two hulls only. |
| **Stamp / chop** (`WiredCircle`) | A rough spot ring, 2 passes, 2.2–2.6 pt, turned −6 to −9° by seed; Display word (fitted). On paper, the word takes a key underprint 1.5 pt down-right. | **Pressing:** 1.15× then 1.0× in 2 held frames, a sound and a haptic. **Set:** held. |
| **Tally of the taken** | Gates of five rough strokes, paper 1.6 pt; the newest stroke in spot at 2.2 pt | **New:** the stroke prints in one held frame. |
| **Colour chip** | A rough blob of the NFT's true colour in a key rule box (1.5 pt); its name in Figures s1 below | One state. The only place a husk's full colour appears. |
| **Class mark** | A registration mark in the spot (circle r 6, crosshair 18 pt) | In a rift, other reapers' marks in their own light, each on a paper name slip |
| **Notice / toast** | A slip from the top safe area, Text s3, 4 s | Enter and leave as for a slip. **Error notice:** the slip's left edge torn in wound, with words and an action. |
| **Loading** | The content's frame drawn in key hatch, static, with "Pulling the proof…" in Text s3 | No shimmer, no spinner. Under a second: show nothing. |
| **Empty** | A key-only print of what would be there, with a sentence and one action | |
| **Divider** (`WiredDivider`) | Raised, 1.4 pt, 1 pass, slight bow | |
| **Icons** | One stroke family: rough seeded paths, 2 passes, round caps, 1.7 pt, no fills except the active hachure. No pictogram fonts. Arrows and swap marks are drawn. | |

---

## 9. Screen notes

Every screen is the vigil page with grain, a top safe area, content within 20 pt side margins (max width 520 pt on tablets), and the tab bar where it applies. One primary action at most.

- **Sanctum** (`ui-sanctum.png`, route `index`).
  - The bound reaper at about 400 pt tall on the stage (backlit light), its lamp disc (centre (204, 224), r 132) behind it, a floor stroke and a hatched cast shadow.
  - Beside the name: the class mark, the name in fitted Display, the level stamp in spot, and "214/400 SOUL TO V · THE LAMP IS WAXING" in Figures s2 ash.
  - Taken: the tally, with the figure.
  - Loose in the veil: up to three husk ghosts at 46 pt, with a sentence; an empty slot is a dashed ash ring with a plus.
  - "Go hunting" as the primary, with "2 nearby →" (the arrow drawn).
  - SOL balance in Figures s3, top right.
- **Oath** (`oath`).
  - Key and paper only until the press. This is the only screen where the second ink is born (section 12.1).
  - After the press: one slip, "Sworn to the Ferryman. This light is borrowed.", and the primary "Into the sanctum".
- **Roster** (`roster`).
  - Every reaper the player holds, as cards. Each card's figure prints in its own light, and its light is named on the card. This is one of the few screens with several lights.
  - The bound reaper's card has crop marks and the word "Bound".
  - The wallet line is in Figures s2.
  - Switching the bound reaper is a primary that names the new light ("Swear to the Weigher").
- **Studio** (`studio`).
  - The reaper on the stage at the roster light, turnable by drag (60 fps, settle spring on release).
  - Controls are radios, switches and secondary buttons.
  - **Colourway picker:** three swatches (Night, Ember, Ash) showing coverage in the same light, never a hue change.
  - Omen and outfit previews are labelled in words.
  - A slip reads "Everything here is a preview".
  - Face close-up frames the hood, veil or helm, never a face.
- **Armory** (`armory`).
  - Weapons as cards: each weapon rendered at the roster light, key line, spot band, and its share of the hit in Figures.
  - The equipped weapon has the selected card state.
  - Arts carry rank pips.
  - Outfits show their key marks on a swatch of cloth.
  - Buying is the primary; equipping is selection.
- **Codex** (`codex`).
  - A grid of the taken, reprinted in your duotone at 96 pt with 1.5 pt lines.
  - A tap opens the specimen plate (the result card without the tallies), with the chips and the "Taken" date.
  - Not-yet-taken species show as key-only silhouettes with their collection name.
- **Vault** (`vault`). One ledger of rows:
  - figure columns in Figures s2;
  - a husk thumbnail per row: ghost wash when loose or held, reprint when absorbed;
  - status words (Held, Loose, Absorbed, Mint);
  - "Buy" is the primary;
  - "Release into the veil" is a destructive action;
  - the stand-in notice is a paper slip at the top.
- **Rifts** (`rifts`, `rift/[site]`).
  - The list shows sites as cards with city and collection; "Enter" is the primary on an open rift.
  - Attunement text in Text s3.
  - In the rift hunt, the HUD is the hunt's. Each other reaper in the circle appears only as a registration-mark tag in its light, pinned over its position, with its name on a paper slip.
  - Shades are key-only, with an ash mark and the word "Shade".
  - The reliquary stack prints as in 6.3.
- **Hunt and practice** (`hunt/[sighting]`, `practice`): section 10.
- **Result** (`ui-result.png`, part of the hunt route). A specimen plate on paper, turned −1.4°, with a rough 2.2 pt key edge and registration marks in your light at the four corners. On it:
  - the reprinted husk on a hatched shadow;
  - "Specimen 38";
  - the name in fitted Display;
  - "TAKEN 25 NOV / WITH 1:47 TO SPARE";
  - "Its colours" with up to five chips.

  The grade chop is pressed half on the plate's edge. Below the plate:
  - XP, Anima and Soul as Display s7 tallies (Soul in spot);
  - the tally;
  - receipt rows in Figures s2 ash;
  - "Hunt again" (primary) and "Back to the sanctum" (secondary, Text s4).

  Practice results add the longest run and the parries, and offer the same stray again.
- **Landing** (`reapers_landing`, web only).
  - The vigil page, in a 520 pt text column, with the type scale ×1.25 on viewports wider than 1024 px.
  - **Hero:** the Ferryman backlit before its lamp, with the name and one primary ("Play").
  - **Reapers:** the six lights in a lineup, each named; the landing may show all six, like the roster.
  - **Husks:** the ghost row and the three printings.
  - **Hunt:** the two hunt proofs as phone frames.
  - **Orb:** the capture beats as six held prints.
  - **Chain and Art:** text sections.
  - **Closing:** primary again.
  - The colophon credits the music (Kevin MacLeod, CC BY 4.0, unchanged) and the faces (SIL OFL 1.1).

---

## 10. Hunt HUD

### 10.1 Layout

Coordinates are pt on a 390 × 844 view. The top group anchors to the top safe area (47 pt on the proof phone); the thumb group anchors to the bottom safe area (34 pt). Nothing else scales: larger screens add space between the groups.

| Element | Position and size | Construction |
| --- | --- | --- |
| **Top slip** | x 16–374, y 52–108 | Paper slip, 2 passes. Leave arrow at its left (target 16–60 × 52–108). The husk's name and rarity pips at x 50, Figures s2. The clock in Figures s4, right-aligned at x 284–360. "Ward" label, Figures s2, at (24, 83). The ward meter at x 68–204, y 86–97, with its figure at x 212. Six resolve marks at x 262 + 17i, y 91, r 5.5. |
| **Sight** | Centre (195, 422) | Four rough arms from r 25 to r 42, paper 2.6 pt on a key underlay 5.2 pt. |
| **Seals** | On the husk's `seal_*` sockets, projected | Closed: a paper ring r 6, 1.6 pt, on key. Open: see 10.3. |
| **Caption slip** | Centred 12 pt under the husk's screen bounds, clamped above y 620; 166 × 26 pt | Figures s2 in key; turned −2°; one line |
| **State ring** | On the husk's floor shadow | See 10.3 |
| **ORB** | Centre (66, 768); disc r 38; soul dial r 48; hit area 96 pt | A paper disc with a key 2.4 pt ring; "Orb" Display s5 and the soul figure Figures s2, in key |
| **FIRE** | Centre (322, 764); drawn r 44 (88 pt); hit area 96 pt, extending to the screen's right and bottom edges | A key disc with a paper ring 2.6 pt; a live spot arc at r 50; "Fire" Display s5 in paper; the weapon's figure in Figures s2 ("4 / 6", "Dives 2", "In reach", "Throws 1") |
| **Weapon tag** | x 154–236, y 786–812, turned −2° | A slip with the weapon's name in Figures s2 and a drawn swap mark. A tap cycles. A long press fans the rail's four slips upward at 30 pt spacing. The one in hand has a spot underline. The 21-tick draw shows as the tag's hachure filling. |
| **Arm** | Hand about (300, 640); muzzle about (232, 574) | Section 6.5 |
| **Off-screen arrow** | At the screen edge nearest the husk, inside the safe area | A paper slip arrow with the turn in words ("Turn left") |

Measured paper budget: 10.7% of the screen outside a shriek and 13.6% in a shriek, against limits of 12% and 15%.

### 10.2 What is removed

From today's hunt screen:

| Removed | Because |
| --- | --- |
| The prompt line | The caption slip says it |
| The top banner | Merged into the top slip |
| The sight tag | The sight says it by shape |
| The four-chip weapon rail | One tag; long press for the fan |
| The cadence meter | It became the hit figure's size |
| Colour-coded state rings and the crosshair "taking the husk's colour" | Shapes and words (10.3) |
| Ammo ticks around FIRE | The count lives on the gun's drum, with the figure on FIRE as the words |
| Paper bursts behind hit numbers | A tag and a scrawl (11.5) |
| Additive muzzle flashes, glows and dark discs (`fx_glow`, `fx_shade`) | Printed halos and key underprints (5.5) |

### 10.3 What becomes part of the creature

These are drawn on or anchored to the husk, in world space, at 60 fps:

- **Seals.** An open seal has four parts. Legibility comes from the key and paper pair, at least 3:1 against any room, never from the spot.
  - a paper disc of r 19;
  - a key outer ring of 2.2 pt;
  - a key inner ring at r 13.5, 1.2 pt;
  - between them, a spot hachure clipped to r 13 (2.6 pt gap at 60°, 1.7 pt).
- **The state ring** on the floor shadow, by shape. The words go on the caption slip.

  | State | Ring | Caption |
  | --- | --- | --- |
  | Nothing coming | A hatched key floor oval | "Seal open · strike", or none |
  | Windup | The oval's outline goes dashed, the dashes lengthening each 18 ticks; the line boils | "It draws breath · get still" |
  | Shriek | The HOLD ring at the sight (10.4) | "It shrieks · keep still" |
  | Charge | A sawtooth ring, teeth in, closing on the husk over 66 ticks | "It charges · swing" |
  | Stunned | Two concentric paper rings | "Stunned · dive in" |
  | Weak or broken | The oval prints in your spot (your light can take it); when broken, ticks round it count the window down | "Weak · orb it" |

- **The struck part** prints in your spot for 4 frames: the light touching the dead.
- **The splat and the scrawl**, pinned to the hit point in the husk's local space (11.1).
- **The shriek strokes:** jagged paper strokes on key, thrown from the mouth to the sides, never across the face.
- **The capture strip** (12.4).

### 10.4 The HOLD ring and the drift

**The ring.**

- Centred on the sight, R 64.
- Key 9 pt under paper 3.4 pt.
- The spot plate is a 3.6 pt spot ring on its own layer.
- Twelve countdown ticks sit outside it (r 73–83, paper on key). One goes out every 5.5 ticks of the 66.
- The sight stays inside it, shortened to r 6–14.
- The word goes on its own slip below the ring, never on the face:
  - slip x 90–300, y 530–596;
  - "HOLD" in Display s7;
  - "It shrieks · keep still" in Figures s2.
- When the husk is low on screen, the ring sits on its chest, and the slip moves above the husk if it would cover the mouth.

**The drift.** `f` is flinch speed, `0.42 + 0.035 × resilience` rad/s, and `ω` is the phone's angular speed, low-passed at 4 Hz.

- `offset = 6 pt × clamp((ω − 0.4f) / (0.6f), 0, 1)`, in the direction of the turn.
- **Dead zone:** no drift below 40% of the flinch speed, so a steady hand sees a perfectly registered print.
- **Return:** a critically damped spring of about 120 ms.
- **The tag:** a slip reading "You are moving" appears pinned to where the plate drifted, once the offset passes 3 pt.

**Comfort cap.**

- Misregistration is only ever a translate of a cached plate.
- It never exceeds 6 pt during the hold, or 10 pt on a flinch, held at most 300 ms.
- It never applies to text below s5.

**During the shriek:**

- nothing boils;
- the arm has dropped (12.3);
- FIRE is disabled: a scrim disc, a dashed ash ring, and "WOULD FLINCH" in Figures s2 ash;
- ORB stays live, because holding it is not a flinch.

---

## 11. Effects, with timing

Frames are at 60 fps. *Held* means the drawing does not change between the frames given.

### 11.1 Ink splat (a hit on the husk)

| When | What |
| --- | --- |
| f1 | Your ink lands at the hit point: a spot blob of r 21 on a seal (r 12 on the body) with 3 to 6 seeded satellites (r 2.4–6) thrown away from the shot. Its key edge (1.6 pt) prints 2 pt left and 1.5 pt down, out of register. |
| f1–f5 | Held. |
| f6 | The key edge springs into register (snap, 120 ms). |
| to 400 ms | Pinned in the husk's local space; it moves with the husk at 60 fps. |
| 400–650 ms | Wipes off in 3 bites on twos: coverage 75%, 45%, then gone. |

### 11.2 Impact frame (hit-stop on the husk only)

- **How long:** the husk's animation freezes for 3 frames on a body hit, 5 on a seal, and 6 on a break or a parry.
- **What keeps moving:** the camera image, the HUD and the arm. An AR camera is never frozen.
- **What prints during the hold:**
  - the struck part prints in your spot (4 frames);
  - the husk's line weight is ×1.25, a pressed frame;
  - the sight's arms step out 3 pt (2 frames).

### 11.3 Speed lines

They are drawn over the camera as paper 2.2 pt on a key 4.8 pt underlay, 40–90 pt long, and never cross the 60 pt circle round the sight.

| Event | Lines | Timing |
| --- | --- | --- |
| **Dive** (knife or class weapon, 13 ticks in) | 6 lines radiating from the screen edges toward the husk | Held from f0 to f12, then gone in one step |
| **Parry** | 3 spot lines struck through the husk, plus two paper rings | Held 4 frames |
| **Charge** | None on the husk; its sawtooth ring is the motion | |
| **Art throw** | Cinder: 2 lines along the arc. Rime: 1 straight line. Gale: 3 curved lines. | Each held 4 frames at release |

### 11.4 Smear

A smear is a ribbon mesh from the weapon's tip (`muzzle`, `knife_tip`, a blade tip) through its last three positions:

- filled key;
- edged in paper, 1.25 pt;
- no line.

| Motion | Smear |
| --- | --- |
| Pistol recoil (muzzle rises 7° and comes back 10 pt) | 1 frame |
| Knife stab | 2 frames |
| Class-weapon swing | 3 frames |

Smears never cross a slip or the 60 pt circle round the sight. A smear that would is clipped there.

### 11.5 Hit figure (the tag)

**Construction.**

- A paper slip of 86 × 50 pt with a rough key outline, 1.8 pt, turned −5° ±3° by seed.
- The figure in Display s7 (40 / 48 / 56 by cadence) in key, with "−27" using the true minus.
- A spot underline: 3 pt, 2 passes.
- On a seal, "Seal ×2" in Figures s2 on a small paper chip below.

**Position.**

- 60–130 pt up and right of the hit point. The proof's tag sits 67 pt right and 122 pt up.
- Clamped inside the safe area, outside the sight's 60 pt circle, and off the caption slip.

**Timing.**

| When | What |
| --- | --- |
| f2–f3 | Appears at 1.15× |
| f4 | 1.0×, held |
| to 600 ms | Rises 18 pt (smooth, settle spring) |
| 600–850 ms | Leaves in 3 stepped shrinks on twos |

**Stacking.**

- A new hit within 600 ms stacks its tag 22 pt above the last.
- At most 3 tags; the oldest leaves at once.

### 11.6 The rest

| Effect | Construction and timing |
| --- | --- |
| **Muzzle burst** | A 10-point star (radii 17 and 6 pt) in spot with a 1.8 pt key outline, at the muzzle, for 2 frames (f0–f1); then the halo of the next chamber |
| **Scrawl** | Three fast key strokes, 2.4 pt, 1 pass, across the splat; appears whole at f2, held, and leaves with the splat |
| **Stepped vignette** (windup and shriek) | Three hard bands round an open oval about the husk (its screen bounds × 1.25): outer key tint 55% plus hatch (3.2 pt gap at 60°, and 4.4 pt at −30°); middle 38% plus hatch 4.2 pt; inner 20% plus hatch 7 pt. They come in by bites at windup ticks 0, 18 and 36, outermost first, and lift in reverse at 0, 6 and 12 ticks after the shriek. |
| **Wound bruise** (struck by a charge, or flinched) | A wound hatch at the screen's four edges, at most 18% of the area, for 6 frames, then gone in 3 stepped bites. Always with words on the caption slip ("Struck · −1 resolve"). |
| **Resolve lost** | The mark is struck through in wound (2.2 pt, 2 passes) on the paper top slip, in one held frame |
| **Wall marks** (a missed shot ray-tested against the room) | Printed decals on the measured surface, 1.8 m reach for melee: bullet chip (a key star with a paper knock); blade cut (a key slash); spear puncture (a key ring with a hatch); blunt crack (3 key lines); bell ripple (2 rough rings); Cinder scorch (key crosshatch); Rime frost (paper hatch on key); Gale dust (3 rough arcs). They fade in 3 stepped bites after 4 s. |
| **Halo** | Section 5.5; steady, and never pulsing except on the capture pulses |

---

## 12. Five moments, beat by beat

### 12.1 The oath draw (the shuffle binds you; you do not choose)

| ms | Beat |
| --- | --- |
| before | The whole game is key and paper. Onboarding has no spot at all. |
| 0–500 | Six face-down plates (key-only cards with registration marks) slide into a row, held on twos. |
| 500–1700 | They shuffle in four stepped passes of 300 ms; each card jumps between held positions with no tweening. |
| 1700–2000 | One card stops in front; the other five drop off the bottom in 3 held steps. |
| 2000–2170 | The press: the card flips in two held drawings, edge-on and then face. The line boils two steps. |
| 2170–2500 | The figure prints in key only: line, bands and hatch, no spot. |
| 2500–2830 | The spot plate slides in from the right 12 pt out of register and lands in register over 4 held frames, with a stamp sound and a heavy haptic. The reaper turns from a grey print into its duotone: **the first time the second ink appears in the game.** |
| 2830–3500 | The lamp prints behind it as halftone only (no soul yet). The name sets two letters per held frame in fitted Display. |
| 3500 | A slip rises: "Sworn to the Ferryman. This light is borrowed." The primary is "Into the sanctum". |

### 12.2 A hit

| Frame | Beat |
| --- | --- |
| f0 | The thumb lands on FIRE and the shot leaves at once. Muzzle burst (2 frames). The pistol recoils (smear 1 frame) and its drum starts a 60° turn over 6 frames (smooth); the spent chamber turns `void`. Light haptic. |
| f1 | The splat lands. The struck part prints in spot (to f4). Hit-stop on the husk only (3 frames body, 5 frames seal). The sight's arms step out 3 pt (to f2). Medium haptic on a seal. |
| f2 | The scrawl and the figure tag appear (1.15×). |
| f4 | Tag at 1.0×. On a seal, the seal prints shut: a paper ring, the spot gone (rules: shut for 44 ticks). |
| f6 | The splat's key edge springs into register. |
| to 600 ms | The tag rises 18 pt; the splat wipes off from 400 ms. |
| 850 ms | All gone. The arm has settled (arm spring, 180 ms). |

### 12.3 The unflinching hold

| Ticks | Beat |
| --- | --- |
| Windup 0 | The husk stops and turns on you; its mouth starts to open. The floor oval goes to dashes, the line boils, the vignette's outer bite prints. Caption: "It draws breath · get still". |
| Windup 18 / 36 | The middle and inner bites. |
| Windup 24–42 | The arm drops 120 pt out of the way (settle spring), so the screen's centre is clear. |
| Shriek 0 | The boil stops: the print goes still. The HOLD ring prints at the sight with its slip. FIRE goes to "Would flinch". The first 7 ticks are forgiven. |
| Shriek 0–66 | The spot plate drifts only if you move (10.4). One countdown tick goes out every 5.5 ticks. |
| Shriek 66: **held** | The ring prints solid spot for 3 frames. The **UNFLINCHING** stamp presses at the sight: Display s6 on a paper slip inside a spot chop ring, turned −6°, 1.15× then 1.0× in 2 held frames, with a stamp sound and a two-pulse success haptic (40 ms apart). Every seal in view opens, one per frame, held. "+10 soul" appears on the ORB dial and its arc grows over 300 ms. A ready art gets its spot tick. **Success is louder than failure.** |
| +0 / 6 / 12 ticks | The vignette lifts in 3 bites. |
| +14 ticks | The arm returns (settle spring). |
| +900 ms | The stamp leaves in 3 stepped shrinks. |
| **Flinch** instead | Quieter: the spot plate snaps 10 pt in the direction of the turn and holds 300 ms. The ring tears open at the top (one held frame). A resolve mark is struck in wound. The caption reads "Flinched · −1 resolve". One light haptic tick. FIRE stays disabled until the shriek ends. |

### 12.4 Capture

The ticks are the rules', from letting go of ORB.

| Ticks | Beat |
| --- | --- |
| While ORB is held | The left hand rises with the orb over 10 frames. A paper ring with a key outline shrinks from the screen edge onto the husk over 72 ticks, smoothly, because it is timing. Two tick rings mark its thirds: Plain (outer), Great (middle) and Excellent (inner), so the grade can be seen as well as felt. |
| 0–27 | The orb flies: a spot core with a key line and knockout. Its talons' hatch turns in stepped quarter turns; its halo goes with it. |
| 27 | It strikes and the roll is drawn. The husk's plates lift off in order: the hatch, then the shade band, then the line, 2 frames apart. |
| 27–48 | The husk, now a paper silhouette, is drawn into the orb as a tapering paper strip. |
| 48–66 | The orb drops to the floor and lands with a held squash frame. |
| 66, 114, 168 | Pulses. On each, the orb's key and spot plates knock 2, 4 and then 6 pt out of register; its line boils one step; a light haptic; then it snaps back (snap spring). |
| A break | It comes on the pulse the rules say. The orb's outline tears along a rough line, the husk prints back in key, and it stands stunned (two paper rings). |
| 222 | The result. |

### 12.5 The result

| ms | Beat |
| --- | --- |
| 0–333 | The specimen plate rises in 4 held steps. |
| 333–500 | The husk prints on it in key only. |
| 500–833 | Your spot plate slides in over it and lands in register over 4 held frames: the dead now wears your ink (the reprint). |
| 833 | The chips print, one per held frame. |
| 1000 | The grade chop presses on the plate's edge, 1.15× then 1.0× in 2 held frames, with a stamp sound and a medium haptic. Plain has 1 notch, Great 2, Excellent 3. |
| 1000–1800 | XP, Anima and Soul count up on twos. The tally gains its newest stroke in spot. |
| 1800 | The receipt rows and the buttons appear. |
| **Escaped** instead | The plate stays key-only and the husk's line goes dashed. "It slipped away. It is still loose." in Text s3. The lost resolve marks are struck in wound, with their words. No chop. |

---

## 13. Motion principles and timings

1. **Fluid where things move.** Everything anchored to the camera or the world, and everything that is timing, runs at 60 fps on springs:
   - husks and arms;
   - the sight;
   - tag positions;
   - the orb and charge rings;
   - the drift and its return;
   - sheets being dragged;
   - the studio turn.
2. **Held only at impact.** Hit-stops of 3 to 6 frames, held stamps, held splat edges.
3. **Printed decoration is on twos (12 fps).** This covers stamps, slips rising, sheets rising, plates sliding into register, the shuffle, the vignette's bites, count-ups and stepped exits. The rhythm of a flip book of prints keeps the interface's motion clearly apart from the room's.
4. **Ink moves by transform only.** Boil is the only redraw, and it uses cached seeds.
5. **Nothing idles on the interface.** The reaper on the stage breathes (its model animation, 60 fps). The lamp, the grain and the HUD are still.

**Springs** (mass 1; stiffness k, damping c):

| Name | k | c | Damping ratio | Settles in | Use |
| --- | --- | --- | --- | --- | --- |
| `snap` | 1537 | 78.4 | 1.0 | about 120 ms | Registration returns, button press, orb pulse return, splat edge |
| `arm` | 683 | 44.4 | 0.85 | about 180 ms | Viewmodel recoil and sway, weapon draw |
| `settle` | 587 | 36.4 | 0.75 | about 220 ms | Tag rise, arm drop and return, sheet release, studio turn release |
| `carry` | 216 | 29.4 | 1.0 | about 320 ms | Screen transitions (a page slides 24 pt and the next prints in) |

**Durations:**

| Motion | Duration |
| --- | --- |
| Screen transition | 280 ms |
| Sheet or slip rise | 4 held steps, 333 ms |
| Stepped exit | 3 held steps, 250 ms |
| Count-up | at most 800 ms |
| Weapon draw | 21 ticks (rules) |
| The arm's idle sway | 0.6 pt at 0.25 Hz, following the phone's motion with the `arm` spring |

---

## 14. Reduced motion

This follows the OS setting and the in-game "Steady print" switch.

**Dependency:** the native shells (iOS `UIAccessibility.isReduceMotionEnabled`, Android `ANIMATOR_DURATION_SCALE` of 0 or the remove-animations setting) must report the setting to the app, as the web build does through `prefers-reduced-motion`.

| Normally | Under reduced motion |
| --- | --- |
| Misregistration drift on the HOLD ring | No movement. As ω passes 40% of the flinch speed, the spot ring thins and turns to dashes in 3 steps, and "You are moving" appears. Same information, no motion. |
| Flinch snap | The ring tears and the words appear. The plate does not move. |
| Primary button at rest 3 pt out of register | In register. A press darkens the hatched lip instead. |
| Boil (windup, orb pulses, oath) | Never. The windup is told by the dashed oval and the caption. |
| Hit-stop | Kept (it is a pause, not motion). |
| Struck part in spot for 4 frames | Kept. Cut to 2 frames if the OS asks for reduced flashing. |
| Splat, scrawl, tag | Appear in place, held, then disappear in one step. The tag does not rise or scale. |
| Speed lines and smears | None |
| Muzzle burst | One frame |
| Stamps, chops, slips, sheets rising | Appear in place, without slide, scale or squash |
| Stepped vignette | Appears in one step at windup tick 0 and lifts in one step |
| Arm drop in the shriek | Hidden in one step instead of lowered |
| Shuffle (oath) | No shuffle. The six cards print in place, then the chosen one is pressed: the key print, then the spot plate appears in register in one step. |
| The orb's flight, draw-in and pulses | Flight kept (it is aim feedback) as a straight line with no spin. Draw-in becomes a cut. Pulses become the words "One", "Two", "Three" on a slip by the orb, with haptics. |
| Count-ups | The final figure, at once |
| Screen transitions | A cut |
| Studio turn | Turns only while dragged; no release spring |
| Wall-mark fades | One step after 4 s |

---

## 15. Accessibility

### 15.1 Contrast

WCAG ratio, foreground on ground, computed by `palette.py`.

| Foreground | paper | vigil | raised | scrim | key |
| --- | --- | --- | --- | --- | --- |
| paper `#ECE3CF` | n/a | **10.28** | **7.97** | **11.51** | **14.49** |
| key `#16121A` | **14.49** | 1.41 | 1.82 | 1.26 | n/a |
| ash `#A39C93` | 2.13 | **4.83** | 3.75 | **5.41** | 6.81 |
| wound `#B92A20` | **4.82** | 2.13 | 1.65 | 2.39 | 3.01 |
| chaff `#F4CD50` | 1.20 | 8.56 | 6.63 | 9.58 | 12.06 |
| lantern `#F07428` | 2.27 | 4.52 | 3.50 | 5.06 | 6.37 |
| verdigris `#2FBD9F` | 1.85 | 5.56 | 4.31 | 6.23 | 7.84 |
| cobalt `#477FE4` | 3.03 | 3.39 | 2.63 | 3.79 | 4.78 |
| wisp `#A8F2D8` | 1.01 | 10.22 | 7.92 | 11.44 | 14.40 |
| mauve `#CDB2F6` | 1.46 | 7.06 | 5.47 | 7.90 | 9.95 |

What the table allows:

| Pair | Ratio | Allowed |
| --- | --- | --- |
| Paper on any dark ground; key on paper | 7.97 to 14.49 | All text |
| Ash on vigil and on scrim | 4.83 and 5.41 | All text. Never on raised under s5 (3.75), never on paper. |
| Wound on paper | 4.82 | Text |
| Wound on any dark ground | 1.65 to 3.01 | Never text; marks only, always with paper words beside them |
| Key on a light | 4.78 to 14.40 | All text |
| A light on dark grounds | 2.63 to 11.44 | Display s5 and above only. On raised, never text: cobalt is 2.63. |
| A light on paper | 1.01 to 3.03 | Only with key (3.2) |
| Non-text | at least 3:1 | Seals, dial rings, the sight and the knockout rely on the key and paper pair (14.49), never on the spot |
| Hunt text | 14.49 | Every hunt word is key on a paper slip, whatever the room |

### 15.2 Colour vision

- The six lights keep at least 13.5 ΔE2000 apart under every simulated deficiency, and at least 6.5 L\* apart.
- Wound keeps at least 18.5 from every light.
- No state relies on hue: rings are shapes, wound is a strike plus words, and lights are named wherever two meet.

### 15.3 Targets, text and input

- **Targets:** at least 44 pt. FIRE and ORB have 96 pt hit areas; tabs are 78 × 56.
- **Text:** at least 11 pt on the page and 12 pt in the hunt.
- **Dynamic type:** sentences (Text) scale with the OS text size up to 200%. Labels and HUD figures scale up to 130%, and the HUD's slips grow to fit.
- **Mirror thumbs:** a setting, off by default, swaps ORB and FIRE for left hands. The arm stays on the right.
- **Haptics:**

  | Event | Haptic |
  | --- | --- |
  | Hit | Light |
  | Hit on a seal | Medium |
  | Parry | Medium |
  | UNFLINCHING | Two pulses |
  | Flinch | One light tick |
  | Capture pulses | Light |
  | Capture | Heavy |
  | Oath press | Heavy |

  Haptics have their own switch.
- **Sound:** every sound has a visual. The shriek's audio always has its caption slip.
- **Screen readers:**
  - Every control has an `en.arb` label.
  - The HUD announces state changes from the caption slip.
  - Resolve is read as "Resolve 5 of 6".
  - The ward is read as a percentage.

### 15.4 Tests the interface must carry

| Test | What it proves |
| --- | --- |
| **W** (wound) | The wound component's type has no constructor without a shape and a localised message. A test enumerates every wound use in the app and asserts both. It also renders each wound state in greyscale and under deuteranopia, and asserts the shape token is present. |
| **C** (contrast) | A unit test over the token table asserts each allowed pair above its threshold (4.5 for text, 3.0 for Display s5+ and non-text), and fails if a token changes without the table. |
| **L** (lights) | Replaces `the_light_is_the_interface_light`: one light per class, matching 3.2. Every pair at least 6.5 L\* apart and 13.5 ΔE2000 apart under each simulation. No light within 15 ΔE of wound. |
| **P** (paper budget) | The hunt HUD's slips, discs and tags cover at most 12% of the view, and 15% in the shriek state. |
| **R** (reduced motion) | Every animation entry in the motion table has a reduced form, and the app reads the native setting. |

---

## 16. Open choices, with defaults

| # | Question | Default |
| --- | --- | --- |
| 1 | Husks in the ghost wash (hue kept, C\* 16) or in key only? | **Ghost wash.** It keeps each collection recognisable, which the NFT game needs; key-only remains a renderer switch (`husk_tint = "key"`). |
| 2 | Adopt the six new lights (one per class, whatever the colourway) in place of today's three accents per class? | **Adopt.** Chooser red goes to cobalt, because it collided with wound; colourways become coverage, and omens become treatment. |
| 3 | The vigil page always, or also a day (paper) page? | **Vigil always.** Paper appears only as objects, and there is no theme switch. |
| 4 | Lock the look before testing over real photographs? | **No.** Lock only after the husk, slips and HOLD ring read over 30 real photographs (bright kitchen, dim bedroom, mid-grey office, outdoors at noon, night street), checked on a Seeker and an iPhone. |
| 5 | A darker paper for slips over very bright rooms? | **No.** One paper, `#ECE3CF`, everywhere; the key outline separates it. |
| 6 | Display for control words? | **Only the primary action, the two dial words and HOLD.** Every other control uses Text or Figures labels. |
| 7 | Allow paper up to 15% of the screen during the shriek (12% otherwise)? | **Yes.** The HOLD slip needs it; measured 13.6%. |
| 8 | Meme animations (collection emotes) on husks? | **Off in hunts.** Allowed later in the codex only, never mocking the collection. |
| 9 | One texture exception: the reliquary stack's portrait faces, posterised to three key values? | **Allow it,** for that model only. |
| 10 | Mirrored thumbs for left hands? | **Available, off by default.** |

---

## Appendix A: changes to the repository

Each item is its own task.

1. **leptoskit native fonts.** Load bundled TTFs on iOS, macOS and Android. This blocks shipping (7.3).
2. **Native reduced motion.** The guest reports the OS setting (14).
3. **`leptoskit_3d` print material.** Accept a custom unlit material per family with the uniforms below, plus a hull pass with front-face culling and a per-vertex push from `_INK_W`. Today `look.rs` can only say tint, glow and rim.
   - `(k, s)` × 3 bands;
   - `t_lit`, `t_deep`, `t_cross`;
   - `spec`, `rim`, `hatch_per_m`;
   - the light in camera space;
   - the spot.
4. **Art build.** Write `_INK_W` (4.3) and `_MASK` (3.5) per vertex. Bake the proofs' overrides into `art/blender`:
   - hands ×1.4;
   - the lantern flame ×2.6;
   - the bird's body ×1.3;
   - chain links ×2 (Weigher's flail, Hollow's whip);
   - seven primaries per Chooser wing;
   - llama legs ×1.25;
   - the Ferryman's hem tatter;
   - fewer ape forms.

   Then rebuild with `cargo xtask build-art`.
5. **`crates/reapers_app/build.rs`.** Keep `_MASK`; go on stripping `COLOR_0`.
6. **`theme.rs` and `render/look.rs`.**
   - Replace `accents` with one light per class (3.2).
   - Colourways become coverages, omens treatments, outfits marks (3.5).
   - Update `the_light_is_the_interface_light` and `an_eclipse_darkens_the_cloth_and_rims_with_the_light` to the new rules (tests L and C).
7. **`docs/gameplay.md`.** "What the screen shows" describes coloured rings (white, amber, red, blue, gold), a crosshair taking the husk's colour, and ammo ticks above FIRE. Rewrite it in shapes and words (10.3). This is a change to how the rules are shown, not to any rule or number.
8. **Viewmodel.** Merge the arm study to at most 6 meshes and 6000 triangles (6.5). Add the knife's dive notches and the drum's chamber states. Replace `fx_glow` and `fx_shade` with halos and key underprints (5.5).
9. **Interface.**
   - Build a Second Ink `leptoskit_skribble` theme from section 3.
   - Implement the kit (8), the HUD (10) and the effects (11).
   - Add tests W, C, P and R (15.4).
   - Move every new string into `en.arb`.

## Appendix B: grafts, and where each must-fix is answered

**Spine:** Second Ink. It took two of the three judges (play and system) and the top total (19, against 18 and 16.5). It is the only pitch whose look is also a mechanic: registration is stillness.

**From Ink on Bone:**

- variable line weight by part class and light side (4.3);
- the outfit axis (3.5);
- the tally of the taken (8, 9);
- the result as a specimen plate (9, 12.5);
- a stage light fixed to the camera (5.4);
- wound as tears against the light's circles (2, 11.6);
- the weapon table (6.4);
- the pass-count rule for rough strokes (8);
- the UNFLINCHING stamp, and success louder than failure (12.3);
- hit-stop on the husk only (11.2).

**From Nightcut:**

- the two-value edge, as the paper knockout (4.1);
- the specular on turning edges only (5.3);
- the 40% thickness clamp (4.3);
- the hunt light fixed to the camera (5.4);
- no boil during the hold (4.5);
- the thumb layout (10.1);
- the arm dropping in the shriek (12.3);
- "Would flinch" (10.4);
- husks keep their hue, as the ghost wash (3.6);
- the stepped vignette (11.6);
- the scrawl on a hit (11.6).

**Must-fixes, and where each is answered:**

| Must-fix | Answered in |
| --- | --- |
| A real night answer | The vigil page and the knockout (3.1, 4.1) |
| All six classes proved | `cel-reaper.png` |
| Give the light its glow | 5.5 |
| No luminance switch | Rule 9 |
| Husk forms | 6.3 |
| A 3D arm | 6.5, `cel-reaper.png`, both hunt proofs |
| Hatch contour and moiré | 5.2 |
| The native-font blocker and glyph fallback | 7.3 |
| Display for names and stamps | 7.1, choice 6 |
| GPU cost | 5.6 |
| Thumb layout and the 96 pt FIRE | 10.1 |
| Seal construction at 3:1 or better | 10.3 |
| Hunt text at 12 pt or more | 7.2 |
| HOLD off the face | 10.4 |
| Drift dead zone and the 120 ms return | 10.4 |
| Wound shape plus words, with a test | Rule 2, test W |
| Deep cap and a real-photo test | 3.6, 4.6, choice 4 |
| Paper budget | Rule 10, test P |
| Success louder than failure | 12.3 |
| Reduced motion reported by the native guest | 14 |
| The orthogonal axes | 3.5 |
| Two hulls, with rarity moved to pips, card and caption | 4, 8 |
| Three typefaces | 7.1 |
| A scrawl and a tag in place of the burst | 11.5, 11.6 |
| Misregistration on cached plates, with a comfort cap | Rule 11, 10.4 |
| The look and the docs updated | Appendix A |
| Other reapers' lights only as tags in a rift | Rule 1, 9 |
