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

use crate::chess::core::{Move, Player};
use crate::chess::position::Position;
use crate::engine::uci::Command;
use crate::search::{find_best_move, Depth};

pub mod openbench;
mod time_manager;
mod uci;

/// The Engine connects everything together and handles commands sent by UCI
/// server.
pub struct Engine<'a, R: BufRead, W: Write> {
    position: Position,
    debug: bool,
    // TODO: time_manager,
    // TODO: transposition_table
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
            debug: false,
            input,
            output,
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
                Command::SetOption { option, value } => todo!(),
                Command::SetPosition { fen, moves } => self.set_position(fen, moves)?,
                Command::NewGame => self.new_game()?,
                Command::Go {
                    max_depth,
                    wtime,
                    btime,
                    winc,
                    binc,
                    nodes,
                    mate,
                    movetime,
                    infinite,
                } => self.go(
                    max_depth, wtime, btime, winc, binc, nodes, mate, movetime, infinite,
                )?,
                Command::Stop => self.stop_search()?,
                Command::Quit => {
                    self.stop_search()?;
                    break;
                },
                Command::State => todo!(),
                Command::Unknown(command) => {
                    writeln!(self.output, "info string Unsupported command: {command}")?;
                },
            }
        }
        Ok(())
    }

    /// Responds to the `uci` handshake command by identifying the engine.
    fn handshake(&mut self) -> anyhow::Result<()> {
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
    fn sync(&mut self) -> anyhow::Result<()> {
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
        nodes: Option<usize>,
        mate: Option<Depth>,
        movetime: Option<Duration>,
        infinite: bool,
    ) -> anyhow::Result<()> {
        if mate.is_some() {
            todo!()
        }
        let (time, increment) = match self.position.us() {
            Player::White => (wtime, winc),
            Player::Black => (btime, binc),
        };
        let next_move = find_best_move(&self.position, max_depth, time, self.output);
        writeln!(self.output, "bestmove {next_move}")?;
        Ok(())
    }

    /// Stops the search immediately.
    ///
    /// NOTE: This is a no-op for now.
    fn stop_search(&mut self) -> anyhow::Result<()> {
        // TODO: Implement this method.
        Ok(())
    }
}

// TODO: Add extensive test suite for the UCI protocol implementation.
