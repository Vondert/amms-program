use std::ops::{Add, Div, Mul, Sub};
use anchor_lang::{AnchorDeserialize, AnchorSerialize, prelude::borsh, InitSpace};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct Q64_64 {
    value: u128,
}

impl Q64_64 {
    pub const FRACTIONAL_BITS: u32 = 64;
    pub const FRACTIONAL_MASK: u128 = (1u128 << Self::FRACTIONAL_BITS) - 1;
    pub const FRACTIONAL_SCALE: f64 = 18446744073709551616.0;
    pub const MAX: Self = Q64_64 { value: u128::MAX };
    pub const fn new(value: u128) -> Self {
        Q64_64 { value }
    }
    pub fn from_u128(value: u128) -> Self {
        Self {
            value: value << Self::FRACTIONAL_BITS,
        }
    }

    pub fn from_u64(value: u64) -> Self {
        Self {
            value: (value as u128) << Self::FRACTIONAL_BITS,
        }
    }
    pub fn from_f64(value: f64) -> Self {
        let integer_part = value.floor() as u128;
        let fractional_part = ((value - integer_part as f64) * Self::FRACTIONAL_SCALE).round() as u128;
        Self{
            value: (integer_part << Self::FRACTIONAL_BITS) | fractional_part
        }
    }
    pub fn to_f64(&self) -> f64 {
        let (high_bits, low_bits) = self.split();
        let fractional_part = (low_bits as f64) / Self::FRACTIONAL_SCALE;
        high_bits as f64 + fractional_part
    }
    pub fn to_u64(&self) -> u64 {
        (self.value >> Self::FRACTIONAL_BITS) as u64
    }
    pub fn split(&self) -> (u64, u64) {
        (self.get_integer_bits(), self.get_fractional_bits())
    }

    pub fn raw_value(&self) -> u128 {
        self.value
    }
    pub fn set_raw_value(&mut self, value: u128) {
        self.value = value;
    }
    pub fn get_integer_bits(&self) -> u64 {
        (self.value >> Self::FRACTIONAL_BITS) as u64
    }
    pub fn get_fractional_bits(&self) -> u64 {
        (self.value & Self::FRACTIONAL_MASK) as u64
    }
    pub fn abs_diff(&self, other: Self) -> Q64_64 {
        Q64_64::new(self.value.abs_diff(other.value))
    }
    pub fn sqrt_from_u128(value: u128) -> Self {
        if value == 0{
            return Q64_64::from_u128(0);
        }
        let scaled_value = U256::from(value) << Self::FRACTIONAL_BITS;
        let mut low = U256::zero();
        let mut high = scaled_value;
        let mut mid;

        while high - low > U256::from(1u128) {
            mid = (low + high) >> 1;

            let square = (mid.saturating_mul(mid)) >> Self::FRACTIONAL_BITS;
            if square <= scaled_value {
                low = mid;
            } else {
                high = mid;
            }
        }

        Self {
            value: low.as_u128(),
        }
    }

    pub fn square_as_u128(self) -> u128 {
        ((U256::from(self.value) * U256::from(self.value)) >> (Self::FRACTIONAL_BITS * 2)).as_u128()
    }
    pub fn checked_square_as_u64(self) -> Option<u64> {
        ((U256::from(self.value) * U256::from(self.value)) >> (Self::FRACTIONAL_BITS * 2)).checked_as_u64()
    }
    pub fn square_as_u64(self) -> u64 {
        ((U256::from(self.value) * U256::from(self.value)) >> (Self::FRACTIONAL_BITS * 2)).as_u64()
    }
    pub fn is_zero(&self) -> bool{
        self.value == 0
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
        let result = ((U256::from(self.value) << Q64_64::FRACTIONAL_BITS) / U256::from(rhs.value)).checked_as_u128();
        result.map(|res| Self { value: res })
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
        let result = (U256::from(self.value) << Q64_64::FRACTIONAL_BITS) / U256::from(rhs.value);
        Self {
            value: result.as_u128(),
        }
    }
}

use uint::construct_uint;

construct_uint! {
	struct U256(4);
}
impl U256 {
    pub fn checked_as_u128(&self) -> Option<u128> {
        if self.leading_zeros() < 128{
            return None;
        }
        Some(self.as_u128())
    }
    pub fn checked_as_u64(&self) -> Option<u64> {
        if self.leading_zeros() < 192{
            return None;
        }
        Some(self.as_u64())
    }
}

#[cfg(test)]
mod tests {
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
            let q = Q64_64::from_f64(42.75);
            let expected_raw_value = (42u128 << Q64_64::FRACTIONAL_BITS)
                | ((0.75 * Q64_64::FRACTIONAL_SCALE) as u128);
            assert_eq!(q.raw_value(), expected_raw_value);
        }

        #[test]
        fn test_to_f64() {
            let q = Q64_64::from_f64(42.75);
            assert!((q.to_f64() - 42.75).abs() < 1e-12);
        }

        #[test]
        fn test_to_u64() {
            let q = Q64_64::from_u64(42);
            assert_eq!(q.to_u64(), 42);
        }
        #[test]
        fn test_type_conversion() {
            let q = Q64_64::from_f64(42.75);
            let as_u64 = q.to_u64();
            let from_u64 = Q64_64::from_u64(as_u64);
            assert!((from_u64.to_f64() - 42.0).abs() < 1e-12);
        }
        #[test]
        fn test_split() {
            let q = Q64_64::from_f64(42.75);
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
            let q1 = Q64_64::from_f64(2.5);
            let q2 = Q64_64::from_f64(4.2);
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

            let min_fraction = Q64_64::from_f64(1.0 / Q64_64::FRACTIONAL_SCALE);
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
        fn test_precision() {
            let original = 2412345.00067890234102;
            let q = Q64_64::from_f64(original);
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
            let q = Q64_64::from_f64(1.5);
            let square = q.square_as_u128();
            let expected = 2u128;

            assert_eq!(square, expected, "Square of fractional number failed");
        }

        #[test]
        fn test_square_small_fraction() {
            let q = Q64_64::from_f64(0.25);
            let square = q.square_as_u128();
            let expected = 0u128;

            assert_eq!(square, expected, "Square of small fraction failed");
        }

        #[test]
        fn test_square_large_fractional_number() {
            let q = Q64_64::from_f64(12345.6789);
            let square = q.square_as_u128();

            let expected: f64 = (12345.6789 * 12345.6789);
            assert_eq!(square, expected.floor() as u128, "Square of large fractional number failed");
        }

        #[test]
        fn test_square_near_one_fraction() {
            let q = Q64_64::from_f64(0.9999);
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