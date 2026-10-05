# Decision reader verification

Checked on 5 October 2026 from `feat/decision-ledger` using the pinned Flutter 3.47.4 / Dart 3.13.3 SDK.

## Automated evidence

- `fvm dart tool/check.dart` passed after all corrections: formatting, fatal static analysis and **529 tests** (334 shared, 155 Flutter and 40 server). The server tests used their configured embedded PostgreSQL. No live server was started.
- The two new reader widget journeys passed on the VM and again with `fvm flutter test --no-pub --platform chrome test/studio/decision_ledger_test.dart` at **1180 × 844** and **320 × 844 with 2× text**. They verify a level-thirty reaper requires remembered hunts to awaken, a birth draw produces an identity, exact feeding rewards, rounded expectation, clearing an old roll after a scenario change, explanatory semantics and no framework exceptions. These are the same two journeys on another runtime, not two extra distinct tests in the 529 count.
- `fvm flutter build web --no-pub -t lib/main_preview.dart` passed, including the compiler's Wasm dry run. This is a JavaScript web build; it is not a deployed public site or a separately exercised Wasm build.
- `git diff --check` passed.

The initial enlarged-text run exposed overflowing shared ledger rows. `LedgerRow` now stacks its label and value when they cannot fit at the current text scale. Browser inspection exposed missing text semantics from `SelectableText`; text inside `SelectionArea` now makes the explanatory bodies and XP table readable and copyable. The new semantics check passed after that correction. These fixes retain the existing theme and game rules.

A final release-browser functional check exposed a web-only error in both new draw buttons: `1 << 32` compiled to zero as a JavaScript bitwise expression. Both upper bounds now use the explicit `0x100000000` literal. The birth-result assertion was added to both widget and physical-phone journeys, and the widget journeys were run in Chrome to exercise compiled JavaScript. An initial Chrome assertion expected VM-style `2.0` where JavaScript rendered `2`; it now uses the typed feeding enum's numeric formatting while retaining exact odds and XP assertions. Temporary diagnostic reporting was removed after locating that test-only mismatch.

## Attached phone

The decision-reader journey passed on the **physical Android Seeker**, USB device `SM02E4060324957`, Android 16:

```sh
fvm dart tool/test_studio.dart android SM02E4060324957 decisions
```

It opens the real standalone preview, enters the book through the studio action, selects Ferryman / The Anchor, raises level to thirty without gaining an awakening, then adds three Stillness marks and sees awakening one. It draws a birth, selects a clean rare absorption, checks 128/300 XP feeding outcomes and 60 anima, draws an illustrative feeding, and reads the NFT chapter's unimplemented-work notice. The final run passed after the accessibility, prose-width and random-bound corrections.

The existing helper uses a separate studio-test application identifier, preserving the normal game's installation and saves. It restored the Android project after the run; no identifier change is part of the committed diff. The first attempt failed because the test searched for a mixed-case functional heading that the ledger renders in capitals; correcting that locator exposed no product failure. A later widget-test cleanup mistake left a semantics handle active; it was fixed with `finally` disposal rather than weakening the assertion.

Phone screenshots are local evidence under `.impeccable/review/`: `decisions-android.png` (opening), `evolution-android.png` (awakening detail), `chance-android.png` (rare feeding detail) and `nft-plan-android.png` (NFT chapter opening). Each is 1200 × 2670. Scrolled detail captures are deliberately not full-page images.

## Browser observation

The local release preview at `http://127.0.0.1:8093/#/decisions` opened from the studio's book button and survived a refresh at that URL. All four chapters were inspected. The accessible browser tree contained the explanatory bodies and all thirty level-table rows after the selection correction.

After loading the final rebuilt bundle, the birth button visibly produced **Maud the Unheaded · Hollow / Old Hunger · Hungry · None** and the feeding button produced **Rich ×1.25 · 63 XP · 20 anima**. These observations verify local demonstrations, not account rewards or NFT claims. The page was left open at Evolution.

The final web packet is under `.impeccable/review/ledger/`:

| File | View |
| --- | --- |
| `desktop.png` | Evolution opening, 1180 × 820. |
| `mobile.png` | Evolution opening, 390 × 844. |
| `user-3896.png` | Evolution opening at the reported user viewport, 3896 × 1344. |
| `chance-desktop.png` | Scrolled feeding and trust explanation, 1180 × 820. |
| `chance-mobile.png` | Scrolled common feeding controls, 390 × 844. |
| `nft-desktop.png` | NFT chapter opening, 1180 × 820. |
| `decisions-desktop-detail.png` | Scrolled reasons behind the rules, 1180 × 820. |

Every saved capture was opened to confirm its content before the fresh finish review. Browser viewport overrides were reset after verification. The preview tab remains the user-facing local deliverable.

## Finish review

The fresh Impeccable reviewer returned **fix** for one material issue: desktop explanatory paragraphs exceeded a comfortable reading measure. Prose and the Soul entry now use the existing 520-pixel `Veil.contentWidth`, while choices, figures and the progression table retain the wider composition. Web and physical-phone journeys passed again after that correction, and all eleven views were recaptured in place.

The same reviewer's verdict scored the prose-width fix **resolved**, with no visible regression introduced by the correction, and returned **ship**. That verdict covers the scored correction, not a new whole-surface approval or any uncaptured device or flow. Its local records are `.impeccable/review/ledger/review.md` and `verdict.md`.

## Limits

This packet does not establish physical iPhone behaviour, tablet layouts, a physical-phone web browser, assistive-technology traversal, computed contrast, release performance, wallet transactions, real escrow/burning or a live-backend journey. The existing studio's earlier iOS evidence is not an approval of this new reader. Debug phone logs contain GPU/startup timing warnings; a passing journey is not performance evidence.

No Serverpod MCP was available, and Dart tooling reported no running debug app to hot restart after the driver completed. The phone test used a full rebuild and launch; the browser loaded the rebuilt release bundle. No server was started as a workaround. The HTML/CSS design detector has no useful native Dart verdict and was not used to claim visual approval. Fresh craft review supplied the visual floor check at the scope recorded above.
