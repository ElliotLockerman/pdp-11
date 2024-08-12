use as_lib::assemble;
use emu_lib::Emulator;
use common::asm::Reg;
use common::constants::DATA_START;
use crate::flags::{Flags, flags};

// Because each test is run on a fresh emulator, unaffected flags will be false
fn run(
    ins: &str,
    r0_init: u16,
    r0_exp: u16,
    flags_exp: Flags,
) {
    let asm = format!(r#"
        {ins} r0
        halt
    "#);
    let bin = assemble(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.get_state_mut().reg_write_word(Reg::R0, r0_init);
    emu.run_at(DATA_START);
    assert_eq!(emu.get_state().reg_read_word(Reg::R0), r0_exp);
    let status = emu.get_state().get_status();
    assert_eq!(status.get_carry(), flags_exp.c, "carry flag");
    assert_eq!(status.get_overflow(),flags_exp.v, "overflow flag");
    assert_eq!(status.get_zero(), flags_exp.z, "zero flag");
    assert_eq!(status.get_negative(), flags_exp.n, "negative flag");
    assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn clr() {
    run("clr", 0o0, 0, flags().z());
    run("clr", 0o4, 0, flags().z());
    run("clr", 0o377, 0, flags().z());
    run("clr", 0o177777, 0, flags().z());
    run("clr", 0o1777, 0, flags().z());
}

#[test]
fn inc() {
    run("inc", 0o0, 0o1, flags());
    run("inc", 0o7, 0o10, flags());
    run("inc", 0o177777, 0o0, flags().z());
    run("inc", 0o177677, 0o177700, flags().n());
    run("inc", 0o077777, 0o100000, flags().n().v());
}

#[test]
fn dec() {
    run("dec", 0o1, 0o0, flags().z());
    run("dec", 0o7, 0o6, flags());
    run("dec", 0o0, 0o177777, flags().n());
    run("dec", 0o100000, 0o077777, flags().v());
    run("dec", 0o177777, 0o177776, flags().n());
    run("dec", 0o177000, 0o176777, flags().n());
}

#[test]
fn neg() {
    run("neg", 0o0, 0o0, flags().z());
    run("neg", 0o1, 0o177777, flags().n().c());
    run("neg", 0o177777, 0o1, flags().c());
    run("neg", 0o077777, 0o100001, flags().n().c());
    run("neg", 0o100000, 0o100000, flags().n().c().v());
    run("neg", 0o6, 0o177772, flags().n().c());
}

#[test]
fn tst() {
    run("tst", 0o0, 0o0, flags().z());
    run("tst", 0o1, 0o1, flags().n());
    run("tst", 0o100, 0o100, flags().n());
    run("tst", 0o100000, 0o100000, flags().n());
    run("tst", 0o100001, 0o100001, flags());
    run("tst", 0o107001, 0o107001, flags());
}


#[test]
fn com() {
    run("com", 0o0, 0o177777, flags().n().c());
    run("com", 0o177777, 0o0, flags().z().c());
    run("com", 0o133333, 0o044444, flags().c());
    run("com", 0o134343, 0o043434, flags().c());
}


#[test]
fn swab() {
    run("swab", 0x0, 0x0, flags().z());
    run("swab", 0x00ff, 0xff00, flags().z());
    run("swab", 0xff00, 0x00ff, flags().n());
    run("swab", 0xbeef, 0xefbe, flags().n());
}

#[test]
fn asr() {
    run("asr", 0o0, 0o0, flags().z());
    run("asr", 0o1, 0o0, flags().z().c().v());
    run("asr", 0o2, 0o1, flags());
    run("asr", 0o50, 0o24, flags());
    run("asr", 0o100000, 0o140000, flags().n().v());
    run("asr", 0o177777, 0o177777, flags().n().c());
    run("asr", 0o077777, 0o037777, flags().c().v());
}

#[test]
fn asl() {
    run("asl", 0o0, 0o0, flags().z());
    run("asl", 0o1, 0o2, flags());
    run("asl", 0o50, 0o120, flags());
    run("asl", 0o077777, 0o177776, flags().n().v());
    run("asl", 0o177777, 0o177776, flags().n().c());
}

#[test]
fn clrb() {
    run("clrb", 0o0, 0, flags().z());
    run("clrb", 0o4, 0, flags().z());
    run("clrb", 0o377, 0, flags().z());
    run("clrb", 0xffff, 0xff00, flags().z());
    run("clrb", 0x3ff, 0x0300, flags().z());
}

#[test]
fn incb() {
    run("incb", 0o0, 0o1, flags());
    run("incb", 0o7, 0o10, flags());
    run("incb", 0xffff, 0xff00, flags().z());
    run("incb", 0xff6f, 0xff70, flags());
    run("incb", 0xff7f, 0xff80, flags().n().v());
}

#[test]
fn decb() {
    run("decb", 0o1, 0o0, flags().z());
    run("decb", 0o7, 0o6, flags());
    run("decb", 0x0, 0xff, flags().n());
    run("decb", 0xff00, 0xffff, flags().n());
    run("decb", 0xff80, 0xff7f, flags().v());
    run("decb", 0xffff, 0xfffe, flags().n());
    run("decb", 0xfff0, 0xffef, flags().n());
}

#[test]
fn negb() {
    run("negb", 0x0, 0x0, flags().z());
    run("negb", 0x1, 0xff, flags().n().c());
    run("negb", 0xffff, 0xff01, flags().c());
    run("negb", 0xff7f, 0xff81, flags().n().c());
    run("negb", 0xff80, 0xff80, flags().n().c().v());
    run("negb", 0x7706, 0x77fa, flags().n().c());
}

#[test]
fn tstb() {
    run("tstb", 0x0, 0x0, flags().z());
    run("tstb", 0x1, 0x1, flags().n());
    run("tstb", 0x5, 0x5, flags().n());
    run("tstb", 0xff80, 0xff80, flags().n());
    run("tstb", 0xfa81, 0xfa81, flags());
    run("tstb", 0xff01, 0xff01, flags().n());
}

#[test]
fn comb() {
    run("comb", 0x0, 0xff, flags().c().n());
    run("comb", 0xffff, 0xff00, flags().c().z());
    run("comb", 0xff38, 0xffc7, flags().c().n());
}

#[test]
fn asrb() {
    run("asrb", 0x0, 0x0, flags().z());
    run("asrb", 0x1, 0x0, flags().z().c().v());
    run("asrb", 0x2, 0x1, flags());
    run("asrb", 0xaa50, 0xaa28, flags());
    run("asrb", 0xaa80, 0xaac0, flags().n().v());
    run("asrb", 0xaaff, 0xaaff, flags().n().c());
    run("asrb", 0xaa7f, 0xaa3f, flags().c().v());
}

#[test]
fn aslb() {
    run("aslb", 0x0, 0x0, flags().z());
    run("aslb", 0x1, 0x2, flags());
    run("aslb", 0xaa30, 0xaa60, flags());
    run("aslb", 0xaa7f, 0xaafe, flags().n().v());
    run("aslb", 0xaaff, 0xaafe, flags().n().c());
}

////////////////////////////////////////////////////////////////////////////////

// multiprecision
fn run_mp(
    ins: &str,
    r0_init: u16,
    r0_exp: u16,
    flags_init: Flags,
    flags_exp: Flags,
) {
    let asm = format!(r#"
        {ins} r0
        halt
    "#);
    let bin = assemble(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.get_state_mut().reg_write_word(Reg::R0, r0_init);
    emu.get_state_mut().get_status_mut().set_flags(flags_init.to_bits());
    emu.run_at(DATA_START);
    assert_eq!(emu.get_state().reg_read_word(Reg::R0), r0_exp);
    let status = emu.get_state().get_status();
    assert_eq!(status.get_carry(), flags_exp.c, "carry flag");
    assert_eq!(status.get_overflow(),flags_exp.v, "overflow flag");
    assert_eq!(status.get_zero(), flags_exp.z, "zero flag");
    assert_eq!(status.get_negative(), flags_exp.n, "negative flag");
    assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn adc() {
    run_mp("adc", 0o0, 0o0, flags(), flags().z());
    run_mp("adc", 0o0, 0o1, flags().c(), flags());
    run_mp("adc", 0o7, 0o10, flags().c(), flags());
    run_mp("adc", 0o7, 0o7, flags(), flags());
    run_mp("adc", 0o177777, 0o0, flags().c(), flags().z().c());
    run_mp("adc", 0o177777, 0o177777, flags(), flags().n());
    run_mp("adc", 0o077777, 0o100000, flags().c(), flags().n().v());
    run_mp("adc", 0o100000, 0o100001, flags().c(), flags().n());
    run_mp("adc", 0o100000, 0o100000, flags(), flags().n());
}

#[test]
fn sbc() {
    run_mp("sbc", 0o0, 0o0, flags(), flags().c().z());
    run_mp("sbc", 0o1, 0o0, flags().c(), flags().z());
    run_mp("sbc", 0o0, 0o177777, flags().c(), flags().c().n());
    run_mp("sbc", 0o5, 0o5, flags(), flags().c());
    run_mp("sbc", 0o5, 0o4, flags().c(), flags().c());
    run_mp("sbc", 0o100000, 0o077777, flags().c(), flags().c());
    run_mp("sbc", 0o100000, 0o100000, flags(), flags().c().v().n());
    run_mp("sbc", 0o100001, 0o100001, flags(), flags().c().n());
    run_mp("sbc", 0o100001, 0o100000, flags().c(), flags().c().v().n());
}

#[test]
fn ror() {
    run_mp("ror", 0o0, 0o0, flags(), flags().z());
    run_mp("ror", 0o1, 0o0, flags(), flags().c().v().z());
    run_mp("ror", 0o2, 0o1, flags(), flags());
    run_mp("ror", 0o50, 0o24, flags(), flags());
    run_mp("ror", 0o0, 0o100000, flags().c(), flags().n().v());
    run_mp("ror", 0o1, 0o100000, flags().c(), flags().n().c());
    run_mp("ror", 0o100000, 0o040000, flags(), flags());
    run_mp("ror", 0o100000, 0o140000, flags().c(), flags().n().v());
    run_mp("ror", 0o177777, 0o077777, flags(), flags().c().v());
    run_mp("ror", 0o177777, 0o177777, flags().c(), flags().c().n());
}


#[test]
fn rol() {
    run_mp("rol", 0o0, 0o0, flags(), flags().z());
    run_mp("rol", 0o1, 0o2, flags(), flags());
    run_mp("rol", 0o50, 0o120, flags(), flags());
    run_mp("rol", 0o0, 0o1, flags().c(), flags());
    run_mp("rol", 0o100000, 0o0, flags(), flags().z().c().v());
    run_mp("rol", 0o100000, 0o1, flags().c(), flags().c().v());
    run_mp("rol", 0o040000, 0o100000, flags(), flags().n().v());
    run_mp("rol", 0o040000, 0o100001, flags().c(), flags().n().v());
    run_mp("rol", 0o140000, 0o100001, flags().c(), flags().n().c());
    run_mp("rol", 0o177777, 0o177776, flags(), flags().n().c());
    run_mp("rol", 0o177777, 0o177777, flags().c(), flags().c().n());
}

#[test]
fn adcb() {
    run_mp("adcb", 0x0, 0x0, flags(), flags().z());
    run_mp("adcb", 0x0, 0x1, flags().c(), flags());
    run_mp("adcb", 0x7, 0x7, flags(), flags());
    run_mp("adcb", 0xff00, 0xff00, flags(), flags().z());
    run_mp("adcb", 0xab07, 0xab08, flags().c(), flags());
    run_mp("adcb", 0xdd07, 0xdd07, flags(), flags());
    run_mp("adcb", 0xddff, 0xdd00, flags().c(), flags().z().c());
    run_mp("adcb", 0xddff, 0xddff, flags(), flags().n());
    run_mp("adcb", 0xdd7f, 0xdd7f, flags(), flags());
    run_mp("adcb", 0xdd7f, 0xdd80, flags().c(), flags().n().v());
    run_mp("adcb", 0xdd80, 0xdd80, flags(), flags().n());
    run_mp("adcb", 0xdd80, 0xdd81, flags().c(), flags().n());
}

#[test]
fn sbcb() {
    run_mp("sbcb", 0x0, 0x0, flags(), flags().c().z());
    run_mp("sbcb", 0x1, 0x0, flags().c(), flags().z());
    run_mp("sbcb", 0xaa00, 0xaaff, flags().c(), flags().c().n());
    run_mp("sbcb", 0xaa05, 0xaa05, flags(), flags().c());
    run_mp("sbcb", 0xaa05, 0xaa04, flags().c(), flags().c());
    run_mp("sbcb", 0xaa80, 0xaa7f, flags().c(), flags().c());
    run_mp("sbcb", 0xaa80, 0xaa80, flags(), flags().c().v().n());
    run_mp("sbcb", 0xaa81, 0xaa81, flags(), flags().c().n());
    run_mp("sbcb", 0xaa81, 0xaa80, flags().c(), flags().c().v().n());
}

#[test]
fn rorb() {
    run_mp("rorb", 0xaa00, 0xaa00, flags(), flags().z());
    run_mp("rorb", 0xaa01, 0xaa00, flags(), flags().c().v().z());
    run_mp("rorb", 0xaa02, 0xaa01, flags(), flags());
    run_mp("rorb", 0xaa50, 0xaa28, flags(), flags());
    run_mp("rorb", 0xaa00, 0xaa80, flags().c(), flags().n().v());
    run_mp("rorb", 0xaa01, 0xaa80, flags().c(), flags().c().n());
    run_mp("rorb", 0xaa80, 0xaa40, flags(), flags());
    run_mp("rorb", 0xaa80, 0xaac0, flags().c(), flags().n().v());
    run_mp("rorb", 0xaaff, 0xaa7f, flags(), flags().c().v());
    run_mp("rorb", 0xaaff, 0xaaff, flags().c(), flags().c().n());
}

#[test]
fn rolb() {
    run_mp("rolb", 0xaa00, 0xaa00, flags(), flags().z());
    run_mp("rolb", 0xaa01, 0xaa02, flags(), flags());
    run_mp("rolb", 0xaa30, 0xaa60, flags(), flags());
    run_mp("rolb", 0xaa00, 0xaa01, flags().c(), flags());
    run_mp("rolb", 0xaa80, 0xaa00, flags(), flags().z().c().v());
    run_mp("rolb", 0xaa80, 0xaa01, flags().c(), flags().c().v());
    run_mp("rolb", 0xaa40, 0xaa80, flags(), flags().n().v());
    run_mp("rolb", 0xaa40, 0xaa81, flags().c(), flags().n().v());
    run_mp("rolb", 0xaac0, 0xaa81, flags().c(), flags().n().c());
    run_mp("rolb", 0xaaff, 0xaafe, flags(), flags().n().c());
    run_mp("rolb", 0xaaff, 0xaaff, flags().c(), flags().c().n());
}


