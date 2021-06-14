pub mod implementations {
    //use std::io::{Read, Seek};
    use std::time::Duration;
    use rodio::Source;

    pub struct InputStreamWrapper//<R>
        //where
        //R: Read + Seek,
    {
        pub data: Vec<f32>,//<R>,
        pub current_offset: usize,
        pub config: StreamConfig,
    }

    //impl<R> InputStreamWrapper<R>
    impl InputStreamWrapper
    {
        pub fn new(dat: Vec<f32>, conf: StreamConfig) -> InputStreamWrapper {
            InputStreamWrapper{data: dat, current_offset: 0, config: conf}
        }
    }

    //impl<R> Iterator for InputStreamWrapper<R>
    impl Iterator for InputStreamWrapper
        //where
        //R: Read + Seek,
    {
        type Item = f32;

        fn next(&mut self) -> Option<f32> {
            if self.current_offset >= self.data.len() {
                return None
            }
            else {
                let sample = self.data[self.current_offset];
                self.current_offset += 1;
                return Some(sample)
            }
        }
    }

    //impl<R> Source for InputStreamWrapper<R>
    impl Source for InputStreamWrapper
        //where
        //R: Read + Seek,
    {
        fn current_frame_len(&self) -> Option<usize> {
            Some(self.data.len() - self.current_offset)
        }

        fn channels(&self) -> u16 {
            self.config.channels
        }

        fn sample_rate(&self) -> u32 {
            return self.config.sample_rate
        }

        fn total_duration(&self) -> Option<Duration> {
            None
        }
    }

    #[derive(Copy, Clone)]
    pub struct StreamConfig {
        pub sample_rate: u32,
        pub channels: u16,
    }
}
