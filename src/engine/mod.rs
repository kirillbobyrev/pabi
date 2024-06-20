//! The engine puts all pieces together and manages resources effectively. It
//! implements the [Universal Chess Interface] (UCI) for communication with the
//! client (e.g. tournament runner with other engines or GUI/Lichess endpoint).
//!
//! [`Engine::uci_loop`] is the "main loop" of the engine which communicates
//! with the environment and executes commands from the input stream.
/// [Universal Chess Interface]: https://www.chessprogramming.org/UCI
use core::panic;
use std::io::{BufRead, Write};
use std::time::Duration;

use itertools::Itertools;

use crate::chess::core::{Move, Player};
use crate::chess::position::Position;
use crate::engine::uci::Command;
use crate::search::go;

mod uci;

/// The Engine connects everything together handles commands sent by UCI server,
/// including I/O.
pub struct Engine<'a, R: BufRead, W: Write> {
    position: Position,
    input: &'a mut R,
    output: &'a mut W,
}

impl<'a, R: BufRead, W: Write> Engine<'a, R, W> {
    /// Creates a new instance of the engine with starting position and provided
    /// I/O.
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
    /// NOTE: The assumption is that the UCI input stream is **correct**. It is
    /// tournament manager's responsibility to send uncorrupted input and make
    /// sure that the commands are in valid format. The engine won't spend too
    /// much time and effort on error recovery. If a command is not valid or
    /// unsupported yet, it will just be skipped.
    ///
    /// For example, if the UCI server sends a corrupted position or illegal
    /// moves to the engine, the behavior is undefined.
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
            match Command::parse(&line) {
                Command::Uci => self.handle_uci()?,
                Command::Debug { on } => todo!(),
                Command::IsReady => self.handle_isready()?,
                Command::SetOption { option, value } => todo!(),
                Command::SetPosition { fen, moves } => todo!(),
                Command::NewGame => todo!(),
                Command::Go {
                    depth,
                    wtime,
                    btime,
                    winc,
                    binc,
                    nodes,
                    mate,
                    movetime,
                    infinite,
                } => todo!(),
                Command::Stop => self.handle_stop()?,
                Command::Quit => {
                    self.handle_stop()?;
                    break;
                },
                Command::Unknown(command) => {
                    writeln!(self.output, "info string Unsupported command: {command}")?
                },
            }
        }
        Ok(())
    }

    /// Responds to the `uci` handshake command by identifying the engine.
    fn handle_uci(&mut self) -> anyhow::Result<()> {
        writeln!(
            self.output,
            "id name {} {}",
            env!("CARGO_PKG_NAME"),
            crate::engine_version()
        )?;
        writeln!(self.output, "id author {}", env!("CARGO_PKG_AUTHORS"))?;
        writeln!(self.output, "uciok")?;
        Ok(())
    }

    /// Syncs with the UCI server by responding with `readyok`.
    fn handle_isready(&mut self) -> anyhow::Result<()> {
        writeln!(self.output, "readyok")?;
        Ok(())
    }

    /// Sets the engine options. This is a no-op for now. In the future this
    /// should at least support setting the transposition table size and search
    /// thread count.
    fn handle_setoption(&mut self, line: String) -> anyhow::Result<()> {
        writeln!(
            self.output,
            "info string `setoption` is no-op for now: received command {line}"
        )?;
        Ok(())
    }

    fn handle_ucinewgame(&mut self) -> anyhow::Result<()> {
        // TODO: Implement this method - reset search state.
        Ok(())
    }

    /// Changes the position of the board to the one specified in the command.
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
                    writeln!(
                        self.output,
                        "info string FEN consists of 6 pieces, got {}",
                        tokens.len() - 2
                    )?;
                }
                self.position = Position::from_fen(&stream.take(FEN_SIZE).join(" "))?;
            },
            _ => writeln!(
                self.output,
                "info string Expected `position [fen <fenstring> | startpos]
                moves <move1> ... <move_i>`, got: {:?}",
                tokens.join(" ")
            )?,
        }
        if stream.next() == Some(&"moves") {
            for next_move in stream {
                match Move::from_uci(next_move) {
                    Ok(next_move) => self.position.make_move(&next_move),
                    Err(e) => writeln!(self.output, "info string Unexpected UCI move: {e}")?,
                }
            }
        }
        Ok(())
    }

    // TODO: Handle: wtime btime winc binc
    fn handle_go(&mut self, command: Command::Go) -> anyhow::Result<()> {
        Ok(())
    }

    /// Stops the search immediately.
    ///
    /// NOTE: This is a no-op for now.
    fn handle_stop(&mut self) -> anyhow::Result<()> {
        // TODO: Implement this method.
        Ok(())
    }
}

// TODO: Add extensive test suite for the UCI protocol implementation.
