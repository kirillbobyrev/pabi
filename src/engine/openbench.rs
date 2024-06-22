//! Implementing [`bench`] command is a [requirement for OpenBench], which is an
//! incredibly important tool for measuring the performance and strenght of the
//! engine.
//!
//! [requirement for OpenBench]: https://github.com/AndyGrant/OpenBench/wiki/Requirements-For-Public-Engines#basic-requirements

/// Runs search on a small set of positions to provide an estimate of engine's
/// performance.
///
/// NOTE: This function **has to run less than 60 seconds**.
///
/// See <https://github.com/AndyGrant/OpenBench/blob/master/Client/bench.py> for more details.
pub fn bench(out: &mut dyn std::io::Write) {
    todo!()
}
