#!/usr/bin/env python3
"""Build a small recorded CC0 effects bank; never ship the source archives."""

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[2]
SOURCES = {
    "firearms": ("firearms.7z", "https://opengameart.org/sites/default/files/Prepared%20SFX%20Library.7z", "cc1ab5a99a0a365105c7c5dd783f4b0b1fe90938114d3ceec53856bfe005f7d6"),
    "rpg": ("rpg.zip", "https://opengameart.org/sites/default/files/RPGsounds_Kenney.zip", "3ae398ad63e293f9c450bda22d5d81c3af69c74df66fc1400f33c012c0bbc231"),
    "creatures": ("creatures.zip", "https://opengameart.org/sites/default/files/80-CC0-creature-SFX_0.zip", "5d4002eea42f278e76543bb36e8ddd2cb978588a13271984191d8228f5d26768"),
    "reload": ("reload.wav", "https://opengameart.org/sites/default/files/gunreload1.wav", "685cac6a184e3cc3ec09a25b6ddb1d6feffd86d469778f339aa423d384035c50"),
}
CLIPS = [
    ("pistol-a", "firearms", "Prepared SFX Library/1911/A_34P.wav", 1.2),
    ("pistol-b", "firearms", "Prepared SFX Library/1911/A_42P.wav", 1.2),
    ("pistol-c", "firearms", "Prepared SFX Library/Walther PPQ/X_31P.wav", 1.2),
    ("reload", "reload", None, 1.4),
    ("loaded", "rpg", "OGG/metalLatch.ogg", 0.6),
    ("blade", "rpg", "OGG/knifeSlice.ogg", 0.65),
    ("heavy", "rpg", "OGG/chop.ogg", 0.8),
    ("impact", "rpg", "OGG/dropLeather.ogg", 0.5),
    ("parry", "rpg", "OGG/metalPot1.ogg", 0.8),
    ("ape", "creatures", "roar_01.ogg", 1.1),
    ("bird", "creatures", "scream_01.ogg", 0.8),
    ("penguin", "creatures", "grunt_03.ogg", 0.7),
    ("llama", "creatures", "barking_02.ogg", 0.9),
    ("robot", "creatures", "alien_04.ogg", 1.0),
    ("boss", "creatures", "monster_04.ogg", 1.2),
    ("shriek", "creatures", "scream_02.ogg", 1.2),
    ("hurt", "creatures", "hurt_02.ogg", 0.6),
    ("catch", "rpg", "OGG/handleCoins.ogg", 0.7),
]


def sha(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def main():
    cache = Path(os.environ.get("REAPERS_AUDIO_CACHE", ROOT / "art/build/sfx"))
    cache.mkdir(parents=True, exist_ok=True)
    for filename, url, checksum in SOURCES.values():
        path = cache / filename
        if not path.exists():
            pending = path.with_suffix(path.suffix + ".part")
            with urllib.request.urlopen(url, timeout=60) as response, pending.open("wb") as output:
                shutil.copyfileobj(response, output)
            if sha(pending) != checksum:
                raise ValueError(f"Checksum mismatch for {filename}")
            pending.replace(path)
        if sha(path) != checksum:
            raise ValueError(f"Checksum mismatch for {filename}")

    output = ROOT / "assets/sfx"
    output.mkdir(parents=True, exist_ok=True)
    report = []
    with tempfile.TemporaryDirectory(prefix="reapers-sfx-") as directory:
        temporary = Path(directory)
        # Extract only the three named pistol recordings. macOS tar and
        # bsdtar support 7z; on other systems install libarchive's bsdtar.
        members = [member for _, source, member, _ in CLIPS if source == "firearms"]
        subprocess.run([shutil.which("bsdtar") or "tar", "-xf", str(cache / "firearms.7z"), "-C", directory, *members], check=True)
        for name, source, member, duration in CLIPS:
            original = cache / SOURCES[source][0]
            if source == "firearms":
                original = temporary / member
            elif member:
                with zipfile.ZipFile(original) as archive:
                    data = archive.read(member)
                original = temporary / (name + Path(member).suffix)
                original.write_bytes(data)
            trim = f"silenceremove=start_periods=1:start_duration=0.002:start_threshold=-50dB:start_silence=0.01,atrim=duration={duration}"
            measured = subprocess.run(["ffmpeg", "-hide_banner", "-i", str(original), "-af", trim + ",volumedetect", "-f", "null", "-"], capture_output=True, text=True, check=True)
            peak = re.search(r"max_volume: (-?[\d.]+) dB", measured.stderr)
            if peak is None:
                raise ValueError(f"No audible signal in {name}")
            gain = -2.0 - float(peak[1])
            destination = output / (name + ".ogg")
            subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", str(original), "-af", trim + f",volume={gain}dB,afade=t=out:st={duration - 0.05}:d=0.05", "-map_metadata", "-1", "-fflags", "+bitexact", "-flags:a", "+bitexact", "-ac", "1", "-ar", "44100", "-c:a", "libvorbis", "-q:a", "4", str(destination)], check=True)
            report.append({"file": destination.name, "source": source, "member": member, "max_seconds": duration, "sha256": sha(destination), "bytes": destination.stat().st_size})
    (output / "manifest.json").write_text(json.dumps({"sources": SOURCES, "clips": report}, indent=2) + "\n")
    print(f"Built {len(report)} recorded effects, {sum(clip['bytes'] for clip in report):,} bytes.")


if __name__ == "__main__":
    main()
