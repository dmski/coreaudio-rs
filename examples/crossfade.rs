//! An example that crossfades between two sine waves using StereoMixer audio unit.

extern crate coreaudio;
extern crate coreaudio_sys;

use coreaudio::audio_unit::{Type, AudioUnit, IOType, MixerType, Scope};
use coreaudio::audio_unit::render_callback::{self, data};
use std::f64::consts::PI;
use coreaudio_sys::{kAudioUnitProperty_ElementCount, kStereoMixerParam_Volume};
use std::time::Duration;

struct Iter {
    value: f64,
    freq: f64
}
impl Iterator for Iter {
    type Item = f64;
    fn next(&mut self) -> Option<f64> {
        self.value += self.freq / 44_100.0;
        Some(self.value)
    }
}

type Args = render_callback::Args<data::NonInterleaved<f32>>;

fn render_callback<T: Iterator<Item=f32>>(args: Args, samples: &mut T) -> Result<(), ()> {
    let Args { num_frames, mut data, .. } = args;
    for i in 0..num_frames {
        let sample = samples.next().unwrap();
        for mut channel in data.channels_mut() {
            channel[i] = sample;
        }
    };

    Ok(())
}

fn main() {
    run().unwrap()
}

fn run() -> Result<(), coreaudio::Error> {

    let mut samples1 = Iter { freq: 440.0, value: 0.0 }
        .map(|phase| (phase * PI * 2.0).sin() as f32 * 0.15);
    let mut samples2 = Iter { freq: 880.0, value: 0.0 }
        .map(|phase| (phase * PI * 2.0).sin() as f32 * 0.15);

    let mut output = AudioUnit::new(IOType::DefaultOutput)?;
    let mut mixer = AudioUnit::new(Type::Mixer(MixerType::StereoMixer))?;

    mixer.set_property(kAudioUnitProperty_ElementCount, Scope::Input, 0, Some(&2))?;

    mixer.set_render_callback(move |args| {
        render_callback(args, &mut samples1)
    }, 0)?;
    mixer.set_render_callback(move |args| {
        render_callback(args, &mut samples2)
    }, 1)?;

    output.set_data_source(0, &mixer, 0)?;

    mixer.set_parameter(kStereoMixerParam_Volume, Scope::Input, 0, 0.0)?;
    mixer.set_parameter(kStereoMixerParam_Volume, Scope::Input, 1, 1.0)?;

    mixer.init()?;
    output.init()?;

    output.start()?;

    let duration = 10000;
    let xfade_steps = 1000;
    let mut xfade_alpha = -1f32;
    let alpha_inc = 2.0 / xfade_steps as f32;

    for _ in 0..xfade_steps {
        std::thread::sleep(Duration::from_millis(duration / xfade_steps));
        xfade_alpha += alpha_inc;
        // something resembling equal-power crossfade, maybe
        let param0 = (0.5 * (1.0 - xfade_alpha)).sqrt();
        let param1 = (0.5 * (1.0 + xfade_alpha)).sqrt();
        mixer.set_parameter(kStereoMixerParam_Volume, Scope::Input, 0, param0)?;
        mixer.set_parameter(kStereoMixerParam_Volume, Scope::Input, 1, param1)?;
    }

    Ok(())
}
