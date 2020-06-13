extern crate minifb;
extern crate sdl2;

use crate::consts::*;
use crate::mem::Mem;

use sdl2::pixels::Color;


enum GpuMode {
    OAM,
    VRAM,
    HBLANK,
    VBLANK
}

pub struct Gpu {
    mode: GpuMode,
    clk: u64,
    switchbg: bool,
    switchlcd: bool,
    switchsprite: bool,
    switchwindow: bool,
    sprite2x: bool,
    bgmap: u16,
    bgtile: u16,
    line: u8,
    scx: u8,
    scy: u8,
    wx: u8,
    wy: u8,
    pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
    pub ctx: sdl2::Sdl
}

impl Gpu {
    fn lcd_control_reg(&mut self, gb_mem: &Mem) {
        let val = gb_mem.read(LCD_CTLP);

        self.switchbg = val & 0x1 != 0;
        self.switchsprite = val & 0x2 != 0;
        self.sprite2x = val & 0x4 != 0;
        self.bgtile = if val & 0x10 != 0 { 0x8000 } else {0x8800};
        self.switchwindow = (val & 0x20 != 0) && self.wx <= self.line;
        let msk = if self.switchwindow { 0x40 } else { 0x8 };
        self.bgmap = if val & msk != 0 { 0x9C00 } else { 0x9800 };
        self.switchlcd = val & 0x80 != 0;
    }


    pub fn update(&mut self, gb_mem: &Mem) {
        self.scy = gb_mem.read(SCYP);
        self.scx = gb_mem.read(SCXP);
        self.line = gb_mem.read(SCLINEP);
        self.wy = gb_mem.read(WYP);
        self.wx = gb_mem.read(WXP);
        self.lcd_control_reg(gb_mem);
    }

    fn get_color(&self, gb_mem: &Mem, cn: u8, addr: u16) -> Color {
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
            _ => Color::BLACK,
        };
    }

    pub fn scanline(&mut self, gb_mem: &mut Mem) {
        if self.switchbg {
            let y = if self.switchwindow {
                self.line - self.wy
            } else {
                (self.scy as u16 + self.line as u16) as u8
            };

            for i in 0..160 {
                let x = if self.switchwindow && i >= self.wx {
                    i - self.wx
                } else {
                    i.wrapping_add(self.scx)
                };

                let tx = (x as u16)/8;
                let ty = ((y as u16)/8) * 32;
                let tnp = self.bgmap + tx + ty;
                let tn = gb_mem.read(tnp);

                let tp = if self.bgtile == 0x8000 {
                    self.bgtile + (tn as u16)*16
                } else {
                    self.bgtile + ((tn as i16 + 128)*16) as u16
                };

                let l = (y as u16 % 8) * 2;

                let l_tile = gb_mem.read(tp + l);
                let h_tile = gb_mem.read(tp + l + 1);

                let cb = (((x as i16 % 8) - 7)*(-1)) as u8;

                let cn = ((((1 << cb) & h_tile != 0) as u8) << 1) | ((1 << cb) & l_tile != 0) as u8;

                let col = self.get_color(gb_mem, cn, BG_PALLP);
                self.canvas.set_draw_color(col);
                self.canvas.draw_point(sdl2::rect::Point::new(i as i32,  self.line as i32)).unwrap_or_default();
            }
        }

        if self.switchsprite {
            for i in 0..40 {
                let addr = SPRITE_BASE + (i as u16)*4;
                let y = gb_mem.read(addr) as i32 - 16;
                let x = gb_mem.read(addr + 1) as i32 - 8;
                let tp = gb_mem.read(addr + 2) as u16 & (0xFF - self.sprite2x as u16);
                let attr = gb_mem.read(addr + 3);

                if attr & 0x80 != 0 {
                    continue;
                }

                let fy = attr & 0x40 != 0;
                let fx = attr & 0x20 != 0;
                let sy = self.sprite2x as u8 * 8;

                if !(y <= self.line as i32 && (y + 8) > self.line as i32) {
                    continue
                }

                let line = if fy {
                    (sy as i32 - (self.line as i32 - y) - 1) as u16
                } else {
                    (self.line as i32 - y) as u16
                };

                let sp = 0x8000 + tp*16 + line*2;
                let l_sprite = gb_mem.read(sp);
                let h_sprite = gb_mem.read(sp + 1);

                for j in (0..8).rev() {
                    let mut cb = j as i8;
                    if fx {
                        cb = (cb - 7)*(-1)
                    }

                    let cn = ((((1 << cb) & h_sprite != 0) as u8) << 1) | ((1 << cb) & l_sprite != 0) as u8;
                    let cp = (attr & 0x10 != 0) as u16 + 0xFF48;
                    let col = self.get_color(gb_mem, cn, cp);

                    if col.r == 136 {
                        continue
                    }

                    let pix = (x as u16 + (7 -j) as u16) as u8;
                    self.canvas.set_draw_color(col);
                    self.canvas.draw_point(sdl2::rect::Point::new(pix as i32,  self.line as i32)).unwrap_or_default();
                }
            }
        }
    }
}

impl Default for Gpu {
    fn default() -> Gpu {
        let ctx = sdl2::init().unwrap();
        let mut gp = Gpu {
            mode: GpuMode::OAM,
            clk: 0,
            bgmap: 0,
            bgtile: 0,
            switchbg: false,
            switchlcd: false,
            switchsprite: false,
            sprite2x: false,
            switchwindow: false,
            line: 0,
            scx: 0,
            scy: 0,
            wx: 0,
            wy: 0,
            canvas: ctx.video().unwrap()
                .window("Gameboy Emu", WIDTH as u32, HEIGHT as u32)
                .position_centered()
                .build().unwrap()
                .into_canvas()
                .build().unwrap(),
            ctx: ctx
        };
        gp.canvas.set_draw_color(PAL_0);
        gp.canvas.clear();
        gp.canvas.present();
        return gp;
    }
}

pub fn gpu_cycle(gb_gpu: &mut Gpu, gb_mem: &mut Mem, clks: u64) {
    gb_gpu.clk = clks;
    gb_gpu.update(gb_mem);
    let mut int_f = gb_mem.read(PINT_F);
    let ints = gb_mem.read(GPU_INTS);

    match &gb_gpu.mode {
        GpuMode::OAM => {
            if gb_gpu.clk >= 80 {
                gb_gpu.mode = GpuMode::VRAM;
                gb_gpu.clk = 0;
            }
        }

        GpuMode::VRAM => {
            if gb_gpu.clk >= 172 {
                gb_gpu.clk = 0;
                gb_gpu.mode = GpuMode::HBLANK;
                if int_f & 0x8 != 0 {
                    int_f |= 0x2;
                    gb_mem.write(PINT_F, int_f)
                }

                gb_gpu.scanline(gb_mem);
            }
        }

        GpuMode::HBLANK => {
            if gb_gpu.clk >= 204 {
                gb_gpu.clk = 0;
                gb_mem.write(SCLINEP, gb_gpu.line+1);
                if ints & 0x40 != 0 && gb_gpu.line+1 == gb_mem.read(LYCP) {
                    int_f |= 0x2;
                    gb_mem.write(PINT_F, int_f);
                }

                // We aren't incrementing gb_gpu.line so 144-1.
                // line is updated in the next frame cycle anyways
                if gb_gpu.line == 142 {
                    gb_gpu.mode = GpuMode::VBLANK;
                    int_f |= 0x1;
                    gb_mem.write(PINT_F, int_f);
                    if ints & 0x10 != 0 {
                        int_f |= 0x2;
                        gb_mem.write(PINT_F, int_f);
                    }
                    gb_gpu.canvas.present();
                } else {
                    gb_gpu.mode = GpuMode::OAM;
                    if int_f & 0x20 != 0 {
                        int_f |= 0x2;
                        gb_mem.write(PINT_F, int_f);
                    }
                }
            }
        }

        GpuMode::VBLANK => {
            if gb_gpu.clk >= 456 {
                gb_gpu.clk = 0;
                gb_mem.write(SCLINEP, gb_gpu.line+1);

                if gb_gpu.line > 152 {
                    gb_gpu.mode = GpuMode::OAM;
                    if int_f & 0x20 != 0 {
                        int_f |= 0x2;
                        gb_mem.write(PINT_F, int_f);
                    }
                    gb_gpu.line = 0;
                    gb_mem.write(SCLINEP, 0);
                }
            }
        }
    }
}