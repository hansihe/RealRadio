use ::liquid;
use liquid::liquid_float_complex as Cplx;

use ::std::ops::{ Mul, Add };

const CPLX_I: liquid::liquid_float_complex = liquid::liquid_float_complex { re: 0.0, im: 1.0 };

impl Mul<f32> for Cplx {
    type Output = Cplx;
    fn mul(self, rhs: f32) -> Cplx {
        Cplx {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}
impl Mul<Cplx> for Cplx {
    type Output = Cplx;
    fn mul(self, rhs: Cplx) -> Cplx {
        Cplx {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl Add<f32> for Cplx {
    type Output = Cplx;
    fn add(self, rhs: f32) -> Cplx {
        Cplx {
            re: self.re + rhs,
            im: self.im,
        }
    }
}
impl Add<Cplx> for Cplx {
    type Output = Cplx;
    fn add(self, rhs: Cplx) -> Cplx {
        Cplx {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}
