use pabi::uci;

fn main() {
    println!("pabi {}", pabi::VERSION);
    println!("{}", pabi::BUILD_INFO);
    println!();

    pabi::print_system_info();
    println!();

    uci::run_loop(&mut std::io::stdin().lock(), &mut std::io::stdout());
}
