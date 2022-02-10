//! Criterion benchmarks measure time of the clearly separated pieces of code.

use std::fs;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pabi::chess::position::Position;
use shakmaty::{CastlingMode, Chess, Position as ShakmatyPosition};

// TODO: Add Throughput.
fn parse(c: &mut Criterion) {
    let positions = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/data/positions.fen"))
        .unwrap()
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    c.bench_with_input(
        BenchmarkId::new(
            "stockfish books",
            format!("{} arbitrary positions", positions.len()),
        ),
        &positions,
        |b, positions| {
            b.iter(|| {
                for position in positions {
                    let pos = Position::try_from(position.as_str());
                    assert!(pos.is_ok());
                }
            });
        },
    );
}

criterion_group! {
    name = position;
    config = Criterion::default().sample_size(10);
    targets = parse
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
    for line in fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/data/positions.fen"))
        .unwrap()
        .lines()
    {
        positions.push(Position::try_from(line).unwrap());
    }
    group.bench_with_input(
        BenchmarkId::new(
            "movegen_pabi",
            format!("{} arbitrary positions", positions.len()),
        ),
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
    for line in fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/data/positions.fen"))
        .unwrap()
        .lines()
    {
        let shakmaty_setup: shakmaty::fen::Fen = line.parse().unwrap();
        shakmaty_positions.push(shakmaty_setup.position(CastlingMode::Standard).unwrap());
    }
    group.bench_with_input(
        BenchmarkId::new(
            "movegen_reference_shakmaty",
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
