use anchor_lang::{AnchorDeserialize, AnchorSerialize, prelude::borsh, Space};
use uint::construct_uint;
use super::{Q64_128};

// Define a 192-bit unsigned integer with Anchor serialization/deserialization.
construct_uint! {
    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub(super) struct U192(3);
}

impl Space for U192 {
	/// Space required for `U192` initialization in bytes.
	const INIT_SPACE: usize = 24;
}

#[cfg(test)]
impl U192 {
	/// Converts `U192` into a `U384` by expanding it into 384 bits.
	///
	/// # Returns
	/// A `U384` instance representing the same value as the `U192` input.
	pub fn into_u384(self) -> U384 {
		U384::from_big_endian(&self.to_big_endian())
	}

	/// Attempts to create a `U192` from a `U384`, ensuring it fits within 192 bits.
	///
	/// # Parameters
	/// - `value`: A `U384` value to be converted into `U192`.
	///
	/// # Returns
	/// An `Option<U192>`:
	/// - `Some(U192)` if the value fits within 192 bits.
	/// - `None` if the value exceeds 192 bits.
	pub fn checked_from_u384(value: U384) -> Option<Self> {
		if value.leading_zeros() < 192 {
			return None;
		}
		Some(U192::from_big_endian(&value.to_big_endian()[24..]))
	}
}

// Define a 384-bit unsigned integer.
construct_uint! {
    pub(super) struct U384(6);
}
impl From<Q64_128> for U384 {
	/// Converts a `Q64_128` value into a `U384` by expanding its internal representation.
	///
	/// # Parameters
	/// - `value`: A `Q64_128` value to be converted.
	///
	/// # Returns
	/// A `U384` instance representing the same value.
	fn from(value: Q64_128) -> Self {
		U384::from_big_endian(&value.raw_value().to_big_endian())
	}
}

impl U384 {
	/// Attempts to convert `U384` into a `u64`, ensuring it fits within 64 bits.
	///
	/// # Returns
	/// An `Option<u64>`:
	/// - `Some(u64)` if the value fits within 64 bits.
	/// - `None` if the value exceeds 64 bits.
	pub fn checked_as_u64(&self) -> Option<u64> {
		if self.leading_zeros() < 320 {
			return None;
		}
		Some(self.as_u64())
	}

	/// Attempts to convert `U384` into a `Q64_128`, ensuring it fits within 192 bits.
	///
	/// # Returns
	/// An `Option<Q64_128>`:
	/// - `Some(Q64_128)` if the value fits within 192 bits.
	/// - `None` if the value exceeds 192 bits.
	pub fn checked_as_q64_128(&self) -> Option<Q64_128> {
		if self.leading_zeros() < 192 {
			return None;
		}
		Some((*self).into())
	}

	/// Rounds and converts a `Q128.256` value to a `u128`.
	///
	/// # Returns
	/// A `u128` value representing the rounded result.
	pub fn q128_256_to_u128_round(&self) -> u128 {
		let mut rounded_value = (self >> 256).as_u128();
		if rounded_value == u128::MAX || rounded_value == 0 {
			return rounded_value;
		}
		if self.get_fractional_bits(256, 8) > 128 {
			rounded_value += 1;
		}
		rounded_value
	}

	/// Rounds and converts a `Q128.256` value to a `Q64_128`.
	///
	/// # Returns
	/// An `Option<Q64_128>`:
	/// - `Some(Q64_128)` if the value fits within the bounds of `Q64_128`.
	/// - `None` if the value overflows.
	pub fn q128_256_to_q64_128_round(&self) -> Option<Q64_128> {
		if self.leading_zeros() < 64 {
			return None;
		}
		let mut rounded_value = Q64_128::from(self >> 128);
		if rounded_value == Q64_128::MAX || rounded_value.is_zero() {
			return Some(rounded_value);
		}
		if self.get_fractional_bits(128, 8) > 128 {
			rounded_value = rounded_value + Q64_128::new(U192::one());
		}
		Some(rounded_value)
	}

	/// Rounds and converts a `Q128.256` value to a `u64`, if possible.
	///
	/// # Returns
	/// An `Option<u64>`:
	/// - `Some(u64)` if the value fits within 64 bits.
	/// - `None` if the value overflows.
	pub fn checked_q128_256_to_u64_round(self) -> Option<u64> {
		let mut rounded_value = (self >> 256).checked_as_u64()?;
		if rounded_value == u64::MAX || rounded_value == 0 {
			return Some(u64::MAX);
		}
		if self.get_fractional_bits(256, 8) > 128 {
			rounded_value += 1;
		}
		Some(rounded_value)
	}

	/// Extracts fractional bits from a `U384` value.
	///
	/// # Parameters
	/// - `fraction_start`: The starting position of the fractional bits (max: 256).
	/// - `bits_amount`: The number of fractional bits to extract.
	///
	/// # Returns
	/// A `u128` value representing the extracted fractional bits.
	pub fn get_fractional_bits(self, fraction_start: u16, bits_amount: u16) -> u128 {
		let limited_start = fraction_start.min(256);
		let mask = (U384::from(1) << limited_start) - 1;
		let shift = limited_start - bits_amount.min(limited_start);
		((self & mask) >> shift).as_u128()
	}
}