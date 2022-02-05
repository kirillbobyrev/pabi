use pabi::chess::position::Position;
use pabi::util;

fn parse_stockfish_book_positions() {
    for book in util::stockfish_books() {
        for serialized_position in util::read_compressed_book(&book).lines() {
            let pos = Position::try_from(serialized_position);
            assert!(pos.is_ok());
        }
    }
}

iai::main!(parse_stockfish_book_positions);
