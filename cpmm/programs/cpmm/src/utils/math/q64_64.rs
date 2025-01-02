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
    use super::Q64_64;

    #[test]
    fn test_from_u64() {
        let value = 42u64;
        let q = Q64_64::from_u64(value);
        assert_eq!(q.to_u64(), value, "from_u64 -> to_u64 failed");
    }

    #[test]
    fn test_from_f64() {
        let value = 42.125;
        let q = Q64_64::from_f64(value);
        let epsilon = 1e-12;
        assert!((q.to_f64() - value).abs() < epsilon, "from_f64 -> to_f64 failed");
    }

    #[test]
    fn test_addition() {
        let q1 = Q64_64::from_u64(10);
        let q2 = Q64_64::from_u64(20);
        let result = q1 + q2;
        assert_eq!(result.to_u64(), 30, "Addition failed");
    }

    #[test]
    fn test_subtraction() {
        let q1 = Q64_64::from_u64(20);
        let q2 = Q64_64::from_u64(10);
        let result = q1 - q2;
        assert_eq!(result.to_u64(), 10, "Subtraction failed");
    }

    #[test]
    fn test_multiplication() {
        let q1 = Q64_64::from_f64(2.5);
        let q2 = Q64_64::from_f64(4.0);
        let result = q1 * q2;
        let expected = 2.5 * 4.0;
        let epsilon = 1e-12;
        assert!((result.to_f64() - expected).abs() < epsilon, "Multiplication failed");
    }

    #[test]
    fn test_division() {
        let q1 = Q64_64::from_f64(10.0);
        let q2 = Q64_64::from_f64(2.0);
        let result = q1 / q2;
        let expected = 10.0 / 2.0;
        let epsilon = 1e-12;
        assert!((result.to_f64() - expected).abs() < epsilon, "Division failed");
    }

    #[test]
    #[should_panic]
    fn test_addition_overflow() {
        let q1 = Q64_64::new(u128::MAX);
        let q2 = Q64_64::from_u64(1);
        let _ = q1 + q2;
    }

    #[test]
    #[should_panic]
    fn test_subtraction_overflow() {
        let q1 = Q64_64::from_u64(1);
        let q2 = Q64_64::from_u64(2);
        let _ = q1 - q2;
    }

    #[test]
    #[should_panic]
    fn test_multiplication_overflow() {
        let q1 = Q64_64::new(u128::MAX);
        let q2 = Q64_64::from_f64(2.0);
        let _ = q1 * q2;
    }

    #[test]
    #[should_panic]
    fn test_division_by_zero() {
        let q1 = Q64_64::from_f64(1.0);
        let q2 = Q64_64::from_f64(0.0);
        let _ = q1 / q2;
    }

    #[test]
    fn test_square_as_u128() {
        let q = Q64_64::from_u64(3);
        let result = q.square_as_u128();
        assert_eq!(result, 9, "Square as u128 failed");
    }

    #[test]
    fn test_abs_diff() {
        let q1 = Q64_64::from_u64(10);
        let q2 = Q64_64::from_u64(20);
        let diff = q1.abs_diff(q2);
        assert_eq!(diff.to_u64(), 10, "Absolute difference failed");
    }

    #[test]
    fn test_is_zero() {
        let q1 = Q64_64::from_u64(0);
        let q2 = Q64_64::from_f64(0.0);
        assert!(q1.is_zero(), "is_zero failed for integer");
        assert!(q2.is_zero(), "is_zero failed for float");
    }

    #[test]
    fn test_split() {
        let q = Q64_64::from_f64(42.75);
        let (integer, fractional) = q.split();
        assert_eq!(integer, 42, "Split integer part failed");
        assert!(fractional > 0, "Split fractional part failed");
    }

    #[test]
    fn test_sqrt_from_u128() {
        let value = 16u128;
        let q = Q64_64::sqrt_from_u128(value);
        let epsilon = 1e-12;
        assert!((q.to_f64() - 4.0).abs() < epsilon, "Square root failed");
    }
}