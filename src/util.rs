//! Convenient utility functions for Pabi that can be used from benchmarks and
//! public tests.

use std::io::Read;
use std::{fs, path};

// TODO: These functions should be tested, documented and maybe eventually
// turned into something safer.
#[must_use]
pub fn stockfish_books() -> Vec<path::PathBuf> {
    let mut books_root = path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    books_root.push("data/books/");
    let mut result = vec![];
    for file in fs::read_dir(books_root).unwrap() {
        // books/ directory contains other files and non-FEN/EPD format books.
        // We only need the ones in correct format.
        let candidate_path = file.unwrap().path();
        if candidate_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .ends_with(".epd.zip")
        {
            result.push(candidate_path);
        }
    }
    result
}

#[must_use]
pub fn read_compressed_book(book: &path::PathBuf) -> String {
    let file = fs::File::open(&book).unwrap();
    let mut archive = zip::read::ZipArchive::new(file).unwrap();
    assert_eq!(archive.len(), 1);
    let mut contents = String::new();
    let status = archive.by_index(0).unwrap().read_to_string(&mut contents);
    assert!(status.is_ok());
    contents
}
