#![no_main]
use libfuzzer_sys::fuzz_target;
use pabi::chess::position;
use shakmaty::{CastlingMode, Chess, Position};

fuzz_target!(|data: &[u8]| {
    let s = std::str::from_utf8(data);
    if s.is_err() {
        return;
    }
    let s = s.unwrap().trim();
    let position = position::Position::try_from(s);
    if position.is_err() {
        return;
    }
    let position = position.unwrap();
    if !position.is_legal() {
        return;
    }
    let shakmaty_setup: shakmaty::fen::Fen = s
        .trim()
        .parse()
        .expect("If we parsed a position, it should be parsed by shakmaty, too.");
    // let shakmaty_position: Chess = shakmaty_setup
    //     .position(CastlingMode::Standard)
    //     .expect("should be able to construct the position we could setup");
    // assert_eq!(
    //     position.generate_moves().len() + 1,
    //     shakmaty_position.legal_moves().len()
    // );
});
