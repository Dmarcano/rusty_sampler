import { useEffect, useRef, useState } from "react";

import { loadSamplerModule } from "./wasm/loadSampler";

const DEFAULT_FREQUENCY = 440;
const DEFAULT_AMPLITUDE = 0.2;
const DEFAULT_RETHROW = false;
const WAVEFORM_WIDTH = 560;
const WAVEFORM_HEIGHT = 220;
const WAVEFORM_SAMPLE_COUNT = 512;

function formatErrorDetails(error) {
  if (!error) {
    return ["Unknown error"];
  }

  if (typeof error === "string") {
    return [error];
  }

  const lines = [];
  const name = error.name || error.constructor?.name;
  const message = error.message || String(error);

  lines.push(name ? `${name}: ${message}` : message);

  if (error.stack) {
    lines.push(...String(error.stack).split("\n").slice(1).map((line) => line.trim()));
  }

  if (error.cause) {
    lines.push(`cause: ${String(error.cause)}`);
  }

  return lines;
}

function formatWorkletDebugInfo(debugInfo) {
  if (!debugInfo || typeof debugInfo !== "object") {
    return [];
  }

  return [
    `bindgenUrl: ${debugInfo.bindgenUrl || "missing"}`,
    `polyfillUrl: ${debugInfo.polyfillUrl || "missing"}`,
  ];
}

function drawWaveform(canvas, samples, frequency) {
  const context = canvas.getContext("2d");

  if (!context) {
    return;
  }

  const width = canvas.width;
  const height = canvas.height;
  const midY = height / 2;
  const horizontalPadding = 18;
  const verticalPadding = 18;

  context.clearRect(0, 0, width, height);

  const glow = context.createLinearGradient(0, 0, width, height);
  glow.addColorStop(0, "rgba(255, 202, 125, 0.18)");
  glow.addColorStop(1, "rgba(255, 107, 61, 0.05)");
  context.fillStyle = glow;
  context.fillRect(0, 0, width, height);

  context.strokeStyle = "rgba(255, 255, 255, 0.08)";
  context.lineWidth = 1;
  context.beginPath();
  context.moveTo(horizontalPadding, midY);
  context.lineTo(width - horizontalPadding, midY);
  context.stroke();

  context.strokeStyle = "rgba(255, 180, 136, 0.22)";
  context.beginPath();
  for (let index = 0; index < 5; index += 1) {
    const x = horizontalPadding + ((width - horizontalPadding * 2) * index) / 4;
    context.moveTo(x, verticalPadding);
    context.lineTo(x, height - verticalPadding);
  }
  context.stroke();

  if (!samples.length) {
    context.fillStyle = "rgba(247, 242, 234, 0.72)";
    context.font = '14px "IBM Plex Mono", monospace';
    context.fillText("Press play to stream samples from the worklet.", 20, midY);
    return;
  }

  const usableWidth = width - horizontalPadding * 2;
  const usableHeight = height - verticalPadding * 2;

  context.strokeStyle = "#ffb66d";
  context.lineWidth = 2.5;
  context.beginPath();

  samples.forEach((sample, index) => {
    const x = horizontalPadding + (usableWidth * index) / Math.max(samples.length - 1, 1);
    const y = midY - sample * (usableHeight / 2);

    if (index === 0) {
      context.moveTo(x, y);
    } else {
      context.lineTo(x, y);
    }
  });

  context.stroke();

  context.fillStyle = "rgba(247, 242, 234, 0.72)";
  context.font = '13px "IBM Plex Mono", monospace';
  context.fillText(`${samples.length} samples`, 20, 24);
  context.fillText(`${frequency.toFixed(1)} Hz preview`, width - 176, 24);
}

export default function App() {
  const engineRef = useRef(null);
  const moduleRef = useRef(null);
  const audioContextRef = useRef(null);
  const workletNodeRef = useRef(null);
  const workletLoadedRef = useRef(false);
  const waveformCanvasRef = useRef(null);
  const waveformFrameRef = useRef(null);
  const latestWaveformRef = useRef(new Float32Array(0));
  const [frequency, setFrequency] = useState(DEFAULT_FREQUENCY);
  const [amplitude, setAmplitude] = useState(DEFAULT_AMPLITUDE);
  const [isPlaying, setIsPlaying] = useState(false);
  const [status, setStatus] = useState("Loading Rust audio engine…");
  const [errorLog, setErrorLog] = useState([]);
  const [rethrowErrors, setRethrowErrors] = useState(DEFAULT_RETHROW);
  const [waveformSummary, setWaveformSummary] = useState("Waiting for audio buffers…");

  useEffect(() => {
    let cancelled = false;

    function pushError(source, messageOrError) {
      const lines = Array.isArray(messageOrError)
        ? messageOrError
        : formatErrorDetails(messageOrError);

      setErrorLog((current) => {
        const next = [...current];

        for (const line of lines) {
          next.push(`[${source}] ${line}`);
        }

        return next.slice(-20);
      });
    }

    function onWindowError(event) {
      pushError("window", event.error || event.message || "Unknown window error");
    }

    function onUnhandledRejection(event) {
      pushError("promise", event.reason);
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
          pushError("boot", error);
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

      if (waveformFrameRef.current) {
        cancelAnimationFrame(waveformFrameRef.current);
        waveformFrameRef.current = null;
      }
    };
  }, []);

  useEffect(() => {
    const canvas = waveformCanvasRef.current;

    if (!canvas) {
      return undefined;
    }

    let mounted = true;

    const render = () => {
      if (!mounted) {
        return;
      }

      drawWaveform(canvas, latestWaveformRef.current, frequency);
      waveformFrameRef.current = requestAnimationFrame(render);
    };

    render();

    return () => {
      mounted = false;

      if (waveformFrameRef.current) {
        cancelAnimationFrame(waveformFrameRef.current);
        waveformFrameRef.current = null;
      }
    };
  }, [frequency]);

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

    const debugInfo = moduleRef.current.create_worklet_debug_info();
    setErrorLog((current) => [
      ...current,
      ...formatWorkletDebugInfo(debugInfo).map((line) => `[worklet] ${line}`),
    ].slice(-20));

    if (debugInfo?.bindgenUrl) {
      const response = await fetch(debugInfo.bindgenUrl);
      setErrorLog((current) => [
        ...current,
        `[worklet] bindgen fetch: ${response.status} ${response.statusText}`,
        `[worklet] bindgen content-type: ${response.headers.get("content-type") || "missing"}`,
      ].slice(-20));
    }

    const workletUrl = moduleRef.current.create_worklet_module_url();
    try {
      await context.audioWorklet.addModule(workletUrl);
    } catch (error) {
      console.error("AudioWorklet addModule failed", {
        error,
        stack: error?.stack,
        workletUrl,
        debugInfo,
      });
      throw error;
    }
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
      node.port.onmessage = (event) => {
        const message = event.data ?? {};

        if (message.type !== "waveform" || !message.samples) {
          return;
        }

        const preview = new Float32Array(message.samples);
        const sampleCount = Math.min(preview.length, WAVEFORM_SAMPLE_COUNT);
        latestWaveformRef.current = preview.slice(0, sampleCount);
        setWaveformSummary(
          `Showing ${sampleCount} streamed samples from the worklet render path.`,
        );
      };
      node.connect(context.destination);
      workletNodeRef.current = node;

      setIsPlaying(true);
      setStatus(
        `Playing ${frequency.toFixed(1)} Hz at ${(amplitude * 100).toFixed(0)}% amplitude.`,
      );
    } catch (error) {
      setStatus(error.message);
      setErrorLog((current) => [
        ...current,
        ...formatErrorDetails(error).map((line) => `[play] ${line}`),
      ].slice(-20));
      console.error("Playback failed", error);

      if (rethrowErrors) {
        setTimeout(() => {
          throw error;
        }, 0);
      }
    }
  }

  function handleStop() {
    if (!workletNodeRef.current) {
      return;
    }

    try {
      workletNodeRef.current.disconnect();
      workletNodeRef.current = null;
      latestWaveformRef.current = new Float32Array(0);
      setWaveformSummary("Waiting for audio buffers…");
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

        <section className="scope-panel" aria-label="Signal preview">
          <div className="scope-copy">
            <p className="scope-label">Signal Scope</p>
            <p className="scope-summary">{waveformSummary}</p>
          </div>
          <canvas
            ref={waveformCanvasRef}
            className="scope-canvas"
            width={WAVEFORM_WIDTH}
            height={WAVEFORM_HEIGHT}
          />
        </section>

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

        <label className="control">
          <span>Debug Behavior</span>
          <strong>{rethrowErrors ? "Rethrow enabled" : "Handled in panel"}</strong>
          <input
            type="checkbox"
            checked={rethrowErrors}
            onChange={(event) => setRethrowErrors(event.target.checked)}
          />
        </label>

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
