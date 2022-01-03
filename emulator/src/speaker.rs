use std::time::Duration;

use rodio::{OutputStream, Sink, Source};

use crate::TARGET_INTERVAL;

pub struct Speaker {
    sink: Sink,
    _stream: OutputStream,
}

impl Speaker {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        // sink.append(SineWave::new(440).amplify(0.20));
        Self {
            sink,
            _stream: stream,
        }
    }

    pub fn play_freq(&self, freq: u32) {
        // log::info!("Play {}", freq);

        // self.sink.stop();
        self.sink.append(
            SquareWave::new(freq)
                .take_duration(Duration::from_secs_f64(TARGET_INTERVAL * 1.01))
                .amplify(0.10),
        );
    }

    pub fn play(&self) {
        self.play_freq(440)
    }

    pub fn stop(&self) {
        // log::info!("Stop");
        // Stub
    }
}

struct SquareWave {
    samples_per_cycle: u32,
    pos: u32,
}

impl SquareWave {
    const SAMPLE_RATE: u32 = 48000;

    pub fn new(freq: u32) -> Self {
        Self {
            pos: 0,
            samples_per_cycle: Self::SAMPLE_RATE / freq,
        }
    }
}

impl Iterator for SquareWave {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.pos = (self.pos + 1) % self.samples_per_cycle;

        let value = if self.pos < self.samples_per_cycle / 2 {
            -1.
        } else {
            1.
        };
        Some(value)
    }
}

impl Source for SquareWave {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        Self::SAMPLE_RATE
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
