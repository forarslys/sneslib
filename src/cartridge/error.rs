use std::{error::Error, fmt, io};

use super::TestFlags;

#[derive(Debug)]
pub enum CartridgeError {
	Io(std::io::Error),
	NotProbableCartridge(NotProbableCartridgeError),
}

impl From<io::Error> for CartridgeError {
	fn from(e: io::Error) -> Self {
		Self::Io(e)
	}
}
/// Indicates the loaded ROM is probably not a SuperNES cartridge.
#[derive(Debug)]
pub struct NotProbableCartridgeError {
	passed: TestFlags,
	required: TestFlags,
}

impl NotProbableCartridgeError {
	pub fn new(passed: TestFlags, required: TestFlags) -> Self {
		Self { passed, required }
	}
}

impl From<NotProbableCartridgeError> for CartridgeError {
	fn from(e: NotProbableCartridgeError) -> Self {
		Self::NotProbableCartridge(e)
	}
}

impl fmt::Display for NotProbableCartridgeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "The cartridge is probably invalid")
	}
}

impl Error for NotProbableCartridgeError {}

impl fmt::Display for CartridgeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use CartridgeError::*;
		match self {
			Io(e) => e.fmt(f),
			NotProbableCartridge(e) => e.fmt(f),
		}
	}
}

impl std::error::Error for CartridgeError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		use CartridgeError::*;
		match self {
			Io(e) => e.source(),
			NotProbableCartridge(e) => e.source(),
		}
	}
}
