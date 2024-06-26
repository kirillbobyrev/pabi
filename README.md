<h1 align="center">
  Pabi
</h1>

[![Codecov](https://codecov.io/gh/kirillbobyrev/pabi/branch/main/graph/badge.svg)](https://codecov.io/gh/kirillbobyrev/pabi)
[![Dependencies](https://deps.rs/repo/github/kirillbobyrev/pabi/status.svg)](https://deps.rs/repo/github/kirillbobyrev/pabi)

[![Build](https://github.com/kirillbobyrev/pabi/actions/workflows/build.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/build.yml)
[![Test Suite](https://github.com/kirillbobyrev/pabi/actions/workflows/test.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/test.yml)
[![Lint](https://github.com/kirillbobyrev/pabi/actions/workflows/lint.yml/badge.svg)](https://github.com/kirillbobyrev/pabi/actions/workflows/lint.yml)

Pabi is a modern chess engine that is currently under development.

## Goals

Pabi is inspired by existing Chess and Go engines (mainly [lc0] and [KataGo]).
It strives to be a high-quality modern engine.

**Modern**: Pabi should use up-to-date [Rust] toolchain, is targeting modern
processor architectures, uses latest developments in the domains of programming
tooling, Chess Programming and Machine Learning.

**High-quality**: Pabi will take full advantage of

- Unit testing and integration testing: the majority of the codebase should be
  well-tested and validated. Pabi also uses advanced testing techniques such as
  [Fuzzing].
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

<!-- TODO: User interface: supported commands + UCI options -->
<!-- Describe high-level features -->

## Project architecture

This document describes high-level architecture of Pabi. If you want to get
familiar with the code and understand the project structure, this is a great
starting point.

### Scope

Implementing a *general-purpose* chess engine that would truly excel in
different domains, such as playing with humans, being able to adjust the
strength, providing the best possible analysis for a position when given a lot
of time and, finally, playing well against other engines is an enormous task,
especially for one person. Therefore, it's crucial to choose the right scope and
design the engine with chosen limitations in mind.

Pabi chooses to be the best possible version of an engine that does well against
other chess engines in online tournaments. That means that it will prioritize
performance under the constraints put by the rules and environments of such
tournaments. Today the most interesting tournaments are are
[TCEC](https://tcec-chess.com/) and
[CCCC](https://www.chess.com/computer-chess-championship), and the most
prominent rating list is [CCRL](https://computerchess.org.uk/ccrl/). The first
goal is reaching 3000 ELO on the rating lists that are accepted by the
organizers of these tournaments.

Most of the competitions are in relatively fast time controls (Blitz, Bullet,
Rapid) with some exceptions in relatively short classical formats. In both [CCRL
rules] and [TCEC rules], as well as in CCCC the engines are usually starting
from pre-determined positions to make the games interesting and unbalanced.

In all cases, the testing environment's CPU is of x86_64 architecture, which is
allows using PEXT and PDEP instructions to significantly increase the
performance of move generator. Also, usually, many CPU cores are available (8
cores on CCRL, 52 on TCEC and up to 256 on CCCC).

Other design choices are deliberate focus on performance over most things
(except simplicity and clarity). This includes graceful error recovery (it
should be minimal: the engine shouldn't crash if it's possible to reject the
malformed inputs and continue working) and support for arcane environments.

### Recipes

Most commands for development, building the engine, testing, checking for errors
and fuzzing it are supported as [just](https://github.com/casey/just) recipes.
See [justfile](/justfile) for a complete list of frequently used commands.

### Code map

Rustdoc developer documentation is pushed at each commit to
<https://kirillbobyrev.github.io/pabi/docs/pabi/>.

##### [`src/chess/`](/src/chess/)

Contains implementation of the chess environment and rules: [Bitboard]-based
board representation, move generation, [Zobrist hashing]. This is the core of
the engine: a fast move generator and convenient board implementation are
crucial for engine's performance.

##### [`src/evaluation/`](/src/evaluation/)

Contains code that extracts features from a given position and runs "static"
[position evaluation]: a Neural Network that considers just a single position
and predicts how good the position is for the player to move.

##### [`src/search/`](/src/search/)

Implements Monte Carlo Tree Search ([MCTS]) and its extensions.

##### [`src/engine/`](/src/engine/)

Assembles all pieces together and manages resources to search effieciently under
given time constraints. It also communicates back and forth with the tournament
manager/server through [Universal Chess Interface] (UCI) protocol
implementation.

#### [`generated/`](/generated/)

Pre-computed constants (such as [Magic Bitboards], [Vector Attacks]) speed up
move generation and search.

#### [`tests/`](/tests/)

Tests the engine through public interfaces. Most tests should go here, unit
tests are only valuable for testing private functions that aren't exposed but
are still not trivial.

#### [`benches/`](/benches/)

Performance is crucial for a chess engine. This directory contains a number of
performance regression tests that should be frequently run to ensure that the
engine is not becoming slower. Patches affecting performance should have
benchmark result deltas in the description.

#### [`fuzz/`](/fuzz/)

[Fuzzers] complement the existing tests by generating random inputs and trying
to increase the coverage. Plenty of bugs can be caught by even relatively simply
fuzzers: writing and running them is highly encouraged.

## [Milestones]

- [ ] [Proof of Concept]
- [ ] [Stable]
- [ ] [Strong]

[Bitboard]: https://www.chessprogramming.org/Bitboards
[CCRL rules]: https://computerchess.org.uk/ccrl/404/about.html
[Computer Chess Championship]: https://www.chess.com/computer-chess-championship
[Dependabot]: https://github.com/dependabot
[Fuzzers]: https://en.wikipedia.org/wiki/Fuzzing
[Fuzzing]: https://en.wikipedia.org/wiki/Fuzzing
[GitHub Actions]: https://github.com/features/actions
[KataGo]: https://github.com/lightvector/KataGo
[MCTS]: https://en.wikipedia.org/wiki/Monte_Carlo_tree_search
[Magic Bitboards]: https://www.chessprogramming.org/Magic_Bitboards
[Milestones]: https://github.com/kirillbobyrev/pabi/milestones
[Proof of Concept]: https://github.com/kirillbobyrev/pabi/milestone/1
[Rust]: https://www.rust-lang.org/
[Stable]: https://github.com/kirillbobyrev/pabi/milestone/2
[Strong]: https://github.com/kirillbobyrev/pabi/milestone/3
[TCEC]: https://tcec-chess.com/
[Universal Chess Interface]: http://wbec-ridderkerk.nl/html/UCIProtocol.html
[Zobrist hashing]: https://www.chessprogramming.org/Zobrist_Hashing
[lc0]: https://lczero.org/
[position evaluation]: https://www.chessprogramming.org/Evaluation
[rustdoc]: https://doc.rust-lang.org/rustdoc
[vector attacks]: https://www.chessprogramming.org/Vector_Attacks
