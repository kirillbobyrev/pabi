# Pabi

[![Codecov](https://codecov.io/gh/kirillbobyrev/pabi/branch/main/graph/badge.svg)](https://codecov.io/gh/kirillbobyrev/pabi)
[![Lines of Code](https://tokei.rs/b1/github/kirillbobyrev/pabi)](https://github.com/kirillbobyrev/pabi/tree/main/src)

[![Build](https://github.com/kirillbobyrev/pabi/actions/workflows/build.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/build.yml)
[![Test Suite](https://github.com/kirillbobyrev/pabi/actions/workflows/test.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/test.yml)
[![Lint](https://github.com/kirillbobyrev/pabi/actions/workflows/lint.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/lint.yml)
[![Security audit](https://github.com/kirillbobyrev/pabi/actions/workflows/audit.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/audit.yml)

[![Dependencies](https://deps.rs/repo/github/kirillbobyrev/pabi/status.svg)](https://deps.rs/repo/github/kirillbobyrev/pabi)

Pabi is a modern chess engine that is currently under development. The engine
itself is implemented in this repository, training and development of the Neural
Network for position evaluation is in
[kirillbobyrev/pabi-brain](https://github.com/kirillbobyrev/pabi-brain).

## Goals

Pabi is inspired by existing Chess and Go engines (mainly [AlphaGo], [lc0],
[Ceres] and [KataGo]). It strives to be a high-quality modern engine.

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
- Documentation: [rustdoc] has awesome features, e.g. warning on
  undocumented code and testing example code in the documentation.

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

[AlphaGo]: https://en.wikipedia.org/wiki/AlphaGo
[lc0]: https://lczero.org/
[Ceres]: https://github.com/dje-dev/Ceres
[KataGo]: https://github.com/lightvector/KataGo
[Rust]: https://www.rust-lang.org/
[Fuzzing]: https://en.wikipedia.org/wiki/Fuzzing
[GitHub Actions]: https://github.com/features/actions
[Dependabot]: https://github.com/dependabot
[rustdoc]: https://doc.rust-lang.org/rustdoc
[Computer Chess Championship]: https://www.chess.com/computer-chess-championship
[TCEC]: https://tcec-chess.com/
[Milestones]: https://github.com/kirillbobyrev/pabi/milestones
[0.1.0]: https://github.com/kirillbobyrev/pabi/milestone/1
[1.0.0]: https://github.com/kirillbobyrev/pabi/milestone/2
[2.0.0]: https://github.com/kirillbobyrev/pabi/milestone/3
