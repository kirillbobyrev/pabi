//! Criterion benchmarks measure time of the clearly separated pieces of code.

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pabi::chess::position::Position;
use pabi::util;
use shakmaty::{CastlingMode, Chess, Position as ShakmatyPosition};

fn parse_positions(positions: &[String]) {
    for position in positions {
        let pos = Position::try_from(position.as_str());
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
        BenchmarkId::new("stockfish books", format!("{} positions", positions.len())),
        &positions,
        |b, positions| {
            b.iter(|| parse_positions(positions));
        },
    );
}

criterion_group! {
    name = position;
    config = Criterion::default().sample_size(10);
    targets = parse
}

fn read_lines<P>(filename: P) -> io::Lines<io::BufReader<File>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).unwrap();
    io::BufReader::new(file).lines()
}

fn generate_moves(positions: &[Position]) {
    for position in positions {
        criterion::black_box(position.generate_moves());
    }
}

// TODO: Add Throughput.
fn movegen_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Move generation");
    let mut positions = vec![];
    for line in read_lines(concat!(env!("CARGO_MANIFEST_DIR"), "/data/positions.fen")) {
        if let Ok(input) = line {
            positions.push(Position::try_from(input.as_str()).unwrap());
        }
    }
    group.bench_with_input(
        BenchmarkId::new("pabi", format!("{} arbitrary positions", positions.len())),
        &positions,
        |b, positions| {
            b.iter(|| generate_moves(positions));
        },
    );
    // Add a benchmark for shakmaty: this is a reasonable reference that has
    // stable performance and can be compared to. Pabi does more during the move
    // generation (namely, calculates attack info including pins and xrays) so
    // it's not important to be faster than this but it's an important reference
    // point.
    let mut shakmaty_positions = Vec::<Chess>::new();
    for line in read_lines(concat!(env!("CARGO_MANIFEST_DIR"), "/data/positions.fen")) {
        if let Ok(input) = line {
            let shakmaty_setup: shakmaty::fen::Fen = input.parse().unwrap();
            shakmaty_positions.push(shakmaty_setup.position(CastlingMode::Standard).unwrap());
        }
    }
    group.bench_with_input(
        BenchmarkId::new(
            "reference implementation: shakmaty",
            format!("{} arbitrary positions", shakmaty_positions.len()),
        ),
        &shakmaty_positions,
        |b, positions| {
            b.iter(|| {
                for position in positions {
                    criterion::black_box(position.legal_moves());
                }
            });
        },
    );
    group.finish();
}

criterion_group! {
    name = movegen;
    config = Criterion::default().sample_size(100);
    targets = movegen_bench
}

// TODO: Perft.

criterion_main!(position, movegen);
