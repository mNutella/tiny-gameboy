use crate::{GPU, VRAM_BEGIN, VRAM_END};

pub fn get_vram_address(address: u16) -> usize {
    address as usize - VRAM_BEGIN
}

pub struct MemoryBus {
    pub memory: [u8; 0xFFFF],
    pub gpu: GPU,
}

impl MemoryBus {
    pub fn read_byte(&self, address: u16) -> u8 {
        match address as usize {
            VRAM_BEGIN..VRAM_END => self.gpu.read_memory(get_vram_address(address)),
            _ => self.memory[address as usize],
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address as usize {
            VRAM_BEGIN..VRAM_END => self.gpu.write_memory(get_vram_address(address), value),
            _ => self.memory[address as usize] = value,
        }
    }
}
