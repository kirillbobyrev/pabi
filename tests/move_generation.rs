use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use itertools::Itertools;
use pabi::chess::core::Move;
use pabi::chess::position::Position;
use pretty_assertions::assert_eq;
use shakmaty::{CastlingMode, Chess, Position as ShakmatyPosition};

fn setup(input: &str) -> Position {
    Position::try_from(input).expect("parsing legal position: {input}")
}

fn get_moves(position: &Position) -> Vec<String> {
    position
        .generate_moves()
        .iter()
        .map(Move::to_string)
        .sorted()
        .collect::<Vec<_>>()
}

fn sorted_moves(moves: &[&str]) -> Vec<String> {
    moves
        .iter()
        .map(|m| (*m).to_string())
        .sorted()
        .collect::<Vec<_>>()
}

#[test]
fn starting_moves() {
    assert_eq!(
        get_moves(&Position::starting()),
        sorted_moves(&[
            "a2a3", "a2a4", "b1a3", "b1c3", "b2b3", "b2b4", "c2c3", "c2c4", "d2d3", "d2d4", "e2e3",
            "e2e4", "f2f3", "f2f4", "g1f3", "g1h3", "g2g3", "g2g4", "h2h3", "h2h4"
        ])
    );
}

#[test]
fn basic_moves() {
    assert_eq!(
        get_moves(&setup("2n4k/1PP5/6K1/3Pp1Q1/3N4/3P4/P3R3/8 w - e6 0 1")),
        sorted_moves(&[
            "a2a3", "a2a4", "d5d6", "d5e6", "b7b8q", "b7b8r", "b7b8b", "b7b8n", "b7c8q", "b7c8r",
            "b7c8b", "b7c8n", "e2e1", "e2e3", "e2e4", "e2e5", "e2b2", "e2c2", "e2d2", "e2f2",
            "e2g2", "e2h2", "d4b3", "d4c2", "d4f3", "d4b5", "d4c6", "d4e6", "d4f5", "g5c1", "g5d2",
            "g5e3", "g5f4", "g5g4", "g5g3", "g5g2", "g5g1", "g5h4", "g5e5", "g5f5", "g5h5", "g5h6",
            "g5f6", "g5e7", "g5d8", "g6f5", "g6h5", "g6f6", "g6h6", "g6f7",
        ])
    );
}

#[test]
fn double_check_evasions() {
    assert_eq!(
        get_moves(&setup("3kn3/R2p1N2/8/8/7B/6K1/3R4/8 b - - 0 1")),
        sorted_moves(&["d8c8"])
    );
    assert_eq!(
        get_moves(&setup("8/5Nk1/7p/4Bp2/3q4/8/8/5KR1 b - - 0 1")),
        sorted_moves(&["g7f8", "g7f7", "g7h7"])
    );
    assert_eq!(
        get_moves(&setup("8/5Pk1/7p/4Bp2/3q4/8/8/5KR1 b - - 0 1")),
        sorted_moves(&["g7f8", "g7f7", "g7h7"])
    );
}

#[test]
fn check_evasions() {
    assert_eq!(
        get_moves(&setup("3kn3/R2p4/8/6B1/8/6K1/3R4/8 b - - 0 1")),
        sorted_moves(&["e8f6", "d8c8"])
    );
    assert_eq!(
        get_moves(&setup("2R5/8/6k1/8/8/8/PPn5/KR6 w - - 0 1")),
        sorted_moves(&["c8c2"])
    );
}

#[test]
fn pins() {
    // The pawn is pinned but can capture en passant.
    assert_eq!(
        get_moves(&setup("6qk/8/8/3Pp3/8/8/K7/8 w - e6 0 1")),
        sorted_moves(&["a2a1", "a2a3", "a2b1", "a2b2", "a2b3", "d5e6"])
    );
    // The pawn is pinned but there is no en passant: it can't move.
    assert_eq!(
        get_moves(&setup("6qk/8/8/3Pp3/8/8/K7/8 w - - 0 1")),
        sorted_moves(&["a2a1", "a2a3", "a2b1", "a2b2", "a2b3"])
    );
    // The pawn is pinned and can't move.
    assert_eq!(
        get_moves(&setup("k7/1p6/8/8/8/8/8/4K2B b - - 0 1")),
        sorted_moves(&["a8a7", "a8b8"])
    );
}

// Artifacts from the fuzzer.
#[test]
fn moves_in_other_positions() {
    assert_eq!(
        get_moves(&setup(
            "2r3r1/3p3k/1p3pp1/1B5P/5P2/2P1pqP1/PP4KP/3R4 w - - 0 34"
        )),
        sorted_moves(&["g2g1", "g2f3", "g2h3"])
    );
    assert_eq!(
        get_moves(&setup(
            "2r3r1/3p3k/1p3pp1/1B5P/5p2/2P1p1P1/PP4KP/3R4 w - - 0 34"
        )),
        sorted_moves(&[
            "a2a3", "a2a4", "b2b3", "b2b4", "c3c4", "b5a4", "b5a6", "b5c6", "b5d7", "b5c4", "b5d3",
            "b5e2", "b5f1", "g3g4", "h2h3", "h2h4", "h5h6", "h5g6", "g2f3", "g2f1", "g2g1", "g2h3",
            "g2h1", "d1a1", "d1b1", "d1c1", "d1e1", "d1f1", "d1g1", "d1h1", "d1d2", "d1d3", "d1d4",
            "d1d5", "d1d6", "d1d7", "g3f4",
        ])
    );
    assert_eq!(
        get_moves(&setup(
            "2r3r1/3p3k/1p3pp1/1B5p/5P2/2P2pP1/PP4KP/3R4 w - - 0 34"
        )),
        sorted_moves(&["g2f1", "g2f2", "g2f3", "g2g1", "g2h1", "g2h3"])
    );
    assert_eq!(
        get_moves(&setup(
            "2r3r1/P3k3/pp3p2/1B5p/5P2/2P3pP/PP4KP/3R4 w - - 0 1"
        )),
        sorted_moves(&[
            "a2a3", "a2a4", "a7a8b", "a7a8n", "a7a8q", "a7a8r", "b2b3", "b2b4", "b5a4", "b5a6",
            "b5c4", "b5c6", "b5d3", "b5d7", "b5e2", "b5e8", "b5f1", "c3c4", "d1a1", "d1b1", "d1c1",
            "d1d2", "d1d3", "d1d4", "d1d5", "d1d6", "d1d7", "d1d8", "d1e1", "d1f1", "d1g1", "d1h1",
            "f4f5", "g2f1", "g2f3", "g2g1", "g2h1", "h2g3", "h3h4",
        ])
    );
    assert_eq!(
        get_moves(&setup(
            "2r3r1/p3k3/pp3p2/1B5p/5P2/2pqp1P1/PPK4P/3R4 w - - 0 34"
        )),
        sorted_moves(&["b5d3", "c2b3", "c2c1", "c2d3", "d1d3"])
    );
    assert_eq!(
        get_moves(&setup(
            "2r3r1/p3k3/pp3p2/1B5p/5P2/2P1p1P1/PP4Kr/3R4 w - - 0 1"
        )),
        sorted_moves(&["g2f1", "g2f3", "g2g1", "g2h2"])
    );
    assert_eq!(
        get_moves(&setup("r3k3/r7/8/5pP1/5QKN/8/8/6RR w - f6 0 1")),
        sorted_moves(&["f4f5", "h4f5", "g4f5", "g4f3", "g4g3", "g4h3", "g5f6", "g4h5"])
    );
    assert_eq!(
        get_moves(&setup("4k1r1/8/8/4PpP1/6K1/8/8/8 w - f6 0 1")),
        sorted_moves(&["g4f4", "g4f3", "g4f5", "g4g3", "g4h3", "g4h4", "g4h5", "e5f6"])
    );
}

#[test]
fn castle() {
    // Can castle both sides.
    assert_eq!(
        get_moves(&setup("r3k2r/8/8/8/8/8/6N1/4K3 b kq - 0 1")),
        sorted_moves(&[
            "a8a7", "a8a6", "a8a5", "a8a4", "a8a3", "a8a2", "a8a1", "a8b8", "a8c8", "a8d8", "h8f8",
            "h8g8", "h8h7", "h8h6", "h8h5", "h8h4", "h8h3", "h8h2", "h8h1", "e8e7", "e8d8", "e8d7",
            "e8f8", "e8f7", "e8c8", "e8g8"
        ])
    );
    // Castling short blocked by a check.
    assert_eq!(
        get_moves(&setup("r3k2r/8/8/8/8/8/6R1/4K3 b kq - 0 1")),
        sorted_moves(&[
            "a8a7", "a8a6", "a8a5", "a8a4", "a8a3", "a8a2", "a8a1", "a8b8", "a8c8", "a8d8", "h8f8",
            "h8g8", "h8h7", "h8h6", "h8h5", "h8h4", "h8h3", "h8h2", "h8h1", "e8e7", "e8d8", "e8d7",
            "e8f8", "e8f7", "e8c8"
        ])
    );
    // Castling short blocked by our piece, castling long is not available.
    assert_eq!(
        get_moves(&setup("r3k2r/8/8/8/8/8/6R1/4K3 b k - 0 1")),
        sorted_moves(&[
            "a8a7", "a8a6", "a8a5", "a8a4", "a8a3", "a8a2", "a8a1", "a8b8", "a8c8", "a8d8", "h8f8",
            "h8g8", "h8h7", "h8h6", "h8h5", "h8h4", "h8h3", "h8h2", "h8h1", "e8e7", "e8d8", "e8d7",
            "e8f8", "e8f7"
        ])
    );
    // Castling long is not blocked: the attacked square is not the one king will
    // walk through.
    assert_eq!(
        get_moves(&setup("r3k2r/8/8/8/8/8/1R6/4K3 b q - 0 1")),
        sorted_moves(&[
            "a8a7", "a8a6", "a8a5", "a8a4", "a8a3", "a8a2", "a8a1", "a8b8", "a8c8", "a8d8", "h8f8",
            "h8g8", "h8h7", "h8h6", "h8h5", "h8h4", "h8h3", "h8h2", "h8h1", "e8e7", "e8d8", "e8d7",
            "e8f8", "e8f7", "e8c8"
        ])
    );
    // Castling long is blocked by an attack and the king is cut off.
    assert_eq!(
        get_moves(&setup("r3k2r/8/8/8/8/8/3R4/4K3 b kq - 0 1")),
        sorted_moves(&[
            "a8a7", "a8a6", "a8a5", "a8a4", "a8a3", "a8a2", "a8a1", "a8b8", "a8c8", "a8d8", "h8f8",
            "h8g8", "h8h7", "h8h6", "h8h5", "h8h4", "h8h3", "h8h2", "h8h1", "e8e7", "e8f8", "e8f7",
            "e8g8"
        ])
    );
}

#[test]
fn chess_programming_wiki_perft_positions() {
    // Positions from https://www.chessprogramming.org/Perft_Results with
    // depth=1.
    // Position 1 is the starting position: handled in detail before.
    // Position 2.
    assert_eq!(
        get_moves(&setup(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
        ))
        .len(),
        48
    );
    // Position 3.
    assert_eq!(
        get_moves(&setup("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1")).len(),
        14,
    );
    // Position 4.
    assert_eq!(
        get_moves(&setup(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"
        ))
        .len(),
        6
    );
    // Mirrored.
    assert_eq!(
        get_moves(&setup(
            "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"
        ))
        .len(),
        6
    );
    // Position 5.
    assert_eq!(
        get_moves(&setup(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"
        ))
        .len(),
        44
    );
    // Position 6
    assert_eq!(
        get_moves(&setup(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
        ))
        .len(),
        46
    );
    // "kiwipete"
    assert_eq!(
        get_moves(&setup(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
        ))
        .len(),
        48
    );
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

// This test is very expensive in the Debug setting (could take 200+ seconds):
// disable it by default.
#[ignore]
#[test]
fn random_positions() {
    for line in read_lines(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/random_positions.fen"
    ))
    .unwrap()
    {
        if let Ok(input) = line {
            let position = Position::from_fen(&input).unwrap();
            let shakmaty_setup: shakmaty::fen::Fen = input.parse().unwrap();
            let shakmaty_position: Chess = shakmaty_setup.position(CastlingMode::Standard).unwrap();
            assert_eq!(
                position
                    .generate_moves()
                    .iter()
                    .map(|m| m.to_string())
                    .sorted()
                    .collect::<Vec<_>>(),
                shakmaty_position
                    .legal_moves()
                    .iter()
                    .map(|m| m.to_uci(CastlingMode::Standard).to_string())
                    .sorted()
                    .collect::<Vec<_>>()
            );
        }
    }
}
