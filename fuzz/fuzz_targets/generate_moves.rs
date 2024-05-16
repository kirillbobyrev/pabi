#![no_main]
use itertools::Itertools;
use libfuzzer_sys::fuzz_target;
use pabi::chess::position;
use pretty_assertions::assert_eq;
use shakmaty::{CastlingMode, Chess, Position};

fuzz_target!(|data: &[u8]| {
    let input = match std::str::from_utf8(data) {
        Ok(input) => input,
        Err(_) => return,
    };
    let position = match position::Position::from_fen(input) {
        Ok(position) => position,
        Err(_) => return,
    };
    let shakmaty_setup: shakmaty::fen::Fen = input
        .parse()
        .expect("when we parsed a valid position it should be accepted by shakmaty");
    let shakmaty_position: Result<Chess, _> = shakmaty_setup.into_position(CastlingMode::Standard);
    if shakmaty_position.is_err() {
        return;
    }
    assert_eq!(
        position
            .generate_moves()
            .iter()
            .map(|m| m.to_string())
            .sorted()
            .collect::<Vec<_>>(),
        shakmaty_position
            .as_ref()
            .unwrap()
            .legal_moves()
            .iter()
            .map(|m| m.to_uci(CastlingMode::Standard).to_string())
            .sorted()
            .collect::<Vec<_>>()
    );
});
