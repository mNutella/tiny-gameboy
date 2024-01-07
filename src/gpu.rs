pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;

#[derive(Copy, Clone)]
enum TilePixelValue {
    White,
    Gray,
    Light,
    Black,
}

type Tile = [[TilePixelValue; 8]; 8];

pub fn empty_tile() -> Tile {
    [[TilePixelValue::Gray; 8]; 8]
}

pub struct GPU {
    memory: [u8; VRAM_SIZE],
    tiles: [Tile; 384],
}

impl GPU {
    pub fn read_memory(&self, address: usize) -> u8 {
        self.memory[address]
    }

    pub fn write_memory(&mut self, address: usize, value: u8) {
        self.memory[address] = value;

        if address >= 0x1800 {
            return;
        }

        let normalized_address = address & 0xFFFE;

        let ms_byte = self.memory[normalized_address];
        let ls_byte = self.memory[normalized_address + 1];

        let tile_address = address / 16;
        let row_address = (address % 16) / 2;

        for pixel_address in 0..8 {
            let mask = 1 << (7 - pixel_address);
            let l_ms_byte = ms_byte & mask;
            let l_ls_byte = ls_byte & mask;

            let value = match (l_ms_byte != 0, l_ls_byte != 0) {
                (true, true) => TilePixelValue::White,
                (true, false) => TilePixelValue::Gray,
                (false, true) => TilePixelValue::Light,
                (false, false) => TilePixelValue::Black,
            };

            self.tiles[tile_address][row_address][pixel_address] = value;
        }
    }
}
