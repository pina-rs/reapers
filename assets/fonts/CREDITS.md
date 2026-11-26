# Type

The game is set in three Skribble faces (the art bible, §7). Nothing here was drawn for the game: each is a hand-drawn cut of an open typeface, released under the [SIL Open Font License 1.1](https://openfontlicense.org) with no Reserved Font Name. Each licence file below travels with the faces it covers, in the repository and in every build.

| File | Face | Role | Drawn from | Licence |
| --- | --- | --- | --- | --- |
| `Skribble-Black-Latin.ttf` | Skribble Black | Display: names, stamps, the one primary action | [Recursive](https://github.com/arrowtype/recursive), by Arrow Type | OFL 1.1, [`SkribbleRecursive-LICENSE.txt`](SkribbleRecursive-LICENSE.txt) |
| `SkribbleMonoGentle-Bold-Latin.ttf` | SkribbleMonoGentle Bold | Figures: every number and uppercase label | [Recursive](https://github.com/arrowtype/recursive), by Arrow Type | OFL 1.1, [`SkribbleRecursive-LICENSE.txt`](SkribbleRecursive-LICENSE.txt) |
| `SkribbleNotoSketch-Latin.ttf` | Skribble Noto Sketch | Text: sentences | [Noto Sans](https://github.com/notofonts/latin-greek-cyrillic), by the Noto Project Authors; hand-drawn modifications by the [Skribble authors](https://github.com/openbudgetfun/skribble) | OFL 1.1, [`SkribbleNotoSketch-LICENSE.txt`](SkribbleNotoSketch-LICENSE.txt) |

## Where they come from

The originals are in the Skribble checkout the workspace already pulls in through `leptoskit_skribble`, at revision `20af150`:

| Original | SHA-256 |
| --- | --- |
| `packages/skribble_font_recursive/assets/fonts/Skribble-Black.ttf` | `d5a6279a7ceb0a178c3c1db7cfd3f22fb145d9d46988b7f81fccb871578e81ea` |
| `packages/skribble_font_recursive/assets/fonts/SkribbleMonoGentle-Bold.ttf` | `a881dc34ba86690cdf5631b8a21316dd007da4d4364e1a5de5f5279c93a1275f` |
| `packages/skribble_font_noto_sketch/assets/fonts/SkribbleNotoSketch-Regular.ttf` | `a629254ffc3259f95946d142b8c4f1476827388462cfdaca7e8f01bf712db99e` |

Each is cut down to Latin (Basic Latin, Latin-1, Latin Extended-A, general punctuation, the minus, the euro and the arrows), keeping every OpenType feature, so the Text face's three drawings of each letter still take turns. Together the three come to 557 KB instead of 1.7 MB. The cut was made with fontTools 4.60.1:

```sh
pyftsubset <original> \
  --unicodes="U+0020-007E,U+00A0-00FF,U+0100-017F,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+2000-206F,U+2074,U+20AC,U+2122,U+2190-2193,U+2212,U+2215,U+FEFF,U+FFFD" \
  --layout-features='*' --name-IDs='*' --name-languages='*' --notdef-outline --glyph-names \
  --output-file=<name>-Latin.ttf
```

The OFL allows this: a subset is a Modified Version, and none of the three carries a Reserved Font Name.

Scripts the cut leaves out (Greek, Cyrillic, Devanagari and the rest) fall back, as the bible's chain says, to the platform's Noto Sans. To set a language in one of them, cut it from the same originals rather than shipping a second family.

The landing page is still set in DM Sans and Fraunces (`DMSans.woff2`, `Fraunces.woff2`, each with its own OFL licence file here); the game no longer ships them.

## Where they are credited

The game's colophon screen names all three faces and their licence, and the repository's README lists them in its licence table.
