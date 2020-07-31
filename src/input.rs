use crate::mem::Mem;
use crate::consts::*;


pub struct Input {
    r0: u8,
    r1: u8
}

pub enum KeyCode {
    Start,
    Select,
    A,
    B,
    Up,
    Down,
    Left,
    Right,
    Uk
}


impl Input {
    pub fn key(&mut self, gb_mem: &mut Mem, key: KeyCode, kp: bool) {
        match key {
            KeyCode::Start => self.update_row(START, kp, false),
            KeyCode::Select => self.update_row(SELECT, kp, false),
            KeyCode::B => self.update_row(KEY_B, kp, false),
            KeyCode::A => self.update_row(KEY_A, kp, false),
            KeyCode::Down => self.update_row(KEY_D, kp, true),
            KeyCode::Up => self.update_row(KEY_U, kp, true),
            KeyCode::Left => self.update_row(KEY_L, kp, true),
            KeyCode::Right => self.update_row(KEY_R, kp, true),
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

    fn update_row(&mut self, key: u8, kp: bool, r0: bool) {
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
