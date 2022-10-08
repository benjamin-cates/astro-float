![Rust](https://github.com/stencillogic/astro-float/workflows/Rust/badge.svg)

Astro-float (astronomically large floating point numbers) is a library that implements arbitrary precision floating point numbers purely in Rust.

The library implements the basic operations and functions. It uses classical algorithms such as Karatsuba, Toom-3, Schönhage-Strassen algorithm, etc.

## Usage

Compute Pi with precision of 1024 bit:

``` rust
use astro_float::BigFloatNumber;
use astro_float::PI;
use astro_float::RoundingMode;
use astro_float::Radix;
use astro_float::Error;

// Rounding of all operations
let rm = RoundingMode::ToEven;

// Compute pi: pi = 6*arctan(1/sqrt(3))
let six = BigFloatNumber::from_word(6, 1).unwrap();
let three = BigFloatNumber::parse("3.0", Radix::Dec, 1024+8, rm).unwrap();  // +8 bits of precision to cover error
let mut pi = six.mul(&three.sqrt(rm).unwrap().reciprocal(rm).unwrap().atan(rm).unwrap(), rm).unwrap();
pi.set_precision(1024, rm).unwrap();
let mut epsilon = BigFloatNumber::from_word(1, 1).unwrap();
epsilon.set_exponent(-1021);

// Use library's constant for verifying the result
let pi_lib = PI.with(|v| -> Result<BigFloatNumber, Error> {
    v.borrow_mut().for_prec(1024, rm)
}).unwrap();

// Compare computed constant with library's constant
assert!(pi.sub(&pi_lib, rm).unwrap().abs().unwrap().cmp(&epsilon) <= 0);

// Print computed result as decimal number.
let s = pi.format(Radix::Dec, rm).unwrap();
println!("{}", s);

// output: 3.14159265358979323846264338327950288419716939937510582097494459230781640628620899862803482534211706798214808651328230664709384460955058223172535940812848111745028410270193852110555964462294895493038196442881097566593344612847564823378678316527120190914564856692346034861045432664821339360726024914127372458698858e+0
```

## Contribution

The library is still young and may lack some features or contain bugs. Issues for these or other cases can be opened here: https://github.com/stencillogic/astro-float/issues 