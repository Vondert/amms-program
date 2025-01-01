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
    pub const FRACTIONAL_SCALE: f64 = 18446744073709551616.0;
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
        Q64_64::from_u128(self.value.abs_diff(other.value))
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
        let result = (U256::from(self.value) * U256::from(self.value)) >> (Self::FRACTIONAL_BITS * 2);
        result.as_u128()
    }
    pub fn square_as_u64(self) -> u64 {
        let result = (U256::from(self.value) * U256::from(self.value)) >> (Self::FRACTIONAL_BITS * 3);
        result.as_u64()
    }
    pub fn is_zero(&self) -> bool{
        self.value == 0
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
    fn test_conversion_cycle() {
        let q6464_1 = Q64_64::from_u64(1);
        let q6464_2 = Q64_64::from_u64(u64::MAX);
        let res = q6464_1 / q6464_2;
        let res2 = 1.0 / 18446744073709551616.0;
        println!("res = {:?}", res);
        println!("res formatted {}", res.to_f64());
        println!("res2 formatted {}", res2);
        assert_eq!(18446744073709551616.0, 0.0, "Conversion cycle mismatch");
    }
}