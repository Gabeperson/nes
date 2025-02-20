#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Copy)]
pub enum Opcode {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

/// Different types of addressing modes that exist in the 6502 assembly.
/// Documentation was taken from
/// https://www.nesdev.org/obelisk-6502-guide/addressing.html
#[derive(Clone, Debug, Copy)]
pub enum AddrMode {
    /// Implied by the instruction itself
    /// Ex: `CLC`, `RTS`
    Implicit,
    /// Operates on the accumulator
    /// Ex: `LSR A`, `ROR A`
    Accumulator,
    /// 8 bit constant specified within the instruction
    /// Ex: `LDA #10`, `LDX #LO, LABEL`
    Immediate,
    /// 8 bit address is specified within the instruction
    /// This allows it to address the first 256 bytes of memory (0x00 to 0xFF)
    /// Ex: `LDA $00`, `ASL ANSWER`
    /// (Same asm regular absolute, but assembler chooses instruction accordingly)
    ZeroPage,
    /// 8 bit address specified within instruction is **wrapping add**ed with the `X` register
    /// Because a **wrapping** add is performed, only addresses 0x00 to 0xFF can be addressed
    /// Ex: `LDA $80,X` when X=0x0F would load from 0x8F
    /// Wrapping Ex: `LDA $80,X` when X=0xFF would load from 0x7F not 0x017F
    ZeroPageX,
    /// 8 bit address specified within instruction is **wrapping add**ed with the `Y` register
    /// Because a **wrapping** add is performed, only addresses 0x00 to 0xFF can be addressed
    /// Note: This is equivalent to `ZeroPageX` but for the `Y` register
    /// and is only used by `LDX` and `STX` instructions
    /// Ex: See `ZeroPageX`
    ZeroPageY,
    /// 8 bit **signed** relative offset is included in instruction itself (-128 to 127)
    /// which is added to PC if condition is true. Since PC is also incremented
    /// by 2 (size of instruction) before instruction is executed, effective branch from
    /// start of the branch instruction is (-126 to 129) bytes
    /// Ex: `BEQ LABEL`, `BNE *+4 (-2 bytes for instruction, skips next 2-byte instruction)`
    Relative,
    /// 16-bit **little endian** value is included in the instruction itself
    /// being **little endian**, the `0x1234` in `JMP $1234` would be stored as 0x34 0x12
    /// Ex: `JMP $1234`, `JSR LABEL`
    Absolute,
    /// 16-bit **little endian** value is included in the instruction itself
    /// This value is added with the `X` register, and the `CARRY` flag
    /// Ex: `LDA $8000,x`, `STA $9000,x`
    AbsoluteX,
    /// 16-bit **little endian** value is included in the instruction itself
    /// This value is added with the `Y` register, and the `CARRY` flag
    /// This is the same as the `AbsoluteX` mode, but with Y instead.
    /// Ex: `LDA $8000,y`, `STA $9000,y`
    AbsoluteY,
    /// 16-bit **little endian** value is included in the instruction itself
    /// This value is the memory address of a **little endian** value.
    /// The value at this memory address is the actual value.
    /// Ex: `JMP ($1234)` and address 1234 contains AB, 1235 contains CD
    /// would compile to `6C 34 12`, would load value from `0xCDAB`
    Indirect,
    /// Also known as Indirect X
    /// 8-bit memory address included in instruction itself
    /// This value is **wrapping** added to the X register
    /// And this value is used to load a **little endian** pointer to the
    /// address of the actual value.
    /// (Essentially, X is an index to the 8 bit zero page address that
    /// contains an array of pointers)
    IndexedIndirect,
    /// Also known as Indirect Y
    /// 8-bit zero-page memory address included in instruction itself
    /// This zero-page 16-bit **little-endian** VALUE (after memory access) is
    /// added to register `Y` to get actual target address
    IndirectIndexed,
}

#[derive(Debug, Clone, Copy)]
pub struct InstructionInfo {
    pub cycles: u16,
    pub size: u16,
    pub cycles_extra: u16,
    pub cycles_extra2: u16,
}

fn info(size: u16, cycles: u16) -> InstructionInfo {
    InstructionInfo {
        cycles,
        size,
        cycles_extra: 0,
        cycles_extra2: 0,
    }
}

fn info_extra(size: u16, cycles: u16, cycles_extra: u16) -> InstructionInfo {
    InstructionInfo {
        cycles,
        size,
        cycles_extra,
        cycles_extra2: 0,
    }
}

fn info_extra2(size: u16, cycles: u16, cycles_extra: u16, cycles_extra2: u16) -> InstructionInfo {
    InstructionInfo {
        cycles,
        size,
        cycles_extra,
        cycles_extra2,
    }
}

pub fn decode(val: u8) -> (Opcode, AddrMode, InstructionInfo) {
    match val {
        0x69 => (Opcode::ADC, AddrMode::Immediate, info(2, 2)),
        0x65 => (Opcode::ADC, AddrMode::ZeroPage, info(2, 3)),
        0x75 => (Opcode::ADC, AddrMode::ZeroPageX, info(2, 4)),
        0x6D => (Opcode::ADC, AddrMode::Absolute, info(2, 4)),
        0x7D => (Opcode::ADC, AddrMode::AbsoluteX, info_extra(2, 4, 1)),
        0x79 => (Opcode::ADC, AddrMode::AbsoluteY, info_extra(2, 4, 1)),
        0x61 => (Opcode::ADC, AddrMode::IndexedIndirect, info(2, 6)),
        0x71 => (Opcode::ADC, AddrMode::IndirectIndexed, info_extra(2, 5, 1)),

        0x29 => (Opcode::AND, AddrMode::Immediate, info(2, 2)),
        0x25 => (Opcode::AND, AddrMode::ZeroPage, info(2, 3)),
        0x35 => (Opcode::AND, AddrMode::ZeroPageX, info(2, 4)),
        0x2D => (Opcode::AND, AddrMode::Absolute, info(3, 4)),
        0x3D => (Opcode::AND, AddrMode::AbsoluteX, info_extra(3, 4, 1)),
        0x39 => (Opcode::AND, AddrMode::AbsoluteY, info_extra(3, 4, 1)),
        0x21 => (Opcode::AND, AddrMode::IndexedIndirect, info(2, 6)),
        0x31 => (Opcode::AND, AddrMode::IndirectIndexed, info_extra(2, 5, 1)),

        0x0A => (Opcode::ASL, AddrMode::Accumulator, info(1, 2)),
        0x06 => (Opcode::ASL, AddrMode::ZeroPage, info(2, 5)),
        0x16 => (Opcode::ASL, AddrMode::ZeroPageX, info(2, 6)),
        0x0E => (Opcode::ASL, AddrMode::Absolute, info(3, 6)),
        0x1E => (Opcode::ASL, AddrMode::AbsoluteX, info(3, 7)),

        0x90 => (Opcode::BCC, AddrMode::Relative, info_extra2(2, 2, 1, 1)),
        0xB0 => (Opcode::BCS, AddrMode::Relative, info_extra2(2, 2, 1, 1)),
        0xF0 => (Opcode::BEQ, AddrMode::Relative, info_extra2(2, 2, 1, 1)),

        0x24 => (Opcode::BIT, AddrMode::ZeroPage, info(2, 3)),
        0x2C => (Opcode::BIT, AddrMode::Absolute, info(3, 4)),

        0x30 => (Opcode::BMI, AddrMode::Relative, info_extra2(2, 2, 1, 1)),
        0xD0 => (Opcode::BNE, AddrMode::Relative, info_extra2(2, 2, 1, 1)),
        0x10 => (Opcode::BPL, AddrMode::Relative, info_extra2(2, 2, 1, 1)),

        0x00 => (Opcode::BRK, AddrMode::Implicit, info(1, 7)),

        0x50 => (Opcode::BVC, AddrMode::Relative, info_extra2(2, 2, 1, 1)),
        0x70 => (Opcode::BVS, AddrMode::Relative, info_extra2(2, 2, 1, 1)),

        0x18 => (Opcode::CLC, AddrMode::Implicit, info(1, 2)),
        0xD8 => (Opcode::CLD, AddrMode::Implicit, info(1, 2)),
        0x58 => (Opcode::CLI, AddrMode::Implicit, info(1, 2)),
        0xB8 => (Opcode::CLV, AddrMode::Implicit, info(1, 2)),

        0xC9 => (Opcode::CMP, AddrMode::Immediate, info(2, 2)),
        0xC5 => (Opcode::CMP, AddrMode::ZeroPage, info(2, 3)),
        0xD5 => (Opcode::CMP, AddrMode::ZeroPageX, info(2, 4)),
        0xCD => (Opcode::CMP, AddrMode::Absolute, info(3, 4)),
        0xDD => (Opcode::CMP, AddrMode::AbsoluteX, info_extra(3, 4, 1)),
        0xD9 => (Opcode::CMP, AddrMode::AbsoluteY, info_extra(3, 4, 1)),
        0xC1 => (Opcode::CMP, AddrMode::IndexedIndirect, info(2, 6)),
        0xD1 => (Opcode::CMP, AddrMode::IndirectIndexed, info_extra(2, 5, 1)),

        0xE0 => (Opcode::CPX, AddrMode::Immediate, info(2, 2)),
        0xE4 => (Opcode::CPX, AddrMode::ZeroPage, info(2, 3)),
        0xEC => (Opcode::CPX, AddrMode::Absolute, info(2, 4)),

        0xC0 => (Opcode::CPY, AddrMode::Immediate, info(2, 2)),
        0xC4 => (Opcode::CPY, AddrMode::ZeroPage, info(2, 3)),
        0xCC => (Opcode::CPY, AddrMode::Absolute, info(2, 4)),

        0xC6 => (Opcode::DEC, AddrMode::ZeroPage, info(2, 5)),
        0xD6 => (Opcode::DEC, AddrMode::ZeroPageX, info(2, 6)),
        0xCE => (Opcode::DEC, AddrMode::Absolute, info(3, 6)),
        0xDE => (Opcode::DEC, AddrMode::AbsoluteX, info(3, 7)),

        0xCA => (Opcode::DEX, AddrMode::Implicit, info(1, 2)),
        0x88 => (Opcode::DEY, AddrMode::Implicit, info(1, 2)),

        0x49 => (Opcode::EOR, AddrMode::Immediate, info(2, 2)),
        0x45 => (Opcode::EOR, AddrMode::ZeroPage, info(2, 3)),
        0x55 => (Opcode::EOR, AddrMode::ZeroPageX, info(2, 4)),
        0x4D => (Opcode::EOR, AddrMode::Absolute, info(3, 4)),
        0x5D => (Opcode::EOR, AddrMode::AbsoluteX, info_extra(3, 4, 1)),
        0x59 => (Opcode::EOR, AddrMode::AbsoluteY, info_extra(3, 4, 1)),
        0x41 => (Opcode::EOR, AddrMode::IndexedIndirect, info(2, 6)),
        0x51 => (Opcode::EOR, AddrMode::IndirectIndexed, info_extra(2, 5, 1)),

        0xE6 => (Opcode::INC, AddrMode::ZeroPage, info(2, 5)),
        0xF6 => (Opcode::INC, AddrMode::ZeroPageX, info(2, 6)),
        0xEE => (Opcode::INC, AddrMode::Absolute, info(3, 6)),
        0xFE => (Opcode::INC, AddrMode::AbsoluteX, info(3, 7)),

        0xE8 => (Opcode::INX, AddrMode::Implicit, info(1, 2)),
        0xC8 => (Opcode::INY, AddrMode::Implicit, info(1, 2)),

        0x4C => (Opcode::JMP, AddrMode::Absolute, info(3, 3)),
        0x6C => (Opcode::JMP, AddrMode::Indirect, info(3, 5)),
        0x20 => (Opcode::JSR, AddrMode::Absolute, info(3, 6)),

        0xA9 => (Opcode::LDA, AddrMode::Immediate, info(2, 2)),
        0xA5 => (Opcode::LDA, AddrMode::ZeroPage, info(2, 3)),
        0xB5 => (Opcode::LDA, AddrMode::ZeroPageX, info(2, 4)),
        0xAD => (Opcode::LDA, AddrMode::Absolute, info(3, 4)),
        0xBD => (Opcode::LDA, AddrMode::AbsoluteX, info_extra(3, 4, 1)),
        0xB9 => (Opcode::LDA, AddrMode::AbsoluteY, info_extra(3, 4, 1)),
        0xA1 => (Opcode::LDA, AddrMode::IndexedIndirect, info(2, 6)),
        0xB1 => (Opcode::LDA, AddrMode::IndirectIndexed, info_extra(2, 5, 1)),

        0xA2 => (Opcode::LDX, AddrMode::Immediate, info(2, 2)),
        0xA6 => (Opcode::LDX, AddrMode::ZeroPage, info(2, 3)),
        0xB6 => (Opcode::LDX, AddrMode::ZeroPageY, info(2, 4)),
        0xAE => (Opcode::LDX, AddrMode::Absolute, info(3, 4)),
        0xBE => (Opcode::LDX, AddrMode::AbsoluteY, info_extra(3, 4, 1)),

        0xA0 => (Opcode::LDY, AddrMode::Immediate, info(2, 2)),
        0xA4 => (Opcode::LDY, AddrMode::ZeroPage, info(2, 3)),
        0xB4 => (Opcode::LDY, AddrMode::ZeroPageX, info(2, 4)),
        0xAC => (Opcode::LDY, AddrMode::Absolute, info(3, 4)),
        0xBC => (Opcode::LDY, AddrMode::AbsoluteX, info_extra(3, 4, 1)),

        0x4A => (Opcode::LSR, AddrMode::Accumulator, info(1, 2)),
        0x46 => (Opcode::LSR, AddrMode::ZeroPage, info(2, 5)),
        0x56 => (Opcode::LSR, AddrMode::ZeroPageX, info(2, 6)),
        0x4E => (Opcode::LSR, AddrMode::Absolute, info(3, 6)),
        0x5E => (Opcode::LSR, AddrMode::AbsoluteX, info(3, 7)),

        0xEA => (Opcode::NOP, AddrMode::Implicit, info(1, 2)),

        0x09 => (Opcode::ORA, AddrMode::Immediate, info(2, 2)),
        0x05 => (Opcode::ORA, AddrMode::ZeroPage, info(2, 3)),
        0x15 => (Opcode::ORA, AddrMode::ZeroPageX, info(2, 4)),
        0x0D => (Opcode::ORA, AddrMode::Absolute, info(3, 4)),
        0x1D => (Opcode::ORA, AddrMode::AbsoluteX, info_extra(3, 4, 1)),
        0x19 => (Opcode::ORA, AddrMode::AbsoluteY, info_extra(3, 4, 1)),
        0x01 => (Opcode::ORA, AddrMode::IndexedIndirect, info(2, 6)),
        0x11 => (Opcode::ORA, AddrMode::IndirectIndexed, info_extra(2, 5, 1)),

        0x48 => (Opcode::PHA, AddrMode::Implicit, info(1, 3)),
        0x08 => (Opcode::PHP, AddrMode::Implicit, info(1, 3)),
        0x68 => (Opcode::PLA, AddrMode::Implicit, info(1, 4)),
        0x28 => (Opcode::PLP, AddrMode::Implicit, info(1, 4)),

        0x2A => (Opcode::ROL, AddrMode::Accumulator, info(1, 2)),
        0x26 => (Opcode::ROL, AddrMode::ZeroPage, info(2, 5)),
        0x36 => (Opcode::ROL, AddrMode::ZeroPageX, info(2, 6)),
        0x2E => (Opcode::ROL, AddrMode::Absolute, info(3, 6)),
        0x3E => (Opcode::ROL, AddrMode::AbsoluteX, info(3, 7)),

        0x6A => (Opcode::ROR, AddrMode::Accumulator, info(1, 2)),
        0x66 => (Opcode::ROR, AddrMode::ZeroPage, info(2, 5)),
        0x76 => (Opcode::ROR, AddrMode::ZeroPageX, info(2, 6)),
        0x6E => (Opcode::ROR, AddrMode::Absolute, info(3, 6)),
        0x7E => (Opcode::ROR, AddrMode::AbsoluteX, info(3, 7)),

        0x40 => (Opcode::RTI, AddrMode::Implicit, info(1, 6)),
        0x60 => (Opcode::RTS, AddrMode::Implicit, info(1, 6)),

        0xE9 => (Opcode::SBC, AddrMode::Immediate, info(2, 2)),
        0xE5 => (Opcode::SBC, AddrMode::ZeroPage, info(2, 3)),
        0xF5 => (Opcode::SBC, AddrMode::ZeroPageX, info(2, 4)),
        0xED => (Opcode::SBC, AddrMode::Absolute, info(3, 4)),
        0xFD => (Opcode::SBC, AddrMode::AbsoluteX, info_extra(3, 4, 1)),
        0xF9 => (Opcode::SBC, AddrMode::AbsoluteY, info_extra(3, 4, 1)),
        0xE1 => (Opcode::SBC, AddrMode::IndexedIndirect, info(2, 6)),
        0xF1 => (Opcode::SBC, AddrMode::IndirectIndexed, info_extra(2, 5, 1)),

        0x38 => (Opcode::SEC, AddrMode::Implicit, info(1, 2)),
        0xF8 => (Opcode::SED, AddrMode::Implicit, info(1, 2)),
        0x78 => (Opcode::SEI, AddrMode::Implicit, info(1, 2)),

        0x85 => (Opcode::STA, AddrMode::ZeroPage, info(2, 3)),
        0x95 => (Opcode::STA, AddrMode::ZeroPageX, info(2, 4)),
        0x8D => (Opcode::STA, AddrMode::Absolute, info(3, 4)),
        0x9D => (Opcode::STA, AddrMode::AbsoluteX, info(3, 5)),
        0x99 => (Opcode::STA, AddrMode::AbsoluteY, info(3, 5)),
        0x81 => (Opcode::STA, AddrMode::IndexedIndirect, info(2, 6)),
        0x91 => (Opcode::STA, AddrMode::IndirectIndexed, info(2, 6)),

        0x86 => (Opcode::STX, AddrMode::ZeroPage, info(2, 3)),
        0x96 => (Opcode::STX, AddrMode::ZeroPageY, info(2, 4)),
        0x8E => (Opcode::STX, AddrMode::Absolute, info(3, 4)),

        0x84 => (Opcode::STY, AddrMode::ZeroPage, info(2, 3)),
        0x94 => (Opcode::STY, AddrMode::ZeroPageX, info(2, 4)),
        0x8C => (Opcode::STY, AddrMode::Absolute, info(3, 4)),

        0xAA => (Opcode::TAX, AddrMode::Implicit, info(1, 2)),
        0xA8 => (Opcode::TAY, AddrMode::Implicit, info(1, 2)),
        0xBA => (Opcode::TSX, AddrMode::Implicit, info(1, 2)),
        0x8A => (Opcode::TXA, AddrMode::Implicit, info(1, 2)),
        0x9A => (Opcode::TXS, AddrMode::Implicit, info(1, 2)),
        0x98 => (Opcode::TYA, AddrMode::Implicit, info(1, 2)),
        other => panic!("Invalid Opcode: `{other:X}`"),
    }
}
