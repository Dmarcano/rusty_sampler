# AudioWorklet Shared-Memory Notes

## Goal

Keep the audio processor and DSP state in Rust while reducing the amount of
browser-native async machinery that has to cross the Rust/WASM boundary.

The immediate design question was whether the `AudioWorkletNode` lifecycle
should stay inside Rust or move back to JavaScript.

## What We Learned So Far

### 1. The original fully Rust-heavy path hit multiple layers of browser/runtime friction

We first tried to keep all of this inside Rust:

- `audioWorklet.addModule(...)`
- `AudioWorkletNode(...)`
- processor bootstrap
- processor state

That exposed several separate issues:

1. `TextDecoder` / `TextEncoder` are not available in the worklet global scope
   in the same way as on the main thread, which required the same style of
   workaround used by the `wasm-bindgen` AudioWorklet example.

2. `processorOptions` initially did not arrive in the processor constructor
   reliably through the original Rust binding path, so we moved node
   construction closer to the plain browser API shape.

3. Passing `WebAssembly.Memory` through `processorOptions` only became viable
   once the project moved to the shared-memory / threaded wasm build path.

   A subtle but important follow-up turned out to be that "shared-memory build
   path" means more than just `-C target-feature=+atomics`.

   The linker/build also needed to:

   - import memory instead of only exporting it
   - mark memory as shared
   - provide a maximum memory size
   - explicitly export TLS/threading symbols used by the wasm-bindgen thread
     transform:
     - `__wasm_init_tls`
     - `__tls_size`
     - `__tls_align`
     - `__tls_base`

   Without those exports, the build could still succeed at the Rust level while
   `wasm-bindgen` later failed to prepare the module for threading.

4. After converting to the shared-memory path, the next failure came from the
   async runtime path rather than from the processor or oscillator itself:

   - `Atomics.waitAsync(...)`
   - `[object Int32Array] is not a shared typed array`

That last error showed that the browser-native async path was still interacting
poorly with the generated Rust async runtime glue.

## Why `addModule(...)` Is the Best Candidate to Move Back to JS

`audioWorklet.addModule(...)` is inherently browser-native and asynchronous.

Even in a Rust-heavy architecture, this step is not DSP logic. It is browser
resource loading and worklet registration.

By moving just this step back to JS, we get:

- clearer control over browser lifecycle
- simpler error reporting
- less dependence on `wasm-bindgen-futures` in the critical path
- less time debugging the async runtime when the real goal is learning the
  worklet and DSP architecture

This still leaves the important parts in Rust:

- processor implementation
- oscillator state
- render loop
- processor message handling

## What Still Needs Async vs What Can Stay Sync

### Must remain async

- `audioContext.resume()`
- `audioWorklet.addModule(...)`
- any browser resource-loading or permission-gated setup step

These are browser lifecycle operations, so keeping them in JS is the clearest
and least surprising option.

### Can remain synchronous

- generating the worklet bootstrap URL in Rust
- constructing the initial Rust processor state
- creating the `AudioWorkletNode` once the module is already registered
- processing audio blocks
- updating oscillator state from messages once the node exists

That means we do not need a Rust future just to create the node itself. The
important boundary is not “Rust versus JS”, but “browser async lifecycle versus
steady-state DSP/control code”.

## Current Recommended Split

### JavaScript owns

- `AudioContext`
- `audioWorklet.addModule(...)`
- `AudioWorkletNode` connection/disconnection
- posting control messages through `node.port`

### Rust owns

- worklet module URL creation
- processor bootstrap code
- processor state (`SineWorkletNode`)
- oscillator render logic
- initial processor configuration

## Why This Is Still a Rust-Heavy Setup

This is not a retreat to a JS audio engine.

It is a narrower and more explicit boundary:

- JS handles browser lifecycle
- Rust handles audio processor logic

That means the architecture is still teaching the right lessons:

- how worklet processors are initialized
- how processor state persists across `process()` calls
- how parameter changes enter the render thread
- how Rust can own the DSP path even if the browser owns the async setup step

## Options We Considered

### Option A: Keep everything in Rust, including async worklet loading

Pros:

- maximal Rust ownership
- elegant in theory

Cons:

- fragile around worklet-global limitations
- more exposure to generated async runtime behavior
- harder to distinguish browser lifecycle errors from DSP errors

### Option B: Move only `addModule(...)` and node lifecycle to JS

Pros:

- still Rust-heavy where it matters
- reduces async/runtime friction
- easier to debug incrementally
- cleaner place to handle browser-specific failures

Cons:

- one more explicit JS boundary
- less aesthetically “pure Rust”

### Option C: JS worklet shell plus Rust called only as a helper

Pros:

- easiest to stabilize

Cons:

- gives up too much of the Rust-owned processor model
- not as aligned with the learning goal

## Chosen Direction

Option B.

We keep:

- Rust processor
- Rust oscillator
- Rust worklet bootstrap

We move:

- `audioWorklet.addModule(...)`
- `AudioContext` setup
- `AudioWorkletNode` lifecycle

to JavaScript.

In practice, that means:

1. JS awaits worklet registration.
2. Rust provides the worklet module URL and processor implementation.
3. JS creates the node after registration.
4. Rust stays focused on render-time state and DSP.

## Remaining Open Questions

1. Do we keep using packed Rust handles through `processorOptions`, or do we
   eventually prefer a cleaner worklet-owned construction model?

2. Should live parameter updates continue to use `node.port.postMessage(...)`,
   or should frequency/amplitude eventually become `AudioParam`s?

3. When DASP enters the picture, do we keep a single Rust worklet processor
   wrapping an internal graph, or expose smaller units of DSP with a thinner
   processor wrapper?

## Near-Term Plan

1. Keep the processor in Rust.
2. Let JS await `addModule(...)`.
3. Let JS create/connect/disconnect the `AudioWorkletNode`.
4. Use `port.postMessage(...)` for frequency and amplitude updates.
5. Once stable, revisit:
   - DASP integration
   - `AudioParam` support
   - graph structure inside the Rust processor
