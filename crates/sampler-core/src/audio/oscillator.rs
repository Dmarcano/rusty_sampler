use crate::audio::{SampleSource, ToneSpec};
use dasp_sample::Sample;
use dasp_signal::{ConstHz, Signal, Sine};

pub struct SineOscillator {
    phase_radians: f32,
    // phase_step_radians: f32,
    amplitude: f32,
    frequency_hz: f32,
    sample_rate: u32,
    sine: dasp_signal::Sine<ConstHz>,
}

impl SineOscillator {
    pub fn new(spec: ToneSpec) -> Self {
        let phase = std::f32::consts::TAU * spec.frequency_hz / spec.sample_rate as f32;
        let mut sine = dasp_signal::rate(spec.sample_rate as f64)
            .const_hz(spec.frequency_hz as f64)
            .sine();

        Self {
            phase_radians: 0.0,
            // phase_step_radians,
            amplitude: spec.amplitude,
            frequency_hz: spec.frequency_hz,
            sample_rate: spec.sample_rate,
            sine,
        }
    }

    // fn update_sine(&mut self) {
    //     self.sine.from_hz_to_hz(self.frequency_hz as f64);
    // }

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

        // let sine_amp = self.sine.next();
        // let pcm = (sine_amp * i16::MAX as f64) // assuming that 16 bit depth is norm. need to update
        //     .round()
        //     .clamp(i16::MIN as f64, i16::MAX as f64);

        self.phase_radians += self.phase_step_radians();

        if self.phase_radians >= std::f32::consts::TAU {
            self.phase_radians -= std::f32::consts::TAU;
        }
        Some(pcm as i16)
    }
}

impl SampleSource<i16> for SineOscillator {
    fn fill_block(&mut self, out: &mut [i16]) {
        for (sample, pcm) in out.iter_mut().zip(self) {
            *sample = pcm;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use crate::audio::{SampleSource, SineOscillator, ToneSpec};
    use dasp_sample::Sample;

    fn oscillator_sample(phase: f32, amplitude: f32) -> i16 {
        let pcm = (phase.sin() * amplitude * i16::MAX as f32)
            .round()
            .clamp(i16::MIN as f32, i16::MAX as f32);

        pcm as i16
    }

    fn wrapped_phase(step: f32, sample_index: usize) -> f32 {
        (sample_index as f32 * step).rem_euclid(TAU)
    }

    fn assert_samples_within_one_lsb(actual: &[i16], expected: &[i16]) {
        assert_eq!(actual.len(), expected.len());

        for (index, (&actual, &expected)) in actual.iter().zip(expected.iter()).enumerate() {
            let error = i32::from(actual) - i32::from(expected);

            assert!(
                error.abs() <= 1,
                "sample {index} differed by {error}: actual={actual}, expected={expected}"
            );
        }
    }

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

    #[test]
    fn oscillator_samples_round_trip_through_f32_within_one_lsb() {
        let spec = ToneSpec {
            amplitude: 0.8,
            ..ToneSpec::default_a440()
        };
        let mut oscillator = SineOscillator::new(spec);
        let mut samples = vec![0i16; 512];

        oscillator.fill_block(&mut samples);

        for sample in samples {
            let round_trip = sample.to_sample::<f32>().to_sample::<i16>();
            let error = i32::from(sample) - i32::from(round_trip);

            assert!(
                error.abs() <= 1,
                "sample {sample} round-tripped to {round_trip} with error {error}"
            );
        }
    }

    #[test]
    fn oscillator_preserves_phase_when_amplitude_changes() {
        let spec = ToneSpec::default_a440();
        let updated_amplitude = 0.65;
        let prefix_len = 137;
        let mut oscillator = SineOscillator::new(spec);
        let mut prefix = vec![0i16; prefix_len];
        let mut actual = vec![0i16; 64];

        oscillator.fill_block(&mut prefix);
        oscillator.set_amplitude(updated_amplitude);
        oscillator.fill_block(&mut actual);

        let step = TAU * spec.frequency_hz / spec.sample_rate as f32;
        let expected = (0..actual.len())
            .map(|offset| {
                let phase = wrapped_phase(step, prefix_len + offset);
                oscillator_sample(phase, updated_amplitude)
            })
            .collect::<Vec<_>>();

        assert_samples_within_one_lsb(&actual, &expected);
    }

    #[test]
    fn oscillator_preserves_phase_when_frequency_changes() {
        let spec = ToneSpec::default_a440();
        let updated_frequency_hz = 660.0;
        let prefix_len = 211;
        let mut oscillator = SineOscillator::new(spec);
        let mut prefix = vec![0i16; prefix_len];
        let mut actual = vec![0i16; 64];

        oscillator.fill_block(&mut prefix);
        oscillator.set_frequency(updated_frequency_hz);
        oscillator.fill_block(&mut actual);

        let old_step = TAU * spec.frequency_hz / spec.sample_rate as f32;
        let new_step = TAU * updated_frequency_hz / spec.sample_rate as f32;
        let phase_at_change = wrapped_phase(old_step, prefix_len);
        let expected = (0..actual.len())
            .map(|offset| {
                let phase = (phase_at_change + (offset as f32 * new_step)).rem_euclid(TAU);
                oscillator_sample(phase, spec.amplitude)
            })
            .collect::<Vec<_>>();

        assert_samples_within_one_lsb(&actual, &expected);
    }
}
