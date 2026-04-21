use crate::audio::{SampleSource, ToneSpec};

pub struct SineOscillator {
    phase_radians: f32,
    // phase_step_radians: f32,
    amplitude: f32,
    frequency_hz: f32,
    sample_rate: u32,
}

impl SineOscillator {
    pub fn new(spec: ToneSpec) -> Self {
        Self {
            phase_radians: 0.0,
            // phase_step_radians,
            amplitude: spec.amplitude,
            frequency_hz: spec.frequency_hz,
            sample_rate: spec.sample_rate,
        }
    }

    fn phase_step_radians(&self) -> f32 {
        std::f32::consts::TAU * self.frequency_hz / self.sample_rate as f32
    }

    pub fn set_frequency(&mut self, frequency_hz: f32) {
        self.frequency_hz = frequency_hz;
    }

    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }
}

impl Iterator for SineOscillator {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let amplitude = self.phase_radians.sin() * self.amplitude;
        let pcm = (amplitude * i16::MAX as f32) // assuming that 16 bit depth is norm. need to update
            .round()
            .clamp(i16::MIN as f32, i16::MAX as f32);

        self.phase_radians += self.phase_step_radians();

        if self.phase_radians >= std::f32::consts::TAU {
            self.phase_radians -= std::f32::consts::TAU;
        }
        Some(pcm as i16)
    }
}

impl SampleSource for SineOscillator {
    fn fill_block(&mut self, out: &mut [i16]) {
        for (sample, pcm) in out.iter_mut().zip(self) {
            *sample = pcm;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::audio::{SampleSource, SineOscillator, ToneSpec};

    #[test]
    fn oscillator_stays_within_requested_amplitude() {
        let spec = ToneSpec {
            amplitude: 0.25,
            ..ToneSpec::default_a440()
        };
        let mut oscillator = SineOscillator::new(spec);
        let mut samples = vec![0i16; spec.total_samples()];
        let expected_peak = (i16::MAX as f32 * spec.amplitude).ceil() as i16;

        oscillator.fill_block(&mut samples);

        assert!(samples.iter().all(|sample| sample.abs() <= expected_peak));
    }

    #[test]
    fn oscillator_keeps_phase_continuous_across_blocks() {
        let spec = ToneSpec::default_a440();
        let mut split_oscillator = SineOscillator::new(spec);
        let mut combined_oscillator = SineOscillator::new(spec);

        let mut split = vec![0i16; 256];
        split_oscillator.fill_block(&mut split[..128]);
        split_oscillator.fill_block(&mut split[128..]);

        let mut combined = vec![0i16; 256];
        combined_oscillator.fill_block(&mut combined);

        assert_eq!(split, combined);
    }

    #[test]
    fn oscillator_is_close_to_a440_over_one_second() {
        let spec = ToneSpec {
            duration_seconds: 1.0,
            ..ToneSpec::default_a440()
        };
        let mut oscillator = SineOscillator::new(spec);
        let mut samples = vec![0i16; spec.total_samples()];

        oscillator.fill_block(&mut samples);

        let crossings = samples
            .windows(2)
            .filter(|window| window[0] <= 0 && window[1] > 0)
            .count();

        assert!((crossings as isize - 440).abs() <= 1);
    }
}
