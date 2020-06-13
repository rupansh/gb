use crate::cpu::Clock;
use crate::consts::*;
use crate::mem::Mem;


pub struct Timer {
    div: u16,
    tima: u16
}

impl Default for Timer {
    fn default() -> Timer {
        Timer {
            div: 0,
            tima: 0
        }
    }
}

impl Timer {
    pub fn inc(&mut self, cp_clks: &Clock, gb_mem: &mut Mem) {
        self.div += cp_clks.prev as u16;
        if self.div >= 0xff {
            self.div -= 0xff;
            let div = gb_mem.read(DIVTP).wrapping_add(1);
            gb_mem.write(DIVTP, div);
        }

        let tac = gb_mem.read(CTLTTP);
        if tac & 0x4 != 0{
            self.tima += cp_clks.prev as u16;
            let freq = match tac & 0x3 {
                0 => 1024,
                1 => 16,
                2 => 64,
                3 => 256,
                _ => panic!("BRUH")
            };

            while self.tima >= freq {
                self.tima -= freq;
                let tima = match gb_mem.read(CNTTP).checked_add(1) {
                    Some(s) => s,
                    None => {
                        let intf = gb_mem.read(PINT_F) | 0x4;
                        gb_mem.write(PINT_F, intf);
                        gb_mem.read(MODTP)
                    }
                };
                gb_mem.write(CNTTP, tima);
            }
        }
    }
}