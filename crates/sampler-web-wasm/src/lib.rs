use sampler_core::audio::ToneSpec;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, GainNode, OscillatorNode, OscillatorType};

#[wasm_bindgen]
pub struct SamplerEngine {
    spec: ToneSpec,
    audio_context: Option<AudioContext>,
    gain_node: Option<GainNode>,
    oscillator: Option<OscillatorNode>,
}

#[wasm_bindgen]
impl SamplerEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        Self {
            spec: ToneSpec::default_a440(),
            audio_context: None,
            gain_node: None,
            oscillator: None,
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

        if let Some(oscillator) = &self.oscillator {
            oscillator.frequency().set_value(value);
        }

        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_amplitude(&mut self, value: f32) -> Result<(), JsValue> {
        if !(0.0..=1.0).contains(&value) {
            return Err(JsValue::from_str("amplitude must stay between 0.0 and 1.0"));
        }

        self.spec.amplitude = value;

        if let Some(gain_node) = &self.gain_node {
            gain_node.gain().set_value(value);
        }

        Ok(())
    }

    #[wasm_bindgen]
    pub fn is_playing(&self) -> bool {
        self.oscillator.is_some()
    }

    #[wasm_bindgen]
    pub fn play(&mut self) -> Result<(), JsValue> {
        if self.is_playing() {
            return Ok(());
        }

        let context = match self.audio_context.take() {
            Some(context) => context,
            None => AudioContext::new()?,
        };

        let _ = context.resume()?;

        let oscillator = context.create_oscillator()?;
        oscillator.frequency().set_value(self.spec.frequency_hz);
        oscillator.set_type(OscillatorType::Sine);

        let gain_node = context.create_gain()?;
        gain_node.gain().set_value(self.spec.amplitude);

        oscillator.connect_with_audio_node(&gain_node)?;
        gain_node.connect_with_audio_node(&context.destination())?;
        oscillator.start()?;

        self.audio_context = Some(context);
        self.gain_node = Some(gain_node);
        self.oscillator = Some(oscillator);

        Ok(())
    }

    #[wasm_bindgen]
    pub fn stop(&mut self) -> Result<(), JsValue> {
        if let Some(oscillator) = self.oscillator.take() {
            oscillator.stop()?;
            oscillator.disconnect()?;
        }

        if let Some(gain_node) = self.gain_node.take() {
            gain_node.disconnect()?;
        }

        Ok(())
    }
}

impl Default for SamplerEngine {
    fn default() -> Self {
        Self::new()
    }
}
