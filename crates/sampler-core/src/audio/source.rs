use dasp_sample::Sample;

pub trait SampleSource<S: Sample> {
    fn fill_block(&mut self, out: &mut [S]);
}
