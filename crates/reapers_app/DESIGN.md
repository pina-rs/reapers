---
name: Reapers
description: Second Ink, a two-plate relief print on a dark page, whose second ink is the bound reaper's own light.
colors:
  paper: "#ECE3CF"
  ash: "#A39C93"
  raised: "#454042"
  vigil: "#342F33"
  scrim: "#2B272C"
  key: "#16121A"
  wound: "#B92A20"
  chaff: "#F4CD50"
  lantern: "#F07428"
  verdigris: "#2FBD9F"
  cobalt: "#477FE4"
  wisp: "#A8F2D8"
  mauve: "#CDB2F6"
typography:
  display-s7:
    fontFamily: "Skribble Black"
    fontSize: "40px"
    lineHeight: "38px"
  display-s6:
    fontFamily: "Skribble Black"
    fontSize: "30px"
    lineHeight: "30px"
  display-s5:
    fontFamily: "Skribble Black"
    fontSize: "22px"
    lineHeight: "24px"
  text-s4:
    fontFamily: "Skribble Noto Sketch"
    fontSize: "1.0625rem"
    lineHeight: 1.294
  text-s3:
    fontFamily: "Skribble Noto Sketch"
    fontSize: "0.875rem"
    lineHeight: 1.357
  figures-s2:
    fontFamily: "Skribble Mono Gentle"
    fontSize: "12px"
    fontWeight: 700
    lineHeight: "16px"
  figures-s1:
    fontFamily: "Skribble Mono Gentle"
    fontSize: "11px"
    fontWeight: 700
    lineHeight: "14px"
    letterSpacing: "0.1em"
rounded:
  none: "0px"
spacing:
  s1: "4px"
  s2: "8px"
  s3: "12px"
  s4: "16px"
  margin: "20px"
  s6: "24px"
  s7: "32px"
  s8: "40px"
  column: "520px"
  target: "44px"
components:
  primary:
    backgroundColor: "{colors.lantern}"
    textColor: "{colors.key}"
    rounded: "{rounded.none}"
    height: "54px"
  secondary:
    backgroundColor: "transparent"
    textColor: "{colors.paper}"
    rounded: "{rounded.none}"
    height: "46px"
  quiet:
    backgroundColor: "transparent"
    textColor: "{colors.paper}"
    rounded: "{rounded.none}"
    height: "44px"
  slip:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.key}"
    rounded: "{rounded.none}"
  sheet:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.key}"
    rounded: "{rounded.none}"
  tab:
    backgroundColor: "transparent"
    textColor: "{colors.ash}"
    width: "78px"
    height: "56px"
---

# Design System: Reapers

## Overview

**Creative North Star: "Second Ink"**

The interface is a relief print pulled from two plates on a dark page. The first plate, key, prints every line and the words on paper; the second, the spot, is the bound reaper's own light, so a screen is printed in the colour of whoever the player swore to. Strokes are drawn by hand (Skribble's rough paths), seeded so every mark is always the same mark, and set slightly out of register where the print should feel pressed.

This document is the contract the code keeps. The section numbers (§) are the art bible's, which it condenses; the bible and its boards (`ui-sanctum.png`, `ui-hunt-*.png`, `ui-result.png`) live with the design work, outside the repository. Where the code departs from the bible, the departure is listed at the end, with why.

[PRODUCT.md](PRODUCT.md) owns product truth; [README](../../README.md), [gameplay](../../docs/gameplay.md) and [architecture](../../docs/architecture.md) own the rules and the build. Visual authority in code:

| Where | What it holds |
| --- | --- |
| [`src/theme.rs`](src/theme.rs) | Every colour, face, step of the type scale, space, stroke width and timing. The one place they are written; a contrast test holds the allowed pairs to §15.1. [`src/ink.rs`](src/ink.rs) only lends the hunt's marks their old names for the same tokens. |
| [`src/ui/ink.rs`](src/ui/ink.rs) | The strokes: `Mark` (what to draw), `Recipe` (a mark, a seed, one pass or two), `Plate` (a box of strokes, floods and fills). |
| [`src/ui/kit/`](src/ui/kit) | The kit (§8): buttons, rows, tabs, slips, the sheet, fields, toggles and marks, with every state named. |
| [`src/dev/kit.rs`](src/dev/kit.rs) | `/kit`, debug builds only: every component in every state on the vigil page, and the strokes its build drew. |
| [`src/screens/sanctum.rs`](src/screens/sanctum.rs) | The first screen built on the kit (§9). |

Screens not yet rebuilt still use the older ledger pieces (`LedgerButton`, `ledger_*`, the dark `sheet` and `dialog` in `src/ui`). They print in Second Ink's colours through the ledger names the theme keeps for them (`BONE` is paper, `INK` is key), and go when their screens move onto the kit.

## Colours (§3)

Every neutral is key printed over paper at a coverage.

| Token | Hex | Coverage | Use |
| --- | --- | --- | --- |
| Paper | `#ECE3CF` | 0 | Slips, sheets, plates; words on every dark ground. |
| Ash | `#A39C93` | about .34 | Labels and asides on vigil and scrim; disabled words; marks. Never on paper. |
| Raised | `#454042` | .78 | Cards and wells, a pressed row, a disabled primary's ground, rules. |
| Vigil | `#342F33` | .86 | The page. Every rebuilt screen is printed on it. |
| Scrim | `#2B272C` | .90 | Wells, a disabled dial's fill. Never under small words. |
| Key | `#16121A` | 1 | Every line; words on paper and on a light. |
| Wound | `#B92A20` | — | Loss and error only. Words only on paper; on a dark ground only as a mark (a strike, a tear, a cross) beside paper words. |

The ledger's old names are aliases of these, kept only while the screens not yet rebuilt say them: `BONE` is `PAPER`, `INK` is `KEY`, `UNBOUND` is `PAPER`. A rebuilt screen says the Second Ink name.

The spot is the **bound reaper's own light** and nothing else's: one colour per class, chosen by `theme::light`, and a screen prints in it only where the reaper's own light is meant (its marks, its stamp, its flood, its tally's newest stroke). A screen never invents an accent, never prints a second light beside the bound reaper's, and never prints the spot on paper. The only exception is a screen that shows several reapers at once (the roster), where each card prints in its own light and names it. **Wound is only for loss and error** — a strike, a tear, a disabled loss, a refusal — never for decoration, never for a selected state, and never as words on a dark ground.

**The six lights** (§3.2), one per class, from `theme::light`:

| Class | Light | Hex |
| --- | --- | --- |
| Harvester | Chaff | `#F4CD50` |
| Ferryman | Lantern | `#F07428` |
| Weigher | Verdigris | `#2FBD9F` |
| Chooser | Cobalt | `#477FE4` |
| Hollow | Wisp | `#A8F2D8` |
| Mourner | Mauve | `#CDB2F6` |

**The Borrowed Light Rule.** A screen prints in one spot: the bound reaper's (`ui::light`). Before the oath it is paper (`UNBOUND`), so nothing borrows a light not yet sworn. Only a screen that shows several reapers at once (the roster) prints several lights, each named. The kit page prints in the Ferryman's lantern until a reaper is bound, so a flood is never paper on paper.

**Contrast** (§15.1, held by `theme::tests::every_allowed_pair_holds_its_contrast` and `the_forbidden_pairs_fail`):

| Pair | Ratio | Allowed |
| --- | --- | --- |
| Paper on vigil, raised, scrim, key | 10.28, 7.97, 11.51, 14.49 | All text |
| Key on paper | 14.49 | All text |
| Ash on vigil, scrim, key | 4.83, 5.41, 6.81 | All text |
| Ash on raised | 3.75 | Display s5 and up, and marks |
| Wound on paper | 4.82 | Text |
| Key on a light | 4.78 (cobalt) to 14.40 (wisp) | All text |
| A light on vigil | 3.39 (cobalt) to 10.22 (wisp) | Display s5 and up only |
| Wound on any dark ground; ash on paper; a light on paper | under 4.5 | Never text |

## Type (§7)

Three faces, each with one weight; nothing is made bold. They are set with `font-[var(--display)]`, `font-[var(--text)]` and `font-[var(--figures)]` (`theme::Face`); the web shell defines the properties with their fallback chains in `assets/app.css` and preloads the files. The faces are OFL 1.1 ([`assets/fonts/CREDITS.md`](../../assets/fonts/CREDITS.md)). Native builds at the pinned leptoskit draw the system face until its native font loading lands; the app's README says how to switch it on.

| Face | Family | For |
| --- | --- | --- |
| Display | Skribble Black | Names, stamps and chops, the one primary action per screen, the dial's words, stamped figures. Nothing else. |
| Text | Skribble Noto Sketch | Sentences, secondary and quiet buttons, notices, errors. |
| Figures | Skribble Mono Gentle Bold | Every number (tabular), and every uppercase label (tracked 0.1 em). |

| Step | Size / leading | Face | For |
| --- | --- | --- | --- |
| s7 | 40 / 38 | Display | Names, result tallies |
| s6 | 30 / 30 | Display | The primary action |
| s5 | 22 / 24 | Display | Dial words, sheet titles; the smallest the spot may carry |
| s4 | 17 / 22 | Text, Figures | Secondary buttons, lead sentences, a row's name |
| s3 | 14 / 19 | Text, Figures | Sentences, the SOL balance, notices |
| s2 | 12 / 16 | Figures | Captions, field labels, the hunt's smallest |
| s1 | 11 / 14 | Figures | Labels; the smallest anywhere, never in the hunt |

- **Fitted Display.** A name is set at its step and shrunk a point at a time until it takes no more than two lines, never under s5 (`theme::fit_display`). A button's label keeps to one line (`fit_display_line`). The widths come from Skribble Black's own advances.
- **Text size.** Sentences are sized in rem and grow with the reader's text size up to 200%. Display, Figures and labels are in points and hold their size (see the departures).

## Ink: how a stroke is drawn

- Every stroke is a `Recipe`: a `Mark` (a line, frame, circle, arc, path, hatch, halftone, waxing disc), a seed, and one pass or two. Two passes for outlines that carry state, one for rules and ticks; never three.
- `leptoskit_skribble::rough` turns a recipe into a rough path. The path is generated once and kept for the life of the game: the same recipe always draws the same stroke, and a screen built again draws none. `ui::ink::drawn()` counts strokes generated and `asked()` counts strokes asked for; `tests/screens.rs` (`a_screen_built_again_draws_no_new_ink`) fails if a second build generates one, and `/kit` prints both counts.
- Seeds come from the words a mark belongs to (`theme::seed_for`) or a fixed number, so a mark is the same mark on every device and every build.
- Ink moves by transform only. Nothing is redrawn to animate; boil (the one redraw the bible allows) is not built.
- A `Plate` lays strokes over its box (`over()`) or takes their size (`sized()`). `Ink::Current` paints in the element's text colour, which is how a press reveals a layer (`active:text-[…]`) without a second stroke.

| Stroke | Width (pt) |
| --- | --- |
| Primary frame | 2.4 |
| Emphasis: active tab underline, focused field, pressed link, wound strike, stamp ring | 2.2 |
| Slip, secondary, field outline | 1.8 |
| Toggles, icons | 1.7 |
| Card, tally, tab rule, class mark | 1.6 |
| Meter | 1.5 |
| Divider | 1.4 |
| Link underline at rest | 1.2 |

## Layout and spacing (§9)

- Every rebuilt screen is the vigil page, with content inside 20 pt side margins and a column no wider than 520 pt (`ui::kit::measure`). Plates are drawn to the measure, so a rule is as long as it is shown.
- Spacing is the 4 pt grid in `theme::space`, and nothing is spaced off it:

| Token | Points | Use |
| --- | --- | --- |
| `space::S1` | 4 | A mark and its word |
| `space::S2` | 8 | A label and what it labels |
| `space::S3` | 12 | Lines of one group |
| `space::S4` | 16 | One group and the next |
| `space::S5` / `MARGIN` | 20 | A screen's side margins |
| `space::S6` | 24 | One section and the next |
| `space::S7` | 32 | Around the primary |
| `space::S8` | 40 | A screen's head and its body |
| `space::COLUMN` | 520 | The widest a content column grows |
| `space::TARGET` | 44 | The smallest touch target |
| `space::PRIMARY` | 54 | The primary button's height |
| `space::SECONDARY` | 46 | A secondary button's height |
| `space::TAB_WIDTH`, `TAB_HEIGHT` | 78 × 56 | A tab's target |
| `space::REGISTER` | 3 | How far the spot plate sits out of register |
| `space::LAMP_RADIUS` | 132 | The sanctum's lamp disc on a phone |

- One primary action per screen, at most.
- Safe areas: the top of a screen pads with `pt-safe-offset-*`, the tab bar with `pb-safe`.

## Elevation and shape

Flat. There are no shadows, no blur, no rounded corners and no gradients. Depth is the print's: a plate out of register (the primary's spot sits 3 pt down and right of its key frame), paper laid on the page, a deckled edge, a hatch. The 3D stage and the hunt keep their real light and shadow; this is the interface only.

## Components (§8)

Every control prints each state it can be in. `kit::State` names them: **Rest** (the only state that takes a press), **Pressed**, **Focused**, **Busy** (takes no press, says it is busy) and **Disabled**. Every press calls `platform::tap()` before its action. A control is built once, with `on_press` for a live one and `print` for a state held on `/kit` (a slip's words alone are `Slip::view`).

### Buttons

**Primary** (`kit::Primary`). The one primary action on a screen, at most: 54 tall, the full measure. A spot flood 3 pt out of register under a key frame of 2.4, a key hatch on the lower 10 pt (the lip), the label in Display fitted to one line from s6, and a detail in Figures s2 with a drawn arrow ("2 nearby →"). States: **Rest** (the flood out of register), **Pressed** (the flood and label come into register over 120 ms), **Focused** (crop marks), **Busy** (`aria-busy`, no second press, the lip held dark) and **Disabled** (a raised ground under a key hatch at 55%, an ash label, and the reason in words beneath it). Under reduced motion it rests in register and a press darkens the lip.

```rust
Primary::new(words.sanctum_go_hunting(), light)
    .detail(words.sanctum_nearby(count))
    .test_id("sanctum-hunt")
    .on_press(|| open(Hunt { sighting }))
```

**Secondary** (`kit::Secondary`). 46 tall, a paper outline of 1.8, the label in Text s4, a figure or a drawn arrow at the right edge. States: **Rest**, **Pressed** (a paper hatch at 60% inside), **Focused** (crop marks), **Busy** (the hatch held, no second press) and **Disabled** (a dashed ash outline, 5 on and 5 off, an ash label). As a destructive action (`.destructive()`) a wound strike of 2.2 runs under the words, and the words say what is lost; a sheet confirms it.

```rust
Secondary::new(words.vault_release())
    .destructive()
    .test_id("vault-release")
    .on_press(|| open(Route::Vault))
```

**Quiet** (`kit::Quiet`). A text link: Text s3 with a paper underline of 1.2 at rest, thickening to 2.2 while pressed. At least 44 pt tall. States: **Rest**, **Pressed**, **Focused** (crop marks), **Busy** and **Disabled** (ash words, no underline). Use it for the lesser of two choices, never for the only way off a screen.

```rust
Quiet::new(words.sanctum_colophon())
    .state(State::Rest)
    .test_id("sanctum-colophon")
    .on_press(|| open(Route::Colophon))
```

### Surfaces

**Row** (`kit::Row`). One line of a ledger, at least 56 tall: a name in Text s4, an optional status label in Figures s1 under it, a figure in Figures s3 at the right, and a drawn arrow, over a `divider`. States: **Rest**, **Pressed** (a raised ground), **Focused** (crop marks), **Busy** (printed pressed, no second press) and **Disabled** (ash words, no arrow). Use rows for a list of destinations, never a column of primaries.

```rust
Row::new(words.roster_title())
    .figure(words.sanctum_roster(count))
    .test_id("sanctum-roster")
    .on_press(|| open(Route::Roster))
```

**Tabs** (`kit::tab_bar`, `kit::Tab`, `kit::TabIcon`). Five tabs of 78 × 56 under a raised rule: Sanctum (a lamp), Vault (a box), Armory (a blade), Codex (a book), Rifts (a crack). Icons are two passes of 1.7. States: **Inactive** (ash icon and label), **Active** (paper icon and label, a spot hachure inside the icon, a spot underline 28 pt long at 2.2, `aria-current="page"`), **Pressed** (the hachure prints at once) and **Focused** (crop marks). The bar is `navigation`.

```rust
Tab {
    icon: TabIcon::Lamp, label: words.sanctum_overline().to_owned(), test_id: "sanctum-tab".to_owned(),
    active: true, state: State::Rest, open: Box::new(|| open(Route::Index)),
} // in tab_bar(vec![…], light)
```

**Slip** (`kit::Slip`). A notice on paper: key words in Text s3 inside a key frame of 1.8, two passes, turned up to 2° either way by its seed. States: **Rest** (it rises 8 pt over 333 ms as it enters), **Entering**, **Leaving** and **Pressed** (a key hatch, once it is a control). `.torn()` tears its left edge in wound for an error, whose words say what went wrong. Its target is at least 48 pt.

```rust
Slip::new(words.sanctum_sighted_banner(&title))
    .test_id("sanctum-sighted-banner")
    .on_press(move || open(Hunt { sighting }))
```

**Sheet** (`kit::PaperSheet`). Full paper risen from the foot of the screen: a deckled top cut by its seed, a key handle 36 × 3, key words, and nothing dimmed behind it. It rises 120 pt over 333 ms, or appears in place under reduced motion; it cannot be dragged yet. `role="dialog"`.

```rust
PaperSheet::new(seed_for("vault-sheet"), vec![
    body(words.vault_stand_in(), Step::S3.size(), KEY),
    Secondary::new(words.common_dismiss()).on_press(|| close()),
])
.view(true)
```

**Field** (`kit::Field`). A text input: a paper outline of 1.8 under a Figures s2 label in ash, the words in Text s4. States (`kit::FieldState`): **Empty** (the placeholder in ash), **Focused** (crop marks and a 2.2 outline), **Filled**, **Error** (a torn wound underline, the reason in paper beside a drawn wound cross, `aria-invalid`) and **Disabled** (dashed ash).

```rust
Field::new(words.roster_asset(), "roster-asset")
    .value(address)
    .placeholder(words.roster_import_help())
    .on_change(move |text| set_address(text.to_owned()))
```

**Toggle** (`kit::Toggle`, `kit::ToggleKind`). A switch, checkbox or radio drawn at 1.7, with "On" or "Off" in Figures s1 beside the mark, so the state is in words as well as shape, in a target at least 44 pt tall. States: **On** (spot hachure, a paper knob or tick), **Off** (an ash outline), **Pressed**, **Focused** and **Disabled** (dashed). Its role and `aria-checked` are set.

```rust
Toggle::new(ToggleKind::Switch, words.colophon_music(), on, light)
    .state(State::Rest)
    .test_id("colophon-music")
    .on_press(move || set_music(!on))
```

### Marks

Marks are drawn, not typed: every one takes its seed and prints in a token's colour. They are re-exported from `kit`, and `/kit` prints each.

- **`divider(seed)`** — a raised rule of 1.4 with a slight bow, drawn at the measure. `divider(seed_for("sanctum"))`.
- **`class_mark(spot, seed)`** — the class's registration mark: a spot circle of r 6 crossed by an 18 pt hair. `class_mark(light, seed_for("sanctum-class"))`.
- **`stamp(numeral, spot, seed)`** — the level chop: a spot ring of 2.2 in two passes, turned −6° to −9° by its seed, "Level" in paper and the numeral in Display in the spot. It lands from 1.15 times its size over 166 ms. `stamp(&roman_numeral(level), light, seed_for("sanctum-stamp"))`.
- **`tally(count, gates, spot, seed)`** — gates of five paper strokes at 1.6, the newest in the spot at 2.2, at most `gates` gates drawn; the figure beside it says the whole count. `tally(count, gates, light, seed_for("sanctum-tally"))`.
- **`pips(count, seed)`** — 1 to 4 paper registration marks beside a name, one for common up to four for epic. There is no rarity colour. `pips(rarity, seed)`.
- **`lamp(radius, fraction, spot, seed)`** — experience to the next level as a disc: the earned part solid spot, the rest a spot halftone, in a rough two-pass ring at R + 3; full, the ring doubles. `lamp(radius, Progression::level_progress(xp), light, seed)`.
- **`soul_dial(soul, spot, seed)`** — a key ring of 7.5 under a 4 pt spot arc, with a paper notch at 40 (an orb's cost). Under 40 the arc is ash and the dial says what it needs; `role="meter"`. `soul_dial(soul, light, seed)`.
- **`meter(fraction, figure, warning, width, seed)`** — a paper strip in a key outline of 1.5, what remains hatched in key at 60°, the figure beside it. A loss under 15% strikes the figure in wound and says so in words beside a wound cross; a husk's ward is never struck. `meter(ward, &figure, Some(words.hunt_ward()), width, seed)`.
- **`arrow(colour, seed)`** — a drawn shaft and open head, 1.7. `arrow(KEY, seed.wrapping_add(3))`.
- **`crop_marks()`** — paper, 2 pt wide and 6 long, 4 pt outside each corner: the **Focused** state, held for `/kit`. A live control gets the same marks from the web shell (`assets/app.css`, `:focus-visible`), and a field shows them while its input is written in.

**`kit::measure()`** is the page's column width in points (the screen less its margins, never over 520); a plate is drawn to it. **`kit::steady()`** is the reader's reduced-motion setting.

## The sanctum (§9)

The screen a player comes home to, built on the kit to the board `ui-sanctum.png`:

1. **The stage** takes the top 52% of the screen and gives way first when the screen is short. Behind the reaper, the lamp disc in the spot (r 132 on a phone), a little right of centre and a little above it, waxing with the experience to the next level. Under its feet a key floor stroke across the stage and a cast shadow hatched in key. Nothing is printed over the model.
2. **The stage's top right corner:** the SOL balance in Figures s3, as §9 and the board print it. Nothing else is printed on the stage, so the rest of it still turns the reaper under a finger.
3. **Who it is**, under the floor: the class mark and the class, the name in fitted Display from s7 beside the level stamp, and "214/400 XP to V · the lamp is waxing" in Figures s2 ash.
4. **Taken:** the tally of the codex, with the whole count beside it.
5. **Loose in the veil**, with "2 / 3": up to three husk ghosts at 46 pt (a press hunts one once it has surfaced; until then it shows its countdown), a sentence naming them, and a dashed ash ring with a plus for the next empty place, which opens the vault.
6. **The rows** at the foot of the page, each a ledger row: the studio, the roster with a figure of "1 reaper" or "N reapers", a practice hunt, the colophon, and the simnet ledger in a simnet build.
7. **The primary**, "Go hunting", with how many husks are near. It hunts the first; with none near it sends the reaper after a stray, so the way out is never shut.
8. **The tab bar**, with the sanctum active.

Below the stage the page scrolls when it must; the primary and the tab bar stay where the hand finds them.

## Motion (§13) and reduced motion (§14)

| Motion | As built |
| --- | --- |
| A press | The snap: a 120 ms ease-out transform |
| A slip or sheet rising | 8 pt (a sheet 120 pt) over 333 ms, ease-out |
| A stamp landing | From 1.15 times its size over 166 ms |
| Springs | `theme::motion`: snap, arm, settle, carry, for what follows a finger or the world |

Nothing idles on the interface: the lamp, the marks and the page are still; only the reaper on its stage breathes.

**Reduced motion** (`kit::steady`, from `platform::reduced_motion`): nothing slides, rises or scales.

- The primary prints in register; a press darkens its lip instead.
- Stamps, slips and sheets appear in place.
- On the web, `assets/app.css` holds every transition and animation to zero under `prefers-reduced-motion`.
- The native shells do not report the setting yet, so a native player sees full motion; that is the bible's stated dependency.

## Accessibility (§15)

- Contrast: the table above, tested.
- Targets: at least 44 pt (tested in `kit::tests`), tabs 78 × 56.
- Text: at least 11 pt; sentences grow to 200%.
- State is never colour alone: rings are shapes, wound is a strike plus words, toggles say "On" and "Off", and lights are named where two meet.
- Roles: buttons and links are `Pressable`s with their words as their names; the dial and meter are `meter`s with values; toggles carry their role and `aria-checked`; the tab bar is `navigation` with the current tab marked; the sheet is a `dialog`; notices are polite live regions; Display names are headings.

## Departures from the bible

| Departure | Why |
| --- | --- |
| The kit draws with `leptoskit_skribble::rough` and its own cache, not leptoskit_skribble's `Wired*` components. | Those regenerate their strokes on every build; the bible wants a mark drawn once and kept. |
| Labels are tracked at 0.1 em. | Tailwind's widest tracking, which leptoskit compiles natively; close to the boards. |
| Display, Figures and labels are in points and do not grow with the text size. | Sentences carry the reading; Display is fitted to its box, and the HUD's 130% cap needs per-step scaling leptoskit does not offer yet. |
| A pressed primary's label moves 3 pt with its flood, not 1.5. | The label rides on the flood's layer; splitting it would draw the plate twice. |
| The busy lip is held dark, not stepped. | leptoskit has no `steps()` timing or keyframes. |
| Rises are smooth eases over their duration, not 4 held steps on twos. | The same: no stepped timing. |
| Slips and sheets leave by a cut. | leptoskit has no exit transition. |
| A sheet cannot be dragged and has no settle. | leptoskit has no drag gesture a sheet can follow. |
| The soul dial's arc does not animate. | It is kept to whole hundredths so a dial draws at most a hundred arcs. |
| Focus marks are drawn on the web only. | leptoskit has no focus variant; native focus is the platform's. |
| The sheet's deckle is a plain polygon. | A filled rough shape would cost a stroke per sheet. |
| The lamp line counts XP ("214/400 XP to V"), not soul. | Soul is the hunt's orb energy (the dial); the lamp is the level. |
| The tab bar is on the sanctum only. | The other screens are not rebuilt yet and keep their back link. |
| The ghosts are the vault's portrait plates, not the ghost wash (§3.6). | The wash needs the portraits rebuilt without their ground by the art build, which this work does not touch. |
| The reaper stands about 270 pt tall on a 439 pt stage, not about 400. | The stage camera (`render/camera.rs`) is shared with the oath and the armory; framing it closer is the 3D look's own change (the bible's appendix A). |
| Studio, roster, practice, colophon and the simnet ledger are rows under the fold. | The board leaves room only for the primary and the tab bar above them; a phone scrolls the page for the rest. |
| No sound or haptic was added for the stamp. | Sounds and haptics are not changed by the interface work. |

## Do's and don'ts

### Do

- Take every colour, size, space, stroke and timing from `theme`, and every control from the kit.
- Give every control each state it can be in, and print a new one on `/kit`.
- Seed every stroke, so it is drawn once and kept; check `/kit`'s counts after adding marks.
- Put every word a player reads in `locales/en.arb`, plurals included.
- Keep one primary per screen, and the spot to the bound reaper's light.
- Use a `Row` or a `Secondary` for every destination that is not the screen's one primary.
- Keep every target at least 44 pt tall, and every sentence in rem so it grows.
- Look at the result at a phone's size, with reduced motion and at 200% text.

### Don't

- Don't print words in wound, ash or a light where §15.1 forbids it.
- Don't print the spot on paper, in a second light beside the bound reaper's, or as decoration.
- Don't use wound for anything but a loss, an error or a refusal, and never as words on a dark ground.
- Don't round a corner, cast a shadow, blur or add a gradient.
- Don't redraw a stroke to animate it; move it by transform.
- Don't print over the reaper on its stage.
- Don't add a second primary, a second tab bar, or a new colour the palette does not hold.
- Don't add a per-scene environment map for a stage; the Seeker's GPU is fragile (`docs/environment.md`).
