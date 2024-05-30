use pabi::uci;

fn main() {
    pabi::print_binary_info();

    uci::run_loop(&mut std::io::stdin().lock(), &mut std::io::stdout());
}
