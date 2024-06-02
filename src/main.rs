fn main() -> anyhow::Result<()> {
    pabi::print_engine_info();
    pabi::print_binary_info();

    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    let mut engine = pabi::Engine::new(&mut input, &mut output);
    engine.uci_loop()
}
