# Recorded combat sounds

All recordings are CC0 1.0 (public domain). Only the selected, trimmed,
peak-normalized mono Vorbis clips ship; no source archive is bundled.
`manifest.json` records the original archive checksums and each selection.

- Pistol shots: Ben Jaszczak and the Free Firearm Sound Library team,
  [The Free Firearm Sound Library](https://opengameart.org/content/the-free-firearm-sound-library).
  Two 1911 takes and one Walther PPQ take, recorded from real firearms.
- Weapon foley and the capture cue: Kenney.nl,
  [RPG Audio](https://kenney.nl/assets/rpg-audio), mirrored by its creator as
  [50 RPG sound effects](https://opengameart.org/content/50-rpg-sound-effects).
- Creature attack tells, shrieks and hurt: rubberduck,
  [80 CC0 creature SFX](https://opengameart.org/content/80-cc0-creature-sfx).
- Reload: SpringySpringo,
  [Gun reload sounds](https://opengameart.org/content/gun-reload-sounds),
  recorded with an airsoft gun.

Rebuild with `cargo xtask build-sfx` (Python 3, FFmpeg with libvorbis, and
libarchive's bsdtar; macOS's tar also works). Downloads stay in the ignored
`art/build/sfx`, or the directory named by `REAPERS_AUDIO_CACHE`.
