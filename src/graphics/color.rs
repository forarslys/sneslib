use serde::{Deserialize, Serialize};

/// RGB color type.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RGB(pub u8, pub u8, pub u8);

/// 15-bit SNES color type.
///
/// `0bbbbbgggggrrrrr`
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct SNESColor(pub u16);

impl From<SNESColor> for RGB {
	fn from(color: SNESColor) -> Self {
		let SNESColor(color) = color;
		let r = (color & 0x1F) as u8;
		let g = ((color >> 5) & 0x1F) as u8;
		let b = ((color >> 10) & 0x1F) as u8;
		RGB(r << 3, g << 3, b << 3)
	}
}

impl From<RGB> for SNESColor {
	fn from(color: RGB) -> SNESColor {
		let RGB(r, g, b) = color;
		let r = std::cmp::min(r as u16 + 4, 0xF8) & 0xF8;
		let g = std::cmp::min(g as u16 + 4, 0xF8) & 0xF8;
		let b = std::cmp::min(b as u16 + 4, 0xF8) & 0xF8;
		SNESColor(r >> 3 | g << 2 | b << 7)
	}
}

impl RGB {
	/// Returns red color.
	#[inline]
	pub const fn r(&self) -> u8 {
		self.0
	}

	/// Returns green color.
	#[inline]
	pub const fn g(&self) -> u8 {
		self.1
	}

	/// Returns blue color.
	#[inline]
	pub const fn b(&self) -> u8 {
		self.2
	}
}

impl SNESColor {
	/// Returns 5-bit red color.
	///
	/// `000rrrrr`
	#[inline]
	pub const fn r(&self) -> u8 {
		(self.0 & 0x1F) as u8
	}

	/// Returns 5-bit green color.
	///
	/// `000ggggg`
	#[inline]
	pub const fn g(&self) -> u8 {
		((self.0 & 0x3E0) >> 5) as u8
	}

	/// Returns 5-bit blue color.
	///
	/// `000bbbbb`
	#[inline]
	pub const fn b(&self) -> u8 {
		((self.0 & 0x7C00) >> 10) as u8
	}

	/// Returns 8-bit PC red color, with lower 3 bits being zero.
	///
	/// `rrrrr000`
	#[inline]
	pub const fn r_pc(&self) -> u8 {
		((self.0 & 0x1F) << 3) as u8
	}

	/// Returns 8-bit PC green color, with lower 3 bits being zero.
	///
	/// `ggggg000`
	#[inline]
	pub const fn g_pc(&self) -> u8 {
		((self.0 & 0x3E0) >> (5 - 3)) as u8
	}

	/// Returns 8-bit PC blue color, with lower 3 bits being zero.
	///
	/// `bbbbb000`
	#[inline]
	pub const fn b_pc(&self) -> u8 {
		((self.0 & 0x7C00) >> (10 - 3)) as u8
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test() {
		let mut color = RGB(0x40, 0x80, 0xC0);
		assert_eq!(color.r(), 0x40);
		assert_eq!(color.g(), 0x80);
		assert_eq!(color.b(), 0xC0);
		assert_eq!(color.0, 0x40);
		assert_eq!(color.1, 0x80);
		assert_eq!(color.2, 0xC0);
		assert_eq!(color, RGB(0x40, 0x80, 0xC0));
		color.0 = 0x3F;
		color.1 += 1;
		color.2 -= 1;
		assert_eq!(color, RGB(0x3F, 0x81, 0xBF));

		let color = RGB(0x40, 0x80, 0xC0);
		let color: SNESColor = color.into();
		assert_eq!(color.r(), 0x08);
		assert_eq!(color.g(), 0x10);
		assert_eq!(color.b(), 0x18);
		assert_eq!(color.r_pc(), 0x40);
		assert_eq!(color.g_pc(), 0x80);
		assert_eq!(color.b_pc(), 0xC0);

		let v = vec![RGB(1, 2, 3), RGB(2, 3, 4), RGB(3, 4, 5)];
		let encoded = bincode::serialize(&v).unwrap();
		assert_eq!(
			&encoded[std::mem::size_of::<u64>()..],
			&[1, 2, 3, 2, 3, 4, 3, 4, 5]
		);
		let decoded: Vec<RGB> = bincode::deserialize(encoded.as_slice()).unwrap();
		assert_eq!(decoded, vec![RGB(1, 2, 3), RGB(2, 3, 4), RGB(3, 4, 5)]);
	}

	#[test]
	fn convert() {
		assert_eq!(SNESColor(0x7FFF), RGB(0xF8, 0xF8, 0xF8).into());
		assert_eq!(SNESColor(0x4210), RGB(0x80, 0x80, 0x80).into());
		assert_eq!(SNESColor(0x001F), RGB(0xF8, 0x00, 0x00).into());
		assert_eq!(SNESColor(0x03E0), RGB(0x00, 0xF8, 0x00).into());
		assert_eq!(SNESColor(0x7C00), RGB(0x00, 0x00, 0xF8).into());
		assert_eq!(RGB(0xF8, 0xF8, 0xF8), SNESColor(0x7FFF).into());
		assert_eq!(RGB(0x80, 0x80, 0x80), SNESColor(0x4210).into());
		assert_eq!(RGB(0xF8, 0x00, 0x00), SNESColor(0x001F).into());
		assert_eq!(RGB(0x00, 0xF8, 0x00), SNESColor(0x03E0).into());
		assert_eq!(RGB(0x00, 0x00, 0xF8), SNESColor(0x7C00).into());
	}
}
