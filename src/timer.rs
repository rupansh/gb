use crate::cpu::Clock;
use crate::consts::*;
use crate::mem::Mem;


pub struct Timer {
    cnt: u32,
    div: u32
}

impl Default for Timer {
    fn default() -> Timer {
        Timer {
            cnt: 0,
            div: 0,
        }
    }
}

impl Timer {
    pub fn inc(&mut self, cp_clks: &Clock, gb_mem: &mut Mem) {
        self.div += cp_clks.prev as u32;
        if self.div >= 256 {
            let mut div = gb_mem.read(DIVTP);
            div = div.wrapping_add(1);
            gb_mem.write(DIVTP, div);
            self.div -= 256;
        }

        let ctrl = gb_mem.read(CTLTTP);
        if gb_mem.read(CTLTTP) as i8 - 0x4 > 0 {
            self.cnt += cp_clks.prev as u32;
            let lim = match ctrl & 0x3 {
                1 => 16,
                2 => 64,
                3 => 256,
                _ => 1024
            };

            if self.cnt >= lim {
                let rcnt = gb_mem.read(CNTTP).wrapping_add(1);
                if rcnt == 0 {
                    gb_mem.write(CNTTP, gb_mem.read(MODTP));
                    let int_f = gb_mem.read(PINT_F);
                    gb_mem.write(PINT_F, int_f | 4);
                }
                self.cnt -= lim;
            }
        }
    }
}