# This is a collection of commonly used recipes for development. Most of them
# are wrappers around Cargo and Cargo extensions with certain setup.

# Enable x86_64 features that are required for maximum performance.
compile_flags := "RUSTFLAGS='-C target-feature=+avx2,+fma,+bmi1,+bmi2'"

build:
  {{ compile_flags }} cargo build --profile=fast

# Runs the engine and enters UCI mode.
run:
  {{ compile_flags}} cargo run --profile=fast

# Checks the code for bad formatting, errors and warnings.
lint:
  cargo +nightly fmt --all -- --check
  cargo +nightly clippy --all-features

# Runs the linters and tries to apply automatic fixes.
fix:
  cargo +nightly fmt --all
  cargo +nightly clippy --all-features --fix --allow-staged

# Run most tests that are fast and are run by default.
test_basic:
  cargo test

# Run tests that are slow and are not run by default.
test_slow:
  {{ compile_flags }} cargo test --profile=fast -- --ignored

# Run all tests.
test: test_basic test_slow

bench:
  {{ compile_flags }} cargo bench --profile=fast

# Lists all fuzzing targets that can be used as inputs for fuzz command.
list_fuzz_targets:
  cd fuzz
  cargo +nightly fuzz list

fuzz target:
  cd fuzz
  {{ compile_flags }} cargo +nightly fuzz run {{ target }}
