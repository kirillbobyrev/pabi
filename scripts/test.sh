typos
cargo +nightly fmt --all
cargo +nightly clippy --all-features --fix --allow-staged
cargo test
cargo test --release -- --ignored
RUSTFLAGS='-C target-feature=+avx2,+fma,+bmi,+bmi2' cargo bench --profile=fast
