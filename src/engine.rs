//! The engine puts all pieces together and manages resources effectively. It
//! implements the [Universal Chess Interface] (UCI) for communication with the
//! client (e.g. tournament runner with other engines or GUI/Lichess endpoint).
//!
//! [`Engine::uci_loop`] is the "main loop" of the engine which communicates with
//! the environment and executes commands from the input stream.
/// [Universal Chess Interface]: https://www.chessprogramming.org/UCI
use core::panic;
use std::io::{BufRead, Write};

use crate::chess::position::Position;

pub struct Engine {
    position: Position,
    search_state: crate::search::SearchState,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            position: Position::starting(),
            search_state: crate::search::SearchState::new(),
        }
    }

    /// Continuously reads the input stream and executes sent UCI commands until
    /// "quit" is sent or it is shut down.
    ///
    /// The implementation here does not aim to be complete and exhaustive,
    /// because the main goal is to make the engine work in relatively
    /// simple setups, making it work with all UCI-compatible GUIs and
    /// corrupted input is not a priority.
    ///
    /// Reads UCI commands from the input stream and executes them accordingly
    /// while writing the responses to the output stream.
    ///
    /// The minimal set of supported commands is:
    ///     - uci
    ///     - isready
    ///     - go
    ///     - go wtime btime winc binc
    ///     - quit
    ///     - ucinewgame
    ///     - setoption
    // TODO: Document the expected behavior.
    // > The engine must always be able to process input from stdin, even while
    // > thinking.
    pub fn uci_loop(
        &mut self,
        input: &mut impl BufRead,
        output: &mut impl Write,
    ) -> anyhow::Result<()> {
        loop {
            let mut line = String::new();

            match input.read_line(&mut line) {
                // EOF reached.
                Ok(0) => break,
                Ok(_) => {},
                Err(e) => panic!("Error reading from input: {}", e),
            }

            let tokens: Vec<&str> = line.split_whitespace().collect();

            match tokens.first() {
                // `uci` is the first command sent to the engine. The response
                // should be `id` and `uciok` followed by all supported options.
                Some(&"uci") => {
                    writeln!(
                        output,
                        "id name {} {}",
                        env!("CARGO_PKG_NAME"),
                        crate::get_version()
                    )?;
                    writeln!(output, "id author {}", env!("CARGO_PKG_AUTHORS"))?;
                    writeln!(output, "uciok")?;

                    // These options don't mean anything for now.
                    writeln!(
                        output,
                        "option name Threads type spin default 1 min 1 max 1"
                    )?;
                    writeln!(output, "option name Hash type spin default 1 min 1 max 1")?;
                },
                // This is a "health check" command. It is usually used to wait
                // for the engine to load necessary files (tablebase, eval
                // weights) or to check that the engine is responsive.
                Some(&"isready") => {
                    println!("readyok");
                },
                // Sets the engine parameter.
                // TODO: Add support for threads, hash size, Syzygy tablebase
                // path.
                Some(&"setoption") => {
                    writeln!(
                        output,
                        "info string `setoption` is no-op for now: received command {line}"
                    )?;
                },
                // Notifies the engine that the next search will be in a new
                // game. For now, it is no-op, in the future it should be the
                // same as `stop`.
                Some(&"ucinewgame") => {
                    // TODO: Stop search, reset the board, etc.
                    todo!();
                },
                // Sets up the position search will start from.
                Some(&"position") => {
                    if tokens.len() < 2 {
                        writeln!(output, "info string Missing position specification")?;
                        continue;
                    }
                    // Set the position.
                    match tokens[1] {
                        "startpos" => {
                            self.position = Position::starting();
                        },
                        "fen" => {
                            const FEN_SIZE: usize = 6;
                            if tokens.len() < 2 + FEN_SIZE {
                                writeln!(
                                    output,
                                    "info string FEN consists of 6 pieces, got {}",
                                    tokens.len() - 2
                                )?;
                            }
                        },
                        _ => {
                            writeln!(
                                output,
                                "info string Expected position [fen <fenstring> | startpos] moves
                                <move1> ... <move_i>, got: {line}"
                            )?;
                        },
                    }
                    if tokens.len() > 2 && tokens[2] == "moves" {
                        // Handle moves
                        for token in tokens.iter().skip(3) {
                            // Process the move
                            todo!();
                        }
                    }
                },
                //
                Some(&"go") => {
                    todo!();
                },
                // TODO: Stop calculating as soon as possible.
                Some(&"stop") => {
                    todo!();
                },
                Some(&"quit") => {
                    // TODO: Stop the search.
                    break;
                },
                Some(&command) => {
                    writeln!(output, "info string Unsupported command: {command}")?;
                },
                None => {},
            }
        }
        Ok(())
    }
}

// TODO: Add extensive test suite for the UCI protocol implementation.
