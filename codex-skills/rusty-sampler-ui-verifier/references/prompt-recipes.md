# Prompt Recipes

Use these as short prompts once the skill is installed, or as copy-paste prompts even if the skill stays repo-local.

## General verification

Run the `rusty_sampler` web UI and verify the current changes. Rebuild wasm only if the changed files require it, start the dev server, open the browser, and report any regressions in the Play/Stop flow, status text, or debug panel.

## Frontend-only smoke test

I only changed files under `apps/web-ui/src`. Skip the wasm rebuild unless you find a concrete reason to run it. Launch the UI, do a quick browser smoke test, and tell me whether the page still reaches Ready and whether Play and Stop still update the status correctly.

## Rust/WASM validation

I changed Rust code for the web path. Rebuild the wasm bundle, relaunch the web UI, and focus on wasm boot, AudioWorklet loading, status changes, and any debug-panel or console errors.

## Pre-commit check

Before I commit, run the fastest reasonable verification for the current diff. Use `npm run web:build` and any other lightweight checks you think are justified, then launch the web UI if browser validation is still needed.

## Regression-focused prompt

Compare the current web UI behavior against the existing milestone expectations in this repo. Call out any behavior regressions first, then summarize what passed.
