use crate::consts::*;
use crate::mem;


pub struct Cpu {
    pub regs: [u8; 8], // Regs A-F, H,L
    pub sp: u16, // Stack pointer
    pub pc: u16, // Program Couter
    pub ime: u8, // interrupt master enable
    pub alt_c: u8, // alt cycles
    pub halt: u8, // HALT mode
    pub stop: u8, // STOP mode
    pub clk: u64, // clock counter
}

impl Cpu {
    pub fn get_hilo(&self, rh: usize, rl: usize) -> u16 {
        return ((self.regs[rh] as u16) << 8) | self.regs[rl] as u16;
    }

    pub fn set_hilo(&mut self, rh: usize, rl: usize, val: u16) {
        self.regs[rh] = ((val & 0xFF00) >> 8) as u8;
        self.regs[rl] = (val & 0x00FF) as u8;
    }

    pub fn get_flag(&self, flag: u8) -> bool {
        return (self.regs[F] & flag) > 0;
    }

    pub fn set_flag(&mut self, val: bool, flag: u8) {
        self.regs[F] = if val {
            self.regs[F] | flag
        } else {
            self.regs[F] & !flag
        };
    }

    pub fn ld_16(&mut self, gb_mem: &mem::Mem, rh: usize, rl: usize) {
        self.clk += 4;
        self.regs[rl] = gb_mem.read(self.pc);
        self.clk += 4;
        self.pc = self.pc.wrapping_add(1);
        self.regs[rh] = gb_mem.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        self.clk += 4
    }

    pub fn ld_8(&mut self, gb_mem: &mem::Mem, r: usize) {
        self.clk += 4;
        self.regs[r] = gb_mem.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        self.clk += 4;
    }

    pub fn ld_p16_8(&mut self, gb_mem: &mut mem::Mem, rh: usize, rl: usize, r8: usize) {
        self.clk += 4;
        gb_mem.write(self.get_hilo(rh, rl), self.regs[r8]);
        self.clk += 4;
    }

    pub fn ld_8_p16(&mut self, gb_mem: &mem::Mem, r8: usize, rh: usize, rl: usize) {
        self.clk += 4;
        self.regs[r8] = gb_mem.read(self.get_hilo(rh, rl));
    }

    pub fn inc_16(&mut self, rh: usize, rl: usize) {
        self.set_hilo(rh, rl, (self.get_hilo(rh, rl) as u32 +1) as u16);
        self.clk += 8;
    }

    pub fn dec_16(&mut self, rh: usize, rl: usize) {
        self.set_hilo(rh, rl, (self.get_hilo(rh, rl) as i32 - 1) as u16);
        self.clk += 8;
    }

    pub fn inc_8(&mut self, r: usize) {
        self.set_flag(false, FL_N);
        self.set_flag((self.regs[r] & 0xF) == 0xF, FL_H);
        self.regs[r] = self.regs[r].wrapping_add(1);
        self.set_flag(self.regs[r] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn dec_8(&mut self, r: usize) {
        self.set_flag(true, FL_N);
        self.set_flag((self.regs[r] & 0xF) == 0x0, FL_H);
        self.regs[r] = self.regs[r].wrapping_sub(1);
        self.set_flag(self.regs[r] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn add_hl_16(&mut self, rh: usize, rl: usize) {
        self.set_flag(false, FL_N);
        let tmp: u32 = self.get_hilo(H, L) as u32 + self.get_hilo(rh, rl) as u32;
        self.set_flag(tmp > 0xFFFF, FL_C);
        self.set_flag((self.get_hilo(H, L) & 0x0FFF) + (self.get_hilo(rh, rl) & 0x0FFF) > 0x0FFF, FL_H);
        self.set_hilo(H, L, tmp as u16);
        self.clk += 8
    }

    pub fn add_a_8(&mut self, r: usize) {
        self.set_flag(false, FL_N);
        let tmp: u32 = self.regs[A] as u32;
        self.set_flag((tmp & 0xF) + ((self.regs[r] as u32) & 0xF) > 0xF, FL_H);
        self.regs[A] = self.regs[A].wrapping_add(self.regs[r]);
        self.set_flag(self.regs[A] == 0, FL_Z);
        self.set_flag(tmp > self.regs[A] as u32, FL_C);
        self.clk += 4;
    }

    pub fn adc_a_8(&mut self, r: usize) {
        self.set_flag(false, FL_N);
        let c = self.get_flag(FL_C) as u8;
        let h = ((self.regs[A] & 0xF) + (self.regs[r] & 0xF) + c) & 0x10;
        let tmp = self.regs[A] as u32 + self.regs[r] as u32 + c as u32;
        let c = tmp & 0x100;
        self.regs[A] =  tmp as u8;
        self.regs[F] = 0;
        if self.regs[A] == 0 {
            self.regs[F] |= 0x80
        }
        if h != 0 {
            self.regs[F] |= 0x20;
        }
        if c != 0 {
            self.regs[F] |= 0x10;
        }
        self.clk += 4;
    }

    pub fn sub_a_8(&mut self, r: usize) {
        self.regs[F] = FL_N;
        self.set_flag((self.regs[A] & 0xF) < (self.regs[r] & 0xF), FL_H);
        self.set_flag(self.regs[A] < self.regs[r], FL_C);
        self.regs[A] = self.regs[A].wrapping_sub(self.regs[r]);
        self.set_flag(self.regs[A] == 0, FL_Z);
        self.clk += 4
    }

    pub fn sbc_a_8(&mut self, r: usize) {
        let carr = self.get_flag(FL_C) as u16;
        let c = (self.regs[A] as u16) < self.regs[r] as u16 + carr;
        let h = ((self.regs[A] & 0xF) as u16) < (self.regs[r] & 0xF) as u16 + carr;
        self.regs[A] = (self.regs[A] as i32 - self.regs[r] as i32 - carr as i32) as u8;
        let z = self.regs[A] == 0;
        self.regs[F] = 0x40;
        if z {
            self.regs[F] |= 0x80;
        }
        if h {
            self.regs[F] |= 0x20;
        }
        if c {
            self.regs[F] |= 0x10;
        }
        self.clk += 4;
    }

    pub fn and_a_8(&mut self, r: usize) { 
        self.set_flag(true, FL_H);
        self.set_flag(false, FL_N | FL_C);
        self.regs[A] &= self.regs[r];
        self.set_flag(self.regs[A] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn xor_a_8(&mut self, r: usize) {
        self.set_flag(false, FL_N | FL_C | FL_H);
        self.regs[A] ^= self.regs[r];
        self.set_flag(self.regs[A] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn or_a_8(&mut self, r: usize) {
        self.set_flag(false, FL_N | FL_C | FL_H);
        self.regs[A] |= self.regs[r];
        self.set_flag(self.regs[A] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn cp_a_8(&mut self, r: usize) {
        self.set_flag(true, FL_N);
        self.set_flag((self.regs[A] & 0xF) < (self.regs[r] & 0xF), FL_H);
        self.set_flag(self.regs[A] < self.regs[r], FL_C);
        self.set_flag(self.regs[A] == self.regs[r], FL_Z);
        self.clk += 4;
    }

    pub fn rst_addr16(&mut self, gb_mem: &mut mem::Mem, addr: u16) {
        self.clk += 8;
        self.sp = (self.sp as i32 - 1) as u16;
        gb_mem.write(self.sp, (self.pc >> 8) as u8);
        self.clk += 4;
        self.sp = (self.sp as i32 - 1) as u16;
        gb_mem.write(self.sp, self.pc as u8);
        self.pc = addr;
        self.clk += 4;
    }

    pub fn push_16(&mut self, gb_mem: &mut mem::Mem, rh: usize, rl: usize) {
        self.clk += 8;
        self.sp = (self.sp as i32 - 1) as u16;
        gb_mem.write(self.sp, self.regs[rh]);
        self.clk += 4;
        self.sp = (self.sp as i32 - 1) as u16;
        gb_mem.write(self.sp, self.regs[rl]);
        self.clk += 4;
    }

    pub fn pop_16(&mut self, gb_mem: &mem::Mem, rh: usize, rl: usize) {
        self.clk += 4;
        self.regs[rl] = gb_mem.read(self.sp);
        self.clk += 4;
        self.sp = (self.sp as u32 + 1) as u16;
        self.regs[rh] = gb_mem.read(self.sp);
        self.sp = (self.sp as u32 + 1) as u16;
        self.clk += 4;
    }

    pub fn call_addr16(&mut self, gb_mem: &mut mem::Mem, cond: bool) {
        if cond {
            self.clk += 4;
            let mut tmp: u32 = gb_mem.read(self.pc) as u32;
            self.clk += 4;
            self.pc = self.pc.wrapping_add(1);
            tmp |= (gb_mem.read(self.pc) as u32) << 8;
            self.pc = self.pc.wrapping_add(1);
            self.clk += 8;
            self.sp = (self.sp as i32 - 1) as u16;
            gb_mem.write(self.sp, ((self.pc & 0xFF00) >> 8) as u8);
            self.clk += 4;
            self.sp = ((self.sp as i32 - 1)) as u16;
            gb_mem.write(self.sp, (self.pc & 0x00FF) as u8);
            self.pc = tmp as u16;
            self.clk += 4;
        } else {
            self.pc = (self.pc as u32 + 2) as u16;
            self.clk += 12;
        }
    }

    pub fn ret(&mut self, gb_mem: &mem::Mem, cond: bool) {
        if cond {
            self.clk += 4;
            let mut tmp: u32 = gb_mem.read(self.sp) as u32;
            self.clk += 4;
            self.sp = (self.sp as u32 + 1) as u16;
            tmp |= (gb_mem.read(self.sp) as u32) << 8;
            self.sp = (self.sp as u32 + 1) as u16;
            self.clk += 4;
            self.pc = tmp as u16;
            self.clk += 8
        } else {
            self.clk += 8;
        }
    }

    pub fn jp_addr16(&mut self, gb_mem: &mem::Mem, cond: bool) {
        if cond {
            self.clk += 4;
            let mut tmp: u32 = gb_mem.read(self.pc) as u32;
            self.clk += 4;
            self.pc = self.pc.wrapping_add(1);
            tmp |= (gb_mem.read(self.pc) as u32) << 8;
            self.pc = self.pc.wrapping_add(1);
            self.clk += 4;
            self.pc = tmp as u16;
            self.clk += 4;
        } else {
            self.pc = (self.pc as u32 + 2) as u16;
            self.clk += 12;
        }
    }

    pub fn jr_addr8(&mut self, gb_mem: &mem::Mem, cond: bool) {
        if cond {
            self.clk += 4;
            let tmp = gb_mem.read(self.pc) as u32;
            self.pc = self.pc.wrapping_add(1);
            self.pc = ((tmp as i8) as i32 + self.pc as i32) as u16;
            self.clk += 8;
        } else {
            self.pc = self.pc.wrapping_add(1);
            self.clk += 8;
        }
    }

    pub fn rlc_8(&mut self, r: usize) {
        self.set_flag(false, FL_N | FL_H);
        self.set_flag(self.regs[r] & 0x80 != 0, FL_C);
        self.regs[r] = (self.regs[r] << 1) | self.get_flag(FL_C) as u8;
        self.set_flag(self.regs[r] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn rrc_8(&mut self, r: usize) {
        self.set_flag(false, FL_N | FL_H);
        self.set_flag(self.regs[r] & 0x1 != 0, FL_C);
        self.regs[r] = (self.regs[r] >> 1) | ((self.get_flag(FL_C) as u8) << 7);
        self.set_flag(self.regs[r] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn rl_8(&mut self, r: usize) {
        self.set_flag(false, FL_N | FL_H);
        let tmp = self.get_flag(FL_C) as u8;
        self.set_flag(self.regs[r] & 0x80 != 0, FL_C);
        self.regs[r] = (self.regs[r] << 1) | tmp;
        self.set_flag(self.regs[r] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn rr_8(&mut self, r: usize) {
        self.set_flag(false, FL_N | FL_H);
        let tmp = self.get_flag(FL_C) as u8;
        self.set_flag(self.regs[r] & 0x1 != 0, FL_C);
        self.regs[r] = (self.regs[r] >> 1) | (tmp << 7);
        self.set_flag(self.regs[r] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn sla_8(&mut self, r: usize) {
        self.set_flag(false, FL_N | FL_H);
        self.set_flag(self.regs[r] & 0x80 != 0, FL_C);
        self.regs[r] = self.regs[r] << 1;
        self.set_flag(self.regs[r] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn sra_8(&mut self, r: usize) {
        self.set_flag(false, FL_N | FL_H);
        self.set_flag(self.regs[r] & 0x01 != 0, FL_C);
        self.regs[r] = (self.regs[r] >> 1) | (self.regs[r] & 0x80);
        self.set_flag(self.regs[r] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn swap_8(&mut self, r: usize) {
        self.set_flag(false, FL_N | FL_H | FL_C);
        self.regs[r] = (self.regs[r] >> 4) | (self.regs[r] << 4);
        self.set_flag(self.regs[r] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn srl_8(&mut self, r: usize) {
        self.set_flag(false, FL_N | FL_H);
        self.set_flag(self.regs[r] & 0x01 != 0, FL_C);
        self.regs[r] = self.regs[r] >> 1;
        self.set_flag(self.regs[r] == 0, FL_Z);
        self.clk += 4;
    }

    pub fn bitnum_8(&mut self, bit: u8, r: usize) {
        self.set_flag(false, FL_N);
        self.set_flag(true, FL_H);
        self.set_flag(self.regs[r] & (1 << bit) == 0, FL_Z);
        self.clk += 4;
    }

    pub fn bitnum_phl(&mut self, gb_mem: &mem::Mem, bit: u8) {
        self.clk += 4;
        self.set_flag(false, FL_N);
        self.set_flag(true, FL_H);
        self.set_flag((gb_mem.read(self.get_hilo(H, L)) & (1 << bit)) == 0, FL_Z);
        self.clk += 4;
    }

    pub fn resnum_8(&mut self, bit: u8, r: usize) {
        self.regs[r] &= !(1 << bit);
        self.clk += 4;
    }

    pub fn resnum_phl(&mut self, gb_mem: &mut mem::Mem, bit: u8) {
        self.clk += 4;
        let tmp = gb_mem.read(self.get_hilo(H, L));
        self.clk += 4;
        gb_mem.write(self.get_hilo(H, L), tmp & (!(1 << bit)));
        self.clk += 4;
    }

    pub fn setnum_8(&mut self, bit: u8, r: usize) {
        self.regs[r] |= 1 << bit;
        self.clk += 4;
    }

    pub fn setnum_phl(&mut self, gb_mem: &mut mem::Mem, bit: u8) {
        self.clk += 4;
        let tmp = gb_mem.read(self.get_hilo(H, L));
        self.clk += 4;
        gb_mem.write(self.get_hilo(H, L), tmp | (1 << bit));
        self.clk += 4;
    }

    pub fn undef(&mut self, opcode: u8) {
        self.clk += 4;
        self.pc -= 1;
        println!("unimplemented {:#x}, PC: {:#x}", opcode, self.pc);
    }
}

impl Default for Cpu {
    fn default() -> Cpu {
        let mut gb_cpu = Cpu {
            regs: [0; 8],
            sp: 0xFFFE,
            pc: 0x100,
            ime: 0,
            alt_c: 0,
            halt: 0,
            stop: 0,
            clk: 0,
        };
        gb_cpu.set_hilo(A, F, 0x01B0);
        gb_cpu.set_hilo(B, C, 0x0013);
        gb_cpu.set_hilo(D, E, 0x00D8);
        gb_cpu.set_hilo(H, L, 0x014D);
    
        return gb_cpu;
    }
}

pub fn cpu_cycle(gb_cpu: &mut Cpu, gb_mem: &mut mem::Mem) {
    let (int_e, int_f) = (gb_mem.read(PINT_E), gb_mem.read(PINT_F));
    if gb_cpu.halt != 0 && (int_e & int_f) != 0 {
        gb_cpu.halt = 0;
        gb_cpu.clk += 4;
    }
    if gb_cpu.ime == 1 && (int_e & int_f) != 0 {
        gb_cpu.ime = 0;

        let n = (int_e & int_f).trailing_zeros();
        if n < 5 {
            gb_mem.write(PINT_F, int_f & !(1 << n));

            gb_cpu.sp = (gb_cpu.sp as i32 - 1) as u16;
            gb_mem.write(gb_cpu.sp, (gb_cpu.pc >> 8) as u8);
            gb_cpu.sp = (gb_cpu.sp as i32 - 1) as u16;
            gb_mem.write(gb_cpu.sp, gb_cpu.pc as u8);
            gb_cpu.pc = 0x40 | ((n as u16) << 3);
            gb_cpu.clk += 20;
            return;
        }
    }

    if gb_cpu.ime == 2 {
        gb_cpu.ime = 1;
    }
    
    if gb_cpu.ime == 3 {
        gb_cpu.ime = 0;
    }

    if gb_cpu.halt == 1 {
        gb_cpu.clk += 4;
        return;
    }

    let mut opcode: u8 = gb_mem.read(gb_cpu.pc);
    /*println!("OP: {:#X} PC: {:#X}, AF: {:#X}, BC: {:#X}, DE: {:#X}, HL: {:#X}, SP: {:#X}", 
        opcode, gb_cpu.pc, gb_cpu.get_hilo(A, F), gb_cpu.get_hilo(B, C), gb_cpu.get_hilo(D, E), gb_cpu.get_hilo(H, L),
        gb_cpu.sp
    );*/
    gb_cpu.pc = (gb_cpu.pc as u32 + 1) as u16;
    match opcode {
        0x00 => // NOP - 1
            gb_cpu.clk += 4,
        0x01 => // LD BC,nnnn - 3
            gb_cpu.ld_16(gb_mem, B, C),
        0x02 => // LD [BC],A - 2
            gb_cpu.ld_p16_8(gb_mem, B, C, A),
        0x03 => // INC BC - 2
            gb_cpu.inc_16(B, C),
        0x04 => // INC B - 1
            gb_cpu.inc_8(B),
        0x05 => // DEC B - 1
            gb_cpu.dec_8(B),
        0x06 => // LD B,n - 2
            gb_cpu.ld_8(gb_mem, B),
        0x07 => // RLCA - 1
        {
            gb_cpu.set_flag(false, FL_N | FL_H | FL_Z);
            gb_cpu.set_flag((gb_cpu.regs[A] & 0x80) != 0, FL_C);
            gb_cpu.regs[A] = (gb_cpu.regs[A] << 1) | gb_cpu.get_flag(FL_C) as u8;
            gb_cpu.clk += 4;
        },
        0x08 => // LD [nnnn],SP - 5
        {
            gb_cpu.clk += 4;
            let mut tmp: u32 = gb_mem.read(gb_cpu.pc) as u32;
            gb_cpu.clk += 4;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            tmp |= (gb_mem.read(gb_cpu.pc) as u32) << 8;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.clk += 4;
            gb_mem.write(tmp as u16, gb_cpu.sp as u8);
            tmp += 1;
            gb_cpu.clk += 4;
            gb_mem.write(tmp as u16, (gb_cpu.sp >> 8) as u8);
            gb_cpu.clk += 4;
        },
        0x09 => // ADD HL,BC - 2
            gb_cpu.add_hl_16(B, C),
        0x0A => // LD A,[BC] - 2
            gb_cpu.ld_8_p16(gb_mem, A, B, C),
        0x0B => // DEC BC - 2
            gb_cpu.dec_16(B, C),
        0x0C => // INC C - 1
            gb_cpu.inc_8(C),
        0x0D => // DEC C - 1
            gb_cpu.dec_8(C),
        0x0E => // LD C,nn - 2
            gb_cpu.ld_8(gb_mem, C),
        0x0F => // RRCA - 1
        {
            gb_cpu.set_flag(false, FL_N | FL_H | FL_Z);
            gb_cpu.set_flag(gb_cpu.regs[A] & 0x01 != 0, FL_C);
            gb_cpu.regs[A] = (gb_cpu.regs[A] >> 1) | ((gb_cpu.get_flag(FL_C) as u8) << 7);
            gb_cpu.clk += 4;
        },
        0x10 => // STOP - 1*
        {
            gb_cpu.clk += 4;
            if gb_mem.read(gb_cpu.pc) != 0 {
                println!("BAD STOP!");
            }
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.clk += 4;
            gb_cpu.stop = 1;
            return;
        },
        0x11 => // LD DE,nnnn - 3
            gb_cpu.ld_16(gb_mem, D, E),
        0x12 => // LD [DE],A - 2
            gb_cpu.ld_p16_8(gb_mem, D, E, A),
        0x13 => // INC DE - 2
            gb_cpu.inc_16(D, E),
        0x14 => // INC D - 1
            gb_cpu.inc_8(D),
        0x15 => // DEC D - 1
            gb_cpu.dec_8(D),
        0x16 => // LD D,nn - 2
            gb_cpu.ld_8(gb_mem, D),
        0x17 => // RLA - 1
        {
            gb_cpu.set_flag(false, FL_N | FL_H | FL_Z);
            let tmp: u32 = gb_cpu.get_flag(FL_C) as u32;// Old carry flag
            gb_cpu.set_flag(gb_cpu.regs[A] & 0x80 != 0, FL_C);
            gb_cpu.regs[A] = (gb_cpu.regs[A] << 1) | tmp as u8;
            gb_cpu.clk += 4;
        },
        0x18 => // JR nn - 3
        {
            gb_cpu.jr_addr8(gb_mem, true);
        }
        0x19 => // ADD HL,DE - 2
            gb_cpu.add_hl_16(D, E),
        0x1A => // LD A,[DE] - 2
            gb_cpu.ld_8_p16(gb_mem, A, D, E),
        0x1B => // DEC DE - 2
            gb_cpu.dec_16(D, E),
        0x1C => // INC E - 1
            gb_cpu.inc_8(E),
        0x1D => // DEC E - 1
            gb_cpu.dec_8(E),
        0x1E => // LD E,nn - 2
            gb_cpu.ld_8(gb_mem, E),
        0x1F => // RRA - 1
        {
            gb_cpu.set_flag(false, FL_N | FL_H | FL_Z);
            let tmp: u32 = gb_cpu.get_flag(FL_C) as u32; // Old carry flag
            gb_cpu.set_flag((gb_cpu.regs[A] & 0x01) != 0, FL_C);
            gb_cpu.regs[A] = (gb_cpu.regs[A] >> 1) | (tmp << 7) as u8;
            gb_cpu.clk += 4;
        }
        0x20 => // JR NZ,nn - 3/2
            gb_cpu.jr_addr8(gb_mem, !gb_cpu.get_flag(FL_Z)),
        0x21 => // LD HL,nnnn - 3
            gb_cpu.ld_16(gb_mem, H, L),
        0x22 => // LD [HL+],A - 2
        {
            gb_cpu.clk += 4;
            gb_mem.write(gb_cpu.get_hilo(H, L), gb_cpu.regs[A]);
            gb_cpu.set_hilo(H, L, (gb_cpu.get_hilo(H, L) as u32 + 1) as u16);
            gb_cpu.clk += 4;
        },
        0x23 => // INC HL - 2
            gb_cpu.inc_16(H, L),
        0x24 => // INC H - 1
            gb_cpu.inc_8(H),
        0x25 => // DEC H - 1
            gb_cpu.dec_8(H),
        0x26 => // LD H,nn - 2
            gb_cpu.ld_8(gb_mem, H),
        0x27 => // DAA - 1
        {
            let tmp: u32 = ((gb_cpu.regs[A] as u32) << 4) | ((((gb_cpu.regs[F] as u32) >> 4) & 7) << 1);
            gb_cpu.regs[A] = DAA_TABLE[tmp as usize];
            gb_cpu.regs[F] = DAA_TABLE[tmp as usize + 1];
            gb_cpu.clk += 4;
        },
        0x28 => // JR Z,nn - 3/2
            gb_cpu.jr_addr8(gb_mem, gb_cpu.get_flag(FL_Z)),
        0x29 => // ADD HL,HL - 2
        {
            gb_cpu.set_flag(false, FL_N);
            gb_cpu.set_flag((gb_cpu.get_hilo(H, L) & 0x8000) != 0, FL_C);
            gb_cpu.set_flag((gb_cpu.get_hilo(H, L) & 0x0800) != 0, FL_H);
            gb_cpu.set_hilo(H, L, ((gb_cpu.get_hilo(H, L) as u32) << 1) as u16);
            gb_cpu.clk += 8;
        }
        0x2A => // LD A,[HL+] - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.regs[A] = gb_mem.read(gb_cpu.get_hilo(H, L));
            gb_cpu.set_hilo(H, L, (gb_cpu.get_hilo(H, L) as u32 + 1) as u16);
            gb_cpu.clk += 4;
        }
        0x2B => // DEC HL - 2
            gb_cpu.dec_16(H, L),
        0x2C => // INC L - 1
            gb_cpu.inc_8(L),
        0x2D => // DEC L - 1
            gb_cpu.dec_8(L),
        0x2E => // LD L,nn - 2
            gb_cpu.ld_8(gb_mem, L),
        0x2F => // CPL - 1
        {
            gb_cpu.set_flag(true, FL_N | FL_H);
            gb_cpu.regs[A] = !(gb_cpu.regs[A]);
            gb_cpu.clk += 4;
        }
        0x30 => // JR NC,nn - 3/2
            gb_cpu.jr_addr8(gb_mem, !(gb_cpu.get_flag(FL_C))),
        0x31 => // LD SP,nnnn - 3
        {
            gb_cpu.clk += 4;
            let tmp = gb_mem.read(gb_cpu.pc);
            gb_cpu.clk += 4;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            let tmp2 = gb_mem.read(gb_cpu.pc);
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.clk += 4;
            gb_cpu.sp = (tmp2 as u16) << 8 | tmp as u16;
        }
        0x32 => // LD [HL-],A - 2
        {
            gb_cpu.clk += 4;
            gb_mem.write(gb_cpu.get_hilo(H, L), gb_cpu.regs[A]);
            gb_cpu.set_hilo(H, L, (gb_cpu.get_hilo(H, L) as i32 - 1) as u16);
            gb_cpu.clk += 4;
        }
        0x33 => // INC SP - 2
        {
            gb_cpu.sp = gb_cpu.sp.wrapping_add(1);
            gb_cpu.clk += 8;
        }
        0x34 => // INC [HL] - 3
        {
            gb_cpu.clk += 4;
            let mut tmp: u32 = gb_mem.read(gb_cpu.get_hilo(H, L)) as u32;
            gb_cpu.clk += 4;
            gb_cpu.set_flag(false, FL_N);
            gb_cpu.set_flag((tmp & 0xF) == 0xF, FL_H);
            tmp = (tmp + 1) & 0xFF;
            gb_cpu.set_flag(tmp == 0, FL_Z);
            gb_mem.write(gb_cpu.get_hilo(H, L), tmp as u8);
            gb_cpu.clk += 4;
        }
        0x35 => // DEC [HL] - 3
        {
            gb_cpu.clk += 4;
            let mut tmp: u32 = gb_mem.read(gb_cpu.get_hilo(H, L)) as u32;
            gb_cpu.clk += 4;
            gb_cpu.set_flag(true, FL_N);
            gb_cpu.set_flag((tmp & 0xF) == 0x0, FL_H);
            tmp = tmp.wrapping_sub(1);
            gb_cpu.set_flag(tmp == 0, FL_Z);
            gb_mem.write(gb_cpu.get_hilo(H, L), tmp as u8);
            gb_cpu.clk += 4;
        }
        0x36 => // LD [HL],n - 3
        {
            gb_cpu.clk += 4;
            let tmp: u32 = gb_mem.read(gb_cpu.pc) as u32;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.clk += 4;
            gb_mem.write(gb_cpu.get_hilo(H, L), tmp as u8);
            gb_cpu.clk += 4;
        }
        0x37 => // SCF - 1
        {
            gb_cpu.set_flag(false, FL_N | FL_H);
            gb_cpu.set_flag(true, FL_C);
            gb_cpu.clk += 4;
        }
        0x38 => // JR C,nn - 3/2
            gb_cpu.jr_addr8(gb_mem, gb_cpu.get_flag(FL_C)),
        0x39 => // ADD HL,SP - 2
        {
            gb_cpu.set_flag(false, FL_N);
            let tmp: u32 = gb_cpu.get_hilo(H, L) as u32 + gb_cpu.sp as u32;
            gb_cpu.set_flag(tmp > 0xFFFF, FL_C);
            gb_cpu.set_flag((gb_cpu.get_hilo(H, L) & 0x0FFF) + (gb_cpu.sp & 0x0FFF) > 0x0FFF, FL_H);
            gb_cpu.set_hilo(H, L, tmp as u16);
            gb_cpu.clk += 8
        }
        0x3A => // LD A,[HL-] - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.regs[A] = gb_mem.read(gb_cpu.get_hilo(H, L));
            gb_cpu.set_hilo(H, L, (gb_cpu.get_hilo(H, L) as i32 - 1) as u16);
            gb_cpu.clk += 4;
        }
        0x3B => // DEC SP - 2
        {
            gb_cpu.sp = (gb_cpu.sp as i32 - 1) as u16;
            gb_cpu.clk += 8;
        }
        0x3C => // INC A - 1
            gb_cpu.inc_8(A),
        0x3D => // DEC A - 1
            gb_cpu.dec_8(A),
        0x3E => // LD A,n - 2
            gb_cpu.ld_8(gb_mem, A),
        0x3F => // CCF - 1
        {
            gb_cpu.set_flag(false, FL_N | FL_H);
            gb_cpu.set_flag(!gb_cpu.get_flag(FL_C), FL_C);
            gb_cpu.clk += 4;
        }
        0x40 => // LD B,B - 1
            gb_cpu.clk += 4,
        0x41 => // LD B,C - 1
        {
            gb_cpu.regs[B] = gb_cpu.regs[C];
            gb_cpu.clk += 4;
        }
        0x42 => // LD B,D - 1
        {
            gb_cpu.regs[B] = gb_cpu.regs[D];
            gb_cpu.clk += 4;
        }
        0x43 => // LD B,E - 1
        {
            gb_cpu.regs[B] = gb_cpu.regs[E];
            gb_cpu.clk += 4;
        }
        0x44 => // LD B,H - 1
        {
            gb_cpu.regs[B] = gb_cpu.regs[H];
            gb_cpu.clk += 4;
        }
        0x45 => // LD B,L - 1
        {
            gb_cpu.regs[B] = gb_cpu.regs[L];
            gb_cpu.clk += 4;
        }
        0x46 => // LD B,[HL] - 2
            gb_cpu.ld_8_p16(gb_mem, B, H, L),
        0x47 => // LD B,A - 1
        {
            gb_cpu.regs[B] = gb_cpu.regs[A];
            gb_cpu.clk += 4;
        }
        0x48 => // LD C,B - 1
        {
            gb_cpu.regs[C] = gb_cpu.regs[B];
            gb_cpu.clk += 4;
        }
        0x49 => // LD C,C - 1
            gb_cpu.clk += 4,
        0x4A => // LD C,D - 1
        {
            gb_cpu.regs[C] = gb_cpu.regs[D];
            gb_cpu.clk += 4;
        }
        0x4B => // LD C,E - 1
        {
            gb_cpu.regs[C] = gb_cpu.regs[E];
            gb_cpu.clk += 4;
        }
        0x4C => // LD C,H - 1
        {
            gb_cpu.regs[C] = gb_cpu.regs[H];
            gb_cpu.clk += 4;
        }
        0x4D => // LD C,L - 1
        {
            gb_cpu.regs[C] = gb_cpu.regs[L];
            gb_cpu.clk += 4;
        }
        0x4E => // LD C,[HL] - 2
            gb_cpu.ld_8_p16(gb_mem, C, H, L),
        0x4F => // LD C,A - 1
        {
            gb_cpu.regs[C] = gb_cpu.regs[A];
            gb_cpu.clk += 4;
        }
        0x50 => // LD D,B - 1
        {
            gb_cpu.regs[D] = gb_cpu.regs[B];
            gb_cpu.clk += 4;
        }
        0x51 => // LD D,C - 1
        {
            gb_cpu.regs[D] = gb_cpu.regs[C];
            gb_cpu.clk += 4;
        }
        0x52 => // LD D,D - 1
            gb_cpu.clk += 4,
        0x53 => // LD D,E - 1
        {
            gb_cpu.regs[D] = gb_cpu.regs[E];
            gb_cpu.clk += 4;
        }
        0x54 => // LD D,H - 1
        {
            gb_cpu.regs[D] = gb_cpu.regs[H];
            gb_cpu.clk += 4;
        }
        0x55 => // LD D,L - 1
        {
            gb_cpu.regs[D] = gb_cpu.regs[L];
            gb_cpu.clk += 4;
        }
        0x56 => // LD D,[HL] - 2
            gb_cpu.ld_8_p16(gb_mem, D, H, L),
        0x57 => // LD D,A - 1
        {
            gb_cpu.regs[D] = gb_cpu.regs[A];
            gb_cpu.clk += 4;
        }
        0x58 => // LD E,B - 1
        {
            gb_cpu.regs[E] = gb_cpu.regs[B];
            gb_cpu.clk += 4;
        }
        0x59 => // LD E,C - 1
        {
            gb_cpu.regs[E] = gb_cpu.regs[C];
            gb_cpu.clk += 4;
        }
        0x5A => // LD E,D - 1
        {
            gb_cpu.regs[E] = gb_cpu.regs[D];
            gb_cpu.clk += 4;
        }
        0x5B => // LD E,E - 1
            gb_cpu.clk += 4,
        0x5C => // LD E,H - 1
        {
            gb_cpu.regs[E] = gb_cpu.regs[H];
            gb_cpu.clk += 4;
        }
        0x5D => // LD E,L - 1
        {
            gb_cpu.regs[E] = gb_cpu.regs[L];
            gb_cpu.clk += 4;
        }
        0x5E => // LD E,[HL] - 2
            gb_cpu.ld_8_p16(gb_mem, E, H, L),
        0x5F => // LD E,A - 1
        {
            gb_cpu.regs[E] = gb_cpu.regs[A];
            gb_cpu.clk += 4;
        }
        0x60 => // LD H,B - 1
        {
            gb_cpu.regs[H] = gb_cpu.regs[B];
            gb_cpu.clk += 4;
        }
        0x61 => // LD H,C - 1
        {
            gb_cpu.regs[H] = gb_cpu.regs[C];
            gb_cpu.clk += 4;
        }
        0x62 => // LD H,D - 1
        {
            gb_cpu.regs[H] = gb_cpu.regs[D];
            gb_cpu.clk += 4;
        }
        0x63 => // LD H,E - 1
        {
            gb_cpu.regs[H] = gb_cpu.regs[E];
            gb_cpu.clk += 4;
        }
        0x64 => // LD H,H - 1
            gb_cpu.clk += 4,
        0x65 => // LD H,L - 1
        {
            gb_cpu.regs[H] = gb_cpu.regs[L];
            gb_cpu.clk += 4;
        }
        0x66 => // LD H,[HL] - 2
            gb_cpu.ld_8_p16(gb_mem, H, H, L),
        0x67 => // LD H,A - 1
        {
            gb_cpu.regs[H] = gb_cpu.regs[A];
            gb_cpu.clk += 4;
        }
        0x68 => // LD L,B - 1
        {
            gb_cpu.regs[L] = gb_cpu.regs[B];
            gb_cpu.clk += 4;
        }
        0x69 => // LD L,C - 1
        {
            gb_cpu.regs[L] = gb_cpu.regs[C];
            gb_cpu.clk += 4;
        }
        0x6A => // LD L,D - 1
        {
            gb_cpu.regs[L] = gb_cpu.regs[D];
            gb_cpu.clk += 4;
        }
        0x6B => // LD L,E - 1
        {
            gb_cpu.regs[L] = gb_cpu.regs[E];
            gb_cpu.clk += 4;
        }
        0x6C => // LD L,H - 1
        {
            gb_cpu.regs[L] = gb_cpu.regs[H];
            gb_cpu.clk += 4;
        }
        0x6D => // LD L,L - 1
            gb_cpu.clk += 4,
        0x6E => // LD L,[HL] - 2
            gb_cpu.ld_8_p16(gb_mem, L, H, L),
        0x6F => // LD L,A - 1
        {
            gb_cpu.regs[L] = gb_cpu.regs[A];
            gb_cpu.clk += 4;
        }
        0x70 => // LD [HL],B - 2
            gb_cpu.ld_p16_8(gb_mem, H, L, B),
        0x71 => // LD [HL],C - 2
            gb_cpu.ld_p16_8(gb_mem, H, L, C),
        0x72 => // LD [HL],D - 2
            gb_cpu.ld_p16_8(gb_mem, H, L, D),
        0x73 => // LD [HL],E - 2
            gb_cpu.ld_p16_8(gb_mem, H, L, E),
        0x74 => // LD [HL],H - 2
            gb_cpu.ld_p16_8(gb_mem, H, L, H),
        0x75 => // LD [HL],L - 2
            gb_cpu.ld_p16_8(gb_mem, H, L, L),
        0x76 => // HALT - 1*
        {
            gb_cpu.clk += 4;
            gb_cpu.halt = 1;
        }
        0x77 => // LD [HL],A - 2
            gb_cpu.ld_p16_8(gb_mem, H, L, A),
        0x78 => // LD A,B - 1
        {
            gb_cpu.regs[A] = gb_cpu.regs[B];
            gb_cpu.clk += 4;
        }
        0x79 => // LD A,C - 1
        {
            gb_cpu.regs[A] = gb_cpu.regs[C];
            gb_cpu.clk += 4;
        }
        0x7A => // LD A,D - 1
        {
            gb_cpu.regs[A] = gb_cpu.regs[D];
            gb_cpu.clk += 4;
        }
        0x7B => // LD A,E - 1
        {
            gb_cpu.regs[A] = gb_cpu.regs[E];
            gb_cpu.clk += 4;
        }
        0x7C => // LD A,H - 1
        {
            gb_cpu.regs[A] = gb_cpu.regs[H];
            gb_cpu.clk += 4;
        }
        0x7D => // LD A,L - 1
        {
            gb_cpu.regs[A] = gb_cpu.regs[L];
            gb_cpu.clk += 4;
        }
        0x7E => // LD A,[HL] - 2
            gb_cpu.ld_8_p16(gb_mem, A, H, L),
        0x7F => // LD A,A - 1
            gb_cpu.clk += 4,
        0x80 => // ADD A,B - 1
            gb_cpu.add_a_8(B),
        0x81 => // ADD A,C - 1
            gb_cpu.add_a_8(C),
        0x82 => // ADD A,D - 1
            gb_cpu.add_a_8(D),
        0x83 => // ADD A,E - 1
            gb_cpu.add_a_8(E),
        0x84 => // ADD A,H - 1
            gb_cpu.add_a_8(H),
        0x85 => // ADD A,L - 1
            gb_cpu.add_a_8(L),
        0x86 => // ADD A,[HL] - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.set_flag(false, FL_N);
            let tmp: u32 = gb_cpu.regs[A] as u32;
            let tmp2: u32 =  gb_mem.read(gb_cpu.get_hilo(H, L)) as u32;
            gb_cpu.set_flag(((tmp & 0xF) + (tmp2 & 0xF)) > 0xF, FL_H);
            gb_cpu.regs[A] = gb_cpu.regs[A].wrapping_add(tmp2 as u8);
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.set_flag(tmp > gb_cpu.regs[A] as u32, FL_C);
            gb_cpu.clk += 4;
        }
        0x87 => // ADD A,A - 1
        {
            gb_cpu.set_flag(false, FL_N);
            gb_cpu.set_flag((gb_cpu.regs[A] & (1 << 3)) != 0, FL_H);
            gb_cpu.set_flag((gb_cpu.regs[A]) & (1 << 7) != 0, FL_C);
            gb_cpu.regs[A] = gb_cpu.regs[A].wrapping_mul(2);
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.clk += 4;
        }
        0x88 => // ADC A,B - 1
            gb_cpu.adc_a_8(B),
        0x89 => // ADC A,C - 1
            gb_cpu.adc_a_8(C),
        0x8A => // ADC A,D - 1
            gb_cpu.adc_a_8(D),
        0x8B => // ADC A,E - 1
            gb_cpu.adc_a_8(E),
        0x8C => // ADC A,H - 1
            gb_cpu.adc_a_8(H),
        0x8D => // ADC A,L - 1
            gb_cpu.adc_a_8(L),
        0x8E => // ADC A,[HL] - 2
        {
            gb_cpu.clk += 4;
            let n = gb_mem.read(gb_cpu.get_hilo(H, L));
            gb_cpu.set_flag(false, FL_N);
            let c = gb_cpu.get_flag(FL_C) as u8;
            let h = ((gb_cpu.regs[A] & 0xF) + (n & 0xF) + c) & 0x10;
            let tmp = gb_cpu.regs[A] as u32 + n as u32 + c as u32;
            let c = tmp & 0x100;
            gb_cpu.regs[A] =  tmp as u8;
            gb_cpu.regs[F] = 0;
            if gb_cpu.regs[A] == 0 {
                gb_cpu.regs[F] |= 0x80
            }
            if h != 0 {
                gb_cpu.regs[F] |= 0x20;
            }
            if c != 0 {
                gb_cpu.regs[F] |= 0x10;
            }
            gb_cpu.clk += 4;
            gb_cpu.clk += 4;
        }
        0x8F => // ADC A,A - 1
        {
            gb_cpu.adc_a_8(A)
        }
        0x90 => // SUB A,B - 1
            gb_cpu.sub_a_8(B),
        0x91 => // SUB A,C - 1
            gb_cpu.sub_a_8(C),
        0x92 => // SUB A,D - 1
            gb_cpu.sub_a_8(D),
        0x93 => // SUB A,E - 1
            gb_cpu.sub_a_8(E),
        0x94 => // SUB A,H - 1
            gb_cpu.sub_a_8(H),
        0x95 => // SUB A,L - 1
            gb_cpu.sub_a_8(L),
        0x96 => // SUB A,[HL] - 2
        {
            gb_cpu.clk += 4;
            let tmp: u32 =  gb_mem.read(gb_cpu.get_hilo(H, L)) as u32;
            gb_cpu.regs[F] = FL_N;
            gb_cpu.set_flag((gb_cpu.regs[A] & 0xF) < (tmp & 0xF) as u8, FL_H);
            gb_cpu.set_flag(gb_cpu.regs[A] < tmp as u8, FL_C);
            gb_cpu.regs[A] = gb_cpu.regs[A].wrapping_sub(tmp as u8);
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.clk += 4;
        }
        0x97 => // SUB A,A - 1
        {
            gb_cpu.regs[F] = FL_N | FL_Z;
            gb_cpu.regs[A] = 0;
            gb_cpu.clk += 4;
        }
        0x98 => // SBC A,B - 1
            gb_cpu.sbc_a_8(B),
        0x99 => // SBC A,C - 1
            gb_cpu.sbc_a_8(C),
        0x9A => // SBC A,D - 1
            gb_cpu.sbc_a_8(D),
        0x9B => // SBC A,E - 1
            gb_cpu.sbc_a_8(E),
        0x9C => // SBC A,H - 1
            gb_cpu.sbc_a_8(H),
        0x9D => // SBC A,L - 1
            gb_cpu.sbc_a_8(L),
        0x9E => // SBC A,[HL] - 2
        {
            let n = gb_mem.read(gb_cpu.get_hilo(H, L));
            let carr = gb_cpu.get_flag(FL_C) as u16;
            let c = (gb_cpu.regs[A] as u16) < n as u16 + carr;
            let h = ((gb_cpu.regs[A] & 0xF) as u16) < (n & 0xF) as u16 + carr;
            gb_cpu.regs[A] = (gb_cpu.regs[A] as i32 - n as i32 - carr as i32) as u8;
            let z = gb_cpu.regs[A] == 0;
            gb_cpu.regs[F] = 0x40;
            if z {
                gb_cpu.regs[F] |= 0x80;
            }
            if h {
                gb_cpu.regs[F] |= 0x20;
            }
            if c {
                gb_cpu.regs[F] |= 0x10;
            }
            gb_cpu.clk += 4;
        }
        0x9F => // SBC A,A - 1
        {
            gb_cpu.sbc_a_8(A)
        }
        0xA0 => // AND A,B - 1
            gb_cpu.and_a_8(B),
        0xA1 => // AND A,C - 1
            gb_cpu.and_a_8(C),
        0xA2 => // AND A,D - 1
            gb_cpu.and_a_8(D),
        0xA3 => // AND A,E - 1
            gb_cpu.and_a_8(E),
        0xA4 => // AND A,H - 1
            gb_cpu.and_a_8(H),
        0xA5 => // AND A,L - 1
            gb_cpu.and_a_8(L),
        0xA6 => // AND A,[HL] - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.set_flag(true, FL_H);
            gb_cpu.set_flag(false, FL_N | FL_C);
            gb_cpu.regs[A] &=  gb_mem.read(gb_cpu.get_hilo(H, L));
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.clk += 4;
        }
        0xA7 => // AND A,A - 1
        {
            gb_cpu.set_flag(true, FL_H);
            gb_cpu.set_flag(false, FL_N | FL_C);
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.clk += 4;
        }
        0xA8 => // XOR A,B - 1
            gb_cpu.xor_a_8(B),
        0xA9 => // XOR A,C - 1
            gb_cpu.xor_a_8(C),
        0xAA => // XOR A,D - 1
            gb_cpu.xor_a_8(D),
        0xAB => // XOR A,E - 1
            gb_cpu.xor_a_8(E),
        0xAC => // XOR A,H - 1
            gb_cpu.xor_a_8(H),
        0xAD => // XOR A,L - 1
            gb_cpu.xor_a_8(L),
        0xAE => // XOR A,[HL] - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.set_flag(false, FL_N | FL_C | FL_H);
            gb_cpu.regs[A] ^=  gb_mem.read(gb_cpu.get_hilo(H, L));
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.clk += 4;
        }
        0xAF => // XOR A,A - 1
        {
            gb_cpu.set_hilo(A, F, FL_Z as u16);
            gb_cpu.clk += 4;
        }
        0xB0 => // OR A,B - 1
            gb_cpu.or_a_8(B),
        0xB1 => // OR A,C - 1
            gb_cpu.or_a_8(C),
        0xB2 => // OR A,D - 1
            gb_cpu.or_a_8(D),
        0xB3 => // OR A,E - 1
            gb_cpu.or_a_8(E),
        0xB4 => // OR A,H - 1
            gb_cpu.or_a_8(H),
        0xB5 => // OR A,L - 1
            gb_cpu.or_a_8(L),
        0xB6 => // OR A,[HL] - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.set_flag(false, FL_N | FL_C | FL_H);
            gb_cpu.regs[A] |=  gb_mem.read(gb_cpu.get_hilo(H, L));
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.clk += 4;
        }
        0xB7 => // OR A,A - 1
        {
            gb_cpu.set_flag(false, FL_N | FL_C | FL_H);
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.clk += 4;
        }
        0xB8 => // CP A,B - 1
            gb_cpu.cp_a_8(B),
        0xB9 => // CP A,C - 1
            gb_cpu.cp_a_8(C),
        0xBA => // CP A,D - 1
            gb_cpu.cp_a_8(D),
        0xBB => // CP A,E - 1
            gb_cpu.cp_a_8(E),
        0xBC => // CP A,H - 1
            gb_cpu.cp_a_8(H),
        0xBD => // CP A,L - 1
            gb_cpu.cp_a_8(L),
        0xBE => // CP A,[HL] - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.set_flag(true, FL_N);
            let tmp: u32 =  gb_mem.read(gb_cpu.get_hilo(H, L)) as u32;
            gb_cpu.set_flag((gb_cpu.regs[A] & 0xF) < (tmp & 0xF) as u8, FL_H);
            gb_cpu.set_flag((gb_cpu.regs[A] as u32) < tmp, FL_C);
            gb_cpu.set_flag(gb_cpu.regs[A] as u32 == tmp, FL_Z);
            gb_cpu.clk += 4;
        }
        0xBF => // CP A,A - 1
        {
            gb_cpu.set_flag(true, FL_N | FL_Z);
            gb_cpu.set_flag(false, FL_H | FL_C);
            gb_cpu.clk += 4;
        }
        0xC0 => // RET NZ - 5/2
            gb_cpu.ret(gb_mem, !gb_cpu.get_flag(FL_Z)),
        0xC1 => // POP BC - 3
            gb_cpu.pop_16(gb_mem, B, C),
        0xC2 => // JP NZ,nnnn - 4/3
            gb_cpu.jp_addr16(gb_mem, !gb_cpu.get_flag(FL_Z)),
        0xC3 => // JP nnnn - 4
        {
            gb_cpu.jp_addr16(gb_mem, true);
        }
        0xC4 => // CALL NZ,nnnn - 6/3
            gb_cpu.call_addr16(gb_mem, !gb_cpu.get_flag(FL_Z)),
        0xC5 => // PUSH BC - 4
            gb_cpu.push_16(gb_mem, B, C),
        0xC6 => // ADD A,nn - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.set_flag(false, FL_N);
            let tmp: u32 = gb_cpu.regs[A] as u32;
            let tmp2: u32 = gb_mem.read(gb_cpu.pc) as u32;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.set_flag(((tmp & 0xF) + (tmp2 & 0xF)) > 0xF, FL_H);
            gb_cpu.regs[A] = gb_cpu.regs[A].wrapping_add(tmp2 as u8);
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.set_flag(tmp > gb_cpu.regs[A] as u32, FL_C);
            gb_cpu.clk += 4;
        }
        0xC7 => // RST 0x0000 - 4
            gb_cpu.rst_addr16(gb_mem, 0x0000),
        0xC8 => // RET Z - 5/2
            gb_cpu.ret(gb_mem, gb_cpu.get_flag(FL_Z)),
        0xC9 => // RET - 4
        {
            gb_cpu.ret(gb_mem, true);
        }
        0xCA => // JP Z,nnnn - 4/3
            gb_cpu.jp_addr16(gb_mem, gb_cpu.get_flag(FL_Z)),
        0xCB =>
        {
            gb_cpu.clk += 4;
            opcode = gb_mem.read(gb_cpu.pc);
            gb_cpu.pc = (gb_cpu.pc as u32 + 1) as u16;
            //println!("CBOP: {:#X} PC: {:#X}", opcode, gb_cpu.pc);

            match opcode {
                0x00 => // RLC B - 2
                    gb_cpu.rlc_8(B),
                0x01 => // RLC C - 2
                    gb_cpu.rlc_8(C),
                0x02 => // RLC D - 2
                    gb_cpu.rlc_8(D),
                0x03 => // RLC E - 2
                    gb_cpu.rlc_8(E),
                0x04 => // RLC H - 2
                    gb_cpu.rlc_8(H),
                0x05 => // RLC L - 2
                    gb_cpu.rlc_8(L),
                0x06 => // RLC [HL] - 4
                {
                    gb_cpu.clk += 4;
                    let mut tmp: u32 =  gb_mem.read(gb_cpu.get_hilo(H, L)) as u32;
                    gb_cpu.clk += 4;
                    gb_cpu.set_flag(false, FL_N | FL_H);
                    gb_cpu.set_flag(tmp & 0x80 != 0, FL_C);
                    tmp = (tmp << 1) | gb_cpu.get_flag(FL_C) as u32;
                    gb_cpu.set_flag(tmp == 0, FL_Z);
                    gb_mem.write(gb_cpu.get_hilo(H, L), tmp as u8);
                    gb_cpu.clk += 4;
                }
                0x07 => // RLC A - 2
                    gb_cpu.rlc_8(A),
                0x08 => // RRC B - 2
                    gb_cpu.rrc_8(B),
                0x09 => // RRC C - 2
                    gb_cpu.rrc_8(C),
                0x0A => // RRC D - 2
                    gb_cpu.rrc_8(D),
                0x0B => // RRC E - 2
                    gb_cpu.rrc_8(E),
                0x0C => // RRC H - 2
                    gb_cpu.rrc_8(H),
                0x0D => // RRC L - 2
                    gb_cpu.rrc_8(L),
                0x0E => // RRC [HL] - 4
                {
                    gb_cpu.clk += 4;
                    let mut tmp: u32 =  gb_mem.read(gb_cpu.get_hilo(H, L)) as u32;
                    gb_cpu.clk += 4;
                    gb_cpu.set_flag(false, FL_N | FL_H);
                    gb_cpu.set_flag(tmp & 0x01 != 0, FL_C);
                    tmp = (tmp >> 1) | ((gb_cpu.get_flag(FL_C) as u32) << 7);
                    gb_cpu.set_flag(tmp == 0, FL_Z);
                    gb_mem.write(gb_cpu.get_hilo(H, L), tmp as u8);
                    gb_cpu.clk += 4;
                }
                0x0F => // RRC A - 2
                    gb_cpu.rrc_8(A),
                0x10 => // RL B - 2
                    gb_cpu.rl_8(B),
                0x11 => // RL C - 2
                    gb_cpu.rl_8(C),
                0x12 => // RL D - 2
                    gb_cpu.rl_8(D),
                0x13 => // RL E - 2
                    gb_cpu.rl_8(E),
                0x14 => // RL H - 2
                    gb_cpu.rl_8(H),
                0x15 => // RL L - 2
                    gb_cpu.rl_8(L),
                0x16 => // RL [HL] - 4
                {
                    let add = gb_cpu.get_hilo(H, L);
                    let c = gb_cpu.get_flag(FL_C) as u32;
                    gb_cpu.regs[F] = 0;
                    let mut hlp = gb_mem.read(add) as u32;
                    gb_cpu.set_flag(hlp & 0x80 != 0, FL_C);
                    gb_mem.write(add, ((hlp << 1) + c) as u8);
                    hlp = gb_mem.read(add) as u32;
                    gb_cpu.set_flag(hlp == 0, FL_Z);
                    gb_cpu.clk += 12;
                }
                0x17 => // RL A - 2
                    gb_cpu.rl_8(A),
                0x18 => // RR B - 2
                    gb_cpu.rr_8(B),
                0x19 => // RR C - 2
                    gb_cpu.rr_8(C),
                0x1A => // RR D - 2
                    gb_cpu.rr_8(D),
                0x1B => // RR E - 2
                    gb_cpu.rr_8(E),
                0x1C => // RR H - 2
                    gb_cpu.rr_8(H),
                0x1D => // RR L - 2
                    gb_cpu.rr_8(L),
                0x1E => // RR [HL] - 4
                {
                    gb_cpu.clk += 4;
                    let mut tmp2: u32 = gb_mem.read(gb_cpu.get_hilo(H, L)) as u32;
                    gb_cpu.clk += 4;
                    gb_cpu.set_flag(false, FL_N | FL_H);
                    let tmp: u32 = gb_cpu.get_flag(FL_C) as u32; // Old carry flag
                    gb_cpu.set_flag(tmp2 & 0x01 != 0, FL_C);
                    tmp2 = (tmp2 >> 1) | (tmp << 7);
                    gb_cpu.set_flag(tmp2 == 0, FL_Z);
                    gb_mem.write(gb_cpu.get_hilo(H, L), tmp2 as u8);
                    gb_cpu.clk += 4;
                }
                0x1F => // RR A - 2
                    gb_cpu.rr_8(A),
                0x20 => // SLA B - 2
                    gb_cpu.sla_8(B),
                0x21 => // SLA C - 2
                    gb_cpu.sla_8(C),
                0x22 => // SLA D - 2
                    gb_cpu.sla_8(D),
                0x23 => // SLA E - 2
                    gb_cpu.sla_8(E),
                0x24 => // SLA H - 2
                    gb_cpu.sla_8(H),
                0x25 => // SLA L - 2
                    gb_cpu.sla_8(L),
                0x26 => // SLA [HL] - 4
                {
                    gb_cpu.clk += 4;
                    let mut tmp: u32 = gb_mem.read(gb_cpu.get_hilo(H, L)) as u32;
                    gb_cpu.clk += 4;
                    gb_cpu.regs[F] = 0;
                    gb_cpu.set_flag(tmp & 0x80 != 0, FL_C);
                    tmp = tmp << 1;
                    gb_mem.write(gb_cpu.get_hilo(H, L), tmp as u8);
                    gb_cpu.set_flag(gb_mem.read(gb_cpu.get_hilo(H, L)) == 0, FL_Z);
                    gb_cpu.clk += 4;
                }
                0x27 => // SLA A - 2
                    gb_cpu.sla_8(A),
                0x28 => // SRA B - 2
                    gb_cpu.sra_8(B),
                0x29 => // SRA C - 2
                    gb_cpu.sra_8(C),
                0x2A => // SRA D - 2
                    gb_cpu.sra_8(D),
                0x2B => // SRA E - 2
                    gb_cpu.sra_8(E),
                0x2C => // SRA H - 2
                    gb_cpu.sra_8(H),
                0x2D => // SRA L - 2
                    gb_cpu.sra_8(L),
                0x2E => // SRA [HL] - 4
                {
                    gb_cpu.clk += 4;
                    let mut tmp: u32 = gb_mem.read(gb_cpu.get_hilo(H, L)) as u32;
                    gb_cpu.clk += 4;
                    gb_cpu.set_flag(false, FL_N | FL_H);
                    gb_cpu.set_flag(tmp & 0x01 != 0, FL_C);
                    tmp = (tmp & 0x80) | (tmp >> 1);
                    gb_cpu.set_flag(tmp == 0, FL_Z);
                    gb_mem.write(gb_cpu.get_hilo(H, L), tmp as u8);
                    gb_cpu.clk += 4;
                }
                0x2F => // SRA A - 2
                    gb_cpu.sra_8(A),
                0x30 => // SWAP B - 2
                    gb_cpu.swap_8(B),
                0x31 => // SWAP C - 2
                    gb_cpu.swap_8(C),
                0x32 => // SWAP D - 2
                    gb_cpu.swap_8(D),
                0x33 => // SWAP E - 2
                    gb_cpu.swap_8(E),
                0x34 => // SWAP H - 2
                    gb_cpu.swap_8(H),
                0x35 => // SWAP L - 2
                    gb_cpu.swap_8(L),
                0x36 => // SWAP [HL] - 4
                {
                    gb_cpu.clk += 4;
                    let mut tmp: u32 = gb_mem.read(gb_cpu.get_hilo(H, L)) as u32;
                    gb_cpu.clk += 4;
                    gb_cpu.set_flag(false, FL_N | FL_H | FL_C);
                    tmp = (tmp >> 4) | (tmp << 4);
                    gb_mem.write(gb_cpu.get_hilo(H, L), tmp as u8);
                    gb_cpu.set_flag(tmp == 0, FL_Z);
                    gb_cpu.clk += 4;
                }
                0x37 => // SWAP A - 2
                    gb_cpu.swap_8(A),
                0x38 => // SRL B - 2
                    gb_cpu.srl_8(B),
                0x39 => // SRL C - 2
                    gb_cpu.srl_8(C),
                0x3A => // SRL D - 2
                    gb_cpu.srl_8(D),
                0x3B => // SRL E - 2
                    gb_cpu.srl_8(E),
                0x3C => // SRL H - 2
                    gb_cpu.srl_8(H),
                0x3D => // SRL L - 2
                    gb_cpu.srl_8(L),
                0x3E => // SRL [HL] - 4
                {
                    gb_cpu.clk += 4;
                    let mut tmp: u32 = gb_mem.read(gb_cpu.get_hilo(H, L)) as u32;
                    gb_cpu.clk += 4;
                    gb_cpu.set_flag(false, FL_N | FL_H);
                    gb_cpu.set_flag(tmp & 0x01 != 0, FL_C);
                    tmp = tmp >> 1;
                    gb_cpu.set_flag(tmp == 0, FL_Z);
                    gb_mem.write(gb_cpu.get_hilo(H, L), tmp as u8);
                    gb_cpu.clk += 4;
                }
                0x3F => // SRL A - 2
                    gb_cpu.srl_8(A),
                0x40 => // BIT 0,B - 2
                    gb_cpu.bitnum_8(0, B),
                0x41 => // BIT 0,C - 2
                    gb_cpu.bitnum_8(0, C),
                0x42 => // BIT 0,D - 2
                    gb_cpu.bitnum_8(0, D),
                0x43 => // BIT 0,E - 2
                    gb_cpu.bitnum_8(0, E),
                0x44 => // BIT 0,H - 2
                    gb_cpu.bitnum_8(0, H),
                0x45 => // BIT 0,L - 2
                    gb_cpu.bitnum_8(0, L),
                0x46 => // BIT 0,[HL] - 3
                    gb_cpu.bitnum_phl(gb_mem, 0),
                0x47 => // BIT 0,A - 2
                    gb_cpu.bitnum_8(0, A),
                0x48 => // BIT 1,B - 2
                    gb_cpu.bitnum_8(1, B),
                0x49 => // BIT 1,C - 2
                    gb_cpu.bitnum_8(1, C),
                0x4A => // BIT 1,D - 2
                    gb_cpu.bitnum_8(1, D),
                0x4B => // BIT 1,E - 2
                    gb_cpu.bitnum_8(1, E),
                0x4C => // BIT 1,H - 2
                    gb_cpu.bitnum_8(1, H),
                0x4D => // BIT 1,L - 2
                    gb_cpu.bitnum_8(1, L),
                0x4E => // BIT 1,[HL] - 3
                    gb_cpu.bitnum_phl(gb_mem, 1),
                0x4F => // BIT 1,A - 2
                    gb_cpu.bitnum_8(1, A),
                0x50 => // BIT 2,B - 2
                    gb_cpu.bitnum_8(2, B),
                0x51 => // BIT 2,C - 2
                    gb_cpu.bitnum_8(2, C),
                0x52 => // BIT 2,D - 2
                    gb_cpu.bitnum_8(2, D),
                0x53 => // BIT 2,E - 2
                    gb_cpu.bitnum_8(2, E),
                0x54 => // BIT 2,H - 2
                    gb_cpu.bitnum_8(2, H),
                0x55 => // BIT 2,L - 2
                    gb_cpu.bitnum_8(2, L),
                0x56 => // BIT 2,[HL] - 3
                    gb_cpu.bitnum_phl(gb_mem, 2),
                0x57 => // BIT 2,A - 2
                    gb_cpu.bitnum_8(2, A),
                0x58 => // BIT 3,B - 2
                    gb_cpu.bitnum_8(3, B),
                0x59 => // BIT 3,C - 2
                    gb_cpu.bitnum_8(3, C),
                0x5A => // BIT 3,D - 2
                    gb_cpu.bitnum_8(3, D),
                0x5B => // BIT 3,E - 2
                    gb_cpu.bitnum_8(3, E),
                0x5C => // BIT 3,H - 2
                    gb_cpu.bitnum_8(3, H),
                0x5D => // BIT 3,L - 2
                    gb_cpu.bitnum_8(3, L),
                0x5E => // BIT 3,[HL] - 3
                    gb_cpu.bitnum_phl(gb_mem, 3),
                0x5F => // BIT 3,A - 2
                    gb_cpu.bitnum_8(3, A),
                0x60 => // BIT 4,B - 2
                    gb_cpu.bitnum_8(4, B),
                0x61 => // BIT 4,C - 2
                    gb_cpu.bitnum_8(4, C),
                0x62 => // BIT 4,D - 2
                    gb_cpu.bitnum_8(4, D),
                0x63 => // BIT 4,E - 2
                    gb_cpu.bitnum_8(4, E),
                0x64 => // BIT 4,H - 2
                    gb_cpu.bitnum_8(4, H),
                0x65 => // BIT 4,L - 2
                    gb_cpu.bitnum_8(4, L),
                0x66 => // BIT 4,[HL] - 3
                    gb_cpu.bitnum_phl(gb_mem, 4),
                0x67 => // BIT 4,A - 2
                    gb_cpu.bitnum_8(4, A),
                0x68 => // BIT 5,B - 2
                    gb_cpu.bitnum_8(5, B),
                0x69 => // BIT 5,C - 2
                    gb_cpu.bitnum_8(5, C),
                0x6A => // BIT 5,D - 2
                    gb_cpu.bitnum_8(5, D),
                0x6B => // BIT 5,E - 2
                    gb_cpu.bitnum_8(5, E),
                0x6C => // BIT 5,H - 2
                    gb_cpu.bitnum_8(5, H),
                0x6D => // BIT 5,L - 2
                    gb_cpu.bitnum_8(5, L),
                0x6E => // BIT 5,[HL] - 3
                    gb_cpu.bitnum_phl(gb_mem, 5),
                0x6F => // BIT 5,A - 2
                    gb_cpu.bitnum_8(5, A),
                0x70 => // BIT 6,B - 2
                    gb_cpu.bitnum_8(6, B),
                0x71 => // BIT 6,C - 2
                    gb_cpu.bitnum_8(6, C),
                0x72 => // BIT 6,D - 2
                    gb_cpu.bitnum_8(6, D),
                0x73 => // BIT 6,E - 2
                    gb_cpu.bitnum_8(6, E),
                0x74 => // BIT 6,H - 2
                    gb_cpu.bitnum_8(6, H),
                0x75 => // BIT 6,L - 2
                    gb_cpu.bitnum_8(6, L),
                0x76 => // BIT 6,[HL] - 3
                    gb_cpu.bitnum_phl(gb_mem, 6),
                0x77 => // BIT 6,A - 2
                    gb_cpu.bitnum_8(6, A),
                0x78 => // BIT 7,B - 2
                    gb_cpu.bitnum_8(7, B),
                0x79 => // BIT 7,C - 2
                    gb_cpu.bitnum_8(7, C),
                0x7A => // BIT 7,D - 2
                    gb_cpu.bitnum_8(7, D),
                0x7B => // BIT 7,E - 2
                    gb_cpu.bitnum_8(7, E),
                0x7C => // BIT 7,H - 2
                    gb_cpu.bitnum_8(7, H),
                0x7D => // BIT 7,L - 2
                    gb_cpu.bitnum_8(7, L),
                0x7E => // BIT 7,[HL] - 3
                    gb_cpu.bitnum_phl(gb_mem, 7),
                0x7F => // BIT 7,A - 2
                    gb_cpu.bitnum_8(7, A),
                0x80 => // RES 0,B - 2
                    gb_cpu.resnum_8(0, B),
                0x81 => // RES 0,C - 2
                    gb_cpu.resnum_8(0, C),
                0x82 => // RES 0,D - 2
                    gb_cpu.resnum_8(0, D),
                0x83 => // RES 0,E - 2
                    gb_cpu.resnum_8(0, E),
                0x84 => // RES 0,H - 2
                    gb_cpu.resnum_8(0, H),
                0x85 => // RES 0,L - 2
                    gb_cpu.resnum_8(0, L),
                0x86 => // RES 0,[HL] - 4
                    gb_cpu.resnum_phl(gb_mem, 0),
                0x87 => // RES 0,A - 2
                    gb_cpu.resnum_8(0, A),
                0x88 => // RES 1,B - 2
                    gb_cpu.resnum_8(1, B),
                0x89 => // RES 1,C - 2
                    gb_cpu.resnum_8(1, C),
                0x8A => // RES 1,D - 2
                    gb_cpu.resnum_8(1, D),
                0x8B => // RES 1,E - 2
                    gb_cpu.resnum_8(1, E),
                0x8C => // RES 1,H - 2
                    gb_cpu.resnum_8(1, H),
                0x8D => // RES 1,L - 2
                    gb_cpu.resnum_8(1, L),
                0x8E => // RES 1,[HL] - 4
                    gb_cpu.resnum_phl(gb_mem, 1),
                0x8F => // RES 1,A - 2
                    gb_cpu.resnum_8(1, A),
                0x90 => // RES 2,B - 2
                    gb_cpu.resnum_8(2, B),
                0x91 => // RES 2,C - 2
                    gb_cpu.resnum_8(2, C),
                0x92 => // RES 2,D - 2
                    gb_cpu.resnum_8(2, D),
                0x93 => // RES 2,E - 2
                    gb_cpu.resnum_8(2, E),
                0x94 => // RES 2,H - 2
                    gb_cpu.resnum_8(2, H),
                0x95 => // RES 2,L - 2
                    gb_cpu.resnum_8(2, L),
                0x96 => // RES 2,[HL] - 4
                    gb_cpu.resnum_phl(gb_mem, 2),
                0x97 => // RES 2,A - 2
                    gb_cpu.resnum_8(2, A),
                0x98 => // RES 3,B - 2
                    gb_cpu.resnum_8(3, B),
                0x99 => // RES 3,C - 2
                    gb_cpu.resnum_8(3, C),
                0x9A => // RES 3,D - 2
                    gb_cpu.resnum_8(3, D),
                0x9B => // RES 3,E - 2
                    gb_cpu.resnum_8(3, E),
                0x9C => // RES 3,H - 2
                    gb_cpu.resnum_8(3, H),
                0x9D => // RES 3,L - 2
                    gb_cpu.resnum_8(3, L),
                0x9E => // RES 3,[HL] - 4
                    gb_cpu.resnum_phl(gb_mem, 3),
                0x9F => // RES 3,A - 2
                    gb_cpu.resnum_8(3, A),
                0xA0 => // RES 4,B - 2
                    gb_cpu.resnum_8(4, B),
                0xA1 => // RES 4,C - 2
                    gb_cpu.resnum_8(4, C),
                0xA2 => // RES 4,D - 2
                    gb_cpu.resnum_8(4, D),
                0xA3 => // RES 4,E - 2
                    gb_cpu.resnum_8(4, E),
                0xA4 => // RES 4,H - 2
                    gb_cpu.resnum_8(4, H),
                0xA5 => // RES 4,L - 2
                    gb_cpu.resnum_8(4, L),
                0xA6 => // RES 4,[HL] - 4
                    gb_cpu.resnum_phl(gb_mem, 4),
                0xA7 => // RES 4,A - 2
                    gb_cpu.resnum_8(4, A),
                0xA8 => // RES 5,B - 2
                    gb_cpu.resnum_8(5, B),
                0xA9 => // RES 5,C - 2
                    gb_cpu.resnum_8(5, C),
                0xAA => // RES 5,D - 2
                    gb_cpu.resnum_8(5, D),
                0xAB => // RES 5,E - 2
                    gb_cpu.resnum_8(5, E),
                0xAC => // RES 5,H - 2
                    gb_cpu.resnum_8(5, H),
                0xAD => // RES 5,L - 2
                    gb_cpu.resnum_8(5, L),
                0xAE => // RES 5,[HL] - 4
                    gb_cpu.resnum_phl(gb_mem, 5),
                0xAF => // RES 5,A - 2
                    gb_cpu.resnum_8(5, A),
                0xB0 => // RES 6,B - 2
                    gb_cpu.resnum_8(6, B),
                0xB1 => // RES 6,C - 2
                    gb_cpu.resnum_8(6, C),
                0xB2 => // RES 6,D - 2
                    gb_cpu.resnum_8(6, D),
                0xB3 => // RES 6,E - 2
                    gb_cpu.resnum_8(6, E),
                0xB4 => // RES 6,H - 2
                    gb_cpu.resnum_8(6, H),
                0xB5 => // RES 6,L - 2
                    gb_cpu.resnum_8(6, L),
                0xB6 => // RES 6,[HL] - 4
                    gb_cpu.resnum_phl(gb_mem, 6),
                0xB7 => // RES 6,A - 2
                    gb_cpu.resnum_8(6, A),
                0xB8 => // RES 7,B - 2
                    gb_cpu.resnum_8(7, B),
                0xB9 => // RES 7,C - 2
                    gb_cpu.resnum_8(7, C),
                0xBA => // RES 7,D - 2
                    gb_cpu.resnum_8(7, D),
                0xBB => // RES 7,E - 2
                    gb_cpu.resnum_8(7, E),
                0xBC => // RES 7,H - 2
                    gb_cpu.resnum_8(7, H),
                0xBD => // RES 7,L - 2
                    gb_cpu.resnum_8(7, L),
                0xBE => // RES 7,[HL] - 4
                    gb_cpu.resnum_phl(gb_mem, 7),
                0xBF => // RES 7,A - 2
                    gb_cpu.resnum_8(7, A),
                0xC0 => // SET 0,B - 2
                    gb_cpu.setnum_8(0, B),
                0xC1 => // SET 0,C - 2
                    gb_cpu.setnum_8(0, C),
                0xC2 => // SET 0,D - 2
                    gb_cpu.setnum_8(0, D),
                0xC3 => // SET 0,E - 2
                    gb_cpu.setnum_8(0, E),
                0xC4 => // SET 0,H - 2
                    gb_cpu.setnum_8(0, H),
                0xC5 => // SET 0,L - 2
                    gb_cpu.setnum_8(0, L),
                0xC6 => // SET 0,[HL] - 4
                    gb_cpu.setnum_phl(gb_mem, 0),
                0xC7 => // SET 0,A - 2
                    gb_cpu.setnum_8(0, A),
                0xC8 => // SET 1,B - 2
                    gb_cpu.setnum_8(1, B),
                0xC9 => // SET 1,C - 2
                    gb_cpu.setnum_8(1, C),
                0xCA => // SET 1,D - 2
                    gb_cpu.setnum_8(1, D),
                0xCB => // SET 1,E - 2
                    gb_cpu.setnum_8(1, E),
                0xCC => // SET 1,H - 2
                    gb_cpu.setnum_8(1, H),
                0xCD => // SET 1,L - 2
                    gb_cpu.setnum_8(1, L),
                0xCE => // SET 1,[HL] - 4
                    gb_cpu.setnum_phl(gb_mem, 1),
                0xCF => // SET 1,A - 2
                    gb_cpu.setnum_8(1, A),
                0xD0 => // SET 2,B - 2
                    gb_cpu.setnum_8(2, B),
                0xD1 => // SET 2,C - 2
                    gb_cpu.setnum_8(2, C),
                0xD2 => // SET 2,D - 2
                    gb_cpu.setnum_8(2, D),
                0xD3 => // SET 2,E - 2
                    gb_cpu.setnum_8(2, E),
                0xD4 => // SET 2,H - 2
                    gb_cpu.setnum_8(2, H),
                0xD5 => // SET 2,L - 2
                    gb_cpu.setnum_8(2, L),
                0xD6 => // SET 2,[HL] - 4
                    gb_cpu.setnum_phl(gb_mem, 2),
                0xD7 => // SET 2,A - 2
                    gb_cpu.setnum_8(2, A),
                0xD8 => // SET 3,B - 2
                    gb_cpu.setnum_8(3, B),
                0xD9 => // SET 3,C - 2
                    gb_cpu.setnum_8(3, C),
                0xDA => // SET 3,D - 2
                    gb_cpu.setnum_8(3, D),
                0xDB => // SET 3,E - 2
                    gb_cpu.setnum_8(3, E),
                0xDC => // SET 3,H - 2
                    gb_cpu.setnum_8(3, H),
                0xDD => // SET 3,L - 2
                    gb_cpu.setnum_8(3, L),
                0xDE => // SET 3,[HL] - 4
                    gb_cpu.setnum_phl(gb_mem, 3),
                0xDF => // SET 3,A - 2
                    gb_cpu.setnum_8(3, A),
                0xE0 => // SET 4,B - 2
                    gb_cpu.setnum_8(4, B),
                0xE1 => // SET 4,C - 2
                    gb_cpu.setnum_8(4, C),
                0xE2 => // SET 4,D - 2
                    gb_cpu.setnum_8(4, D),
                0xE3 => // SET 4,E - 2
                    gb_cpu.setnum_8(4, E),
                0xE4 => // SET 4,H - 2
                    gb_cpu.setnum_8(4, H),
                0xE5 => // SET 4,L - 2
                    gb_cpu.setnum_8(4, L),
                0xE6 => // SET 4,[HL] - 4
                    gb_cpu.setnum_phl(gb_mem, 4),
                0xE7 => // SET 4,A - 2
                    gb_cpu.setnum_8(4, A),
                0xE8 => // SET 5,B - 2
                    gb_cpu.setnum_8(5, B),
                0xE9 => // SET 5,C - 2
                    gb_cpu.setnum_8(5, C),
                0xEA => // SET 5,D - 2
                    gb_cpu.setnum_8(5, D),
                0xEB => // SET 5,E - 2
                    gb_cpu.setnum_8(5, E),
                0xEC => // SET 5,H - 2
                    gb_cpu.setnum_8(5, H),
                0xED => // SET 5,L - 2
                    gb_cpu.setnum_8(5, L),
                0xEE => // SET 5,[HL] - 4
                    gb_cpu.setnum_phl(gb_mem, 5),
                0xEF => // SET 5,A - 2
                    gb_cpu.setnum_8(5, A),
                0xF0 => // SET 6,B - 2
                    gb_cpu.setnum_8(6, B),
                0xF1 => // SET 6,C - 2
                    gb_cpu.setnum_8(6, C),
                0xF2 => // SET 6,D - 2
                    gb_cpu.setnum_8(6, D),
                0xF3 => // SET 6,E - 2
                    gb_cpu.setnum_8(6, E),
                0xF4 => // SET 6,H - 2
                    gb_cpu.setnum_8(6, H),
                0xF5 => // SET 6,L - 2
                    gb_cpu.setnum_8(6, L),
                0xF6 => // SET 6,[HL] - 4
                    gb_cpu.setnum_phl(gb_mem, 6),
                0xF7 => // SET 6,A - 2
                    gb_cpu.setnum_8(6, A),
                0xF8 => // SET 7,B - 2
                    gb_cpu.setnum_8(7, B),
                0xF9 => // SET 7,C - 2
                    gb_cpu.setnum_8(7, C),
                0xFA => // SET 7,D - 2
                    gb_cpu.setnum_8(7, D),
                0xFB => // SET 7,E - 2
                    gb_cpu.setnum_8(7, E),
                0xFC => // SET 7,H - 2
                    gb_cpu.setnum_8(7, H),
                0xFD => // SET 7,L - 2
                    gb_cpu.setnum_8(7, L),
                0xFE => // SET 7,[HL] - 4
                    gb_cpu.setnum_phl(gb_mem, 7),
                0xFF => // SET 7,A - 2
                    gb_cpu.setnum_8(7, A),
            } // End 0xCB
        }

        0xCC => // CALL Z,nnnn - 6/3
            gb_cpu.call_addr16(gb_mem, gb_cpu.get_flag(FL_Z)),
        0xCD => // CALL nnnn - 6
            gb_cpu.call_addr16(gb_mem, true),
        0xCE => // ADC A,nn - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.set_flag(false, FL_N);
            let tmp: u32 = gb_mem.read(gb_cpu.pc) as u32;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            let tmp2: u32 = gb_cpu.regs[A] as u32 + tmp + gb_cpu.get_flag(FL_C) as u32;
            gb_cpu.set_flag(((gb_cpu.regs[A] & 0xF) + (tmp & 0xF) as u8 + gb_cpu.get_flag(FL_C) as u8) > 0xF, FL_H);
            gb_cpu.set_flag(tmp2 > 0xFF, FL_C);
            gb_cpu.regs[A] = tmp2 as u8;
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.clk += 4;
        }
        0xCF => // RST 0x0008 - 4
            gb_cpu.rst_addr16(gb_mem, 0x0008),
        0xD0 => // RET NC - 5/2
            gb_cpu.ret(gb_mem, !gb_cpu.get_flag(FL_C)),
        0xD1 => // POP DE - 3
            gb_cpu.pop_16(gb_mem, D, E),
        0xD2 => // JP NC,nnnn - 4/3
            gb_cpu.jp_addr16(gb_mem, !gb_cpu.get_flag(FL_C)),
        0xD3 => // Undefined - *
            gb_cpu.undef(opcode),
        0xD4 => // CALL NC,nnnn - 6/3
            gb_cpu.call_addr16(gb_mem, !gb_cpu.get_flag(FL_C)),
        0xD5 => // PUSH DE - 4
            gb_cpu.push_16(gb_mem, D, E),
        0xD6 => // SUB A,nn - 2
        {
            gb_cpu.clk += 4;
            let tmp = gb_mem.read(gb_cpu.pc).wrapping_sub(0) as u32;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.regs[F] = FL_N;
            gb_cpu.set_flag((gb_cpu.regs[A] & 0xF) < (tmp & 0xF) as u8, FL_H);
            gb_cpu.set_flag((gb_cpu.regs[A] as u32) < tmp, FL_C);
            gb_cpu.regs[A] = gb_cpu.regs[A].wrapping_sub(tmp as u8);
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.clk += 4;
        }
        0xD7 => // RST 0x0010 - 4
            gb_cpu.rst_addr16(gb_mem, 0x0010),
        0xD8 => // RET C - 5/2
            gb_cpu.ret(gb_mem, gb_cpu.get_flag(FL_C)),
        0xD9 => // RETI - 4
        {
            gb_cpu.clk += 4;
            let mut tmp: u32 = gb_mem.read(gb_cpu.sp) as u32;
            gb_cpu.clk += 4;
            gb_cpu.sp = (gb_cpu.sp as u32 + 1) as u16;
            tmp |= (gb_mem.read(gb_cpu.sp) as u32) << 8;
            gb_cpu.sp = (gb_cpu.sp as u32 + 1) as u16;
            gb_cpu.clk += 4;
            gb_cpu.pc = tmp as u16;
            gb_cpu.ime = 1;
            gb_cpu.clk += 4;
        }
        0xDA => // JP C,nnnn - 4/3
            gb_cpu.jp_addr16(gb_mem, gb_cpu.get_flag(FL_C)),
        0xDB => // Undefined - *
            gb_cpu.undef(opcode),
        0xDC => // CALL C,nnnn - 6/3
            gb_cpu.call_addr16(gb_mem, gb_cpu.get_flag(FL_C)),
        0xDD => // Undefined - *
            gb_cpu.undef(opcode),
        0xDE => // SBC A,nn - 2
        {
            gb_cpu.clk += 4;
            let tmp2: u32 = gb_mem.read(gb_cpu.pc) as u32;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            let tmp: u32 = (gb_cpu.regs[A] as u32).wrapping_sub(tmp2).wrapping_sub((gb_cpu.get_flag(FL_C)) as u32);
            gb_cpu.regs[F] = if tmp & !0xFF != 0 {
                FL_C
            } else {
                0
            } | if tmp & 0xFF != 0{
                0
            } else {
                FL_Z
            } | FL_N;
            gb_cpu.set_flag(((gb_cpu.regs[A] as u32 ^ tmp2 ^ tmp) & 0x10) != 0, FL_H);
            gb_cpu.regs[A] = tmp as u8;
            gb_cpu.clk += 4;
        }
        0xDF => // RST 0x0018 - 4
            gb_cpu.rst_addr16(gb_mem, 0x0018),
        0xE0 => // LD [0xFF00+nn],A - 3
        {
            gb_cpu.clk += 4;
            let tmp: u32 = 0xFF00 + gb_mem.read(gb_cpu.pc) as u32;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.clk += 4;
            gb_mem.write(tmp as u16, gb_cpu.regs[A]);
            gb_cpu.clk += 4;
        }
        0xE1 => // POP HL - 3
            gb_cpu.pop_16(gb_mem, H, L),
        0xE2 => // LD [0xFF00+C],A - 2
        {
            gb_cpu.clk += 4;
            gb_mem.write(0xFF00 + gb_cpu.regs[C] as u16, gb_cpu.regs[A]);
            gb_cpu.clk += 4;
        }
        0xE3 => // Undefined - *
            gb_cpu.undef(opcode),
        0xE4 => // Undefined - *
            gb_cpu.undef(opcode),
        0xE5 => // PUSH HL - 4
            gb_cpu.push_16(gb_mem, H, L),
        0xE6 => // AND A,nn - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.set_flag(false, FL_N | FL_C);
            gb_cpu.set_flag(true, FL_H);
            gb_cpu.regs[A] &= gb_mem.read(gb_cpu.pc);
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.clk += 4;
        }
        0xE7 => // RST 0x0020 - 4
            gb_cpu.rst_addr16(gb_mem, 0x0020),
        0xE8 => // ADD SP,nn - 4
        {
            gb_cpu.clk += 4;
            // Expand sign
            let tmp: u32 = gb_mem.read(gb_cpu.pc) as i8 as i16 as u16 as u32;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.regs[F] = 0;
            gb_cpu.set_flag((gb_cpu.sp & 0x00FF) + (tmp & 0x00FF) as u16 > 0x00FF, FL_C);
            gb_cpu.set_flag((gb_cpu.sp & 0x000F) + (tmp & 0x000F) as u16 > 0x000F, FL_H);
            gb_cpu.sp = (gb_cpu.sp as u32 + tmp) as u16;
            gb_cpu.clk += 12;
        }
        0xE9 => // JP HL - 1
        {
            gb_cpu.pc = gb_cpu.get_hilo(H, L);
            gb_cpu.clk += 4;
        }
        0xEA => // LD [nnnn],A - 4
        {
            gb_cpu.clk += 4;
            let mut tmp: u32 = gb_mem.read(gb_cpu.pc) as u32;
            gb_cpu.clk += 4;
            gb_cpu.pc = (gb_cpu.pc as u32 + 1) as u16;
            tmp |= (gb_mem.read(gb_cpu.pc) as u32) << 8;
            gb_cpu.pc = (gb_cpu.pc as u32 + 1) as u16;
            gb_cpu.clk += 4;
            gb_mem.write(tmp as u16, gb_cpu.regs[A]);
            gb_cpu.clk += 4;
        }
        0xEB => // Undefined - *
            gb_cpu.undef(opcode),
        0xEC => // Undefined - *
            gb_cpu.undef(opcode),
        0xED => // Undefined - *
            gb_cpu.undef(opcode),
        0xEE => // XOR A,nn - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.set_flag(false, FL_N | FL_C | FL_H);
            gb_cpu.regs[A] ^= gb_mem.read(gb_cpu.pc);
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.clk += 4;
        }
        0xEF => // RST 0x0028 - 4
            gb_cpu.rst_addr16(gb_mem, 0x0028),

        0xF0 => // LD A,[0xFF00+nn] - 3
        {
            gb_cpu.clk += 4;
            let tmp: u32 = 0xFF00 + gb_mem.read(gb_cpu.pc) as u32;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.clk += 4;
            gb_cpu.regs[A] = gb_mem.read(tmp as u16);
            gb_cpu.clk += 4;
        }
        0xF1 => // POP AF - 3
        {
            gb_cpu.pop_16(gb_mem, A, F);
            gb_cpu.regs[F] &= 0xF0; // Lower 4 bits are always 0
        }
        0xF2 => // LD A,[0xFF00+C] - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.regs[A] = gb_mem.read(0xFF00 + gb_cpu.regs[C] as u16);
            gb_cpu.clk += 4;
        }
        0xF3 => // DI - 1
        {
            gb_cpu.ime = 3;
            gb_cpu.clk += 4;
        }
        0xF4 => // Undefined - *
            gb_cpu.undef(opcode),
        0xF5 => // PUSH AF - 4
            gb_cpu.push_16(gb_mem, A, F),
        0xF6 => // OR A,nn - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.set_flag(false, FL_N | FL_C | FL_H);
            gb_cpu.regs[A] |= gb_mem.read(gb_cpu.pc);
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.set_flag(gb_cpu.regs[A] == 0, FL_Z);
            gb_cpu.clk += 4;
        }
        0xF7 => // RST 0x0030 - 4
            gb_cpu.rst_addr16(gb_mem, 0x0030),
        0xF8 => // LD HL,SP+nn - 3
        {
            gb_cpu.clk += 4;
            let tmp: i32 = gb_mem.read(gb_cpu.pc) as i8 as i32;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            let res = gb_cpu.sp as i32 + tmp;
            gb_cpu.set_hilo(H, L, res as u16);
            gb_cpu.regs[F] = 0;
            gb_cpu.set_flag((gb_cpu.sp & 0x00FF) as i32 + (tmp & 0x00FF) > 0x00FF, FL_C);
            gb_cpu.set_flag((gb_cpu.sp & 0x000F) as i32 + (tmp & 0x000F) > 0x000F, FL_H);
            gb_cpu.clk += 8;
        }
        0xF9 => // LD SP,HL - 2
        {
            gb_cpu.sp = gb_cpu.get_hilo(H, L);
            gb_cpu.clk += 8;
        }
        0xFA => // LD A,[nnnn] - 4
        {
            gb_cpu.clk += 4;
            let mut tmp: u32 = gb_mem.read(gb_cpu.pc) as u32;
            gb_cpu.clk += 4;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            tmp |= (gb_mem.read(gb_cpu.pc) as u32) << 8;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            gb_cpu.clk += 4;
            gb_cpu.regs[A] = gb_mem.read(tmp as u16);
            gb_cpu.clk += 4;
        }
        0xFB => // EI - 1
        {
            gb_cpu.ime = 2;
            gb_cpu.clk += 4;
        }
        0xFC => // Undefined - *
            gb_cpu.undef(opcode),
        0xFD => // Undefined - *
            gb_cpu.undef(opcode),
        0xFE => // CP A,nn - 2
        {
            gb_cpu.clk += 4;
            gb_cpu.set_flag(true, FL_N);
            let tmp: u32 = gb_mem.read(gb_cpu.pc) as u32;
            gb_cpu.pc = gb_cpu.pc.wrapping_add(1);
            let tmp2: u32 = gb_cpu.regs[A] as u32;
            gb_cpu.set_flag((tmp2 & 0xF) < (tmp & 0xF), FL_H);
            gb_cpu.set_flag(tmp2 < tmp, FL_C);
            gb_cpu.set_flag(tmp2 == tmp, FL_Z);
            gb_cpu.clk += 4;
        }
        0xFF => // RST 0x0038 - 4
            gb_cpu.rst_addr16(gb_mem, 0x0038),

    }
}