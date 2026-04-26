use sampler_core::audio::{SineOscillator, ToneSpec};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, AudioWorkletNode};

#[wasm_bindgen]
pub struct SineWorkletNode {
    oscillator: SineOscillator,
    connected: bool,
}

impl SineWorkletNode {
    pub fn new_with_spec(tone_spec: ToneSpec) -> Self {
        let oscillator = SineOscillator::new(tone_spec);
        Self {
            oscillator,
            connected: true,
        }
    }
}

#[wasm_bindgen]
impl SineWorkletNode {
    pub fn new() -> Self {
        Self::new_with_spec(ToneSpec::default_a440())
    }

    pub fn process(&mut self, buf: &mut [f32]) -> bool {
        if !self.connected {
            return true;
        }

        for (sample, pcm) in buf.iter_mut().zip(&mut self.oscillator) {
            let normalized = (pcm as f32 / i16::MAX as f32).clamp(-1.0, 1.0);
            *sample = normalized;
        }
        true
    }

    pub fn pack(self) -> usize {
        Box::into_raw(Box::new(self)) as usize
    }
    pub unsafe fn unpack(val: usize) -> Self {
        unsafe { *Box::from_raw(val as *mut _) }
    }

    pub fn disconnect(&mut self) -> Result<(), JsValue> {
        // self.node.disconnect()
        self.connected = false;
        Ok(())
    }

    pub fn set_frequency_hz_inner(&mut self, value: f32) {
        self.oscillator.set_frequency(value);
    }

    pub fn set_amplitude_inner(&mut self, value: f32) -> () {
        self.oscillator.set_amplitude(value);
    }
}

#[wasm_bindgen]
pub struct SamplerEngine {
    spec: ToneSpec,
}

#[wasm_bindgen]
impl SamplerEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        Self {
            spec: ToneSpec::default_a440(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn frequency_hz(&self) -> f32 {
        self.spec.frequency_hz
    }

    #[wasm_bindgen(getter)]
    pub fn amplitude(&self) -> f32 {
        self.spec.amplitude
    }

    #[wasm_bindgen]
    pub fn set_frequency_hz(&mut self, value: f32) -> Result<(), JsValue> {
        if value <= 0.0 {
            return Err(JsValue::from_str("frequency_hz must be greater than zero"));
        }

        self.spec.frequency_hz = value;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_amplitude(&mut self, value: f32) -> Result<(), JsValue> {
        if !(0.0..=1.0).contains(&value) {
            return Err(JsValue::from_str("amplitude must stay between 0.0 and 1.0"));
        }

        self.spec.amplitude = value;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn create_audio_worklet_node(
        &self,
        ctx: &AudioContext,
    ) -> Result<AudioWorkletNode, JsValue> {
        wasm_audio_node(ctx, self.spec)
    }
}

impl Default for SamplerEngine {
    fn default() -> Self {
        Self::new()
    }
}

// taken from https://github.com/wasm-bindgen/wasm-bindgen/blob/main/examples/wasm-audio-worklet/src/wasm_audio.rs
#[wasm_bindgen(inline_js = "
function buildWorkletDebugInfo() {
    // This inline module is at: snippets/<crate>-<hash>/inline0.js
    // Main module is at: sampler_web_wasm.js (2 levels up)
    const bindgenUrl = new URL('../../sampler_web_wasm.js', import.meta.url).href;
    const polyfillUrl = URL.createObjectURL(new Blob([`
        if (!globalThis.TextDecoder) {
            globalThis.TextDecoder = class TextDecoder {
                decode(arg) {
                    if (typeof arg !== 'undefined') {
                        throw Error('TextDecoder stub called');
                    } else {
                        return '';
                    }
                }
            };
        }

        if (!globalThis.TextEncoder) {
            globalThis.TextEncoder = class TextEncoder {
                encode(arg) {
                    if (typeof arg !== 'undefined') {
                        throw Error('TextEncoder stub called');
                    } else {
                        return new Uint8Array(0);
                    }
                }
            };
        }

        export function nop() {}
    `], { type: 'text/javascript' }));

    const workletSource = `
        import '${polyfillUrl}';
        import * as bindgen from '${bindgenUrl}';

        registerProcessor('SineWorkletNode', class SineWorkletNode extends AudioWorkletProcessor {
            constructor(options) {
                super();
                const processorOptions = options?.processorOptions;
                if (!processorOptions) {
                    throw new Error('Missing processorOptions in AudioWorkletProcessor constructor');
                }
                let [module, memory, handle] = processorOptions;
                bindgen.initSync({ module, memory });
                this.processor = bindgen.SineWorkletNode.unpack(handle);

                this.port.onmessage = (event) => {
                    const message = event.data ?? {};

                    switch (message.type) {
                        case 'setFrequency':
                            this.processor.set_frequency_hz_inner(message.value);
                            break;
                        case 'setAmplitude':
                            this.processor.set_amplitude_inner(message.value);
                            break;
                    }
                };
            }
            process(inputs, outputs) {
                return this.processor.process(outputs[0][0]);
            }
        });
    `;

    return {
        bindgenUrl,
        polyfillUrl,
        workletSource,
    };
}

export function createWorkletModuleUrl() {
    const info = buildWorkletDebugInfo();
    return URL.createObjectURL(new Blob([info.workletSource], { type: 'text/javascript' }));
}

export function createWorkletDebugInfo() {
    return buildWorkletDebugInfo();
}

export function createWorkletNode(ctx, module, memory, handle) {
    return new AudioWorkletNode(ctx, 'SineWorkletNode', {
        processorOptions: [module, memory, handle],
    });
}
")]
extern "C" {
    fn createWorkletModuleUrl() -> String;
    fn createWorkletDebugInfo() -> JsValue;
    fn createWorkletNode(
        ctx: &AudioContext,
        module: JsValue,
        memory: JsValue,
        handle: JsValue,
    ) -> AudioWorkletNode;
}

#[wasm_bindgen]
pub fn create_worklet_module_url() -> String {
    createWorkletModuleUrl()
}

#[wasm_bindgen]
pub fn create_worklet_debug_info() -> JsValue {
    createWorkletDebugInfo()
}

pub fn wasm_audio_node(ctx: &AudioContext, spec: ToneSpec) -> Result<AudioWorkletNode, JsValue> {
    let handle: JsValue = (SineWorkletNode::new_with_spec(spec).pack() as u32).into();

    std::panic::catch_unwind(|| {
        createWorkletNode(ctx, wasm_bindgen::module(), wasm_bindgen::memory(), handle)
    })
    .map_err(|_| {
        JsValue::from_str("AudioWorkletNode construction panicked before returning a node")
    })
}
