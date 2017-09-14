#[recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

extern crate alsa;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate tokio_serial;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_core;
extern crate futures;
extern crate bytes;

use ::std::ffi::CString;
use futures::Stream;

use tokio_core::reactor::Core;

mod liquid;
mod effect;
mod complex;
mod sound_loop;
mod config;
mod front_panel_input;

pub mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            TomlDe(::toml::de::Error);
        }

        links {
        }
    }
}

fn main() {

    let sound_loop_handle = sound_loop::start();

    let config = config::read_config().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let front_panel = front_panel_input::open(&config.front_panel_serial, &handle);
    let (send, receive) = front_panel.split();

    let t = receive.for_each(|event| {
        println!("{:?}", event);
        match event {
            ::front_panel_input::PanelInputEvent::Wiper(val) => {
                let tuning = (200.0 - (val as f32)) / 100.0;
                sound_loop_handle.send(
                    ::sound_loop::SoundLoopMessage::SetTuning(tuning));
            },
            _ => (),
        }
        Ok(())
    });

    core.run(t).unwrap();

    //use std::ffi::CString;
    //use alsa::{Direction, ValueOr, Output};
    //use alsa::pcm::{PCM, HwParams, Format, Access, State};

    //let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();
    //let hwp = HwParams::any(&pcm).unwrap();
    //hwp.set_channels(1).unwrap();
    //hwp.set_rate(44100, ValueOr::Nearest).unwrap();
    //hwp.set_format(Format::s16()).unwrap();
    //hwp.set_access(Access::RWInterleaved).unwrap();
    //pcm.hw_params(&hwp).unwrap();

    //let pcm_i = PCM::open(&*CString::new("default").unwrap(), Direction::Capture, false).unwrap();
    //let hwp = HwParams::any(&pcm_i).unwrap();
    //hwp.set_channels(1).unwrap();
    //hwp.set_rate(44100, ValueOr::Nearest).unwrap();
    //hwp.set_format(Format::s16()).unwrap();
    //hwp.set_access(Access::RWInterleaved).unwrap();
    //pcm_i.hw_params(&hwp).unwrap();

    //println!("PCM status: {:?}, {:?}", pcm_i.state(), pcm_i.hw_params_current().unwrap());
    //let mut outp = Output::buffer_open().unwrap();
    //pcm.dump(&mut outp).unwrap();
    //println!("== PCM dump ==\n{}", outp);

    //let mut init: bool = true;

    //let mut buf: [i16; 2048] = [0; 2048]; //[i16; 1024] = [0; 1024];
    //let mut buf_flt: [f32; 2048] = [0.0; 2048];

    //use effect::{ Effect, AmRadioEffect, FmRadioEffect };
    //let mut am_radio = AmRadioEffect::new(2048);
    //let mut fm_radio = FmRadioEffect::new(2048);

    //loop {
    //    let num_read;
    //    {
    //        let in_io = pcm_i.io_i16().unwrap();
    //        num_read = in_io.readi(&mut buf).unwrap();
    //        if num_read == 0 {
    //            println!("wait..");
    //            pcm_i.wait(None).unwrap();
    //            continue;
    //        }
    //    }

    //    for (num, sample) in buf.iter().enumerate() {
    //        buf_flt[num] = *sample as f32 / 32768.0;
    //    }

    //    fm_radio.execute(&mut buf_flt[0..num_read]);

    //    for (num, sample) in buf_flt.iter().enumerate() {
    //        buf[num] = (*sample * 32768.0) as i16;
    //    }

    //    {
    //        let mut num_written = 0;
    //        let out_io = pcm.io_i16().unwrap();
    //        while num_written < num_read {
    //            let to_write = &buf[num_written..num_read];
    //            match out_io.writei(to_write) {
    //                Ok(num) => num_written += num,
    //                Err(ref err) if err.code() == -32 => {
    //                    println!("underrun, recovering...");
    //                    pcm.prepare().unwrap();
    //                }
    //                err => panic!("{:?}", err),
    //            }
    //        }
    //    }

    //    if init {
    //        if pcm.state() != State::Running {
    //            pcm.start().unwrap();
    //        }
    //        if pcm_i.state() != State::Running {
    //            pcm_i.start().unwrap();
    //        }
    //        init = false;
    //    }
    //}

}
