use std::cmp::Ordering;
use uint::construct_uint;

construct_uint! {
	pub(super) struct U256(4);
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
    pub fn get_fractional_bits(self, bits_amount: u8) -> u64 {
        let mask = (U256::from(1) << 128) - 1;
        let shift = 128 - bits_amount.min(128);
        ((self & mask) >> shift).as_u64()
    }
    pub fn sqrt(self) -> Self {
        if self == U256::zero(){
            return self;
        }
        if self == U256::one(){
            return self;
        }
        if self == U256::MAX{
            return U256::from(u128::MAX);
        }
        
        let mut low = U256::one();
        let leading_zeros = self.leading_zeros();
        let shift = (256 - leading_zeros - if leading_zeros != 255 { 1 } else { 0 }) / 2;
        let mut high = self >> shift;
        let mut mid;
        let tolerance = U256::one();
        while high - low > tolerance {
            mid = (low + high) >> 1;

            let square = mid.saturating_mul(mid);

            match square.cmp(&self) {
                Ordering::Less => low = mid,
                Ordering::Equal =>{
                    return mid;
                },
                Ordering::Greater => high = mid,
            }
        }
        if high.leading_zeros() >= 128 && (high * high) == self{
            return high;
        }
        low
    }
    
    pub fn q128_128_to_u128_round(self) -> u128{

        let mut square_as_u128 = (self >> 128).as_u128();
        if square_as_u128 == u128::MAX || square_as_u128 == 0 {
            return square_as_u128;
        }
        if self.get_fractional_bits(8) > 128{
            square_as_u128 += 1;
        }
        square_as_u128
    }
    pub fn checked_q128_128_to_u64_round(self) -> Option<u64>{
        let mut square_as_u64 = (self >> 128).checked_as_u64()?;
        if square_as_u64 == u64::MAX || square_as_u64 == 0{
            return Some(u64::MAX);
        }
        if self.get_fractional_bits(8) > 128{
            square_as_u64 += 1;
        }
        Some(square_as_u64)
    }
}

#[cfg(test)]
mod fuzz_tests {
    use super::*;
    use proptest::prelude::*;

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
                Just(32),
                Just(32),
                Just(65534),
                Just(1024),
                Just(9756157671691129856 << 64),
                Just(18446744073709551615 << 64),
            ]
    }

    proptest! {
            #![proptest_config(ProptestConfig::with_cases(10000))]

           #[test]
            fn test_u256_sqrt(value_1 in arbitrary_u128(), value_2 in arbitrary_u128()) {
                let value_u256 = U256::from(value_1) * U256::from(value_2);
                prop_assert_eq!(value_u256.sqrt(), value_u256.integer_sqrt());
            }
    }
}