use clap::StructOpt;
use pabi::chess::position::Position;
use pabi::Opts;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let _opts = Opts::parse();
    pabi::print_system_info();
    let mut rl = Editor::<()>::new();
    let mut position = Position::starting();
    loop {
        let readline = rl.readline("pabi> ");
        match readline {
            Ok(line) => {
                if let Some(fen) = line.strip_prefix("position ") {
                    position = match Position::try_from(fen) {
                        Ok(pos) => pos,
                        Err(e) => {
                            println!("Error reading the position: {e}");
                            continue;
                        },
                    };
                } else if line == "moves" {
                    println!("{:?}", position.generate_moves());
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
