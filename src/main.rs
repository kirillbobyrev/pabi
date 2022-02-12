use pabi::chess::position::Position;
use std::io;
use std::io::prelude::*;

fn main() {
    pabi::print_system_info();
    let mut position = Position::starting();
    let stdin = io::stdin();
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
