use crate::fetch_decode::Opcode;
use fetch_decode::{AddrMode, decode};
use log::warn;

mod fetch_decode;
pub mod rom;
use rom::*;

bitflags::bitflags! {
    pub struct Flags: u8 {
        const NEGATIVE = 0b_1000_0000;
        const OVERFLOW = 0b_0100_0000;
        const ALWAYSON = 0b_0010_0000;
        const UNUSED = 0b_0010_0000;
        const BREAK = 0b_0001_0000;
        const DECIMAL = 0b_0000_1000;
        const INTERRUPTDISABLE = 0b_0000_0100;
        const ZERO = 0b_0000_0010;
        const CARRY = 0b_0000_0001;
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self::from_bits_retain(0)
    }
}

pub struct Bus {
    pub cpu_ram: [u8; 0x800],
    pub rom: Rom,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Bus {
            cpu_ram: [0; 0x800],
            rom,
        }
    }
}

impl Bus {
    pub fn read(&self, pos: u16) -> u8 {
        match pos {
            // CPU
            0x0000..=0x1FFF => {
                let masked = pos & 0x07ff;
                self.cpu_ram[masked as usize]
            }
            // PPU
            0x2000..=0x3FFF => {
                let _masked = pos & 0x2007;
                todo!("PPU")
            }
            0x8000..=0xFFFF => {
                let mut pos = pos - 0x8000;
                // if self.rom.prg_rom.len() == 0x4000 && pos >= 0x4000 {
                //     pos %= 0x4000;
                // }
                if self.rom.prg_rom.len() == 0x4000 {
                    pos %= 0x4000;
                }
                self.rom.prg_rom[pos as usize]
            }
            // 0xfffc..=0xfffd => {
            //     let masked = pos & 0x1;
            //     self.pc_start_mem[masked as usize]
            // }
            _ => {
                warn!("Unknown memory address 0x{pos:04X} accessed, ignoring...");
                0
            }
        }
    }
    pub fn write(&mut self, pos: u16, val: u8) {
        match pos {
            // CPU
            0x0000..=0x1FFF => {
                let masked = pos & 0x07ff;
                self.cpu_ram[masked as usize] = val;
            }
            // PPU
            0x2000..=0x3FFF => {
                let _masked = pos & 0x2007;
                todo!("PPU")
            }
            0x8000..=0xFFFF => {
                panic!("Attempted to write into PRG rom")
            }
            _ => {
                warn!("Unknown memory address 0x{pos:04X} accessed, ignoring...");
            }
        }
    }
    pub fn read_u16(&self, pos: u16) -> u16 {
        // dbg!(pos);
        let low = self.read(pos) as u16;
        // dbg!(low);
        let high = self.read(pos + 1) as u16;
        // dbg!(high);
        (high << 8) | low
    }
    pub fn write_u16(&mut self, pos: u16, val: u16) {
        let low = (val & 0xff) as u8;
        let high = (val >> 8) as u8;
        self.write(pos, low);
        self.write(pos + 1, high);
    }
    pub fn load_to(&mut self, pos: u16, slice: &[u8]) {
        let pos = pos as usize;
        self.cpu_ram[pos..(pos + slice.len())].copy_from_slice(slice)
    }
}

pub struct Cpu {
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub stack_ptr: u8,
    pub pc: u16,
    pub status: Flags,
    pub memory: Bus,
}

const STACK_RESET: u8 = 0xfd;
const STACK_START: u16 = 0x100;

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        let mut me = Self {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            stack_ptr: 0,
            pc: 0,
            status: Flags::empty(),
            memory: bus,
        };
        me.reset();
        me
    }
    pub fn reset(&mut self) {
        self.status = Flags::INTERRUPTDISABLE | Flags::ALWAYSON;
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.stack_ptr = STACK_RESET;
        let pc = self.memory.read_u16(0xFFFC);
        self.pc = pc;
    }
    pub fn load_to(&mut self, start: u16, program: &[u8]) {
        self.memory.load_to(start, program);
        self.memory.write_u16(0xFFFC, 0x8000);
    }
    pub fn run_with_callback<F: FnMut(&mut Cpu)>(&mut self, mut f: F) {
        while !self.status.contains(Flags::BREAK) {
            f(self);
            self.step();
        }
    }
    pub fn run(&mut self) {
        self.run_with_callback(|_| {})
    }
    fn update_zero_negative(&mut self, value: u8) {
        self.status.set(Flags::ZERO, value == 0);
        self.status.set(Flags::NEGATIVE, value & 0x80 != 0);
    }
}

impl Cpu {
    /// Get the value at the correct addressing mode
    /// Assumes the `pc` is still set at the instruction beginning
    fn get_addr_mode_dest(&self, addr_mode: AddrMode) -> u16 {
        match addr_mode {
            AddrMode::Implicit => panic!("Implicit should not need a memory load"),
            AddrMode::Accumulator => panic!("Accumulator should not need a memory load"),
            AddrMode::Immediate => self.pc + 1,
            AddrMode::ZeroPage => self.memory.read(self.pc + 1) as u16,
            AddrMode::ZeroPageX => self.memory.read(self.pc + 1).wrapping_add(self.reg_x) as u16,
            AddrMode::ZeroPageY => self.memory.read(self.pc + 1).wrapping_add(self.reg_y) as u16,
            AddrMode::Relative => self.pc + 1,
            AddrMode::Absolute => self.memory.read_u16(self.pc + 1),
            AddrMode::AbsoluteX => {
                self.memory
                    .read_u16(self.pc + 1)
                    .wrapping_add(self.reg_x as u16)
                // + self.status.contains(Flags::CARRY) as u16
            }
            AddrMode::AbsoluteY => {
                self.memory
                    .read_u16(self.pc + 1)
                    .wrapping_add(self.reg_y as u16)
                // + self.status.contains(Flags::CARRY) as u16
            }
            AddrMode::Indirect => {
                panic!("Should be implemented outside")
                // let imm = self.memory.read_u16(self.pc + 1);
                // self.memory.read_u16(imm)
            }
            AddrMode::IndexedIndirect => {
                let addr = self.memory.read(self.pc + 1).wrapping_add(self.reg_x);
                let low = self.memory.read(addr as u16);
                let high = self.memory.read((addr as u16).wrapping_add(1));
                ((high as u16) << 8) | (low as u16)
            }
            AddrMode::IndirectIndexed => {
                let base_loc = self.memory.read(self.pc + 1);
                let low = self.memory.read(base_loc as u16);
                let high = self.memory.read(base_loc.wrapping_add(1) as u16);
                let base = ((high as u16) << 8) | (low as u16);
                base + self.reg_y as u16
            }
        }
    }
    pub fn step(&mut self) {
        // trace!(
        //     "PC: {:02X}, values {:02X}, {:02X}, {:02X}",
        //     self.pc,
        //     self.memory.read(self.pc),
        //     self.memory.read(self.pc + 1),
        //     self.memory.read(self.pc + 2)
        // );
        let instruction_byte = self.memory.read(self.pc);
        let (opcode, addr_mode, inst_info) = decode(instruction_byte);
        // trace!("{opcode:?}, {addr_mode:?}, from {instruction_byte:02X}");
        match (opcode, addr_mode) {
            (Opcode::ADC, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                let val = self.memory.read(addr);
                self.add_a(val);
            }
            (Opcode::AND, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                let val = self.memory.read(addr);
                self.reg_a &= val;
                self.update_zero_negative(self.reg_a);
            }
            (Opcode::ASL, addr_mode) => {
                if let AddrMode::Accumulator = addr_mode {
                    let carry = self.reg_a & 0x80 != 0;
                    self.reg_a <<= 1;
                    self.status.set(Flags::CARRY, carry);
                    self.update_zero_negative(self.reg_a);
                } else {
                    let addr = self.get_addr_mode_dest(addr_mode);
                    let val = self.memory.read(addr);
                    let carry = val & 0x80 != 0;
                    let new_val = val << 1;
                    self.memory.write(addr, new_val);
                    self.status.set(Flags::CARRY, carry);
                    self.update_zero_negative(new_val);
                }
            }
            (Opcode::BCC, addr_mode) => {
                self.branch_impl(addr_mode, Flags::CARRY, false);
            }
            (Opcode::BCS, addr_mode) => {
                self.branch_impl(addr_mode, Flags::CARRY, true);
            }
            (Opcode::BEQ, addr_mode) => {
                self.branch_impl(addr_mode, Flags::ZERO, true);
            }
            (Opcode::BIT, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                let val = self.memory.read(addr);
                self.status.set(Flags::OVERFLOW, val & 0b0100_0000 != 0);
                self.status.set(Flags::NEGATIVE, val & 0b1000_0000 != 0);
                self.status.set(Flags::ZERO, self.reg_a & val == 0);
            }
            (Opcode::BMI, addr_mode) => {
                self.branch_impl(addr_mode, Flags::NEGATIVE, true);
            }
            (Opcode::BNE, addr_mode) => {
                self.branch_impl(addr_mode, Flags::ZERO, false);
            }
            (Opcode::BPL, addr_mode) => {
                self.branch_impl(addr_mode, Flags::NEGATIVE, false);
            }
            (Opcode::BRK, _addr_mode) => {
                self.status.insert(Flags::BREAK);
                return;
                // self.push_stack_u16(self.pc);
                // self.push_stack(self.status.bits());
                // self.pc = self.memory.read_u16(0xfffe);
            }
            (Opcode::BVC, addr_mode) => {
                self.branch_impl(addr_mode, Flags::OVERFLOW, false);
            }
            (Opcode::BVS, addr_mode) => {
                self.branch_impl(addr_mode, Flags::OVERFLOW, true);
            }
            (Opcode::CLC, _addr_mode) => {
                self.status.remove(Flags::CARRY);
            }
            (Opcode::CLD, _addr_mode) => {
                self.status.remove(Flags::DECIMAL);
            }
            (Opcode::CLI, _addr_mode) => {
                self.status.remove(Flags::INTERRUPTDISABLE);
            }
            (Opcode::CLV, _addr_mode) => {
                self.status.remove(Flags::OVERFLOW);
            }
            (Opcode::CMP, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                let m = self.memory.read(addr);
                self.status.set(Flags::CARRY, self.reg_a >= m);
                self.update_zero_negative(self.reg_a.wrapping_sub(m));
            }
            (Opcode::CPX, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                let m = self.memory.read(addr);
                self.status.set(Flags::CARRY, self.reg_x >= m);
                self.update_zero_negative(self.reg_x.wrapping_sub(m));
            }
            (Opcode::CPY, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                let m = self.memory.read(addr);
                self.status.set(Flags::CARRY, self.reg_y >= m);
                self.update_zero_negative(self.reg_y.wrapping_sub(m));
            }
            (Opcode::DEC, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                let val = self.memory.read(addr);
                let val_m = val.wrapping_sub(1);
                self.memory.write(addr, val_m);
                self.update_zero_negative(val_m);
            }
            (Opcode::DEX, _addr_mode) => {
                self.reg_x = self.reg_x.wrapping_sub(1);
                self.update_zero_negative(self.reg_x);
            }
            (Opcode::DEY, _addr_mode) => {
                self.reg_y = self.reg_x.wrapping_sub(1);
                self.update_zero_negative(self.reg_y);
            }
            (Opcode::EOR, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                self.reg_a ^= self.memory.read(addr);
                self.update_zero_negative(self.reg_a);
            }
            (Opcode::INC, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                let val = self.memory.read(addr);
                let val_m = val.wrapping_add(1);
                self.memory.write(addr, val_m);
                self.update_zero_negative(val_m);
            }
            (Opcode::INX, _addr_mode) => {
                self.reg_x = self.reg_x.wrapping_add(1);
                self.update_zero_negative(self.reg_x);
            }
            (Opcode::INY, _addr_mode) => {
                self.reg_y = self.reg_y.wrapping_add(1);
                self.update_zero_negative(self.reg_y);
            }
            (Opcode::JMP, addr_mode) => match addr_mode {
                AddrMode::Absolute => {
                    let addr = self.get_addr_mode_dest(addr_mode);
                    self.pc = addr;
                    return;
                }
                AddrMode::Indirect => {
                    let indirect_addr = self.memory.read_u16(self.pc + 1);
                    let new_pc = if indirect_addr as u8 == 0xff {
                        let low = self.memory.read(indirect_addr) as u16;
                        let high = self.memory.read(indirect_addr & 0xff00) as u16;
                        (high << 8) | low
                    } else {
                        self.memory.read_u16(indirect_addr)
                    };
                    self.pc = new_pc;
                    return;
                }
                _ => unreachable!(),
            },
            (Opcode::JSR, addr_mode) => {
                match addr_mode {
                    AddrMode::Absolute => (),
                    _ => panic!("Invalid addr mode for JSR: {addr_mode:?}"),
                }
                let return_loc = self.pc + 3; // jsr is 3 bytes
                let fn_addr = self.memory.read_u16(self.pc + 1); // absolute
                self.push_stack_u16(return_loc - 1); // rti is 1 byte so it'll be incremented
                self.pc = fn_addr;
                return;
            }
            (Opcode::LDA, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                let val = self.memory.read(addr);
                self.reg_a = val;
                self.update_zero_negative(val);
            }
            (Opcode::LDX, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                let val = self.memory.read(addr);
                self.reg_x = val;
                self.update_zero_negative(val);
            }
            (Opcode::LDY, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                let val = self.memory.read(addr);
                self.reg_y = val;
                self.update_zero_negative(val);
            }
            (Opcode::LSR, addr_mode) => {
                if let AddrMode::Accumulator = addr_mode {
                    let carry = self.reg_a & 0x1 != 0;
                    self.reg_a >>= 1;
                    self.status.set(Flags::CARRY, carry);
                    self.update_zero_negative(self.reg_a);
                } else {
                    let addr = self.get_addr_mode_dest(addr_mode);
                    let val = self.memory.read(addr);
                    let carry = val & 0x1 != 0;
                    let new_val = val >> 1;
                    self.memory.write(addr, new_val);
                    self.status.set(Flags::CARRY, carry);
                    self.update_zero_negative(new_val);
                }
            }
            (Opcode::NOP, _addr_mode) => {}
            (Opcode::ORA, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                self.reg_a |= self.memory.read(addr);
                self.update_zero_negative(self.reg_a);
            }
            (Opcode::PHA, _addr_mode) => {
                self.push_stack(self.reg_a);
            }
            (Opcode::PHP, _addr_mode) => {
                self.push_stack(self.status.bits());
            }
            (Opcode::PLA, _addr_mode) => {
                self.reg_a = self.pop_stack();
                self.update_zero_negative(self.reg_a);
            }
            (Opcode::PLP, _addr_mode) => {
                self.status = Flags::from_bits_retain(self.pop_stack());
            }
            (Opcode::ROL, addr_mode) => {
                if let AddrMode::Accumulator = addr_mode {
                    let carry = self.reg_a & 0x80 != 0;
                    self.reg_a <<= 1;
                    self.reg_a += self.status.contains(Flags::CARRY) as u8;
                    self.status.set(Flags::CARRY, carry);
                    self.update_zero_negative(self.reg_a);
                } else {
                    let addr = self.get_addr_mode_dest(addr_mode);
                    let val = self.memory.read(addr);
                    let carry = val & 0x80 != 0;
                    let mut new_val = val << 1;
                    new_val += self.status.contains(Flags::CARRY) as u8;
                    self.memory.write(addr, new_val);
                    self.status.set(Flags::CARRY, carry);
                    self.update_zero_negative(new_val);
                }
            }
            (Opcode::ROR, addr_mode) => {
                if let AddrMode::Accumulator = addr_mode {
                    let carry = self.reg_a & 0x1 != 0;
                    self.reg_a >>= 1;
                    self.reg_a += (self.status.contains(Flags::CARRY) as u8) << 7;
                    self.status.set(Flags::CARRY, carry);
                    self.update_zero_negative(self.reg_a);
                } else {
                    let addr = self.get_addr_mode_dest(addr_mode);
                    let val = self.memory.read(addr);
                    let carry = val & 0x1 != 0;
                    let mut new_val = val >> 1;
                    new_val += (self.status.contains(Flags::CARRY) as u8) << 7;
                    self.memory.write(addr, new_val);
                    self.status.set(Flags::CARRY, carry);
                    self.update_zero_negative(new_val);
                }
            }
            (Opcode::RTI, _addr_mode) => {
                self.status = Flags::from_bits_retain(self.pop_stack());
                self.pc = self.pop_stack_u16();
            }
            (Opcode::RTS, _addr_mode) => {
                self.pc = self.pop_stack_u16();
            }
            (Opcode::SBC, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                let val = self.memory.read(addr);
                self.add_a(!val);
            }
            (Opcode::SEC, _addr_mode) => self.status.insert(Flags::CARRY),
            (Opcode::SED, _addr_mode) => self.status.insert(Flags::DECIMAL),
            (Opcode::SEI, _addr_mode) => self.status.insert(Flags::INTERRUPTDISABLE),
            (Opcode::STA, _addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                self.memory.write(addr, self.reg_a);
            }
            (Opcode::STX, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                self.memory.write(addr, self.reg_x);
            }
            (Opcode::STY, addr_mode) => {
                let addr = self.get_addr_mode_dest(addr_mode);
                self.memory.write(addr, self.reg_y);
            }
            (Opcode::TAX, _addr_mode) => {
                self.reg_x = self.reg_a;
                self.update_zero_negative(self.reg_x);
            }
            (Opcode::TAY, _addr_mode) => {
                self.reg_y = self.reg_a;
                self.update_zero_negative(self.reg_y);
            }
            (Opcode::TSX, _addr_mode) => {
                self.reg_x = self.stack_ptr;
                self.update_zero_negative(self.reg_x);
            }
            (Opcode::TXA, _addr_mode) => {
                self.reg_a = self.reg_x;
                self.update_zero_negative(self.reg_a);
            }
            (Opcode::TXS, _addr_mode) => {
                self.stack_ptr = self.reg_x;
                self.update_zero_negative(self.stack_ptr);
            }
            (Opcode::TYA, _addr_mode) => {
                self.reg_a = self.reg_y;
                self.update_zero_negative(self.reg_a);
            } // _ => todo!(),
        }
        self.pc += inst_info.size;
    }

    fn branch_impl(&mut self, addr_mode: AddrMode, flag: Flags, set: bool) {
        let addr = self.get_addr_mode_dest(addr_mode);
        let val = self.memory.read(addr) as i8 as i16;
        // contains, set => true
        // !contains, set => false
        // contains, !set => false
        // !contains, !set => true
        // xor truth table
        if !self.status.contains(flag) ^ set {
            self.pc = self.pc.wrapping_add_signed(val);
        }
    }

    fn add_a(&mut self, val: u8) {
        let sum = self.reg_a as u16 + val as u16 + self.status.contains(Flags::CARRY) as u16;
        self.status.set(Flags::CARRY, sum > 0xFF);
        let res = sum as u8;
        // https://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        let overflow = (val ^ res) & (res ^ self.reg_a) & 0x80 != 0;
        self.status.set(Flags::OVERFLOW, overflow);
        self.reg_a = res;
        self.update_zero_negative(res);
    }

    fn push_stack(&mut self, val: u8) {
        self.memory.write(STACK_START + self.stack_ptr as u16, val);
        self.stack_ptr = self.stack_ptr.wrapping_sub(1);
    }
    fn pop_stack(&mut self) -> u8 {
        self.stack_ptr = self.stack_ptr.wrapping_add(1);
        self.memory.read(STACK_START + self.stack_ptr as u16)
    }
    fn push_stack_u16(&mut self, val: u16) {
        let low = val as u8;
        let high = (val >> 8) as u8;
        self.push_stack(high);
        self.push_stack(low);
    }
    fn pop_stack_u16(&mut self) -> u16 {
        let low = self.pop_stack();
        let high = self.pop_stack();
        ((high as u16) << 8) | (low as u16)
    }
}
