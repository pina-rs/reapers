---
name: Reapers
description: A funeral ledger around a spirit realm, with the phone as the blade.
colors:
  ink: "#07080B"
  ink-raised: "#0F1116"
  bone: "#E9E4D4"
  ash: "#8E897D"
  hairline: "#E9E4D42E"
  wound: "#D4553F"
  unbound: "#BFC8CF"
typography:
  display:
    fontFamily: "Fraunces"
    fontSize: "64px"
    fontWeight: 420
    lineHeight: 0.98
    letterSpacing: "-1.408px"
  headline:
    fontFamily: "Fraunces"
    fontSize: "26px"
    fontWeight: 420
    lineHeight: 0.98
    letterSpacing: "-0.572px"
  title:
    fontFamily: "Fraunces"
    fontSize: "22px"
    fontWeight: 420
    lineHeight: 0.98
    letterSpacing: "-0.484px"
  body:
    fontFamily: "DM Sans"
    fontSize: "15px"
    fontWeight: 400
    lineHeight: 1.42
  label:
    fontFamily: "DM Sans"
    fontSize: "11px"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "2.42px"
  figure:
    fontFamily: "DM Sans"
    fontSize: "15px"
    fontWeight: 500
    lineHeight: 1.2
rounded:
  ledger: "0px"
components:
  ledger-action:
    backgroundColor: "transparent"
    textColor: "{colors.bone}"
    rounded: "{rounded.ledger}"
    padding: "16px 18px"
  ledger-action-quiet:
    backgroundColor: "transparent"
    textColor: "{colors.bone}"
    rounded: "{rounded.ledger}"
    padding: "16px 18px"
---

# Design System: Reapers

## Overview

**Creative North Star: "The Funeral Ledger"**

The native interface is a funeral ledger set in warm bone on blue-black ink. Large, sharp words give names and numbers ceremony; small ruled entries keep the game readable. The spirit realm supplies movement and depth behind the ledger. This records the incumbent Flutter world and its branding and model-viewer extensions, rather than a replacement identity.

The product premise comes from [README](../README.md), [gameplay](../docs/gameplay.md) and [architecture](../docs/architecture.md). Visual authority is [Veil](lib/theme/veil.dart), the [ledger kit](lib/widgets/ledger.dart), and the implemented [threshold](lib/screens/threshold_screen.dart), [arrival](lib/brand/splash_gate.dart) and [studio](lib/studio/reaper_studio.dart). Values in the frontmatter translate Flutter logical pixels to portable `px`; the Dart source remains the implementation authority.

**Key Characteristics:**

- Warm bone words and fine rules on tinted ink.
- Fraunces for names and large numerals; DM Sans for instructions, labels and figures.
- A reaper's single light carries the interface's active colour.
- Spacious ceremonial entry, with compact ledger controls inside the game.

## Colors

The palette holds the interface in shadow while reserving light for words, actions and the bound reaper.

### Primary

- **Reaper light:** supplied to `Veil.theme`, rather than fixed in this document. It fills a pressed ledger action, marks keyboard focus and highlights important standings. In a specimen viewer it comes from the displayed genome. **Unbound** is the provisional player's light before the oath.
- **Wound:** a loss, refusal or error. It carries a meaning, rather than becoming a second decorative accent.

### Neutral

- **Ink:** page and native launch background.
- **Raised ink:** modest separation for a surface above the page.
- **Bone:** primary text, the emblem and visible rules.
- **Ash:** supporting text and unavailable actions.
- **Hairline:** low-opacity bone for dividers, dotted leaders and quiet borders. The frontmatter uses CSS RGBA hex; the Dart constant uses ARGB order.

**The Borrowed Light Rule.** Active interface colour comes from the displayed reaper; new screens use the theme's primary colour rather than inventing an accent.

## Typography

Both variable fonts are bundled in [pubspec.yaml](pubspec.yaml). The incumbent pairing is retained because it already defines the native game.

- **Display:** Fraunces, with the theme's large/medium/small sizes at 64, 44 and 32. Headline and title use 26 and 22. `Veil.display` tracks at −2.2% of size, sets optical size to the text size within 9–144, and uses `SOFT=0`, `WONK=1`.
- **Body:** DM Sans at 15 by default; the theme also uses 16 and 13. Body line height leaves instructions room to breathe.
- **Label:** DM Sans in tracked capitals for action labels, units and functional ledger headings; the theme also uses 12 and 10. Tracking is 22% of size. This does not establish decorative eyebrows.
- **Figure:** DM Sans with tabular numerals for counts, prices and standings.

**The Name and Number Rule.** Names and ceremonial numerals use `Veil.display`; instructions and ledger figures use their respective `Veil` helpers.

## Layout

Standard ledger content columns grow only to `Veil.contentWidth` (520 logical pixels). Safe areas protect phone edges. The threshold uses a scrollable column with a 28-pixel horizontal inset, so a short viewport can scroll; its wordmark scales down to fit. Ledger action details wrap beneath their label when the pair cannot share a line. Retain these behaviours when adapting a surface; model-viewer compositions are recorded in their own surface briefs.

The threshold places the mark above the large wordmark, premise, outlined oath action and quiet practice action. Its open upper field leaves room for the blade arc and moving veil. This composition belongs to arrival; it is not a template for inventory or combat screens.

## Elevation & Depth

The ledger controls are flat: tonal surfaces, transparent controls and fine borders carry hierarchy without drop shadows. The native 3D realm and its atmosphere provide depth. This describes the UI kit; it does not prohibit the game's real scene lighting or creature shadows.

## Shapes

Ledger actions have square corners and straight borders. Rows use dotted leaders; resource meters use a narrow bar with a visible end notch; pips are small diamonds. The emblem's broken halo and scythe curve belong to the ceremonial brand, while ledger controls retain their rectilinear form.

## Components

### Ledger actions

`LedgerButton` is a full-width entry with a minimum height of 56. At rest, its outline is bone at 70% opacity; the quiet variant omits that outline. Pressing fills either variant with the reaper's light and changes text to ink over 90 ms. Keyboard focus gives either variant a visible border in that light, including the quiet action. Disabled actions use ash text and, for outlined actions, a hairline border. Detail text may wrap; the primary and quiet actions retain equal touch geometry.

Transparent `Material` and `InkWell` provide touch and Enter/Space activation without introducing a splash, hover fill, rounded shape or elevation. Semantics exposes the combined label and detail, enabled and focus states, and an activation action. Disabled actions cannot request focus or activate.

### Model viewers

Inspect the actual game model through `ReaperStage`, retaining the specimen's light, materials and manual drag rotation. Automatic turning respects reduced motion. Supported targets are browser, Android and iOS: the browser uses WebGL 2 with the same authored models and surface arithmetic, while mobile uses Flutter GPU. The studio adds a finite demonstration and keeps its unsaved specimen separate from owned growth; its composition, controls and review scope belong to the [studio surface brief](.impeccable/surfaces/reapers-flutter-lib-studio-reaper-studio-dart.md).

### Mark and wordmark

**Quiet Cut is the provisional default**, a scythe-R inside a broken halo. The Veil, Bound Sigil and Last Ember remain saved alternatives; no permanent user selection is implied. Use `ReapersMark` so launcher, native still and Flutter screens share the generated master. The wordmark is live Fraunces text, not lettering baked into the emblem. See [branding.md](../docs/branding.md) for masters, provenance and rebuild steps.

### Arrival gate

The native still matches the first Flutter mark and ink background. After image priming, one 1.8-second reaping arc appears around the stationary emblem and the wordmark resolves; the exit fades over 220 ms. Reduced motion substitutes a 250-ms still and a direct release. During arrival, the mounted game excludes touch, focus, semantics and tickers. A four-second startup deadline begins when the gate mounts and releases the loading or recovery screen even if the backend remains slow; arrival must not become an indefinite spinner.

The threshold then draws its blade over 1.6 seconds and slowly turns the veil over four minutes. Reduced motion stops the drift and shows the settled composition.

The final reviewer returned **ship** for the resolved threshold eyebrow fix on the valid [iOS phone capture](../.impeccable/review/phone-ios.png). That verdict covers the named correction. Android compilation passed; Android and tablet visual captures were unavailable. This is not visual approval of those layouts or proof of installed launcher icons, every cold-start frame, or real-device motion.

## Do's and Don'ts

### Do:

- **Do** reuse `Veil`, the ledger kit and `ReapersMark` on new native screens.
- **Do** let short screens scroll, scale the wordmark down and wrap action details.
- **Do** keep launch motion finite, honour reduced motion and expose startup recovery.
- **Do** make ledger actions operable by touch, keyboard and assistive technology, with visible focus in the displayed reaper's light.
- **Do** rebuild and relaunch after changing installed native artwork.

### Don't:

- **Don't** treat the provisional logo default as a final user selection.
- **Don't** add decorative eyebrows to arrival; its mark, name and premise already establish the hierarchy.
- **Don't** expose the underlying game to interaction or accessibility focus while the arrival gate covers it.
- **Don't** infer Android or tablet visual approval from a successful compile or the iOS correction verdict.
