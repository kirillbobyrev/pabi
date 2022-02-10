//! Convenient utility functions for Pabi that can be used from benchmarks and
//! public tests.

// TODO: Docs.
#[must_use]
pub fn sanitize_fen(position: &str) -> String {
    let mut position = position.trim();
    for prefix in ["fen ", "epd "] {
        if let Some(stripped) = position.strip_prefix(prefix) {
            position = stripped;
        }
    }
    match position.split_ascii_whitespace().count() {
        6 => position.to_string(),
        // Patch EPD to validate produced FEN.
        4 => position.to_string() + " 0 1",
        _ => unreachable!(),
    }
}
