use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::ops::{Add, BitAnd, Sub};

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
impl_into![u16, u32, u64, u128; Address16];
impl_into_usize![Address16; "32", "64"];

impl_from![u8, u16, Address16; Address24];
impl_try_from![i8, i16, i32, i64, i128, u32, u64, u128; Address24];
impl_into![u32, u64, u128; Address24];
impl_into_usize![Address24; "32", "64"];

impl fmt::Display for Address16 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "${:04X}", self.0)
	}
}

impl fmt::Display for Address24 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "${:02X}:{:04X}", self.0 >> 16, self.0 as u16)
	}
}

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
	/// let addr = Address24::new(0x123456);
	/// assert_eq!(addr.low(), 0x56);
	/// ```
	#[inline]
	pub fn low(&self) -> u8 {
		(self.0 & 0xFF) as u8
	}

	/// Returns a middle byte of the address.
	/// ```
	/// # use sneslib::address::*;
	/// let addr = Address24::new(0x123456);
	/// assert_eq!(addr.middle(), 0x34);
	/// ```
	#[inline]
	pub fn middle(&self) -> u8 {
		(self.0 >> 8) as u8
	}

	/// Returns a high byte of the address.
	/// ```
	/// # use sneslib::address::*;
	/// let addr = Address24::new(0x123456);
	/// assert_eq!(addr.high(), 0x12);
	/// ```
	#[inline]
	pub fn high(&self) -> u8 {
		(self.0 >> 16) as u8
	}

	/// Returns a lower 16-bit of the address.
	/// ```
	/// # use sneslib::address::*;
	/// let addr = Address24::new(0x123456);
	/// assert_eq!(addr.get_lower_address16(), Address16::new(0x3456));
	/// ```
	#[inline]
	pub fn get_lower_address16(&self) -> Address16 {
		Address16(self.0 as u16)
	}
}

macro_rules! impl_op {
	($t:ident  $trait:ident:$fn:ident:$internalfn:ident $($mask:expr)?) => {
		impl $trait<Self> for $t {
			type Output = $t;
			#[inline]
			fn $fn(self, rhs: $t) -> $t {
				let ad = self.0.$internalfn(rhs.0);
				$t(ad $(& $mask)?)
			}
		}
	};
}

macro_rules! impl_and {
	($t:ident $prm:ident) => {
		impl BitAnd<$prm> for $t {
			type Output = $t;
			#[inline]
			fn bitand(self, rhs: $prm) -> $t {
				$t(self.0 & rhs)
			}
		}
	};
}

macro_rules! forward_ref_binop {
	(impl $imp:ident:$method:ident for $t:ty, $u:ty) => {
		impl<'a> $imp<$u> for &'a $t {
			type Output = <$t as $imp<$u>>::Output;
			#[inline]
			fn $method(self, other: $u) -> <$t as $imp<$u>>::Output {
				$imp::$method(*self, other)
			}
		}

		impl<'a> $imp<&'a $u> for $t {
			type Output = <$t as $imp<$u>>::Output;
			#[inline]
			fn $method(self, other: &'a $u) -> <$t as $imp<$u>>::Output {
				$imp::$method(self, *other)
			}
		}

		impl<'a, 'b> $imp<&'a $u> for &'b $t {
			type Output = <$t as $imp<$u>>::Output;
			#[inline]
			fn $method(self, other: &'a $u) -> <$t as $imp<$u>>::Output {
				$imp::$method(*self, *other)
			}
		}
	};
}

impl_op!(Address16 Add:add:wrapping_add);
impl_op!(Address16 Sub:sub:wrapping_sub);
impl_and!(Address16 u16);
impl_op!(Address24 Add:add:wrapping_add 0xFFFFFF);
impl_op!(Address24 Sub:sub:wrapping_sub 0xFFFFFF);
impl_and!(Address24 u32);

impl Add<Address16> for Address24 {
	type Output = Self;
	#[inline]
	fn add(self, rhs: Address16) -> Self::Output {
		Self(self.0 & 0xFF0000 | (self.get_lower_address16() + rhs).0 as u32)
	}
}

forward_ref_binop!(impl Add:add for Address16, Address16);
forward_ref_binop!(impl Sub:sub for Address16, Address16);
forward_ref_binop!(impl BitAnd:bitand for Address16, u16);
forward_ref_binop!(impl Add:add for Address24, Address24);
forward_ref_binop!(impl Sub:sub for Address24, Address24);
forward_ref_binop!(impl BitAnd:bitand for Address24, u32);
forward_ref_binop!(impl Add:add for Address24, Address16);

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

	#[test]
	fn ops() {
		let a = Address16::new(0x1234);
		let b = Address16::new(0x4321);
		assert_eq!(a + b, Address16::new(0x5555));
		assert_eq!(&a + b, Address16::new(0x5555));
		assert_eq!(a + &b, Address16::new(0x5555));
		assert_eq!(&a + &b, Address16::new(0x5555));
		assert_eq!(a - b, Address16::new(0xCF13));
		assert_eq!(&a - b, Address16::new(0xCF13));
		assert_eq!(a - &b, Address16::new(0xCF13));
		assert_eq!(&a - &b, Address16::new(0xCF13));
		assert_eq!(a & 0x5555, Address16::new(0x1014));
		assert_eq!(&a & 0x5555, Address16::new(0x1014));
		assert_eq!(a & &0x5555, Address16::new(0x1014));
		assert_eq!(&a & &0x5555, Address16::new(0x1014));

		let a = Address24::new(0x123456);
		let b = Address24::new(0x654321);
		assert_eq!(a + b, Address24::new(0x777777));
		assert_eq!(&a + b, Address24::new(0x777777));
		assert_eq!(a + &b, Address24::new(0x777777));
		assert_eq!(&a + &b, Address24::new(0x777777));
		assert_eq!(a - b, Address24::new(0xACF135));
		assert_eq!(&a - b, Address24::new(0xACF135));
		assert_eq!(a - &b, Address24::new(0xACF135));
		assert_eq!(&a - &b, Address24::new(0xACF135));
		assert_eq!(a & 0x555555, Address24::new(0x101454));
		assert_eq!(&a & 0x555555, Address24::new(0x101454));
		assert_eq!(a & &0x555555, Address24::new(0x101454));
		assert_eq!(&a & &0x555555, Address24::new(0x101454));

		let a = Address24::new(0x7EFF00);
		let b = Address16::new(0x200);
		assert_eq!(a + b, Address24::new(0x7E0100));
		assert_eq!(&a + b, Address24::new(0x7E0100));
		assert_eq!(a + &b, Address24::new(0x7E0100));
		assert_eq!(&a + &b, Address24::new(0x7E0100));
	}
}
