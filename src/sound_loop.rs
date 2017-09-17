use ::std::sync::mpsc;

const FRAME_SIZE: usize = 2048;

#[derive(Clone)]
pub struct SoundLoopHandle {
    sender: mpsc::Sender<SoundLoopMessage>,
}
impl SoundLoopHandle {

    pub fn send(&self, message: SoundLoopMessage) {
        self.sender.send(message);
    }

}

pub enum SoundLoopMessage {
    Stop,
    SetTuning(f32),
}

pub fn start() -> SoundLoopHandle {
    let (sender, receiver) = mpsc::channel();

    ::std::thread::spawn(move || start_inner(receiver));

    SoundLoopHandle {
        sender: sender,
    }
}

use ::alsa::{ Direction, ValueOr, Output };
use ::alsa::pcm::{ PCM, HwParams, Format, Access, State };
use ::std::ffi::CString;

struct SoundLoopState {
    pcm_out: PCM,
    pcm_in: PCM,

    buf: [i16; FRAME_SIZE],
    buf_flt: [f32; FRAME_SIZE],

    init: bool,

    effect: Box<::effect::Effect>,
}

impl SoundLoopState {

    fn new() -> SoundLoopState {
        let pcm_out = PCM::open(&*CString::new("default").unwrap(),
                                Direction::Playback, false).unwrap();
        {
            let hwp = HwParams::any(&pcm_out).unwrap();
            hwp.set_channels(1).unwrap();
            hwp.set_rate(44100, ValueOr::Nearest).unwrap();
            hwp.set_format(Format::s16()).unwrap();
            hwp.set_access(Access::RWInterleaved).unwrap();
            pcm_out.hw_params(&hwp).unwrap();
        }

        let pcm_in = PCM::open(&*CString::new("default").unwrap(),
                               Direction::Capture, false).unwrap();
        {
            let hwp = HwParams::any(&pcm_in).unwrap();
            hwp.set_channels(1).unwrap();
            hwp.set_rate(44100, ValueOr::Nearest).unwrap();
            hwp.set_format(Format::s16()).unwrap();
            hwp.set_access(Access::RWInterleaved).unwrap();
            pcm_in.hw_params(&hwp).unwrap();
        }

        SoundLoopState {
            pcm_out: pcm_out,
            pcm_in: pcm_in,

            buf: [0; FRAME_SIZE],
            buf_flt: [0.0; FRAME_SIZE],

            init: true,

            effect: Box::new(::effect::NopEffect::new()),
        }
    }

    fn process_frame(&mut self) {
        let num_read;
        {
            let in_io = self.pcm_in.io_i16().unwrap();
            num_read = in_io.readi(&mut self.buf).unwrap();
            if num_read == 0 {
                println!("wait..");
                self.pcm_in.wait(None).unwrap();
                return;
            }
        }

        for (num, sample) in self.buf.iter().enumerate() {
            self.buf_flt[num] = *sample as f32 / 32768.0;
        }

        self.effect.execute(&mut self.buf_flt);

        for (num, sample) in self.buf_flt.iter().enumerate() {
            self.buf[num] = (*sample * 32768.0) as i16;
        }

        {
            let mut num_written = 0;
            let out_io = self.pcm_out.io_i16().unwrap();
            while num_written < num_read {
                let to_write = &self.buf[num_written..num_read];
                match out_io.writei(to_write) {
                    Ok(num) => num_written += num,
                    Err(ref err) if err.code() == -32 => {
                        println!("underrun, recovering...");
                        self.pcm_out.prepare().unwrap();
                    }
                    err => panic!("{:?}", err),
                }
            }
        }

        if self.init {
            if self.pcm_out.state() != State::Running {
                self.pcm_out.start().unwrap();
            }
            if self.pcm_in.state() != State::Running {
                self.pcm_in.start().unwrap();
            }
            self.init = false;
        }
    }

}

fn start_inner(receiver: mpsc::Receiver<SoundLoopMessage>) {
    let mut state = SoundLoopState::new();

    loop {
        'message_loop: loop {
            match receiver.try_recv() {
                Ok(message) => {
                    match message {
                        SoundLoopMessage::Stop => return,
                        SoundLoopMessage::SetTuning(tuning) => {
                            state.effect.set_tuning(tuning);
                        },
                    }
                },
                Err(mpsc::TryRecvError::Empty) => break 'message_loop,
                Err(mpsc::TryRecvError::Disconnected) => return,
            }
        }

        state.process_frame();
    }

}
