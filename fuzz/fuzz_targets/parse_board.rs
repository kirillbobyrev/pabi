#![no_main]
use libfuzzer_sys::fuzz_target;
use pabi::chess::position;

fuzz_target!(|data: &[u8]| {
    let input = match std::str::from_utf8(data) {
        Ok(input) => input,
        Err(_) => return,
    };
    drop(position::Position::try_from(input))
});
