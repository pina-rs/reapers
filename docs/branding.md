# Reapers branding

Four imagegen masters live in `art/branding`: **Quiet Cut** (a scythe R and broken halo), **The Veil** (a faceless shroud), **Bound Sigil** (an angular knot) and **Last Ember** (a captured spirit). Quiet Cut is the current default. Generation prompts, mode and the production cleanup direction are in `art/branding/prompts.json`.

The warm ivory emblem on Veil ink keeps the game's Fraunces wordmark and existing palette. The splash opens with the same mark at the same position as the native still, draws one fine reaping arc, then reveals the name. It respects reduced motion and cannot hide loading or recovery indefinitely.

Rebuild from the root, with ImageMagick installed:

```sh
fvm dart tool/build_branding.dart
fvm dart tool/build_branding.dart art/branding/veil.png
```

The Dart tool scales and packs the chosen master, then runs `flutter_launcher_icons` and `flutter_native_splash`. Commit the master, build configuration and generated assets together. Android adaptive and themed icons use a safe inset; the splash's 1152px canvas holds a 720px emblem inside Android 12's safe circle. Android notification small icons are white with alpha and generated at all five phone densities. They are ready for a future notification sender; this change does not add push delivery or ask for notification permission.

Changing native artwork needs a full rebuild and relaunch. Hot restart can review the Flutter motion but cannot replace the installed native splash or launcher icon. Check cold launch on both Android and iOS, with reduced motion enabled, and check the launcher icon in light, dark and themed launchers.

The iOS arrival integration journey uses `fvm dart tool/test_arrival.dart <device-id>` from the root. It installs `com.reapers.app.arrivalTest` with a volatile save, keeping the regular simulator install separate. Do not run this journey directly with `flutter test` against the regular identifier: that harness reinstalls its target. The driver saves the settled threshold capture under `.impeccable/review/phone-ios.png` for review.
