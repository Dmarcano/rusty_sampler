# Rusty Sampler Workspace

This workspace is split into three layers:

- `crates/sampler-core` - portable Rust audio logic and the desktop tone generator
- `crates/sampler-web-wasm` - WebAssembly adapter that uses Web Audio from Rust
- `apps/web-ui` - React + Vite browser UI

## First browser milestone

The first milestone is intentionally small:

- load the Rust/WASM module
- create a sine oscillator from Rust
- control it with Play and Stop buttons in the browser

## Commands

```bash
cargo test -p sampler-core
npm run web:setup
npm run web:dev
```

If you only changed Rust in `sampler-web-wasm`, rebuild the wasm bundle and then restart or refresh the Vite app:

```bash
npm run wasm:build
npm run web:build
```

You can still render the desktop WAV reference tone too:

```bash
cargo run -p sampler-core --bin gen_tone -- --out output/a440.wav
```

The generated WASM package lands in `apps/web-ui/src/wasm/pkg`.

## Shared-memory notes

The AudioWorklet path uses the shared-memory/threaded wasm setup from the
`wasm-bindgen` AudioWorklet example. That means:

- the wasm bundle is built with `cargo +nightly`
- the standard library is rebuilt with atomics enabled
- the browser must be cross-origin isolated

The Vite config now serves the app with the required isolation headers during
`npm run web:dev` and `npm run web:build` preview flows.
