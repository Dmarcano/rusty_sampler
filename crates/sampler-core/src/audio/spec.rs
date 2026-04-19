#[derive(Debug, Clone, Copy)]
pub struct ToneSpec {
    pub sample_rate: u32,
    pub frequency_hz: f32,
    pub duration_seconds: f32,
    pub amplitude: f32,
    pub block_size: usize,
}

impl ToneSpec {
    pub fn default_a440() -> Self {
        Self {
            sample_rate: 44_100,
            frequency_hz: 440.0,
            duration_seconds: 2.0,
            amplitude: 0.2,
            block_size: 128,
        }
    }

    pub fn total_samples(self) -> usize {
        (self.duration_seconds * self.sample_rate as f32).round() as usize
    }

    pub fn validate(self) -> Result<(), &'static str> {
        if self.sample_rate == 0 {
            return Err("sample_rate must be greater than zero");
        }

        if self.frequency_hz <= 0.0 {
            return Err("frequency_hz must be greater than zero");
        }

        if self.duration_seconds <= 0.0 {
            return Err("duration_seconds must be greater than zero");
        }

        if !(0.0..=1.0).contains(&self.amplitude) {
            return Err("amplitude must be between 0.0 and 1.0");
        }

        if self.block_size == 0 {
            return Err("block_size must be greater than zero");
        }

        Ok(())
    }
}
