#[derive(Clone, Debug)]
pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub mirroring: Mirroring,
}

impl Rom {
    pub fn new(data: &[u8]) -> Result<Rom, String> {
        parse_ines(data)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

const NES_MAGIC: [u8; 4] = *b"NES\x1A";
fn parse_ines(data: &[u8]) -> Result<Rom, String> {
    if data[0..4] != NES_MAGIC {
        return Err(String::from("Expected NES magic number"));
    }

    let [prg_rom_size, chr_rom_size, flags6, flags7, _prg_ram_len, ..] = data[4..] else {
        return Err(String::from("too short"));
    };

    let prg_rom_size = prg_rom_size as usize * 0x4000;
    let chr_rom_size = chr_rom_size as usize * 0x2000;

    let rom_mapper_lower = flags6 >> 4;
    let four_screen = (flags6 >> 3) & 0x1 != 0;
    let trainer = (flags6 >> 2) & 0x1 != 0;
    // let battery_ram = (flags6 >> 1) & 0x1 != 0;
    let vert_horiz = flags6 & 0x1 != 0;

    let rom_mapper_upper = flags7 >> 4;
    let ines_fmt_bits = (flags7 >> 2) & 0b11;

    if ines_fmt_bits != 0 {
        return Err(String::from("Only NES1.0 supported"));
    }

    let mirroring = match (four_screen, vert_horiz) {
        (true, _) => Mirroring::FourScreen,
        (false, true) => Mirroring::Vertical,
        (false, false) => Mirroring::Horizontal,
    };

    let trainer_offset = 512 & (-(trainer as isize) as usize);

    let mapper = rom_mapper_lower & (rom_mapper_upper << 4);
    let prg_rom_start = 16 + trainer_offset;
    let chr_rom_start = prg_rom_start + prg_rom_size;

    Ok(Rom {
        prg_rom: Vec::from(&data[prg_rom_start..prg_rom_start + prg_rom_size]),
        chr_rom: Vec::from(&data[chr_rom_start..chr_rom_start + chr_rom_size]),
        mapper,
        mirroring,
    })
}
