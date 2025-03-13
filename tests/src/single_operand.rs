use crate::flags::{C, N, V, Z, check_flags};
use as_lib::assemble_raw;
use common::asm::Reg;
use common::constants::DATA_START;
use common::mem::ToU16P;
use emu_lib::Emulator;

// Because each test is run on a fresh emulator, unaffected flags will be false
fn run(ins: &str, r0_init: u16, r0_exp: u16, flags_exp: u16) {
    let asm = format!(
        r#"
        {ins} r0
        halt
    "#
    );
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.reg_write_word(Reg::R0, r0_init);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), r0_exp);
    check_flags(&emu, flags_exp);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p()
    );
}

#[test]
fn clr() {
    run("clr", 0o0, 0, Z);
    run("clr", 0o4, 0, Z);
    run("clr", 0o377, 0, Z);
    run("clr", 0o177777, 0, Z);
    run("clr", 0o1777, 0, Z);
}

#[test]
fn inc() {
    run("inc", 0o0, 0o1, 0);
    run("inc", 0o7, 0o10, 0);
    run("inc", 0o177777, 0o0, Z);
    run("inc", 0o177677, 0o177700, N);
    run("inc", 0o077777, 0o100000, N | V);
}

#[test]
fn dec() {
    run("dec", 0o1, 0o0, Z);
    run("dec", 0o7, 0o6, 0);
    run("dec", 0o0, 0o177777, N);
    run("dec", 0o100000, 0o077777, V);
    run("dec", 0o177777, 0o177776, N);
    run("dec", 0o177000, 0o176777, N);
}

#[test]
fn neg() {
    run("neg", 0o0, 0o0, Z);
    run("neg", 0o1, 0o177777, N | C);
    run("neg", 0o177777, 0o1, C);
    run("neg", 0o077777, 0o100001, N | C);
    run("neg", 0o100000, 0o100000, N | C | V);
    run("neg", 0o6, 0o177772, N | C);
}

#[test]
fn tst() {
    run("tst", 0o0, 0o0, Z);
    run("tst", 0o1, 0o1, N);
    run("tst", 0o100, 0o100, N);
    run("tst", 0o100000, 0o100000, N);
    run("tst", 0o100001, 0o100001, 0);
    run("tst", 0o107001, 0o107001, 0);
}

#[test]
fn com() {
    run("com", 0o0, 0o177777, N | C);
    run("com", 0o177777, 0o0, Z | C);
    run("com", 0o133333, 0o044444, C);
    run("com", 0o134343, 0o043434, C);
}

#[test]
fn swab() {
    run("swab", 0x0, 0x0, Z);
    run("swab", 0x00ff, 0xff00, Z);
    run("swab", 0xff00, 0x00ff, N);
    run("swab", 0xbeef, 0xefbe, N);
}

#[test]
fn asr() {
    run("asr", 0o0, 0o0, Z);
    run("asr", 0o1, 0o0, Z | C | V);
    run("asr", 0o2, 0o1, 0);
    run("asr", 0o50, 0o24, 0);
    run("asr", 0o100000, 0o140000, N | V);
    run("asr", 0o177777, 0o177777, N | C);
    run("asr", 0o077777, 0o037777, C | V);
}

#[test]
fn asl() {
    run("asl", 0o0, 0o0, Z);
    run("asl", 0o1, 0o2, 0);
    run("asl", 0o50, 0o120, 0);
    run("asl", 0o077777, 0o177776, N | V);
    run("asl", 0o177777, 0o177776, N | C);
}

#[test]
fn clrb() {
    run("clrb", 0o0, 0, Z);
    run("clrb", 0o4, 0, Z);
    run("clrb", 0o377, 0, Z);
    run("clrb", 0xffff, 0xff00, Z);
    run("clrb", 0x3ff, 0x0300, Z);
}

#[test]
fn incb() {
    run("incb", 0o0, 0o1, 0);
    run("incb", 0o7, 0o10, 0);
    run("incb", 0xffff, 0xff00, Z);
    run("incb", 0xff6f, 0xff70, 0);
    run("incb", 0xff7f, 0xff80, N | V);
}

#[test]
fn decb() {
    run("decb", 0o1, 0o0, Z);
    run("decb", 0o7, 0o6, 0);
    run("decb", 0x0, 0xff, N);
    run("decb", 0xff00, 0xffff, N);
    run("decb", 0xff80, 0xff7f, V);
    run("decb", 0xffff, 0xfffe, N);
    run("decb", 0xfff0, 0xffef, N);
}

#[test]
fn negb() {
    run("negb", 0x0, 0x0, Z);
    run("negb", 0x1, 0xff, N | C);
    run("negb", 0xffff, 0xff01, C);
    run("negb", 0xff7f, 0xff81, N | C);
    run("negb", 0xff80, 0xff80, N | C | V);
    run("negb", 0x7706, 0x77fa, N | C);
}

#[test]
fn tstb() {
    run("tstb", 0x0, 0x0, Z);
    run("tstb", 0x1, 0x1, N);
    run("tstb", 0x5, 0x5, N);
    run("tstb", 0xff80, 0xff80, N);
    run("tstb", 0xfa81, 0xfa81, 0);
    run("tstb", 0xff01, 0xff01, N);
}

#[test]
fn comb() {
    run("comb", 0x0, 0xff, C | N);
    run("comb", 0xffff, 0xff00, C | Z);
    run("comb", 0xff38, 0xffc7, C | N);
}

#[test]
fn asrb() {
    run("asrb", 0x0, 0x0, Z);
    run("asrb", 0x1, 0x0, Z | C | V);
    run("asrb", 0x2, 0x1, 0);
    run("asrb", 0xaa50, 0xaa28, 0);
    run("asrb", 0xaa80, 0xaac0, N | V);
    run("asrb", 0xaaff, 0xaaff, N | C);
    run("asrb", 0xaa7f, 0xaa3f, C | V);
}

#[test]
fn aslb() {
    run("aslb", 0x0, 0x0, Z);
    run("aslb", 0x1, 0x2, 0);
    run("aslb", 0xaa30, 0xaa60, 0);
    run("aslb", 0xaa7f, 0xaafe, N | V);
    run("aslb", 0xaaff, 0xaafe, N | C);
}

////////////////////////////////////////////////////////////////////////////////

// multiprecision
fn run_mp(ins: &str, r0_init: u16, r0_exp: u16, flags_init: u16, flags_exp: u16) {
    let asm = format!(
        r#"
        {ins} r0
        halt
    "#
    );
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.reg_write_word(Reg::R0, r0_init);
    emu.get_state_mut().get_status_mut().set_flags(flags_init);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), r0_exp);
    check_flags(&emu, flags_exp);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p()
    );
}

#[test]
fn adc() {
    run_mp("adc", 0o0, 0o0, 0, Z);
    run_mp("adc", 0o0, 0o1, C, 0);
    run_mp("adc", 0o7, 0o10, C, 0);
    run_mp("adc", 0o7, 0o7, 0, 0);
    run_mp("adc", 0o177777, 0o0, C, Z | C);
    run_mp("adc", 0o177777, 0o177777, 0, N);
    run_mp("adc", 0o077777, 0o100000, C, N | V);
    run_mp("adc", 0o100000, 0o100001, C, N);
    run_mp("adc", 0o100000, 0o100000, 0, N);
}

#[test]
fn sbc() {
    run_mp("sbc", 0o0, 0o0, 0, C | Z);
    run_mp("sbc", 0o1, 0o0, C, Z);
    run_mp("sbc", 0o0, 0o177777, C, C | N);
    run_mp("sbc", 0o5, 0o5, 0, C);
    run_mp("sbc", 0o5, 0o4, C, C);
    run_mp("sbc", 0o100000, 0o077777, C, C);
    run_mp("sbc", 0o100000, 0o100000, 0, C | V | N);
    run_mp("sbc", 0o100001, 0o100001, 0, C | N);
    run_mp("sbc", 0o100001, 0o100000, C, C | V | N);
}

#[test]
fn ror() {
    run_mp("ror", 0o0, 0o0, 0, Z);
    run_mp("ror", 0o1, 0o0, 0, C | V | Z);
    run_mp("ror", 0o2, 0o1, 0, 0);
    run_mp("ror", 0o50, 0o24, 0, 0);
    run_mp("ror", 0o0, 0o100000, C, N | V);
    run_mp("ror", 0o1, 0o100000, C, N | C);
    run_mp("ror", 0o100000, 0o040000, 0, 0);
    run_mp("ror", 0o100000, 0o140000, C, N | V);
    run_mp("ror", 0o177777, 0o077777, 0, C | V);
    run_mp("ror", 0o177777, 0o177777, C, C | N);
}

#[test]
fn rol() {
    run_mp("rol", 0o0, 0o0, 0, Z);
    run_mp("rol", 0o1, 0o2, 0, 0);
    run_mp("rol", 0o50, 0o120, 0, 0);
    run_mp("rol", 0o0, 0o1, C, 0);
    run_mp("rol", 0o100000, 0o0, 0, Z | C | V);
    run_mp("rol", 0o100000, 0o1, C, C | V);
    run_mp("rol", 0o040000, 0o100000, 0, N | V);
    run_mp("rol", 0o040000, 0o100001, C, N | V);
    run_mp("rol", 0o140000, 0o100001, C, N | C);
    run_mp("rol", 0o177777, 0o177776, 0, N | C);
    run_mp("rol", 0o177777, 0o177777, C, C | N);
}

#[test]
fn adcb() {
    run_mp("adcb", 0x0, 0x0, 0, Z);
    run_mp("adcb", 0x0, 0x1, C, 0);
    run_mp("adcb", 0x7, 0x7, 0, 0);
    run_mp("adcb", 0xff00, 0xff00, 0, Z);
    run_mp("adcb", 0xab07, 0xab08, C, 0);
    run_mp("adcb", 0xdd07, 0xdd07, 0, 0);
    run_mp("adcb", 0xddff, 0xdd00, C, Z | C);
    run_mp("adcb", 0xddff, 0xddff, 0, N);
    run_mp("adcb", 0xdd7f, 0xdd7f, 0, 0);
    run_mp("adcb", 0xdd7f, 0xdd80, C, N | V);
    run_mp("adcb", 0xdd80, 0xdd80, 0, N);
    run_mp("adcb", 0xdd80, 0xdd81, C, N);
}

#[test]
fn sbcb() {
    run_mp("sbcb", 0x0, 0x0, 0, C | Z);
    run_mp("sbcb", 0x1, 0x0, C, Z);
    run_mp("sbcb", 0xaa00, 0xaaff, C, C | N);
    run_mp("sbcb", 0xaa05, 0xaa05, 0, C);
    run_mp("sbcb", 0xaa05, 0xaa04, C, C);
    run_mp("sbcb", 0xaa80, 0xaa7f, C, C);
    run_mp("sbcb", 0xaa80, 0xaa80, 0, C | V | N);
    run_mp("sbcb", 0xaa81, 0xaa81, 0, C | N);
    run_mp("sbcb", 0xaa81, 0xaa80, C, C | V | N);
}

#[test]
fn rorb() {
    run_mp("rorb", 0xaa00, 0xaa00, 0, Z);
    run_mp("rorb", 0xaa01, 0xaa00, 0, C | V | Z);
    run_mp("rorb", 0xaa02, 0xaa01, 0, 0);
    run_mp("rorb", 0xaa50, 0xaa28, 0, 0);
    run_mp("rorb", 0xaa00, 0xaa80, C, N | V);
    run_mp("rorb", 0xaa01, 0xaa80, C, C | N);
    run_mp("rorb", 0xaa80, 0xaa40, 0, 0);
    run_mp("rorb", 0xaa80, 0xaac0, C, N | V);
    run_mp("rorb", 0xaaff, 0xaa7f, 0, C | V);
    run_mp("rorb", 0xaaff, 0xaaff, C, C | N);
}

#[test]
fn rolb() {
    run_mp("rolb", 0xaa00, 0xaa00, 0, Z);
    run_mp("rolb", 0xaa01, 0xaa02, 0, 0);
    run_mp("rolb", 0xaa30, 0xaa60, 0, 0);
    run_mp("rolb", 0xaa00, 0xaa01, C, 0);
    run_mp("rolb", 0xaa80, 0xaa00, 0, Z | C | V);
    run_mp("rolb", 0xaa80, 0xaa01, C, C | V);
    run_mp("rolb", 0xaa40, 0xaa80, 0, N | V);
    run_mp("rolb", 0xaa40, 0xaa81, C, N | V);
    run_mp("rolb", 0xaac0, 0xaa81, C, N | C);
    run_mp("rolb", 0xaaff, 0xaafe, 0, N | C);
    run_mp("rolb", 0xaaff, 0xaaff, C, C | N);
}
