use std::io::BufWriter;
use std::io::Write;

use nes::{Bus, Cpu, rom::Rom, trace::trace};

#[test]
fn main() {
    const CODE: &[u8] = include_bytes!("../nestest.nes");
    const CORRECT_LOG: &[u8] = include_bytes!("../nestest.txt");
    let rom = Rom::new(CODE).unwrap();
    let bus = Bus::new(rom);
    let mut cpu = Cpu::new(bus);
    cpu.reset();
    // cpu.pc = 0xc000;

    let mut file = BufWriter::new(std::fs::File::create("my_log.txt").unwrap());

    cpu.run_with_callback(|cpu| {
        let line = trace(cpu);
        writeln!(file, "{}", line).unwrap();
        file.flush().unwrap();
        println!("{}", line);
    });
}
