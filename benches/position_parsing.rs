//! FEN/EPD serialized positions parsing.

use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pabi::chess::position::Position;
use pabi::util;

fn parse_positions(positions: &[String]) {
    for position in positions {
        let pos = Position::try_from(position.as_str().trim());
        assert!(pos.is_ok());
    }
}

// TODO: Add Throughput.
fn parse(c: &mut Criterion) {
    let mut positions = vec![];
    for book in util::stockfish_books() {
        for serialized_position in util::read_compressed_book(&book).lines() {
            positions.push(serialized_position.to_string());
        }
    }
    c.bench_with_input(
        BenchmarkId::new(
            "position parsing",
            format!("{} stockfish positions", positions.len()),
        ),
        &positions,
        |b, positions| {
            b.iter(|| parse_positions(positions));
        },
    );
}

criterion_group! {
    name = position;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(120))
        .measurement_time(Duration::from_secs(60));
    targets = parse
}

criterion_main!(position);
