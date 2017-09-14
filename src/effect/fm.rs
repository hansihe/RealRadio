use ::liquid;
use super::Effect;

pub struct FmRadioEffect {
    buffer_len: usize,
    buf_cplx: Vec<liquid::liquid_float_complex>,

    freq_mod: liquid::freqmod,
    mixer_tx: liquid::nco_crcf,
    channel: liquid::channel_cccf,
    //mixer_rx: liquid::nco_crcf,
    iirfilt_rx: liquid::iirfilt_crcf,
    freq_dem: liquid::freqdem,
}

impl Effect for FmRadioEffect {

    fn execute(&mut self, data: &mut [f32]) {
        assert!(data.len() <= self.buffer_len);

        unsafe {
            liquid::freqmod_modulate_block(
                self.freq_mod, data.as_mut_ptr(),
                data.len() as u32, (&mut self.buf_cplx).as_mut_ptr());
        }

        for sample in self.buf_cplx.iter_mut() {
            unsafe {
                liquid::nco_crcf_mix_up(self.mixer_tx, *sample, sample);
                let mut v2 = liquid::liquid_float_complex { re: sample.re, im: 0.0 };
                liquid::nco_crcf_step(self.mixer_tx);

                liquid::channel_cccf_execute(self.channel, v2, &mut v2);

                //liquid::nco_crcf_mix_down(self.mixer_rx, v2, sample);
                liquid::iirfilt_crcf_execute(self.iirfilt_rx, *sample, sample);
                //liquid::nco_crcf_step(self.mixer_rx);
            }
        }

        unsafe {
            liquid::freqdem_demodulate_block(
                self.freq_dem, (&mut self.buf_cplx).as_mut_ptr(),
                data.len() as u32, data.as_mut_ptr());
        }
    }

    fn set_tuning(&mut self, tuning: f32) {
        unsafe {
            liquid::nco_crcf_set_frequency(
                self.mixer_tx, tuning * 2.0 * ::std::f32::consts::PI);
        }
    }

}

impl FmRadioEffect {

    pub fn new(buffer_len: usize) -> Self {
        let fc1: f32 = 0.0;
        //let fc2: f32 = 0.0;

        let freq_mod = unsafe {
            liquid::freqmod_create(0.2)
        };
        let freq_dem = unsafe {
            liquid::freqdem_create(0.2)
        };
        let mixer_tx = unsafe {
            let m = liquid::nco_crcf_create(liquid::liquid_ncotype::LIQUID_VCO);
            liquid::nco_crcf_set_frequency(m, fc1 * 2.0 * ::std::f32::consts::PI);
            m
        };
        //let mixer_rx = unsafe {
        //    let m = liquid::nco_crcf_create(liquid::liquid_ncotype::LIQUID_VCO);
        //    liquid::nco_crcf_set_frequency(m, fc2 * 2.0 * ::std::f32::consts::PI);
        //    m
        //};
        let iirfilt_rx = unsafe {
            liquid::iirfilt_crcf_create_lowpass(15, 0.15)
        };
        let channel = unsafe {
            let c = liquid::channel_cccf_create();

            let noise_floor: f32 = -60.0; // dB
            let snr: f32 = 30.0; // dB
            liquid::channel_cccf_add_awgn(c, noise_floor, snr);

            liquid::channel_cccf_add_shadowing(c, 1.0, 0.1);

            c
        };

        let cplx_zero = liquid::liquid_float_complex { re: 0.0, im: 0.0 };
        FmRadioEffect {
            buffer_len: buffer_len,
            buf_cplx: vec![cplx_zero; buffer_len],

            freq_mod: freq_mod,
            freq_dem: freq_dem,
            mixer_tx: mixer_tx,
            //mixer_rx: mixer_rx,
            iirfilt_rx: iirfilt_rx,
            channel: channel,
        }
    }

}
