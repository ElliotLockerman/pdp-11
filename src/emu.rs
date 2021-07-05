
mod common {
    pub mod asm;
    pub mod decoder;
    pub mod mem;
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
    use super::common::mem::as_byte_slice;

    #[test]
    fn halt() {
        let bin = &[
            0, // halt
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let mut emu = Emulator::new(2 * DATA_START);
        emu.load_image(bin, DATA_START);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + 2);
    }

    #[test]
    fn mov_reg_reg() {
        let bin = &[
            0o10001, // mov r0, r1
            0, // halt
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let val = 0xabcd;
        let mut emu = Emulator::new(2 * DATA_START);
        emu.load_image(bin, DATA_START);
        emu.reg_write_word(Reg::R0, val);
        assert_eq!(emu.reg_read_word(Reg::R1), 0);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R1), val);
    }

    #[test]
    fn mov_imm_reg() {
        let val = 0xabcd;
        let bin = &[
            0o12700, val, // mov #0xabcd, r0
            0,            // halt
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let mut emu = Emulator::new(2 * DATA_START);
        emu.load_image(bin, DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), 0);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), val);
    }
    
    #[test]
    fn add() {
        let bin = &[
            0x15c0, 0, // mov #0, r0
            0x65c0, 1, // add #1, r0
            0x0000     // halt
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let mut emu = Emulator::new(2 * DATA_START);
        emu.load_image(bin, DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), 0);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), 1);
    }

    #[test]
    fn autoinc() {
        let arr = DATA_START + 18;
        let bin = &[
            0o12700, arr,   // mov  #arr, r0
            0o62720, 0o1,   // add  #1, (r0)+
            0o62720, 0o1,   // add  #1, (r0)+
            0o62720, 0o1,   // add  #1, (r0)+
            0o0,            // halt

        // arr:
            0o1, 0o2, 0o3   // .word 1 2 3
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let mut emu = Emulator::new(2 * DATA_START);
        emu.load_image(bin, DATA_START);
        assert_eq!(emu.mem_read_word(arr), 1);
        assert_eq!(emu.mem_read_word(arr + 2), 2);
        assert_eq!(emu.mem_read_word(arr + 4), 3);
        emu.run_at(DATA_START);
        assert_eq!(emu.mem_read_word(arr), 2);
        assert_eq!(emu.mem_read_word(arr + 2), 3);
        assert_eq!(emu.mem_read_word(arr + 4), 4);
    }

    #[test]
    fn looop() {
        let bin = &[
            0o12700, 0,     // mov #0, r0
            0o12701, 10,    // mov #10, r1

            0o62700, 1,     // add #1, r0
            0o162701, 1,    // sub #1, r1
            0o1373,         // bne -10

            0               // halt
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let mut emu = Emulator::new(2 * DATA_START);
        emu.load_image(bin, DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), 0);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), 10);
    }

    #[test]
    fn call() {
        let bin = &[
            0o12701, 0o0,   // mov #0, r1
            0o12702, 0o0,   // mov #0, r1
            0o407,          // br start

            0o12702, 0o2,   // mov #2, r2 ; shouldn't be executed

        // fun:
            0o12701, 0o1,   // mov #1, r1
            0o207,          // rts pc

            0o12702, 0o2,   // mov #2, r2 ; shouldn't be executed

        // start:
            0o4767, 0o177764,   // jsr pc, fun
            0o0                 // halt
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let mut emu = Emulator::new(3 * DATA_START);
        emu.load_image(bin, DATA_START);
        emu.reg_write_word(Reg::SP, 2 * DATA_START);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R1), 1);
        assert_eq!(emu.reg_read_word(Reg::R2), 0);
    }
}

