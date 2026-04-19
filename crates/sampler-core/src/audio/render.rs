use crate::audio::{AudioSink, SampleSource};

pub fn render_source_to_sink<S, K>(
    source: &mut S,
    sink: &mut K,
    total_samples: usize,
    block_size: usize,
) -> Result<(), K::Error>
where
    S: SampleSource,
    K: AudioSink,
{
    let mut block = vec![0i16; block_size];
    let mut remaining = total_samples;

    while remaining > 0 {
        let current_len = remaining.min(block_size);
        let current_block = &mut block[..current_len];

        source.fill_block(current_block);
        sink.write_block(current_block)?;

        remaining -= current_len;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::audio::{AudioSink, SampleSource, render_source_to_sink};

    struct CountingSource {
        next_sample: i16,
    }

    impl SampleSource for CountingSource {
        fn fill_block(&mut self, out: &mut [i16]) {
            for sample in out {
                *sample = self.next_sample;
                self.next_sample += 1;
            }
        }
    }

    #[derive(Default)]
    struct CollectSink {
        samples: Vec<i16>,
        writes: Vec<usize>,
    }

    impl AudioSink for CollectSink {
        type Error = std::convert::Infallible;

        fn write_block(&mut self, input: &[i16]) -> Result<(), Self::Error> {
            self.writes.push(input.len());
            self.samples.extend_from_slice(input);
            Ok(())
        }
    }

    #[test]
    fn render_writes_all_requested_samples() {
        let mut source = CountingSource { next_sample: 0 };
        let mut sink = CollectSink::default();

        render_source_to_sink(&mut source, &mut sink, 10, 4).unwrap();

        assert_eq!(sink.samples, (0..10).collect::<Vec<_>>());
        assert_eq!(sink.writes, vec![4, 4, 2]);
    }

    #[test]
    fn render_handles_exact_block_boundaries() {
        let mut source = CountingSource { next_sample: 5 };
        let mut sink = CollectSink::default();

        render_source_to_sink(&mut source, &mut sink, 12, 4).unwrap();

        assert_eq!(sink.samples.len(), 12);
        assert_eq!(sink.writes, vec![4, 4, 4]);
        assert_eq!(sink.samples.first(), Some(&5));
        assert_eq!(sink.samples.last(), Some(&16));
    }
}
