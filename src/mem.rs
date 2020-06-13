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
                //self.io[0] = val & 0x30 | (self.io[0] & 0xCF);
                self.io[0] = val;
                self.input_update = true;
            }
            0xFF01..=0xFF7F => {
                if addr == 0xFF02 {
                    if val == 0x81 {
                        let intf = self.read(0xFF0F);
                        self.write(0xFF0F, intf | 0x8);
                    }
                }
                if addr == 0xFF04 {
                    self.io[4] = 0; // divide timer reg
                } else if addr == 0xFF07 {
                    self.io[7] = val & 0x7;
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
        Mem {
            rom: [0; 16384],
            rom_bank: [0; 16384],
            vram: [0; 8192],
            exram: [0; 8192],
            wram: [0; 8192],
            sdata: [0; 160],
            io: [0; 128],
            zero_pg: [0; 128],
            input_update: false,
        }
    }
}