//! A basic output stream example, using an Output AudioUnit to generate a sine wave.
//! This one uses interleaved stereo.

extern crate coreaudio;

use coreaudio::audio_unit::{AudioUnit, IOType, SampleFormat, Scope};
use coreaudio::audio_unit::render_callback::{self, data};
use coreaudio::audio_unit::audio_format::LinearPcmFlags;
use std::f64::consts::PI;


// NOTE: temporary replacement for unstable `std::iter::iterate`
struct Iter {
    value: f64,
}
impl Iterator for Iter {
    type Item = f64;
    fn next(&mut self) -> Option<f64> {
        self.value += 440.0 / 44_100.0;
        Some(self.value)
    }
}


fn main() {
    run().unwrap()
}

fn run() -> Result<(), coreaudio::Error> {

    // 440hz sine wave generator.
    let mut samples = Iter { value: 0.0 }
        .map(|phase| (phase * PI * 2.0).sin() as f32 * 0.5);

    // Construct an Output audio unit that delivers audio to the default output device.
    let mut audio_unit = AudioUnit::new(IOType::DefaultOutput)?;

    let mut stream_format = audio_unit.input_stream_format(0)?;
    // Explicitly unset non interleaved flag (it's set by default).
    stream_format.flags &= !LinearPcmFlags::IS_NON_INTERLEAVED;
    audio_unit.set_stream_format(stream_format, Scope::Input, 0)?;
    println!("{:#?}", audio_unit.input_stream_format(0));

    // For this example, our sine wave expects `f32` data.
    assert_eq!(SampleFormat::F32, stream_format.sample_format);

    type Args = render_callback::Args<data::Interleaved<'static, f32>>;
    audio_unit.set_render_callback(move |args| {
        let Args { num_frames, mut data, .. } = args;
        for frm in 0..num_frames {
            let sample = samples.next().unwrap();
            let chan_count = data.channel_count();
            for ch in 0..chan_count {
                data.samples_mut()[chan_count * frm + ch] = sample;
            }
        }
        Ok(())
    }, 0)?;

    audio_unit.init()?;
    audio_unit.start()?;

    std::thread::sleep(std::time::Duration::from_millis(3000));

    Ok(())
}
