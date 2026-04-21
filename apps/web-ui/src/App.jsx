import { useEffect, useRef, useState } from "react";

import { loadSamplerModule } from "./wasm/loadSampler";

const DEFAULT_FREQUENCY = 440;
const DEFAULT_AMPLITUDE = 0.2;

export default function App() {
  const engineRef = useRef(null);
  const moduleRef = useRef(null);
  const audioContextRef = useRef(null);
  const workletNodeRef = useRef(null);
  const workletLoadedRef = useRef(false);
  const [frequency, setFrequency] = useState(DEFAULT_FREQUENCY);
  const [amplitude, setAmplitude] = useState(DEFAULT_AMPLITUDE);
  const [isPlaying, setIsPlaying] = useState(false);
  const [status, setStatus] = useState("Loading Rust audio engine…");
  const [errorLog, setErrorLog] = useState([]);

  useEffect(() => {
    let cancelled = false;

    function pushError(source, message) {
      setErrorLog((current) => [...current.slice(-7), `[${source}] ${message}`]);
    }

    function onWindowError(event) {
      pushError("window", event.message || "Unknown window error");
    }

    function onUnhandledRejection(event) {
      const reason =
        typeof event.reason === "string"
          ? event.reason
          : event.reason?.message || String(event.reason);
      pushError("promise", reason);
    }

    window.addEventListener("error", onWindowError);
    window.addEventListener("unhandledrejection", onUnhandledRejection);

    loadSamplerModule()
      .then((wasmModule) => {
        if (cancelled) {
          return;
        }

        moduleRef.current = wasmModule;

        const { SamplerEngine } = wasmModule;
        const engine = new SamplerEngine();
        engine.set_frequency_hz(DEFAULT_FREQUENCY);
        engine.set_amplitude(DEFAULT_AMPLITUDE);

        engineRef.current = engine;
        setStatus("Ready. Press play to hear A440 from the Rust/WASM engine.");
        pushError("boot", "WASM module loaded successfully");
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(error.message);
          pushError("boot", error.message);
        }
      });

    return () => {
      cancelled = true;
      window.removeEventListener("error", onWindowError);
      window.removeEventListener("unhandledrejection", onUnhandledRejection);

      if (engineRef.current) {
        engineRef.current = null;
      }

      if (workletNodeRef.current) {
        workletNodeRef.current.disconnect();
        workletNodeRef.current = null;
      }

      if (audioContextRef.current) {
        audioContextRef.current.close();
        audioContextRef.current = null;
      }
    };
  }, []);

  async function ensureAudioContext() {
    if (!audioContextRef.current) {
      const AudioContextCtor = window.AudioContext || window.webkitAudioContext;

      if (!AudioContextCtor) {
        throw new Error("This browser does not expose AudioContext.");
      }

      audioContextRef.current = new AudioContextCtor();
    }

    await audioContextRef.current.resume();
    return audioContextRef.current;
  }

  async function ensureWorkletModule(context) {
    if (workletLoadedRef.current) {
      return;
    }

    const workletUrl = moduleRef.current.create_worklet_module_url();
    await context.audioWorklet.addModule(workletUrl);
    workletLoadedRef.current = true;
  }

  async function handlePlay() {
    if (!engineRef.current || !moduleRef.current) {
      return;
    }

    try {
      engineRef.current.set_frequency_hz(frequency);
      engineRef.current.set_amplitude(amplitude);

      const context = await ensureAudioContext();
      await ensureWorkletModule(context);

      if (workletNodeRef.current) {
        workletNodeRef.current.disconnect();
      }

      const node = engineRef.current.create_audio_worklet_node(context);
      node.connect(context.destination);
      workletNodeRef.current = node;

      setIsPlaying(true);
      setStatus(
        `Playing ${frequency.toFixed(1)} Hz at ${(amplitude * 100).toFixed(0)}% amplitude.`,
      );
    } catch (error) {
      setStatus(error.message);
      setErrorLog((current) => [...current.slice(-7), `[play] ${error.message}`]);
    }
  }

  function handleStop() {
    if (!workletNodeRef.current) {
      return;
    }

    try {
      workletNodeRef.current.disconnect();
      workletNodeRef.current = null;
      setIsPlaying(false);
      setStatus("Stopped.");
    } catch (error) {
      setStatus(error.message);
    }
  }

  function updateFrequency(event) {
    const value = Number(event.target.value);
    setFrequency(value);

    if (engineRef.current) {
      engineRef.current.set_frequency_hz(value);
    }

    if (workletNodeRef.current) {
      workletNodeRef.current.port.postMessage({
        type: "setFrequency",
        value,
      });
    }
  }

  function updateAmplitude(event) {
    const value = Number(event.target.value);
    setAmplitude(value);

    if (engineRef.current) {
      engineRef.current.set_amplitude(value);
    }

    if (workletNodeRef.current) {
      workletNodeRef.current.port.postMessage({
        type: "setAmplitude",
        value,
      });
    }
  }

  return (
    <main className="shell">
      <section className="panel">
        <p className="eyebrow">Rust + WASM + React</p>
        <h1>Rusty Sampler</h1>
        <p className="lede">
          This first browser milestone keeps the UI in React while the audio
          engine lives in Rust and compiles to WebAssembly.
        </p>

        <div className="controls">
          <label className="control">
            <span>Frequency</span>
            <strong>{frequency.toFixed(1)} Hz</strong>
            <input
              type="range"
              min="110"
              max="880"
              step="1"
              value={frequency}
              onChange={updateFrequency}
            />
          </label>

          <label className="control">
            <span>Amplitude</span>
            <strong>{(amplitude * 100).toFixed(0)}%</strong>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={amplitude}
              onChange={updateAmplitude}
            />
          </label>
        </div>

        <div className="actions">
          <button
            className="primary"
            onClick={handlePlay}
            disabled={isPlaying || !engineRef.current}
          >
            Play
          </button>
          <button
            className="secondary"
            onClick={handleStop}
            disabled={!isPlaying || !engineRef.current}
          >
            Stop
          </button>
        </div>

        <p className="status">{status}</p>

        <section className="debug-panel" aria-label="Debug output">
          <p className="debug-title">Debug</p>
          {errorLog.length === 0 ? (
            <p className="debug-line">No runtime errors captured yet.</p>
          ) : (
            errorLog.map((line, index) => (
              <p className="debug-line" key={`${index}-${line}`}>
                {line}
              </p>
            ))
          )}
        </section>
      </section>
    </main>
  );
}
