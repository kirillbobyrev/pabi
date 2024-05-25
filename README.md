<h1 align="center">
  Pabi
</h1>

[![Codecov](https://codecov.io/gh/kirillbobyrev/pabi/branch/main/graph/badge.svg)](https://codecov.io/gh/kirillbobyrev/pabi)
[![Dependencies](https://deps.rs/repo/github/kirillbobyrev/pabi/status.svg)](https://deps.rs/repo/github/kirillbobyrev/pabi)

[![Build](https://github.com/kirillbobyrev/pabi/actions/workflows/build.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/build.yml)
[![Test Suite](https://github.com/kirillbobyrev/pabi/actions/workflows/test.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/test.yml)
[![Lint](https://github.com/kirillbobyrev/pabi/actions/workflows/lint.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/lint.yml)
[![Security audit](https://github.com/kirillbobyrev/pabi/actions/workflows/audit.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/audit.yml)

Pabi is a modern chess engine that is currently under development. The engine
itself is implemented in this repository, training and development of the Neural
Network for position evaluation is in
[kirillbobyrev/pabi-brain](https://github.com/kirillbobyrev/pabi-brain).

For architecture, design and development process overview, please see
[CONTRIBUTING.md](/CONTRIBUTING.md).

## Goals

Pabi is inspired by existing Chess and Go engines (mainly [Stockfish], [lc0],
and [KataGo]). It strives to be a high-quality modern engine.

**Modern**: Pabi should use up-to-date [Rust] toolchain, is targeting modern
processor architectures, uses latest developments in the domains of programming
tooling, Chess Programming and Machine Learning.

**High-quality**: Pabi will take full advantage of

- Unit testing and integration testing: the majority of the codebase is
  well-tested and the HEAD is in the functional state. Pabi also uses advanced
  testing techniques such as [Fuzzing].
- Performance testing and benchmarking: Pabi should be **fast**. Continuous
  integration runs benchmarks and tests after every change.
- Continuous quality assurance: [GitHub Actions] make it possible to lint, test,
  benchmark and warn on regressions/failures. [Dependabot] will warn about
  outdated dependencies and security vulnerabilities.
- Documentation: leveraging [rustdoc] to full potential and making the codebase
  accessible.

Pabi strives to be **strong**. The ultimate goal is to enter the lists of top
chess engines and participate in tournaments, such as Chess.com [Computer Chess
Championship] and [TCEC]. As such, it should be tested in appropriate time
formats (mainly Blitz and Rapid) that are popular in these rating lists and
designed to perform well against other engines. Performance and good
time-management are crucial for doing well under time pressure.

## [Milestones]

The project is in pre-0.1.0 stage, the plan is vague and subject to change.

- [0.1.0] should be the skeleton of a chess engine. Most important features are
  implemented or drafted, the performance testing is in-place. There is a
  Command-Line Interface for interacting with the engine, debugging it and
  trying it out.
- [1.0.0]: Pabi is a conventional and fairly strong chess engine. It can be
  played locally or online against other engines or human players, although
  its strength is not State-of-The-Art strength yet.
- [2.0.0] is possibly the ultimate goal. The Neural Networks architectures and
  training are designed specifically with Pabi architecture in mind and new
  ideas are explored.

[0.1.0]: https://github.com/kirillbobyrev/pabi/milestone/1
[1.0.0]: https://github.com/kirillbobyrev/pabi/milestone/2
[2.0.0]: https://github.com/kirillbobyrev/pabi/milestone/3
[Computer Chess Championship]: https://www.chess.com/computer-chess-championship
[Dependabot]: https://github.com/dependabot
[Fuzzing]: https://en.wikipedia.org/wiki/Fuzzing
[GitHub Actions]: https://github.com/features/actions
[KataGo]: https://github.com/lightvector/KataGo
[Milestones]: https://github.com/kirillbobyrev/pabi/milestones
[Rust]: https://www.rust-lang.org/
[Stockfish]: https://stockfishchess.org/
[TCEC]: https://tcec-chess.com/
[lc0]: https://lczero.org/
[rustdoc]: https://doc.rust-lang.org/rustdoc
