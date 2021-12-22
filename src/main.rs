use clap::Parser;
use rustyline::error::ReadlineError;

fn main() {
    // E.g. `RUST_LOG=info ./pabi` to set the logging level through shell environment.
    env_logger::init();
    pabi::log_system_info();
    let _opts = pabi::Opts::parse();
    // TODO: Implement command completer.
    // TODO: Store history (?).
    let mut rl = rustyline::Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
