
mod common {
    pub mod asm;
}

mod emulator {
    pub mod emulator;
}

use emulator::emulator::{Emulator, MAX_MEM};

use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "emu", about = "Intel 8008 Emulator")]
struct Opt {
    #[structopt(help = "Binary to execute")]
    bin: String,
}



fn main() {
    let opt = Opt::from_args();
    let mut emu = Emulator::new(MAX_MEM);

    let buf = std::fs::read(opt.bin).unwrap();
    emu.load_image(buf.as_slice(), 0);

    emu.run();
}


#[cfg(test)]
mod tests {
    use super::emulator::emulator::{Emulator, DATA_START};
    use super::common::asm::Reg;

    fn to_u8(arr: &[u16]) -> Vec<u8> {
        let mut out = Vec::new();
        for word in arr {
            out.push(*word as u8);
            out.push((word >> 8) as u8);
        }
        out
    }

    #[test]
    fn halt() {
        let bin = to_u8(&[
            0, // halt
        ]);

        let mut emu = Emulator::new(2 * DATA_START);
        emu.load_image(bin.as_slice(), DATA_START);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + 2);
    }

    #[test]
    fn mov_reg_reg() {
        let bin = to_u8(&[
            0o10001, // mov r0, r1
            0, // halt
        ]);
        let val = 0xabcd;

        let mut emu = Emulator::new(2 * DATA_START);
        emu.load_image(bin.as_slice(), DATA_START);
        emu.reg_write_word(Reg::R0, val);
        assert_eq!(emu.reg_read_word(Reg::R1), 0);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R1), val);
    }

    #[test]
    fn mov_imm_reg() {
        let val = 0xabcd;
        let bin = to_u8(&[
            0o12700, val, // mov #0xabcd, r0
            0,            // halt
        ]);

        let mut emu = Emulator::new(2 * DATA_START);
        emu.load_image(bin.as_slice(), DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), 0);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), val);
    }
    
    #[test]
    fn add() {
        let bin = to_u8(&[
            0x15c0, 0, // mov #0, r0
            0x65c0, 1, // add #1, r0
            0x0000     // halt
        ]);

        let mut emu = Emulator::new(2 * DATA_START);
        emu.load_image(bin.as_slice(), DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), 0);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), 1);
    }


    #[test]
    fn looop() {
        let bin = to_u8(&[
            0o12700, 0,     // mov #0, r0
            0o12701, 10,    // mov #10, r1

            0o62700, 1,    // add #1, r0
            0o162701, 1,    // sub #1, r1
            0o1373,         // bne -10

            0               // halt
        ]);

        let mut emu = Emulator::new(2 * DATA_START);
        emu.load_image(bin.as_slice(), DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), 0);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), 10);
    }
}

