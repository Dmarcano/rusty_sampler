use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use sampler_core::audio::{SampleSource, SineOscillator, ToneSpec};

const BEFORE_HZ: f32 = 440.0;
const AFTER_HZ: f32 = 550.0;
const AMPLITUDE: f32 = 0.8;
const TOTAL_SAMPLES: usize = 256;
const RETUNE_PHASE_RADIANS: f32 = 1.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = output_path();

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let spec = ToneSpec {
        frequency_hz: BEFORE_HZ,
        amplitude: AMPLITUDE,
        duration_seconds: TOTAL_SAMPLES as f32 / 44_100.0,
        ..ToneSpec::default_a440()
    };

    let retune_sample = ((RETUNE_PHASE_RADIANS
        / (std::f32::consts::TAU * BEFORE_HZ / spec.sample_rate as f32))
        .round() as usize)
        .max(1)
        .min(TOTAL_SAMPLES - 1);

    let baseline_440 = render_constant(spec, BEFORE_HZ, TOTAL_SAMPLES);
    let fresh_550 = render_constant(spec, AFTER_HZ, TOTAL_SAMPLES);
    let retuned = render_retuned(spec, BEFORE_HZ, AFTER_HZ, retune_sample, TOTAL_SAMPLES);

    write_csv(&output, &baseline_440, &retuned, &fresh_550)?;

    println!("Wrote comparison data to {}", output.display());
    println!("Retune sample index: {retune_sample}");
    println!(
        "Run: gnuplot -e \"datafile='{}'; retune_sample={retune_sample}; outfile='output/sine-retune.png'\" crates/sampler-core/examples/plot_sine_retune.gp",
        output.display()
    );
    println!("PNG output: output/sine-retune.png");

    Ok(())
}

fn output_path() -> PathBuf {
    env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("output/sine-retune.csv"))
}

fn render_constant(spec: ToneSpec, frequency_hz: f32, total_samples: usize) -> Vec<f32> {
    let mut oscillator = SineOscillator::new(ToneSpec {
        frequency_hz,
        ..spec
    });
    let mut pcm = vec![0i16; total_samples];
    oscillator.fill_block(&mut pcm);
    normalize(&pcm)
}

fn render_retuned(
    spec: ToneSpec,
    before_hz: f32,
    after_hz: f32,
    retune_sample: usize,
    total_samples: usize,
) -> Vec<f32> {
    let mut oscillator = SineOscillator::new(ToneSpec {
        frequency_hz: before_hz,
        ..spec
    });
    let mut pcm = vec![0i16; total_samples];

    oscillator.fill_block(&mut pcm[..retune_sample]);
    oscillator.set_frequency(after_hz);
    oscillator.fill_block(&mut pcm[retune_sample..]);

    normalize(&pcm)
}

fn normalize(samples: &[i16]) -> Vec<f32> {
    samples
        .iter()
        .map(|sample| (*sample as f32 / i16::MAX as f32).clamp(-1.0, 1.0))
        .collect()
}

fn write_csv(
    output: &PathBuf,
    baseline_440: &[f32],
    retuned: &[f32],
    fresh_550: &[f32],
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "sample,baseline_440,retuned_440_to_550,fresh_550")?;

    for index in 0..baseline_440.len() {
        writeln!(
            writer,
            "{index},{},{},{}",
            baseline_440[index], retuned[index], fresh_550[index]
        )?;
    }

    writer.flush()?;
    Ok(())
}
