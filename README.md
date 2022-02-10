# Pabi

[![Development](https://img.shields.io/badge/development-work%20in%20progress-red)](https://github.com/github/kirillbobyrev/pabi)
[![Codecov](https://codecov.io/gh/kirillbobyrev/pabi/branch/main/graph/badge.svg)](https://codecov.io/gh/kirillbobyrev/pabi)
[![Lines of Code](https://tokei.rs/b1/github/kirillbobyrev/pabi)](https://github.com/kirillbobyrev/pabi/tree/main/src)

[![Build](https://github.com/kirillbobyrev/pabi/actions/workflows/build.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/build.yml)
[![Test Suite](https://github.com/kirillbobyrev/pabi/actions/workflows/test.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/test.yml)
[![Lint](https://github.com/kirillbobyrev/pabi/actions/workflows/lint.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/lint.yml)
[![Security audit](https://github.com/kirillbobyrev/pabi/actions/workflows/audit.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/audit.yml)

[![Dependencies](https://deps.rs/repo/github/kirillbobyrev/pabi/status.svg)](https://deps.rs/repo/github/kirillbobyrev/pabi)

Pabi is a hypermodern chess engine.

## Goals

Pabi is inspired by existing Chess and Go engines (mainly [AlphaGo], [lc0],
[Ceres] and [KataGo]). It strives to be a high-quality hypermodern engine.

**Hypermodern**: Pabi should use up-to-date [Rust] toolchain, is targeting new
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

Pabi strives to be **user-** and **developer-friendly**.

- Building/downloading it should be straightforward. Anyone willing to try
  it out can either run it locally (possibly in the [Web UI]) or on Lichess.org
  with stronger hardware ([Lichess deployment]).
- The code is easy to follow and understand. Where appropriate, links to
  [Chess Programming Wiki] complement the documentation.
- Development process is low-effort: the infrastructure automates most tedious
  tasks and requires minimal manual labor.
- Contributing is low-effort: a [guide] and [architecture overview] define
  important concepts for potential contributors.
- Modular and orthogonal architecture makes it easy to isolate changes,
  making their impact obvious in combination with CI and automated checks
  run in each Pull Request.

Pabi strives to be a **platform** for experimentation. Testing new ideas is only
valuable if the outcome the experiment results are quantified and easily
interpreted. There are plenty of interesting ideas to be tried (e.g. different
evaluation network architecture, different training methods and network
distillation/quantization as means of compression). See [Resources] for a list
of candidates.

Pabi strives to be **strong**. Ultimately, when it is strong enough it should
participate in Chess.com [Computer Chess Championship][^cccc] and
[TCEC]. As such, it should be tested in appropriate time formats (mainly Blitz
and Rapid) that are popular in these rating lists and designed to perform well
against other engines. Performance and good time-management are crucial for
doing well under time pressure.

## Getting involved

If you want to start contributing to the project, here are the resources that
will help you get started:

- [Building guide](/BUILDING.md)
- [Contribution guide](/CONTRIBUTING.md)
- [Architecture overview](/ARCHITECTURE.md)
- [GitHub Issues](https://github.com/kirillbobyrev/pabi/issues) have some tasks
  you can pick up. Issues with ["good first issue"
  label](https://github.com/kirillbobyrev/pabi/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)
  are the best if you are new to the project.

[Developer Documentation](https://kirillbobyrev.github.io/pabi/docs/pabi/index.html)
For easier code navigation, read the developer documentation that is updated
after each commit.

## [Milestones]

The project is in pre-0.1.0 stage and the plan can be changed.

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
[Web UI]: https://github.com/kirillbobyrev/pabi/issues/14
[Lichess deployment]: https://github.com/kirillbobyrev/pabi/issues/14
[Chess Programming Wiki]: https://www.chessprogramming.org/Main_Page
[guide]: https://github.com/kirillbobyrev/pabi/issues/15
[architecture overview]: https://github.com/kirillbobyrev/pabi/issues/4
[Resources]: https://github.com/kirillbobyrev/pabi/wiki/Resources
[Computer Chess Championship]: https://www.chess.com/computer-chess-championship
[TCEC]: https://tcec-chess.com/
[Milestones]: https://github.com/kirillbobyrev/pabi/milestones
[0.1.0]: https://github.com/kirillbobyrev/pabi/milestone/1
[1.0.0]: https://github.com/kirillbobyrev/pabi/milestone/2
[2.0.0]: https://github.com/kirillbobyrev/pabi/milestone/3

[^cccc]:
    More details on CCCC:
    <https://www.chess.com/news/view/announcing-the-new-computer-chess-championship>
