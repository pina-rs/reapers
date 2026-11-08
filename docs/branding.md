# Reapers branding

Four imagegen masters live in `art/branding`: **Quiet Cut** (a scythe R and broken halo), **The Veil** (a faceless shroud), **Bound Sigil** (an angular knot) and **Last Ember** (a captured spirit). Bound Sigil is the current default, selected by Ifiok on 5 October 2026. All four masters remain available. Generation prompts, mode and the production cleanup direction are in `art/branding/prompts.json`.

The warm ivory emblem on Veil ink keeps the game's Fraunces wordmark and existing palette. The arrival opens with the mark at the same position as the launch screen's still, draws one fine reaping arc, then reveals the name. It respects reduced motion and cannot hide loading or recovery indefinitely.

## What the game uses

The sized images are in `assets/branding`, packed from Bound Sigil:

| File | Canvas | What it is |
| --- | --- | --- |
| `mark.png` | 512 px, transparent | The mark: the web app's icon and its arrival. |
| `splash_mark.png` | 1152 px, transparent, mark 720 px | The launch still, framed for Android 12's 288 dp artboard: a 180 dp mark inside its 192 dp safe circle. |
| `app_icon.png` | 1024 px on `#07080B`, mark 780 px | The launcher icon. |
| `adaptive_foreground.png` | 1080 px, transparent, mark 600 px | Android's adaptive icon, inside its safe inset. |
| `adaptive_monochrome.png` | 1080 px, white, mark 600 px | Android's themed icon. |

The app's `build.rs` mirrors them into the bundle beside the models. In the browser the mark is `mark.png` and the launch still `splash_mark.png` (`crates/reapers_app/src/brand.rs`); the native hosts cannot name a bundled picture to an image yet, so there the mark is drawn as a vector: the knot's outer strokes, in the same ivory. The native hosts' launcher icons and launch screens are not yet generated from these files.

## Rebuilding from another master

The tool that packed these, and generated the native icon sets from them, went with the first game. The sizes above are made with ImageMagick from a master, trimmed to the mark and centred on its canvas; for example, with The Veil:

```sh
master=art/branding/veil.png
magick "$master" -trim +repage -resize 512x512 -gravity center -background none -extent 512x512 assets/branding/mark.png
magick "$master" -trim +repage -resize 720x720 -gravity center -background none -extent 1152x1152 assets/branding/splash_mark.png
magick "$master" -trim +repage -resize 780x780 -gravity center -background '#07080B' -extent 1024x1024 -alpha remove -alpha off assets/branding/app_icon.png
magick "$master" -trim +repage -resize 600x600 -gravity center -background none -extent 1080x1080 assets/branding/adaptive_foreground.png
magick "$master" -trim +repage -resize 600x600 -channel RGB -fill white -colorize 100 +channel -gravity center -background none -extent 1080x1080 assets/branding/adaptive_monochrome.png
```

Commit the master's choice and the images together, and say which master in this file.

Changing native artwork needs a full rebuild and reinstall of the app. Check a cold launch on both Android and iOS, with reduced motion enabled, and check the launcher icon in light, dark and themed launchers.
