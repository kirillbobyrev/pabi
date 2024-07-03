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

use anyhow::bail;

use crate::chess::core::{Color, Move};
use crate::chess::position::Position;
use crate::engine::uci::Command;
use crate::search::Depth;

mod time_manager;
mod uci;

/// The Engine connects everything together and handles commands sent by UCI
/// server. It is created when the program is started and implement the "main
/// loop" via [`Engine::uci_loop`].
pub struct Engine<'a, R: BufRead, W: Write> {
    /// Next search will start from this position.
    position: Position,
    debug: bool,
    // TODO: time_manager,
    // TODO: transposition_table
    /// UCI commands will be read from this stream.
    input: &'a mut R,
    /// Responses to UCI commands will be written to this stream.
    out: &'a mut W,
}

impl<'a, R: BufRead, W: Write> Engine<'a, R, W> {
    /// Creates a new instance of the engine with the starting position as the
    /// search root.
    #[must_use]
    pub fn new(input: &'a mut R, out: &'a mut W) -> Self {
        Self {
            position: Position::starting(),
            debug: false,
            input,
            out,
        }
    }

    /// Continuously reads the input stream and executes sent UCI commands until
    /// "quit" is sent.
    ///
    /// The implementation here does not aim to be complete and exhaustive,
    /// because the main goal is to make the engine work in relatively simple
    /// setups, making it work with all UCI-compatible GUIs and corrupted input
    /// is not a priority. For supported commands and their options see
    /// [`Command`].
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
                Command::Uci => self.handshake()?,
                Command::Debug { on } => self.debug = on,
                Command::IsReady => self.sync()?,
                Command::SetOption { option, value } => match option {
                    uci::EngineOption::Hash => match value {
                        uci::OptionValue::Integer(_) => todo!(),
                        uci::OptionValue::String(value) => writeln!(
                            self.out,
                            "info string Invalid value for Hash option: {value}"
                        )?,
                    },
                    uci::EngineOption::Threads => todo!(),
                    uci::EngineOption::SyzygyTablebase => todo!(),
                },
                Command::SetPosition { fen, moves } => self.set_position(fen, moves)?,
                Command::NewGame => self.new_game()?,
                Command::Go {
                    max_depth,
                    wtime,
                    btime,
                    winc,
                    binc,
                    movetime,
                    infinite,
                } => self.go(max_depth, wtime, btime, winc, binc, movetime, infinite)?,
                Command::Stop => self.stop_search()?,
                Command::Quit => {
                    self.stop_search()?;
                    break;
                },
                Command::State => todo!(),
                Command::Unknown(command) => {
                    writeln!(self.out, "info string Unsupported command: {command}")?;
                },
            }
        }
        Ok(())
    }

    /// Responds to the `uci` handshake command by identifying the engine.
    fn handshake(&mut self) -> anyhow::Result<()> {
        writeln!(
            self.out,
            "id name {} {}",
            env!("CARGO_PKG_NAME"),
            crate::engine_version()
        )?;
        writeln!(self.out, "id author {}", env!("CARGO_PKG_AUTHORS"))?;
        writeln!(self.out, "uciok")?;
        Ok(())
    }

    /// Syncs with the UCI server by responding with `readyok`.
    fn sync(&mut self) -> anyhow::Result<()> {
        writeln!(self.out, "readyok")?;
        Ok(())
    }

    fn new_game(&mut self) -> anyhow::Result<()> {
        // TODO: Reset search state.
        // TODO: Clear transposition table.
        // TODO: Reset time manager.
        Ok(())
    }

    /// Changes the position of the board to the one specified in the command.
    fn set_position(&mut self, fen: Option<String>, moves: Vec<String>) -> anyhow::Result<()> {
        match fen {
            Some(fen) => self.position = Position::from_fen(&fen)?,
            None => self.position = Position::starting(),
        };
        for next_move in moves {
            match Move::from_uci(&next_move) {
                Ok(next_move) => self.position.make_move(&next_move),
                Err(_) => unreachable!(),
            }
        }
        Ok(())
    }

    fn go(
        &mut self,
        max_depth: Option<Depth>,
        wtime: Option<Duration>,
        btime: Option<Duration>,
        winc: Option<Duration>,
        binc: Option<Duration>,
        movetime: Option<Duration>,
        infinite: bool,
    ) -> anyhow::Result<()> {
        if infinite && (wtime.is_some() || btime.is_some() || movetime.is_some()) {
            bail!("Infinite is set, but wtime, btime or movetime is also set");
        }
        if movetime.is_some() && (wtime.is_some() || btime.is_some()) {
            bail!("Movetime is set, but wtime or btime is also set");
        }
        let (time, increment) = match self.position.us() {
            Color::White => (wtime, winc),
            Color::Black => (btime, binc),
        };
        todo!();
    }

    /// Stops the search immediately.
    ///
    /// NOTE: This is a no-op for now.
    fn stop_search(&mut self) -> anyhow::Result<()> {
        // TODO: Implement this method.
        Ok(())
    }
}

/// Runs search on a small set of positions to provide an estimate of engine's
/// performance.
///
/// Implementing `bench` CLI command is a [requirement for OpenBench].
///
/// NOTE: This function **has to run less than 60 seconds**.
///
/// See <https://github.com/AndyGrant/OpenBench/blob/master/Client/bench.py> for
/// more details.
///
/// [requirement for OpenBench]: https://github.com/AndyGrant/OpenBench/wiki/Requirements-For-Public-Engines#basic-requirements
pub fn openbench() {
    todo!()
}

// TODO: Add extensive test suite for the UCI protocol implementation.
