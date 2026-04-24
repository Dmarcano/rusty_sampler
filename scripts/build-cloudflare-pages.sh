#!/bin/sh

set -eu

if [ -f "$HOME/.cargo/env" ]; then
  . "$HOME/.cargo/env"
fi

if ! command -v cargo >/dev/null 2>&1; then
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  . "$HOME/.cargo/env"
fi

rustup toolchain install nightly --profile minimal
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo install -f wasm-bindgen-cli --version 0.2.118

npm run web:setup
npm run web:build
