use crate::audio::{SampleSource, ToneSpec};

pub struct SineOscillator {
    phase_radians: f32,
    phase_step_radians: f32,
    amplitude: f32,
}

impl SineOscillator {
    pub fn new(spec: ToneSpec) -> Self {
        let phase_step_radians =
            std::f32::consts::TAU * spec.frequency_hz / spec.sample_rate as f32;

        Self {
            phase_radians: 0.0,
            phase_step_radians,
            amplitude: spec.amplitude,
        }
    }
}

impl SampleSource for SineOscillator {
    fn fill_block(&mut self, out: &mut [i16]) {
        for sample in out {
            let amplitude = self.phase_radians.sin() * self.amplitude;
            let pcm = (amplitude * i16::MAX as f32)
                .round()
                .clamp(i16::MIN as f32, i16::MAX as f32);

            *sample = pcm as i16;

            self.phase_radians += self.phase_step_radians;

            if self.phase_radians >= std::f32::consts::TAU {
                self.phase_radians -= std::f32::consts::TAU;
            }
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
