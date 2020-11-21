use std::sync::atomic::{self, AtomicU8};

use crate::address::Address24;
use crate::cartridge::Cartridge;

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
	pub fn from_cartridge(cartridge: Cartridge) -> Self {
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

		// WRMA
		// $xx:0000-$xx:1FFF
		for i in 0..0x40 {
			memory_map.map(&[
				MapInfo::WRAM {
					src: 0,
					dst: i << 16,
					len: 0x2000,
				},
				MapInfo::WRAM {
					src: 0,
					dst: (i | 0x80) << 16,
					len: 0x2000,
				},
			]);
		}

		// $7E-$7F
		memory_map.map(&[MapInfo::WRAM {
			src: 0,
			dst: 0x7E0000,
			len: 2 * PAGE_SIZE,
		}]);

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
