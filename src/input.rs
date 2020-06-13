extern crate sdl2;

use crate::mem::Mem;
use crate::consts::*;

use sdl2::keyboard::Keycode;


pub struct Input {
    r0: u8,
    r1: u8
}


impl Input {
    pub fn key(&mut self, gb_mem: &mut Mem, key: Keycode, kp: bool) {
        match key {
            Keycode::Return => self.update_row(START, kp, false),
            Keycode::Space => self.update_row(SELECT, kp, false),
            Keycode::E => self.update_row(KEY_B, kp, false),
            Keycode::Q => self.update_row(KEY_A, kp, false),
            Keycode::S => self.update_row(KEY_D, kp, true),
            Keycode::W => self.update_row(KEY_U, kp, true),
            Keycode::A => self.update_row(KEY_L, kp, true),
            Keycode::D => self.update_row(KEY_R, kp, true),
            _ => {}
        };

        self.update(gb_mem);
    }

    pub fn update(&mut self, gb_mem: &mut Mem) {
        let dat = gb_mem.read(JOYP);
        let mut val = 0xF;

        if dat & 0x10 == 0 {
            val &= self.r0;
        }

        if dat & 0x20 == 0 {
            val &= self.r1;
        }

        if dat & 0xF != 0 && val != 0xF {
            let int_f = gb_mem.read(PINT_F);
            gb_mem.write(PINT_F, int_f | 0x10);
        }

        gb_mem.write(JOYP, (dat & 0xF0) | val);
        gb_mem.input_update = false;
    }

    pub fn update_row(&mut self, key: u8, kp: bool, r0: bool) {
        if r0 {
            if kp {
                self.r0 &= !key;
            } else {
                self.r0 |= key;
            }
        } else {
            if kp {
                self.r1 &= !key;
            } else {
                self.r1 |= key;
            }
        }
    }
}

impl Default for Input {
    fn default() -> Input {
        Input {
            r0: 0x0F,
            r1: 0x0F
        }
    }
}
