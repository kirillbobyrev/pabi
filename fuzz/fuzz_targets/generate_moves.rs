#![no_main]
use itertools::Itertools;
use libfuzzer_sys::fuzz_target;
use pabi::chess::position;
use pretty_assertions::assert_eq;
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
    let s = pabi::util::sanitize_fen(s);
    let shakmaty_setup: shakmaty::fen::Fen = s
        .parse()
        .expect("If we parsed a position, it should be parsed by shakmaty, too.");
    let shakmaty_position: Result<Chess, _> = shakmaty_setup.position(CastlingMode::Standard);
    if shakmaty_position.is_err() {
        return;
    }
    dbg!();
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
