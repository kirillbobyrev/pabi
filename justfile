# This is a collection of commonly used recipes for development. Most of them
# are wrappers around Cargo with the right flags and settings.

build:
  cargo build --profile=release

# Runs the engine and enters UCI mode.
run:
  cargo run --profile=release

# Format all code.
fmt:
  cargo +nightly fmt --all

# Checks the code for bad formatting, errors and warnings.
lint:
  cargo +nightly fmt --all -- --check
  cargo clippy --all-targets --all-features

# Runs the linters and tries to apply automatic fixes.
fix: fmt
  cargo clippy --all-targets --all-features --fix --allow-staged

# Run most tests in debug mode to (potentially) catch more errors with
# debug_assert.
test:
  cargo test

# Run tests that are slow and are not run by default.
test_slow:
  cargo test --profile=release -- --ignored

# Run all tests.
test_all: test test_slow

bench:
  cargo bench --profile=release

# Lists all fuzzing targets that can be used as inputs for fuzz command.
list_fuzz_targets:
  cd fuzz
  cargo +nightly fuzz list

fuzz target:
  cd fuzz
  cargo +nightly --profile=release fuzz run {{ target }}

# Build developer documentation.
doc:
  cargo doc --document-private-items --no-deps
