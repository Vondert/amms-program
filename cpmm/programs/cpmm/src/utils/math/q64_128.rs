use anchor_lang::{AnchorDeserialize, AnchorSerialize, prelude::borsh, InitSpace};
use std::ops::{Add, Div, Mul, Sub};
use super::{U384, U192};

/// Represents a fixed-point number with 64 integer bits and 128 fractional bits.
///
/// The `Q64_128` type is useful for high-precision arithmetic where fractional values
/// need to be represented without losing precision. Internally, it uses a 192-bit
/// unsigned integer (`U192`) to store the value, where the most significant 64 bits
/// represent the integer part, and the least significant 128 bits represent the fractional part.
///
/// This type provides utilities for fixed-point arithmetic, conversions from primitive types,
/// and accessing the integer and fractional components of the value.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct Q64_128 {
	/// The internal representation of the fixed-point value as a 192-bit unsigned integer.
	value: U192,
}

impl Q64_128 {
	/// The number of fractional bits (128).
	pub const FRACTIONAL_BITS: u32 = 128;

	/// The number of integer bits (64).
	pub const INTEGER_BITS: u32 = 64;

	/// The bitmask used to extract fractional bits.
	const FRACTIONAL_MASK: U192 = U192([u64::MAX, u64::MAX, 0]);

	/// The minimal fraction required for rounding.
	const ROUND_FRACTION: U192 = U192([0, 18446744073709551360, 0]);

	/// The scaling factor used for converting floating-point values to fixed-point.
	#[cfg(test)]
	const FRACTIONAL_SCALE: f64 = 340282366920938463463374607431768211456.0;

	/// The maximum representable value for `Q64_128`.
	pub const MAX: Self = Q64_128::new(U192::MAX);

	/// A constant representing the value 1 in `Q64_128` format.
	pub const ONE: Self = Q64_128::from_u64(1u64);

	/// Creates a new `Q64_128` instance from a `U192` value.
	///
	/// # Parameters
	/// - `value`: The raw 192-bit unsigned integer representing the fixed-point value.
	///
	/// # Returns
	/// A new `Q64_128` instance.
	pub(super) const fn new(value: U192) -> Self {
		Q64_128 { value }
	}

	/// Creates a `Q64_128` instance from an unsigned 64-bit integer.
	///
	/// # Parameters
	/// - `value`: The integer to be converted to a fixed-point value.
	///
	/// # Returns
	/// A `Q64_128` instance representing the integer as a fixed-point number.
	pub const fn from_u64(value: u64) -> Self {
		Self {
			value: U192([0, 0, value]),
		}
	}

	/// Creates a `Q64_128` instance from high and low bits.
	///
	/// # Parameters
	/// - `high_bits`: The 64 most significant bits representing the integer part.
	/// - `low_bits`: The 128 least significant bits representing the fractional part.
	///
	/// # Returns
	/// A `Q64_128` instance combining the integer and fractional parts.
	pub const fn from_bits(high_bits: u64, low_bits: u128) -> Self {
		Self::new(U192([
			(low_bits & (u64::MAX as u128)) as u64,
			(low_bits >> 64) as u64,
			high_bits,
		]))
	}

	/// Converts the fixed-point value to a 64-bit unsigned integer.
	///
	/// # Returns
	/// The integer part of the fixed-point value.
	pub fn as_u64(&self) -> u64 {
		(self.value >> Self::FRACTIONAL_BITS).as_u64()
	}
	
	/// Converts the fixed-point value to a 64-bit unsigned integer with rounding.
	///
	/// This method extracts the integer part of the fixed-point value stored in the `value` field
	/// and rounds it up if the fractional part meets or exceeds the rounding threshold.
	///
	/// # Returns
	/// - A `u64` representing the rounded integer value of the fixed-point number.
	pub fn as_u64_round(&self) -> u64 {
		let mut integer = (self.value >> Self::FRACTIONAL_BITS).as_u64();
		if integer < u64::MAX && (self.value & Self::FRACTIONAL_MASK >= Self::ROUND_FRACTION){
			integer += 1;
		}
		integer
	}
	/// Splits the fixed-point value into its integer and fractional components.
	///
	/// # Returns
	/// A tuple `(integer_part, fractional_part)`, where:
	/// - `integer_part` is the integer portion of the value.
	/// - `fractional_part` is the fractional portion as a 128-bit unsigned integer.
	pub fn split(&self) -> (u64, u128) {
		(self.get_integer_bits(), self.get_fractional_bits())
	}

	/// Retrieves the raw 192-bit value of the fixed-point number.
	///
	/// # Returns
	/// The raw `U192` value.
	pub(super) fn raw_value(&self) -> U192 {
		self.value
	}

	/// Extracts the integer bits from the fixed-point value.
	///
	/// # Returns
	/// The integer part as a 64-bit unsigned integer.
	pub fn get_integer_bits(&self) -> u64 {
		(self.value >> Self::FRACTIONAL_BITS).as_u64()
	}

	/// Extracts the fractional bits from the fixed-point value.
	///
	/// # Returns
	/// The fractional part as a 128-bit unsigned integer.
	pub fn get_fractional_bits(&self) -> u128 {
		(self.value & Self::FRACTIONAL_MASK).as_u128()
	}

	/// Checks if the value is zero.
	///
	/// # Returns
	/// `true` if the value is zero, otherwise `false`.
	pub fn is_zero(&self) -> bool {
		self.value == U192::zero()
	}

	/// Checks if the value is one.
	///
	/// # Returns
	/// `true` if the value is one, otherwise `false`.
	pub fn is_one(&self) -> bool {
		self.value == Q64_128::ONE.value
	}

	/// Creates a `Q64_128` instance from a `f64` value (for testing purposes).
	///
	/// # Parameters
	/// - `value`: A floating-point value to be converted to `Q64_128`.
	///
	/// # Returns
	/// An `Option<Q64_128>`:
	/// - `Some` containing the converted value if it is within the valid range.
	/// - `None` if the value is outside the valid range for `Q64_128`.
	///
	/// # Valid Range
	/// - The value must be within `[0.0, 2^64)`.
	#[cfg(test)]
	pub fn from_f64(value: f64) -> Option<Self> {
		if !(0.0..18446744073709551616.0).contains(&value) {
			return None;
		}
		let integer_part = value.floor() as u64;
		let fractional_part = ((value - integer_part as f64) * Self::FRACTIONAL_SCALE).round() as u128;
		Some(Self::from_bits(integer_part, fractional_part))
	}
}

/// Implements conversion from `U384` to `Q64_128`.
///
/// # Panic
/// - Panics if the `U384` value cannot fit into 192 bits. This occurs when
///   the number of leading zeros in the `U384` value is less than 192.
///
/// # Parameters
/// - `value`: A `U384` value to be converted into `Q64_128`.
///
/// # Returns
/// A `Q64_128` instance representing the input `U384` value.
impl From<U384> for Q64_128 {
	fn from(value: U384) -> Self {
		if value.leading_zeros() < 192 {
			panic!("Cannot convert U384 to Q64_128");
		}
		let u192 = U192::from_big_endian(&(value.to_big_endian()[24..]));
		Self::new(u192)
	}
}

/// Implements conversion from `Q64_128` to `f64` (testing only).
///
/// # Notes
/// This implementation is provided for testing purposes to convert a `Q64_128`
/// value into a floating-point number. It separates the integer and fractional
/// parts and combines them to form the `f64` value.
///
/// # Parameters
/// - `value`: A `Q64_128` instance to be converted into a `f64`.
///
/// # Returns
/// A `f64` representation of the input `Q64_128`.
#[cfg(test)]
impl From<Q64_128> for f64 {
	fn from(value: Q64_128) -> Self {
		let (high_bits, low_bits) = value.split();
		let fractional_part = (low_bits as f64) / Q64_128::FRACTIONAL_SCALE;
		high_bits as f64 + fractional_part
	}
}

/// Implements addition for `Q64_128`.
///
/// # Behavior
/// Adds two `Q64_128` values together by performing an addition on their raw
/// `U192` representations.
///
/// # Returns
/// A new `Q64_128` instance containing the sum of the two inputs.
impl Add for Q64_128 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		let result = self.value + rhs.value;
		Self { value: result }
	}
}

/// Implements subtraction for `Q64_128`.
///
/// # Behavior
/// Subtracts one `Q64_128` value from another by performing a subtraction on their
/// raw `U192` representations.
///
/// # Returns
/// A new `Q64_128` instance containing the result of the subtraction.
impl Sub for Q64_128 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			value: self.value - rhs.value,
		}
	}
}

/// Implements multiplication for `Q64_128`.
///
/// # Behavior
/// Multiplies two `Q64_128` values by converting them to `U384`, performing the
/// multiplication, and then shifting the result to adjust for the fractional bits.
///
/// # Returns
/// A new `Q64_128` instance containing the product of the two inputs.
impl Mul for Q64_128 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let result = (U384::from(self) * U384::from(rhs)) >> Q64_128::FRACTIONAL_BITS;
		result.into()
	}
}

/// Implements division for `Q64_128`.
///
/// # Behavior
/// Divides one `Q64_128` value by another by converting them to `U384`, performing
/// the division, and adjusting for the fractional bits by shifting the numerator.
///
/// # Panic
/// - Panics if the divisor (`rhs`) is zero to avoid division by zero.
///
/// # Returns
/// A new `Q64_128` instance containing the result of the division.
impl Div for Q64_128 {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		if rhs.is_zero() {
			panic!("Division by zero!");
		}
		let result = (U384::from(self) << Self::FRACTIONAL_BITS) / U384::from(rhs);
		result.into()
	}
}

impl Q64_128 {
	/// Calculates the absolute difference between two `Q64_128` values.
	///
	/// # Parameters
	/// - `self`: The first `Q64_128` value.
	/// - `other`: The second `Q64_128` value.
	///
	/// # Returns
	/// A new `Q64_128` instance representing the absolute difference between `self` and `other`.
	pub fn abs_diff(self, other: Self) -> Q64_128 {
		Q64_128::new(self.value.abs_diff(other.value))
	}

	/// Performs a checked addition of two `Q64_128` values.
	///
	/// # Parameters
	/// - `self`: The first `Q64_128` value.
	/// - `rhs`: The second `Q64_128` value to add to `self`.
	///
	/// # Returns
	/// An `Option<Q64_128>`:
	/// - `Some(Q64_128)` containing the result if the addition does not overflow.
	/// - `None` if the addition overflows.
	pub fn checked_add(self, rhs: Self) -> Option<Self> {
		self.value.checked_add(rhs.value).map(|res| Self { value: res })
	}

	/// Performs a checked subtraction of two `Q64_128` values.
	///
	/// # Parameters
	/// - `self`: The first `Q64_128` value.
	/// - `rhs`: The second `Q64_128` value to subtract from `self`.
	///
	/// # Returns
	/// An `Option<Q64_128>`:
	/// - `Some(Q64_128)` containing the result if the subtraction does not underflow.
	/// - `None` if the subtraction underflows.
	pub fn checked_sub(self, rhs: Self) -> Option<Self> {
		self.value.checked_sub(rhs.value).map(|res| Self { value: res })
	}

	/// Performs a checked multiplication of two `Q64_128` values.
	///
	/// # Parameters
	/// - `self`: The first `Q64_128` value.
	/// - `rhs`: The second `Q64_128` value to multiply with `self`.
	///
	/// # Returns
	/// An `Option<Q64_128>`:
	/// - `Some(Q64_128)` containing the result if the multiplication does not overflow.
	/// - `None` if the multiplication overflows.
	pub fn checked_mul(self, rhs: Self) -> Option<Self> {
		((U384::from(self) * U384::from(rhs)) >> Q64_128::FRACTIONAL_BITS).checked_as_q64_128()
	}

	/// Performs a checked division of two `Q64_128` values.
	///
	/// # Parameters
	/// - `self`: The numerator `Q64_128` value.
	/// - `rhs`: The denominator `Q64_128` value.
	///
	/// # Returns
	/// An `Option<Q64_128>`:
	/// - `Some(Q64_128)` containing the result if the division is valid.
	/// - `None` if the denominator is zero.
	pub fn checked_div(self, rhs: Self) -> Option<Self> {
		if rhs.is_zero() {
			return None;
		}
		let result = (U384::from(self) << Self::FRACTIONAL_BITS) / U384::from(rhs);
		result.checked_as_q64_128()
	}

	/// Performs a saturating multiplication of two `Q64_128` values.
	///
	/// # Parameters
	/// - `self`: The first `Q64_128` value.
	/// - `rhs`: The second `Q64_128` value to multiply with `self`.
	///
	/// # Returns
	/// A `Q64_128` instance:
	/// - The product of the two values if the multiplication does not overflow.
	/// - `Q64_128::MAX` if the multiplication overflows.
	pub fn saturating_mul(self, rhs: Self) -> Self {
		self.checked_mul(rhs).map_or(Q64_128::MAX, |res| res)
	}

	/// Performs a saturating division of two `Q64_128` values with overflow handling.
	///
	/// # Parameters
	/// - `self`: The numerator `Q64_128` value.
	/// - `rhs`: The denominator `Q64_128` value.
	///
	/// # Returns
	/// An `Option<Q64_128>`:
	/// - `Some(Q64_128)` containing the result of the division.
	/// - `None` if the denominator is zero.
	/// - `Some(Q64_128::MAX)` if the division overflows.
	pub fn saturating_checked_div(self, rhs: Self) -> Option<Self> {
		if rhs.is_zero() {
			return None;
		}
		let result = (U384::from(self) << Self::FRACTIONAL_BITS) / U384::from(rhs);
		result.checked_as_q64_128().map_or(Some(Q64_128::MAX), Some)
	}
}


impl Q64_128 {
	/// Calculates the square of the `Q64_128` value.
	///
	/// # Behavior
	/// - Computes the square of the value as a fixed-point number.
	/// - Returns `None` if the result cannot fit within the bounds of `Q64_128`.
	///
	/// # Returns
	/// An `Option<Q64_128>`:
	/// - `Some(Q64_128)` containing the square of the value.
	/// - `None` if the result overflows.
	fn square(self) -> Option<Self> {
		let square_as_u384 = self.square_as_u384();
		if self.is_zero() || self.is_one() {
			return Some(self);
		}
		square_as_u384.q128_256_to_q64_128_round()
	}

	/// Computes the square of the `Q64_128` value as a `U384`.
	///
	/// # Behavior
	/// - Multiplies the value by itself and returns the result as a `U384`.
	///
	/// # Returns
	/// A `U384` representing the square of the value.
	fn square_as_u384(self) -> U384 {
		if self.is_zero() {
			return U384::zero();
		}
		if self.is_one() {
			return U384::from(1u128);
		}
		U384::from(self) * U384::from(self)
	}

	/// Computes the square of the `Q64_128` value as a `u128`.
	///
	/// # Behavior
	/// - Computes the square and rounds the result to fit within a `u128`.
	///
	/// # Returns
	/// A `u128` representing the square of the value.
	pub fn square_as_u128(self) -> u128 {
		let square_as_u384 = self.square_as_u384();
		if square_as_u384 == U384::one() || self.is_zero() {
			return square_as_u384.as_u128();
		}
		square_as_u384.q128_256_to_u128_round()
	}

	/// Computes the square of the `Q64_128` value as a `u64` if possible.
	///
	/// # Behavior
	/// - Computes the square and rounds the result to fit within a `u64`.
	/// - Returns `None` if the result overflows a `u64`.
	///
	/// # Returns
	/// An `Option<u64>`:
	/// - `Some(u64)` containing the rounded square of the value.
	/// - `None` if the result overflows a `u64`.
	pub fn checked_square_as_u64(self) -> Option<u64> {
		let square_as_u384 = self.square_as_u384();
		if square_as_u384 == U384::one() || self.is_zero() {
			return Some(square_as_u384.as_u64());
		}
		square_as_u384.checked_q128_256_to_u64_round()
	}

	/// Computes the square of the `Q64_128` value as a `u64`.
	///
	/// # Behavior
	/// - Computes the square and rounds the result to fit within a `u64`.
	/// - Panics if the result overflows a `u64`.
	///
	/// # Returns
	/// A `u64` representing the square of the value.
	pub fn square_as_u64(self) -> u64 {
		self.checked_square_as_u64().unwrap()
	}

	/// Computes the square root of a `u128` value and scales it to `Q64_128`.
	///
	/// # Parameters
	/// - `value`: A `u128` value for which the square root is to be computed.
	///
	/// # Returns
	/// A `Q64_128` instance representing the square root of the input value.
	pub fn sqrt_from_u128(value: u128) -> Self {
		let scaled_value = U384::from(value) << (2 * Self::FRACTIONAL_BITS);
		Q64_128::from(scaled_value.integer_sqrt())
	}

	/// Computes the square root of the `Q64_128` value.
	///
	/// # Behavior
	/// - Computes the square root of the value as a fixed-point number.
	///
	/// # Returns
	/// A `Q64_128` instance representing the square root of the value.
	pub fn sqrt(self) -> Self {
		let scaled_value = U384::from(self) << Self::FRACTIONAL_BITS;
		Q64_128::from(scaled_value.integer_sqrt())
	}

	/// Computes the square root of the division of two `Q64_128` values.
	///
	/// # Parameters
	/// - `q1`: The numerator `Q64_128` value.
	/// - `q2`: The denominator `Q64_128` value.
	///
	/// # Behavior
	/// - Computes the division of `q1` by `q2` and then takes the square root of the result.
	/// - Returns `None` if `q2` is zero.
	/// - If `q1` equals `q2`, the result is `Q64_128::ONE`.
	///
	/// # Returns
	/// An `Option<Q64_128>`:
	/// - `Some(Q64_128)` containing the square root of the division result.
	/// - `None` if the division is invalid (e.g., division by zero).
	pub fn checked_div_sqrt(q1: Self, q2: Self) -> Option<Self> {
		if q2.is_zero() {
			return None;
		} else if q1 == q2 {
			return Some(Q64_128::ONE);
		}
		let division_result = U384::from(q1.checked_div(q2)?);
		let result = Q64_128::from((division_result << Self::FRACTIONAL_BITS).integer_sqrt());
		Some(result)
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	/// Unit tests for the `Q64_128`.
	mod unit_tests{
		use super::*;
		mod constructors_and_conversions {
			use super::*;

			/// Verifies the creation of a new `Q64_128` instance using `new` and checks its raw value.
			#[test]
			fn test_new() {
				let initial_value = U192::from(12345);
				let q64_128_instance = Q64_128::new(initial_value);
				assert_eq!(
					q64_128_instance.raw_value(),
					initial_value,
					"The raw value of the created `Q64_128` instance should match the input value."
				);
			}

			/// Tests the conversion from `u64` to `Q64_128`.
			#[test]
			fn test_from_u64() {
				let value_u64 = 10;
				let q64_128_instance = Q64_128::from_u64(value_u64);
				let expected_raw_value = U192::from(value_u64) << Q64_128::FRACTIONAL_BITS;
				assert_eq!(
					q64_128_instance.raw_value(),
					expected_raw_value,
					"The raw value of `Q64_128` after conversion from `u64` should be correctly shifted."
				);
			}

			/// Tests the conversion from `f64` to `Q64_128`.
			#[test]
			fn test_from_f64() {
				let value_f64 = 42.75;
				let q64_128_instance = Q64_128::from_f64(value_f64).unwrap();
				let expected_raw_value = (U192::from(42u128) << Q64_128::FRACTIONAL_BITS)
					| U192::from((0.75 * Q64_128::FRACTIONAL_SCALE) as u128);
				assert_eq!(
					q64_128_instance.raw_value(),
					expected_raw_value,
					"The raw value of `Q64_128` after conversion from `f64` should be accurate."
				);
			}
			/// Validates that invalid `f64` values return `None` during conversion to `Q64_128`.
			#[test]
			fn test_invalid_from_f64() {
				let negative_value = Q64_128::from_f64(-0.01);
				let out_of_range_value = Q64_128::from_f64(Q64_128::FRACTIONAL_SCALE);
				assert!(
					negative_value.is_none() && out_of_range_value.is_none(),
					"Conversion from invalid `f64` values should return `None`."
				);
			}

			/// Tests the conversion of `Q64_128` to `f64`.
			#[test]
			fn test_to_f64() {
				let value_f64: f64 = Q64_128::from_f64(42.75).unwrap().into();
				assert!(
					(value_f64 - 42.75).abs() < 1e-12,
					"The `f64` value converted from `Q64_128` should closely match the original value."
				);
			}

			/// Verifies the conversion of `Q64_128` back to `u64`.
			#[test]
			fn test_to_u64() {
				let value_u64 = 42;
				let q64_128_instance = Q64_128::from_u64(value_u64);
				assert_eq!(
					q64_128_instance.as_u64(),
					value_u64,
					"The `u64` value obtained from `Q64_128` should match the original input."
				);
			}

			/// Tests type conversions between `Q64_128` and `u64`, and validates accuracy.
			#[test]
			fn test_type_conversion() {
				let q64_128_instance = Q64_128::from_f64(42.75).unwrap();
				let as_u64 = q64_128_instance.as_u64();
				let back_to_f64: f64 = Q64_128::from_u64(as_u64).into();
				assert!(
					(back_to_f64 - 42.0).abs() < 1e-12,
					"Type conversions between `Q64_128` and `u64` should maintain accuracy."
				);
			}

			/// Checks splitting a `Q64_128` instance into integer and fractional parts.
			#[test]
			fn test_split() {
				let q64_128_instance = Q64_128::from_f64(42.75).unwrap();
				let (integer_part, fractional_part) = q64_128_instance.split();
				assert_eq!(
					integer_part, 42,
					"The integer part of the split `Q64_128` should match the expected value."
				);
				let expected_fractional = (0.75 * Q64_128::FRACTIONAL_SCALE) as u128;
				assert_eq!(
					fractional_part, expected_fractional,
					"The fractional part of the split `Q64_128` should match the expected value."
				);
			}

		}
		mod arithmetic_operations {
			use super::*;

			/// Tests addition of two `Q64_128` instances.
			#[test]
			fn test_add() {
				let value1 = Q64_128::from_u64(10);
				let value2 = Q64_128::from_u64(20);
				assert_eq!(
					(value1 + value2).as_u64(),
					30,
					"Addition of `Q64_128` instances failed: expected 30."
				);
			}

			/// Tests subtraction of two `Q64_128` instances.
			#[test]
			fn test_sub() {
				let value1 = Q64_128::from_u64(50);
				let value2 = Q64_128::from_u64(20);
				assert_eq!(
					(value1 - value2).as_u64(),
					30,
					"Subtraction of `Q64_128` instances failed: expected 30."
				);
			}

			/// Tests multiplication of two `Q64_128` instances.
			#[test]
			fn test_mul() {
				let value1 = Q64_128::from_u64(3);
				let value2 = Q64_128::from_u64(4);
				assert_eq!(
					(value1 * value2).as_u64(),
					12,
					"Multiplication of `Q64_128` instances failed: expected 12."
				);
			}

			/// Tests division of two `Q64_128` instances.
			#[test]
			fn test_div() {
				let value1 = Q64_128::from_u64(10);
				let value2 = Q64_128::from_u64(2);
				assert_eq!(
					(value1 / value2).as_u64(),
					5,
					"Division of `Q64_128` instances failed: expected 5."
				);
			}

			/// Tests absolute difference calculation between two `Q64_128` instances.
			#[test]
			fn test_abs_diff() {
				let value1 = Q64_128::from_u64(50);
				let value2 = Q64_128::from_u64(20);
				assert_eq!(
					value1.abs_diff(value2).as_u64(),
					30,
					"Absolute difference calculation failed: expected 30."
				);
			}

			/// Tests fractional multiplication of two `Q64_128` instances.
			#[test]
			fn test_fractional_multiplication() {
				let value1 = Q64_128::from_f64(2.5).unwrap();
				let value2 = Q64_128::from_f64(4.2).unwrap();
				let result: f64 = (value1 * value2).into();

				assert!(
					(result - 10.5).abs() < 1e-12,
					"Fractional multiplication failed: expected approximately 10.5."
				);
			}
		}

		mod checked_operations {
			use super::*;

			/// Tests checked addition of two `Q64_128` instances with overflow handling.
			#[test]
			fn test_checked_add() {
				let max_value = Q64_128::new(U192::MAX >> 1);
				let overflow_value = Q64_128::new(U192::MAX);
				let value1 = Q64_128::from_u64(1);
				let value2 = Q64_128::from_u64(100);

				assert!(
					max_value.checked_add(overflow_value).is_none(),
					"Checked addition should return `None` for overflow."
				);
				assert_eq!(
					value1.checked_add(value2).unwrap().as_u64(),
					101,
					"Checked addition failed: expected 101."
				);
			}

			/// Tests checked subtraction of two `Q64_128` instances with underflow handling.
			#[test]
			fn test_checked_sub() {
				let value1 = Q64_128::from_u64(50);
				let value2 = Q64_128::from_u64(100);

				assert!(
					value1.checked_sub(value2).is_none(),
					"Checked subtraction should return `None` for underflow."
				);
				assert_eq!(
					value2.checked_sub(value1).unwrap().as_u64(),
					50,
					"Checked subtraction failed: expected 50."
				);
			}

			/// Tests checked multiplication of two `Q64_128` instances with overflow handling.
			#[test]
			fn test_checked_mul() {
				let max_value = Q64_128::new(U192::MAX);
				let multiplier1 = Q64_128::from_u64(2);
				let multiplier2 = Q64_128::from_u64(31);

				assert!(
					max_value.checked_mul(multiplier1).is_none(),
					"Checked multiplication should return `None` for overflow."
				);
				assert_eq!(
					multiplier1.checked_mul(multiplier2).unwrap().as_u64(),
					62,
					"Checked multiplication failed: expected 62."
				);
			}

			/// Tests checked division of two `Q64_128` instances with zero division handling.
			#[test]
			fn test_checked_div() {
				let numerator = Q64_128::from_u64(100);
				let zero_denominator = Q64_128::from_u64(0);
				let divisor = Q64_128::from_u64(150);
				let result: f64 = divisor.checked_div(numerator).unwrap().into();

				assert!(
					numerator.checked_div(zero_denominator).is_none(),
					"Checked division should return `None` for division by zero."
				);
				assert!(
					(result - 1.5).abs() < 1e-12,
					"Checked division failed: expected approximately 1.5."
				);
			}

			/// Tests checked squaring of a `Q64_128` instance with overflow handling.
			#[test]
			fn test_checked_square_overflow() {
				let large_value = Q64_128::new(U192::MAX >> 1);

				assert!(
					large_value.checked_square_as_u64().is_none(),
					"Checked squaring should return `None` for overflow."
				);
			}
		}

		mod edge_cases {
			use super::*;

			/// Tests the behavior of `Q64_128` with extreme values.
			#[test]
			fn test_extreme_values() {
				// Maximum possible value.
				let max = Q64_128::MAX;
				assert_eq!(
					max.raw_value(),
					U192::MAX,
					"The raw value of `Q64_128::MAX` should equal `U192::MAX`."
				);

				// Minimum representable fractional value.
				let min_fraction = Q64_128::from_f64(1.0 / Q64_128::FRACTIONAL_SCALE).unwrap();
				assert_eq!(
					min_fraction.raw_value(),
					U192::one(),
					"The raw value of the smallest representable fraction should be `1`."
				);

				// Value near the maximum.
				let near_max = Q64_128::new(U192::MAX);
				assert_eq!(
					near_max.as_u64(),
					u64::MAX,
					"Converting a near-maximum `Q64_128` instance to `u64` should result in `u64::MAX`."
				);
			}

			/// Verifies that the `is_zero` method correctly identifies zero values.
			#[test]
			fn test_is_zero() {
				let zero = Q64_128::from_u64(0);
				assert!(
					zero.is_zero(),
					"`is_zero` should return `true` for a zero value."
				);

				let non_zero = Q64_128::from_u64(1);
				assert!(
					!non_zero.is_zero(),
					"`is_zero` should return `false` for a non-zero value."
				);
			}

			/// Verifies that the `is_one` method correctly identifies unit values.
			#[test]
			fn test_is_one() {
				let one = Q64_128::from_u64(1);
				assert!(
					one.is_one(),
					"`is_one` should return `true` for a value of one."
				);

				let not_one = Q64_128::from_u64(0);
				assert!(
					!not_one.is_one(),
					"`is_one` should return `false` for a value other than one."
				);
			}

			/// Tests the precision of converting between `f64` and `Q64_128`.
			#[test]
			fn test_precision() {
				let original_value = 2_412_345.000_678_902_5;
				let q = Q64_128::from_f64(original_value).unwrap();
				let back_to_f64: f64 = q.into();

				assert!(
					(back_to_f64 - original_value).abs() < 1e-12,
					"Precision loss detected during `f64` to `Q64_128` conversion and back."
				);
			}
		}

		mod square_and_sqrt {
			use super::*;

			/// Tests square root calculation from a `u128` value.
			#[test]
			fn test_sqrt_from_u128() {
				let integer_sqrt = Q64_128::sqrt_from_u128(256);
				assert_eq!(
					integer_sqrt.as_u64(),
					16,
					"Square root of 256 should be 16."
				);

				let fractional_sqrt: f64 = Q64_128::sqrt_from_u128(10).into();
				assert!(
					(fractional_sqrt - 3.16227766).abs() < 1e-9,
					"Square root approximation of 10 failed."
				);
			}

			/// Tests various square operations for integer numbers.
			#[test]
			fn test_square_operations() {
				let value = Q64_128::from_u64(3);
				assert_eq!(
					value.square_as_u128(),
					9,
					"Square operation for `Q64_128` failed: expected 9."
				);
				assert_eq!(
					value.checked_square_as_u64(),
					Some(9),
					"Checked square operation failed: expected 9."
				);
				assert_eq!(
					value.square_as_u64(),
					9,
					"Square operation (as `u64`) failed: expected 9."
				);
			}

			/// Tests squaring a fractional number.
			#[test]
			fn test_square_fractional_number() {
				let fractional_value = Q64_128::from_f64(1.5).unwrap();
				let square = fractional_value.square_as_u128();
				let expected = 2u128;

				assert_eq!(
					square, expected,
					"Square of fractional number failed: expected 2."
				);
			}

			/// Tests squaring a small fractional number close to zero.
			#[test]
			fn test_square_small_fraction() {
				let small_fraction = Q64_128::from_f64(0.25).unwrap();
				let square = small_fraction.square_as_u128();
				let expected = 0u128;

				assert_eq!(
					square, expected,
					"Square of small fractional number failed: expected 0."
				);
			}

			/// Tests squaring a large fractional number.
			#[test]
			fn test_square_large_fractional_number() {
				let large_fraction = Q64_128::from_f64(12345.6789).unwrap();
				let square = large_fraction.square_as_u128();

				let expected: f64 = 12345.6789 * 12345.6789;
				assert_eq!(
					square,
					expected.floor() as u128,
					"Square of large fractional number failed."
				);
			}

			/// Tests squaring a fractional number near one.
			#[test]
			fn test_square_near_one_fraction() {
				let near_one = Q64_128::from_f64(0.9999).unwrap();
				let square = near_one.square_as_u128();

				let expected = 0;

				assert_eq!(
					square, expected,
					"Square of a fraction close to one failed: expected 0."
				);
			}
		}

		mod panic_tests {
			use super::*;
			
			/// Tests that division by zero results in a panic.
			#[test]
			#[should_panic(expected = "Division by zero!")]
			fn test_div_panic() {
				let numerator = Q64_128::from_u64(10);
				let denominator = Q64_128::from_u64(0);
				let _ = numerator / denominator;
			}
		}
	}

	/// Fuzz tests for the `Q64_128`.
	mod fuzz_tests{
		use super::*;
		use proptest::prelude::*;
		/// Strategy for generating arbitrary `f64` values.
		fn arbitrary_f64() -> impl Strategy<Value = f64> {
			prop_oneof![
            	0.0..18446744073709551616.0, // Range of valid `f64` values
            	Just(0.00000001),           // Very small fractional value
            	Just(1.0),                  // Unit value
            	Just(18446744073709551616.0 * 0.99), // Large valid value near the max
        	]
		}

		/// Strategy for generating invalid `f64` values.
		fn invalid_arbitrary_f64() -> impl Strategy<Value = f64> {
			prop_oneof![
            	f64::MIN..0.0,              // Negative range
            	18446744073709551616.0..f64::MAX, // Overflow range
            	Just(f64::NEG_INFINITY),    // Negative infinity
            	Just(f64::INFINITY),        // Positive infinity
            	Just(f64::NAN),             // Not-a-Number
        	]
		}

		/// Strategy for generating arbitrary `u128` values.
		fn arbitrary_u128() -> impl Strategy<Value = u128> {
			prop_oneof![
            	0..=u128::MAX,              // Full valid range
            	Just(0),
            	Just(1),
            	Just(2),
            	Just(u128::MAX),
            	Just(u128::MAX / 2),
            	Just(u128::MAX - 1),
            	Just(1024), Just(4095), Just(8191),
            	Just(12345), Just(54321), Just(99999),
            	Just(9756157671691129856 << 64),
            	Just(18446744073709551615 << 64),
        	]
		}

		/// Strategy for generating arbitrary `u64` values.
		fn arbitrary_u64() -> impl Strategy<Value = u64> {
			prop_oneof![
            	Just(0),
            	Just(1),
            	Just(2),
            	Just(u64::MAX),
            	Just(u64::MAX / 2),
            	Just(u64::MAX - 1),
           		Just(1024), Just(4095), Just(8191),
            	Just(12345), Just(54321), Just(99999),
        	]
		}
		proptest! {
            #![proptest_config(ProptestConfig::with_cases(10000))]

        	/// Tests round-trip type conversion between `f64`, `u64`, and `Q64_128`.
        	#[test]
        	fn test_types_conversion_round_trip(value_f64 in arbitrary_f64()) {
        	    let value_u64 = value_f64.floor() as u64;
        	    let value_round_f64 = value_u64 as f64;
			
        	    let q_f64 = Q64_128::from_f64(value_f64).unwrap();
        	    let q_u64 = Q64_128::from_u64(value_u64);
			
        	    let converted_back_f64: f64 = q_f64.into();
        	    let converted_back_u64 = q_f64.as_u64();
			
        	    let f64_from_u64: f64 = q_u64.into();
			
        	    prop_assert!((value_f64 - converted_back_f64).abs() < 1e-12, "Precision loss detected for value: {}", value_f64);
        	    prop_assert!((value_round_f64 - f64_from_u64).abs() < 1e-12, "Precision loss detected for round f64 from u64: {}, f64_from_u64: {}", value_round_f64, f64_from_u64);
        	    prop_assert_eq!(value_u64, converted_back_u64, "Value mismatch for u64 conversion: original u64: {}, converted_back_u64: {}", value_u64, converted_back_u64);
        	    prop_assert!((value_f64 - value_round_f64).abs() < 1.0, "Fractional loss detected: value_f64: {}, value_round_f64: {}", value_f64, value_round_f64);
        	}
			
        	/// Tests that invalid `f64` values are not convertible to `Q64_128`.
        	#[test]
        	fn test_invalid_f64_conversion(value_f64 in invalid_arbitrary_f64()) {
        	    let q_f64 = Q64_128::from_f64(value_f64);
        	    prop_assert!(q_f64.is_none(), "Invalid value should not be convertible: {}", value_f64);
        	}

            /// Tests the round-trip behavior of square root and squaring operations on `u128` values.
			///
			/// This ensures that taking the square root of a `u128` value and then squaring it
			/// yields a value that is close to the original, within an acceptable tolerance.
			#[test]
			fn test_u128_sqrt_and_square_round_trip(value in arbitrary_u128()) {
			    // Calculate the square root of the input value
			    let sqrt = Q64_128::sqrt_from_u128(value);
			    let sqrt_as_f64: f64 = sqrt.into();
			
			    // Calculate the square of the result from the square root
			    let square_as_u128 = sqrt.square_as_u128();
			    let square_as_u64_option = sqrt.checked_square_as_u64();
			
			    // Define tolerance based on the leading zeros of the input value
			    let tolerance = 2 >> (value.leading_zeros().max(3) - 3);
			
			    // Check if the square of the square root is within the tolerance range of the original value
			    prop_assert!(
			        square_as_u128.abs_diff(value) <= tolerance,
			        "Square mismatch: expected value {}, got square {} (sqrt as f64: {})",
			        value,
			        square_as_u128,
			        sqrt_as_f64
			    );
			
			    if value >> 64 != 0 {
			        // If the value cannot fit in a u64, ensure checked_square_as_u64 returns None
			        prop_assert!(
			            square_as_u64_option.is_none(),
			            "Square should not be convertible to u64 for large values: {}",
			            value
			        );
			    } else {
			        // If the value can fit in a u64, validate the conversion
			        let value_as_u64 = value as u64;
			        let square_as_u64 = square_as_u64_option.unwrap();
			        prop_assert!(
			            square_as_u64.abs_diff(value_as_u64) <= tolerance as u64,
			            "Square mismatch for u64: expected value {}, got square {} (sqrt as f64: {})",
			            value_as_u64,
			            square_as_u64,
			            sqrt_as_f64
			        );
			    }
			}
			
            /// Tests the round-trip behavior of square root and squaring operations on `Q64_128` values.
			///
			/// Ensures that taking the square root of a `Q64_128` value and then squaring it
			/// yields a result that closely matches the original value.
			#[test]
			fn test_q64_128_sqrt_and_square_round_trip(value1 in arbitrary_u128(), value2 in arbitrary_u64()) {
			    // Combine inputs into a U192 value
			    let value = U192::from(value1) * U192::from(value2);
			    let q64_128 = Q64_128::new(value);
			
			    // Calculate square root and square
			    let sqrt = q64_128.sqrt();
			    let square = sqrt.square().unwrap();
			
			    // Calculate differences
			    let diff = square.abs_diff(q64_128);
			    let diff_as_f64: f64 = diff.into();
			
			    // Ensure the square of the square root is within tolerance
			    prop_assert!(
			        diff_as_f64 <= 1e-28,
			        "Square mismatch: expected value {}, got square {} (sqrt as f64: {})",
			        q64_128.raw_value(),
			        square.raw_value(),
			        f64::from(sqrt)
			    );
			}
			
			/// Tests the combined behavior of division, square root, and squaring on `Q64_128` values.
			///
			/// Ensures that dividing two `Q64_128` values, taking the square root of the result,
			/// and then squaring the square root produces a value close to the original division result.
			#[test]
			fn test_q64_128_checked_div_sqrt_and_square_round_trip(
			    value1 in arbitrary_u128(),
			    value2 in arbitrary_u64(),
			    value3 in arbitrary_u128(),
			    value4 in arbitrary_u64()
			) {
			    // Create Q64_128 values from inputs
			    let q64_128_value1 = Q64_128::new(U192::from(value1) * U192::from(value2));
			    let q64_128_value2 = Q64_128::new(U192::from(value3) * U192::from(value4));
			
			    // Perform checked division and square root
			    let div_sqrt_result = Q64_128::checked_div_sqrt(q64_128_value1, q64_128_value2);
			
			    if let Some(initial_square) = q64_128_value1.checked_div(q64_128_value2) {
			        let div_sqrt = div_sqrt_result.unwrap();
			        let div_sqrt_as_f64: f64 = div_sqrt.into();
			
			        // Calculate the square of the square root
			        let square = div_sqrt.square().unwrap();
			
			        // Calculate differences
			        let diff = square.abs_diff(initial_square);
			        let diff_as_f64: f64 = diff.into();
			
			        // Ensure the difference is within tolerance
			        prop_assert!(
			            diff_as_f64 <= 1e-28 && diff.raw_value() <= U192::from(1u64 << 33),
			            "Square mismatch: expected {}, got {}, sqrt: {}, initial square: {}",
			            square.raw_value(),
			            initial_square.raw_value(),
			            div_sqrt_as_f64,
			            f64::from(initial_square)
			        );
			    } else {
			        // Ensure checked_div_sqrt returns None when division fails
			        prop_assert!(
			            div_sqrt_result.is_none(),
			            "checked_div_sqrt should return None for invalid division values."
			        );
			    }
			}

            /// Tests the absolute difference calculation between two `Q64_128` values.
			///
			/// Ensures that the `abs_diff` method produces a correct result that matches the
			/// absolute difference of the raw underlying values.
			#[test]
			fn test_abs_diff(value1_1 in arbitrary_u128(), value1_2 in arbitrary_u64(), value2_1 in arbitrary_u128(), value2_2 in arbitrary_u64()) {
			    let value1 = U192::from(value1_1) * U192::from(value1_2);
			    let value2 = U192::from(value2_1) * U192::from(value2_2);
			    let q1 = Q64_128::new(value1);
			    let q2 = Q64_128::new(value2);
			
			    let abs_diff = q1.abs_diff(q2);
			    let expected_diff = value1.abs_diff(value2);
			
			    prop_assert_eq!(
			        abs_diff.raw_value(),
			        expected_diff,
			        "Invalid abs_diff result: got {}, expected {} for values {} and {}",
			        abs_diff.raw_value(),
			        expected_diff,
			        value1,
			        value2
			    );
			}
			
			/// Tests the `checked_add` operation for `Q64_128` values.
			///
			/// Verifies that the `checked_add` method correctly handles overflow scenarios
			/// and produces valid results when addition is possible.
			#[test]
			fn test_checked_add(value1_1 in arbitrary_u128(), value1_2 in arbitrary_u64(), value2_1 in arbitrary_u128(), value2_2 in arbitrary_u64()) {
			    let value1 = U192::from(value1_1) * U192::from(value1_2);
			    let value2 = U192::from(value2_1) * U192::from(value2_2);
			    let q1 = Q64_128::new(value1);
			    let q2 = Q64_128::new(value2);
			
			    let result = q1.checked_add(q2);
			
			    if let Some(sum) = result {
			        let expected_sum = value1.checked_add(value2).unwrap();
			        prop_assert_eq!(
			            sum.raw_value(),
			            expected_sum,
			            "Invalid checked_add result: got {}, expected {} for values {} and {}",
			            sum.raw_value(),
			            expected_sum,
			            value1,
			            value2
			        );
			    } else {
			        prop_assert!(
			            value1.checked_add(value2).is_none(),
			            "checked_add should return None for overflowing values {} and {}",
			            value1,
			            value2
			        );
			    }
			}
			
			/// Tests the `checked_sub` operation for `Q64_128` values.
			///
			/// Ensures that `checked_sub` handles underflow scenarios correctly and produces
			/// valid results when subtraction is possible.
			#[test]
			fn test_checked_sub(value1_1 in arbitrary_u128(), value1_2 in arbitrary_u64(), value2_1 in arbitrary_u128(), value2_2 in arbitrary_u64()) {
			    let value1 = U192::from(value1_1) * U192::from(value1_2);
			    let value2 = U192::from(value2_1) * U192::from(value2_2);
			    let q1 = Q64_128::new(value1);
			    let q2 = Q64_128::new(value2);
			
			    let result = q1.checked_sub(q2);
			
			    if let Some(diff) = result {
			        let expected_diff = value1.checked_sub(value2).unwrap();
			        prop_assert_eq!(
			            diff.raw_value(),
			            expected_diff,
			            "Invalid checked_sub result: got {}, expected {} for values {} and {}",
			            diff.raw_value(),
			            expected_diff,
			            value1,
			            value2
			        );
			    } else {
			        prop_assert!(
			            value1.checked_sub(value2).is_none(),
			            "checked_sub should return None for underflowing values {} and {}",
			            value1,
			            value2
			        );
			    }
			}
			
			/// Tests the `checked_mul` operation for `Q64_128` values.
			///
			/// Ensures that `checked_mul` handles overflow scenarios correctly and produces
			/// valid results when multiplication is possible.
			#[test]
			fn test_checked_mul(value1_1 in arbitrary_u128(), value1_2 in arbitrary_u64(), value2_1 in arbitrary_u128(), value2_2 in arbitrary_u64()) {
			    let value1 = U192::from(value1_1) * U192::from(value1_2);
			    let value2 = U192::from(value2_1) * U192::from(value2_2);
			    let q1 = Q64_128::new(value1);
			    let q2 = Q64_128::new(value2);
			
			    let result = q1.checked_mul(q2);
			
			    if let Some(product) = result {
			        let expected_product: Q64_128 = ((value1.into_u384() * value2.into_u384()) >> Q64_128::FRACTIONAL_BITS).into();
			        prop_assert_eq!(
			            product,
			            expected_product,
			            "Invalid checked_mul result: got {}, expected {} for values {} and {}",
			            product.raw_value(),
			            expected_product.raw_value(),
			            value1,
			            value2
			        );
			    } else {
			        let expected_product: U384 = (value1.into_u384() * value2.into_u384()) >> Q64_128::FRACTIONAL_BITS;
			        prop_assert!(
			            expected_product.leading_zeros() < 192,
			            "checked_mul should return None for overflowing values {} and {}",
			            value1,
			            value2
			        );
			    }
			}
			
			/// Tests the `checked_div` operation for `Q64_128` values.
			///
			/// Verifies that `checked_div` handles division by zero and overflow scenarios correctly
			/// while producing valid results for valid divisions.
			#[test]
			fn test_checked_div(value1_1 in arbitrary_u128(), value1_2 in arbitrary_u64(), value2_1 in arbitrary_u128(), value2_2 in arbitrary_u64()) {
			    let value1 = U192::from(value1_1) * U192::from(value1_2);
			    let value2 = U192::from(value2_1) * U192::from(value2_2);
			    let q1 = Q64_128::new(value1);
			    let q2 = Q64_128::new(value2);
			
			    let result = q1.checked_div(q2);
			
			    if value2 == U192::zero() {
			        prop_assert!(result.is_none(), "Division by zero should return None");
			    } else if let Some(quotient) = result {
			        let expected_quotient: Q64_128 = ((value1.into_u384() << Q64_128::FRACTIONAL_BITS) / value2.into_u384()).into();
			        prop_assert_eq!(
			            quotient,
			            expected_quotient,
			            "Invalid checked_div result: got {}, expected {} for values {} and {}",
			            quotient.raw_value(),
			            expected_quotient.raw_value(),
			            value1,
			            value2
			        );
			    } else {
			        let expected_quotient = (value1.into_u384() << Q64_128::FRACTIONAL_BITS) / value2.into_u384();
			        prop_assert!(
			            expected_quotient.leading_zeros() < 192,
			            "checked_div should return None for overflowing values {} and {}",
			            value1,
			            value2
			        );
			    }
			}

        }
	}
}