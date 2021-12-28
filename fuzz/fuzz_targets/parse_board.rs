#![no_main]
use libfuzzer_sys::fuzz_target;
use pabi::board;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = board::Board::try_from(s);
        // TODO: Check printing the board back to FEN.
    }
});
