

#[cfg(test)]
mod tests {
    use as_lib::assemble;
    use emu_lib::Emulator;
    use common::asm::Reg;
    use common::constants::DATA_START;
    use crate::flags::Flags;

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
}


