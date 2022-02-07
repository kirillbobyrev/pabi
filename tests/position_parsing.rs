use pabi::chess::position::Position;
use pabi::util;
use pretty_assertions::assert_eq;

// TODO: Inline and check error messages.
fn illegal_position(input: &str) {
    assert!(Position::from_fen(input).is_err());
}

fn legal_position(input: &str) {
    let position = Position::from_fen(input).expect("we are parsing valid position: {input}");
    assert_eq!(position.to_string(), util::sanitize_fen(input));
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

// TODO: Check precise error messages.
#[test]
fn illegal_positions() {
    // No white kings.
    illegal_position("3k4/8/8/8/8/8/8/8 w - - 0 1");
    // No white kings.
    illegal_position("3k4/8/8/8/8/8/8/8 w - - 0 1");
    // No black kings.
    illegal_position("8/8/8/8/8/8/8/3K4 w - - 0 1");
    // Too many kings.
    illegal_position("1kkk4/8/8/8/8/8/8/1KKK4 w - - 0 1");
    // Too many white pawns.
    illegal_position("rnbqkbnr/pppppppp/8/8/8/P7/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    // Too many black pawns.
    illegal_position("rnbqkbnr/pppppppp/p7/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    // Pawns on backranks.
    illegal_position("3kr3/8/8/8/8/5Q2/8/1KP5 w - - 0 1");
    // En passant square not behind a pushed pawn.
    illegal_position("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq d3 0 1");
    // Wrong en passant rank.
    illegal_position("rnbqkbnr/pppppppp/8/4P3/8/8/PPPP1PPP/RNBQKBNR b KQkq e4 0 1");
    // En passant can't result in double check.
    illegal_position("r2qkbnr/ppp3Np/8/4Q3/4P3/8/PP4PP/RNB1KB1R w KQkq e3 0 1");
    // The check was not delivered by the doubly pushed pawn.
    illegal_position("rnbqk1nr/bb3p1p/1q2r3/2pPp3/3P4/7P/1PP1NpPP/R1BQKBNR w KQkq c6 0 1");
    // Three checks.
    illegal_position("2r3r1/P3k3/prp5/1B5p/5P2/2Q1n2p/PP4KP/3R4 w - - 0 34");
    // Doubly pushed pawn can not block a diagonal check.
    illegal_position("q6k/8/8/3pP3/8/8/8/7K w - d6 0 1");
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
    assert!(Position::from_fen(
        "\n epd rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R w KQkq -\n"
    )
    .is_err());
}

#[test]
fn no_crash() {
    assert!(Position::try_from("3k2p1N/82/8/8/7B/6K1/3R4/8 b - - 0 1").is_err());
    assert!(Position::try_from("3kn3/R2p1N2/8/8/70000000000000000B/6K1/3R4/8 b - - 0 1").is_err());
    assert!(Position::try_from("3kn3/R4N2/8/8/7B/6K1/3R4/8 b - - 0 48 b - - 0 4/8 b").is_err());
    assert!(Position::try_from("\tfen3kn3/R2p1N2/8/8/7B/6K1/3R4/8 b - - 0 23").is_err());
    assert!(Position::try_from("fen3kn3/R2p1N2/8/8/7B/6K1/3R4/8 b - - 0 23").is_err());
    assert!(Position::try_from("3kn3/R4N2/8/8/7B/6K1/3r4/8 b - - +8 1").is_err());
}

// This test is very expensive in the Debug setting (could take 200+ seconds):
// disable it by default.
#[ignore]
#[test]
fn stockfish_books() {
    for book in util::stockfish_books() {
        for serialized_position in util::read_compressed_book(&book).lines() {
            let position = Position::try_from(serialized_position).unwrap();
            assert_eq!(
                position.to_string(),
                util::sanitize_fen(serialized_position)
            );
        }
    }
}
