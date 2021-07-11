#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "linux")]
pub mod linux;

extern crate anyhow;
extern crate cpal;

pub trait FilterBox {
    fn play(&self) -> Result<(), anyhow::Error>;
    fn set_filter(&self, freq: f32, target_high_pass: bool);
    fn is_finished(&self) -> bool;
    fn finish(&self);
    fn set_volume(&self, factor: u16);
}
