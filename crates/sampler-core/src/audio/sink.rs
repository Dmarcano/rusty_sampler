use std::fs::File;
use std::io::{self, BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

pub trait AudioSink {
    type Error;

    fn write_block(&mut self, input: &[i16]) -> Result<(), Self::Error>;
}

pub struct WavFileSink {
    writer: BufWriter<File>,
    sample_rate: u32,
    samples_written: u32,
}

impl WavFileSink {
    const HEADER_SIZE_BYTES: u64 = 44;
    const CHANNELS: u16 = 1;
    const BITS_PER_SAMPLE: u16 = 16;

    pub fn create(path: impl AsRef<Path>, sample_rate: u32) -> io::Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Reserve the header so we can backfill the final sizes after rendering.
        writer.write_all(&[0; Self::HEADER_SIZE_BYTES as usize])?;

        Ok(Self {
            writer,
            sample_rate,
            samples_written: 0,
        })
    }

    pub fn finalize(mut self) -> io::Result<()> {
        let data_size_bytes = self.samples_written * u32::from(Self::BITS_PER_SAMPLE / 8);
        let riff_chunk_size = 36 + data_size_bytes;
        let byte_rate =
            self.sample_rate * u32::from(Self::CHANNELS) * u32::from(Self::BITS_PER_SAMPLE / 8);
        let block_align = Self::CHANNELS * (Self::BITS_PER_SAMPLE / 8);

        self.writer.flush()?;
        self.writer.seek(SeekFrom::Start(0))?;

        self.writer.write_all(b"RIFF")?;
        self.writer.write_all(&riff_chunk_size.to_le_bytes())?;
        self.writer.write_all(b"WAVE")?;

        self.writer.write_all(b"fmt ")?;
        self.writer.write_all(&16u32.to_le_bytes())?;
        self.writer.write_all(&1u16.to_le_bytes())?;
        self.writer.write_all(&Self::CHANNELS.to_le_bytes())?;
        self.writer.write_all(&self.sample_rate.to_le_bytes())?;
        self.writer.write_all(&byte_rate.to_le_bytes())?;
        self.writer.write_all(&block_align.to_le_bytes())?;
        self.writer
            .write_all(&Self::BITS_PER_SAMPLE.to_le_bytes())?;

        self.writer.write_all(b"data")?;
        self.writer.write_all(&data_size_bytes.to_le_bytes())?;
        self.writer.flush()
    }
}

impl AudioSink for WavFileSink {
    type Error = io::Error;

    fn write_block(&mut self, input: &[i16]) -> Result<(), Self::Error> {
        for sample in input {
            self.writer.write_all(&sample.to_le_bytes())?;
        }

        self.samples_written = self
            .samples_written
            .checked_add(input.len() as u32)
            .ok_or_else(|| io::Error::other("sample count overflowed u32"))?;

        Ok(())
    }
}
