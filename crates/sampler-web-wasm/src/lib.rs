use sampler_core::audio::{SampleSource, SineOscillator, ToneSpec};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioContext, AudioWorkletNode, AudioWorkletNodeOptions};

#[wasm_bindgen]
struct SineWorkletNode {
    oscillator: SineOscillator,
    connected: bool,
}

#[wasm_bindgen]
impl SineWorkletNode {
    pub fn new() -> Self {
        let tone_spec = ToneSpec::default_a440();
        let oscillator = SineOscillator::new(tone_spec);
        Self {
            oscillator,
            connected: true,
        }
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
    audio_context: Option<AudioContext>,
    node: Option<AudioWorkletNode>,
    // oscillator: Option<SineOscillator>,
}

#[wasm_bindgen]
impl SamplerEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        Self {
            spec: ToneSpec::default_a440(),
            audio_context: None,
            node: None,
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

        if let Some(node) = &mut self.node {
            // TODO: update frequency_hz on node via some param
        }

        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_amplitude(&mut self, value: f32) -> Result<(), JsValue> {
        if !(0.0..=1.0).contains(&value) {
            return Err(JsValue::from_str("amplitude must stay between 0.0 and 1.0"));
        }

        self.spec.amplitude = value;

        if let Some(node) = &mut self.node {
            // TODO: update amplitude on node via some param
        }

        Ok(())
    }

    #[wasm_bindgen]
    pub fn is_playing(&self) -> bool {
        self.node.is_some()
    }

    #[wasm_bindgen]
    pub async fn play(&mut self) -> Result<(), JsValue> {
        if self.is_playing() {
            return Ok(());
        }

        let context = match self.audio_context.take() {
            Some(context) => context,
            None => AudioContext::new()?,
        };

        let _ = context.resume()?;

        // let oscillator = context.create_oscillator()?;
        // oscillator.frequency().set_value(self.spec.frequency_hz);
        // oscillator.set_type(OscillatorType::Sine);

        prepare_wasm_audio(&context).await?;
        let node = wasm_audio_node(&context)?;
        node.connect_with_audio_node(&context.destination())?;

        // let gain_node = context.create_gain()?;
        // gain_node.gain().set_value(self.spec.amplitude);

        // oscillator.connect_with_audio_node(&gain_node)?;
        // gain_node.connect_with_audio_node(&context.destination())?;
        // oscillator.start()?;

        self.audio_context = Some(context);
        self.node = Some(node);

        // self.gain_node = Some(gain_node);
        // self.oscillator = Some(oscillator);

        Ok(())
    }

    #[wasm_bindgen]
    pub fn stop(&mut self) -> Result<(), JsValue> {
        if let Some(node) = self.node.take() {
            // oscillator.stop()?;
            node.disconnect()?;
        }

        // if let Some(gain_node) = self.gain_node.take() {
        //     gain_node.disconnect()?;
        // }

        Ok(())
    }
}

impl Default for SamplerEngine {
    fn default() -> Self {
        Self::new()
    }
}

// taken from https://github.com/wasm-bindgen/wasm-bindgen/blob/main/examples/wasm-audio-worklet/src/wasm_audio.rs
#[wasm_bindgen(inline_js = "
export function createWorkletModuleUrl() {
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

    return URL.createObjectURL(new Blob([`
        import '${polyfillUrl}';
        import * as bindgen from '${bindgenUrl}';

        registerProcessor('SineWorkletNode', class SineWorkletNode extends AudioWorkletProcessor {
            constructor(options) {
                super();
                let [module, memory, handle] = options.processorOptions;
                bindgen.initSync({ module, memory });
                this.processor = bindgen.SineWorkletNode.unpack(handle);
            }
            process(inputs, outputs) {
                return this.processor.process(outputs[0][0]);
            }
        });
    `], { type: 'text/javascript' }));
}
")]
extern "C" {
    fn createWorkletModuleUrl() -> String;
}

pub fn wasm_audio_node(ctx: &AudioContext) -> Result<AudioWorkletNode, JsValue> {
    let options = AudioWorkletNodeOptions::new();
    options.set_processor_options(Some(&js_sys::Array::of(&[
        wasm_bindgen::module(),
        wasm_bindgen::memory(),
        SineWorkletNode::new().pack().into(),
    ])));
    AudioWorkletNode::new_with_options(ctx, "SineWorkletNode", &options)
}

pub async fn prepare_wasm_audio(ctx: &AudioContext) -> Result<(), JsValue> {
    let mod_url = createWorkletModuleUrl();
    JsFuture::from(ctx.audio_worklet()?.add_module(&mod_url)?).await?;
    Ok(())
}
