use std::{error::Error, fmt, num};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressError {
	TryFromIntError,
}

impl From<num::TryFromIntError> for AddressError {
	fn from(_: num::TryFromIntError) -> Self {
		Self::TryFromIntError
	}
}

impl From<std::convert::Infallible> for AddressError {
	fn from(_: std::convert::Infallible) -> Self {
		unreachable!()
	}
}

impl fmt::Display for AddressError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use AddressError::*;
		match self {
			TryFromIntError => "out of range integral type conversion attempted".fmt(f),
		}
	}
}

impl Error for AddressError {}
