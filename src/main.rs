fn main() -> anyhow::Result<()> {
    pabi::print_engine_info();
    pabi::print_binary_info();

    let mut engine = pabi::Engine::new();
    engine.uci_loop(&mut std::io::stdin().lock(), &mut std::io::stdout())
}
