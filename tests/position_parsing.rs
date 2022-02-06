use pabi::chess::position::Position;
use pabi::util;

fn check(serialized_position: &str) {
    let position = Position::try_from(serialized_position)
        .unwrap_or_else(|_| panic!("we are checking valid positions: {serialized_position}"));
    assert_eq!(
        position.to_string(),
        util::sanitize_fen(serialized_position)
    );
    assert!(position.is_legal());
}

// This test is very expensive in the Debug setting (could take 200+ seconds):
// disable it by default.
#[ignore]
#[test]
fn stockfish_books() {
    for book in util::stockfish_books() {
        for serialized_position in util::read_compressed_book(&book).lines() {
            check(serialized_position);
        }
    }
}
