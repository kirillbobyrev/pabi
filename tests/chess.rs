use std::fs;

use itertools::Itertools;
use pabi::chess::core::{Move, Promotion, Square};
use pabi::chess::position::{perft, Position};
use pabi::util;
use pretty_assertions::assert_eq;
use shakmaty::{CastlingMode, Chess, Position as ShakmatyPosition};

fn legal_position(input: &str) {
    let position = Position::from_fen(input).expect("we are parsing valid position: {input}");
    assert_eq!(position.fen(), util::sanitize_fen(input));
}

// TODO: Validate the precise contents of the bitboard directly.
// TODO: Add incorrect ones and validate parsing errors.
#[test]
#[allow(unused_results)]
fn basic_positions() {
    // Full FEN.
    legal_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    legal_position("2r3r1/p3k3/1p3pp1/1B5p/5P2/2P1p1P1/PP4KP/3R4 w - - 0 34");
    legal_position("rnbqk1nr/p3bppp/1p2p3/2ppP3/3P4/P7/1PP1NPPP/R1BQKBNR w KQkq c6 0 7");
    legal_position("r2qkb1r/1pp1pp1p/p1np1np1/1B6/3PP1b1/2N1BN2/PPP2PPP/R2QK2R w KQkq - 0 7");
    legal_position("r3k3/5p2/2p5/p7/P3r3/2N2n2/1PP2P2/2K2B2 w q - 0 24");
    legal_position("r1b1qrk1/ppp2pbp/n2p1np1/4p1B1/2PPP3/2NB1N1P/PP3PP1/R2QK2R w KQ e6 0 9");
    legal_position("8/8/8/8/2P5/3k4/8/KB6 b - c3 0 1");
    legal_position("rnbq1rk1/pp4pp/1b1ppn2/2p2p2/2PP4/1P2PN2/PB2BPPP/RN1Q1RK1 w - c6 0 9");
    // Trimmed FEN.
    legal_position("rnbqkb1r/pp2pppp/3p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R b KQkq -");
}

#[test]
#[should_panic(expected = "expected 1 white king, got 0")]
fn no_white_king() {
    Position::try_from("3k4/8/8/8/8/8/8/8 w - - 0 1").unwrap();
}

#[test]
#[should_panic(expected = "expected 1 black king, got 0")]
fn no_black_king() {
    Position::try_from("8/8/8/8/8/8/8/3K4 w - - 0 1").unwrap();
}

#[test]
#[should_panic(expected = "expected 1 white king, got 3")]
fn too_many_kings() {
    Position::try_from("1kkk4/8/8/8/8/8/8/1KKK4 w - - 0 1").unwrap();
}

#[test]
#[should_panic(expected = "expected <= 8 white pawns, got 9")]
fn too_many_white_pawns() {
    Position::try_from("rnbqkbnr/pppppppp/8/8/8/P7/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
}

#[test]
#[should_panic(expected = "expected <= 8 black pawns, got 9")]
fn too_many_black_pawns() {
    Position::try_from("rnbqkbnr/pppppppp/p7/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
}

#[test]
#[should_panic(expected = "pawns can not be placed on backranks")]
fn pawns_on_backranks() {
    Position::try_from("3kr3/8/8/8/8/5Q2/8/1KP5 w - - 0 1").unwrap();
}

#[test]
#[should_panic(expected = "expected en passant square to be on rank 6, got 3")]
fn wrong_en_passant_player() {
    Position::try_from("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e3 0 1").unwrap();
}

#[test]
#[should_panic(expected = "expected en passant square to be on rank 3, got 4")]
fn wrong_en_passant_rank() {
    Position::try_from("rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq e4 0 1").unwrap();
}

#[test]
#[should_panic(expected = "en passant square is not beyond pushed pawn")]
fn en_passant_not_beyond_pawn() {
    Position::try_from("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq d3 0 1").unwrap();
}

#[test]
#[should_panic(expected = "more than 1 check after double pawn push is impossible")]
fn en_passant_double_check() {
    Position::try_from("r2qkbnr/ppp3Np/8/4Q3/4P3/8/PP4PP/RNB1KB1R b KQkq e3 0 1").unwrap();
}

#[test]
#[should_panic(expected = "expected <= 2 checks, got 3")]
fn tripple_check() {
    Position::try_from("2r3r1/P3k3/prp5/1B5p/5P2/2Q1n2p/PP4KP/3R4 w - - 0 34").unwrap();
}

#[test]
#[should_panic(
    expected = "the only possible checks after double pawn push are either discovery targeting the \
    original pawn square or the pushed pawn itself"
)]
fn check_with_unrelated_en_passant() {
    Position::try_from("rnbqk1nr/bb3p1p/1q2r3/2pPp3/3P4/7P/1PP1NpPP/R1BQKBNR w KQkq c6 0 1")
        .unwrap();
}

#[test]
#[should_panic(expected = "doubly pushed pawn can not be the only blocker on a diagonal")]
fn double_push_blocks_existing_check() {
    Position::try_from("q6k/8/8/3pP3/8/8/8/7K w - d6 0 1").unwrap();
}

#[test]
fn clean_board_str() {
    // Prefix with "fen".
    assert!(Position::try_from(
        "fen rn1qkb1r/pp3ppp/2p1pn2/3p1b2/2PP4/5NP1/PP2PPBP/RNBQK2R w KQkq - 0 1"
    )
    .is_ok());
    // Prefix with "epd".
    assert!(Position::try_from(
        "epd rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R w KQkq -"
    )
    .is_ok());
    // No prefix: infer EPD.
    assert!(Position::try_from("rnbqkbnr/pp2pppp/8/3p4/3P4/3B4/PPP2PPP/RNBQK1NR b KQkq -").is_ok());
    // No prefix: infer FEN.
    assert!(
        Position::try_from("rnbqkbnr/pp2pppp/8/3p4/3P4/3B4/PPP2PPP/RNBQK1NR b KQkq - 0 1").is_ok()
    );
    // Don't crash on unicode symbols.
    assert!(Position::try_from("8/8/8/8/8/8/8/8 b 88 🔠 🔠 ").is_err());
    // Whitespaces at the start/end of the input are not accepted in from_fen but
    // will be cleaned up by try_from.
    assert!(Position::try_from(
        "rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R w KQkq -\n"
    )
    .is_ok());
    assert!(Position::try_from(
        "\n epd rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R w KQkq -"
    )
    .is_ok());
}

// TODO: Test precise error messages.
#[test]
fn no_crash() {
    assert!(Position::try_from("3k2p1N/82/8/8/7B/6K1/3R4/8 b - - 0 1").is_err());
    assert!(Position::try_from("3kn3/R2p1N2/8/8/70000000000000000B/6K1/3R4/8 b - - 0 1").is_err());
    assert!(Position::try_from("3kn3/R4N2/8/8/7B/6K1/3R4/8 b - - 0 48 b - - 0 4/8 b").is_err());
    assert!(Position::try_from("\tfen3kn3/R2p1N2/8/8/7B/6K1/3R4/8 b - - 0 23").is_err());
    assert!(Position::try_from("fen3kn3/R2p1N2/8/8/7B/6K1/3R4/8 b - - 0 23").is_err());
    assert!(Position::try_from("3kn3/R4N2/8/8/7B/6K1/3r4/8 b - - +8 1").is_err());
    assert!(Position::from_fen(
        "\n epd rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R w KQkq -\n"
    )
    .is_err());
}

// This test is very expensive in the Debug setting (could take 200+ seconds):
// disable it by default.
#[ignore]
#[test]
fn arbitrary_positions() {
    for serialized_position in
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/data/positions.fen"))
            .unwrap()
            .lines()
    {
        let position = Position::try_from(serialized_position).unwrap();
        assert_eq!(position.fen(), util::sanitize_fen(serialized_position));
    }
}

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
fn starting_moves_generation() {
    assert_eq!(
        get_moves(&Position::starting()),
        sorted_moves(&[
            "a2a3", "a2a4", "b1a3", "b1c3", "b2b3", "b2b4", "c2c3", "c2c4", "d2d3", "d2d4", "e2e3",
            "e2e4", "f2f3", "f2f4", "g1f3", "g1h3", "g2g3", "g2g4", "h2h3", "h2h4"
        ])
    );
}

#[test]
fn basic_moves_generation() {
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

// Artifacts from the fuzzer or perft.
#[test]
fn fuzzing_artifact_moves() {
    assert_eq!(
        get_moves(&setup(
            "2r3r1/3p3k/1p3pp1/1B5P/5P2/2P1pqP1/PP4KP/3R4 w - - 0 34"
        )),
        sorted_moves(&["g2g1", "g2f3", "g2h3"])
    );
    assert_eq!(
        get_moves(&setup("K7/8/8/8/1R2Pp1k/8/8/8 b - e3 0 1")),
        sorted_moves(&["h4h5", "h4h3", "h4g4", "h4g5", "h4g3", "f4f3"])
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
    assert_eq!(
        get_moves(&setup("8/2p5/3p4/1P5r/KR3p1k/8/4P1P1/8 b - - 1 1")),
        sorted_moves(&[
            "c7c6", "c7c5", "d6d5", "h5b5", "h5c5", "h5d5", "h5e5", "h5g5", "h5f5", "h5h6", "h5h7",
            "h5h8", "h4g4", "h4g5", "h4g3"
        ])
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

// This test is very expensive in the Debug setting (could take 200+ seconds):
// disable it by default.
#[ignore]
#[test]
fn random_positions() {
    for serialized_position in
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/data/positions.fen"))
            .unwrap()
            .lines()
    {
        let position = Position::from_fen(&serialized_position).unwrap();
        let shakmaty_setup: shakmaty::fen::Fen = serialized_position.parse().unwrap();
        let shakmaty_position: Chess = shakmaty_setup.position(CastlingMode::Standard).unwrap();
        let moves = position.generate_moves();
        assert_eq!(
            moves
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
        // TODO: Shakmaty en passant implementation deviates from the standard.
        // for next_move in moves {
        //     let mut next_position = position.clone();
        //     next_position.make_move(&next_move);
        //     let uci =
        // shakmaty::uci::Uci::from_ascii(next_move.to_string().as_bytes()).
        // unwrap();     let mut next_shakmaty_position =
        // shakmaty_position.clone();     next_shakmaty_position.
        // play_unchecked(&uci.to_move(&next_shakmaty_position).unwrap());
        //     assert_eq!(
        //         next_position.fen(),
        //         shakmaty::fen::fen(&next_shakmaty_position),
        //         "original position: {}, move: {}, position: {}, shakmaty:
        // {}",         position.fen(),
        //         next_move.to_string(),
        //         next_position.fen(),
        //         shakmaty::fen::fen(&next_shakmaty_position),
        //     );
        // }
    }
}

#[test]
fn basic_moves() {
    let mut position = setup("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    position.make_move(&Move::new(Square::E2, Square::E4, None));
    assert_eq!(
        position.fen(),
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
    );
    position.make_move(&Move::new(Square::E7, Square::E5, None));
    assert_eq!(
        position.fen(),
        "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2"
    );
    position.make_move(&Move::new(Square::G1, Square::F3, None));
    assert_eq!(
        position.fen(),
        "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2"
    );
    position.make_move(&Move::new(Square::E8, Square::E7, None));
    assert_eq!(
        position.fen(),
        "rnbq1bnr/ppppkppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQ - 2 3"
    );
}

#[test]
fn promotion_moves() {
    let mut position = setup("2n4k/1PP5/6K1/3Pp1Q1/3N4/3P4/P3R3/8 w - - 0 1");
    position.make_move(&Move::new(Square::B7, Square::C8, Some(Promotion::Queen)));
    assert_eq!(
        position.fen(),
        "2Q4k/2P5/6K1/3Pp1Q1/3N4/3P4/P3R3/8 b - - 0 1"
    );
}

#[test]
fn castling_reset() {
    let mut position = setup("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    position.make_move(&Move::new(Square::A1, Square::A8, None));
    assert_eq!(position.fen(), "R3k2r/8/8/8/8/8/8/4K2R b Kk - 0 1");
}

#[test]
fn perft_starting_position() {
    let position = Position::starting();
    assert_eq!(perft(&position, 0), 1);
    assert_eq!(perft(&position, 1), 20);
    assert_eq!(perft(&position, 2), 400);
    assert_eq!(perft(&position, 3), 8902);
    assert_eq!(perft(&position, 4), 197281);
    assert_eq!(perft(&position, 5), 4865609);
}

// Position 2.
#[test]
fn perft_kiwipete() {
    let position = setup("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
    assert_eq!(perft(&position, 1), 48);
    assert_eq!(perft(&position, 2), 2039);
    assert_eq!(perft(&position, 3), 97862);
    assert_eq!(perft(&position, 4), 4085603);
    assert_eq!(perft(&position, 5), 193690690);
}

// Position 3.
#[test]
fn perft_endgame() {
    let position = setup("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    assert_eq!(perft(&position, 1), 14);
    assert_eq!(perft(&position, 2), 191);
    assert_eq!(perft(&position, 3), 2812);
    assert_eq!(perft(&position, 4), 43238);
    assert_eq!(perft(&position, 5), 674624);
}

// Position 4.
#[test]
fn perft_complex() {
    let position = setup("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1");
    assert_eq!(perft(&position, 1), 6);
    assert_eq!(perft(&position, 2), 264);
    assert_eq!(perft(&position, 3), 9467);
    assert_eq!(perft(&position, 4), 422333);
    assert_eq!(perft(&position, 5), 15833292);
}

// Position 5.
#[test]
fn perft_fifth() {
    let position = setup("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    assert_eq!(perft(&position, 1), 44);
    assert_eq!(perft(&position, 2), 1486);
    assert_eq!(perft(&position, 3), 62379);
    assert_eq!(perft(&position, 4), 2103487);
    assert_eq!(perft(&position, 5), 89941194);
}

// Position 6.
#[test]
fn perft_sixth() {
    let position =
        setup("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    assert_eq!(perft(&position, 1), 46);
    assert_eq!(perft(&position, 2), 2079);
    assert_eq!(perft(&position, 3), 89890);
    assert_eq!(perft(&position, 4), 3894594);
    assert_eq!(perft(&position, 5), 164075551);
}

#[test]
fn perft_artifacts() {
    let position = setup("");
    assert_eq!(perft(&position, 1), 46);
    assert_eq!(perft(&position, 2), 2079);
    assert_eq!(perft(&position, 3), 89890);
    assert_eq!(perft(&position, 4), 3894594);
    assert_eq!(perft(&position, 5), 164075551);
}

// TODO: Expensive tests with depth of 6 or so.
// TODO: Parallel perft for higher depths?

// fn validated_perft(position: &Position, depth: u8) -> usize {
//     let mut positions = vec![position.fen()];
//     for level in 0..depth {
//         println!("level: {level}");
//         let mut new_positions = vec![];
//         for position in positions {
//             let generated_positions = setup(&position)
//                 .generate_moves()
//                 .iter()
//                 .map(|m| {
//                     let mut next_pos = setup(&position);
//                     next_pos.make_move(&m);
//                     next_pos.fen()
//                 })
//                 .sorted()
//                 .collect::<Vec<_>>();
//             if level >= 6 {
//                 let shakmaty_setup: shakmaty::fen::Fen =
// position.parse().unwrap();                 let shakmaty_position: Chess =
//                     shakmaty_setup.position(CastlingMode::Standard).unwrap();
//                 let reference_positions = shakmaty_position
//                     .legal_moves()
//                     .iter()
//                     .map(|m| {
//                         let mut next_pos = shakmaty_position.clone();
//                         next_pos.play_unchecked(&m);
//                         shakmaty::fen::fen(&next_pos)
//                     })
//                     .sorted()
//                     .collect::<Vec<_>>();
//                 assert_eq!(
//                     generated_positions.len(),
//                     reference_positions.len(),
//                     "{}",
//                     position
//                 );
//             }
//             for pos in generated_positions {
//                 new_positions.push(pos);
//             }
//         }
//         positions = new_positions;
//     }
//     positions.len()
// }

// // TODO: This is only needed for debugging problems.
// #[test]
// fn shakmaty_perft() {
//     assert_eq!(
//         validated_perft(
//             
// &Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w
// kq - 0 1")                 .unwrap(),
//             6,
//         ),
//         15833292
//     );
// }
