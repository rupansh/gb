pub struct Mem {
    pub rom: [u8; 16384], // rom mem
    pub rom_bank: [u8; 16384], // rom bank
    vram: [u8; 8192], // video ram
    exram: [u8; 8192], // external ram
    wram: [u8; 8192], // work ram
    sdata: [u8; 160], // sprite data
    pub io: [u8; 128], // I/O mem
    zero_pg: [u8; 128], // Zero Page
    pub input_update: bool, // Tell input to update joy io reg
}

impl Mem {
    pub fn read(&self, address: u16) -> u8 {
        let addr = address as usize;
        return match addr {
            0x0000..=0x3FFF => self.rom[addr], // Rom
            0x4000..=0x7FFF => self.rom_bank[addr - 0x4000], // Rom Bank
            0x8000..=0x9FFF => self.vram[addr - 0x8000], // Video Ram
            0xA000..=0xBFFF => self.exram[addr - 0xA000], // External Ram
            0xC000..=0xDFFF => self.wram[addr - 0xC000], // Work Ram
            0xE000..=0xFDFF => self.wram[addr & 0x1FFF], // Work Ram copy
            0xFE00..=0xFE9F => self.sdata[addr - 0xFE00], // Sprite Data/Object Mem
            0xFF00..=0xFF7F => self.io[addr - 0xFF00],// I/O 
            0xFF80..=0xFFFF => self.zero_pg[addr - 0xFF80], // Zero Page
            _ => 0 // Includes skipped FE10 to FEFF
        };
    }

    pub fn write(&mut self, address: u16, val: u8) {
        let addr = address as usize;
        match addr {
            0x8000..=0x9FFF => self.vram[addr - 0x8000] = val, // Video Ram
            0xA000..=0xBFFF => self.exram[addr - 0xA000] = val, // External Ram
            0xC000..=0xDFFF => self.wram[addr - 0xC000] = val, // Work Ram
            0xE000..=0xFDFF => self.wram[addr & 0x1FFF] = val, // Work Ram copy
            0xFE00..=0xFE9F => self.sdata[addr - 0xFE00] = val, // Sprite Data/Object Mem
            0xFF00 => {
                self.io[0] = val;
                self.input_update = true;
            }
            0xFF01..=0xFF7F => {
                if addr == 0xFF04 {
                    self.io[4] = 0; // divide timer reg
                } else if addr == 0xFF07 {
                    self.io[7] = val & 0x7;
                } else if addr == 0xFF46 {
                    self.io[0x46] = val;
                    for i in 0..160 {
                        self.sdata[i] = self.read(((val as u16) << 8) + i as u16) // OAM DMA
                    }
                } else {
                    self.io[addr - 0xFF00] = val;
                }
            }, // I/O 
            0xFF80..=0xFFFF => self.zero_pg[addr - 0xFF80] = val, // Zero Page
            _ => return
        };
    }
}

impl Default for Mem {
    fn default() -> Mem {
        let mut m = Mem {
            rom: [0; 16384],
            rom_bank: [0; 16384],
            vram: [0; 8192],
            exram: [0; 8192],
            wram: [0; 8192],
            sdata: [0; 160],
            io: [0; 128],
            zero_pg: [0; 128],
            input_update: false,
        };
        m.io[0x10] = 0x80;
        m.io[0x11] = 0xBF;
        m.io[0x12] = 0xF3;
        m.io[0x14] = 0xBF;
        m.io[0x16] = 0x3F;
        m.io[0x19] = 0xBF;
        m.io[0x1A] = 0x7F;
        m.io[0x1B] = 0xFF;
        m.io[0x1C] = 0x9F;
        m.io[0x1E] = 0xBF;
        m.io[0x20] = 0xFF;
        m.io[0x23] = 0xBF;
        m.io[0x24] = 0x77;
        m.io[0x25] = 0xF3;
        m.io[0x26] = 0xF1;
        m.io[0x40] = 0x91;
        m.io[0x47] = 0xFC;
        m.io[0x48] = 0xFF;
        m.io[0x49] = 0xFF;
        return m;
    }
}