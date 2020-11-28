use std::sync::atomic::{self, AtomicU8};

use crate::address::Address24;
use crate::cartridge::{Cartridge, ROMType};

const PAGE_SIZE: usize = 64 * 1024;
const MAP_SIZE: usize = 256 * PAGE_SIZE;

type ReadableMemory = Box<[Option<*const AtomicU8>]>;
type WritableMemory = Box<[Option<*const AtomicU8>]>;
type RAM = Box<[AtomicU8]>;
type ROM = Box<[AtomicU8]>;

pub struct MemoryMap {
	readable: ReadableMemory,
	writable: WritableMemory,
	rom: ROM,
	wram: RAM,
	sram: Option<RAM>,
}

#[derive(Debug, Clone, Copy)]
pub enum MapInfo {
	ROM { src: usize, dst: usize, len: usize },
	WRAM { src: usize, dst: usize, len: usize },
	SRAM { src: usize, dst: usize, len: usize },
}

fn new_ram(n: usize) -> RAM {
	(0..n)
		.map(|_| AtomicU8::default())
		.collect::<Vec<_>>()
		.into_boxed_slice()
}

impl MemoryMap {
	pub fn from_cartridge(cartridge: Cartridge, hint: Option<ROMType>) -> Self {
		let wram = new_ram(2 * PAGE_SIZE);
		let sram = None;
		let rom = cartridge
			.rom
			.iter()
			.map(|&b| AtomicU8::new(b))
			.collect::<Vec<_>>()
			.into_boxed_slice();
		let writable = vec![None; MAP_SIZE].into_boxed_slice();
		let readable = vec![None; MAP_SIZE].into_boxed_slice();

		let mut memory_map = Self {
			readable,
			writable,
			rom,
			wram,
			sram,
		};

		let mut map_info = Vec::new();

		// WRMA
		// $xx:0000-$xx:1FFF
		map_info.extend((0x00..=0x3F).chain(0x80..=0xBF).map(|i| MapInfo::WRAM {
			src: 0,
			dst: i << 16,
			len: 0x2000,
		}));

		// $7E-$7F
		map_info.push(MapInfo::WRAM {
			src: 0,
			dst: 0x7E0000,
			len: memory_map.wram.len(),
		});

		match hint {
			Some(ROMType::LoROM) => {
				assert!(memory_map.rom.len() <= 0x400000);
				// ROM
				map_info.extend(
					(0x00..=0x7D)
						.chain(0x80..=0xFF)
						.filter(|&i| (i & 0x7F) * 0x8000 < memory_map.rom.len())
						.map(|i| MapInfo::ROM {
							src: (i & 0x7F) * 0x8000,
							dst: i << 16 | 0x8000,
							len: 0x8000,
						}),
				);

				if let Some(sram_size) = memory_map.sram.as_ref().map(|sram| sram.len()) {
					// with SRAM
					if sram_size <= 0x8000 {
						// small SRAM
						map_info.extend(
							(0x70..=0x7D)
								.chain(0xF0..=0xFF)
								.flat_map(|i| (i << 16..i << 16 | 0x8000).step_by(sram_size))
								.map(|dst| MapInfo::SRAM {
									src: 0,
									dst,
									len: sram_size,
								}),
						);
					} else {
						// large SRAM
						map_info.extend(
							(0x700000..=0x7DFFFF)
								.chain(0xF00000..=0xFFFFFF)
								.step_by(sram_size)
								.map(|dst| MapInfo::SRAM {
									src: 0,
									dst,
									len: sram_size,
								}),
						);
					}
				} else {
					// without SRAM
					// ROM mirror
					map_info.extend(
						(0x40..=0x7D)
							.chain(0xC0..=0xFF)
							.filter(|&i| (i & 0x7F) * 0x8000 < memory_map.rom.len())
							.map(|i| MapInfo::ROM {
								src: (i & 0x7F) * 0x8000,
								dst: i << 16,
								len: 0x8000,
							}),
					);
				}
			}
			Some(ROMType::HiROM) => {
				assert!(memory_map.sram.as_ref().map(|sram| sram.len()).unwrap_or(0) <= 0x2000);
				// ROM
				map_info.extend(
					(0x00..=0x3F)
						.chain(0x80..=0xBF)
						.filter(|&i| (i & 0x3F) << 16 | 0x8000 < memory_map.rom.len())
						.map(|i| MapInfo::ROM {
							src: (i & 0x3F) << 16 | 0x8000,
							dst: i << 16 | 0x8000,
							len: 0x8000,
						}),
				);
				map_info.extend(
					(0x40..=0x7D)
						.chain(0xC0..=0xFF)
						.filter(|&i| (i & 0x3F) << 16 < memory_map.rom.len())
						.map(|i| MapInfo::ROM {
							src: (i & 0x3F) << 16,
							dst: i << 16,
							len: 0x10000,
						}),
				);

				if let Some(sram_size) = memory_map.sram.as_ref().map(|sram| sram.len()) {
					// with SRAM
					map_info.extend(
						(0x20..=0x3F)
							.chain(0xA0..=0xBF)
							.flat_map(|i| (i << 16 | 0x6000..i << 16 | 0x8000).step_by(sram_size))
							.map(|dst| MapInfo::SRAM {
								src: 0,
								dst,
								len: sram_size,
							}),
					);
				}
			}
			None => {
				todo!()
			}
		}

		memory_map.map(&map_info);

		memory_map
	}

	fn map(&mut self, info: &[MapInfo]) {
		for &info in info.iter() {
			match info {
				MapInfo::ROM { src, dst, len } => {
					let src = src..src.checked_add(len).unwrap();
					let dst = dst..dst.checked_add(len).unwrap();
					for (readable, src) in self.readable[dst.clone()]
						.iter_mut()
						.zip(self.rom[src].iter())
					{
						*readable = Some(src as *const _);
					}
				}
				MapInfo::WRAM { src, dst, len } => {
					let src = src..src.checked_add(len).unwrap();
					let dst = dst..dst.checked_add(len).unwrap();
					for ((readable, writable), src) in self.readable[dst.clone()]
						.iter_mut()
						.zip(self.writable[dst].iter_mut())
						.zip(self.wram[src].iter())
					{
						*readable = Some(src as *const _);
						*writable = Some(src as *const _);
					}
				}
				MapInfo::SRAM { src, dst, len } => {
					let src = src..src.checked_add(len).unwrap();
					let dst = dst..dst.checked_add(len).unwrap();
					for ((readable, writable), src) in self.readable[dst.clone()]
						.iter_mut()
						.zip(self.writable[dst].iter_mut())
						.zip(self.sram.as_ref().unwrap()[src].iter())
					{
						*readable = Some(src as *const _);
						*writable = Some(src as *const _);
					}
				}
			}
		}
	}

	#[inline]
	pub fn read(&self, offset: Address24) -> u8 {
		unsafe {
			if let Some(p) = self
				.readable
				.get_unchecked(Into::<usize>::into(offset))
				.clone()
			{
				debug_assert!(
					self.wram.as_ptr_range().contains(&p)
						|| self.rom.as_ptr_range().contains(&p)
						|| self
							.sram
							.as_ref()
							.map_or(false, |sram| sram.as_ptr_range().contains(&p))
				);
				(*p).load(atomic::Ordering::SeqCst)
			} else {
				0x55
			}
		}
	}

	#[inline]
	pub fn write(&self, offset: Address24, value: u8) {
		unsafe {
			if let Some(p) = self
				.writable
				.get_unchecked(Into::<usize>::into(offset))
				.clone()
			{
				debug_assert!(
					self.wram.as_ptr_range().contains(&p)
						|| self.rom.as_ptr_range().contains(&p)
						|| self
							.sram
							.as_ref()
							.map_or(false, |sram| sram.as_ptr_range().contains(&p))
				);
				(*p).store(value, atomic::Ordering::SeqCst);
			}
		}
	}
}
