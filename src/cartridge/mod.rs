use std::fmt;

pub mod error;

use error::*;
pub type CartridgeResult = Result<Cartridge, CartridgeError>;

bitflags::bitflags! {
	/// Flags for a ROM test on loading.
	pub struct TestFlags: u32 {
		/// Tests if the ROM size is a multiple of `0x8000`
		const SIZE = 0x01;
		/// Tests the ROM checksum with values at the offset `7FDCh-7FDFh`
		const CHECKSUM_LO = 0x02;
		/// Tests the ROM checksum with values at the offset `FFDCh-FFDFh`
		const CHECKSUM_HI = 0x04;

		const CHECKSUM_EITHER = Self::CHECKSUM_LO.bits | Self::CHECKSUM_HI.bits;
	}
}

impl Default for TestFlags {
	fn default() -> Self {
		TestFlags::all()
	}
}

#[derive(Clone)]
pub struct Cartridge {
	rom: Vec<u8>,
	passed: TestFlags,
}

impl Cartridge {
	pub fn from_file<P>(path: P, test_flags: TestFlags) -> CartridgeResult
	where
		P: AsRef<std::path::Path>,
	{
		let mut file = std::fs::File::open(path)?;

		use std::io::Read;
		let mut rom = Vec::new();
		file.read_to_end(&mut rom)?;

		Self::new(rom, test_flags)
	}

	pub fn new<T>(rom: T, test_flags: TestFlags) -> CartridgeResult
	where
		T: Into<Vec<u8>>,
	{
		let rom = rom.into();
		let passed = Self::rom_test(&rom);

		if !test_flags.contains(passed) {
			return Err(NotProbableCartridgeError::new(passed, test_flags).into());
		}

		Ok(Cartridge { rom, passed })
	}

	fn rom_test(rom: &[u8]) -> TestFlags {
		let flag_size = if rom.len() % 0x8000 == 0 && rom.len() > 0 {
			TestFlags::SIZE
		} else {
			TestFlags::empty()
		};

		let read_u16 = |offset: usize| {
			rom.get(offset)
				.zip(rom.get(offset + 1))
				.map(|(&l, &h)| (h as u16) << 8 | (l as u16))
		};

		// calculate the data sum
		let sum = rom.iter().fold(0u16, |r, &b| r.wrapping_add(b as u16));

		let test_checksum = |offset| {
			let compl = read_u16(offset);
			let checksum = read_u16(offset + 2);
			Some(sum ^ 0xFFFF) == compl && Some(sum) == checksum
		};
		let flag_checksum_lo = if test_checksum(0x7FDC) {
			TestFlags::CHECKSUM_LO
		} else {
			TestFlags::empty()
		};

		let flag_checksum_hi = if test_checksum(0xFFDC) {
			TestFlags::CHECKSUM_HI
		} else {
			TestFlags::empty()
		};

		flag_size | flag_checksum_lo | flag_checksum_hi
	}
}

impl std::fmt::Debug for Cartridge {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Cartridge")
			.field("rom", &self.rom.len())
			.field("passed", &self.passed)
			.finish()
	}
}
