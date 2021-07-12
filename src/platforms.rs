#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "linux")]
pub mod linux;

pub const DEFAULT_MIN_FREQ: f32 = 10.0;

pub fn get_max_freq(sample_rate: f32) -> f32 {
    sample_rate / 2f32 - 1000f32
}

pub trait FilterBox {
    fn play(&self) -> Result<(), anyhow::Error>;
    fn set_filter(&self, freq: f32, target_high_pass: bool);
    fn is_finished(&self) -> bool;
    fn finish(&self);
    fn set_volume(&self, factor: u16);
}
