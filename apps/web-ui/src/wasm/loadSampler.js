let wasmModulePromise;

export async function loadSamplerModule() {
  if (!wasmModulePromise) {
    wasmModulePromise = import("./pkg/sampler_web_wasm.js")
      .then(async (module) => {
        await module.default();
        return module;
      })
      .catch((error) => {
        wasmModulePromise = undefined;

        throw new Error(
          "Rust WASM bundle not found. Run `npm run wasm:build` from the repo root before starting the Vite app.\n\n" +
            error.message,
        );
      });
  }

  return wasmModulePromise;
}
