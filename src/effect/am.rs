use ::liquid;
use super::Effect;

pub struct AmRadioEffect {
    buffer_len: usize,
    buf_cplx: Vec<liquid::liquid_float_complex>,

    ampmodem_mod: liquid::ampmodem, // AM modulate
    mixer_tx: liquid::nco_crcf, // Upmix baseband
    channel: liquid::channel_cccf, // Channel distortions
    mixer_rx: liquid::nco_crcf, // Downmix baseband
    iirfilt_rx: liquid::iirfilt_crcf, // Lowpass
    agc_rx: liquid::agc_crcf, // Gain control
    ampmodem_dem: liquid::ampmodem, // AM Demodulate
}

impl Effect for AmRadioEffect {

    fn execute(&mut self, data: &mut [f32]) {
        assert!(data.len() <= self.buffer_len);

        //for sample in self.buf_cplx.iter_mut() {
        //    *sample = *sample * 32000.0;
        //}

        unsafe {
            liquid::ampmodem_modulate_block(
                self.ampmodem_mod, data.as_mut_ptr(),
                data.len() as u32, (&mut self.buf_cplx).as_mut_ptr());
        }

        for sample in self.buf_cplx.iter_mut() {
            unsafe {
                liquid::nco_crcf_mix_up(self.mixer_tx, *sample, sample);
                liquid::nco_crcf_step(self.mixer_tx);
                //*sample = liquid::liquid_float_complex { re: sample.re, im: 0.0 };

                liquid::channel_cccf_execute(self.channel, *sample, sample);

                //liquid::nco_crcf_mix_down(self.mixer_rx, *sample, sample);
                //liquid::nco_crcf_step(self.mixer_rx);
                liquid::iirfilt_crcf_execute(self.iirfilt_rx, *sample, sample);

                //liquid::agc_crcf_execute(self.agc_rx, *sample, sample);
                //sample.re *= 500.0;
                //sample.im *= 500.0;
            }
        }

        unsafe {
            liquid::ampmodem_demodulate_block(
                self.ampmodem_dem, (&mut self.buf_cplx).as_mut_ptr(),
                data.len() as u32, data.as_mut_ptr());
        }

        for sample in self.buf_cplx.iter_mut() {
            *sample = *sample * 10.0;
        //    *sample = *sample * (1.0/32000.0);
        }

    }

    fn set_tuning(&mut self, tuning: f32) {
    }

}

impl AmRadioEffect {

    pub fn new(buffer_len: usize) -> Self {
        let fc1: f32 = 4.0;
        let fc2: f32 = 20.0;

        let ampmodem_mod = unsafe {
            liquid::ampmodem_create(
                0.1, 0.0, liquid::liquid_ampmodem_type::LIQUID_AMPMODEM_DSB, 1)
        };
        let ampmodem_dem = unsafe {
            liquid::ampmodem_create(
                0.1, 0.0, liquid::liquid_ampmodem_type::LIQUID_AMPMODEM_DSB, 1)
        };
        let mixer_tx = unsafe {
            let m = liquid::nco_crcf_create(liquid::liquid_ncotype::LIQUID_VCO);
            liquid::nco_crcf_set_frequency(m, fc1 * 2.0 * ::std::f32::consts::PI);
            m
        };
        let mixer_rx = unsafe {
            let m = liquid::nco_crcf_create(liquid::liquid_ncotype::LIQUID_VCO);
            liquid::nco_crcf_set_frequency(m, fc2 * 2.0 * ::std::f32::consts::PI);
            m
        };
        let iirfilt_rx = unsafe {
            liquid::iirfilt_crcf_create_lowpass(15, 0.11)
        };
        let agc_rx = unsafe {
            let a = liquid::agc_crcf_create();
            liquid::agc_crcf_set_bandwidth(a, 0.001);
            a
        };
        let channel = unsafe {
            let c = liquid::channel_cccf_create();

            let noise_floor: f32 = -10.0; // dB
            let snr: f32 = 30.0; // dB
            liquid::channel_cccf_add_awgn(c, noise_floor, snr);

            liquid::channel_cccf_add_shadowing(c, 1.0, 0.1);

            c
        };

        let cplx_zero = liquid::liquid_float_complex { re: 0.0, im: 0.0 };
        AmRadioEffect {
            buffer_len: buffer_len,
            buf_cplx: vec![cplx_zero; buffer_len],

            ampmodem_mod: ampmodem_mod,
            ampmodem_dem: ampmodem_dem,
            mixer_tx: mixer_tx,
            mixer_rx: mixer_rx,
            iirfilt_rx: iirfilt_rx,
            agc_rx: agc_rx,
            channel: channel,
        }
    }

}
