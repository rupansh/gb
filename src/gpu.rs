use crate::consts::*;
use crate::frontend::FrontEnd;
use crate::mem::Mem;

#[derive(PartialEq)]
enum GpuMode {
    OAM,
    VRAM,
    HBLANK,
    VBLANK
}

pub struct Gpu {
    mode: GpuMode,
    clk: u64,
    prev: u64,
    pub frames: f64,
    pub front: FrontEnd,
}


impl Default for Gpu {
    fn default() -> Gpu {
        let gp = Gpu {
            mode: GpuMode::OAM,
            clk: 0,
            frames: 0.,
            prev: 0,
            front: FrontEnd::default()
        };
        return gp;
    }
}

impl Gpu {
    fn lcd_on(&self, gb_mem: &Mem) -> bool {
        return gb_mem.read(LCD_CTLP) & 0x80 != 0;
    }

    fn line(&self, gb_mem: &Mem) -> u8 {
        return gb_mem.read(SCLINEP);
    }

    fn set_line(&mut self, gb_mem: &mut Mem, val: u8) {
        gb_mem.write(SCLINEP, val);
        let gint = gb_mem.read(GPU_INTS);
        if val == self.lyc(gb_mem) {
            if gint & 0x40 != 0  {
                gb_mem.write(PINT_F, gb_mem.read(PINT_F) | 0x2)
            }
            gb_mem.write(GPU_INTS, gint | 0x4);
        } else {
            gb_mem.write(GPU_INTS, gint & 0xFB)
        }
    }

    fn lyc(&self, gb_mem: &Mem) -> u8 {
        return gb_mem.read(LYCP);
    }

    fn set_mode(&mut self, gb_mem: &mut Mem, mode: GpuMode) {
        self.mode = mode;
        let i_mode = match self.mode {
            GpuMode::HBLANK => 0,
            GpuMode::VBLANK => 1,
            GpuMode::OAM => 2,
            GpuMode::VRAM => 3,
        };
        let gint = gb_mem.read(GPU_INTS) & 0xFC | i_mode;
        gb_mem.write(GPU_INTS, gint);
        if i_mode != 3 && gint & (1 << 3+i_mode) != 0 {
            gb_mem.write(PINT_F, gb_mem.read(PINT_F) | 0x2)
        }
    }

    fn get_color(&self, gb_mem: &Mem, cn: u8, addr: u16) -> (u8, u8, u8) {
        let pallete = gb_mem.read(addr);
        let col = if cn <= 3 {
            ((((1 << (1 + (2*cn))) & pallete != 0) as u8) << 1) | ((1 << (2*cn)) & pallete != 0) as u8
        } else {
            (((1 & pallete != 0) as u8) << 1) | (1 & pallete != 0) as u8
        };

        return match col {
            0 => PAL_0,
            1 => PAL_1,
            2 => PAL_2,
            3 => PAL_3,
            _ => (0, 0, 0),
        };
    }

    fn draw_line(&mut self, gb_mem: &mut Mem) {
        let val = gb_mem.read(LCD_CTLP);
        let sline = self.line(gb_mem);
        let mut bgpix = [false; 160];

        if val & 0x1 != 0 {
            let wy = gb_mem.read(WYP);
            let sw = (val & 0x20 != 0) && wy <= sline;
            let y = if sw {
                sline - wy
            } else {
                (gb_mem.read(SCYP) as u16 + sline as u16) as u8
            };

            for i in 0..160 {
                let wx = gb_mem.read(WXP);
                let x = if sw && i >= wx {
                    i - wx
                } else {
                    i.wrapping_add(gb_mem.read(SCXP))
                };

                let tx = (x as u16)/8;
                let ty = ((y as u16)/8) * 32;
                let msk = if sw { 0x40 } else { 0x8 };
                let bgmap = if val & msk != 0 { 0x9C00 } else { 0x9800 };
                let tnp = bgmap + tx + ty;
                let tn = gb_mem.read(tnp);

                let bgtile = if val & 0x10 != 0 { 0x8000 } else {0x8800};
                let tp = if bgtile == 0x8000 {
                    bgtile + (tn as u16)*16
                } else {
                    bgtile + ((tn as i8 as u16 + 128)*16) as u16
                };

                let l = (y as u16 % 8) * 2;

                let l_tile = gb_mem.read(tp + l);
                let h_tile = gb_mem.read(tp + l + 1);

                let cb = (((x as i16 % 8) - 7)*(-1)) as u8;

                let cn = ((((1 << cb as u8) & h_tile != 0) as u8) << 1) | ((1 << cb as u8) & l_tile != 0) as u8;

                let col = self.get_color(gb_mem, cn, BG_PALLP);
                bgpix[i as usize] = col.0 == 224;
                self.front.draw_pix(i as i32, sline as i32, col);

            }
        }

        if val & 0x2 != 0 {
            let s2x = val & 0x4 != 0;
            for i in 0..40 {
                let addr = SPRITE_BASE + (i as u16)*4;
                let y = gb_mem.read(addr) as i32 - 16;
                let x = gb_mem.read(addr + 1) as i32 - 8;
                let tp = gb_mem.read(addr + 2) as u16 & (0xFF - s2x as u16);
                let attr = gb_mem.read(addr + 3);
                let bg_prio = attr & 0x80 != 0;
                let fy = attr & 0x40 != 0;
                let fx = attr & 0x20 != 0;
                let sy = (s2x as u8 + 1) * 8;

                if !(y <= sline as i32 && (y+8) > sline as i32) {
                    continue
                }

                let line = if fy {
                    (sy as i32 - (sline as i32 - y) - 1) as u16
                } else {
                    (sline as i32 - y) as u16
                };

                let sp = (0x8000 + tp as u32 * 16 + line as u32 *2) as u16;
                let l_sprite = gb_mem.read(sp);
                let h_sprite = gb_mem.read(sp + 1);
                for j in (0..8).rev() {
                    let mut cb = j as i8;
                    if fx {
                        cb = 7 - cb
                    }

                    let cn = ((((1 << cb as u8) & h_sprite != 0) as u8) << 1) | ((1 << cb as u8) & l_sprite != 0) as u8;
                    let cp = (attr & 0x10 != 0) as u16 + OBJPALBP;
                    let col = self.get_color(gb_mem, cn, cp);

                    if bg_prio && (val & 0x1) != 0 && !bgpix[((x + j) % 160) as usize] {
                        continue;
                    }

                    let pix = (x as u8).wrapping_add(7-j as u8);
                    self.front.draw_pix(pix as i32, sline as i32, col);
                }
            }
        }
    }
}

pub fn gpu_cycle(gb_gpu: &mut Gpu, gb_mem: &mut Mem, clks: u64) {
    if !gb_gpu.lcd_on(gb_mem) { return; }

    gb_gpu.clk += clks - gb_gpu.prev;
    gb_gpu.prev = clks;

    match gb_gpu.mode {
        GpuMode::HBLANK => {
            if gb_gpu.clk >= 204 {
                if gb_gpu.line(gb_mem) == 143 {
                    gb_gpu.set_mode(gb_mem, GpuMode::VBLANK);
                    gb_gpu.front.render();
                    gb_mem.write(PINT_F, gb_mem.read(PINT_F) | 0x1);
                } else {
                    gb_gpu.set_mode(gb_mem, GpuMode::OAM);
                }
                gb_gpu.set_line(gb_mem, gb_gpu.line(gb_mem) + 1);
                gb_gpu.clk -= 204;
            }
        },
        GpuMode::VBLANK => {
            if gb_gpu.clk >= 456 {
                gb_gpu.clk -= 456;
                gb_gpu.set_line(gb_mem, gb_gpu.line(gb_mem) + 1);
                if gb_gpu.line(gb_mem) > 153 {
                    gb_gpu.set_line(gb_mem, 0);
                    gb_gpu.set_mode(gb_mem, GpuMode::OAM);
                }
            }
        },
        GpuMode::OAM => {
            if gb_gpu.clk >= 80 {
                gb_gpu.clk -= 80;
                gb_gpu.set_mode(gb_mem, GpuMode::VRAM);
            }
        },
        GpuMode::VRAM => {
            if gb_gpu.clk >= 172 {
                gb_gpu.clk -= 172;
                gb_gpu.set_mode(gb_mem, GpuMode::HBLANK);
                gb_gpu.draw_line(gb_mem);
            }
        }
    }
}
