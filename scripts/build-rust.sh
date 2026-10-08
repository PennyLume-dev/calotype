#!/usr/bin/env bash
# Builds the bridge for Android and regenerates its Kotlin bindings.
#   scripts/build-rust.sh            # arm64-v8a + x86_64
#   scripts/build-rust.sh x86_64     # one ABI (emulator)
# Needs: rustup targets aarch64-linux-android x86_64-linux-android, Android NDK r28+.
# CARGO_TARGET_DIR may point at a bigger disk; a cold build takes ~20 min per ABI.
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT=$PWD

SDK=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$LOCALAPPDATA/Android/Sdk}}
NDK=${ANDROID_NDK_HOME:-$(ls -d "$SDK"/ndk/* | sort -V | tail -1)}
case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*) HOST=windows-x86_64; EXT=.cmd ;;
  Darwin) HOST=darwin-x86_64; EXT= ;;
  *) HOST=linux-x86_64; EXT= ;;
esac
TC=$NDK/toolchains/llvm/prebuilt/$HOST/bin
API=26
TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/rust/target}

ABIS=${*:-arm64-v8a x86_64}
for ABI in $ABIS; do
  case $ABI in
    arm64-v8a|arm64) ABI=arm64-v8a; TRIPLE=aarch64-linux-android ;;
    x86_64) TRIPLE=x86_64-linux-android ;;
    *) echo "unknown ABI $ABI" >&2; exit 2 ;;
  esac
  ENV_TRIPLE=$(echo "$TRIPLE" | tr 'a-z-' 'A-Z_')
  export "CARGO_TARGET_${ENV_TRIPLE}_LINKER=$TC/$TRIPLE$API-clang$EXT"
  export "AR_${TRIPLE//-/_}=$TC/llvm-ar"
  export "CC_${TRIPLE//-/_}=$TC/$TRIPLE$API-clang$EXT"
  echo "== $ABI ($TRIPLE)"
  cargo build --manifest-path rust/Cargo.toml -p calotype-bridge --release --target "$TRIPLE"
  LIB="$TARGET_DIR/$TRIPLE/release/libcalotype_bridge.so"
  mkdir -p "app/src/main/jniLibs/$ABI"
  # target/ keeps line tables for crash symbolication; the app gets a stripped copy
  "$TC/llvm-strip$( [ -n "$EXT" ] && echo .exe )" --strip-debug -o "app/src/main/jniLibs/$ABI/libcalotype_bridge.so" "$LIB"
done

echo "== Kotlin bindings"
# uniffi-bindgen reads the crate's uniffi.toml via `cargo metadata`, so run it inside the workspace
(cd rust && cargo run -p uniffi-bindgen --release -- \
  generate --library "$LIB" --language kotlin --no-format --out-dir "$ROOT/app/src/main/java")
