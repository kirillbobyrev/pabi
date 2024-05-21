//! Implementation of [Universal Chess Interface] (UCI) protocol for
//! communication between the engine and the UCI client (e.g. GUI, tournament
//! runner).
//!
//! The implementation here does not aim to be complete and exhaustive, because
//! the main goal is to make the engine work in relatively simple setups, making
//! it work with all UCI-compatible GUIs and corrupted input is not a priority.
//!
//! [Universal Chess Interface]: https://www.chessprogramming.org/UCI

use core::panic;
use std::io::{BufRead, Write};

use crate::VERSION;

/// Reads UCI commands from the input stream and executes them accordingly while
/// writing the responses to the output stream.
// TODO: Document the expected behavior.
// > The engine must always be able to process input from stdin, even while
// > thinking.
pub fn run_loop(input: &mut impl BufRead, output: &mut impl Write) {
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
            // uci
            //
            // Tell engine to use the uci (universal chess interface), this will
            // be sent once as a first command after program boot to tell the
            // engine to switch to uci mode.
            //
            // After receiving the uci command the engine must identify itself
            // with the "id" command and send the "option" commands to tell the
            // GUI which engine settings the engine supports if any.
            // After that the engine should send "uciok" to acknowledge the uci
            // mode. If no "uciok" is sent within a certain time period, the
            // engine task will be killed by the GUI.
            Some(&"uci") => {
                writeln!(output, "id name {} {}", env!("CARGO_PKG_NAME"), VERSION).unwrap();
                writeln!(output, "id author {}", env!("CARGO_PKG_AUTHORS")).unwrap();
                writeln!(output, "uciok").unwrap();
                // Potentially send "option"? Should the engine have any
                // configurable options at all?
            },
            // debug [ on | off ]
            //
            // 	Switch the debug mode of the engine on and off.
            //
            //  In debug mode the engine should send additional infos to the
            //  GUI, e.g. with the "info string" command, to help debugging,
            //  e.g. the commands that the engine has received etc. This mode
            //  should be switched off by default and this command can be sent
            //  any time, also when the engine is thinking.
            Some(&"debug") => {
                todo!();
            },
            // isready
            //
            //  This is used to synchronize the engine with the GUI. When the
            //  GUI has sent a command or multiple commands that can take some
            //  time to complete, this command can be used to wait for the
            //  engine to be ready again or to ping the engine to find out if it
            //  is still alive.
            //
            //  E.g. this should be sent after setting the path to the
            //  tablebases as this can take some time.  This command is also
            //  required once before the engine is asked to do any search to
            //  wait for the engine to finish initializing.
            //
            //  This command must always be answered with "readyok" and can be
            //  sent also when the engine is calculating in which case the
            //  engine should also immediately answer with "readyok" without
            //  stopping the search.
            Some(&"isready") => {
                println!("readyok");
            },
            // setoption name <id> [value <x>]
            //
            //  This is sent to the engine when the user wants to change the
            //  internal parameters of the engine. For the "button" type no
            //  value is needed.
            //
            //  One string will be sent for each parameter and this will only be
            //  sent when the engine is waiting.
            //
            //  The name and value of the option in <id> should not be case
            //  sensitive and can include spaces.
            //
            //  The substrings "value" and "name" should be avoided in <id> and
            //  <x> to allow unambiguous parsing,
            //
            // 	for example do not use <name> = "draw value".
            //
            // 	Here are some strings for the example below:
            // 	    - "setoption name Nullmove value true\n"
            //      - "setoption name Selectivity value 3\n"
            // 	    - "setoption name Style value Risky\n"
            // 	    - "setoption name Clear Hash\n"
            Some(&"setoption") => {
                todo!();
            },
            // ucinewgame
            //
            // This is sent to the engine when the next search (started with
            // "position" and "go") will be from a different game. This can
            // be a new game the engine should play or a new game it should
            // analyze but also the next position from a testsuite with
            // positions only.
            //
            // If the GUI hasn't sent a "ucinewgame" before the first "position"
            // command, the engine shouldn't expect any further ucinewgame
            // commands as the GUI is probably not supporting the ucinewgame
            // command. So the engine should not rely on this command even
            // though all new GUIs should support it. As the engine's reaction
            // to "ucinewgame" can take some time the GUI should always send
            // "isready" after "ucinewgame" to wait for the engine to finish its
            // operation.
            Some(&"ucinewgame") => {
                // This is practically no-op for now, maybe always. Not sure
                // what should change when the new game is started.
            },
            // position [fen <fenstring> | startpos ]  moves <move1> .... <move_i>
            //
            // Set up the position described in `fenstring` on the internal board and
            // play the moves on the internal chess board.
            //
            // If the game was played  from the start position the string
            // "startpos" will be sent Note: no "new" command is needed.
            //
            // However, if this position is from a different game than the last
            // position sent to the engine, the GUI should have sent a
            // "ucinewgame" inbetween.
            Some(&"position") => {
                // Handle position setup
                if tokens[1] == "startpos" {
                    // Handle starting position
                    todo!();
                } else {
                    // Handle FEN position
                    todo!();
                }
                if tokens.len() > 2 && tokens[2] == "moves" {
                    // Handle moves
                    for token in tokens.iter().skip(3) {
                        // Process the move
                        todo!();
                    }
                }
            },
            // stop
            //
            // Stop calculating as soon as possible,
            //
            // Don't forget the "bestmove" and possibly the "ponder" token when
            // finishing the search
            Some(&"stop") => {
                todo!();
            },
            // ponderhit
            //
            // The user has played the expected move. This will be sent if the
            // engine was told to ponder on the same move The user has played.
            // The engine should continue searching but switch from pondering to
            // normal search.
            Some(&"ponderhit") => {
                todo!();
            },
            Some(&"go") => {
                todo!();
            },
            Some(&"quit") => {
                break;
            },
            _ => {
                writeln!(output, "Unknown command: {}", line).unwrap();
            },
        }
    }
}
