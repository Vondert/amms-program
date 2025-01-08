use std::ops::{Add, Div, Mul, Sub};
use anchor_lang::{AnchorDeserialize, AnchorSerialize, prelude::borsh, InitSpace};
use super::U256;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct Q64_64 {
    value: u128,
}

impl Q64_64 {
    pub const FRACTIONAL_BITS: u32 = 64;
    pub const FRACTIONAL_MASK: u128 = (1u128 << Self::FRACTIONAL_BITS) - 1;
    pub const FRACTIONAL_SCALE: f64 = 18446744073709551616.0;
    pub const MAX: Self = Q64_64::new(u128::MAX);
    pub const ONE: Self = Q64_64::from_u128(1u128);
    pub const fn new(value: u128) -> Self {
        Q64_64 { value }
    }
    pub const fn from_u128(value: u128) -> Self {
        Self {
            value: value << Self::FRACTIONAL_BITS,
        }
    }

    pub const fn from_u64(value: u64) -> Self {
        Self {
            value: (value as u128) << Self::FRACTIONAL_BITS,
        }
    }
    pub fn from_f64(value: f64) -> Option<Self> {
        if !(0.0..Self::FRACTIONAL_SCALE).contains(&value){
            return None;
        }
        let integer_part = value.floor() as u128;
        let fractional_part = ((value - integer_part as f64) * Self::FRACTIONAL_SCALE).round() as u128;

        Some(Self{
            value: (integer_part << Self::FRACTIONAL_BITS) | fractional_part
        })
    }
    pub fn from_bits(high_bits: u64, low_bits: u64) -> Self {
        let integer_part = (high_bits as u128) << Self::FRACTIONAL_BITS;
        Self::new(integer_part + (low_bits as u128))
    }
    pub fn to_f64(self) -> f64 {
        let (high_bits, low_bits) = self.split();
        let fractional_part = (low_bits as f64) / Self::FRACTIONAL_SCALE;
        high_bits as f64 + fractional_part
    }
    pub fn to_u64(self) -> u64 {
        (self.value >> Self::FRACTIONAL_BITS) as u64
    }
    pub fn split(self) -> (u64, u64) {
        (self.get_integer_bits(), self.get_fractional_bits())
    }

    pub fn raw_value(self) -> u128 {
        self.value
    }
    pub fn set_raw_value(&mut self, value: u128) {
        self.value = value;
    }
    pub fn get_integer_bits(self) -> u64 {
        (self.value >> Self::FRACTIONAL_BITS) as u64
    }
    pub fn get_fractional_bits(self) -> u64 {
        (self.value & Self::FRACTIONAL_MASK) as u64
    }
    pub fn is_zero(self) -> bool{
        self.value == 0
    }
    fn is_one(self) -> bool{
        self.value == Q64_64::ONE.value
    }

}
impl Q64_64{
    pub fn abs_diff(self, other: Self) -> Q64_64 {
        Q64_64::new(self.value.abs_diff(other.value))
    }
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.value.checked_add(rhs.value).map(|res| Self { value: res })
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.value.checked_sub(rhs.value).map(|res| Self { value: res })
    }

    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        let result = ((U256::from(self.value) * U256::from(rhs.value)) >> Q64_64::FRACTIONAL_BITS).checked_as_u128();

        result.map(|res| Self { value: res })
    }

    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.value == 0 {
            return None;
        }
        let result = (U256::from(self.value) << Self::FRACTIONAL_BITS) / U256::from(rhs.value);
        result.checked_as_u128().map(|res| Self { value: res })
    }
}
impl Q64_64{
    fn square_as_u256(self) -> U256 {
        if self.is_zero() {
            return U256::zero();
        }
        if self.is_one() {
            return U256::from(1u128);
        }
        U256::from(self.value) * U256::from(self.value)
    }
    pub fn square_as_u128(self) -> u128 {
        let square_as_u256 = self.square_as_u256();
        if square_as_u256 == U256::one() || self.is_zero(){
            return square_as_u256.as_u128();
        }
        square_as_u256.q128_128_to_u128_round()
    }

    pub fn checked_square_as_u64(self) -> Option<u64> {
        let square_as_u256 = self.square_as_u256();
        if square_as_u256 == U256::one() || self.is_zero(){
            return Some(square_as_u256.as_u64());
        }
        square_as_u256.checked_q128_128_to_u64_round()
    }
    pub fn square_as_q64_64(self) -> Self {
        if self.is_zero() || self.is_one(){
            return self;
        }
        let square_as_u256 = self.square_as_u256() << Self::FRACTIONAL_BITS;
        Q64_64::new(square_as_u256.q128_128_to_u128_round())
    }
    pub fn square_as_u64(self) -> u64 {
        self.checked_square_as_u64().unwrap()
    }
    pub fn sqrt_from_u128(value: u128) -> Self {
        let scaled_value = U256::from(value) << (2 * Self::FRACTIONAL_BITS);
        Self{
            value: scaled_value.sqrt().as_u128()
        }
    }
    pub fn sqrt(self) -> Self {
        let scaled_value = U256::from(self.value) << Self::FRACTIONAL_BITS;
        Self{
            value: scaled_value.sqrt().as_u128()
        }
    }
    pub fn div_sqrt(q1: Self, q2: Self) -> Option<Self> {
        if q2.raw_value() == 0{
            return None;
        }
        else if q1.raw_value() == q2.raw_value(){
            return Some(Q64_64::ONE)
        }
        let result = ((U256::from(q1.raw_value()) << (2 * Self::FRACTIONAL_BITS)) / U256::from(q2.raw_value())).sqrt();
        Some(Self{
            value: result.as_u128()
        })
    }
}
impl Add for Q64_64 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let result = self.value + rhs.value;
        Self { value: result }
    }
}
impl Sub for Q64_64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value - rhs.value,
        }
    }
}
impl Mul for Q64_64 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let result = (U256::from(self.value) * U256::from(rhs.value)) >> Q64_64::FRACTIONAL_BITS;
        Self {
            value: result.as_u128(),
        }
    }
}
impl Div for Q64_64 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.value == 0 {
            panic!("Division by zero!");
        }
        let result = (U256::from(self.value) << Self::FRACTIONAL_BITS) / U256::from(rhs.value);
        Self {
            value: result.as_u128(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod unit_tests{
        use super::*;
        mod constructors_and_conversions {
            use super::*;

            #[test]
            fn test_new() {
                let q = Q64_64::new(12345);
                assert_eq!(q.raw_value(), 12345);
            }

            #[test]
            fn test_from_u128() {
                let q = Q64_64::from_u128(10);
                assert_eq!(q.raw_value(), 10 << Q64_64::FRACTIONAL_BITS);
            }

            #[test]
            fn test_from_u64() {
                let q = Q64_64::from_u64(42);
                assert_eq!(q.raw_value(), (42u128) << Q64_64::FRACTIONAL_BITS);
            }

            #[test]
            fn test_from_f64() {
                let q = Q64_64::from_f64(42.75).unwrap();
                let expected_raw_value = (42u128 << Q64_64::FRACTIONAL_BITS)
                    | ((0.75 * Q64_64::FRACTIONAL_SCALE) as u128);
                assert_eq!(q.raw_value(), expected_raw_value);
            }
            #[test]
            fn test_invalid_from_f64() {
                let q1 = Q64_64::from_f64(-0.01);
                let q2 = Q64_64::from_f64(Q64_64::FRACTIONAL_SCALE);
                assert!(q1.is_none() && q2.is_none());
            }
            #[test]
            fn test_to_f64() {
                let q = Q64_64::from_f64(42.75).unwrap();
                assert!((q.to_f64() - 42.75).abs() < 1e-12);
            }
            #[test]
            fn test_to_u64() {
                let q = Q64_64::from_u64(42);
                assert_eq!(q.to_u64(), 42);
            }
            #[test]
            fn test_type_conversion() {
                let q = Q64_64::from_f64(42.75).unwrap();
                let as_u64 = q.to_u64();
                let from_u64 = Q64_64::from_u64(as_u64);
                assert!((from_u64.to_f64() - 42.0).abs() < 1e-12);
            }
            #[test]
            fn test_split() {
                let q = Q64_64::from_f64(42.75).unwrap();
                let (integer, fractional) = q.split();
                assert_eq!(integer, 42);
                let expected_fractional = (0.75 * Q64_64::FRACTIONAL_SCALE) as u64;
                assert_eq!(fractional, expected_fractional);
            }
            #[test]
            fn test_raw_value_access() {
                let mut q = Q64_64::new(0);
                q.set_raw_value(123456);
                assert_eq!(q.raw_value(), 123456);
            }
        }
        mod arithmetic_operations {
            use super::*;

            #[test]
            fn test_add() {
                let q1 = Q64_64::from_u64(10);
                let q2 = Q64_64::from_u64(20);
                assert_eq!((q1 + q2).to_u64(), 30);
            }

            #[test]
            fn test_sub() {
                let q1 = Q64_64::from_u64(50);
                let q2 = Q64_64::from_u64(20);
                assert_eq!((q1 - q2).to_u64(), 30);
            }

            #[test]
            fn test_mul() {
                let q1 = Q64_64::from_u64(3);
                let q2 = Q64_64::from_u64(4);
                assert_eq!((q1 * q2).to_u64(), 12);
            }

            #[test]
            fn test_div() {
                let q1 = Q64_64::from_u64(10);
                let q2 = Q64_64::from_u64(2);
                assert_eq!((q1 / q2).to_u64(), 5);
            }
            #[test]
            fn test_abs_diff() {
                let q1 = Q64_64::from_u64(50);
                let q2 = Q64_64::from_u64(20);
                assert_eq!(q1.abs_diff(q2).to_u64(), 30);
            }
            #[test]
            fn test_fractional_multiplication() {
                let q1 = Q64_64::from_f64(2.5).unwrap();
                let q2 = Q64_64::from_f64(4.2).unwrap();
                let result = q1 * q2;

                assert!((result.to_f64() - 10.5).abs() < 1e-12, "Multiplication failed");
            }
        }
        mod checked_operations {
            use super::*;

            #[test]
            fn test_checked_add() {
                let q1 = Q64_64::from_u128(u128::MAX >> 1);
                let q2 = Q64_64::from_u128(u128::MAX >> 1);
                let q3 = Q64_64::from_u128(1u128);
                let q4 = Q64_64::from_u128(100u128);
                assert!(q1.checked_add(q2).is_none());
                assert_eq!(q3.checked_add(q4).unwrap().to_u64(), 101);
            }

            #[test]
            fn test_checked_sub() {
                let q1 = Q64_64::from_u64(50);
                let q2 = Q64_64::from_u64(100);
                assert!(q1.checked_sub(q2).is_none());
                assert_eq!(q2.checked_sub(q1).unwrap().to_u64(), 50u64);
            }

            #[test]
            fn test_checked_mul() {
                let q1 = Q64_64::from_u128(u128::MAX >> 1);
                let q2 = Q64_64::from_u128(2);
                let q3 = Q64_64::from_u128(31);
                assert!(q1.checked_mul(q2).is_none());
                assert_eq!(q2.checked_mul(q3).unwrap().to_u64(), 62u64);
            }

            #[test]
            fn test_checked_div() {
                let q1 = Q64_64::from_u64(100);
                let q2 = Q64_64::from_u64(0);
                let q3 = Q64_64::from_u64(150);
                let result = q3.checked_div(q1).unwrap();
                assert!(q1.checked_div(q2).is_none());
                assert!((result.to_f64() - 1.5).abs() < 1e-12);
            }

            #[test]
            fn test_checked_square_overflow() {
                let q = Q64_64::from_u128(u128::MAX >> 1);
                assert!(q.checked_square_as_u64().is_none(), "Checked square overflow failed");
            }
        }
        mod edge_cases {
            use super::*;

            #[test]
            fn test_extreme_values() {
                let max = Q64_64::MAX;
                assert_eq!(max.raw_value(), u128::MAX);

                let min_fraction = Q64_64::from_f64(1.0 / Q64_64::FRACTIONAL_SCALE).unwrap();
                assert_eq!(min_fraction.raw_value(), 1);

                let near_max = Q64_64::from_u128(u128::MAX >> 1);
                assert!(near_max.raw_value() < u128::MAX);
            }

            #[test]
            fn test_is_zero() {
                let q = Q64_64::from_u64(0);
                assert!(q.is_zero());
                let q = Q64_64::from_u64(1);
                assert!(!q.is_zero());
            }
            #[test]
            fn test_is_one() {
                let q = Q64_64::from_u64(1);
                assert!(q.is_one());
                let q = Q64_64::from_u64(0);
                assert!(!q.is_one());
            }
            #[test]
            fn test_precision() {
                let original = 2412345.00067890234102;
                let q = Q64_64::from_f64(original).unwrap();
                let back_to_f64 = q.to_f64();

                assert!((back_to_f64 - original).abs() < 1e-12, "Precision loss detected");
            }
        }
        mod square_and_sqrt {
            use super::*;

            #[test]
            fn test_sqrt_from_u128() {
                let q = Q64_64::sqrt_from_u128(256);
                assert_eq!(q.to_u64(), 16);

                let q = Q64_64::sqrt_from_u128(10);
                assert!((q.to_f64() - 3.16227766).abs() < 1e-9, "Square root approximation failed");
            }

            #[test]
            fn test_square_operations() {
                let q = Q64_64::from_u64(3);
                assert_eq!(q.square_as_u128(), 9);
                assert_eq!(q.checked_square_as_u64(), Some(9));
                assert_eq!(q.square_as_u64(), 9);
            }

            #[test]
            fn test_square_fractional_number() {
                let q = Q64_64::from_f64(1.5).unwrap();
                let square = q.square_as_u128();
                let expected = 2u128;

                assert_eq!(square, expected, "Square of fractional number failed");
            }

            #[test]
            fn test_square_small_fraction() {
                let q = Q64_64::from_f64(0.25).unwrap();
                let square = q.square_as_u128();
                let expected = 0u128;

                assert_eq!(square, expected, "Square of small fraction failed");
            }

            #[test]
            fn test_square_large_fractional_number() {
                let q = Q64_64::from_f64(12345.6789).unwrap();
                let square = q.square_as_u128();

                let expected: f64 = (12345.6789 * 12345.6789);
                assert_eq!(square, expected.floor() as u128, "Square of large fractional number failed");
            }

            #[test]
            fn test_square_near_one_fraction() {
                let q = Q64_64::from_f64(0.9999).unwrap();
                let square = q.square_as_u128();

                let expected = 0;

                assert_eq!(square, expected, "Square of near-one fraction failed");
            }
        }
        mod panic_tests {
            use super::*;
            #[test]
            #[should_panic(expected = "Division by zero!")]
            fn test_div_panic() {
                let q1 = Q64_64::from_u64(10);
                let q2 = Q64_64::from_u64(0);
                let _ = q1 / q2;
            }
        }
    }
    
    mod fuzz_tests{
        use super::*;
        use proptest::prelude::*;
        fn arbitrary_f64() -> impl Strategy<Value = f64> {
            prop_oneof![
                0.0..Q64_64::FRACTIONAL_SCALE,
                Just(0.00000001),
                Just(1.0),
                Just(Q64_64::FRACTIONAL_SCALE * 0.99),
            ]
        }
        fn invalid_arbitrary_f64() -> impl Strategy<Value = f64>{
            prop_oneof![
                f64::MIN..0.0,
                Q64_64::FRACTIONAL_SCALE..f64::MAX,
                Just(f64::NEG_INFINITY),
                Just(f64::INFINITY),
                Just(f64::NAN),
            ]
        }
        fn arbitrary_u128() -> impl Strategy<Value = u128> {
            prop_oneof![
                0..=u128::MAX,
                Just(0),
                Just(1),
                Just(2),
                Just(u128::MAX),
                Just(u128::MAX / 2),
                Just(u128::MAX - 1),
                Just(1024),
                Just(4095),
                Just(8191),
                Just(10000),
                Just(12321),
                Just(65535),
                Just(12345),
                Just(54321),
                Just(99999),
                Just(45678),
                Just(87654),
                Just(10001),
                Just(9999),
                Just(2047),
                Just(65534),
                Just(9756157671691129856 << 64),
                Just(18446744073709551615 << 64),
            ]
        }
        fn arbitrary_u64() -> impl Strategy<Value = u64> {
            prop_oneof![
            Just(0),
            Just(1),
            Just(2),
            Just(u64::MAX),
            Just(u64::MAX / 2),
            Just(u64::MAX - 1),
            Just(1024),
            Just(4095),
            Just(8191),
            Just(10000),
            Just(12321),
            Just(65535),
            Just(12345),
            Just(54321),
            Just(99999),
            Just(45678),
            Just(87654),
            Just(10001),
            Just(9999),
            Just(2047),
            Just(65534)
            ]
        }
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(10000))]
            
            #[test]
            fn test_types_conversion_round_trip(value_f64 in arbitrary_f64()) {
                let value_u64 = value_f64.floor() as u64;
                let value_round_f64 = value_u64 as f64;
            
                let q_f64 = Q64_64::from_f64(value_f64).unwrap();
                let q_u64 = Q64_64::from_u64(value_u64);
            
                let converted_back_f64 = q_f64.to_f64();
                let converted_back_u64 = q_f64.to_u64();
            
                let f64_from_u64 = q_u64.to_f64();
            
                prop_assert!((value_f64 - converted_back_f64).abs() < 1e-12, "Precision loss detected for value: {}", value_f64);
            
                prop_assert!((value_round_f64 - f64_from_u64).abs() < 1e-12, "Precision loss detected for round f64 from u64: {}, f64_from_u64: {}", value_round_f64, f64_from_u64);
            
                prop_assert_eq!(value_u64, converted_back_u64, "Value mismatch for u64 conversion: original u64: {}, converted_back_u64: {}", value_u64, converted_back_u64);
            
                prop_assert!((value_f64 - value_round_f64).abs() < 1.0, "Fractional loss detected: value_f64: {}, value_round_f64: {}", value_f64, value_round_f64);
            }
            
            #[test]
            fn test_invalid_f64_conversion(value_f64 in invalid_arbitrary_f64()) {
                let q_f64 = Q64_64::from_f64(value_f64);
                prop_assert!(q_f64.is_none(), "Invalid value should not be convertible: {}", value_f64);
            }
            
            #[test]
            fn test_u128_sqrt_and_square_round_trip(value in arbitrary_u128()) {
                let sqrt = Q64_64::sqrt_from_u128(value);
                let square_u128 = sqrt.square_as_u128();
                let square_option_u64 = sqrt.checked_square_as_u64();
                let tolerance = 2 >> (value.leading_zeros().max(3) - 3);
                prop_assert!(square_u128.abs_diff(value) <= tolerance, "Invalid square u128 {} from sqrt {} for: {}", square_u128, sqrt.to_f64(), value);
                    
                if value >> 64 != 0{
                    prop_assert!(square_option_u64.is_none(), "Invalid value should not be convertible to u64: {}", value);
                }
                else{
                    let value_u64 = value as u64;
                    let square_u64 = square_option_u64.unwrap();
                    prop_assert!(square_u64.abs_diff(value_u64) <= tolerance as u64, "Invalid square u64 {} from sqrt {} for: {}", square_u64, sqrt.to_f64(), value_u64);
                }
            }
            #[test]
            fn test_q64_64_sqrt_and_square_round_trip(value in arbitrary_u128()) {
                let q6464 = Q64_64::new(value);
                let sqrt = q6464.sqrt();
                let square = sqrt.square_as_q64_64();
                println!("\nValue {}", value);
                println!("Q6464 {}", q6464.to_f64());
                println!("Sqrt {}", sqrt.raw_value());
                println!("Sqrt f64 {}", sqrt.to_f64());
                println!("Square {}", square.raw_value());
                println!("Square f64 {}", square.to_f64());
                let diff = square.abs_diff(q6464);
                println!("Diff {} {}", diff.raw_value(), diff.to_f64());
                prop_assert!(diff.to_f64() <= 1e-9, "Invalid square Q6464 {} from sqrt {} for: {}", square.raw_value(), sqrt.to_f64(), value);
            }
            #[test]
            fn test_q64_64_div_sqrt_and_square_round_trip(value1 in arbitrary_u128(), value2 in arbitrary_u128()) {
                let q6464_value1 = Q64_64::new(value1);
                let q6464_value2 = Q64_64::new(value2);
                let div_sqrt_optional = Q64_64::div_sqrt(q6464_value1, q6464_value2);

                println!("\nValues {} {}", value1, value2);
                if value2 != 0 {
                    let div_sqrt = div_sqrt_optional.unwrap();
                    println!("Div sqrt raw value{}", div_sqrt.raw_value());
                    println!("Div sqrt f64 {}", div_sqrt.to_f64());
                    if q6464_value1.value.leading_zeros() + q6464_value2.value.leading_zeros() <= Q64_64::FRACTIONAL_BITS {
                        let checked_div_square = q6464_value1.checked_div(q6464_value2).unwrap();
                        let square = div_sqrt.square_as_q64_64();
                        println!("Square from div square raw value {}", square.raw_value());
                        println!("Square from div square f64 {}", square.to_f64());
                        println!("Checked div square f64 {}", checked_div_square.to_f64());
                        let diff = (square.to_f64() - checked_div_square.to_f64()).abs();
                        println!("Difference {}", diff);
                        prop_assert!(diff <= 1e-9, "Invalid square Q6464 {} from sqrt {} for: {}", square.raw_value(), div_sqrt.to_f64(), checked_div_square.to_f64());
                    }
                    else{
                        let float = (value1 as f64 / value2 as f64);
                        println!("Square f64 {}", float);
                        let diff = (div_sqrt.to_f64() - float.sqrt()).abs();
                        println!("Difference {}", diff);
                        prop_assert!(diff <= 2048.0, "Invalid square sqrt {}; {} for: {}", div_sqrt.to_f64(), float.sqrt(), float);
                    }

                }
                else{
                    prop_assert!(div_sqrt_optional.is_none());
                }

            }
  /*          #[test]
            fn test_square_mul_and_div(value in arbitrary_u128(), factor in arbitrary_u128()) {
                let sqrt = Q64_64::sqrt_from_u128(value);

                let square_u128 = sqrt.square_as_u128();
                let tolerance = value / 1_000_000_000_000;
                prop_assert!(square_u128.abs_diff(value) <= tolerance, "Invalid square u128 {} from sqrt {} for: {}", square_u128, sqrt.to_f64(), value);
                let factor_q64 = Q64_64::new(factor);
                let factor_leading_zeros = factor.leading_zeros();
                let square_leading_zeros = square_u128.leading_zeros();                 
  
                let square_mul_u128 = sqrt.checked_square_mul_as_u128(factor_q64);
                let square_div_u128 = sqrt.checked_square_div_as_u128(factor_q64);
                if factor == 0 || square_u128 == 0{
                    prop_assert!(square_mul_u128.unwrap() == 0, "Square mul must be 0 for square {} and factor {}. But it is {}", square_u128, factor, square_mul_u128.unwrap());
                }
                else if factor_leading_zeros >= 64 || square_leading_zeros + factor_leading_zeros >= 64{
                    prop_assert!(square_mul_u128.is_some(), "Square mul must be some for square {} and factor {}", value, factor);
                }
                else {
                    prop_assert!(square_mul_u128.is_none(), "Square mul must be none for square {} and factor {}. But it is {}", square_u128, factor, square_mul_u128.unwrap());
                }
                
                if factor == 0 {
                    prop_assert!(square_div_u128.is_none(), "Square div must be none for square {} and factor {}. But it is {}", square_u128, factor, square_div_u128.unwrap());
                }
                else if square_u128 == 0{
                    prop_assert!(square_div_u128.unwrap() == 0, "Square div must be 0 for square {} and factor {}. But it is {}", square_u128, factor, square_div_u128.unwrap());
                }
                else if 64 + factor_leading_zeros < square_leading_zeros{
                    prop_assert!(square_div_u128.unwrap() == 0, "Square div must be 0 for square {} and factor {}. But it is {}", square_u128, factor, square_div_u128.unwrap());
                }
                else if square_leading_zeros < 64 && square_leading_zeros + factor_leading_zeros > 64{
                    prop_assert!(square_div_u128.is_none(), "Square div must be none for square {} and factor {}. But it is {}", square_u128, factor, square_div_u128.unwrap());
                }
                else{
                    prop_assert!(square_div_u128.unwrap() != 0, "Square div some for square {} and factor {}. But it is {}", square_u128, factor, square_div_u128.unwrap());
                }
            }*/
            
            #[test]
            fn test_abs_diff(value1 in arbitrary_u128(), value2 in arbitrary_u128()) {
                let q1 = Q64_64::new(value1);
                let q2 = Q64_64::new(value2);
            
                let abs_diff = q1.abs_diff(q2);
                let expected_diff = value1.abs_diff(value2);
            
                prop_assert_eq!(
                    abs_diff.raw_value(),
                    expected_diff,
                    "Invalid abs_diff {} for values: {} and {}",
                    abs_diff.raw_value(),
                    value1,
                    value2
                );
            }
            
            #[test]
            fn test_checked_add(value1 in arbitrary_u128(), value2 in arbitrary_u128()) {
                let q1 = Q64_64::new(value1);
                let q2 = Q64_64::new(value2);
            
                let result = q1.checked_add(q2);
            
                if let Some(sum) = result {
                    let expected_sum = value1.checked_add(value2).unwrap();
                    prop_assert_eq!(
                        sum.raw_value(),
                        expected_sum,
                        "Invalid checked_add result {} for values: {} and {}",
                        sum.raw_value(),
                        value1,
                        value2
                    );
                } else {
                    prop_assert!(
                        value1.checked_add(value2).is_none(),
                        "checked_add should return None for values: {} and {}",
                        value1,
                        value2
                    );
                }
            }
            #[test]
            fn test_checked_sub(value1 in arbitrary_u128(), value2 in arbitrary_u128()) {
                let q1 = Q64_64::new(value1);
                let q2 = Q64_64::new(value2);
            
                let result = q1.checked_sub(q2);
            
                if let Some(diff) = result {
                    let expected_diff = value1.checked_sub(value2).unwrap();
                    prop_assert_eq!(
                        diff.raw_value(),
                        expected_diff,
                        "Invalid checked_sub result {} for values: {} and {}",
                        diff.raw_value(),
                        value1,
                        value2
                    );
                } else {
                    prop_assert!(
                        value1.checked_sub(value2).is_none(),
                        "checked_sub should return None for values: {} and {}",
                        value1,
                        value2
                    );
                }
            }
            
            #[test]
            fn test_checked_mul(value1 in arbitrary_u128(), value2 in arbitrary_u128()) {
                let q1 = Q64_64::new(value1);
                let q2 = Q64_64::new(value2);
            
                let result = q1.checked_mul(q2);
            
                if let Some(product) = result {
                    let expected_product = ((U256::from(value1) * U256::from(value2)) >> Q64_64::FRACTIONAL_BITS)
                        .checked_as_u128()
                        .unwrap();
                    prop_assert_eq!(
                        product.raw_value(),
                        expected_product,
                        "Invalid checked_mul result {} for values: {} and {}",
                        product.raw_value(),
                        value1,
                        value2
                    );
                } else {
                    prop_assert!(
                        ((U256::from(value1) * U256::from(value2)) >> Q64_64::FRACTIONAL_BITS)
                            .checked_as_u128()
                            .is_none(),
                        "checked_mul should return None for values: {} and {}",
                        value1,
                        value2
                    );
                }
            }
            
            #[test]
            fn test_checked_div(value1 in arbitrary_u128(), value2 in arbitrary_u128()) {
                let q1 = Q64_64::new(value1);
                let q2 = Q64_64::new(value2);
            
                let result = q1.checked_div(q2);
            
                if value2 == 0 {
                    prop_assert!(result.is_none(), "Division by zero should return None");
                } else if let Some(quotient) = result {
                    let expected_quotient = ((U256::from(value1) << Q64_64::FRACTIONAL_BITS) / U256::from(value2))
                        .checked_as_u128()
                        .unwrap();
                    prop_assert_eq!(
                        quotient.raw_value(),
                        expected_quotient,
                        "Invalid checked_div result {} for values: {} and {}",
                        quotient.raw_value(),
                        value1,
                        value2
                    );
                } else {
                    prop_assert!(
                        ((U256::from(value1) << Q64_64::FRACTIONAL_BITS) / U256::from(value2))
                            .checked_as_u128()
                            .is_none(),
                        "checked_div should return None for values: {} and {}",
                        value1,
                        value2
                    );
                }
            }
        }
    }
}