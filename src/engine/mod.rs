//! The engine puts all pieces together and manages resources effectively. It
//! implements the [Universal Chess Interface] (UCI) for communication with the
//! client (e.g. tournament runner with other engines or GUI/Lichess endpoint).
//!
//! [`Engine::uci_loop`] is the "main loop" of the engine which communicates
//! with the environment and executes commands from the input stream.
//!
//! [Universal Chess Interface]: https://www.chessprogramming.org/UCI
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crate::chess::core::Move;
use crate::chess::game::{self, Game};
use crate::chess::position::Position;
use crate::engine::uci::Command;
use crate::environment::{Environment, Player};
use crate::search::mcts;

mod time_manager;
mod uci;

/// A search running in a background thread. The search can be stopped through
/// the flag; joining the handle guarantees that `bestmove` has been printed.
struct SearchThread {
    stop: Arc<AtomicBool>,
    handle: thread::JoinHandle<()>,
    /// Whether the search would run forever unless stopped externally.
    infinite: bool,
}

/// The Engine connects everything together and handles commands sent by UCI
/// server. It is created when the program is started and implement the "main
/// loop" via [`Engine::uci_loop`].
pub struct Engine<R: BufRead, W: Write + Send + 'static> {
    /// The game the next search will start from.
    game: Game,
    debug: bool,
    /// Requested transposition table size in MB. Stored for the time when the
    /// search gets a transposition table.
    hash_size_mb: usize,
    /// Number of search threads. Only single-threaded search is implemented so
    /// far.
    threads: usize,
    /// UCI commands will be read from this stream.
    input: R,
    /// Responses to UCI commands will be written to this stream. Shared with
    /// the search thread, which reports progress and the best move.
    out: Arc<Mutex<W>>,
    search: Option<SearchThread>,
}

impl<R: BufRead, W: Write + Send + 'static> Engine<R, W> {
    /// Creates a new instance of the engine with the starting position as the
    /// search root.
    #[must_use]
    pub fn new(input: R, out: W) -> Self {
        Self {
            game: Game::new(Position::starting()),
            debug: false,
            hash_size_mb: 16,
            threads: 1,
            input,
            out: Arc::new(Mutex::new(out)),
            search: None,
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
    pub fn uci_loop(&mut self) -> anyhow::Result<()> {
        loop {
            let mut line = String::new();
            match self.input.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => {
                    self.stop_search();
                    return Err(anyhow::Error::from(e).context("error reading UCI input"));
                }
            }
            match Command::parse(&line) {
                Command::Uci => self.handshake()?,
                Command::Debug { on } => self.debug = on,
                Command::IsReady => self.sync()?,
                Command::SetOption { option, value } => self.set_option(&option, value)?,
                Command::SetPosition { fen, moves } => {
                    if let Err(e) = self.set_position(fen, &moves) {
                        self.send(&format!("info string invalid position: {e:#}"))?;
                    }
                }
                Command::NewGame => self.new_game(),
                Command::Go(go) => {
                    if go.depth.is_some() {
                        self.send(
                            "info string depth limit is not supported by the MCTS search and is \
                             ignored",
                        )?;
                    }
                    self.go(&go);
                }
                Command::Stop => self.stop_search(),
                Command::Quit => {
                    // Let a search with finite limits run to completion so
                    // that scripted input (where `quit` arrives right after
                    // `go`) still produces a meaningful `bestmove`. GUIs that
                    // want to terminate the search immediately send `stop`
                    // first. Infinite searches are always interrupted.
                    self.finish_search(false);
                    break;
                }
                Command::State => self.state()?,
                Command::Unknown(command) => {
                    let command = command.trim();
                    if !command.is_empty() {
                        self.send(&format!("info string Unsupported command: {command}"))?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Writes a line to the output stream and flushes it.
    fn send(&self, line: &str) -> anyhow::Result<()> {
        let mut out = self.out.lock().expect("output lock poisoned");
        writeln!(out, "{line}")?;
        out.flush()?;
        Ok(())
    }

    /// Responds to the `uci` handshake command by identifying the engine.
    fn handshake(&mut self) -> anyhow::Result<()> {
        self.send(&format!(
            "id name {} {}",
            env!("CARGO_PKG_NAME"),
            crate::engine_version()
        ))?;
        self.send(&format!("id author {}", env!("CARGO_PKG_AUTHORS")))?;
        self.send("option name Hash type spin default 16 min 1 max 1048576")?;
        self.send("option name Threads type spin default 1 min 1 max 1")?;
        self.send("option name SyzygyTablebase type string default <empty>")?;
        self.send("uciok")?;
        Ok(())
    }

    /// Syncs with the UCI server by responding with `readyok`.
    fn sync(&self) -> anyhow::Result<()> {
        self.send("readyok")
    }

    fn set_option(
        &mut self,
        option: &uci::EngineOption,
        value: uci::OptionValue,
    ) -> anyhow::Result<()> {
        match (option, value) {
            (uci::EngineOption::Hash, uci::OptionValue::Integer(megabytes)) => {
                self.hash_size_mb = megabytes.max(1);
            }
            (uci::EngineOption::Threads, uci::OptionValue::Integer(threads)) => {
                if threads > 1 {
                    self.send("info string only 1 search thread is supported for now")?;
                }
                self.threads = 1;
            }
            (uci::EngineOption::SyzygyTablebase, uci::OptionValue::String(path)) => {
                self.set_tablebase(&path)?;
            }
            (option, value) => {
                self.send(&format!(
                    "info string invalid value for option {option:?}: {value:?}"
                ))?;
            }
        }
        Ok(())
    }

    /// Loads (or, for an empty path, clears) the Syzygy tablebase and shares it
    /// with the current game.
    fn set_tablebase(&mut self, path: &str) -> anyhow::Result<()> {
        let path = path.trim();
        if path.is_empty() || path == "<empty>" {
            self.game.set_tablebase(None);
            return Ok(());
        }
        match game::load_tablebase(PathBuf::from(path).as_ref()) {
            Ok(tablebase) => {
                let tablebase = Arc::new(tablebase);
                self.send(&format!(
                    "info string loaded Syzygy tablebases from {path} (up to {} pieces)",
                    tablebase.max_pieces()
                ))?;
                self.game.set_tablebase(Some(tablebase));
            }
            Err(e) => {
                self.send(&format!(
                    "info string failed to load SyzygyTablebase: {e:#}"
                ))?;
            }
        }
        Ok(())
    }

    fn new_game(&mut self) {
        self.stop_search();
        self.game = self.fresh_game(Position::starting());
    }

    /// Changes the position of the board to the one specified in the command.
    /// Each move is checked to be legal in the position it is applied to.
    fn set_position(&mut self, fen: Option<String>, moves: &[String]) -> anyhow::Result<()> {
        self.stop_search();
        let root = match fen {
            Some(fen) => Position::from_fen(&fen)?,
            None => Position::starting(),
        };
        let mut game = self.fresh_game(root);
        for next_move in moves {
            let next_move = Move::from_uci(next_move)
                .map_err(|e| e.context(format!("invalid move: {next_move}")))?;
            if !game.actions().contains(&next_move) {
                anyhow::bail!("illegal move {next_move} in position {}", game.position());
            }
            game.apply(&next_move);
        }
        self.game = game;
        Ok(())
    }

    /// Creates a game from `root` that keeps the currently loaded tablebase.
    fn fresh_game(&self, root: Position) -> Game {
        let mut game = Game::new(root);
        game.set_tablebase(self.game.tablebase());
        game
    }

    /// Translates a `go` command into concrete [`mcts::Limits`]. An explicit
    /// move time or `infinite` flag is used directly; otherwise a budget is
    /// derived from the clock of the side to move, falling back to an infinite
    /// search when no limit is given at all.
    fn limits(&self, go: &uci::Go) -> mcts::Limits {
        let mut limits = mcts::Limits {
            move_time: go.movetime,
            nodes: go.nodes,
            infinite: go.infinite,
        };
        if limits.move_time.is_none() && !limits.infinite {
            let (time, increment) = match self.game.position().us() {
                Player::White => (go.wtime, go.winc),
                Player::Black => (go.btime, go.binc),
            };
            limits.move_time =
                time.map(|time| time_manager::time_budget(time, increment.unwrap_or_default()));
        }
        if limits.move_time.is_none() && limits.nodes.is_none() {
            limits.infinite = true;
        }
        limits
    }

    /// Starts the search in a background thread. The thread reports progress
    /// through `info` lines and always finishes by printing `bestmove`.
    fn go(&mut self, go: &uci::Go) {
        self.stop_search();

        let limits = self.limits(go);
        let infinite = limits.infinite;
        let stop = Arc::new(AtomicBool::new(false));
        let position = self.game.position().clone();
        let history = self.game.history().to_vec();
        let tablebase = self.game.tablebase();
        let out = Arc::clone(&self.out);
        let search_stop = Arc::clone(&stop);

        let handle = thread::spawn(move || {
            let result = mcts::search(
                &position,
                &history,
                tablebase.as_deref(),
                &limits,
                &search_stop,
                |info| {
                    let mut out = out.lock().expect("output lock poisoned");
                    let _ = writeln!(out, "{info}");
                    let _ = out.flush();
                },
            );
            let best_move = result
                .best_move
                .map_or_else(|| "(none)".to_string(), |best_move| best_move.to_string());
            let mut out = out.lock().expect("output lock poisoned");
            let _ = writeln!(out, "bestmove {best_move}");
            let _ = out.flush();
        });

        self.search = Some(SearchThread {
            stop,
            handle,
            infinite,
        });
    }

    /// Stops the running search (if any) and waits for it to print `bestmove`.
    fn stop_search(&mut self) {
        self.finish_search(true);
    }

    /// Waits for the running search (if any) to print `bestmove`. When `force`
    /// is set, the search is interrupted; otherwise it runs until its own
    /// limits expire (infinite searches are interrupted regardless, as they
    /// have no limits to expire).
    fn finish_search(&mut self, force: bool) {
        if let Some(search) = self.search.take() {
            if force || search.infinite {
                search.stop.store(true, Ordering::Relaxed);
            }
            let _ = search.handle.join();
        }
    }

    /// Responds to the non-standard `state` command with debugging information
    /// about the engine state.
    fn state(&self) -> anyhow::Result<()> {
        let position = self.game.position();
        self.send(&format!("info string position fen {position}"))?;
        self.send(&format!(
            "info string static eval cp {}",
            crate::evaluation::evaluate(position)
        ))?;
        self.send(&format!(
            "info string legal moves {}",
            self.game.actions().len()
        ))?;
        self.send(&format!(
            "info string game plies {}",
            self.game.history().len() - 1
        ))?;
        self.send(&format!("info string debug {}", self.debug))?;
        self.send(&format!("info string option Hash {}", self.hash_size_mb))?;
        self.send(&format!("info string option Threads {}", self.threads))?;
        self.send(&format!(
            "info string option SyzygyTablebase {}",
            self.game.tablebase().map_or_else(
                || "<empty>".to_string(),
                |tablebase| format!("loaded, up to {} pieces", tablebase.max_pieces())
            )
        ))?;
        self.send(&format!(
            "info string search running {}",
            self.search
                .as_ref()
                .is_some_and(|s| !s.handle.is_finished())
        ))?;
        Ok(())
    }
}

impl<R: BufRead, W: Write + Send + 'static> Drop for Engine<R, W> {
    fn drop(&mut self) {
        self.stop_search();
    }
}

/// Runs search on a small set of positions to provide an estimate of engine's
/// performance.
///
/// Implementing `bench` CLI command is a [requirement for OpenBench].
///
/// NOTE: This function **has to run less than 60 seconds**. Ideally, it should
/// be just under 5 seconds.
///
/// See <https://github.com/AndyGrant/OpenBench/blob/master/Client/bench.py> for
/// more details.
///
/// [requirement for OpenBench]: https://github.com/AndyGrant/OpenBench/wiki/Requirements-For-Public-Engines#basic-requirements
pub fn openbench() {
    const NODES_PER_POSITION: u64 = 5000;
    const POSITIONS: &[&str] = &[
        // Starting position.
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        // Italian game.
        "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
        // Kiwipete: a tactically rich middlegame position.
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        // Rook endgame.
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        // Promotion-heavy position.
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        // Closed middlegame.
        "r4rk1/1pp1qppp/p1np1n2/2b1p1b1/2B1P1B1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        // Queen vs rook endgame.
        "4k3/8/8/5r2/4KQ2/8/8/8 w - - 0 1",
        // King and pawn endgame.
        "4k3/8/8/8/8/8/4P3/4K3 w - - 0 1",
    ];

    let start = Instant::now();
    let mut total_nodes = 0;
    for fen in POSITIONS {
        let position = Position::from_fen(fen).expect("bench positions are valid");
        let limits = mcts::Limits {
            nodes: Some(NODES_PER_POSITION),
            ..mcts::Limits::default()
        };
        let stop = AtomicBool::new(false);
        let result = mcts::search(&position, &[position.hash()], None, &limits, &stop, |_| {});
        total_nodes += result.nodes;
        println!(
            "info string bench bestmove {} for {fen}",
            result
                .best_move
                .map_or_else(|| "(none)".to_string(), |m| m.to_string())
        );
    }

    let elapsed = start.elapsed().as_secs_f64();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        reason = "nps can not realistically overflow or be negative"
    )]
    let nps = if elapsed > 0.0 {
        (total_nodes as f64 / elapsed) as u64
    } else {
        0
    };
    println!("{total_nodes} nodes {nps} nps");
}
