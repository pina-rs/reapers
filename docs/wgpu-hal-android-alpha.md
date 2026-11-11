# The Android alpha backport in wgpu-hal

Reapers draws its arms in a transparent `Scene3d` above the world scene. On
Android, wgpu-hal 30.0.1 could give that surface no alpha channel, and the
window composited it as opaque black: it hid the camera and the world husk
while its own arms and the HUD still drew.

## Why 30.0.1 does not do it

`wgpu-hal`'s `src/gles/egl.rs` adds `EGL_ALPHA_SIZE = 8` to its
`eglChooseConfig` attribute list only when it has an sRGB framebuffer
(`SrgbFrameBufferKind` is not `None`). A device whose EGL is 1.4 and does not
advertise `EGL_KHR_gl_colorspace` — the Android 16 API 36 emulator with
ANGLE/SwiftShader is one — has `srgb_kind = None`. The first config returned
then has `EGL_ALPHA_SIZE = 0` and `EGL_NATIVE_VISUAL_ID = RGBX_8888`.
`Instance::create_surface` writes that visual id to the `ANativeWindow`, so
every frame is RGBX and `TextureView.isOpaque = false` has no alpha to blend.

The newest wgpu-hal release is still 30.0.1, and `gfx-rs/wgpu` trunk still has
the conditional as of 2026-11-11, so no published release fixes it.

## The vendored package

| | |
| --- | --- |
| Release | `wgpu-hal` 30.0.1 |
| Package | `https://static.crates.io/crates/wgpu-hal/wgpu-hal-30.0.1.crate` |
| SHA-256 | `b6b7fb58561a792bc237628ba0792e332de418fefe145f13b5ed8201e6d52f58` — the checksum `Cargo.lock` records for the registry package |
| Upstream source commit | `40f4a34ebaf56f9a046231f54125ad046239d3f3`, from the package's `.cargo_vcs_info.json` |
| Licence | MIT OR Apache-2.0, with `LICENSE.MIT` and `LICENSE.APACHE` kept |

`vendor/wgpu-hal` is that package unpacked byte for byte, except for the hunk
below. `Cargo.toml` is Cargo's normalized crates.io manifest, and the published
`Cargo.lock`, `Cargo.toml.orig`, sources, examples and licences are all
unedited. The crate is not a workspace member, so `cargo fmt --all` and the
workspace lints never rewrite it.

## The local change

`src/gles/egl.rs`, in `choose_config`: the empty `None` arm exists only when
the target is not Android. Android therefore always asks for eight alpha bits,
whether or not it has sRGB; every other target keeps the old selection.

```diff
         match srgb_kind {
+            #[cfg(not(target_os = "android"))]
             SrgbFrameBufferKind::None => {}
             _ => {
                 attributes.push(khronos_egl::ALPHA_SIZE);
                 attributes.push(8);
             }
         }
```

The root `Cargo.toml` binds it with `wgpu-hal = { path = "vendor/wgpu-hal" }`
under `[patch.crates-io]`, so the app and the generated Android host both
resolve the one package to this source.

## Removing it

Delete this note and `vendor/wgpu-hal`, drop the `wgpu-hal` line from
`[patch.crates-io]`, and run `cargo update -p wgpu-hal` when the release
Reapers resolves carries the fix — Android requesting alpha when
`srgb_kind` is `None` — or when the arms no longer need a second, transparent
surface. Read the release's `src/gles/egl.rs` before deciding.

## The proof

The fixture is a leptoskit app whose home screen is the hunt's shape: a blue
root, a lower `Scene3d` with a green backdrop and a model at x = +0.85, a
second `Scene3d` with `Backdrop::Transparent` and a model at x = −0.85, and a
text/button HUD over both. Its button unmounts and remounts the upper scene,
so attach, detach and reopen are all exercised. The fixture is not committed:
it is scaffolding around this one check.

On `emulator-5556` (Android 16 / API 36, arm64, ANGLE over SwiftShader, EGL
1.4 without `EGL_KHR_gl_colorspace`), both builds ran the same fixture and the
same framework; only `wgpu-hal` differed.

| build | upper scene | (20, 600) | (300, 1200) | (800, 1500) |
| --- | --- | --- | --- | --- |
| published 30.0.1 | attached | `(0, 0, 0)` | upper model | `(0, 0, 0)` |
| published 30.0.1 | detached | `(0, 255, 0)` | `(0, 255, 0)` | lower model |
| this change | attached | `(0, 255, 0)` | upper model | lower model |
| this change | detached | `(0, 255, 0)` | `(0, 255, 0)` | lower model |

The detached rows matter: with the published crate, removing the transparent
`Scene3d` reveals the lower green scene and its model, so the black rectangle
belongs to the transparent surface, not to stacking order or the lower scene.
With this change the transparent scene draws its model over the lower scene
while the lower scene, both models and the HUD stay visible; detaching and
reattaching it (`Scene3d` is unmounted and a new surface is created) keeps the
same result. The EGL probe on the device lists the alpha-less config
(`id=1 alpha=0 visual=2`) first without `EGL_ALPHA_SIZE`, and
`id=2 alpha=8 visual=1` first with it.

This was checked on the emulator only. A phone whose EGL is 1.5, or that
advertises `EGL_KHR_gl_colorspace`, already selected an alpha config before
the change.
