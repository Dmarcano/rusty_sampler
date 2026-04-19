import { useEffect, useRef, useState } from "react";

import { loadSamplerModule } from "./wasm/loadSampler";

const DEFAULT_FREQUENCY = 440;
const DEFAULT_AMPLITUDE = 0.2;

export default function App() {
  const engineRef = useRef(null);
  const [frequency, setFrequency] = useState(DEFAULT_FREQUENCY);
  const [amplitude, setAmplitude] = useState(DEFAULT_AMPLITUDE);
  const [isPlaying, setIsPlaying] = useState(false);
  const [status, setStatus] = useState("Loading Rust audio engine…");

  useEffect(() => {
    let cancelled = false;

    loadSamplerModule()
      .then(({ SamplerEngine }) => {
        if (cancelled) {
          return;
        }

        const engine = new SamplerEngine();
        engine.set_frequency_hz(DEFAULT_FREQUENCY);
        engine.set_amplitude(DEFAULT_AMPLITUDE);

        engineRef.current = engine;
        setStatus("Ready. Press play to hear A440 from the Rust/WASM engine.");
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(error.message);
        }
      });

    return () => {
      cancelled = true;

      if (engineRef.current) {
        engineRef.current.stop();
        engineRef.current = null;
      }
    };
  }, []);

  async function handlePlay() {
    if (!engineRef.current) {
      return;
    }

    try {
      engineRef.current.set_frequency_hz(frequency);
      engineRef.current.set_amplitude(amplitude);
      engineRef.current.play();

      setIsPlaying(true);
      setStatus(
        `Playing ${frequency.toFixed(1)} Hz at ${(amplitude * 100).toFixed(0)}% amplitude.`,
      );
    } catch (error) {
      setStatus(error.message);
    }
  }

  function handleStop() {
    if (!engineRef.current) {
      return;
    }

    try {
      engineRef.current.stop();
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
  }

  function updateAmplitude(event) {
    const value = Number(event.target.value);
    setAmplitude(value);

    if (engineRef.current) {
      engineRef.current.set_amplitude(value);
    }
  }

  return (
    <main className="shell">
      <section className="panel">
        <p className="eyebrow">Rust + WASM + React</p>
        <h1>Rusty Sampler</h1>
        <p className="lede">
          This first browser milestone keeps the UI in React while the audio engine lives in Rust
          and compiles to WebAssembly.
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
      </section>
    </main>
  );
}
