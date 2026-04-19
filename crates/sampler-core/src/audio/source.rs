pub trait SampleSource {
    fn fill_block(&mut self, out: &mut [i16]);
}
