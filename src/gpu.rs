pub const VRAM_START: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
const VRAM_SIZE: usize = VRAM_END - VRAM_START + 1;

#[derive(Clone, Copy)]
enum PixelValue {
    Zero,
    One,
    Two,
    Three,
}

type Tile = [[PixelValue; 8]; 8];

fn empty_tile() -> Tile {
    [[PixelValue::Zero; 8]; 8]
}

pub struct GPU {
    vram: [u8; VRAM_SIZE],
    tile_set: [Tile; 384],
}

impl GPU {
    pub fn new() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            tile_set: [empty_tile(); 384],
        }
    }

    pub fn read_vram(&self, address: usize) -> u8 {
        self.vram[address]
    }

    pub fn write_vram(&mut self, address: usize, value: u8) {
        self.vram[address] = value;

        // Outside of tileset area. Nothing else to do;
        if address >= 0x1800 {
            return;
        }

        // Tiles are encoded in two bytes, the first is always on an even address
        let aligned_address = address & 0xFFFE;

        // Tile row
        let byte_1 = self.vram[aligned_address];
        let byte_2 = self.vram[aligned_address + 1];

        //
        let tile_address = address / 16;
        let row_address = (address % 16) / 2;

        for pixel_index in 0..8 {
            let mask = 1 << (7 - pixel_index);
            let lo = byte_1 & mask;
            let hi = byte_2 & mask;

            let value = match (lo != 0, hi != 0) {
                (true, true) => PixelValue::Three,
                (false, true) => PixelValue::Two,
                (true, false) => PixelValue::One,
                (false, false) => PixelValue::Zero,
            };

            self.tile_set[tile_address][row_address][pixel_index] = value;
        }
    }
}
