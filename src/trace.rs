use crate::{
    Cpu,
    fetch_decode::{AddrMode, Opcode, decode},
};

pub fn trace(cpu: &Cpu) -> String {
    let code = cpu.memory.read(cpu.pc);
    let (opcode, addrmode, info) = decode(code);

    let pc = cpu.pc;
    let mut hex_dump = vec![];
    hex_dump.push(code);

    let (mem_addr, stored_value) = match addrmode {
        AddrMode::Immediate | AddrMode::Implicit | AddrMode::Relative | AddrMode::Accumulator => {
            (0, 0)
        }
        _ => {
            let addr = cpu.get_addr_mode_dest_ext(addrmode, pc);
            (addr, cpu.memory.read(addr))
        }
    };

    let tmp = match info.size {
        1 => match code {
            0x0a | 0x4a | 0x2a | 0x6a => "A ".to_string(),
            _ => String::from(""),
        },
        2 => {
            let address: u8 = cpu.memory.read(pc + 1);
            // let value = cpu.mem_read(address));
            hex_dump.push(address);

            match addrmode {
                AddrMode::Immediate => format!("#${:02x}", address),
                AddrMode::ZeroPage => format!("${:02x} = {:02x}", mem_addr, stored_value),
                AddrMode::ZeroPageX => format!(
                    "${:02x},X @ {:02x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddrMode::ZeroPageY => format!(
                    "${:02x},Y @ {:02x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddrMode::IndexedIndirect => format!(
                    "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                    address,
                    (address.wrapping_add(cpu.reg_x)),
                    mem_addr,
                    stored_value
                ),
                AddrMode::IndirectIndexed => format!(
                    "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                    address,
                    (mem_addr.wrapping_sub(cpu.reg_y as u16)),
                    mem_addr,
                    stored_value
                ),
                AddrMode::Implicit | AddrMode::Relative => {
                    // assuming local jumps: BNE, BVS, etc....
                    let address: usize = (pc as usize + 2).wrapping_add((address as i8) as usize);
                    format!("${:04x}", address)
                }

                _ => unreachable!(
                    "unexpected addressing mode {:?} has ops-len 2. code {:02x}",
                    addrmode, code
                ),
            }
        }
        3 => {
            let address_lo = cpu.memory.read(pc + 1);
            let address_hi = cpu.memory.read(pc + 2);
            hex_dump.push(address_lo);
            hex_dump.push(address_hi);

            let address = cpu.memory.read_u16(pc + 1);

            match addrmode {
                AddrMode::Relative | AddrMode::Indirect => {
                    if code == 0x6c {
                        //jmp indirect
                        let jmp_addr = if address & 0x00FF == 0x00FF {
                            let lo = cpu.memory.read(address);
                            let hi = cpu.memory.read(address & 0xFF00);
                            ((hi as u16) << 8) | (lo as u16)
                        } else {
                            cpu.memory.read_u16(address)
                        };

                        dbg!("ran");
                        // let jmp_addr = cpu.mem_read_u16(address);
                        format!("(${:04x}) = {:04x}", address, jmp_addr)
                    } else {
                        format!("${:04x}", address)
                    }
                }
                AddrMode::Absolute => {
                    match opcode {
                        Opcode::JMP | Opcode::JSR => {
                            format!("${:04x}", mem_addr)
                        }
                        _ => {
                            format!("${:04x} = {:02x}", mem_addr, stored_value)
                        }
                    }
                    // if let Opcode::JMP = opcode {
                    //     format!("${:04x}", mem_addr)
                    // } else {
                    //     format!("${:04x} = {:02x}", mem_addr, stored_value)
                    // }
                }
                AddrMode::AbsoluteX => format!(
                    "${:04x},X @ {:04x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddrMode::AbsoluteY => format!(
                    "${:04x},Y @ {:04x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                _ => panic!(
                    "unexpected addressing mode {:?} has ops-len 3. code {:02x}",
                    addrmode, code
                ),
            }
        }
        _ => String::from(""),
    };

    let hex_str = hex_dump
        .iter()
        .map(|z| format!("{:02x}", z))
        .collect::<Vec<String>>()
        .join(" ");
    let asm_str = format!("{:04x}  {:8}  {: >4?} {}", pc, hex_str, opcode, tmp)
        .trim()
        .to_string();

    format!(
        "{:48} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}",
        asm_str, cpu.reg_a, cpu.reg_x, cpu.reg_y, cpu.status, cpu.stack_ptr,
    )
    .to_ascii_uppercase()
}
