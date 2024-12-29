use super::U256;
use std::ops::{Add, Div, Mul, Sub};
use anchor_lang::{AnchorDeserialize, AnchorSerialize, prelude::borsh, InitSpace};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct Q64_64 {
    value: u128,
}

impl Q64_64 {
    pub const FRACTIONAL_BITS: u32 = 64;
    pub const FRACTIONAL_MASK: u128 = (1u128 << Self::FRACTIONAL_BITS) - 1;
    pub const MAX: Self = Q64_64 { value: u128::MAX };
    pub fn new(value: u128) -> Self {
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
    pub fn abs_diff(&self, other: Self) -> u128 {
        self.value.abs_diff(other.value)
    }
    pub fn sqrt_from_u128(value: u128) -> Self {
        let scaled_value = U256::from(value) << Self::FRACTIONAL_BITS;
        let mut low = U256::zero();
        let mut high = scaled_value;
        let mut mid;

        while high - low > U256::from(1u128) {
            mid = (low + high) / 2;

            let square = (mid.saturating_mul(mid)) >> Self::FRACTIONAL_BITS;
            if square <= scaled_value {
                low = mid;
            } else {
                high = mid;
            }
        }

        let low_square = (low.saturating_mul(low)) >> Self::FRACTIONAL_BITS;
        let high_square = (high.saturating_mul(high)) >> Self::FRACTIONAL_BITS;

        let result = if scaled_value.abs_diff(low_square) <= scaled_value.abs_diff(high_square) {
            low
        } else {
            high
        };

        Self {
            value: result.as_u128(),
        }
    }

    pub fn square_as_u128(self) -> u128 {
        let result = (U256::from(self.value) * U256::from(self.value)) >> (Self::FRACTIONAL_BITS * 2);
        result.as_u128()
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

#[cfg(test)]
mod tests {
    use super::Q64_64;
    
    #[test]
    fn test_add() {
        let a = Q64_64::sqrt_from_u128(1);
        let b = Q64_64::sqrt_from_u128(u64::MAX as u128);
        let result = (a / b).split();
        println!("{:?}.{:?}", result.0.to_be_bytes(), result.1.to_be_bytes());
        assert_eq!(result.0, result.1);
    }
}