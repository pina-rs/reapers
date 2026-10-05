# Reaper identities: delivery evidence

Verified 5 October 2026 on Flutter 3.47.4 / Dart 3.13.3. This report covers the
birth-gift, memory and awakening extension, selected branding and launch guards.
It is not a production launch certificate. See [launch readiness](../launch-readiness.md)
for outstanding engineering and the inputs needed from Ifiok.

## Automated checks

`fvm dart tool/check.dart` passed after the final application corrections:

- Formatter and static analysis, including fatal infos, in the tools and all
  three handwritten packages.
- 334 shared-rule tests, 153 Flutter tests and 40 server tests: **527 total**.
- The gift test matrix exercises all 18 activation triggers through simulation
  inputs, rather than replacing the simulation with mocks. It also covers
  bounded protection, genuine landed-hit memory and awakened defensive effects.
- Soul tests cover permanent birth mapping, all 72 mechanical birth builds,
  rank gates, per-reaper settlement, legacy saves, serialization and replay.
- Server integration tests cover unsupported combat-log versions without state
  mutation and preservation of earned memory across verified NFT ownership
  transfer. The server tests use embedded PostgreSQL and fake chain responses;
  they do not establish a live RPC or mainnet transaction.
- Studio widget tests cover 320-, 390- and 1180-pixel widths, 2× text at 320,
  level without memory, earned awakening, gift changes, full potential and
  isolation from saved progression.

## Running application

The isolated studio journey passed on the physical **Solana Seeker** and an
**iPhone 17 Pro simulator (iOS 26.5)**. It loads the real renderer, chooses
Reprisal, raises the level without awakening, then earns three illustrative
Stillness memories and observes the first awakening without replacing the
birth gift. Test application identifiers keep the normal installation and
its saved reapers separate. Run with `fvm dart tool/test_studio.dart android
<device-id>` or the corresponding `ios` command.

The isolated iOS arrival journey also passed with Bound Sigil and the finite
arrival gate. This does not establish every Android cold-start frame or an
installed App Store release.

The release browser studio built successfully with `lib/main_preview.dart`.
Its actual WebGL reaper and Soul inspector were inspected at 1180×820 and
390×844. The preview runs locally; it is not publicly hosted. Native and web
review captures live in the ignored `.impeccable/review/` directory. Preserve
the textual design record rather than depending on those local files being
available in a future clone.

The fresh design reviewer initially returned **fix** for three findings:
repeated category labels above the gift/temperament, a truncated gift selector
heading and stale design documentation. One correction batch was rebuilt and
recaptured. Its verdict returned **ship**, scoring all three **resolved** with
no visible regression from that batch. This result covers those corrections,
not whole-surface approval or any uncaptured device or game flow. The durable
record is in the Flutter package's `DESIGN.md` and studio surface brief.

## Reproducible pacing and artwork

[Soul pacing trials](../research/soul-pacing-trials.md) contain **1,152**
deterministic encounters across 72 builds, eight seeds and two growth stages.
The fixed rare-husk scenario reveals substantial differences at birth.
Awakened builds improve, but these bot results do not establish human enjoyment,
physical comfort, beginner balance or cooperative fairness.

All four logo masters remain unchanged. Bound Sigil is the selected generated
application artwork. A provenance scan covered all 86 shipped branding and
platform PNGs and found no missing entries. Platform generation now restores
provenance after the launcher/splash generators strip source metadata.

## Release failure paths and limits

An Android app-bundle release build with no signing values failed as intended,
listing the four missing configuration names without exposing values. This
proves the guard; it is not a successful signed release build. Debug builds and
iOS simulator builds passed without accessing deployment secrets.

Staging/production simulation refusal is tested. Combat logs now identify
rules version 5 and reject impossible directions or overlong encounters.
There are no table changes or manually edited Serverpod generated files.

The Serverpod MCP connection was unavailable, so no live backend was started,
reloaded or deployed. No real NFT acquisition, escrow, burn, public hosting,
store submission or mainnet transaction was performed. Physical iPhone,
tablet, camera/location lifecycle and release battery/thermal measurements
remain unverified by this delivery. Debug startup frame warnings are not
production performance evidence.
