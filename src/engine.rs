//! The engine puts all pieces together and manages resources effectively. It
//! implements the [Universal Chess Interface] (UCI) for communication with the
//! client (e.g. tournament runner with other engines or GUI/Lichess endpoint).
//!
//! [`Engine::uci_loop`] is the "main loop" of the engine which communicates with
//! the environment and executes commands from the input stream.
/// [Universal Chess Interface]: https://www.chessprogramming.org/UCI
use core::panic;
use std::io::{BufRead, Write};

use crate::{
    chess::{core::Move, position::Position},
    search::SearchState,
};

/// The Engine manages all resources, keeps track of the time and handles
/// commands sent by UCI server.
pub struct Engine<'a, R: BufRead, W: Write> {
    position: Position,
    input: &'a mut R,
    output: &'a mut W,
}

impl<'a, R: BufRead, W: Write> Engine<'a, R, W> {
    #[must_use]
    pub fn new(input: &'a mut R, output: &'a mut W) -> Self {
        Self {
            position: Position::starting(),
            input,
            output,
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
    /// The minimal set of supported commands should be:
    ///     - uci
    ///     - isready
    ///     - go
    ///     - go wtime btime winc binc movetime
    ///     - quit
    ///     - ucinewgame
    ///     - setoption
    ///     - stop (?)
    ///
    /// NOTE: The assumption is that the UCI input stream is **correct**. It is
    /// tournament manager's responsibility to send uncorrupted input and make
    /// sure that the commands are in valid format. The engine won't spend too
    /// much time and effort on error recovery. If a command is not valid or
    /// unsupported yet, it will just be skipped.
    ///
    /// For example, if the UCI server sends a corrupted position or illegal
    /// moves to the engine, the behavior is undefined.
    // > The engine must always be able to process input from stdin, even while
    // > thinking.
    pub fn uci_loop(&mut self) -> anyhow::Result<()> {
        loop {
            let mut line = String::new();
            match self.input.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {},
                Err(e) => {
                    panic!("Error reading from input: {}", e);
                },
            }
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let mut stream = tokens.iter();
            match stream.next() {
                Some(&"uci") => self.handle_uci()?,
                Some(&"isready") => self.handle_isready()?,
                Some(&"setoption") => self.handle_setoption(line)?,
                Some(&"ucinewgame") => self.handle_ucinewgame()?,
                Some(&"position") => self.handle_position(&mut stream, &tokens)?,
                Some(&"go") => self.handle_go()?,
                Some(&"stop") => self.handle_stop()?,
                Some(&"quit") => break,
                Some(&command) => {
                    writeln!(self.output, "info string Unsupported command: {command}")?
                },
                None => {},
            }
        }
        Ok(())
    }

    fn handle_uci(&mut self) -> anyhow::Result<()> {
        writeln!(
            self.output,
            "id name {} {}",
            env!("CARGO_PKG_NAME"),
            crate::get_version()
        )?;
        writeln!(self.output, "id author {}", env!("CARGO_PKG_AUTHORS"))?;
        writeln!(self.output, "uciok")?;
        Ok(())
    }

    fn handle_isready(&mut self) -> anyhow::Result<()> {
        writeln!(self.output, "readyok")?;
        Ok(())
    }

    fn handle_setoption(&mut self, line: String) -> anyhow::Result<()> {
        writeln!(
            self.output,
            "info string `setoption` is no-op for now: received command {line}"
        )?;
        Ok(())
    }

    fn handle_ucinewgame(&mut self) -> anyhow::Result<()> {
        // TODO: Implement this method.
        Ok(())
    }

    fn handle_position(
        &mut self,
        stream: &mut std::slice::Iter<&str>,
        tokens: &[&str],
    ) -> anyhow::Result<()> {
        match stream.next() {
            Some(&"startpos") => self.position = Position::starting(),
            Some(&"fen") => {
                const FEN_SIZE: usize = 6;
                const COMMAND_START_SIZE: usize = 2;
                if tokens.len() < COMMAND_START_SIZE + FEN_SIZE {
                    writeln!(self.output, "info string FEN consists of 6 pieces, got {}", tokens.len() - 2)?;
                }
                todo!();
            },
            _ => writeln!(self.output, "info string Expected `position [fen <fenstring> | startpos] moves <move1> ... <move_i>`, got: {:?}", tokens)?,
        }
        if let Some(&"moves") = stream.next() {
            for next_move in stream {
                match Move::from_uci(next_move) {
                    Ok(next_move) => self.position.make_move(&next_move),
                    Err(e) => writeln!(self.output, "info string Unexpected UCI move: {e}")?,
                }
            }
        }
        Ok(())
    }

    fn handle_go(&mut self) -> anyhow::Result<()> {
        let mut state = SearchState::new();
        let MAX_DEPTH = 3;
        state.reset(&self.position);
        let search_result = crate::search::minimax::negamax(
            &mut state,
            MAX_DEPTH,
            &crate::evaluation::material::material_advantage,
        );
        writeln!(self.output, "bestmove {}", search_result.best_move.unwrap())?;
        Ok(())
    }

    fn handle_stop(&mut self) -> anyhow::Result<()> {
        // TODO: Implement this method.
        Ok(())
    }

    fn print_state(&mut self, info: &str) -> anyhow::Result<()> {
        todo!();
    }
}

// TODO: Add extensive test suite for the UCI protocol implementation.