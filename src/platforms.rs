#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod windows;

use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::sync::Arc;

pub const DEFAULT_MIN_FREQ: f32 = 10.0;

pub fn get_max_freq(sample_rate: f32) -> f32 {
    sample_rate / 2f32 - 1000f32
}

pub struct SpectrumAnalyzer {
    bins: usize,
    freq_strengths: Vec<f32>,
    buffer_size: usize,
    buffer: Vec<f32>,
    fft: Arc<dyn Fft<f32>>,
}

impl SpectrumAnalyzer {
    pub fn new(bins: usize, bs: usize) -> SpectrumAnalyzer {
        assert!(bins <= bs);

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(bs);

        SpectrumAnalyzer {
            bins: bins,
            freq_strengths: vec![0.0; bins],
            buffer_size: bs,
            buffer: vec![0.0; bs],
            fft,
        }
    }

    fn put_data(&mut self, data: Vec<f32>) {
        // append as many datapoints to self.buffer as possible
        let fit = (self.buffer_size - self.buffer.len()).min(data.len());
        self.buffer.extend(data[..fit].iter());

        // if full, compute the new spectrum
        if fit < data.len() {
            let mut fft_buffer: Vec<Complex<f32>> = self
                .buffer
                .iter()
                .map(|x| Complex { re: *x, im: 0.0f32 })
                .collect();
            self.fft.process(&mut fft_buffer[..]);
            self.freq_strengths = fft_buffer
                .iter()
                .map(|x| x.norm().powi(2) / self.buffer_size as f32)
                .collect();

            // reset the buffer and append the rest of the samples that werent included before
            self.buffer = data[fit..].to_vec();
        }
    }

    fn get_spectrum(&self) -> &Vec<f32> {
        &self.freq_strengths
    }
}

pub trait FilterBox {
    fn play(&self, spectrum_analyzer: &mut SpectrumAnalyzer) -> Result<(), anyhow::Error>;
    fn set_filter(&self, freq: f32, target_high_pass: bool);
    fn is_finished(&self) -> bool;
    fn finish(&self);
    fn set_volume(&self, factor: u16);
}
