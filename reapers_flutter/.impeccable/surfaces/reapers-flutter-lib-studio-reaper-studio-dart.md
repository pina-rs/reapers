---
version: 1
slug: "reapers-flutter-lib-studio-reaper-studio-dart"
primary_target: "reapers_flutter/lib/studio/reaper_studio.dart"
related_targets: ["reapers_flutter/lib/main_preview.dart", "reapers_flutter/lib/screens/roster_screen.dart", "reapers_flutter/lib/widgets/ledger.dart", "reapers_flutter/lib/widgets/soul_ledger.dart", "reapers_flutter/lib/render/reaper_stage_web.dart", "reapers_flutter/web/index.html"]
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

**Soul** extends the same inspector. Its level selector shares the specimen's XP with Growth; “Try a gift” offers the three fixed gifts of the displayed calling. Across the six callings there are eighteen gifts and four genome-derived temperaments. Birth traits and Growth include quiet links into Soul. The shared `SoulLedger` presents one gift heading, its current effect, awakening and next requirement, then a single `{name} temperament` heading with its benefit and cost. The gift's lore can expand; earned and locked milestone rows show the three awakenings. This is an extension of the established surface: no new concept tournament or approved comp was commissioned.

“Hunts remembered” shows Unseen, Stillness and Rhythm with independent counts and the selected gift's preferred path highlighted. The inspector reads the gates from shared `SoulRules`: level 5 with 3 preferred-path marks, level 12 with 12, and level 20 with 30. Preview hunt actions illustrate an ambush, an unflinching hold or five hits in cadence. Each adds the corresponding mark and illustrative XP through shared rules. Increasing level alone does not earn marks. “See full potential” in Growth sets the level cap, hollowed equipment and 30 marks in all three paths; “Clear specimen memories” clears only those counts, retaining XP and equipment.

Player-facing labels are localized. Trait sliders and numbered variants have meaningful independent semantic names; absorption feedback is a live region. Ledger actions use transparent Material/InkWell for native keyboard activation, expose semantic activation and show focus in specimen light. Labels and action details may wrap rather than colliding.

At enlarged text sizes the tab control becomes vertical when `textScaler.scale(14) > 20`. The gift-choice heading uses the short localized action “Try a gift”. Gift and temperament fields each use a single title instead of a category eyebrow stack.

## Product truth

The standalone preview opens no preferences, saved player, server or wallet. Trait edits, gift choices, illustrative absorption, level changes, equipment choices and remembered hunts are local unsaved specimen state and charge no currency. The same boundary holds when opening the studio from the game. Birth gifts and temperaments remain fixed for an owned reaper; each reaper earns its own growth and memories through successful hunts. The studio's preview actions neither submit hunt claims nor change a saved player or wallet. The account roster is a separate surface for the starter and verified NFT reapers. A configured marketplace opens externally; absent collection configuration means imports are unavailable, without asserting a launch date or live purchase capability. Launch instructions remain in [preview.md](../../../docs/preview.md).

## Observed finish and limits

The previously recorded studio review identified five fixes and its verdict returned **ship** after those five were resolved: persisted context, accessible controls, truthful progression labels, finite rotation and roster configuration copy. That is historical evidence for those corrections, not a verdict on the Soul extension. Its earlier report and captures are not in the current review packet. Earlier native desktop captures were exploratory and do not define supported targets.

The [Soul finish review](../../../.impeccable/review/soul-review.md) returned **fix** on 5 October 2026. It found repeated gift/temperament eyebrows, an ellipsized gift-choice heading and stale design records. The source now uses the single gift heading, `{name} temperament` and “Try a gift”; this documentation records the implemented extension and provisional Bound Sigil selection. The same reviewer's [fix verdict](../../../.impeccable/review/soul-verdict.md) returned **ship**: all three scored corrections were resolved, with no visible regression introduced by that batch in the supplied recaptures. It covers those three corrections, not the whole surface or uncaptured devices and flows. All ten captures remain current; no application changes followed this verdict.

The current packet contains ten captures. Web uses [desktop](../../../.impeccable/review/desktop.png) at 1180 × 820, [phone opening](../../../.impeccable/review/mobile-opening.png) and [Soul detail](../../../.impeccable/review/mobile.png) at 390 × 844. Physical Seeker captures show [studio](../../../.impeccable/review/studio-android.png), [gift and temperament](../../../.impeccable/review/gift-android.png) and [Soul controls](../../../.impeccable/review/soul-android.png). The iPhone 17 Pro simulator shows [studio](../../../.impeccable/review/studio-iOS.png), [gift and temperament](../../../.impeccable/review/gift-iOS.png), [Soul controls](../../../.impeccable/review/soul-iOS.png) and [arrival](../../../.impeccable/review/phone-ios.png). Android rasters are 1200 × 2670; iOS rasters are 1206 × 2622. Detail captures intentionally show scrolled content. Arrival visibly uses the current provisional Bound Sigil; all four original masters remain retained.

The [delivery evidence](../../../docs/verification/reaper-identities.md) records the passing full check (334 shared, 153 Flutter and 40 server tests), studio journeys on the physical Seeker and iOS simulator, widget checks at 320, 390 and 1180 logical pixels, and a 320-pixel check with 2× text. Those widget checks use a substitute stage and establish control/layout behavior rather than GPU rendering. Earlier browser checks exercised calling and colour changes, manual rotation, absorption feedback, independent inspector scrolling and full-potential hollowing; their 1280 × 720 desktop capture is historical, superseded by the current 1180 × 820 packet. Material contract tests bind the authored surface to the renderer's shader template and check upgrade uniforms and explicit failures.

The HTML/CSS detector was run on Dart targets and supplies no useful native verdict; neither its result nor the earlier regex fallback establishes computed contrast. Tablets, physical iPhone behavior, Android cold start, live-backend journeys, wallet trades, release performance and full-game roster/codex/hunt renders remain unproved by this packet. Native captures do not establish physical-phone browser behavior, animation smoothness or Back gestures. The shared codex ledger was inspected in source, not approved through a codex capture.
