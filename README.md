# Calotype

A private raw photo editor for Android. No account, no subscription, works offline: your photos never leave the phone.

Calotype is a native Kotlin + Jetpack Compose app. All image work is done by an open-source Rust raw engine
(see [NOTICE](NOTICE)), reached through a small bridge crate with [UniFFI](https://mozilla.github.io/uniffi-rs/) bindings.

**Status:** milestone 1 (bridge). The app opens one raw file and shows a developed preview. Editing, library,
export and presets come next.

## Layout

| Path | What |
| --- | --- |
| `rust/bridge` | `calotype-bridge` cdylib: `Editor` with `import`, `list`, `preview(size)`, `setControls`, `undo`, `export` |
| `rust/uniffi-bindgen` | Generates the Kotlin bindings from the built library |
| `app` | The Android app (Compose, single activity) |
| `scripts/build-rust.sh` | Builds the bridge for `arm64-v8a` and `x86_64` and regenerates the bindings |
| `licenses/` | Engine licence texts and NOTICE, packaged into the APK |

## Build

Needs Rust 1.90+, `rustup target add aarch64-linux-android x86_64-linux-android`, the Android SDK with NDK r28+,
and JDK 17+. Works in Git Bash on Windows, and on macOS or Linux.

```sh
scripts/build-rust.sh                    # both ABIs; or: scripts/build-rust.sh x86_64
./gradlew assembleDebug --max-workers=1  # or assemblePreview for a release-like build
```

A cold Rust build takes about 20 minutes per ABI; later builds reuse `rust/target` (set `CARGO_TARGET_DIR` to keep
it on another disk).

## Test

```sh
CALOTYPE_SAMPLES=/path/to/samples cargo test --release --manifest-path rust/Cargo.toml
```

`CALOTYPE_SAMPLES` is a folder holding `pixel6pro.dng` (CC0, from [raw.pixls.us](https://raw.pixls.us/)). Without
it the real-photo tests skip and only the corrupt-input tests run. Release builds are checked on emulator phones
and tablets before they ship.

## Licence

Apache-2.0 for Calotype's own code. The engine is MIT OR Apache-2.0; see [NOTICE](NOTICE).
