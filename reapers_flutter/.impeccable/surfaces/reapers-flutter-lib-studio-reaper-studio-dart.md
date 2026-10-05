---
version: 1
slug: "reapers-flutter-lib-studio-reaper-studio-dart"
primary_target: "reapers_flutter/lib/studio/reaper_studio.dart"
related_targets: ["reapers_flutter/lib/main_preview.dart", "reapers_flutter/lib/screens/roster_screen.dart", "reapers_flutter/lib/widgets/ledger.dart", "reapers_flutter/lib/render/reaper_stage_web.dart", "reapers_flutter/web/index.html"]
---

# Reaper studio

## Strategy

**Experience leads; Operate controls.** The actual lit game model is the subject. The inspector lets the player try a specimen and understand its growth while retaining the funeral ledger's ink, bone, Fraunces and DM Sans. The displayed genome supplies the single active light. No new identity, palette, font or logo is introduced.

The opening source comment persists THESIS, OWN-WORLD, STORY, FIRST VIEWPORT and FORM. This is a precisely scoped extension of the incumbent model viewer and armory vocabulary; FORM seed is n/a under that extension exemption.

## Composition and operation

At 720 logical pixels and wider, place the model on the left and an independently scrollable inspector on the right, separated by one hairline. Bound the whole composition to 1400; the inspector is 340 below 1024 and 430 from 1024. Below 720, stack a 390-pixel portrait above the controls in one scrollable page; cap the control column at `Veil.contentWidth` (520). Both forms use safe areas and 24-pixel inspector padding. These measurements belong to this surface, not the global spacing scale.

The portrait carries the specimen's calling, name and drag instruction. One slow 25-second automatic turn then holds the model; the player can pause, replay or drag manually. Reduced motion suppresses automatic turning. The model remains the visual memory of the first viewport.

Supported targets are web, Android and iOS. Desktop clients are outside scope. The browser portrait uses WebGL 2, the actual GLBs and the authored material arithmetic; Android and iOS keep Flutter GPU. Browser pointer observation preserves drag rotation alongside the rendering dependency's own gesture recognizers. The standalone web entry enables semantics for the inspector and provides a localized notice when WebGL 2 is unavailable. Browser launch icons come from the same installed brand artwork.

**Birth traits** explores the fixed genome through calling and variant choices and trait sliders. **Growth** independently demonstrates shared progression, strike comparisons and unlocked weapon/outfit choices. The level selector and XP meter describe different quantities; the capped meter reads Max. Native segmented controls, choice chips, sliders and list tiles carry the task, while primary actions retain the square, flat ledger geometry.

Player-facing labels are localized. Trait sliders and numbered variants have meaningful independent semantic names; absorption feedback is a live region. Ledger actions use transparent Material/InkWell for native keyboard activation, expose semantic activation and show focus in specimen light. Labels and action details may wrap rather than colliding.

## Product truth

The standalone preview opens no preferences, saved player, server or wallet. Trait edits, illustrative absorption, level changes and equipment choices are unsaved and charge no currency. Birth traits are fixed at minting; each owned reaper earns growth separately. The account roster is a separate surface for the starter and verified NFT reapers. A configured marketplace opens externally; absent collection configuration means imports are unavailable, without asserting a launch date or live purchase capability. Launch instructions remain in [preview.md](../../../docs/preview.md).

## Observed finish and limits

The [initial review](../../../.impeccable/review/studio-review.md) identified five fixes; the [final verdict](../../../.impeccable/review/studio-verdict.md) returned **ship** after all five were resolved: persisted context, accessible controls, truthful progression labels, finite rotation and roster configuration copy.

Earlier native desktop captures were exploratory and do not define supported targets. The [iOS phone traits](../../../.impeccable/review/studio-phone-ios-traits.png) and [iOS phone growth](../../../.impeccable/review/studio-phone-ios-growth.png) captures show the mobile model and inspector. The prior verdict covers its five fixes, not whole-surface approval.

The web adaptation was inspected at 1280 × 720 and 390 × 844. Calling and colour changes, manual rotation, absorption feedback, independent inspector scrolling and full-potential hollowing were exercised in the running browser. Current browser logs were clean. The material contract tests bind the actual authored surface to the renderer's real shader template and check upgrade uniforms and explicit failures. The mechanical design detector fell back to regex because its HTML parser dependencies were absent; it does not establish computed contrast. Physical Android/iPhone browsers, tablets, large text and roster visual evidence remain unavailable.
