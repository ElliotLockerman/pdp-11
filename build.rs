
fn main() {
    // lalrpop::process_root().unwrap();
    lalrpop::Configuration::new().process_file("src/assembler/grammar.lalrpop").unwrap();
}

