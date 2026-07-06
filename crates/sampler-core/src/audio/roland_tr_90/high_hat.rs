use super::from_compiled_wav_file;
use crate::audio::SampleSource;

impl SampleSource<f32> for HHCD2 {
    fn fill_block(&mut self, out: &mut [f32]) {
        for (sample, pcm) in out.iter_mut().zip(self.samples().iter()) {
            *sample = *pcm;
        }
    }
}

pub struct HHCD2 {
    samples: Vec<f32>,
}

impl HHCD2 {
    pub fn new() -> Self {
        let bytes = include_bytes!("assets/HHCD2.WAV");
        let samples = from_compiled_wav_file(bytes);
        Self { samples }
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }
}
