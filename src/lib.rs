//! Multiple precision floating point numbers implemented purely in Rust. 
//! Number has fixed-size mantissa and exponent, but increased precision compared to f32 or f64 values.
//!
//! Number characteristics:
//!
//! | Name                          | Value  |
//! |:------------------------------|-------:|
//! | Decimal positions in mantissa |     40 |
//! | Exponent minimum value        |   -128 |
//! | Exponent maximum value        |    127 |
//! 
//! ## Examples
//! 
//! ```
//! use num_bigfloat::BigFloat;
//! use num_bigfloat::ONE;
//! use num_bigfloat::PI;
//! 
//! // compute pi: pi = 6*arctan(1/sqrt(3))
//! let six: BigFloat = 6.0.into();
//! let three: BigFloat = 3.0.into();
//! let pi = six * (ONE / three.sqrt()).atan();
//! let epsilon = 1.0e-38.into();
//! 
//! assert!((pi - PI).abs() < epsilon);
//! 
//! println!("{}", pi);
//! // output: 3.141592653589793238462643383279502884196e-39
//! ```
//! 
//! ## Performance
//! 
//! The fixed-size mantissa allowed the introduction of precomputed tables to speed up most calculations.
//! With regard to anything else, the implementation is straightforward and does not utilize sophisticated algorithms.
//! 
//! ## no_std
//!
//! Library can be used without the standard Rust library. This can be achieved by turning off `std` feature.

#![deny(clippy::suspicious)]

mod defs;
mod inc;
mod ops;
mod ext;

#[cfg(feature = "std")]
mod parser;

pub use crate::ext::BigFloat;
pub use crate::ext::MAX;
pub use crate::ext::MAX_EXP;
pub use crate::ext::MIN;
pub use crate::ext::MIN_EXP;
pub use crate::ext::MIN_POSITIVE;
pub use crate::ext::RADIX;
pub use crate::ext::NAN;
pub use crate::ext::INF_POS;
pub use crate::ext::INF_NEG;
pub use crate::ext::ZERO;
pub use crate::ext::ONE;
pub use crate::ext::TWO;
pub use crate::ext::E;
pub use crate::ext::PI;
pub use crate::ext::HALF_PI;


#[cfg(test)]
mod tests {

    use rand::random;
    use crate::{
        BigFloat,
        INF_POS, 
        INF_NEG, 
        MIN_POSITIVE,
        ONE,
        TWO,
        MIN,
        MAX,
    };
    use crate::defs::{
        DECIMAL_SIGN_POS, 
        DECIMAL_MIN_EXPONENT, 
        DECIMAL_MAX_EXPONENT, 
        DECIMAL_POSITIONS, DECIMAL_BASE, DECIMAL_PARTS, DECIMAL_SIGN_NEG,
    };


    #[test]
    fn test_bigfloat() {

        let mut d1;
        let mut d2;
        let mut d3;
        let mut ref_num;

        // creation & deconstruction

        // regular buf
        let bytes1: [u8; 20] = [1,2,3,4,5,6,7,8,9,10,11,112,13,14,15,16,17,18,19,20];
        let expected1: [u8; 30] = [1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,0,0,0,0,0,0,0,0,0,0];
        let exp1 = 123;
        let d4 = BigFloat::from_bytes(&bytes1, 1, exp1);

        let mut mantissa_buf1 = [0; 30];
        d4.get_mantissa_bytes(&mut mantissa_buf1);
        assert!(mantissa_buf1 == expected1);
        assert!(d4.get_mantissa_len() == bytes1.len());
        assert!(d4.get_sign() == 1);
        assert!(d4.get_exponent() == exp1);

        // too long buf
        let bytes2: [u8; 45] = [1,2,3,4,5,6,7,8,9,10,11,112,13,14,15,16,17,18,19,20,1,2,3,4,5,6,7,8,9,10,11,112,13,14,15,16,17,18,19,20,21,22,3,4,5];
        let expected2: [u8; 42] = [1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,0,0];
        let exp2 = -128;
        let d4 = BigFloat::from_bytes(&bytes2, -2, exp2);

        let mut mantissa_buf2 = [0; 42];
        d4.get_mantissa_bytes(&mut mantissa_buf2);
        assert!(mantissa_buf2 == expected2);
        assert!(d4.get_mantissa_len() == 40);
        assert!(d4.get_sign() == -1);
        assert!(d4.get_exponent() == exp2);

        // conversions

        // inf
        assert!(BigFloat::from_f64(f64::INFINITY).cmp(&INF_POS) == Some(0));
        assert!(BigFloat::from_f64(f64::NEG_INFINITY).cmp(&INF_NEG) == Some(0));

        // nan
        assert!(BigFloat::from_f64(f64::NAN).is_nan());

        // 0.0
        assert!(BigFloat::from_f64(0.0).to_f64() == 0.0);
        

        // conversions
        for _ in 0..10000 {
            let f: f64 = random_f64_exp(50, 25);
            if f.is_finite() && f != 0.0 {
                d1 = BigFloat::from_f64(f);
                assert!((d1.to_f64() / f - 1.0).abs() < 10.0*f64::EPSILON);
                if (f as f32).is_finite() && (f as f32) != 0.0 {
                    d1 = BigFloat::from_f32(f as f32);
                    assert!((d1.to_f32() / f as f32 - 1.0).abs() < 10.0*f32::EPSILON);
                }
            }
        }

        // 0 * 0
        d1 = BigFloat::new();
        d2 = BigFloat::new();
        ref_num = BigFloat::new();
        d3 = d1.mul(&d2);
        assert!(d3.cmp(&ref_num) == Some(0));

        // 0.99 * 0
        d1 = BigFloat::from_f64(0.99);
        d3 = d1.mul(&d2);
        assert!(d3.cmp(&ref_num) == Some(0));

        // 0 * 12349999
        d1 = BigFloat::new();
        d2 = BigFloat::from_f64(12349999.0);
        d3 = d1.mul(&d2);
        assert!(d3.cmp(&ref_num) == Some(0));

        // 1 * 1
        d1 = BigFloat::from_f64(1.0);
        d2 = BigFloat::from_f64(1.0);
        d3 = d1.mul(&d2);
        assert!(d3.cmp(&d1) == Some(0));

        // 1 * -1
        d1 = BigFloat::from_f64(1.0);
        d2 = BigFloat::from_f64(1.0).inv_sign();
        d3 = d1.mul(&d2);
        assert!(d3.cmp(&d2) == Some(0));

        // -1 * 1
        d3 = d2.mul(&d1);
        assert!(d3.cmp(&d2) == Some(0));

        // -1 * -1
        d1 = d1.inv_sign();
        d3 = d1.mul(&d2);
        ref_num = BigFloat::from_f64(1.0);
        assert!(d3.cmp(&ref_num) == Some(0));

        // 0 / 0 
        d1 = BigFloat::new();
        d2 = BigFloat::new();
        assert!(d1.div(&d2).is_nan());

        // d2 / 0
        d2 = BigFloat::from_f64(123.0);
        assert!(d2.div(&d1).is_inf_pos());

        // 0 / d2
        d3 = d1.div(&d2);
        ref_num = BigFloat::new();
        assert!(d3.cmp(&ref_num) == Some(0));

        // 0 / -d2
        d2 = d2.inv_sign();
        d3 = d1.div(&d2);
        assert!(d3.cmp(&ref_num) == Some(0));


        // add & sub & cmp
        for _ in 0..10000 {
            // avoid subnormal numbers
            let f1 = random_f64_exp(50, 25);
            let f2 = random_f64_exp(50, 25);
            if f1.is_finite() && f2.is_finite() {
                let f3 = f1 + f2;
                let f4 = f1 - f2;
                d1 = BigFloat::from_f64(f1);
                d2 = BigFloat::from_f64(f2);
                if f3 == 0.0 {
                    assert!(d1.add(&d2).to_f64().abs() <= 10000.0*f64::EPSILON);
                } else {
                    assert!((d1.add(&d2).to_f64() / f3 - 1.0).abs() <= 10000.0*f64::EPSILON);
                }
                if f4 == 0.0 {
                    assert!(d1.sub(&d2).to_f64().abs() <= 10000.0*f64::EPSILON);
                } else {
                    assert!((d1.sub(&d2).to_f64() / f4 - 1.0).abs() <= 10000.0*f64::EPSILON);
                }
                if f1 > f2 {
                    assert!(d1.cmp(&d2).unwrap() > 0);
                } else if f1 < f2 {
                    assert!(d1.cmp(&d2).unwrap() < 0);
                } else {
                    assert!(d1.cmp(&d2).unwrap() == 0);
                }
            }
        }

        // mul & div
        for _ in 0..10000 {
            // avoid subnormal numbers
            let f1 = random_f64_exp(50, 25);
            let f2 = random_f64_exp(50, 25);
            if f1.is_finite() && f2.is_finite() && f2 != 0.0 {
                let f3 = f1*f2;
                let f4 = f1/f2;
                d1 = BigFloat::from_f64(f1);
                d2 = BigFloat::from_f64(f2);
                assert!((d1.mul(&d2).to_f64() / f3 - 1.0).abs() <= 10000.0*f64::EPSILON);
                assert!((d1.div(&d2).to_f64() / f4 - 1.0).abs() <= 10000.0*f64::EPSILON);
            }
        }


        // subnormal numbers
        d1 = MIN_POSITIVE;
        d2 = MIN_POSITIVE;
        ref_num = MIN_POSITIVE.mul(&TWO);

        // min_positive + min_positive = 2*min_positive
        assert!(d1.add(&d2).cmp(&ref_num) == Some(0));
        assert!(d1.add(&d2).cmp(&d1).unwrap() > 0);
        assert!(d1.cmp(&d1.add(&d2)).unwrap() < 0);

        // min_positive - min_positive = 0
        ref_num = BigFloat::new();
        assert!(d1.sub(&d2).cmp(&ref_num) == Some(0));

        // 1 * min_positive = min_positive
        assert!(ONE.mul(&d2).cmp(&d2) == Some(0));

        // min_positive / 1 = min_positive
        assert!(d2.div(&ONE).cmp(&d2) == Some(0));

        // min_positive / 1 = min_positive
        assert!(d2.div(&ONE).cmp(&d2) == Some(0));

        // normal -> subnormal -> normal
        d1 = ONE;
        d1.set_exponent(DECIMAL_MIN_EXPONENT);
        d2 = MIN_POSITIVE;
        assert!(!d1.is_subnormal());
        assert!(d1.sub(&d2).cmp(&d1).unwrap() < 0);
        assert!(d1.cmp(&d1.sub(&d2)).unwrap() > 0);
        d1 = d1.sub(&d2);
        assert!(d1.is_subnormal());
        d1 = d1.add(&d2);
        assert!(!d1.is_subnormal());

        // overflow
        d1 = ONE;
        d1.set_exponent(DECIMAL_MAX_EXPONENT - (DECIMAL_POSITIONS as i8 - 1));
        assert!(MAX.add(&d1).is_inf_pos());
        assert!(MIN.sub(&d1).is_inf_neg());
        assert!(MAX.mul(&MAX).is_inf_pos());
        d1 = ONE;
        d1.set_exponent(DECIMAL_MIN_EXPONENT);
        assert!(MAX.div(&d1).is_inf_pos());

        // fract & int
        let f1 = 12345.6789;
        d1 = BigFloat::from_f64(f1);
        assert!((d1.frac().to_f64() - f1.fract()).abs() < 100000.0*f64::EPSILON);
        assert!((d1.int().to_f64() - (f1 as u64) as f64).abs() < 100000.0*f64::EPSILON);

        let f1 = -0.006789;
        d1 = BigFloat::from_f64(f1);
        assert!(d1.frac().cmp(&d1) == Some(0));
        assert!(d1.int().is_zero());

        d1 = BigFloat::from_bytes(&[2,2,2,2,2,0,0,0], DECIMAL_SIGN_POS, -2);
        assert!(d1.frac().is_zero());
        assert!(d1.int().cmp(&d1) == Some(0));

        assert!(MIN_POSITIVE.frac().cmp(&MIN_POSITIVE) == Some(0));
        assert!(MIN_POSITIVE.int().is_zero());

        d1 = BigFloat::new();
        assert!(d1.frac().is_zero());
        assert!(d1.int().is_zero());

        // ceil & floor
        d1 = BigFloat::from_f64(12.3);
        assert!(d1.floor().to_f64() == 12.0);
        assert!(d1.ceil().to_f64() == 13.0);
        d1 = BigFloat::from_f64(12.0);
        assert!(d1.floor().to_f64() == 12.0);
        assert!(d1.ceil().to_f64() == 12.0);

        d1 = BigFloat::from_f64(-12.3);
        assert!(d1.floor().to_f64() == -13.0);
        assert!(d1.ceil().to_f64() == -12.0);
        d1 = BigFloat::from_f64(-12.0);
        assert!(d1.floor().to_f64() == -12.0);
        assert!(d1.ceil().to_f64() == -12.0);

        // abs
        d1 = BigFloat::from_f64(12.3);
        assert!(d1.abs().to_f64() == 12.3);
        d1 = BigFloat::from_f64(-12.3);
        assert!(d1.abs().to_f64() == 12.3);

        // sqrt
        for _ in 0..10000 {
            let num = random_normal_float(256, 128).abs();
            let sq = num.sqrt();
            let ret = sq.mul(&sq);
            assert!(num.sub(&ret).abs().get_mantissa_len() < 2);
        }

        // pow
        for _ in 0..10000 {
            let a = random_normal_float(4, 40).abs();
            let n = random_normal_float(4, 40).abs();
            let inv = ONE.div(&n);
            let p = a.pow(&n);
            if  !p.is_inf() && p.get_mantissa_len() >= DECIMAL_POSITIONS - 1 {
                let ret = p.pow(&inv);
                assert!(a.sub(&ret).abs().get_mantissa_len() <= 2);
            }
        }

    }

    fn random_f64_exp(exp_range: i32, exp_shift: i32) -> f64 {
        let mut f: f64 = random();
        f = f.powi(random::<i32>().abs() % exp_range - exp_shift);
        if random::<i8>() & 1 == 0 {
            f = -f;
        }
        f
    }

    fn random_normal_float(exp_range: i32, exp_shift: i32) -> BigFloat {
        let mut mantissa = [0i16; DECIMAL_PARTS];
        for i in 0..DECIMAL_PARTS {
            mantissa[i] = (random::<u16>() % DECIMAL_BASE as u16) as i16;
        }
        if mantissa[DECIMAL_PARTS-1] == 0 {
            mantissa[DECIMAL_PARTS-1] = (DECIMAL_BASE-1) as i16;
        }
        while mantissa[DECIMAL_PARTS-1] / 1000 == 0 {
            mantissa[DECIMAL_PARTS-1] *= 10;
        }
        let sign = if random::<i8>() & 1 == 0 {DECIMAL_SIGN_POS} else {DECIMAL_SIGN_NEG};
        let exp = random::<i32>().abs() % exp_range - exp_shift;
        BigFloat::from_raw_parts(mantissa, DECIMAL_POSITIONS as i16, sign, exp as i8)
    }

}