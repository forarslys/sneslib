use std::fmt;

pub mod error;

use error::*;
pub type CartridgeResult = Result<Cartridge, CartridgeError>;

bitflags::bitflags! {
	/// Flags for a ROM test on loading.
	pub struct TestFlags: u32 {
		/// Tests if the ROM size is a multiple of `0x8000`
		const SIZE = 1 << 0;
		/// Tests the ROM checksum with values at the offset `7FDCh-7FDFh`
		const CHECKSUM_LO = 1 << 1;
		/// Tests the ROM checksum with values at the offset `FFDCh-FFDFh`
		const CHECKSUM_HI = 1 << 2;
		/// Tests the ROM Speed and Map Mode at the offset `7FD5h`
		const ROM_SPEED_AND_MAP_LO = 1 << 3;
		/// Tests the ROM Speed and Map Mode at the offset `FFD5h`
		const ROM_SPEED_AND_MAP_HI = 1 << 4;
		/// Tests the chipset at the offset `7FD6h`
		const CHIPSET_LO = 1 << 5;
		/// Tests the chipset at the offset `FFD6h`
		const CHIPSET_HI = 1 << 6;
		/// Tests the country code at the offset `7FD9h`
		const COUNTRY_LO = 1 << 7;
		/// Tests the country code at the offset `FFD9h`
		const COUNTRY_HI = 1 << 8;
	}
}

impl Default for TestFlags {
	fn default() -> Self {
		TestFlags::all()
	}
}

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
		T: AsRef<[u8]>,
	{
		let passed = Self::rom_test(rom.as_ref());

		if !test_flags.contains(passed) {
			return Err(NotProbableCartridgeError::new(passed, test_flags).into());
		}

		let rom = rom.as_ref().into();
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

		macro_rules! test {
			($test:ident $offset:expr, $loflag:ident, $hiflag:ident) => {
				if $test($offset) { TestFlags::$loflag } else { TestFlags::empty() } |
				if $test($offset | 0x8000) { TestFlags::$hiflag } else { TestFlags::empty() }
			}
		}

		// checksum
		let sum = rom.iter().fold(0u16, |r, &b| r.wrapping_add(b as u16));
		let test_checksum = |offset| {
			let compl = read_u16(offset);
			let checksum = read_u16(offset + 2);
			Some(sum ^ 0xFFFF) == compl && Some(sum) == checksum
		};
		let flag_checksum = test!(test_checksum 0x7FDC, CHECKSUM_LO, CHECKSUM_HI);

		// ROM makeup
		let test_rom_makeup = |offset| {
			rom.get(offset).map_or(false, |b| {
				b & 0xE0 == 0x20 && matches!(b & 0xF, 0 | 1 | 2 | 3 | 5 | 0xA)
			})
		};
		let flag_rommakeup =
			test!(test_rom_makeup 0x7FD5, ROM_SPEED_AND_MAP_LO, ROM_SPEED_AND_MAP_HI);

		// chipset
		let test_chipset = |offset| {
			rom.get(offset).map_or(false, |&b| {
				matches!(b,
					0x00..=0x05 | 0x13..=0x15 | 0x1A | 0x25 | 0x32 | 0x34 | 0x35 |
					0x43 | 0x45 | 0x55 | 0xE3 | 0xE5 | 0xF3 | 0xF5 | 0xF6 | 0xF9)
			})
		};
		let flag_chipset = test!(test_chipset 0x7FD6, CHIPSET_LO, CHIPSET_HI);

		// country
		let test_country = |offset| rom.get(offset).map_or(false, |&b| matches!(b, 0..=0x14));
		let flag_country = test!(test_country 0x7FD9, COUNTRY_LO, COUNTRY_HI);

		flag_size | flag_checksum | flag_rommakeup | flag_chipset | flag_country
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
