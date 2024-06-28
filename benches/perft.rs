//! Criterion benchmarks measure time of move generation and perft calculation.

use std::fs;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pabi::chess::position::Position;
use shakmaty::{CastlingMode, Chess, Position as ShakmatyPosition};

fn generate_moves(positions: &[Position]) {
    for position in positions {
        std::hint::black_box(position.generate_moves());
    }
}

// TODO: Add Throughput.
fn movegen_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Move generation");
    let mut positions = vec![];
    for line in fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/positions.fen"
    ))
    .unwrap()
    .lines()
    {
        positions.push(Position::try_from(line).unwrap());
    }
    group.throughput(criterion::Throughput::Elements(positions.len() as u64));
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
    for line in fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/positions.fen"
    ))
    .unwrap()
    .lines()
    {
        let shakmaty_setup: shakmaty::fen::Fen = line.parse().unwrap();
        shakmaty_positions.push(
            shakmaty_setup
                .into_position(CastlingMode::Standard)
                .unwrap(),
        );
    }
    group.throughput(criterion::Throughput::Elements(
        shakmaty_positions.len() as u64
    ));
    group.bench_with_input(
        BenchmarkId::new(
            "movegen_reference_shakmaty",
            format!("{} arbitrary positions", shakmaty_positions.len()),
        ),
        &shakmaty_positions,
        |b, positions| {
            b.iter(|| {
                for position in positions {
                    std::hint::black_box(position.legal_moves());
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

// This acts both as performance and correctness test.
fn perft_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("perft");
    // TODO: Add Throughput - it should be the number of nodes.
    for (position, depth, nodes) in &[
        // Position 1.
        (Position::starting(), 5, 4_865_609),
        (Position::starting(), 6, 119_060_324),
        // Position 3.
        (
            Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap(),
            6,
            11_030_083,
        ),
        // Position 4.
        (
            Position::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1")
                .unwrap(),
            6,
            706_045_033,
        ),
        // Position 6.
        (
            Position::from_fen(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            )
            .unwrap(),
            5,
            164_075_551,
        ),
        // Other positions.
        (
            Position::from_fen("r1bqkbnr/pppppppp/2n5/8/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 1 2")
                .unwrap(),
            6,
            336_655_487,
        ),
        (
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/N7/PPPPPPPP/R1BQKBNR b KQkq - 1 1")
                .unwrap(),
            6,
            120_142_144,
        ),
    ] {
        group.throughput(criterion::Throughput::Elements(*nodes));
        group.bench_with_input(
            BenchmarkId::new(
                "perft",
                format!("position {position}, depth {depth}, nodes {nodes}"),
            ),
            depth,
            |b, &depth| {
                b.iter(|| {
                    assert_eq!(pabi::chess::position::perft(position, depth), *nodes);
                });
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = perft;
    config = Criterion::default().sample_size(10);
    targets = perft_bench
}

criterion_main!(movegen, perft);
