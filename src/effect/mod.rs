mod am;
pub use self::am::AmRadioEffect;

mod fm;
pub use self::fm::FmRadioEffect;

mod nop;
pub use self::nop::NopEffect;

pub trait Effect {

    fn execute(&mut self, data: &mut [f32]);

    fn set_tuning(&mut self, tuning: f32);

}
