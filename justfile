# This is a collection of commonly used recipes for development. Most of them
# are wrappers around Cargo and Cargo extensions with certain setup.

# Enable x86_64 features that are required for maximum performance.
compile_flags := "RUSTFLAGS='-C target-feature=+avx2,+fma,+bmi1,+bmi2'"

build:
  {{ compile_flags }} cargo build --profile=release

# Runs the engine and enters UCI mode.
run:
  {{ compile_flags}} cargo run --profile=release

fmt:
  cargo +nightly fmt --all

# Checks the code for bad formatting, errors and warnings.
lint:
  cargo +nightly fmt --all -- --check
  cargo +nightly clippy --all-targets --all-features

# Runs the linters and tries to apply automatic fixes.
fix:
  cargo +nightly fmt --all
  cargo +nightly clippy --all-targets --all-features --fix --allow-staged

# Run most tests in debug mode to (potentially) cat more errors with
# debug_assert.
test:
  {{ compile_flags }} cargo test

# Run tests that are slow and are not run by default.
test_slow:
  {{ compile_flags }} cargo test --profile=release -- --ignored

# Run all tests.
test_all: test test_slow

bench:
  {{ compile_flags }} cargo bench --profile=release

# Lists all fuzzing targets that can be used as inputs for fuzz command.
list_fuzz_targets:
  cd fuzz
  cargo +nightly fuzz list

fuzz target:
  cd fuzz
  {{ compile_flags }} cargo +nightly --profile=release fuzz run {{ target }}

# Play a single game between two engine versions in 2'+1'' format and save the
# game PGN.
play engine1_cmd engine2_cmd outfile:
  cutechess-cli -engine cmd={{ engine1_cmd }} -engine cmd={{ engine2_cmd }} -each proto=uci tc=120+1 -pgnout file min
