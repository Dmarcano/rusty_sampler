mod oscillator;
mod render;
pub mod roland_tr_90;
mod sink;
mod source;
mod spec;

pub use oscillator::sine::SineOscillator;
pub use render::render_source_to_sink;
pub use sink::{AudioSink, WavFileSink};
pub use source::SampleSource;
pub use spec::ToneSpec;
