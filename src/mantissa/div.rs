//! Recursive division.

use crate::defs::DIGIT_BIT_SIZE;
use crate::defs::DIGIT_BASE;
use crate::defs::Error;
use crate::defs::Digit;
use crate::defs::DoubleDigit;
use crate::mantissa::Mantissa;
use crate::mantissa::buf::DigitBuf;
use crate::mantissa::util::SliceWithSign;


impl Mantissa {

    // Basic integer division.
    fn div_basic(m1: &[Digit], m2: &[Digit]) -> Result<(DigitBuf, DigitBuf), Error> {
        let l1 = m1.len();
        let l2 = m2.len();
        let mut c: DoubleDigit;
        let mut j: usize;
        let mut qh: DoubleDigit;
        let mut k: DoubleDigit;
        let mut rh: DoubleDigit;
        let mut buf = DigitBuf::new(l1 + l2 + 2)?;
        let (buf1, buf2) = (&mut buf).split_at_mut(l1 + 1);
        let n = l2 - 1;
        let m = l1 - 1;
        let mut m3 = DigitBuf::new(m - n + 1)?;
        let mut rem = DigitBuf::new(l2)?;

        if n == 0 {
            // division by single digit
            let d = m2[0] as DoubleDigit;
            rh = 0;
            let mut j = l1 - l2 + 1;
            let mut iter = m1.iter().rev();
            let mut val = *iter.next().unwrap_or(&0) as DoubleDigit;
            let mut m3iter = m3.iter_mut().rev();
            if val < d {
                rh = val;
                val = *iter.next().unwrap_or(&0) as DoubleDigit;
                *m3iter.next().unwrap() = 0;
                rem[0] = rh as Digit;
                j -= 1;
            }
        
            if j > 0 {
                loop {
                    qh = rh * DIGIT_BASE as DoubleDigit + val;
                    rh = qh % d;
                
                    if let Some(v) = m3iter.next() {
                        *v = (qh / d) as Digit;
                        rem[0] = rh as Digit;
                    } else {
                        break;
                    }
                    val = *iter.next().unwrap_or(&0) as DoubleDigit;
                }
            } else {
                for v in m3iter {
                    *v = 0;
                }
            }
        } else {
            // normalize: buf1 = d1 * d, buf2 = d2 * d
            let d = DIGIT_BASE / (m2[n] as DoubleDigit + 1); // factor d: d * m2[most significant] is close to DIGIT_MAX

            if d == 1 {
                buf1[..l1].clone_from_slice(m1);
                buf2[..l2].clone_from_slice(m2);
                buf1[l1] = 0;
                buf2[l2] = 0;
            } else {
                Self::mul_by_digit(m1, d, buf1);
                Self::mul_by_digit(m2, d, buf2);
            }

            let v1 = buf2[n] as DoubleDigit;
            let v2 = buf2[n - 1] as DoubleDigit;

            j = m - n;
            let mut m3iter = m3.iter_mut().rev();
            let mut in_loop = false;
            let mut buf12;
            let mut buf11;
            let mut buf10;
            loop {
                buf12 = buf1[j + n + 1] as DoubleDigit;
                buf11 = buf1[j + n] as DoubleDigit;
                buf10 = buf1[j + n - 1] as DoubleDigit;

                qh = buf12 * DIGIT_BASE + buf11;
                rh = qh % v1;
                qh /= v1;

                if qh >= DIGIT_BASE || (qh * v2 > DIGIT_BASE * rh + buf10) {
                    qh -= 1;
                    rh += v1;
                    if rh < DIGIT_BASE && 
                        (qh >= DIGIT_BASE || (qh * v2 > DIGIT_BASE * rh + buf10)) {
                            qh -= 1;
                    }
                }

                // n1_j = n1_j - n2 * qh
                c = 0;
                k = 0;
                for (a, b) in buf2[..n+2].iter().zip(buf1[j..j+n+2].iter_mut()) {
                    k = *a as DoubleDigit * qh + k / DIGIT_BASE;
                    let val = k % DIGIT_BASE + c;
                    if (*b as DoubleDigit) < val {
                        *b += (DIGIT_BASE - val) as Digit;
                        c = 1;
                    } else {
                        *b -= val as Digit;
                        c = 0;
                    }
                }

                if c > 0 {
                    // compensate
                    qh -= 1;
                    c = 0;
                    for (a, b) in buf2[..n+2].iter().zip(buf1[j..j+n+2].iter_mut()) {
                        let mut val = *b as DoubleDigit;
                        val += *a as DoubleDigit + c;
                        if val >= DIGIT_BASE {
                            val -= DIGIT_BASE;
                            c = 1;
                        } else {
                            c = 0;
                        }
                        *b = val as Digit;
                    }
                    debug_assert!(c > 0);
                }

                if let Some(v) = m3iter.next() {
                    if in_loop || qh > 0 {
                        *v = qh as Digit;
                    } else {
                        *v = 0;
                    }
                } else {
                    break;
                }
        
                if j == 0 {
                    break;
                }
                j -= 1;
                in_loop = true;
            }

            for v in m3iter {
                *v = 0;
            }

            if d > 1 {
                // restore remainder
                rh = 0;
                let mut j = l1 + 1;
                let mut iter = buf1[..l2].iter().rev();
                let mut val = *iter.next().unwrap_or(&0) as DoubleDigit;
                let mut remiter = rem.iter_mut().rev();
                if val < d {
                    rh = val;
                    val = *iter.next().unwrap_or(&0) as DoubleDigit;
                    *remiter.next().unwrap() = 0;
                    j -= 1;
                }
            
                if j > 0 {
                    loop {
                        qh = rh * DIGIT_BASE as DoubleDigit + val;
                        rh = qh % d;
    
                        if let Some(v) = remiter.next() {
                            *v = (qh / d) as Digit;
                        } else {
                            break;
                        }
                        val = *iter.next().unwrap_or(&0) as DoubleDigit;
                    }
                } else {
                    for v in remiter {
                        *v = 0;
                    }
                }
            } else {
                rem.copy_from_slice(&buf1[..l2]);
            }
        }

        Ok((m3, rem))
    }

    // recursive division correction
    fn div_correction(a: &mut SliceWithSign, q: &mut SliceWithSign, step: SliceWithSign, work_buf: &mut [Digit]) {
        while a.sign() < 0 {
            q.decrement_abs();
            a.add_assign(&step, work_buf);
        }
    }

    // Recursive integer division from the book of Richard P. Brent and Paul Zimmermann.
    // Divides m1 by m2, returns quotinent and remainder.
    // prereq: m <= n, m2 is balanced
    fn div_recursive(m1: &[Digit], m2: &[Digit]) -> Result<(DigitBuf, DigitBuf), Error> {
        let m = m1.len() - m2.len();
        if m < 2 {
            // basic div
            Self::div_basic(m1, m2)
        } else {
            let k = m / 2;
            let k2 = k << 1;

            let mut rembuf = DigitBuf::new(m1.len())?;
            let mut buf = DigitBuf::new(2*(m1.len()+1))?;
            buf.fill(0);
            let (buf2, rest) = buf.split_at_mut(m1.len()+1);
            let buf3 = rest;

            let a = SliceWithSign::new(m1, 1);
            let a1 = SliceWithSign::new(&m1[k2..], 1);  // m1 div 2^(2*k)

            let b = SliceWithSign::new(m2, 1);
            let b1 = SliceWithSign::new(&m2[k..], 1);   // m2 div 2^k

            let (mut q1buf, _r1) = Self::div_recursive(&a1, &b1)?;
            let mut q1 = SliceWithSign::new_mut(&mut q1buf, 1);

            // a3 = a - b*q1*2^k
            let mut a3 = SliceWithSign::new_mut(&mut rembuf, 1);
            a3.copy_from(&a);
            
            Self::mul_slices(&q1, &b, &mut buf2[k..])?;
            let bqk = SliceWithSign::new(buf2, 1);

            a3.sub_assign(&bqk, buf3);

            if a3.sign() < 0 {
                // correction
                buf2.fill(0);
                let mut bk = SliceWithSign::new_mut(&mut buf2[k..], 1);
                bk.copy_from(&b);
                let bk = SliceWithSign::new(buf2, 1);
                Self::div_correction(&mut a3, &mut q1, bk, buf3);
            }

            let mut ub = a3.len();
            for v in (&a3).iter().rev() {
                if *v == 0 {
                    ub -= 1;
                }
            }

            q1buf.try_extend((m + 1)*DIGIT_BIT_SIZE)?;
            let mut q1 = SliceWithSign::new_mut(&mut q1buf, 1);

            if ub > k {
                let a31 = SliceWithSign::new(&rembuf[k..ub], 1);  // a3 div 2^(k)
                let (mut q0, _r0) = Self::div_recursive(&a31, &b1)?;
                let mut q0 = SliceWithSign::new_mut(&mut q0, 1);
                let mut a3 = SliceWithSign::new_mut(&mut rembuf, 1);

                // a3 = a3 - q0*b
                Self::mul_slices(&q0, &b, buf2)?;
                let qb = SliceWithSign::new(buf2, 1);
                a3.sub_assign(&qb, buf3);

                if a3.sign() < 0 {
                    // correction
                    Self::div_correction(&mut a3, &mut q0, b, buf3);
                }

                // quot = q1 * 2^k + q0;
                q1.add_assign(&q0, buf3);
            }

            Ok((q1buf, rembuf))
        }
    }

    pub(super) fn div_unbalanced(m1: &[Digit], m2: &[Digit]) -> Result<(DigitBuf, DigitBuf), Error> {
        let mut m = m1.len() - m2.len();
        let n = m2.len();
        if m <= n {
            Self::div_recursive(m1, m2)
        } else if n < 2 {
            Self::div_basic(m1, m2)
        } else {
            let mut buf1 = DigitBuf::new(m + 1)?;
            buf1.fill(0);
            let mut tmp_buf = DigitBuf::new(m1.len()*3+1)?;
            let (buf2, rest) = tmp_buf.split_at_mut(m1.len());
            let (buf3, rest) = rest.split_at_mut(m1.len());
            let buf4 = rest;
            buf3.copy_from_slice(m1);
            let mut a = SliceWithSign::new_mut(buf3, 1);
            let mut nn = 0;
    
            while m > n {
                let mn = m - n;
                let a1 = SliceWithSign::new(&a[mn..m1.len()-nn], 1);  // m1 div 2^(m-n)
                nn += n;
    
                let (q, _r) = Self::div_recursive(&a1, m2)?;
                let q = SliceWithSign::new(&q, 1);
    
                let mut full_q = SliceWithSign::new_mut(&mut buf1[mn..], 1);
                full_q.add_assign(&q, buf2);
    
                buf4.fill(0);
                Self::mul_slices(m2, &q, &mut buf4[mn..])?;
    
                let qbmn = SliceWithSign::new(buf4, 1);
                a.sub_assign(&qbmn, buf2);
                m -= n;
            }
    
            let (q, r) = Self::div_recursive(&a[..m1.len()-nn], m2)?;
            let q = SliceWithSign::new(&q, 1);
            let mut full_q = SliceWithSign::new_mut(&mut buf1, 1);
            full_q.add_assign(&q, buf2);
    
            Ok((buf1, r))
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use crate::defs::DIGIT_SIGNIFICANT_BIT;
    use rand::random;

    #[ignore]
    #[test]
    fn test_div_perf() {

        for _ in 0..5 {
            let sz1 = 10000;
            let sz2 = 5000;
            let f = random_normalized_slice(sz1, sz1);
            let mut n = vec![];
            let l = 1;
            for _ in 0..l {
                let v = random_normalized_slice(sz2, sz2);
                n.push(v);
            }
            
            // basic
            let start_time = std::time::Instant::now();
            for ni in &n {
                let _ = Mantissa::div_basic(&f, ni).unwrap();
            }
            let time = start_time.elapsed();
            println!("div_basic {}", time.as_millis());
            
            // unbalanced
            let start_time = std::time::Instant::now();
            for ni in &n {
                let _ = Mantissa::div_unbalanced(&f, ni).unwrap();
            }
            let time = start_time.elapsed();
            println!("div_unbalanced {}", time.as_millis());
        }
    }

    #[test]
    fn test_div_unbalanced() {

        const MAX_BUF: usize = 100;
        let mut wb = [0; MAX_BUF];
        let mut buf = [0; MAX_BUF];
        for _ in 0..1000 {
            let s1 = random_normalized_slice(1, MAX_BUF);
            let s2 = random_normalized_slice(s1.len(), MAX_BUF);

            //println!("s1{:?}\ns2{:?}", s1, s2);

            let (q, r) = Mantissa::div_unbalanced(&s2, &s1).unwrap();

            buf[..s1.len()].copy_from_slice(&s1);
            buf[s1.len()..].fill(0);
            let mut d1 = SliceWithSign::new_mut(&mut buf, 1);
            let d2 = SliceWithSign::new(&q, 1);
            let d3 = SliceWithSign::new(&r, 1);
            d1.mul_assign(&d2, &mut wb);
            d1.add_assign(&d3, &mut wb);
            //println!("{:?}\n{:?}\n{:?}\n", q, r, &d1[..s2.len()]);
            assert!(s2 == d1[..s2.len()]);
        }
    }

    fn random_normalized_slice(min_len: usize, max_len: usize) -> Vec<Digit> {
        let mut s1 = Vec::new();
        let l = if max_len > min_len {
            random::<usize>() % (max_len - min_len) + min_len
        } else {
            min_len
        };
        for _ in 0..l {
            s1.push(random());
        }
        let l = s1.len();
        s1[l-1] |= DIGIT_SIGNIFICANT_BIT;
        s1
    }
}