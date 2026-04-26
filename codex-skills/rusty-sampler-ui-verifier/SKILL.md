---
name: rusty-sampler-ui-verifier
description: Use when working in the rusty_sampler repository and the goal is to run, rebuild, inspect, or verify the local React + Vite web UI after Rust/WASM or frontend changes. Trigger for requests to launch the app, smoke-test the Play and Stop flow, rebuild wasm after Rust edits, or validate browser behavior before or after code changes.
---

# Rusty Sampler UI Verifier

Use this skill from the `rusty_sampler` repo root to keep browser verification repeatable and lightweight.

## Working assumptions

- The repo root contains `package.json`, `apps/web-ui`, `crates/sampler-web-wasm`, and `crates/sampler-core`.
- The dev server runs at `http://127.0.0.1:5173`.
- The current browser milestone is the React page in `apps/web-ui/src/App.jsx` with:
  - heading `Rusty Sampler`
  - a status line that should reach `Ready. Press play to hear A440 from the Rust/WASM engine.`
  - `Play` and `Stop` buttons
  - `Frequency` and `Amplitude` sliders
  - a `Debug` panel for runtime output
- `apps/web-ui/vite.config.js` already sets the cross-origin isolation headers needed by the AudioWorklet flow. Preserve the configured host and port unless the user asks otherwise.

## Verification flow

1. Inspect the changed files first with `git status --short` and, when useful, `git diff --name-only`.
2. Choose the lightest prep step that matches the change:
   - Frontend-only changes in `apps/web-ui/src/**`: usually go straight to `npm run web:dev`.
   - Changes in `crates/sampler-web-wasm/**`, `crates/sampler-core/**`, or wasm glue expectations: run `npm run wasm:build` before launching the UI.
   - Fresh clone, missing dependencies, or missing wasm artifacts: run `npm run web:setup`.
3. Run a fast non-browser check when it adds confidence:
   - `npm run web:build` for frontend bundling
   - `cargo test -p sampler-core` when core audio logic changed
4. Start the dev server with `npm run web:dev` in a PTY and wait for the local URL.
5. Use the in-app browser to open `http://127.0.0.1:5173`.
6. Verify the visible behavior:
   - the page renders without a blank screen
   - the heading and controls appear
   - the status reaches the ready message after wasm boot
   - `Play` becomes usable and updates the status to `Playing ...`
   - `Stop` returns the status to `Stopped.`
   - the debug panel does not show unexpected runtime failures
7. Report what was verified, whether wasm was rebuilt, and anything that still needs a manual listen test.

## Audio-specific guidance

- Browser audio playback requires a user gesture, so the `Play` click is part of the verification flow.
- UI state, debug output, and runtime errors are the main automation-friendly signals. Treat audible quality as a separate manual validation step unless the user specifically asks for listening checks.
- If playback fails, inspect the debug panel and worklet/module-loading errors before changing unrelated UI code.

## Common failure paths

- If `npm run web:dev` fails because dependencies are missing, run `npm run web:install` or `npm run web:setup`.
- If the page loads but the wasm behavior looks stale after Rust edits, rerun `npm run wasm:build`.
- If AudioWorklet loading fails, inspect `apps/web-ui/src/wasm/loadSampler.js`, `crates/sampler-web-wasm/src/lib.rs`, and the generated `apps/web-ui/src/wasm/pkg/**` outputs together.
- Do not hand-edit `apps/web-ui/src/wasm/pkg/**` unless the user explicitly asks for generated file changes.

## Prompt recipes

See [references/prompt-recipes.md](references/prompt-recipes.md) for reusable prompts.
