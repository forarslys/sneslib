use std::convert::{TryFrom, TryInto};

pub mod error;

/// 16-bit address type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address16(u16);

/// 24-bit address type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address24(u32);

macro_rules! impl_from {
	($($ty:ty),+; $address:ident) => {
		$(
			impl From<$ty> for $address {
				#[inline]
				fn from(src: $ty) -> Self {
					Self(src.into())
				}
			}
		)+
	};
}

macro_rules! impl_try_from {
	($($ty:ty),+; Address16) => {
		$(
			impl TryFrom<$ty> for Address16 {
				type Error = error::AddressError;
				#[inline]
				fn try_from(src: $ty) -> Result<Self, Self::Error> {
					let ad: u16 = src.try_into()?;
					Ok(Self(ad))
				}
			}
		)+
	};
	($($ty:ty),+; Address24) => {
		$(
			impl TryFrom<$ty> for Address24 {
				type Error = error::AddressError;
				#[inline]
				fn try_from(src: $ty) -> Result<Self, Self::Error> {
					let ad: u32 = src.try_into()?;
					if ad < 1 << 24 {
						Ok(Self(ad))
					} else {
						Err(error::AddressError::TryFromIntError)
					}
				}
			}
		)+
	};
}

macro_rules! impl_into {
	($($ty:ty),+; $address:ident) => {
		$(
			impl Into<$ty> for $address {
				#[inline]
				fn into(self) -> $ty {
					self.0.into()
				}
			}
		)+
	};
}

macro_rules! impl_into_usize {
	($address:ident; $($width:expr),+) => {
		$(
			#[cfg(target_pointer_width = $width)]
			impl Into<usize> for $address {
				#[inline]
				fn into(self) -> usize {
					self.0 as usize
				}
			}
		)+
	}
}

impl_from![u8, u16; Address16];
impl_try_from![i8, i16, i32, i64, i128, u32, u64, u128; Address16];
impl_into![u32, u64, u128; Address16];
impl_into_usize![Address16; "32", "64"];

impl_from![u8, u16, Address16; Address24];
impl_try_from![i8, i16, i32, i64, i128, u32, u64, u128; Address24];
impl_into![u32, u64, u128; Address24];
impl_into_usize![Address24; "32", "64"];

impl Address16 {
	/// Creates a new `Address16` with the given `u16` value.
	/// ```
	/// # use sneslib::address::*;
	/// let addr = Address16::new(0x1234);
	/// ```
	#[inline]
	pub fn new(address: u16) -> Self {
		Address16(address)
	}

	/// Returns a low byte of the address.
	/// ```
	/// # use sneslib::address::*;
	/// let addr = Address16::new(0x1234);
	/// assert_eq!(addr.low(), 0x34);
	/// ```
	#[inline]
	pub fn low(&self) -> u8 {
		(self.0 & 0xFF) as u8
	}

	/// Returns a high byte of the address.
	/// ```
	/// # use sneslib::address::*;
	/// let addr = Address16::new(0x1234);
	/// assert_eq!(addr.high(), 0x12);
	/// ```
	#[inline]
	pub fn high(&self) -> u8 {
		(self.0 >> 8) as u8
	}
}

impl Address24 {
	/// Creates a new `Address24` with the given `u32` value truncating the highest 8-bit.
	/// ```
	/// # use sneslib::address::*;
	/// let addr = Address24::new(0x12345678);
	/// assert_eq!(addr, Address24::new(0x345678));
	/// ```
	#[inline]
	pub fn new(address: u32) -> Self {
		Address24(address & 0xFFFFFF)
	}

	/// Returns a low byte of the address.
	/// ```
	/// # use sneslib::address::*;
	/// # use std::convert::TryFrom;
	/// let addr = Address24::try_from(0x123456u32).unwrap();
	/// assert_eq!(addr.low(), 0x56);
	/// ```
	#[inline]
	pub fn low(&self) -> u8 {
		(self.0 & 0xFF) as u8
	}

	/// Returns a middle byte of the address.
	/// ```
	/// # use sneslib::address::*;
	/// # use std::convert::TryFrom;
	/// let addr = Address24::try_from(0x123456u32).unwrap();
	/// assert_eq!(addr.middle(), 0x34);
	/// ```
	#[inline]
	pub fn middle(&self) -> u8 {
		(self.0 >> 8) as u8
	}

	/// Returns a high byte of the address.
	/// ```
	/// # use sneslib::address::*;
	/// # use std::convert::TryFrom;
	/// let addr = Address24::try_from(0x123456u32).unwrap();
	/// assert_eq!(addr.high(), 0x12);
	/// ```
	#[inline]
	pub fn high(&self) -> u8 {
		(self.0 >> 16) as u8
	}

	/// Returns a lower 16-bit of the address.
	/// ```
	/// # use sneslib::address::*;
	/// # use std::convert::TryFrom;
	/// let addr = Address24::try_from(0x123456u32).unwrap();
	/// assert_eq!(addr.get_lower_address16(), Address16::new(0x3456u16));
	/// ```
	#[inline]
	pub fn get_lower_address16(&self) -> Address16 {
		Address16(self.0 as u16)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test() {
		use error::AddressError::*;

		assert_eq!(Address16::from(0xFFu8), Address16(0xFF));
		assert_eq!(Address16::from(0xFFFFu16), Address16(0xFFFF));
		assert_eq!(Address16::try_from(0xFFFFu32), Ok(Address16(0xFFFF)));
		assert_eq!(Address16::try_from(0x10000u32), Err(TryFromIntError));
		assert_eq!(Address16::try_from(0xFFFFi32), Ok(Address16(0xFFFF)));
		assert_eq!(Address16::try_from(-0xFFFFi32), Err(TryFromIntError));
		assert_eq!(Into::<usize>::into(Address16(0x1234)), 0x1234usize);

		assert_eq!(Address24::from(0xFFu8), Address24(0xFF));
		assert_eq!(Address24::from(0xFFFFu16), Address24(0xFFFF));
		assert_eq!(Address24::try_from(0xFFFFFFu32), Ok(Address24(0xFFFFFF)));
		assert_eq!(Address24::try_from(0x1000000u32), Err(TryFromIntError));
		assert_eq!(Address24::try_from(0xFFFFFFi32), Ok(Address24(0xFFFFFF)));
		assert_eq!(Address24::try_from(-0xFFFFFFi32), Err(TryFromIntError));
		assert_eq!(Into::<usize>::into(Address24(0x123456)), 0x123456usize);
		assert_eq!(Address24::from(Address16(0x1234)), Address24(0x1234));
	}
}
