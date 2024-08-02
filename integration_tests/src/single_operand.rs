

#[cfg(test)]
mod tests {
    use as_lib::assemble;
    use emu_lib::Emulator;
    use common::asm::Reg;
    use common::constants::DATA_START;
    use crate::flags::{flags, Flags};

    const T: bool = true;
    const F: bool = false;

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
    fn test_clr() {
        run("clr", 0o0, 0, Flags{z:T, n:F, c:F, v:F});
        run("clr", 0o4, 0, Flags{z:T, n:F, c:F, v:F});
        run("clr", 0o377, 0, Flags{z:T, n:F, c:F, v:F});
        run("clr", 0o177777, 0, Flags{z:T, n:F, c:F, v:F});
        run("clr", 0o1777, 0, Flags{z:T, n:F, c:F, v:F});
    }

    #[test]
    fn test_inc() {
        run("inc", 0o0, 0o1, Flags{z:F, n:F, c:F, v:F});
        run("inc", 0o7, 0o10, Flags{z:F, n:F, c:F, v:F});
        run("inc", 0o177777, 0o0, Flags{z:T, n:F, c:F, v:F});
        run("inc", 0o177677, 0o177700, Flags{z:F, n:T, c:F, v:F});
        run("inc", 0o077777, 0o100000, Flags{z:F, n:T, c:F, v:T});
    }

    #[test]
    fn test_dec() {
        run("dec", 0o1, 0o0, Flags{z:T, n:F, c:F, v:F});
        run("dec", 0o7, 0o6, Flags{z:F, n:F, c:F, v:F});
        run("dec", 0o0, 0o177777, Flags{z:F, n:T, c:F, v:F});
        run("dec", 0o100000, 0o077777, Flags{z:F, n:F, c:F, v:T});
        run("dec", 0o177777, 0o177776, Flags{z:F, n:T, c:F, v:F});
        run("dec", 0o177000, 0o176777, Flags{z:F, n:T, c:F, v:F});
    }

    #[test]
    fn test_neg() {
        run("neg", 0o0, 0o0, Flags{z:T, n:F, c:F, v:F});
        run("neg", 0o1, 0o177777, Flags{z:F, n:T, c:T, v:F});
        run("neg", 0o177777, 0o1, Flags{z:F, n:F, c:T, v:F});
        run("neg", 0o077777, 0o100001, Flags{z:F, n:T, c:T, v:F});
        run("neg", 0o100000, 0o100000, Flags{z:F, n:T, c:T, v:T});
        run("neg", 0o6, 0o177772, Flags{z:F, n:T, c:T, v:F});
    }

    #[test]
    fn test_tst() {
        run("tst", 0o0, 0o0, Flags{z:T, n:F, c:F, v:F});
        run("tst", 0o1, 0o1, Flags{z:F, n:T, c:F, v:F});
        run("tst", 0o100, 0o100, Flags{z:F, n:T, c:F, v:F});
        run("tst", 0o100000, 0o100000, Flags{z:F, n:T, c:F, v:F});
        run("tst", 0o100001, 0o100001, Flags{z:F, n:F, c:F, v:F});
        run("tst", 0o107001, 0o107001, Flags{z:F, n:F, c:F, v:F});
    }


    #[test]
    fn test_com() {
        run("com", 0o0, 0o177777, Flags{z:F, n:T, c:T, v:F});
        run("com", 0o177777, 0o0, Flags{z:T, n:F, c:T, v:F});
        run("com", 0o133333, 0o044444, Flags{z:F, n:F, c:T, v:F});
        run("com", 0o134343, 0o043434, Flags{z:F, n:F, c:T, v:F});
    }


    #[test]
    fn test_swab() {
        run("swab", 0x0, 0x0, flags().z());
        run("swab", 0x00ff, 0xff00, flags().z());
        run("swab", 0xff00, 0x00ff, flags().n());
        run("swab", 0xbeef, 0xefbe, flags().n());
    }

    #[test]
    fn test_asr() {
        run("asr", 0o0, 0o0, flags().z());
        run("asr", 0o1, 0o0, flags().z().c().v());
        run("asr", 0o2, 0o1, flags());
        run("asr", 0o50, 0o24, flags());
        run("asr", 0o100000, 0o140000, flags().n().v());
        run("asr", 0o177777, 0o177777, flags().n().c());
        run("asr", 0o077777, 0o037777, flags().c().v());
    }

    #[test]
    fn test_asl() {
        run("asl", 0o0, 0o0, flags().z());
        run("asl", 0o1, 0o2, flags());
        run("asl", 0o50, 0o120, flags());
        run("asl", 0o077777, 0o177776, flags().n().v());
        run("asl", 0o177777, 0o177776, flags().n().c());
    }

    #[test]
    fn test_clrb() {
        run("clrb", 0o0, 0, flags().z());
        run("clrb", 0o4, 0, flags().z());
        run("clrb", 0o377, 0, flags().z());
        run("clrb", 0xffff, 0xff00, flags().z());
        run("clrb", 0x3ff, 0x0300, flags().z());
    }

    #[test]
    fn test_incb() {
        run("incb", 0o0, 0o1, flags());
        run("incb", 0o7, 0o10, flags());
        run("incb", 0xffff, 0xff00, flags().z());
        run("incb", 0xff6f, 0xff70, flags());
        run("incb", 0xff7f, 0xff80, flags().n().v());
    }

    #[test]
    fn test_decb() {
        run("decb", 0o1, 0o0, flags().z());
        run("decb", 0o7, 0o6, flags());
        run("decb", 0x0, 0xff, flags().n());
        run("decb", 0xff00, 0xffff, flags().n());
        run("decb", 0xff80, 0xff7f, flags().v());
        run("decb", 0xffff, 0xfffe, flags().n());
        run("decb", 0xfff0, 0xffef, flags().n());
    }

    #[test]
    fn test_negb() {
        run("negb", 0x0, 0x0, flags().z());
        run("negb", 0x1, 0xff, flags().n().c());
        run("negb", 0xffff, 0xff01, flags().c());
        run("negb", 0xff7f, 0xff81, flags().n().c());
        run("negb", 0xff80, 0xff80, flags().n().c().v());
        run("negb", 0x7706, 0x77fa, flags().n().c());
    }

    #[test]
    fn test_tstb() {
        run("tstb", 0x0, 0x0, flags().z());
        run("tstb", 0x1, 0x1, flags().n());
        run("tstb", 0x5, 0x5, flags().n());
        run("tstb", 0xff80, 0xff80, flags().n());
        run("tstb", 0xfa81, 0xfa81, flags());
        run("tstb", 0xff01, 0xff01, flags().n());
    }

    #[test]
    fn test_comb() {
        run("comb", 0x0, 0xff, flags().c().n());
        run("comb", 0xffff, 0xff00, flags().c().z());
        run("comb", 0xff38, 0xffc7, flags().c().n());
    }
}



#[cfg(test)]
mod multiple_precision_tests {
    use as_lib::assemble;
    use emu_lib::Emulator;
    use common::asm::Reg;
    use common::constants::DATA_START;
    use crate::flags::{flags, Flags};

    fn run(
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
    fn test_adc() {
        run("adc", 0o0, 0o0, flags(), flags().z());
        run("adc", 0o0, 0o1, flags().c(), flags());
        run("adc", 0o7, 0o10, flags().c(), flags());
        run("adc", 0o7, 0o7, flags(), flags());
        run("adc", 0o177777, 0o0, flags().c(), flags().z().c());
        run("adc", 0o177777, 0o177777, flags(), flags().n());
        run("adc", 0o077777, 0o100000, flags().c(), flags().n().v());
        run("adc", 0o100000, 0o100001, flags().c(), flags().n());
        run("adc", 0o100000, 0o100000, flags(), flags().n());
    }

    #[test]
    fn test_sbc() {
        run("sbc", 0o0, 0o0, flags(), flags().c().z());
        run("sbc", 0o1, 0o0, flags().c(), flags().z());
        run("sbc", 0o0, 0o177777, flags().c(), flags().c().n());
        run("sbc", 0o5, 0o5, flags(), flags().c());
        run("sbc", 0o5, 0o4, flags().c(), flags().c());
        run("sbc", 0o100000, 0o077777, flags().c(), flags().c());
        run("sbc", 0o100000, 0o100000, flags(), flags().c().v().n());
        run("sbc", 0o100001, 0o100001, flags(), flags().c().n());
        run("sbc", 0o100001, 0o100000, flags().c(), flags().c().v().n());
    }

    #[test]
    fn test_ror() {
        run("ror", 0o0, 0o0, flags(), flags().z());
        run("ror", 0o1, 0o0, flags(), flags().c().v().z());
        run("ror", 0o2, 0o1, flags(), flags());
        run("ror", 0o50, 0o24, flags(), flags());
        run("ror", 0o0, 0o100000, flags().c(), flags().n().v());
        run("ror", 0o1, 0o100000, flags().c(), flags().n().c());
        run("ror", 0o100000, 0o040000, flags(), flags());
        run("ror", 0o100000, 0o140000, flags().c(), flags().n().v());
        run("ror", 0o177777, 0o077777, flags(), flags().c().v());
        run("ror", 0o177777, 0o177777, flags().c(), flags().c().n());
    }


    #[test]
    fn test_rol() {
        run("rol", 0o0, 0o0, flags(), flags().z());
        run("rol", 0o1, 0o2, flags(), flags());
        run("rol", 0o50, 0o120, flags(), flags());
        run("rol", 0o0, 0o1, flags().c(), flags());
        run("rol", 0o100000, 0o0, flags(), flags().z().c().v());
        run("rol", 0o100000, 0o1, flags().c(), flags().c().v());
        run("rol", 0o040000, 0o100000, flags(), flags().n().v());
        run("rol", 0o040000, 0o100001, flags().c(), flags().n().v());
        run("rol", 0o140000, 0o100001, flags().c(), flags().n().c());
        run("rol", 0o177777, 0o177776, flags(), flags().n().c());
        run("rol", 0o177777, 0o177777, flags().c(), flags().c().n());
    }
    
    #[test]
    fn test_adcb() {
        run("adcb", 0x0, 0x0, flags(), flags().z());
        run("adcb", 0x0, 0x1, flags().c(), flags());
        run("adcb", 0x7, 0x7, flags(), flags());
        run("adcb", 0xff00, 0xff00, flags(), flags().z());
        run("adcb", 0xab07, 0xab08, flags().c(), flags());
        run("adcb", 0xdd07, 0xdd07, flags(), flags());
        run("adcb", 0xddff, 0xdd00, flags().c(), flags().z().c());
        run("adcb", 0xddff, 0xddff, flags(), flags().n());
        run("adcb", 0xdd7f, 0xdd7f, flags(), flags());
        run("adcb", 0xdd7f, 0xdd80, flags().c(), flags().n().v());
        run("adcb", 0xdd80, 0xdd80, flags(), flags().n());
        run("adcb", 0xdd80, 0xdd81, flags().c(), flags().n());
    }

    #[test]
    fn test_sbcb() {
        run("sbcb", 0x0, 0x0, flags(), flags().c().z());
        run("sbcb", 0x1, 0x0, flags().c(), flags().z());
        run("sbcb", 0xaa00, 0xaaff, flags().c(), flags().c().n());
        run("sbcb", 0xaa05, 0xaa05, flags(), flags().c());
        run("sbcb", 0xaa05, 0xaa04, flags().c(), flags().c());
        run("sbcb", 0xaa80, 0xaa7f, flags().c(), flags().c());
        run("sbcb", 0xaa80, 0xaa80, flags(), flags().c().v().n());
        run("sbcb", 0xaa81, 0xaa81, flags(), flags().c().n());
        run("sbcb", 0xaa81, 0xaa80, flags().c(), flags().c().v().n());
    }
}


