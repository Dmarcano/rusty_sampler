use std::io::Cursor;

pub mod bass;
pub mod high_hat;

fn from_compiled_wav_file(bytes: &'static [u8]) -> Vec<f32> {
    let cursor = Cursor::new(bytes);
    let mut reader = hound::WavReader::new(cursor).unwrap();

    let spec = reader.spec();

    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| {
                // Clamp just in case the file contains values outside normal audio range.
                s.map(|x| x.clamp(-1.0, 1.0))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("failed to read Float sample format"),
        hound::SampleFormat::Int => {
            let bits = spec.bits_per_sample;
            let max_amplitude = 2_f32.powi(bits as i32 - 1);
            reader
                .samples::<i32>()
                .map(|s| {
                    s.map(|x| {
                        let normalized = x as f32 / max_amplitude;
                        normalized.clamp(-1.0, 1.0)
                    })
                })
                .collect::<Result<Vec<_>, _>>()
                .expect("failed to read Int sample format")
        }
    };
    samples
}
