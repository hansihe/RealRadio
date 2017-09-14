//use ::errors::*;
use ::tokio_serial as serial;

use ::std::str;
use ::std::io;
use ::tokio_io::AsyncRead;
use ::tokio_io::codec::{ Decoder, Encoder, Framed };
use ::bytes::BytesMut;
use ::tokio_core::reactor::Handle;
use ::tokio_serial::{ SerialPortSettings, Serial, BaudRate };
use ::futures::Stream;
use ::std::fmt::Write;

#[derive(Debug)]
pub enum AdvButtonState {
}

#[derive(Debug)]
pub enum PanelInputEvent {
    VolumeTurn(i32),
    VolumeButton(AdvButtonState),
    Wiper(i32),
}

#[derive(Debug)]
pub enum Light {
    Left,
    Right,
}

#[derive(Debug)]
pub enum PanelOutputEvent {
    SetLight(Light, u8),
}

pub struct EventsCodec;
impl Decoder for EventsCodec {
    type Item = PanelInputEvent;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');

        if src.len() > 20 && newline.is_none() {
            println!("corrupted serial input: {:?}", src);
            src.clear();
            return Ok(None);
            //panic!("corrupted serial input");
        }

        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(decode_message(s)),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}
impl Encoder for EventsCodec {
    type Item = PanelOutputEvent;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            PanelOutputEvent::SetLight(Light::Left, num) => write!(dst, "b{}\n", num),
            PanelOutputEvent::SetLight(Light::Right, num) => write!(dst, "a{}\n", num),
        }.unwrap();

        Ok(())
    }
}

fn decode_message(msg: &str) -> Option<PanelInputEvent> {
    let tail = msg[1..].trim();
    match msg.as_bytes()[0] {
        b'r' => {
            Some(PanelInputEvent::VolumeTurn(
                str::parse(tail).unwrap()))
        },
        b'w' => {
            Some(PanelInputEvent::Wiper(
                str::parse(tail).unwrap()))
        },
        _ => {
            println!("unimplemented: {:?}", msg);
            None
        },
    }
}


pub fn open(port_path: &str, handle: &Handle) -> Framed<Serial, EventsCodec> {

    let mut settings = SerialPortSettings::default();
    settings.baud_rate = BaudRate::Baud9600;

    let mut port = Serial::from_path(port_path, &settings, &handle).unwrap();

    port.set_exclusive(false)
        .expect("unable to set port exclusive");

    let framed = port.framed(EventsCodec);

    framed
}
