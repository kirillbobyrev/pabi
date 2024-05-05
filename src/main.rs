use std::io;
use std::io::prelude::*;

use pabi::chess::position::Position;

fn main() {
    pabi::print_system_info();
    let mut position = Position::starting();
    let stdin = io::stdin();
    // TODO: Pull into a reasonable interface.
    // TODO: Add perft.
    println!("pabi {}", pabi::VERSION);
    for line in stdin.lock().lines() {
        let line = line.unwrap();
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
    }
}
