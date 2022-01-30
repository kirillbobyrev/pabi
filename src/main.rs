use clap::StructOpt;
use pabi::chess::position::Position;
use pabi::Opts;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let _opts = Opts::parse();
    // TODO: Allow configuring tracer.
    tracing_subscriber::fmt::init();
    pabi::log_system_info();
    let mut rl = Editor::<()>::new();
    let mut position = Position::starting();
    loop {
        let readline = rl.readline("pabi> ");
        match readline {
            Ok(line) => {
                if line.starts_with("position ") {
                    position = Position::try_from(&line["position ".len()..]).unwrap();
                } else if line == "d" {
                    println!("{position:?}");
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            },
        }
    }
}
