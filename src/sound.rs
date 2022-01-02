use std::time::Duration;

use rodio::{source::SineWave, OutputStream, Sink, Source};

mod speaker;
fn main() {
    let s = speaker::Speaker::new();
    s.play();
    loop{}
    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // let sink = Sink::try_new(&stream_handle).unwrap();

    // // Add a dummy source of the sake of the example.
    // let source = SineWave::new(440)
    //     // .take_duration(Duration::from_secs_f32(0.25))
    //     .amplify(0.20);
    // sink.append(source);
    // sink.sleep_until_end();
}
