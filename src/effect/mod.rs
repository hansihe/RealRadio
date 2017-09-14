mod am;
pub use self::am::AmRadioEffect;

mod fm;
pub use self::fm::FmRadioEffect;

pub trait Effect {

    fn execute(&mut self, data: &mut [f32]);

    fn set_tuning(&mut self, tuning: f32);

}
