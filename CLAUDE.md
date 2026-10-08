# Calotype — app rules

Private, no-account Android raw editor by Pennylume. Plan: the "Calotype: Android App Plan" Claude Doc
(milestones M0–M6, gates, risks). Studio rules in `../CLAUDE.md` apply.

## Product
- Open core + one-time Pro unlock (Play Billing). Pro is enforced in Kotlin only; the bridge and engine stay unaware of billing.
- Our code is Apache-2.0. Ship the engine's licence texts and NOTICE (`licenses/`, packaged as APK assets).
- Never use the LightCraft, ArtCraft or Adobe names, logos, icons or art in the app, store listing or site.
- No INTERNET permission. Nothing leaves the device.

## Engine
- Upstream engine is a git dependency pinned to a commit in `rust/bridge/Cargo.toml`. Bump on purpose, never float.
- Bridge code never panics (no unwrap/expect/panic outside tests); bad input returns `BridgeError`. Every crash fix gets a test in `rust/bridge/tests/`.
- Previews render at screen size; full resolution only for export.

## Build and test
- `scripts/build-rust.sh [abi]` then `./gradlew assembleDebug --max-workers=1`. Keep `rust/target` (it is the ~20 min cache); `[profile.release]` must match upstream's or the cache misses.
- C: is nearly full: check `df -h /c` before big builds.
- `CALOTYPE_SAMPLES=<dir> cargo test --release --manifest-path rust/Cargo.toml` must pass before a commit.
- Emulator gates on Limrun, phone and tablet: `lim android create --os-version 15 --model phone|tablet --install <apk> --connect`. `LIM_API_KEY` comes from the environment only, never a file. Git Bash: `MSYS_NO_PATHCONV=1` for adb `/sdcard` paths. Screenshot every step; tap via `element-tree`, never guessed coordinates. Delete instances after use.

## Git
- Commit as Pennylume (repo-local config), no attribution trailers. Repo: PennyLume-dev/calotype (public). Check `gh auth status` shows `pennylume` before pushing.
