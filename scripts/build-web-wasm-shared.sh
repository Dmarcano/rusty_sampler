#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
OUT_DIR="$ROOT_DIR/apps/web-ui/src/wasm/pkg"
WASM_PATH="$ROOT_DIR/target/wasm32-unknown-unknown/release/sampler_web_wasm.wasm"

export RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals -C link-arg=--shared-memory -C link-arg=--import-memory -C link-arg=--max-memory=1073741824 -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base'

cargo +nightly build \
  -p sampler-web-wasm \
  --target wasm32-unknown-unknown \
  --release \
  -Z build-std=std,panic_abort

wasm-bindgen "$WASM_PATH" \
  --out-dir "$OUT_DIR" \
  --target web
