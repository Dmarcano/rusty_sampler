use std::marker::PhantomData;

use crate::audio::SampleSource;
use dasp_sample::{FromSample, Sample, ToSample};

enum EnvelopeState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct LinearEnvelope<S: Sample> {
    // public vars
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    // state vars
    point: i32,
    sample_rate: f32,
    amplitude_coefficient: f32,
    state: EnvelopeState,
    _data: PhantomData<S>,
}

const MIN_AMPLITUDE: f32 = 0.0001;

impl<S> LinearEnvelope<S>
where
    S: Sample + ToSample<f32> + FromSample<f32>,
{
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        let amplitude_coefficient = MIN_AMPLITUDE;

        Self {
            attack,
            decay,
            sustain,
            release,
            point: 0,
            amplitude_coefficient,
            sample_rate: 44100.0,
            state: EnvelopeState::Off,
            _data: PhantomData,
        }
    }

    // based on
    // https://www.musicdsp.org/en/latest/Synthesis/189-fast-exponential-envelope-generator.html?highlight=envelope
    fn calculate_multiplier(start: f32, end: f32, LengthInSamples: f32) -> f32 {
        todo!()
    }
}

impl<S> Iterator for LinearEnvelope<S>
where
    S: Sample + ToSample<f32> + FromSample<f32>,
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        let out: f32 = match self.state {
            EnvelopeState::Off => MIN_AMPLITUDE,
            EnvelopeState::Attack => todo!(),
            EnvelopeState::Decay => todo!(),
            EnvelopeState::Sustain => todo!(),
            EnvelopeState::Release => todo!(),
        };

        todo!()
    }
}

impl<S> SampleSource<S> for LinearEnvelope<S>
where
    S: Sample + ToSample<f32> + FromSample<f32>,
{
    fn fill_block(&mut self, out: &mut [S]) {
        for (sample, pcm) in out.iter_mut().zip(self) {
            let mul = S::to_sample::<f32>(*sample) * S::to_sample::<f32>(pcm);
            *sample = S::from_sample(mul);
        }
    }
}
