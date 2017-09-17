use super::Effect;

pub struct NopEffect {
}

impl Effect for NopEffect {

    fn execute(&mut self, data: &mut [f32]) {
    }

    fn set_tuning(&mut self, tuning: f32) {
    }

}

impl NopEffect {

    pub fn new() -> Self {

        NopEffect {}

    }

}
