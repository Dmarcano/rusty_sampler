use super::from_compiled_wav_file;
use crate::audio::SampleSource;

impl SampleSource<f32> for BT0A0A7 {
    fn fill_block(&mut self, out: &mut [f32]) {
        for (sample, pcm) in out.iter_mut().zip(self.samples().iter()) {
            *sample = *pcm;
        }
    }
}

pub struct BT0A0A7 {
    samples: Vec<f32>,
}

impl BT0A0A7 {
    pub fn new() -> Self {
        let bytes = include_bytes!("assets/BT0A0A7.WAV");
        let samples = from_compiled_wav_file(bytes);
        Self { samples }
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }
}
