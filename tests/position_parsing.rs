use pabi::chess::position::Position;
use pabi::util;

fn check(serialized_position: &str) {
    let position = Position::try_from(serialized_position.trim());
    assert!(position.is_ok());
    let position = position.unwrap();
    assert_eq!(
        position.to_string(),
        match serialized_position.split_ascii_whitespace().count() {
            6 => serialized_position.trim().to_string(),
            // Patch EPD to validate produced FEN.
            4 => serialized_position.trim().to_string() + " 0 1",
            _ => unreachable!(),
        }
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
