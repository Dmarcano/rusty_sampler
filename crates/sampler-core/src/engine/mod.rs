use crate::audio::SampleSource;

#[derive(Debug, Clone)]
struct Sample(Vec<f32>);

// A musical track is a vector of samples
#[derive(Debug, Clone)]
pub struct Track {
    capacity: usize,
    samples: [Sample; 16],
    curr_idx: usize,
}

// lets assume that a track is 4 steps, fill one with nothing, one with rest
impl Track {
    pub fn push_sample(&mut self, index: usize, block: &[f32]) {
        let owned: Vec<f32> = block.into();
        self.samples[index] = Sample(owned);
    }

    fn increment(&mut self) {
        let mut idx = self.curr_idx;
        idx += 1;

        if (idx >= self.capacity) {
            idx = 0;
        }
        self.curr_idx = idx;
    }

    pub fn new() -> Self {
        Self {
            capacity: 4,
            curr_idx: 0,
            samples: [
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
                Sample(vec![]),
            ],
        }
    }
}

impl SampleSource<f32> for Track {
    fn fill_block(&mut self, out: &mut [f32]) {
        let samples = &self.samples[self.curr_idx];
        for (sample, pcm) in out.iter_mut().zip(samples.0.iter()) {
            *sample = *pcm;
        }
        self.increment();
    }
}
