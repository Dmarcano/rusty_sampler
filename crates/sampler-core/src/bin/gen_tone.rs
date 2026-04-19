use std::env;
use std::path::PathBuf;
use std::process;

use sampler_core::audio::{SineOscillator, ToneSpec, WavFileSink, render_source_to_sink};

struct Args {
    output: PathBuf,
    spec: ToneSpec,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let mut output = PathBuf::from("output/a440.wav");
        let mut spec = ToneSpec::default_a440();

        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--out" => {
                    output = PathBuf::from(next_value(&mut args, "--out")?);
                }
                "--freq" => {
                    spec.frequency_hz = parse_value(&mut args, "--freq")?;
                }
                "--duration" => {
                    spec.duration_seconds = parse_value(&mut args, "--duration")?;
                }
                "--amp" => {
                    spec.amplitude = parse_value(&mut args, "--amp")?;
                }
                "--sample-rate" => {
                    spec.sample_rate = parse_value(&mut args, "--sample-rate")?;
                }
                "--block-size" => {
                    spec.block_size = parse_value(&mut args, "--block-size")?;
                }
                "--help" | "-h" => {
                    return Err(usage());
                }
                other => {
                    return Err(format!("unknown argument: {other}\n\n{}", usage()));
                }
            }
        }

        spec.validate()
            .map_err(|message| format!("{message}\n\n{}", usage()))?;

        Ok(Self { output, spec })
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("missing value for {flag}\n\n{}", usage()))
}

fn parse_value<T>(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<T, String>
where
    T: std::str::FromStr,
{
    let value = next_value(args, flag)?;

    value
        .parse::<T>()
        .map_err(|_| format!("could not parse value for {flag}: {value}\n\n{}", usage()))
}

fn usage() -> String {
    "Usage: cargo run -p sampler-core --bin gen_tone -- [--out PATH] [--freq HZ] [--duration SECONDS] [--amp 0.0..1.0] [--sample-rate HZ] [--block-size N]".to_string()
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse().map_err(std::io::Error::other)?;

    if let Some(parent) = args.output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let mut source = SineOscillator::new(args.spec);
    let mut sink = WavFileSink::create(&args.output, args.spec.sample_rate)?;

    render_source_to_sink(
        &mut source,
        &mut sink,
        args.spec.total_samples(),
        args.spec.block_size,
    )?;
    sink.finalize()?;

    println!(
        "Wrote {} samples to {}",
        args.spec.total_samples(),
        args.output.display()
    );

    Ok(())
}
