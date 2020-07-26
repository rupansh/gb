use crate::consts::*;
use crate::mem::Mem;


pub struct Timer {
    cnt: u32,
    div: u32,
    prev: u64
}

impl Default for Timer {
    fn default() -> Timer {
        Timer {
            cnt: 0,
            div: 0,
            prev: 0
        }
    }
}

impl Timer {
    pub fn inc(&mut self, cp_clks: u64, gb_mem: &mut Mem) {
        let tclk = cp_clks - self.prev;
        self.prev = cp_clks;
        self.div += tclk as u32;
        while self.div >= 256 {
            let div = gb_mem.read(DIVTP).wrapping_add(1);
            gb_mem.io[4] = div;
            self.div -= 256;
        }

        let ctrl = gb_mem.read(CTLTTP);
        if ctrl as i8 - 0x4 > 0 {
            self.cnt += tclk as u32;
            let lim = match ctrl & 0x3 {
                0 => 1024,
                1 => 16,
                2 => 64,
                3 => 256,
                _ => panic!("BRUH")
            };

            while self.cnt >= lim {
                match gb_mem.read(CNTTP).checked_add(1) {
                    Some(s) => gb_mem.write(CNTTP, s),
                    None => {
                        gb_mem.write(CNTTP, gb_mem.read(MODTP));
                        let int_f = gb_mem.read(PINT_F);
                        gb_mem.write(PINT_F, int_f | 4);
                    }
                }
                self.cnt -= lim;
            }
        }
    }
}