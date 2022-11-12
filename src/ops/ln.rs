//! Natural logarithm.

use crate::Exponent;
use crate::common::consts::ONE;
use crate::common::consts::TWO;
use crate::common::util::count_leading_ones;
use crate::common::util::get_add_cost;
use crate::common::util::get_mul_cost;
use crate::common::util::get_sqrt_cost;
use crate::num::BigFloatNumber;
use crate::defs::RoundingMode;
use crate::defs::Error;
use crate::defs::Sign;
use crate::ops::series::PolycoeffGen;
use crate::ops::series::ArgReductionEstimator;
use crate::ops::series::series_run;
use crate::ops::series::series_cost_optimize;
use crate::ops::consts::Consts;


// Polynomial coefficient generator.
struct AtanhPolycoeffGen {
    acc: BigFloatNumber,
    one_full_p: BigFloatNumber,
    val: BigFloatNumber,
    iter_cost: usize,
}

impl AtanhPolycoeffGen {

    fn new(p: usize) -> Result<Self, Error> {

        let acc = BigFloatNumber::from_word(1, 1)?;
        let one_full_p = BigFloatNumber::from_word(1, p)?;
        let val = BigFloatNumber::from_word(1, p)?;

        let iter_cost = get_add_cost(p) + get_add_cost(1); // div is linear, since add is O(1)

        Ok(AtanhPolycoeffGen {
            acc,
            one_full_p,
            val,
            iter_cost,
        })
    }
}

impl PolycoeffGen for AtanhPolycoeffGen {
    fn next(&mut self, rm: RoundingMode) -> Result<&BigFloatNumber, Error> {

        self.acc = self.acc.add(&TWO, rm)?;
        self.val = self.one_full_p.div(&self.acc, rm)?;

        Ok(&self.val)
    }

    #[inline]
    fn get_iter_cost(&self) -> usize {
        self.iter_cost
    }
}

struct LnArgReductionEstimator {}

impl ArgReductionEstimator for LnArgReductionEstimator {

    /// Estimates cost of reduction n times for number with precision p.
    fn get_reduction_cost(n: usize, p: usize) -> usize {

        // cost(shift) + n*cost(sqrt)
        let cost_mul = get_mul_cost(p);
        let cost_add = get_add_cost(p);
        let sqrt_cost = get_sqrt_cost(p, cost_mul, cost_add);

        n * sqrt_cost
    }

    /// Given m, the negative power of 2 of a number, returns the negative power of 2 if reduction is applied n times.
    #[inline]
    fn reduction_effect(n: usize, m: isize) -> usize {
        (m + n as isize) as usize
    }
}

impl BigFloatNumber {

    /// Computes the natural logarithm of a number. The result is rounded using the rounding mode `rm`.
    /// This function requires constants cache `cc` for computing the result.
    /// 
    /// ## Errors
    /// 
    ///  - InvalidArgument: the argument is zero or negative.
    ///  - MemoryAllocation: failed to allocate memory.
    pub fn ln(&self, rm: RoundingMode, cc: &mut Consts) -> Result<Self, Error> {

        // factoring: ln(self) = ln(x * 2^n) = ln(x) + n*ln(2), 0.5 <= x < 1
        // reduction: ln(x) = 2*ln(sqrt(x))
        // replacement: ln(x) = 2*atanh((x-1)/(x+1))
        // atanh(x) = x + x^3/3 + x^5/5 + ...

        if self.is_zero() || self.is_negative() {

            return Err(Error::InvalidArgument);
        }

        let mut x = self.clone()?;
        let e = x.normalize2() as isize;
        let e = self.get_exponent() as isize - e;

        x.set_exponent(0);

        let additional_prec = count_leading_ones(x.get_mantissa_digits()) + 2;
        x.set_precision(x.get_mantissa_max_bit_len() + additional_prec, RoundingMode::None)?;

        let p1 = Self::ln_series(x, RoundingMode::None)?;

        let mut ret = if e == 0 {

            p1

        } else {

            let p2 = cc.ln_2(self.get_mantissa_max_bit_len() + 2, RoundingMode::None)?;

            let mut n = Self::from_usize(e.unsigned_abs())?;
            if e < 0 {
                n.set_sign(Sign::Neg);
            }

            let p2n = p2.mul(&n, RoundingMode::None)?;
            p1.add(&p2n, RoundingMode::None)?
        };

        ret.set_precision(self.get_mantissa_max_bit_len(), rm)?;

        Ok(ret)
    }

    fn ln_series(mut x: Self, rm: RoundingMode) -> Result<Self, Error> {

        let p = x.get_mantissa_max_bit_len();
        let mut polycoeff_gen = AtanhPolycoeffGen::new(p)?;
        let (reduction_times, niter) = series_cost_optimize::<AtanhPolycoeffGen, LnArgReductionEstimator>(
            p, &polycoeff_gen, 0, 2, false);

        let err = niter * 3 + reduction_times + 4;
        x.set_precision(x.get_mantissa_max_bit_len() + err, rm)?;

        let arg = if reduction_times > 0 {
            Self::ln_arg_reduce(x, reduction_times, rm)?
        } else {
            x
        };

        // x-1 / x+1
        let x1 = arg.sub(&ONE, rm)?;
        let x2 = arg.add(&ONE, rm)?;
        let z = x1.div(&x2, rm)?;

        let x_step = z.mul(&z, rm)?;   // x^2
        let x_first = z.mul(&x_step, rm)?;   // x^3

        let ret = series_run(z, x_first, x_step, niter, &mut polycoeff_gen, rm)?;

        Self::ln_arg_restore(ret, reduction_times + 1)
    }

    // reduce argument n times.
    fn ln_arg_reduce(mut x: Self, n: usize, rm: RoundingMode) -> Result<Self, Error> {

        for _ in 0..n {
            x = x.sqrt(rm)?;
        }

        Ok(x)
    }

    // restore value for the argument reduced n times.
    fn ln_arg_restore(mut x: Self, n: usize) -> Result<Self, Error> {

        x.set_exponent(x.get_exponent() + n as Exponent);

        Ok(x)
    }
}


#[cfg(test)]
mod tests {

    use crate::common::util::log2_ceil;

    use super::*;

    #[test]
    fn test_ln() {
        let mut cc = Consts::new().unwrap();

        let rm = RoundingMode::ToEven;
        let n1 = BigFloatNumber::from_word(123,3200).unwrap();

        let mut n2 = n1.ln(rm, &mut cc).unwrap();
        n2.set_sign(Sign::Pos);

        //println!("{:?}", n2.fp3(crate::Radix::Dec, rm).unwrap());

        // near 1
        let d1 = BigFloatNumber::parse("F.FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF2DC85F7E77EC4872DC85F7E77EC487_e-1", crate::Radix::Hex, 320, RoundingMode::None).unwrap();
        let d2 = d1.ln(RoundingMode::ToEven, &mut cc).unwrap();
        let d3 = BigFloatNumber::parse("-D.237A0818813B78D237A0818813B7900000000000000000000564FA7B56FC57E9FBF3EE86C58F3F4_e-33", crate::Radix::Hex, 320, RoundingMode::None).unwrap();

        // println!("{:?}", d2.format(crate::Radix::Hex, RoundingMode::None).unwrap());

        assert!(d2.cmp(&d3) == 0);

        let d1 = BigFloatNumber::parse("1.FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF2DC85F7E77EC4872DC85F7E77EC487", crate::Radix::Hex, 320, RoundingMode::None).unwrap();
        let d2 = d1.ln(RoundingMode::ToEven, &mut cc).unwrap();
        let d3 = BigFloatNumber::parse("B.17217F7D1CF79ABC9E3B39803F2F6AF40F343267298B62D837B5A577F6A5C6F7E9CA5DFA9E1D0D0_e-1", crate::Radix::Hex, 320, RoundingMode::None).unwrap();

        // println!("{:?}", d2.format(crate::Radix::Hex, RoundingMode::None).unwrap());

        assert!(d2.cmp(&d3) == 0);

        // MAX
        let prec = 3200;
        let mut eps = ONE.clone().unwrap();

        let d1 = BigFloatNumber::max_value(prec).unwrap();
        let d2 = d1.ln(RoundingMode::ToEven, &mut cc).unwrap();
        let d3 = d2.exp(RoundingMode::ToEven, &mut cc).unwrap();
        eps.set_exponent(d1.get_exponent() - prec as Exponent + 
                        log2_ceil(d1.get_exponent().unsigned_abs() as usize) as Exponent);

        assert!(d1.sub(&d3, RoundingMode::ToEven).unwrap().abs().unwrap().cmp(&eps) < 0);

        // MIN
        let mut d1 = BigFloatNumber::min_positive(prec).unwrap();
        d1.set_exponent(d1.get_exponent() + d1.get_mantissa_max_bit_len() as Exponent + 2); // avoid exp() overflow
        let d2 = d1.ln(RoundingMode::ToEven, &mut cc).unwrap();
        let d3 = d2.exp(RoundingMode::ToEven, &mut cc).unwrap();
        let eps = BigFloatNumber::min_positive_normal(prec).unwrap();

        assert!(d1.sub(&d3, RoundingMode::ToEven).unwrap().abs().unwrap().cmp(&eps) <= 0);
    }

    #[ignore]
    #[test]
    #[cfg(feature="std")]
    fn ln_perf() {
        let mut cc = Consts::new().unwrap();
        let mut n = vec![];
        for _ in 0..10000 {
            let mut nn = BigFloatNumber::random_normal(133, -100, 100).unwrap();
            nn.set_sign(Sign::Pos);
            n.push(nn);
        }

        for _ in 0..5 {
            let start_time = std::time::Instant::now();
            for ni in n.iter() {
                let _f = ni.ln(RoundingMode::ToEven, &mut cc).unwrap();
            }
            let time = start_time.elapsed();
            println!("{}", time.as_millis());
        }
    }

}